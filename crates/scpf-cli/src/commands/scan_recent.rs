use crate::output;
use anyhow::Result;
use scpf_core::{Cache, ContractFetcher, Scanner, TemplateLoader};
use scpf_types::ScanResult;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

pub async fn scan_recent_contracts(
    days: u64,
    min_severity: &str,
    templates_path: &Option<PathBuf>,
) -> Result<()> {
    eprintln!("🔍 Scanning contracts updated in last {} days...", days);
    eprintln!(
        "   Severity filter: {} and above",
        min_severity.to_uppercase()
    );

    let api_keys = crate::keys::load_api_keys_from_env();
    let fetcher = Arc::new(ContractFetcher::new(api_keys)?);

    let chains = vec![
        scpf_types::Chain::Ethereum,
        scpf_types::Chain::Bsc,
        scpf_types::Chain::Polygon,
        scpf_types::Chain::Arbitrum,
        scpf_types::Chain::Optimism,
        scpf_types::Chain::Base,
    ];

    let mut all_contracts = Vec::new();
    for chain in &chains {
        eprintln!("📡 Fetching from {}...", chain.as_str());
        match fetcher.fetch_recent_contracts(*chain, days).await {
            Ok(addresses) => {
                eprintln!("   ✓ Found {} contracts", addresses.len());
                for addr in addresses {
                    all_contracts.push((addr, *chain));
                }
            }
            Err(e) => {
                eprintln!("   ✗ Failed: {}", e);
            }
        }
    }

    if all_contracts.is_empty() {
        eprintln!("⚠️  No recent contracts found");
        return Ok(());
    }

    eprintln!();
    eprintln!("🔎 Scanning {} contracts...", all_contracts.len());
    eprintln!();

    let templates_dir = templates_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("templates"));
    let templates = TemplateLoader::load_from_dir(&templates_dir).await?;
    let scanner = Arc::new(tokio::sync::Mutex::new(Scanner::new(templates)?));

    let cache_dir = dirs::cache_dir()
        .map(|d| d.join("scpf"))
        .unwrap_or_else(|| PathBuf::from(".cache"));
    let cache = Arc::new(Cache::new(cache_dir).await?);

    let min_sev = parse_severity(min_severity);
    let mut all_scan_results = Vec::new();

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
                    eprintln!("✗ {} - Failed: {}", &address[..10], e);
                    continue;
                }
            }
        };

        let start = Instant::now();
        let matches = scanner
            .lock()
            .await
            .scan(&source, PathBuf::from(&address))?;
        let scan_time_ms = start.elapsed().as_millis() as u64;

        let filtered_matches: Vec<_> = matches
            .into_iter()
            .filter(|m| m.severity >= min_sev)
            .collect();

        if !filtered_matches.is_empty() {
            eprintln!(
                "⚠️  {} ({}) - {} issues",
                &address[..10],
                chain.as_str(),
                filtered_matches.len()
            );
        } else {
            eprintln!("✓ {} ({}) - Clean", &address[..10], chain.as_str());
        }

        all_scan_results.push(ScanResult {
            address,
            chain: chain.to_string(),
            matches: filtered_matches,
            scan_time_ms,
            solidity_version: extract_solidity_version(&source),
        });
    }

    let mut scan_results: Vec<_> = all_scan_results
        .into_iter()
        .filter(|r| !r.matches.is_empty())
        .collect();

    scan_results.sort_by(|a, b| b.total_risk_score().cmp(&a.total_risk_score()));
    let top_40: Vec<_> = scan_results.into_iter().take(40).collect();

    let mut with_poc_scores: Vec<_> = top_40
        .into_iter()
        .map(|r| {
            let poc_score: f32 = r.matches.iter().map(|m| m.exploitability_score()).sum();
            (r, poc_score)
        })
        .collect();

    with_poc_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let scan_results: Vec<_> = with_poc_scores
        .into_iter()
        .take(20)
        .map(|(r, _)| r)
        .collect();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let report_dir = dirs::home_dir()
        .map(|h| h.join("smartcontractpatternfinder"))
        .unwrap_or_else(|| PathBuf::from("results"));
    std::fs::create_dir_all(&report_dir)?;

    let output_file = report_dir.join(format!("{}_scan.json", timestamp));
    let json_output = output::format_json(&scan_results)?;
    std::fs::write(&output_file, json_output)?;

    eprintln!();
    eprintln!("📊 Results saved to: {}", output_file.display());
    eprintln!("📋 Found {} vulnerable contracts", scan_results.len());

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

fn extract_solidity_version(source: &str) -> Option<String> {
    let pragma_regex = regex::Regex::new(r"pragma\s+solidity\s+([^;]+);").ok()?;
    pragma_regex
        .captures(source)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}
