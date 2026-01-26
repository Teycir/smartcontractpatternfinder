use crate::cli::FetchZeroDayArgs;
use anyhow::Result;
use colored::Colorize;
use scpf_core::ZeroDayFetcher;
use std::fs;
use std::path::PathBuf;

pub async fn run(args: FetchZeroDayArgs) -> Result<()> {
    eprintln!("{}", "🔍 SCPF 0-Day News Extractor".cyan().bold());
    eprintln!("{}", "═".repeat(50).cyan());
    eprintln!();

    let fetcher = ZeroDayFetcher::new()?;

    // Use 30 days minimum to ensure we get data
    let days_to_fetch = if args.days < 30 { 30 } else { args.days };

    eprintln!(
        "{}  Fetching exploits from last {} days...",
        "📡".cyan(),
        days_to_fetch
    );
    let mut exploits = fetcher.fetch_recent_exploits(days_to_fetch as i64).await?;

    if exploits.is_empty() {
        eprintln!("{}  No recent exploits found", "⚠️".yellow());
        return Ok(());
    }

    eprintln!("{}  Found {} recent exploits", "✓".green(), exploits.len());
    
    // Sort by date (newest first)
    exploits.sort_by(|a, b| b.date.cmp(&a.date));
    
    let with_addr = exploits.iter().filter(|e| e.contract_address.is_some()).count();
    eprintln!("   ✓ {} with addresses, {} without", with_addr, exploits.len() - with_addr);
    
    eprintln!("⏳ Processing exploits...");
    eprintln!();

    let root_dir = if let Some(output_path) = &args.output {
        output_path.parent().unwrap().to_path_buf()
    } else {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        PathBuf::from(std::env::var("SCPF_REPORT_DIR").unwrap_or_else(|_| {
            format!(
                "/home/teycir/smartcontractpatternfinderReports/report_{}",
                timestamp
            )
        }))
    };
    std::fs::create_dir_all(&root_dir)?;
    let zeroday_summary = args.output.clone().unwrap_or_else(|| root_dir.join("0day_summary.md"));

    let mut summary = String::new();
    summary.push_str("# 🔥 0-Day Exploit Summary\n\n");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    summary.push_str(&format!("**Generated:** {}\n", timestamp));
    summary.push_str(&format!("**Period:** Last {} days (fetched {})

", args.days, days_to_fetch));
    summary.push_str("---\n\n");
    let with_address: Vec<_> = exploits.iter().filter(|e| e.contract_address.is_some()).collect();

    summary.push_str(&format!(
        "## 📊 Overview\n\n- **Total Exploits:** {}\n- **With Contract Address:** {}\n\n",
        exploits.len(),
        with_address.len()
    ));

    summary.push_str("## 📰 Exploits with Contract Addresses\n\n");

    for exploit in &with_address {
        summary.push_str(&format!("### {}\n\n", exploit.title));
        summary.push_str(&format!("**Date:** {} | **Source:** {}\n\n", exploit.date.format("%Y-%m-%d"), exploit.source));

        if let Some(addr) = &exploit.contract_address {
            summary.push_str(&format!("**Contract:** `{}`\n\n", addr));
        }
        if let Some(tx) = &exploit.tx_hash {
            summary.push_str(&format!("**TX Hash:** `{}`\n\n", tx));
        }
        if let Some(chain) = &exploit.chain {
            summary.push_str(&format!("**Chain:** {}\n\n", chain));
        }
        if let Some(loss) = exploit.loss_usd {
            summary.push_str(&format!("**Loss:** ${}\n\n", format_loss(loss)));
        }

        summary.push_str("**Links:**\n\n");

        if let Some(addr) = &exploit.contract_address {
            if let Some(chain) = &exploit.chain {
                let explorer = match chain.as_str() {
                    "ethereum" => format!("https://etherscan.io/address/{}", addr),
                    "bsc" => format!("https://bscscan.com/address/{}", addr),
                    "polygon" => format!("https://polygonscan.com/address/{}", addr),
                    "arbitrum" => format!("https://arbiscan.io/address/{}", addr),
                    "base" => format!("https://basescan.org/address/{}", addr),
                    _ => format!("https://etherscan.io/address/{}", addr),
                };
                summary.push_str(&format!("- [View Contract on Explorer]({}#code)\n", explorer));
            }
        }

        if let Some(tx) = &exploit.tx_hash {
            if let Some(chain) = &exploit.chain {
                let tx_url = match chain.as_str() {
                    "ethereum" => format!("https://etherscan.io/tx/{}", tx),
                    "bsc" => format!("https://bscscan.com/tx/{}", tx),
                    "polygon" => format!("https://polygonscan.com/tx/{}", tx),
                    "arbitrum" => format!("https://arbiscan.io/tx/{}", tx),
                    "base" => format!("https://basescan.org/tx/{}", tx),
                    _ => format!("https://etherscan.io/tx/{}", tx),
                };
                summary.push_str(&format!("- [View Transaction]({}#eventlog)\n", tx_url));
            }
        }

        let search_query = exploit.title.replace(' ', "+");
        summary.push_str(&format!("- [Search PoC on DeFiHackLabs](https://github.com/SunWeb3Sec/DeFiHackLabs/search?q={})\n", search_query));

        summary.push_str("\n---\n\n");
    }

    fs::write(&zeroday_summary, summary)?;



    eprintln!("✅ Processing complete");
    eprintln!();
    eprintln!("{}", "═".repeat(50).cyan());
    eprintln!(
        "{}  0-Day summary: {}",
        "✅".green(),
        zeroday_summary.display()
    );
    eprintln!();

    Ok(())
}

fn format_loss(loss: u64) -> String {
    if loss >= 1_000_000 {
        format!("{:.1}M", loss as f64 / 1_000_000.0)
    } else if loss >= 1_000 {
        format!("{:.1}K", loss as f64 / 1_000.0)
    } else {
        loss.to_string()
    }
}
