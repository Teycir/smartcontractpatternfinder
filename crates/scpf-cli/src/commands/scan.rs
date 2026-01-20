use crate::cli::ScanArgs;
use crate::output;
use anyhow::Result;
use colored::Colorize;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use scpf_core::{Cache, ContractFetcher, Scanner, TemplateLoader};
use scpf_types::{ApiKeyConfig, ScanResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use walkdir::WalkDir;

static TEMPLATE_CACHE: Mutex<Option<Vec<scpf_types::Template>>> = Mutex::new(None);

pub async fn run(args: ScanArgs) -> Result<()> {
    // Update templates if requested
    if let Some(days) = args.update_templates {
        if days > 0 {
            println!("{}  Updating templates with 0-day exploits from last {} days...", "📡".cyan(), days);
            let zeroday_fetcher = scpf_core::ZeroDayFetcher::new()?;
            let exploits = zeroday_fetcher.fetch_recent_exploits(days).await?;
            if !exploits.is_empty() {
                let zeroday_path = PathBuf::from("templates/zero_day_live.yaml");
                zeroday_fetcher.generate_template(exploits.clone(), &zeroday_path).await?;
                println!("   ✓ Updated with {} recent exploits\n", exploits.len());
            } else {
                println!("   ℹ No new exploits found\n");
            }
        }
    }

    // If no addresses provided and not local scan, fetch recent contracts
    if args.addresses.is_empty() && args.diff.is_none() {
        let should_scan_local = std::path::Path::new(".").join("contracts").exists()
            || std::path::Path::new(".").join("src").exists()
            || std::fs::read_dir(".")
                .ok()
                .and_then(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .find(|e| e.path().extension().and_then(|s| s.to_str()) == Some("sol"))
                })
                .is_some();

        if should_scan_local {
            return scan_local_project(args).await;
        } else {
            // Fetch recent contracts from blockchain
            return scan_recent_contracts(args).await;
        }
    }

    if args.addresses.is_empty() {
        return scan_local_project(args).await;
    }

    let templates = load_templates(&args.templates).await?;
    let scanner = Arc::new(tokio::sync::Mutex::new(Scanner::new(templates)?));
    let api_keys = ApiKeyConfig::from_env();
    let fetcher = Arc::new(ContractFetcher::new(api_keys)?);

    let cache_dir = dirs::cache_dir()
        .map(|d| d.join("scpf"))
        .unwrap_or_else(|| PathBuf::from(".cache"));
    let cache = Arc::new(Cache::new(cache_dir).await?);
    let chain = args.chain;

    let pb = create_progress_bar(args.addresses.len() as u64);

    let results = stream::iter(args.addresses.iter())
        .map(|address| {
            let scanner = Arc::clone(&scanner);
            let fetcher = Arc::clone(&fetcher);
            let cache = Arc::clone(&cache);
            let address = address.clone();
            let pb = pb.clone();

            async move {
                let start = Instant::now();
                pb.set_message(format!("Scanning {}", &address[..10]));

                let cache_key = format!("{}:{}", chain, address);
                let source = if let Some(cached) = cache.get(&cache_key).await {
                    cached
                } else {
                    match fetcher.fetch_source(&address, chain).await {
                        Ok(src) => {
                            cache.set(&cache_key, &src).await?;
                            src
                        }
                        Err(e) => {
                            pb.inc(1);
                            let dummy_result = ScanResult {
                                address: address.clone(),
                                chain: chain.to_string(),
                                matches: Vec::new(),
                                scan_time_ms: 0,
                            };
                            return Ok::<_, anyhow::Error>((dummy_result, Err(e)));
                        }
                    }
                };

                let matches = scanner
                    .lock()
                    .await
                    .scan(&source, PathBuf::from(&address))?;
                let scan_time_ms = start.elapsed().as_millis() as u64;
                pb.inc(1);

                Ok((
                    ScanResult {
                        address,
                        chain: chain.to_string(),
                        matches,
                        scan_time_ms,
                    },
                    Ok(()),
                ))
            }
        })
        .buffer_unordered(args.concurrency)
        .collect::<Vec<_>>()
        .await;

    pb.finish_with_message("Scan complete");
    println!();

    let mut scan_results = Vec::new();
    let mut failed = 0;

    for result in results {
        match result {
            Ok((scan_result, Ok(()))) => {
                scan_results.push(scan_result);
            }
            Ok((scan_result, Err(e))) => {
                failed += 1;
                eprintln!("{}  {} - Failed: {}", "✗".red(), scan_result.address, e);
            }
            Err(e) => {
                failed += 1;
                eprintln!("{}  Error: {}", "✗".red(), e);
            }
        }
    }

    match args.output {
        crate::cli::OutputFormat::Json => println!("{}", output::format_json(&scan_results)?),
        crate::cli::OutputFormat::Sarif => println!("{}", output::format_sarif(&scan_results)?),
        crate::cli::OutputFormat::Console => print_console(&scan_results, failed),
    }

    Ok(())
}

