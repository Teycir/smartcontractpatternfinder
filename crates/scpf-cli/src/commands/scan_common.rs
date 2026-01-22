use anyhow::Result;
use scpf_core::{Cache, ContractFetcher, Scanner};
use scpf_types::{Chain, ScanResult, Severity, Template};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

pub fn get_supported_chains() -> Vec<Chain> {
    vec![Chain::Ethereum, Chain::Polygon, Chain::Arbitrum]
}

pub async fn fetch_contracts(
    fetcher: &ContractFetcher,
    chains: &[Chain],
    days: u64,
) -> Vec<(String, Chain)> {
    let mut all_contracts = Vec::new();
    for chain in chains {
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
    all_contracts
}

pub async fn scan_contracts(
    contracts: Vec<(String, Chain)>,
    templates: Vec<Template>,
    fetcher: Arc<ContractFetcher>,
    min_severity: Severity,
) -> Result<Vec<ScanResult>> {
    let scanner = Arc::new(tokio::sync::Mutex::new(Scanner::new(templates)?));
    let cache_dir = dirs::cache_dir()
        .map(|d| d.join("scpf"))
        .unwrap_or_else(|| PathBuf::from(".cache"));
    let cache = Arc::new(Cache::new(cache_dir).await?);

    let mut all_scan_results = Vec::new();

    for (address, chain) in contracts {
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
            .filter(|m| m.severity >= min_severity && m.severity >= Severity::High)
            .collect();

        let analyzed_matches: Vec<_> = filtered_matches
            .into_iter()
            .map(|m| {
                let analysis = scpf_core::analyze_exploitability(&m);
                (m, analysis)
            })
            .collect();

        let exploitable_count = analyzed_matches
            .iter()
            .filter(|(_, a)| a.is_exploitable)
            .count();

        if exploitable_count > 0 {
            eprintln!(
                "🚨 {} ({}) - {} exploitable",
                &address[..10],
                chain.as_str(),
                exploitable_count
            );
        } else if !analyzed_matches.is_empty() {
            eprintln!(
                "⚠️  {} ({}) - {} needs review",
                &address[..10],
                chain.as_str(),
                analyzed_matches.len()
            );
        } else {
            eprintln!("✓ {} ({}) - Clean", &address[..10], chain.as_str());
        }

        all_scan_results.push(ScanResult {
            address,
            chain: chain.to_string(),
            matches: analyzed_matches.into_iter().map(|(m, _)| m).collect(),
            scan_time_ms,
            solidity_version: extract_solidity_version(&source),
        });
    }

    Ok(all_scan_results)
}

pub fn rank_and_score(mut scan_results: Vec<ScanResult>) -> Vec<ScanResult> {
    scan_results.retain(|r| !r.matches.is_empty());
    scan_results.sort_by_key(|b| std::cmp::Reverse(b.total_risk_score()));

    let top_100: Vec<_> = scan_results.into_iter().take(100).collect();
    let mut with_poc_scores: Vec<_> = top_100
        .into_iter()
        .map(|r| {
            let poc_score: f32 = r.matches.iter().map(|m| m.exploitability_score()).sum();
            (r, poc_score)
        })
        .collect();

    with_poc_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    eprintln!("\n🎯 Top 10 by PoC Score:");
    for (i, (r, score)) in with_poc_scores.iter().take(10).enumerate() {
        eprintln!(
            "  {}. {} - PoC: {:.1} (Risk: {})",
            i + 1,
            &r.address[..12],
            score,
            r.total_risk_score()
        );
    }
    eprintln!();

    with_poc_scores.into_iter().map(|(r, _)| r).collect()
}

pub fn parse_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        _ => panic!(
            "Invalid severity: {}. Only 'high' or 'critical' allowed.",
            s
        ),
    }
}

fn extract_solidity_version(source: &str) -> Option<String> {
    let pragma_regex = regex::Regex::new(r"pragma\s+solidity\s+([^;]+);").ok()?;
    pragma_regex
        .captures(source)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}
