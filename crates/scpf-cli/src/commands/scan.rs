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

    let report_dir = PathBuf::from("/home/teycir/smartcontractpatternfinderReports/scans");
    std::fs::create_dir_all(&report_dir)?;

    let output_file = report_dir.join(format!("{}_scan.json", timestamp));
    let txt_file = report_dir.join(format!("{}_scan.txt", timestamp));

    let stats = categorize_findings(&scan_results);

    let enriched_results: Vec<_> = scan_results
        .iter()
        .map(|result| {
            let exploitable: Vec<_> = stats
                .exploitable
                .iter()
                .filter(|(idx, _, _)| scan_results[*idx].address == result.address)
                .map(|(_, m, a)| {
                    serde_json::json!({
                        "pattern_id": m.pattern_id,
                        "template_id": m.template_id,
                        "severity": format!("{:?}", m.severity),
                        "line_number": m.line_number,
                        "message": m.message,
                        "matched_text": m.matched_text,
                        "function": m.function_context.as_ref().map(|ctx| serde_json::json!({
                            "name": ctx.name,
                            "visibility": format!("{:?}", ctx.visibility),
                            "mutability": format!("{:?}", ctx.mutability),
                            "modifiers": ctx.modifiers,
                        })),
                        "exploitability": {
                            "is_exploitable": a.is_exploitable,
                            "confidence": format!("{:?}", a.confidence),
                            "reason": a.reason,
                        }
                    })
                })
                .collect();

            let fps: Vec<_> = stats
                .false_positives
                .iter()
                .filter(|(idx, _, _)| scan_results[*idx].address == result.address)
                .map(|(_, m, a)| {
                    serde_json::json!({
                        "pattern_id": m.pattern_id,
                        "severity": format!("{:?}", m.severity),
                        "line_number": m.line_number,
                        "exploitability": { "reason": a.reason }
                    })
                })
                .collect();

            let review: Vec<_> = stats
                .needs_review
                .iter()
                .filter(|(idx, _, _)| scan_results[*idx].address == result.address)
                .map(|(_, m, a)| {
                    serde_json::json!({
                        "pattern_id": m.pattern_id,
                        "severity": format!("{:?}", m.severity),
                        "line_number": m.line_number,
                        "exploitability": { "reason": a.reason }
                    })
                })
                .collect();

            serde_json::json!({
                "address": result.address,
                "chain": result.chain,
                "risk_score": result.total_risk_score(),
                "solidity_version": result.solidity_version,
                "scan_time_ms": result.scan_time_ms,
                "summary": {
                    "exploitable": exploitable.len(),
                    "false_positives": fps.len(),
                    "needs_review": review.len(),
                    "total": result.matches.len(),
                },
                "findings": {
                    "exploitable": exploitable,
                    "false_positives": fps,
                    "needs_review": review,
                }
            })
        })
        .collect();

    std::fs::write(
        &output_file,
        serde_json::to_string_pretty(&enriched_results)?,
    )?;
    eprintln!("\n📊 Results saved to: {}", output_file.display());
    eprintln!(
        "📋 Found {} vulnerable contracts (High/Critical only)\n",
        scan_results.len()
    );

    let mut txt_output = String::new();
    txt_output.push_str(&format!(
        "Smart Contract Pattern Finder - Scan Report\nTimestamp: {}\nTotal Contracts: {}\n\n{}\n\n",
        timestamp,
        scan_results.len(),
        "=".repeat(80)
    ));
    txt_output.push_str("🌳 EXPLOITABLE CONTRACTS (sorted by exploitable count)\n\n");
    eprintln!("🌳 Exploitable Contracts (sorted by exploitable count):\n");

    let mut exploitable_contracts = std::collections::HashSet::new();
    for (idx, _, _) in &stats.exploitable {
        exploitable_contracts.insert(*idx);
    }

    let exploitable_count = exploitable_contracts.len();

    for idx in &exploitable_contracts {
        let result = &scan_results[*idx];
        let exploitable: Vec<_> = stats
            .exploitable
            .iter()
            .filter(|(i, _, _)| i == idx)
            .collect();
        let fps: Vec<_> = stats
            .false_positives
            .iter()
            .filter(|(i, _, _)| i == idx)
            .collect();
        let review: Vec<_> = stats
            .needs_review
            .iter()
            .filter(|(i, _, _)| i == idx)
            .collect();

        let header = format!("{}. 🚨 {} ({})", idx + 1, result.address, result.chain);
        let stats_line = format!(
            "   Risk Score: {} | Exploitable: {} | False Positives: {} | Needs Review: {}",
            result.total_risk_score(),
            exploitable.len(),
            fps.len(),
            review.len()
        );

        eprintln!("{}", header);
        eprintln!("{}", stats_line);
        eprintln!();
        txt_output.push_str(&format!("{}\n{}\n\n", header, stats_line));

        for (i, (_, m, analysis)) in exploitable.iter().enumerate() {
            let is_last = i == exploitable.len() - 1 && fps.is_empty() && review.is_empty();
            let prefix = if is_last { "   └─" } else { "   ├─" };

            if let Some(ctx) = &m.function_context {
                let lines = vec![
                    format!(
                        "{} ✅ EXPLOITABLE (confidence: {:?})",
                        prefix, analysis.confidence
                    ),
                    format!("   │  Function: {}() [{:?}]", ctx.name, ctx.visibility),
                    format!("   │  Vulnerability: {} ({:?})", m.pattern_id, m.severity),
                    format!("   │  Line: {} | Message: {}", m.line_number, m.message),
                    format!("   │  Assessment: {}", analysis.reason),
                ];

                for line in &lines {
                    eprintln!("{}", line);
                    txt_output.push_str(&format!("{}\n", line));
                }

                if !is_last {
                    eprintln!("   │");
                    txt_output.push_str("   │\n");
                }
            }
        }

        if !fps.is_empty() {
            eprintln!("   │");
            eprintln!("   ├─ ❌ {} FALSE POSITIVES", fps.len());
            txt_output.push_str(&format!("   │\n   ├─ ❌ {} FALSE POSITIVES\n", fps.len()));

            for (_, m, analysis) in fps.iter().take(2) {
                if let Some(ctx) = &m.function_context {
                    let line = format!(
                        "   │  • {}() - {} ({})",
                        ctx.name, m.pattern_id, analysis.reason
                    );
                    eprintln!("{}", line);
                    txt_output.push_str(&format!("{}\n", line));
                }
            }
            if fps.len() > 2 {
                let line = format!("   │  • ... and {} more", fps.len() - 2);
                eprintln!("{}", line);
                txt_output.push_str(&format!("{}\n", line));
            }
        }

        if !review.is_empty() {
            eprintln!("   │");
            eprintln!("   └─ ⚠️  {} NEEDS REVIEW", review.len());
            txt_output.push_str(&format!("   │\n   └─ ⚠️  {} NEEDS REVIEW\n", review.len()));

            for (_, m, analysis) in review.iter().take(2) {
                if let Some(ctx) = &m.function_context {
                    let line = format!(
                        "      • {}() - {} ({})",
                        ctx.name, m.pattern_id, analysis.reason
                    );
                    eprintln!("{}", line);
                    txt_output.push_str(&format!("{}\n", line));
                }
            }
            if review.len() > 2 {
                let line = format!("      • ... and {} more", review.len() - 2);
                eprintln!("{}", line);
                txt_output.push_str(&format!("{}\n", line));
            }
        }

        eprintln!();
        txt_output.push('\n');
    }

    let summary = format!("\n📈 Summary:\n   🚨 Exploitable: {} contracts with {} findings\n   ❌ False Positives: {} findings\n   ⚠️  Needs Review: {} findings\n   📊 Total: {} findings across {} contracts",
        exploitable_count, stats.exploitable.len(), stats.false_positives.len(), stats.needs_review.len(),
        stats.exploitable.len() + stats.false_positives.len() + stats.needs_review.len(), scan_results.len());

    eprintln!("{}", summary);
    txt_output.push_str(&format!("\n{}\n\n{}", "=".repeat(80), summary));

    std::fs::write(&txt_file, txt_output)?;
    eprintln!("\n📝 Human-readable report: {}", txt_file.display());

    // Generate single executive summary at root
    let root_dir = PathBuf::from("/home/teycir/smartcontractpatternfinderReports");
    let exec_summary = root_dir.join("EXECUTIVE_SUMMARY.md");
    
    let mut summary = String::new();
    summary.push_str("# 🚨 SCPF Full Security Report\n\n");
    summary.push_str(&format!("**Generated:** {}\n", timestamp));
    summary.push_str(&format!("**Period:** Last {} days\n\n", args.days));
    summary.push_str("---\n\n");
    
    // Include 0-day info if available
    let zeroday_dir = root_dir.join("0days");
    if let Ok(entries) = std::fs::read_dir(&zeroday_dir) {
        if let Some(latest) = entries.filter_map(|e| e.ok()).filter(|e| e.path().extension().map_or(false, |ext| ext == "md")).max_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH)) {
            if let Ok(content) = std::fs::read_to_string(latest.path()) {
                if let Some(exploits_section) = content.split("## 📰 Recent Exploits").nth(1) {
                    summary.push_str("## 🔥 Recent 0-Day Exploits\n\n");
                    summary.push_str("## 📰 Recent Exploits");
                    summary.push_str(&exploits_section.lines().take(50).collect::<Vec<_>>().join("\n"));
                    summary.push_str("\n\n---\n\n");
                }
            }
        }
    }
    
    summary.push_str("## 📊 Vulnerability Scan Results\n\n");
    summary.push_str(&format!("- **Contracts Scanned:** {}\n", scan_results.len()));
    summary.push_str(&format!("- **Exploitable Contracts:** {}\n", exploitable_count));
    summary.push_str(&format!("- **Total Findings:** {}\n\n", stats.exploitable.len() + stats.false_positives.len() + stats.needs_review.len()));
    
    if exploitable_count > 0 {
        summary.push_str("## 🚨 CRITICAL: Exploitable Contracts\n\n");
        for idx in &exploitable_contracts {
            let result = &scan_results[*idx];
            let exploitable: Vec<_> = stats.exploitable.iter().filter(|(i, _, _)| i == idx).collect();
            
            summary.push_str(&format!("### {} ({})", result.address, result.chain));
            summary.push_str(&format!(" - Risk Score: {}\n\n", result.total_risk_score()));
            
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
    summary.push_str("## 📂 Full Reports\n\n");
    summary.push_str(&format!("- 0-Day News: `{}/0days/`\n", root_dir.display()));
    summary.push_str(&format!("- Vulnerability Scans: `{}/scans/`\n", root_dir.display()));
    
    std::fs::write(&exec_summary, summary)?;
    eprintln!("📊 Executive summary: {}", exec_summary.display());

    Ok(())
}