fn print_console(results: &[ScanResult], failed: usize) {
    if results.is_empty() && failed == 0 {
        return;
    }

    println!("\n{}", "═".repeat(60).cyan());

    // Group by file/address
    let mut by_address: std::collections::HashMap<String, Vec<&scpf_types::Match>> =
        std::collections::HashMap::new();
    for result in results {
        for m in &result.matches {
            by_address
                .entry(result.address.clone())
                .or_default()
                .push(m);
        }
    }

    let mut total_matches = 0;

    for result in results {
        total_matches += result.matches.len();

        if result.matches.is_empty() {
            println!(
                "{}  {} ({}ms)",
                "✓".green(),
                result.address,
                result.scan_time_ms
            );
            println!("   No issues found");
        } else {
            println!(
                "{}  {} ({}ms)",
                "!".yellow(),
                result.address,
                result.scan_time_ms
            );

            for m in result.matches.iter().take(5) {
                let severity_str = match m.severity {
                    scpf_types::Severity::Critical => "CRITICAL".red().bold(),
                    scpf_types::Severity::High => "HIGH".red(),
                    scpf_types::Severity::Medium => "MEDIUM".yellow(),
                    scpf_types::Severity::Low => "LOW".blue(),
                    scpf_types::Severity::Info => "INFO".cyan(),
                };
                println!(
                    "   [{}] Line {}: {}",
                    severity_str, m.line_number, m.message
                );
            }

            if result.matches.len() > 5 {
                println!(
                    "   {} and {} more issues",
                    "...".dimmed(),
                    result.matches.len() - 5
                );
            }
        }
        println!();
    }

    println!("{}", "─".repeat(60).cyan());
    println!("{}  Summary:", "📊".cyan());

    // Count by severity
    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;
    let mut info = 0;

    for result in results {
        for m in &result.matches {
            match m.severity {
                scpf_types::Severity::Critical => critical += 1,
                scpf_types::Severity::High => high += 1,
                scpf_types::Severity::Medium => medium += 1,
                scpf_types::Severity::Low => low += 1,
                scpf_types::Severity::Info => info += 1,
            }
        }
    }

    println!("   Scanned: {} | Failed: {}", results.len(), failed);
    if total_matches > 0 {
        print!("   Severity: ");
        let mut parts = Vec::new();
        if critical > 0 {
            parts.push(format!("{} {}", "CRITICAL:".red().bold(), critical));
        }
        if high > 0 {
            parts.push(format!("{} {}", "HIGH:".red(), high));
        }
        if medium > 0 {
            parts.push(format!("{} {}", "MEDIUM:".yellow(), medium));
        }
        if low > 0 {
            parts.push(format!("{} {}", "LOW:".blue(), low));
        }
        if info > 0 {
            parts.push(format!("{} {}", "INFO:".cyan(), info));
        }
        println!("{}", parts.join(" | "));
    }
    println!("   Total issues: {}", total_matches);

    // Display risk scores
    if !results.is_empty() {
        let total_risk: u32 = results.iter().map(|r| r.total_risk_score()).sum();
        let avg_risk = total_risk / results.len() as u32;
        let max_risk = results
            .iter()
            .map(|r| r.total_risk_score())
            .max()
            .unwrap_or(0);
        println!(
            "   Risk Score: {} (avg: {}, max: {})",
            total_risk, avg_risk, max_risk
        );
    }

    if total_matches == 0 && failed == 0 {
        println!(
            "\n{} {}",
            "✓".green().bold(),
            "No issues found! Your contracts look good.".green()
        );
    } else if total_matches > 0 {
        println!("\n{} Next steps:", "→".cyan().bold());
        if critical > 0 || high > 0 {
            println!(
                "  • {} Fix CRITICAL and HIGH severity issues first",
                "⚠️".red()
            );
        }
        println!("  • Export to JSON: scpf scan ... --output json > results.json");
        println!("  • Export to SARIF: scpf scan ... --output sarif");
        println!("  • More info: https://github.com/Teycir/smartcontractpatternfinder");
    }

    println!("{}", "═".repeat(60).cyan());
}

