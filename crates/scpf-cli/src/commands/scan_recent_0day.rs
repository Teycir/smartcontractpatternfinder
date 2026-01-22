use anyhow::Result;
use scpf_core::{ContractFetcher, TemplateLoader, ZeroDayFetcher};
use std::path::PathBuf;
use std::sync::Arc;

use super::scan_common::{fetch_contracts, get_supported_chains, parse_severity, rank_and_score, scan_contracts};

pub async fn scan_recent_0day_contracts(
    days: u64,
    min_severity: &str,
    templates_path: &Option<PathBuf>,
) -> Result<()> {
    eprintln!("🔍 Scanning contracts with 0-day templates (last {} days)...", days);
    eprintln!("   Severity filter: {} and above", min_severity.to_uppercase());

    eprintln!("📡 Fetching 0-day exploits...");
    let zeroday_fetcher = ZeroDayFetcher::new()?;
    let exploits = zeroday_fetcher.fetch_recent_exploits(days as i64).await?;

    if exploits.is_empty() {
        eprintln!("⚠️  No 0-day exploits found in last {} days", days);
        return Ok(());
    }

    eprintln!("   ✓ Found {} exploits", exploits.len());

    let temp_dir = std::env::temp_dir().join("scpf-zeroday");
    std::fs::create_dir_all(&temp_dir)?;
    let template_path = temp_dir.join("zeroday_live.yaml");

    zeroday_fetcher.generate_template(exploits, &template_path).await?;
    eprintln!("   ✓ Generated 0-day template");

    let mut templates = TemplateLoader::load_from_dir(&temp_dir).await?;
    eprintln!("   ✓ Loaded {} 0-day templates", templates.len());

    if let Some(static_path) = templates_path {
        match TemplateLoader::load_from_dir(static_path).await {
            Ok(static_templates) => {
                eprintln!("   ✓ Loaded {} static templates", static_templates.len());
                templates.extend(static_templates);
            }
            Err(e) => eprintln!("   ⚠️  Failed to load static templates: {}", e),
        }
    }

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

    let min_sev = parse_severity(min_severity);
    let all_scan_results = scan_contracts(all_contracts, templates, fetcher, min_sev).await?;
    let scan_results = rank_and_score(all_scan_results);
    // Save results
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let report_dir = dirs::home_dir()
        .map(|h| h.join("smartcontractpatternfinder"))
        .unwrap_or_else(|| PathBuf::from("results"));
    std::fs::create_dir_all(&report_dir)?;

    let output_file = report_dir.join(format!("{}_0day_scan.json", timestamp));
    let txt_file = report_dir.join(format!("{}_0day_scan.txt", timestamp));

    // Build enriched JSON
    let enriched_results: Vec<_> = scan_results
        .iter()
        .map(|result| {
            let mut exploitable = Vec::new();
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
                    "exploitability": {
                        "is_exploitable": analysis.is_exploitable,
                        "confidence": format!("{:?}", analysis.confidence),
                        "reason": analysis.reason,
                    }
                });

                if analysis.is_exploitable {
                    exploitable.push(enriched);
                } else {
                    needs_review.push(enriched);
                }
            }

            serde_json::json!({
                "address": result.address,
                "chain": result.chain,
                "risk_score": result.total_risk_score(),
                "summary": {
                    "exploitable": exploitable.len(),
                    "needs_review": needs_review.len(),
                    "total": result.matches.len(),
                },
                "findings": {
                    "exploitable": exploitable,
                    "needs_review": needs_review,
                }
            })
        })
        .collect();

    let json_output = serde_json::to_string_pretty(&enriched_results)?;
    std::fs::write(&output_file, json_output)?;

    eprintln!("\n📊 Results saved to: {}", output_file.display());
    eprintln!("📋 Found {} vulnerable contracts\n", scan_results.len());

    // Build TXT output
    let mut txt_output = String::new();
    txt_output.push_str("Smart Contract Pattern Finder - 0-Day Scan Report\n");
    txt_output.push_str(&format!("Timestamp: {}\n", timestamp));
    txt_output.push_str(&format!("Total Contracts: {}\n\n", scan_results.len()));
    txt_output.push_str(&"=".repeat(80));
    txt_output.push_str("\n\n");

    let mut exploitable_contracts = 0;
    let mut exploitable_findings = 0;

    txt_output.push_str("🌳 0-DAY EXPLOITABLE CONTRACTS\n\n");
    eprintln!("🌳 0-Day Exploitable Contracts:\n");

    for (idx, result) in scan_results.iter().enumerate() {
        let mut exploitable = Vec::new();
        let mut needs_review = Vec::new();

        for m in &result.matches {
            let analysis = scpf_core::analyze_exploitability(m);
            if analysis.is_exploitable {
                exploitable.push((m, analysis));
            } else {
                needs_review.push((m, analysis));
            }
        }

        if !exploitable.is_empty() {
            exploitable_contracts += 1;
            exploitable_findings += exploitable.len();

            let header = format!("{}. 🚨 {} ({})", idx + 1, result.address, result.chain);
            let stats = format!(
                "   Risk Score: {} | Exploitable: {} | Needs Review: {}",
                result.total_risk_score(),
                exploitable.len(),
                needs_review.len()
            );

            eprintln!("{}", header);
            eprintln!("{}", stats);
            eprintln!();

            txt_output.push_str(&format!("{}\n", header));
            txt_output.push_str(&format!("{}\n\n", stats));

            for (i, (m, analysis)) in exploitable.iter().enumerate() {
                let is_last = i == exploitable.len() - 1 && needs_review.is_empty();
                let prefix = if is_last { "   └─" } else { "   ├─" };

                let lines = vec![
                    format!(
                        "{} ✅ EXPLOITABLE (confidence: {:?})",
                        prefix, analysis.confidence
                    ),
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

            eprintln!();
            txt_output.push_str("\n");
        }
    }

    let summary = vec![
        format!("\n📈 Summary:"),
        format!(
            "   🚨 Exploitable: {} contracts with {} 0-day findings",
            exploitable_contracts, exploitable_findings
        ),
        format!("   📊 Total: {} contracts scanned", scan_results.len()),
    ];

    for line in &summary {
        eprintln!("{}", line);
    }

    txt_output.push_str("\n");
    txt_output.push_str(&"=".repeat(80));
    txt_output.push_str("\n\n");
    txt_output.push_str(&summary.join("\n"));
    txt_output.push_str("\n");

    std::fs::write(&txt_file, txt_output)?;
    eprintln!("\n📝 Human-readable report: {}", txt_file.display());

    Ok(())
}
