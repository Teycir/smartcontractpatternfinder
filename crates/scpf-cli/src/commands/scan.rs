use crate::cli::ScanArgs;
use anyhow::Result;
use futures::stream::{self, StreamExt};
use scpf_core::{Cache, ContractFetcher, Scanner, TemplateLoader};
use scpf_types::{ApiKeyConfig, ScanResult};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

pub async fn run(args: ScanArgs) -> Result<()> {
    let templates_dir = args.templates.unwrap_or_else(|| PathBuf::from("templates"));
    let templates = TemplateLoader::load_from_dir(&templates_dir).await?;

    if templates.is_empty() {
        anyhow::bail!("No templates found in {:?}", templates_dir);
    }

    let scanner = Arc::new(Scanner::new(templates)?);
    let api_keys = ApiKeyConfig::from_env();
    let fetcher = Arc::new(ContractFetcher::new(api_keys)?);

    let cache_dir = dirs::cache_dir()
        .map(|d| d.join("scpf"))
        .unwrap_or_else(|| PathBuf::from(".cache"));
    let cache = Arc::new(Cache::new(cache_dir).await?);
    let chain = args.chain;

    let results = stream::iter(args.addresses.iter())
        .map(|address| {
            let scanner = Arc::clone(&scanner);
            let fetcher = Arc::clone(&fetcher);
            let cache = Arc::clone(&cache);
            let address = address.clone();

            async move {
                let start = Instant::now();
                println!("Scanning {}...", address);

                let cache_key = format!("{}:{}", chain, address);
                let source = if let Some(cached) = cache.get(&cache_key).await {
                    cached
                } else {
                    let src = fetcher.fetch_source(&address, chain).await?;
                    cache.set(&cache_key, &src).await?;
                    src
                };

                let matches = scanner.scan(&source, PathBuf::from(&address))?;
                let scan_time_ms = start.elapsed().as_millis() as u64;

                Ok::<_, anyhow::Error>(ScanResult {
                    address,
                    chain: chain.to_string(),
                    matches,
                    scan_time_ms,
                })
            }
        })
        .buffer_unordered(args.concurrency)
        .collect::<Vec<_>>()
        .await;

    let scan_results: Vec<ScanResult> = results
        .into_iter()
        .filter_map(|r| match r {
            Ok(result) => Some(result),
            Err(e) => {
                eprintln!("Error: {}", e);
                None
            }
        })
        .collect();

    match args.output {
        crate::cli::OutputFormat::Json => print_json(&scan_results)?,
        crate::cli::OutputFormat::Sarif => print_sarif(&scan_results)?,
        crate::cli::OutputFormat::Console => print_console(&scan_results),
    }

    Ok(())
}

fn print_console(results: &[ScanResult]) {
    for result in results {
        println!(
            "\n{}: Found {} matches ({}ms)",
            result.address,
            result.matches.len(),
            result.scan_time_ms
        );
        for m in &result.matches {
            println!("  [{}:{}] {}", m.line_number, m.column, m.message);
        }
    }
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