async fn scan_local_project(args: ScanArgs) -> Result<()> {
    println!("{}  Scanning local project...", "🔍".cyan());

    let sol_files = if let Some(ref diff_spec) = args.diff {
        discover_diff_files(diff_spec)?
    } else {
        discover_solidity_files(&args)?
    };

    if sol_files.is_empty() {
        if args.diff.is_some() {
            println!("{}  No .sol files changed in diff", "✓".green());
            return Ok(());
        }
        anyhow::bail!("No .sol files found. Use 'scpf scan 0x...' for blockchain scanning.");
    }

    println!("{}  Found {} Solidity files", "✓".green(), sol_files.len());

    let templates = load_templates(&args.templates).await?;
    let mut scanner = Scanner::new(templates)?;
    let pb = create_progress_bar(sol_files.len() as u64);

    let mut scan_results = Vec::new();
    for file_path in sol_files {
        let start = Instant::now();
        pb.set_message(format!("Scanning {}", file_path.display()));

        let source = tokio::fs::read_to_string(&file_path).await?;
        let matches = scanner.scan(&source, file_path.clone())?;
        let scan_time_ms = start.elapsed().as_millis() as u64;

        scan_results.push(ScanResult {
            address: file_path.display().to_string(),
            chain: "local".to_string(),
            matches,
            scan_time_ms,
        });
        pb.inc(1);
    }

    pb.finish_with_message("Scan complete");
    println!();

    match args.output {
        crate::cli::OutputFormat::Json => println!("{}", output::format_json(&scan_results)?),
        crate::cli::OutputFormat::Sarif => println!("{}", output::format_sarif(&scan_results)?),
        crate::cli::OutputFormat::Console => print_console(&scan_results, 0),
    }

    // Exit with error code if high/critical issues found
    let has_critical = scan_results.iter().any(|r| {
        r.matches.iter().any(|m| {
            matches!(
                m.severity,
                scpf_types::Severity::Critical | scpf_types::Severity::High
            )
        })
    });
    if has_critical {
        std::process::exit(1);
    }

    Ok(())
}

fn discover_solidity_files(_args: &ScanArgs) -> Result<Vec<PathBuf>> {
    let search_paths = vec![".", "contracts", "src"];
    let mut sol_files = Vec::new();

    for base_path in search_paths {
        if !std::path::Path::new(base_path).exists() {
            continue;
        }

        for entry in WalkDir::new(base_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("sol") {
                // Skip node_modules, build artifacts
                let path_str = path.to_string_lossy();
                if path_str.contains("node_modules")
                    || path_str.contains("/build/")
                    || path_str.contains("/out/")
                    || path_str.contains("/artifacts/")
                {
                    continue;
                }
                sol_files.push(path.to_path_buf());
            }
        }
    }

    sol_files.sort();
    sol_files.dedup();
    Ok(sol_files)
}

async fn load_templates(templates_path: &Option<PathBuf>) -> Result<Vec<scpf_types::Template>> {
    let templates_dir = templates_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("templates"));

    if let Ok(cache) = TEMPLATE_CACHE.lock() {
        if let Some(cached) = cache.as_ref() {
            return Ok(cached.clone());
        }
    }

    let templates = TemplateLoader::load_from_dir(&templates_dir).await?;

    if templates.is_empty() {
        anyhow::bail!("No templates found in {:?}", templates_dir);
    }

    println!("{}  Loaded {} templates", "✓".green(), templates.len());

    if let Ok(mut cache) = TEMPLATE_CACHE.lock() {
        *cache = Some(templates.clone());
    }

    Ok(templates)
}

fn create_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}

