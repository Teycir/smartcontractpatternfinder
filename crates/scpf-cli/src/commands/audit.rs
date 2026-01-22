use crate::cli::ScanArgs;
use anyhow::Result;
use colored::Colorize;
use scpf_core::{Cache, ContractFetcher, Scanner, TemplateLoader, ZeroDayFetcher};
use scpf_types::{Chain, ScanResult, Severity, Template};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

pub async fn run_full_audit(_addresses: Vec<String>, args: ScanArgs) -> Result<()> {
    println!("{}", "🔍 SCPF Full Security Audit".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();

    // Create timestamped output directory
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let output_dir = PathBuf::from(format!("audit_results/{}", timestamp));
    fs::create_dir_all(&output_dir).await?;
    println!("📁 Output: {}", output_dir.display());
    println!();

    // Step 1: Fetch 7-day 0-day exploits and generate template
    println!("{} Fetching 7-day 0-day exploits...", "📡".cyan());
    let zeroday_fetcher = ZeroDayFetcher::new()?;
    let exploits = zeroday_fetcher.fetch_recent_exploits(7).await?;

    let zeroday_template_path = output_dir.join("zeroday_template.yaml");
    let zeroday_templates = if !exploits.is_empty() {
        zeroday_fetcher
            .generate_template(exploits.clone(), &zeroday_template_path)
            .await?;
        println!(
            "   ✓ Generated 0-day template from {} exploits",
            exploits.len()
        );
        vec![TemplateLoader::load_from_dir(&output_dir)
            .await?
            .into_iter()
            .next()
            .unwrap()]
    } else {
        println!("   ℹ No 0-day exploits found");
        vec![]
    };
    println!();

    // Step 2: Load static templates
    println!("{} Loading static templates...", "📚".cyan());
    let templates_dir = args
        .templates
        .clone()
        .unwrap_or_else(|| PathBuf::from("templates"));
    let static_templates: Vec<Template> = TemplateLoader::load_from_dir(&templates_dir)
        .await?
        .into_iter()
        .filter(|t| matches!(t.severity, Severity::Critical | Severity::High))
        .collect();
    println!("   ✓ Loaded {} static templates", static_templates.len());
    println!();

    // Step 3: Fetch 7-day contracts from all chains
    println!("{} Fetching 7-day contracts (all chains)...", "🌐".cyan());
    let api_keys = crate::keys::load_api_keys_from_env();
    let fetcher = Arc::new(ContractFetcher::new(api_keys)?);

    let chains = vec![
        Chain::Ethereum,
        Chain::Bsc,
        Chain::Polygon,
        Chain::Arbitrum,
        Chain::Optimism,
        Chain::Base,
        Chain::Avalanche,
        Chain::Fantom,
        Chain::ZkSync,
        Chain::Linea,
        Chain::Scroll,
        Chain::Zora,
    ];
    let mut all_addresses = Vec::new();

    for chain in &chains {
        print!("   {}... ", chain.as_str());
        match fetcher.fetch_recent_contracts(*chain, 7).await {
            Ok(addrs) => {
                let count = addrs.len().min(20);
                all_addresses.extend(addrs.into_iter().take(20).map(|a| (a, *chain)));
                println!("{} {} contracts", "✓".green(), count);
            }
            Err(e) => println!("{} {}", "✗".red(), e),
        }
    }
    println!("   Total: {} contracts", all_addresses.len());
    println!();

    // Step 4: Fetch contract sources (shared for both scans)
    println!("{} Fetching contract sources...", "📥".cyan());
    let cache_dir = dirs::cache_dir()
        .map(|d| d.join("scpf"))
        .unwrap_or_else(|| PathBuf::from(".cache"));
    let cache = Arc::new(Cache::new(cache_dir).await?);

    let mut sources = Vec::new();
    for (idx, (address, chain)) in all_addresses.iter().enumerate() {
        print!(
            "   [{}/{}] {}... ",
            idx + 1,
            all_addresses.len(),
            &address[..10]
        );

        let cache_key = format!("{}:{}", chain, address);
        let source = if let Some(cached) = cache.get(&cache_key).await {
            cached
        } else {
            match fetcher.fetch_source(address, *chain).await {
                Ok(src) => {
                    cache.set(&cache_key, &src).await?;
                    src
                }
                Err(e) => {
                    println!("{} {}", "✗".red(), e);
                    continue;
                }
            }
        };

        sources.push((address.clone(), chain.to_string(), source));
        println!("{}", "✓".green());
    }
    println!();

    // Step 5: Parallel scanning (0-day + static)
    println!("{} Running parallel scans...", "🔎".cyan());

    let sources_arc = Arc::new(sources);
    let sources_clone = Arc::clone(&sources_arc);

    let zeroday_handle =
        tokio::spawn(
            async move { scan_with_templates(zeroday_templates, sources_arc, "0-day").await },
        );

    let static_handle = tokio::spawn(async move {
        scan_with_templates(static_templates, sources_clone, "static").await
    });

    let (zeroday_results, static_results) = tokio::try_join!(zeroday_handle, static_handle)?;
    let zeroday_results = zeroday_results?;
    let static_results = static_results?;

    println!(
        "   ✓ 0-day scan: {} findings",
        count_findings(&zeroday_results)
    );
    println!(
        "   ✓ Static scan: {} findings",
        count_findings(&static_results)
    );
    println!();

    // Step 6: Write 4 output files
    println!("{} Writing output files...", "💾".cyan());

    write_json(&output_dir.join("zeroday_results.json"), &zeroday_results).await?;
    write_txt(&output_dir.join("zeroday_results.txt"), &zeroday_results).await?;
    write_json(&output_dir.join("static_results.json"), &static_results).await?;
    write_txt(&output_dir.join("static_results.txt"), &static_results).await?;

    println!("   ✓ zeroday_results.json");
    println!("   ✓ zeroday_results.txt");
    println!("   ✓ static_results.json");
    println!("   ✓ static_results.txt");
    println!();

    // Step 7: Summary report
    generate_summary(&zeroday_results, &static_results)?;

    Ok(())
}

async fn scan_with_templates(
    templates: Vec<Template>,
    sources: Arc<Vec<(String, String, String)>>,
    _scan_type: &str,
) -> Result<Vec<ScanResult>> {
    if templates.is_empty() {
        return Ok(vec![]);
    }

    let mut scanner = Scanner::new(templates)?;
    let mut results = Vec::new();

    for (address, chain, source) in sources.iter() {
        let matches = scanner.scan(source, PathBuf::from(address))?;

        if !matches.is_empty() {
            results.push(ScanResult {
                address: address.clone(),
                chain: chain.clone(),
                matches,
                scan_time_ms: 0,
                solidity_version: None,
            });
        }
    }

    Ok(results)
}

fn count_findings(results: &[ScanResult]) -> usize {
    results.iter().map(|r| r.matches.len()).sum()
}

async fn write_json(path: &PathBuf, results: &[ScanResult]) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    fs::write(path, json).await?;
    Ok(())
}

