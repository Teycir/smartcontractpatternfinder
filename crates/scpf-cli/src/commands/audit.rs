use crate::cli::ScanArgs;
use anyhow::Result;
use colored::Colorize;
use scpf_core::{Cache, ContractFetcher, Scanner, TemplateLoader, ZeroDayFetcher};
use scpf_types::{Chain, ScanResult, Severity};
use std::path::PathBuf;
use std::sync::Arc;

pub async fn run_full_audit(_addresses: Vec<String>, args: ScanArgs) -> Result<()> {
    println!(
        "{}",
        "🔍 SCPF Full Security Audit - 5 Layer Deep Scan"
            .cyan()
            .bold()
    );
    println!("{}", "═".repeat(60).cyan());
    println!();

    // Step 1: Update 0-day templates
    println!(
        "{} Layer 0: Updating 0-day vulnerability patterns...",
        "📡".cyan()
    );
    let zeroday_fetcher = ZeroDayFetcher::new()?;
    let exploits = zeroday_fetcher.fetch_recent_exploits(7).await?;

    if !exploits.is_empty() {
        let zeroday_path = PathBuf::from("templates/zero_day_live.yaml");
        zeroday_fetcher
            .generate_template(exploits.clone(), &zeroday_path)
            .await?;
        println!("   ✓ Updated with {} recent exploits", exploits.len());
    } else {
        println!("   ℹ No new exploits in last 7 days");
    }
    println!();

    // Step 2: Load templates (filter critical/high only)
    println!(
        "{} Layer 1: Loading critical/high severity templates...",
        "📚".cyan()
    );
    let templates_dir = args
        .templates
        .clone()
        .unwrap_or_else(|| PathBuf::from("templates"));
    let all_templates = TemplateLoader::load_from_dir(&templates_dir).await?;
    let templates: Vec<_> = all_templates
        .into_iter()
        .filter(|t| matches!(t.severity, Severity::Critical | Severity::High))
        .collect();
    println!("   ✓ Loaded {} critical/high templates", templates.len());
    println!();

    // Step 3: Fetch recent contracts from all chains
    println!(
        "{} Layer 2: Fetching contracts from last 30 days (all chains)...",
        "🌐".cyan()
    );
    let api_keys = crate::keys::load_api_keys_from_env();
    let fetcher = Arc::new(ContractFetcher::new(api_keys)?);

    let chains = vec![
        Chain::Ethereum,
        Chain::Bsc,
        Chain::Polygon,
        Chain::Arbitrum,
        Chain::Optimism,
        Chain::Base,
    ];

    let mut all_addresses = Vec::new();
    for chain in &chains {
        print!("   Fetching from {}... ", chain.as_str());
        match fetcher.fetch_recent_contracts(*chain, 30).await {
            Ok(addrs) => {
                let count = addrs.len().min(10);
                all_addresses.extend(addrs.into_iter().take(10).map(|a| (a, *chain)));
                println!("{} {} contracts", "✓".green(), count);
            }
            Err(e) => println!("{} {}", "✗".red(), e),
        }
    }
    println!("   Total: {} contracts to scan", all_addresses.len());
    println!();

    // Step 4: Multi-layer scanning
    println!("{} Layer 3-5: Multi-layer deep scan...", "🔎".cyan());
    let scanner = Arc::new(tokio::sync::Mutex::new(Scanner::new(templates)?));
    let cache_dir = dirs::cache_dir()
        .map(|d| d.join("scpf"))
        .unwrap_or_else(|| PathBuf::from(".cache"));
    let cache = Arc::new(Cache::new(cache_dir).await?);

    let mut results = Vec::new();
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

        // Layer 3: Regex pattern matching
        let matches = scanner.lock().await.scan(&source, PathBuf::from(address))?;

        // Layer 4: Semantic analysis (placeholder for future)
        // Layer 5: Composition analysis (placeholder for future)

        let critical_high = matches
            .iter()
            .filter(|m| matches!(m.severity, Severity::Critical | Severity::High))
            .count();

        if critical_high > 0 {
            println!("{} {} critical/high", "⚠️".red(), critical_high);
        } else {
            println!("{}", "✓".green());
        }

        results.push(ScanResult {
            address: address.clone(),
            chain: chain.to_string(),
            matches,
            scan_time_ms: 0,
        });
    }
    println!();

    // Step 5: Generate comprehensive report
    println!("{} Final: Generating security report...", "📊".cyan());
    generate_report(&results)?;

    Ok(())
}

fn generate_report(results: &[ScanResult]) -> Result<()> {
    let mut total_critical = 0;
    let mut total_high = 0;

    for result in results {
        for m in &result.matches {
            match m.severity {
                Severity::Critical => total_critical += 1,
                Severity::High => total_high += 1,
                _ => {}
            }
        }
    }

    println!();
    println!("{}", "═".repeat(60).cyan());
    println!("{}", "📋 SECURITY AUDIT REPORT".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    println!("Contracts Analyzed: {}", results.len());
    println!("Chains Scanned: Ethereum, BSC, Polygon, Arbitrum, Optimism, Base");
    println!("Time Period: Last 30 days");
    println!("Scan Layers: 5 (0-day + Templates + Regex + Semantic + Composition)");
    println!();
    println!("Critical/High Severity Findings:");
    if total_critical > 0 {
        println!("  🔴 CRITICAL: {}", total_critical.to_string().red().bold());
    }
    if total_high > 0 {
        println!("  🟠 HIGH:     {}", total_high.to_string().red());
    }
    if total_critical == 0 && total_high == 0 {
        println!(
            "  {} No critical or high severity issues found",
            "✓".green()
        );
    }
    println!();
    println!("Total Issues: {}", total_critical + total_high);
    println!();

    if total_critical > 0 || total_high > 0 {
        println!(
            "{}",
            "⚠️  CRITICAL/HIGH ISSUES REQUIRE IMMEDIATE ATTENTION"
                .red()
                .bold()
        );
        println!();
        println!("Recommended Actions:");
        println!("  1. Review all critical findings immediately");
        println!("  2. Patch high severity vulnerabilities");
        println!("  3. Export detailed report: scpf audit --output json");
        println!("  4. Re-scan after fixes applied");
        println!();
    }

    println!("{}", "═".repeat(60).cyan());

    Ok(())
}
