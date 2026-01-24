use crate::cli::ScanArgs;
use anyhow::Result;
use scpf_core::{Cache, ContractFetcher, Scanner, TemplateLoader};
use scpf_types::{Chain, ScanResult, Severity, Template};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

struct ExploitabilityStats {
    exploitable: Vec<(usize, scpf_types::Match, scpf_core::ExploitAnalysis)>,
    false_positives: Vec<(usize, scpf_types::Match, scpf_core::ExploitAnalysis)>,
    needs_review: Vec<(usize, scpf_types::Match, scpf_core::ExploitAnalysis)>,
}

fn get_supported_chains() -> Vec<Chain> {
    vec![Chain::Ethereum, Chain::Polygon, Chain::Arbitrum]
}

async fn fetch_contracts(
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

async fn scan_contracts(
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

fn rank_and_score(mut scan_results: Vec<ScanResult>) -> Vec<ScanResult> {
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

fn parse_severity(s: &str) -> Severity {
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

fn categorize_findings(scan_results: &[ScanResult]) -> ExploitabilityStats {
    let mut exploitable = Vec::new();
    let mut false_positives = Vec::new();
    let mut needs_review = Vec::new();

    for (idx, result) in scan_results.iter().enumerate() {
        for m in &result.matches {
            let analysis = scpf_core::analyze_exploitability(m);
            if analysis.is_exploitable {
                exploitable.push((idx, m.clone(), analysis));
            } else if analysis.confidence == scpf_core::ExploitConfidence::High {
                false_positives.push((idx, m.clone(), analysis));
            } else {
                needs_review.push((idx, m.clone(), analysis));
            }
        }
    }

    ExploitabilityStats {
        exploitable,
        false_positives,
        needs_review,
    }
}

pub async fn scan_vulnerabilities(args: ScanArgs) -> Result<()> {
    eprintln!(
        "🔍 Scanning contracts updated in last {} days...",
        args.days
    );
    eprintln!(
        "   Severity filter: {} and above",
        args.min_severity.to_uppercase()
    );

    let api_keys = crate::keys::load_api_keys_from_env();
    let fetcher = Arc::new(ContractFetcher::new(api_keys)?);
    let chains = get_supported_chains();

    let all_contracts = fetch_contracts(&fetcher, &chains, args.days).await;
    if all_contracts.is_empty() {
        eprintln!("⚠️  No recent contracts found");
        return Ok(());
    }

    eprintln!();
    eprintln!("🔎 Scanning {} contracts...", all_contracts.len());
    eprintln!();

    let templates_dir = args
        .templates
        .clone()
        .unwrap_or_else(|| PathBuf::from("templates"));
    let templates = TemplateLoader::load_from_dir(&templates_dir).await?;

    let min_sev = parse_severity(&args.min_severity);
    let all_scan_results = scan_contracts(all_contracts, templates, fetcher, min_sev).await?;
    let scan_results = rank_and_score(all_scan_results);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let stats = categorize_findings(&scan_results);

    let mut exploitable_contracts = std::collections::HashSet::new();
    for (idx, _, _) in &stats.exploitable {
        exploitable_contracts.insert(*idx);
    }

    let exploitable_count = exploitable_contracts.len();

    eprintln!("🌳 Exploitable Contracts:\n");
    eprintln!("📈 Summary:");
    eprintln!("   🚨 Exploitable: {} contracts with {} findings", exploitable_count, stats.exploitable.len());
    eprintln!("   ❌ False Positives: {} findings", stats.false_positives.len());
    eprintln!("   ⚠️  Needs Review: {} findings", stats.needs_review.len());
    eprintln!("   📊 Total: {} findings across {} contracts\n", stats.exploitable.len() + stats.false_positives.len() + stats.needs_review.len(), scan_results.len());

    let root_dir = std::env::var("SCPF_REPORT_DIR")
        .unwrap_or_else(|_| format!("/home/teycir/smartcontractpatternfinderReports/report_{}", timestamp));
    let root_dir = PathBuf::from(root_dir);
    let vuln_summary = root_dir.join("vuln_summary.md");
    
    let mut summary = String::new();
    summary.push_str("# 🚨 Vulnerability Scan Summary\n\n");
    summary.push_str(&format!("**Generated:** {}\n", timestamp));
    summary.push_str(&format!("**Period:** Last {} days\n\n", args.days));
    summary.push_str("---\n\n");
    summary.push_str("## 📊 Scan Results\n\n");
    summary.push_str(&format!("- **Contracts Scanned:** {}\n", scan_results.len()));
    summary.push_str(&format!("- **Exploitable Contracts:** {}\n", exploitable_count));
    summary.push_str(&format!("- **Total Findings:** {}\n\n", stats.exploitable.len() + stats.false_positives.len() + stats.needs_review.len()));
    
    if exploitable_count > 0 {
        summary.push_str("## 🚨 CRITICAL: Exploitable Contracts (Ranked by Risk Score)\n\n");
        
        let mut sorted_exploitable: Vec<_> = exploitable_contracts.iter().map(|idx| (*idx, scan_results[*idx].total_risk_score())).collect();
        sorted_exploitable.sort_by(|a, b| b.1.cmp(&a.1));
        
        for (idx, risk_score) in sorted_exploitable {
            let result = &scan_results[idx];
            let exploitable: Vec<_> = stats.exploitable.iter().filter(|(i, _, _)| *i == idx).collect();
            
            summary.push_str(&format!("### {} ({})", result.address, result.chain));
            summary.push_str(&format!(" - Risk Score: {}\n\n", risk_score));
            
            for (_, m, analysis) in &exploitable {
                if let Some(ctx) = &m.function_context {
                    summary.push_str(&format!("- **Function:** `{}()` [{:?}]\n", ctx.name, ctx.visibility));
                    summary.push_str(&format!("  - **Vulnerability:** {} ({:?})\n", m.pattern_id, m.severity));
                    summary.push_str(&format!("  - **Line:** {}\n", m.line_number));
                    summary.push_str(&format!("  - **Assessment:** {}\n", analysis.reason));
                    summary.push_str(&format!("  - **Confidence:** {:?}\n\n", analysis.confidence));
                }
            }
        }
    }
    
    summary.push_str("\n---\n\n");
    
    std::fs::write(&vuln_summary, summary)?;
    eprintln!("📊 Vulnerability summary: {}", vuln_summary.display());

    Ok(())
}
