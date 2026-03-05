use crate::cli::ScanArgs;
use anyhow::Result;
use chrono;
use futures::stream::{self, StreamExt};
use scpf_core::{Cache, ContractFetcher, HoneypotFilter, Scanner, TemplateLoader};
use scpf_types::{Chain, ScanResult, Severity, Template};
use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

async fn get_balance(address: &str, chain: &str, _fetcher: &ContractFetcher) -> u64 {
    let rpc_url = match chain {
        "ethereum" => "https://eth.llamarpc.com",
        "polygon" => "https://polygon.llamarpc.com",
        "arbitrum" => "https://arbitrum.llamarpc.com",
        _ => return 0,
    };

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getBalance",
        "params": [address, "latest"],
        "id": 1
    });

    match client.post(rpc_url).json(&payload).send().await {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(result) = json.get("result").and_then(|r| r.as_str()) {
                    return u64::from_str_radix(result.trim_start_matches("0x"), 16).unwrap_or(0);
                }
            }
            0
        }
        Err(_) => 0,
    }
}

fn get_supported_chains() -> Vec<Chain> {
    vec![Chain::Ethereum, Chain::Polygon, Chain::Arbitrum]
}

async fn fetch_contracts(
    fetcher: &ContractFetcher,
    chains: &[Chain],
    pages: u64,
) -> Vec<(String, Chain)> {
    let mut all_contracts = Vec::new();
    for chain in chains {
        eprintln!("📡 Fetching from {}...", chain.as_str());
        match fetcher.fetch_recent_contracts(*chain, pages).await {
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
    concurrency: usize,
) -> Result<(
    Vec<ScanResult>,
    Arc<Cache>,
    Vec<(String, Chain, f64)>,
    chrono::DateTime<chrono::Local>,
    chrono::DateTime<chrono::Local>,
)> {
    let scanner = Arc::new(Scanner::new(templates)?);
    let cache_dir = dirs::cache_dir()
        .map(|d| d.join("scpf"))
        .unwrap_or_else(|| PathBuf::from(".cache"));
    let cache = Arc::new(Cache::new(cache_dir).await?);

    let total = contracts.len();
    let scan_start = Instant::now();
    let start_time = chrono::Local::now();

    eprintln!("⏳ Scanning {} contracts...", total);
    eprintln!("🕐 Started: {}", start_time.format("%Y-%m-%d %H:%M:%S"));

    let is_tty = std::io::stderr().is_terminal();
    let progress = Arc::new(tokio::sync::Mutex::new(0usize));
    let findings_count = Arc::new(tokio::sync::Mutex::new(0usize));
    let seen_hashes = Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new()));
    let honeypot_filter =
        Arc::new(HoneypotFilter::new().expect("Failed to create honeypot filter"));
    let honeypot_count = Arc::new(tokio::sync::Mutex::new(0usize));

    // Use concurrency parameter for both API and scan limits
    let api_semaphore = Arc::new(Semaphore::new(concurrency));
    let scan_semaphore = Arc::new(Semaphore::new(concurrency));

    let results: Vec<_> = stream::iter(contracts.into_iter().enumerate())
        .map(|(_idx, (address, chain))| {
            let scanner = Arc::clone(&scanner);
            let fetcher = Arc::clone(&fetcher);
            let cache = Arc::clone(&cache);
            let progress = Arc::clone(&progress);
            let findings_count = Arc::clone(&findings_count);
            let seen_hashes = Arc::clone(&seen_hashes);
            let honeypot_filter = Arc::clone(&honeypot_filter);
            let honeypot_count = Arc::clone(&honeypot_count);
            let api_semaphore = Arc::clone(&api_semaphore);
            let scan_semaphore = Arc::clone(&scan_semaphore);

            async move {
                let short_addr = &address;

                // Progress reporting
                {
                    let mut p = progress.lock().await;
                    *p += 1;
                    if is_tty {
                        if *p % 10 == 0 {
                            let elapsed = scan_start.elapsed().as_secs_f64();
                            let rate = *p as f64 / elapsed;
                            let remaining = total - *p;
                            let eta_secs = (remaining as f64 / rate) as u64;
                            let eta_mins = eta_secs / 60;
                            let findings = findings_count.lock().await;

                            eprint!(
                                "\r📊 {} / {} contracts scanned ({:.1}%) • {:.1}/s | ETA: {}m{}s | Critical: {}   ",
                                *p, total, (*p as f64 / total as f64) * 100.0,
                                rate, eta_mins, eta_secs % 60, *findings
                            );
                            use std::io::Write;
                            std::io::stderr().flush().ok();
                        }
                    } else {
                        eprintln!(
                            "   [{}/{}] ({:.1}%) Scanning {} ({})...",
                            *p, total, (*p as f64 / total as f64) * 100.0, short_addr, chain.as_str()
                        );
                    }
                }
                let cache_key = format!("{}:{}", chain, address);
                let cached_source = cache.get(&cache_key).await.filter(|s| !s.is_empty());

                let (source, from_cache) = if let Some(cached) = cached_source {
                    (cached, true)
                } else {
                    // Acquire API semaphore for rate limiting
                    let _permit = api_semaphore.acquire().await.unwrap();
                    // Add 300ms delay between API calls to avoid rate limits (safe buffer)
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    if !is_tty {
                        eprintln!("      📡 Fetching source from API...");
                    }
                    match fetcher.fetch_source(&address, chain).await {
                        Ok(src) => {
                            if !src.is_empty() {
                                cache.set(&cache_key, &src).await.ok();
                            }
                            if !is_tty {
                                eprintln!("      ✓ Source fetched ({:.1} KB)", src.len() as f64 / 1024.0);
                            }
                            (src, false)
                        }
                        Err(e) => {
                            eprintln!("      ✗ Error fetching {}: {}", short_addr, e);
                            return Ok::<_, anyhow::Error>(None);
                        }
                    }
                };

                if !is_tty && from_cache {
                    eprintln!("      📦 Using cached source ({:.1} KB)", source.len() as f64 / 1024.0);
                }

                // Content-based deduplication using hash
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                source.hash(&mut hasher);
                let content_hash = hasher.finish();

                {
                    let mut hashes = seen_hashes.lock().await;
                    if !hashes.insert(content_hash) {
                        if !is_tty {
                            eprintln!("      ⏭️  Skipping duplicate content (hash: {:x})", content_hash);
                        }
                        return Ok(None);
                    }
                }

                let source_size_kb = source.len() as f64 / 1024.0;
                if source_size_kb > 5120.0 {
                    if !is_tty {
                        eprintln!("      ⏭️  Skipping extremely large contract ({:.1} KB)", source_size_kb);
                    }
                    return Ok(None);
                }

                // Honeypot detection
                let (is_honeypot, honeypot_patterns) = honeypot_filter.is_honeypot(&source);
                if is_honeypot {
                    let mut hp_count = honeypot_count.lock().await;
                    *hp_count += 1;
                    if !is_tty {
                        eprintln!("      🍯 Honeypot detected - skipping (patterns: {})", honeypot_patterns.join(", "));
                    }
                    return Ok(None);
                }

                let start = Instant::now();

                // Acquire scan semaphore for CPU-bound work
                let _permit = scan_semaphore.acquire().await.unwrap();
                let scanner_clone = Arc::clone(&scanner);
                let source_clone = source.clone();
                let address_clone = address.clone();

                let scan_task = tokio::task::spawn_blocking(move || {
                    scanner_clone.scan(&source_clone, PathBuf::from(&address_clone))
                });

                let matches = match tokio::time::timeout(std::time::Duration::from_secs(60), scan_task).await {
                    Ok(Ok(Ok(m))) => m,
                    Ok(Ok(Err(e))) => {
                        eprintln!("      ✗ Scan error: {}", e);
                        return Ok(None);
                    }
                    Ok(Err(e)) => {
                        eprintln!("      ✗ Task error: {}", e);
                        return Ok(None);
                    }
                    Err(_) => {
                        eprintln!("      ⏱️  TIMEOUT after 60s - {} ({}) - {:.1} KB", address, chain.as_str(), source_size_kb);
                        return Ok(Some((None, Some((address.clone(), chain, source_size_kb)))));
                    }
                };

                let scan_time_ms = start.elapsed().as_millis() as u64;

                let filtered_matches: Vec<_> = matches
                    .into_iter()
                    .filter(|m| m.severity >= min_severity && m.severity == Severity::Critical && !m.filtered)
                    .collect();

                if !filtered_matches.is_empty() {
                    let mut findings = findings_count.lock().await;
                    *findings += filtered_matches.len();
                    if !is_tty {
                        eprintln!("      🔍 Scanned in {}ms - {} findings", scan_time_ms, filtered_matches.len());
                    }
                } else if !is_tty {
                    eprintln!("      🔍 Scanned in {}ms - clean", scan_time_ms);
                }

                let analyzed_matches: Vec<_> = filtered_matches
                    .into_iter()
                    .map(|m| {
                        let analysis = scpf_core::analyze_exploitability(&m);
                        (m, analysis)
                    })
                    .collect();

                let result = ScanResult {
                    address,
                    chain: chain.to_string(),
                    matches: analyzed_matches.into_iter().map(|(m, _)| m).collect(),
                    scan_time_ms,
                    solidity_version: extract_solidity_version(&source),
                    source_size_kb: Some(source_size_kb),
                };

                Ok(Some((Some(result), None)))
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    eprintln!("\n✅ Scanning complete\n");

    let end_time = chrono::Local::now();
    let duration = scan_start.elapsed();
    let duration_mins = duration.as_secs() / 60;
    let duration_secs = duration.as_secs() % 60;

    let honeypot_total = *honeypot_count.lock().await;

    eprintln!("🕐 Started:  {}", start_time.format("%Y-%m-%d %H:%M:%S"));
    eprintln!("🕐 Finished: {}", end_time.format("%Y-%m-%d %H:%M:%S"));
    eprintln!("⏱️  Duration: {}m {}s", duration_mins, duration_secs);
    eprintln!(
        "📊 Rate: {:.2} contracts/sec",
        total as f64 / duration.as_secs().max(1) as f64
    );
    if honeypot_total > 0 {
        eprintln!("🍯 Honeypots filtered: {}", honeypot_total);
    }
    eprintln!();

    let mut all_scan_results = Vec::new();
    let mut skipped_timeouts = Vec::new();

    for result in results {
        match result {
            Ok(Some((Some(scan_result), _))) => all_scan_results.push(scan_result),
            Ok(Some((None, Some(timeout)))) => skipped_timeouts.push(timeout),
            _ => {}
        }
    }

    Ok((
        all_scan_results,
        cache,
        skipped_timeouts,
        start_time,
        end_time,
    ))
}

fn rank_and_score(mut scan_results: Vec<ScanResult>) -> Vec<ScanResult> {
    scan_results.retain(|r| !r.matches.is_empty());

    // Deduplicate by size (3 decimal precision)
    let mut seen_sizes = std::collections::HashSet::new();
    scan_results.retain(|r| {
        if let Some(size) = r.source_size_kb {
            let size_key = (size * 1000.0).round() as i64;
            seen_sizes.insert(size_key)
        } else {
            true
        }
    });

    // Sort by weighted risk score (normalized by size)
    scan_results.sort_by(|a, b| {
        b.weighted_risk_score()
            .partial_cmp(&a.weighted_risk_score())
            .unwrap()
    });

    let top_ranked: Vec<_> = scan_results.into_iter().take(200).collect();
    let mut with_poc_scores: Vec<_> = top_ranked
        .into_iter()
        .map(|r| {
            let poc_score: f32 = r.weighted_risk_score() as f32;
            (r, poc_score)
        })
        .collect();

    with_poc_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    eprintln!("\n🎯 Top 10 by PoC Score:");
    for (i, (r, score)) in with_poc_scores.iter().take(10).enumerate() {
        let weighted = r.weighted_risk_score();
        eprintln!(
            "  {}. {} - PoC: {:.1} (Weighted Risk: {:.1}, Raw: {})",
            i + 1,
            &r.address,
            score,
            weighted,
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

pub async fn scan_vulnerabilities(args: ScanArgs) -> Result<()> {
    let api_keys = crate::keys::load_api_keys_from_env();
    let fetcher = Arc::new(ContractFetcher::new(api_keys)?);
    let chains = if args.chains.is_empty() {
        get_supported_chains()
    } else {
        args.chains.clone()
    };

    // If addresses provided, use them directly instead of fetching
    let all_contracts = if !args.addresses.is_empty() {
        eprintln!("🔍 Scanning {} provided addresses...", args.addresses.len());
        args.addresses
            .iter()
            .map(|addr| {
                (
                    addr.clone(),
                    chains.first().cloned().unwrap_or(Chain::Ethereum),
                )
            })
            .collect()
    } else {
        eprintln!("🔍 Fetching {} pages of contracts...", args.pages);
        eprintln!(
            "   Severity filter: {} and above",
            args.min_severity.to_uppercase()
        );
        fetch_contracts(&fetcher, &chains, args.pages).await
    };

    let _total_contracts_fetched = all_contracts.len();
    if all_contracts.is_empty() {
        eprintln!("⚠️  No recent contracts found");

        // If 0-day fetch is enabled, continue to generate 0-day report even without contract scanning
        if args.fetch_zero_day.is_some() {
            eprintln!("   ℹ️  Skipping contract scanning, generating 0-day report only...\n");

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let root_dir = std::env::var("SCPF_REPORT_DIR").unwrap_or_else(|_| {
                format!(
                    "/home/teycir/smartcontractpatternfinderReports/report_{}",
                    timestamp
                )
            });
            let root_dir = PathBuf::from(root_dir);
            std::fs::create_dir_all(&root_dir)?;

            // Generate 0-day summary
            if let Some(days) = args.fetch_zero_day {
                let zeroday_args = crate::cli::FetchZeroDayArgs {
                    days,
                    output: Some(root_dir.join("0day_summary.md")),
                    dry_run: false,
                };
                if let Err(e) = crate::commands::fetch_zeroday::run(zeroday_args).await {
                    eprintln!("⚠️  Failed to fetch 0-day exploits: {}", e);
                } else {
                    eprintln!("\n✅ 0-day report generated successfully");
                    eprintln!("📂 Report directory: {}", root_dir.display());
                }
            }

            return Ok(());
        }

        return Ok(());
    }

    eprintln!();
    eprintln!("⏳ Loading templates...");
    let templates_dir = args
        .templates
        .clone()
        .unwrap_or_else(|| PathBuf::from("templates"));
    let templates = TemplateLoader::load_from_dir(&templates_dir).await?;

    let template_count = templates.len();
    eprintln!("✅ Loaded {} templates", template_count);

    let mut template_by_severity = std::collections::HashMap::new();
    for t in &templates {
        *template_by_severity.entry(t.severity).or_insert(0) += 1;
    }

    eprintln!("   📋 By severity:");
    if let Some(count) = template_by_severity.get(&Severity::Critical) {
        eprintln!("      - Critical: {}", count);
    }
    if let Some(count) = template_by_severity.get(&Severity::High) {
        eprintln!("      - High: {}", count);
    }
    eprintln!();

    let min_sev = parse_severity(&args.min_severity);
    let fetcher_clone = Arc::clone(&fetcher);
    let (all_scan_results, cache, skipped_timeouts, _scan_start_time, _scan_end_time) =
        scan_contracts(
            all_contracts,
            templates,
            fetcher_clone,
            min_sev,
            args.concurrency,
        )
        .await?;
    let _total_scanned = all_scan_results.len();
    let scan_results = rank_and_score(all_scan_results);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    eprintln!("\n📊 Scan Summary:");
    eprintln!(
        "   📋 Total Findings: {} across {} contracts\n",
        scan_results.iter().map(|r| r.matches.len()).sum::<usize>(),
        scan_results.len()
    );

    let root_dir = std::env::var("SCPF_REPORT_DIR").unwrap_or_else(|_| {
        format!(
            "/home/teycir/smartcontractpatternfinderReports/report_{}",
            timestamp
        )
    });
    let root_dir = PathBuf::from(root_dir);

    // Create report directory
    std::fs::create_dir_all(&root_dir)?;

    let vuln_summary = root_dir.join("vuln_summary.md");

    // Generate enhanced summary
    let mut summary = String::new();
    summary.push_str("# 🚨 Vulnerability Scan Summary\n\n");
    summary.push_str(&format!("**Generated:** {}\n", timestamp));
    summary.push_str(&format!("**Pages:** {}\n", args.pages));
    summary.push_str(&format!(
        "**Chains:** {}\n",
        chains
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    summary.push_str(&format!(
        "**Min Severity:** {}\n\n",
        args.min_severity.to_uppercase()
    ));
    summary.push_str("---\n\n");

    summary.push_str("## 📊 Scan Results\n\n");
    summary.push_str(&format!(
        "- **Contracts Scanned:** {}\n",
        scan_results.len()
    ));
    summary.push_str(&format!(
        "- **Total Findings:** {}\n\n",
        scan_results.iter().map(|r| r.matches.len()).sum::<usize>()
    ));

    // Pattern frequency analysis
    // Pattern frequency analysis
    let mut pattern_counts = std::collections::HashMap::new();
    let mut template_counts = std::collections::HashMap::new();
    let mut severity_counts = std::collections::HashMap::new();

    for result in &scan_results {
        for m in &result.matches {
            *pattern_counts.entry(m.pattern_id.as_str()).or_insert(0) += 1;
            *template_counts.entry(m.template_id.as_str()).or_insert(0) += 1;
            *severity_counts.entry(m.severity).or_insert(0) += 1;
        }
    }
    let mut pattern_vec: Vec<_> = pattern_counts.into_iter().collect();
    pattern_vec.sort_by(|a, b| b.1.cmp(&a.1));

    summary.push_str("## 🔍 Pattern Frequency\n\n");
    for (pattern, count) in pattern_vec.iter().take(10) {
        summary.push_str(&format!("- **{}**: {} occurrences\n", pattern, count));
    }
    summary.push('\n');

    // Template breakdown
    summary.push_str("## 📋 Findings by Template\n\n");
    let mut template_vec: Vec<_> = template_counts.into_iter().collect();
    template_vec.sort_by(|a, b| b.1.cmp(&a.1));
    for (template, count) in template_vec {
        summary.push_str(&format!("- **{}**: {} findings\n", template, count));
    }
    summary.push('\n');

    // Severity breakdown
    summary.push_str("## ⚠️ Findings by Severity\n\n");
    if let Some(count) = severity_counts.get(&Severity::Critical) {
        summary.push_str(&format!("- **Critical**: {}\n", count));
    }
    if let Some(count) = severity_counts.get(&Severity::High) {
        summary.push_str(&format!("- **High**: {}\n", count));
    }
    summary.push('\n');

    // Chain distribution
    let mut chain_counts = std::collections::HashMap::new();
    for result in &scan_results {
        *chain_counts.entry(result.chain.as_str()).or_insert(0) += 1;
    }
    summary.push_str("## 🌐 Chain Distribution\n\n");
    for (chain, count) in chain_counts {
        summary.push_str(&format!("- **{}**: {} contracts\n", chain, count));
    }
    summary.push('\n');

    // Top 20 contracts table
    if !scan_results.is_empty() {
        summary.push_str("## 🎯 Top 20 Riskiest Contracts\n\n");
        summary.push_str("| Rank | Address | Chain | Risk Score | Findings | Size (KB) |\n");
        summary.push_str("|------|---------|-------|------------|----------|-----------|\n");
        for (i, result) in scan_results.iter().take(20).enumerate() {
            let risk = result.weighted_risk_score();
            let size = result.source_size_kb.unwrap_or(0.0);
            summary.push_str(&format!(
                "| {} | {} | {} | {:.1} | {} | {:.1} |\n",
                i + 1,
                &result.address,
                result.chain,
                risk,
                result.matches.len(),
                size
            ));
        }
        summary.push('\n');
    }

    summary.push_str("\n---\n\n");

    std::fs::write(&vuln_summary, summary)?;
    eprintln!("📊 Vulnerability summary: {}", vuln_summary.display());

    // Generate 0-day summary if fetch_zero_day was enabled
    if let Some(days) = args.fetch_zero_day {
        let zeroday_args = crate::cli::FetchZeroDayArgs {
            days,
            output: Some(root_dir.join("0day_summary.md")),
            dry_run: false,
        };
        if let Err(e) = crate::commands::fetch_zeroday::run(zeroday_args).await {
            eprintln!("⚠️  Failed to fetch 0-day exploits: {}", e);
        }
    }

    // Extract top N contract sources
    let top_n = std::env::var("SCPF_EXTRACT_TOP_N")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    let extract_by_risk = std::env::var("SCPF_EXTRACT_BY_RISK")
        .ok()
        .map(|s| s == "1" || s.to_lowercase() == "true")
        .unwrap_or(false);

    // Extract top contracts by risk score
    let _extracted_count = if extract_by_risk && !scan_results.is_empty() {
        eprintln!("\n📄 Extracting top {} contracts by risk score...", top_n);

        for (count, result) in scan_results.iter().take(top_n).enumerate() {
            let cache_key = format!("{}:{}", result.chain, result.address);
            let weighted_risk = result.weighted_risk_score();

            if let Some(source) = cache.get(&cache_key).await {
                let output_file = root_dir.join(format!(
                    "{}_{}_risk{:.0}.sol",
                    count + 1,
                    result.address,
                    weighted_risk
                ));
                let lines = source.lines().count();
                std::fs::write(&output_file, &source)?;
                let size = std::fs::metadata(&output_file)?.len();
                eprintln!(
                    "   ✅ [{}] {} - Weighted Risk: {:.1} ({} KB, {} lines)",
                    count + 1,
                    &result.address,
                    weighted_risk,
                    size / 1024,
                    lines
                );
            }
        }
        scan_results.iter().take(top_n).count()
    } else {
        0
    };

    // Extract top N riskiest contract sources if --extract-sources is set
    let _total_extracted = if let Some(extract_count) = args.extract_sources {
        let sources_dir = root_dir.join("sources");
        std::fs::create_dir_all(&sources_dir)?;

        eprintln!(
            "\n📁 Extracting top {} riskiest contract sources...",
            extract_count
        );

        let mut saved_count = 0;
        let mut total_size: u64 = 0;
        let mut skipped_zero_balance = 0;

        // scan_results is already sorted by weighted risk score (from rank_and_score)
        for (rank, result) in scan_results.iter().take(extract_count).enumerate() {
            let cache_key = format!("{}:{}", result.chain, result.address);

            if let Some(source) = cache.get(&cache_key).await {
                // Balance check if flag enabled
                if args.skip_zero_balance {
                    let balance = get_balance(&result.address, &result.chain, &fetcher).await;
                    if balance == 0 {
                        skipped_zero_balance += 1;
                        eprintln!(
                            "   [{}/{}] {} ({}) - SKIPPED (0 ETH balance)",
                            rank + 1,
                            extract_count,
                            &result.address,
                            result.chain
                        );
                        continue;
                    }
                }

                // Create chain subdirectory
                let chain_dir = sources_dir.join(&result.chain);
                std::fs::create_dir_all(&chain_dir)?;

                let weighted_risk = result.weighted_risk_score();
                let output_file = chain_dir.join(format!(
                    "{:03}_{}_risk{:.0}.sol",
                    rank + 1,
                    result.address,
                    weighted_risk
                ));
                std::fs::write(&output_file, &source)?;

                let size = source.len() as u64;
                total_size += size;
                saved_count += 1;

                eprintln!(
                    "   [{}/{}] {} ({}) - Risk: {:.1}",
                    rank + 1,
                    extract_count,
                    &result.address,
                    result.chain,
                    weighted_risk
                );
            }
        }

        if skipped_zero_balance > 0 {
            eprintln!(
                "\n   ⏭️  Skipped {} contracts with 0 ETH balance",
                skipped_zero_balance
            );
        }

        eprintln!(
            "\n   ✅ Extracted {} contract sources ({:.1} MB total) to {}",
            saved_count,
            total_size as f64 / (1024.0 * 1024.0),
            sources_dir.display()
        );
        saved_count
    } else {
        0
    };

    eprintln!("\n{}", "=".repeat(80));
    eprintln!("📂 Report directory: {}", root_dir.display());
    eprintln!("{}", "=".repeat(80));

    // Report skipped contracts
    if !skipped_timeouts.is_empty() {
        eprintln!(
            "\n⏱️  {} contracts skipped due to timeout (>60s):",
            skipped_timeouts.len()
        );
        for (addr, chain, size_kb) in &skipped_timeouts {
            eprintln!("   - {} ({}) - {:.1} KB", addr, chain.as_str(), size_kb);
        }

        // Write to file
        let timeout_log = root_dir.join("timeouts.txt");
        let mut timeout_content = "Contracts that timed out (>60s scan time):\n\n".to_string();
        for (addr, chain, size_kb) in &skipped_timeouts {
            timeout_content.push_str(&format!(
                "{} ({}) - {:.1} KB\n",
                addr,
                chain.as_str(),
                size_kb
            ));
        }
        std::fs::write(&timeout_log, timeout_content)?;
        eprintln!("   📝 Timeout list saved to: {}", timeout_log.display());
    }

    // Play notification sound
    eprintln!("\x07"); // Bell character

    Ok(())
}
