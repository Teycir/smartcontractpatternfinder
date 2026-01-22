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

    // Load static templates
    let mut templates = TemplateLoader::load_from_dir(&templates_dir).await?;

    // Load 0-day templates if available
    let zeroday_dir = PathBuf::from("templates-zeroday/latest");
    if zeroday_dir.exists() {
        match TemplateLoader::load_from_dir(&zeroday_dir).await {
            Ok(zeroday_templates) => {
                eprintln!("📡 Loaded {} 0-day templates", zeroday_templates.len());
                templates.extend(zeroday_templates);
            }
            Err(e) => eprintln!("⚠️  Failed to load 0-day templates: {}", e),
        }
    }

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
            .filter(|m| m.severity >= min_sev && m.severity >= scpf_types::Severity::High)
            .collect();

        // Analyze exploitability
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
        let false_positive_count = analyzed_matches
            .iter()
            .filter(|(_, a)| {
                !a.is_exploitable && a.confidence == scpf_core::ExploitConfidence::High
            })
            .count();

        if exploitable_count > 0 {
            eprintln!(
                "🚨 {} ({}) - {} exploitable, {} false positives",
                &address[..10],
                chain.as_str(),
                exploitable_count,
                false_positive_count
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

    let mut scan_results: Vec<_> = all_scan_results
        .into_iter()
        .filter(|r| !r.matches.is_empty())
        .collect();

    // Sort by risk score and take top 100
    scan_results.sort_by(|a, b| b.total_risk_score().cmp(&a.total_risk_score()));
    let top_100: Vec<_> = scan_results.into_iter().take(100).collect();

    // Calculate PoC scores and sort by exploitability
    let mut with_poc_scores: Vec<_> = top_100
        .into_iter()
        .map(|r| {
            let poc_score: f32 = r.matches.iter().map(|m| m.exploitability_score()).sum();
            (r, poc_score)
        })
        .collect();

    with_poc_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Debug: Print PoC scores
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

    // Keep all 100 ranked by PoC
    let mut scan_results: Vec<_> = with_poc_scores.into_iter().map(|(r, _)| r).collect();

    // Sort by exploitable count (most exploitable first)
    scan_results.sort_by(|a, b| {
        let a_exploitable = a
            .matches
            .iter()
            .filter(|m| {
                if let Some(_) = &m.function_context {
                    scpf_core::analyze_exploitability(m).is_exploitable
                } else {
                    false
                }
            })
            .count();
        let b_exploitable = b
            .matches
            .iter()
            .filter(|m| {
                if let Some(_) = &m.function_context {
                    scpf_core::analyze_exploitability(m).is_exploitable
                } else {
                    false
                }
            })
            .count();
        b_exploitable.cmp(&a_exploitable)
    });

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let report_dir = dirs::home_dir()
        .map(|h| h.join("smartcontractpatternfinder"))
        .unwrap_or_else(|| PathBuf::from("results"));
    std::fs::create_dir_all(&report_dir)?;

    let output_file = report_dir.join(format!("{}_scan.json", timestamp));
    let txt_file = report_dir.join(format!("{}_scan.txt", timestamp));

    // Build enriched JSON with exploitability analysis
    let enriched_results: Vec<_> = scan_results
        .iter()
        .map(|result| {
            let mut exploitable = Vec::new();
            let mut false_positives = Vec::new();
            let mut needs_review = Vec::new();

            for m in &result.matches {
                let analysis = scpf_core::analyze_exploitability(m);
                let enriched = serde_json::json!({
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
                        "is_exploitable": analysis.is_exploitable,
                        "confidence": format!("{:?}", analysis.confidence),
                        "reason": analysis.reason,
                    }
                });

                if analysis.is_exploitable {
                    exploitable.push(enriched);
                } else if analysis.confidence == scpf_core::ExploitConfidence::High {
                    false_positives.push(enriched);
                } else {
                    needs_review.push(enriched);
                }
            }

            serde_json::json!({
                "address": result.address,
                "chain": result.chain,
                "risk_score": result.total_risk_score(),
                "solidity_version": result.solidity_version,
                "scan_time_ms": result.scan_time_ms,
                "summary": {
                    "exploitable": exploitable.len(),
                    "false_positives": false_positives.len(),
                    "needs_review": needs_review.len(),
                    "total": result.matches.len(),
                },
                "findings": {
                    "exploitable": exploitable,
                    "false_positives": false_positives,
                    "needs_review": needs_review,
                }
            })
        })
        .collect();

    let json_output = serde_json::to_string_pretty(&enriched_results)?;
    std::fs::write(&output_file, json_output)?;

    eprintln!("\n📊 Results saved to: {}", output_file.display());
    eprintln!(
        "📋 Found {} vulnerable contracts (High/Critical only)\n",
        scan_results.len()
    );

    // Build human-readable TXT output
    let mut txt_output = String::new();
    txt_output.push_str(&format!("Smart Contract Pattern Finder - Scan Report\n"));
    txt_output.push_str(&format!("Timestamp: {}\n", timestamp));
    txt_output.push_str(&format!("Total Contracts: {}\n\n", scan_results.len()));
    txt_output.push_str("=".repeat(80).as_str());
    txt_output.push_str("\n\n");

    // Calculate exploitability stats and print tree view
    let mut exploitable_contracts = 0;
    let mut exploitable_findings = 0;
    let mut false_positive_findings = 0;
    let mut needs_review_findings = 0;

    txt_output.push_str("🌳 EXPLOITABLE CONTRACTS (sorted by exploitable count)\n\n");
    eprintln!("🌳 Exploitable Contracts (sorted by exploitable count):\n");

    for (idx, result) in scan_results.iter().enumerate() {
        let mut exploitable = Vec::new();
        let mut false_positives = Vec::new();
        let mut needs_review = Vec::new();

        for m in &result.matches {
            if let Some(_) = &m.function_context {
                let analysis = scpf_core::analyze_exploitability(m);
                if analysis.is_exploitable {
                    exploitable.push((m, analysis));
                } else if analysis.confidence == scpf_core::ExploitConfidence::High {
                    false_positives.push((m, analysis));
                } else {
                    needs_review.push((m, analysis));
                }
            } else {
                let analysis = scpf_core::analyze_exploitability(m);
                needs_review.push((m, analysis));
            }
        }

        if !exploitable.is_empty() {
            exploitable_contracts += 1;
            exploitable_findings += exploitable.len();

            let header = format!("{}. 🚨 {} ({})", idx + 1, result.address, result.chain);
            let stats = format!(
                "   Risk Score: {} | Exploitable: {} | False Positives: {} | Needs Review: {}",
                result.total_risk_score(),
                exploitable.len(),
                false_positives.len(),
                needs_review.len()
            );

            eprintln!("{}", header);
            eprintln!("{}", stats);
            eprintln!();

            txt_output.push_str(&format!("{}\n", header));
            txt_output.push_str(&format!("{}\n\n", stats));

            for (i, (m, analysis)) in exploitable.iter().enumerate() {
                let is_last = i == exploitable.len() - 1
                    && false_positives.is_empty()
                    && needs_review.is_empty();
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

            if !false_positives.is_empty() {
                eprintln!("   │");
                eprintln!("   ├─ ❌ {} FALSE POSITIVES", false_positives.len());
                txt_output.push_str("   │\n");
                txt_output.push_str(&format!(
                    "   ├─ ❌ {} FALSE POSITIVES\n",
                    false_positives.len()
                ));

                for (m, analysis) in false_positives.iter().take(2) {
                    if let Some(ctx) = &m.function_context {
                        let line = format!(
                            "   │  • {}() - {} ({})",
                            ctx.name, m.pattern_id, analysis.reason
                        );
                        eprintln!("{}", line);
                        txt_output.push_str(&format!("{}\n", line));
                    }
                }
                if false_positives.len() > 2 {
                    let line = format!("   │  • ... and {} more", false_positives.len() - 2);
                    eprintln!("{}", line);
                    txt_output.push_str(&format!("{}\n", line));
                }
            }

            if !needs_review.is_empty() {
                eprintln!("   │");
                eprintln!("   └─ ⚠️  {} NEEDS REVIEW", needs_review.len());
                txt_output.push_str("   │\n");
                txt_output.push_str(&format!("   └─ ⚠️  {} NEEDS REVIEW\n", needs_review.len()));

                for (m, analysis) in needs_review.iter().take(2) {
                    if let Some(ctx) = &m.function_context {
                        let line = format!(
                            "      • {}() - {} ({})",
                            ctx.name, m.pattern_id, analysis.reason
                        );
                        eprintln!("{}", line);
                        txt_output.push_str(&format!("{}\n", line));
                    }
                }
                if needs_review.len() > 2 {
                    let line = format!("      • ... and {} more", needs_review.len() - 2);
                    eprintln!("{}", line);
                    txt_output.push_str(&format!("{}\n", line));
                }
            }

            eprintln!();
            txt_output.push_str("\n");
        }

        false_positive_findings += false_positives.len();
        needs_review_findings += needs_review.len();
    }

    let total_findings = exploitable_findings + false_positive_findings + needs_review_findings;

    let summary = vec![
        format!("\n📈 Summary:"),
        format!(
            "   🚨 Exploitable: {} contracts with {} findings",
            exploitable_contracts, exploitable_findings
        ),
        format!(
            "   ❌ False Positives: {} findings",
            false_positive_findings
        ),
        format!("   ⚠️  Needs Review: {} findings", needs_review_findings),
        format!(
            "   📊 Total: {} findings across {} contracts",
            total_findings,
            scan_results.len()
        ),
    ];

    for line in &summary {
        eprintln!("{}", line);
    }

    txt_output.push_str("\n");
    txt_output.push_str("=".repeat(80).as_str());
    txt_output.push_str("\n\n");
    txt_output.push_str(&summary.join("\n"));
    txt_output.push_str("\n");

    std::fs::write(&txt_file, txt_output)?;
    eprintln!("\n📝 Human-readable report: {}", txt_file.display());

    Ok(())
}

fn parse_severity(s: &str) -> scpf_types::Severity {
    match s.to_lowercase().as_str() {
        "critical" => scpf_types::Severity::Critical,
        "high" => scpf_types::Severity::High,
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
