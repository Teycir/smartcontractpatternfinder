use anyhow::Result;
use scpf_core::{ContractFetcher, TemplateLoader};
use std::path::PathBuf;
use std::sync::Arc;

use super::scan_common::{fetch_contracts, get_supported_chains, parse_severity, rank_and_score, scan_contracts};

pub async fn scan_recent_contracts(
    days: u64,
    min_severity: &str,
    templates_path: &Option<PathBuf>,
) -> Result<()> {
    eprintln!("🔍 Scanning contracts updated in last {} days...", days);
    eprintln!("   Severity filter: {} and above", min_severity.to_uppercase());

    let api_keys = crate::keys::load_api_keys_from_env();
    let fetcher = Arc::new(ContractFetcher::new(api_keys)?);
    let chains = get_supported_chains();

    let all_contracts = fetch_contracts(&fetcher, &chains, days).await;
    if all_contracts.is_empty() {
        eprintln!("⚠️  No recent contracts found");
        return Ok(());
    }

    eprintln!();
    eprintln!("🔎 Scanning {} contracts...", all_contracts.len());
    eprintln!();

    let templates_dir = templates_path.clone().unwrap_or_else(|| PathBuf::from("templates"));
    let templates = TemplateLoader::load_from_dir(&templates_dir).await?;

    let min_sev = parse_severity(min_severity);
    let all_scan_results = scan_contracts(all_contracts, templates, fetcher, min_sev).await?;
    let scan_results = rank_and_score(all_scan_results);
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
