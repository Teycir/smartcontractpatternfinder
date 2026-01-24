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

    eprintln!(
        "{}  Fetching exploits from last {} days...",
        "📡".cyan(),
        args.days
    );
    let exploits = fetcher.fetch_recent_exploits(args.days as i64).await?;

    if exploits.is_empty() {
        eprintln!("{}  No recent exploits found", "⚠️".yellow());
        return Ok(());
    }

    eprintln!("{}  Found {} recent exploits", "✓".green(), exploits.len());
    eprintln!("⏳ Processing exploits...");
    eprintln!();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let root_dir = std::env::var("SCPF_REPORT_DIR")
        .unwrap_or_else(|_| format!("/home/teycir/smartcontractpatternfinderReports/report_{}", timestamp));
    let root_dir = PathBuf::from(root_dir);
    let zeroday_summary = root_dir.join("0day_summary.md");
    
    let mut summary = String::new();
    summary.push_str("# 🔥 0-Day Exploit Summary\n\n");
    summary.push_str(&format!("**Generated:** {}\n", timestamp));
    summary.push_str(&format!("**Period:** Last {} days\n\n", args.days));
    summary.push_str("---\n\n");
    summary.push_str(&format!("## 📊 Overview\n\n**Total Exploits Found:** {}\n\n", exploits.len()));
    summary.push_str("## 📰 Recent Exploits\n\n");
    
    for exploit in &exploits {
        if exploit.contract_address.is_none() {
            continue;
        }
        
        summary.push_str(&format!("### {}\n\n", exploit.title));
        summary.push_str(&format!("**{}** | **{}", exploit.date.format("%Y-%m-%d"), exploit.source));
        
        if let Some(addr) = &exploit.contract_address {
            summary.push_str(&format!(" | `{}`", addr));
        }
        if let Some(tx) = &exploit.tx_hash {
            summary.push_str(&format!(" | `{}`", tx));
        }
        summary.push_str("\n\n**Links:** ");
        
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
                summary.push_str(&format!("[Contract]({}#code) | ", explorer));
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
                summary.push_str(&format!("[TX]({}#eventlog) | ", tx_url));
            }
        }
        
        let search_query = exploit.title.replace(' ', "+");
        summary.push_str(&format!("[PoC](https://github.com/SunWeb3Sec/DeFiHackLabs/search?q={})\n\n", search_query));
        
        if let Some(loss) = exploit.loss_usd {
            summary.push_str(&format!("**Loss:** ${}\n\n", format_loss(loss)));
        }
        if let Some(chain) = &exploit.chain {
            summary.push_str(&format!("**Chain:** {}\n\n", chain));
        }
        
        summary.push_str("---\n\n");
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