async fn write_txt(path: &PathBuf, results: &[ScanResult]) -> Result<()> {
    let mut output = String::new();

    for result in results {
        output.push_str(&format!("\n{} ({})", result.address, result.chain));
        output.push_str(&format!("\n{}", "─".repeat(60)));

        for m in &result.matches {
            output.push_str(&format!(
                "\n  [{:?}] {} (Line {})",
                m.severity, m.template_id, m.line_number
            ));
            output.push_str(&format!("\n  Pattern: {}", m.pattern_id));
            output.push_str(&format!("\n  Message: {}", m.message));
            output.push_str("\n");
        }
    }

    fs::write(path, output).await?;
    Ok(())
}

fn generate_summary(zeroday_results: &[ScanResult], static_results: &[ScanResult]) -> Result<()> {
    let zeroday_critical = count_by_severity(zeroday_results, Severity::Critical);
    let zeroday_high = count_by_severity(zeroday_results, Severity::High);
    let static_critical = count_by_severity(static_results, Severity::Critical);
    let static_high = count_by_severity(static_results, Severity::High);

    println!("{}", "═".repeat(60).cyan());
    println!("{}", "📋 AUDIT SUMMARY".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    println!("0-Day Template Scan:");
    println!("  🔴 CRITICAL: {}", zeroday_critical);
    println!("  🟠 HIGH:     {}", zeroday_high);
    println!();
    println!("Static Template Scan:");
    println!("  🔴 CRITICAL: {}", static_critical);
    println!("  🟠 HIGH:     {}", static_high);
    println!();
    println!(
        "Total Findings: {}",
        zeroday_critical + zeroday_high + static_critical + static_high
    );
    println!("{}", "═".repeat(60).cyan());

    Ok(())
}

fn count_by_severity(results: &[ScanResult], severity: Severity) -> usize {
    results
        .iter()
        .flat_map(|r| &r.matches)
        .filter(|m| m.severity == severity)
        .count()
}