fn discover_diff_files(diff_spec: &str) -> Result<Vec<PathBuf>> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["diff", "--name-only", diff_spec])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let files = String::from_utf8(output.stdout)?;
    let sol_files: Vec<PathBuf> = files
        .lines()
        .filter(|line| line.ends_with(".sol"))
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect();

    Ok(sol_files)
}

async fn scan_recent_contracts(args: ScanArgs) -> Result<()> {
    println!("{}  Scanning contracts updated in last {} days...", "🔍".cyan(), args.days);
    println!("   Severity filter: {} and above", args.min_severity.to_uppercase());
    
    let api_keys = ApiKeyConfig::from_env();
    let fetcher = Arc::new(ContractFetcher::new(api_keys)?);
    
    let chains = if args.all_chains {
        vec![
            scpf_types::Chain::Ethereum,
            scpf_types::Chain::Bsc,
            scpf_types::Chain::Polygon,
            scpf_types::Chain::Arbitrum,
            scpf_types::Chain::Optimism,
            scpf_types::Chain::Base,
        ]
    } else {
        vec![args.chain]
    };

    let mut all_contracts = Vec::new();
    for chain in &chains {
        println!("{}  Fetching from {}...", "📡".cyan(), chain.as_str());
        match fetcher.fetch_recent_contracts(*chain, args.days).await {
            Ok(addresses) => {
                println!("   ✓ Found {} contracts", addresses.len());
                for addr in addresses.into_iter().take(10) {
                    all_contracts.push((addr, *chain));
                }
            }
            Err(e) => {
                println!("   ✗ Failed: {}", e);
            }
        }
    }

    if all_contracts.is_empty() {
        println!("{}  No recent contracts found", "⚠️".yellow());
        return Ok(());
    }

    println!();
    println!("{}  Scanning {} contracts...", "🔎".cyan(), all_contracts.len());
    println!();
    
    let templates = load_templates(&args.templates).await?;
    let scanner = Arc::new(tokio::sync::Mutex::new(Scanner::new(templates)?));
    let cache_dir = dirs::cache_dir()
        .map(|d| d.join("scpf"))
        .unwrap_or_else(|| PathBuf::from(".cache"));
    let cache = Arc::new(Cache::new(cache_dir).await?);

    let min_severity = parse_severity(&args.min_severity);
    let mut scan_results = Vec::new();

    for (address, chain) in all_contracts {
        let cache_key = format!("{}:{}", chain, address);
        let source = if let Some(cached) = cache.get(&cache_key).await {
            cached
        } else {
            match fetcher.fetch_source(&address, chain).await {
                Ok(src) => {
                    cache.set(&cache_key, &src).await?;
                    src
                }
                Err(e) => {
                    println!("{}  {} - Failed: {}", "✗".red(), &address[..10], e);
                    continue;
                }
            }
        };
        
        let start = Instant::now();
        let matches = scanner.lock().await.scan(&source, PathBuf::from(&address))?;
        let scan_time_ms = start.elapsed().as_millis() as u64;
        
        let filtered_matches: Vec<_> = matches
            .into_iter()
            .filter(|m| m.severity >= min_severity)
            .collect();
        
        if !filtered_matches.is_empty() {
            println!("{}  {} ({}) - {} issues", "⚠️".yellow(), &address[..10], chain.as_str(), filtered_matches.len());
            scan_results.push(ScanResult {
                address,
                chain: chain.to_string(),
                matches: filtered_matches,
                scan_time_ms,
            });
        } else {
            println!("{}  {} ({}) - Clean", "✓".green(), &address[..10], chain.as_str());
        }
    }
    
    println!();
    match args.output {
        crate::cli::OutputFormat::Json => println!("{}", output::format_json(&scan_results)?),
        crate::cli::OutputFormat::Sarif => println!("{}", output::format_sarif(&scan_results)?),
        crate::cli::OutputFormat::Console => print_console(&scan_results, 0),
    }
    
    Ok(())
}

fn parse_severity(s: &str) -> scpf_types::Severity {
    match s.to_lowercase().as_str() {
        "critical" => scpf_types::Severity::Critical,
        "high" => scpf_types::Severity::High,
        "medium" => scpf_types::Severity::Medium,
        "low" => scpf_types::Severity::Low,
        _ => scpf_types::Severity::Info,
    }
}
