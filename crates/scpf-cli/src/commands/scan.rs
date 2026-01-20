use crate::cli::ScanArgs;
use anyhow::Result;
use colored::Colorize;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use scpf_core::{Cache, ContractFetcher, Scanner, TemplateLoader};
use scpf_types::{ApiKeyConfig, ScanResult};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

pub async fn run(args: ScanArgs) -> Result<()> {
    // Auto-detect if no addresses provided
    if args.addresses.is_empty() {
        println!("{}  No addresses provided, checking for local contracts...", "🔍".cyan());
        
        for path in ["contracts", "src/contracts", "src"] {
            if std::path::Path::new(path).exists() {
                println!("{}  Found contracts directory: {}", "✓".green(), path);
                println!("{}  Note: Auto-detect only works with local files. For blockchain scanning, provide addresses.", "!".yellow());
                anyhow::bail!("Auto-detect for local files not yet implemented. Please provide contract addresses.");
            }
        }
        
        anyhow::bail!("No contract addresses provided and no local contracts found.\n\n{}\n   1. Provide addresses: scpf scan 0x...\n   2. Or create contracts/ directory with Solidity files", "Fix:".yellow().bold());
    }
    
    let templates_dir = args.templates.unwrap_or_else(|| PathBuf::from("templates"));
    let templates = TemplateLoader::load_from_dir(&templates_dir).await?;

    if templates.is_empty() {
        anyhow::bail!("No templates found in {:?}", templates_dir);
    }

    println!("{}  Loaded {} templates", "✓".green(), templates.len());
    
    if !args.no_cache {
        println!("{}  Cache enabled", "📦".cyan());
    }

    let scanner = Arc::new(tokio::sync::Mutex::new(Scanner::new(templates)?));
    let api_keys = ApiKeyConfig::from_env();
    let fetcher = Arc::new(ContractFetcher::new(api_keys)?);

    let cache_dir = dirs::cache_dir()
        .map(|d| d.join("scpf"))
        .unwrap_or_else(|| PathBuf::from(".cache"));
    let cache = Arc::new(Cache::new(cache_dir).await?);
    let chain = args.chain;

    let pb = ProgressBar::new(args.addresses.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

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

                let matches = scanner.lock().await.scan(&source, PathBuf::from(&address))?;
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
        crate::cli::OutputFormat::Json => print_json(&scan_results)?,
        crate::cli::OutputFormat::Sarif => print_sarif(&scan_results)?,
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

fn print_json(results: &[ScanResult]) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    println!("{}", json);
    Ok(())
}

fn print_sarif(results: &[ScanResult]) -> Result<()> {
    let mut runs = Vec::new();

    for result in results {
        let mut sarif_results = Vec::new();

        for m in &result.matches {
            sarif_results.push(serde_json::json!({
                "ruleId": m.template_id,
                "level": match m.severity {
                    scpf_types::Severity::Critical | scpf_types::Severity::High => "error",
                    scpf_types::Severity::Medium => "warning",
                    _ => "note",
                },
                "message": {
                    "text": m.message
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": result.address
                        },
                        "region": {
                            "startLine": m.line_number,
                            "startColumn": m.column
                        }
                    }
                }]
            }));
        }

        runs.push(serde_json::json!({
            "tool": {
                "driver": {
                    "name": "SCPF",
                    "version": "0.1.0"
                }
            },
            "results": sarif_results
        }));
    }

    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": runs
    });

    println!("{}", serde_json::to_string_pretty(&sarif)?);
    Ok(())
}
