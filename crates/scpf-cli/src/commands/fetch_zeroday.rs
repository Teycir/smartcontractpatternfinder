use crate::cli::FetchZeroDayArgs;
use anyhow::Result;
use colored::Colorize;
use scpf_core::ZeroDayFetcher;
use std::fs;
use std::path::PathBuf;

pub async fn run(args: FetchZeroDayArgs) -> Result<()> {
    println!("{}", "🔍 SCPF 0-Day News Extractor".cyan().bold());
    println!("{}", "═".repeat(50).cyan());
    println!();

    let fetcher = ZeroDayFetcher::new()?;

    println!(
        "{}  Fetching exploits from last {} days...",
        "📡".cyan(),
        args.days
    );
    let exploits = fetcher.fetch_recent_exploits(args.days as i64).await?;

    if exploits.is_empty() {
        println!("{}  No recent exploits found", "⚠️".yellow());
        return Ok(());
    }

    println!("{}  Found {} recent exploits:", "✓".green(), exploits.len());
    println!();

    // Display all exploits
    for exploit in &exploits {
        println!(
            "  📰 {} - {} ({})",
            exploit.title.bright_white(),
            exploit.source.dimmed(),
            exploit.date.format("%Y-%m-%d").to_string().dimmed()
        );

        if let Some(loss) = exploit.loss_usd {
            println!("     💰 Loss: ${}", format_loss(loss).red());
        }

        if let Some(chain) = &exploit.chain {
            println!("     ⛓️  Chain: {}", chain.dimmed());
        }

        if let Some(addr) = &exploit.contract_address {
            println!("     📍 Contract: {}", addr.dimmed());
        }
    }

    // Generate markdown report
    let output_dir = dirs::home_dir()
        .map(|h| h.join("smartcontractpatternfinder/0day-research"))
        .unwrap_or_else(|| PathBuf::from("0day-research"));

    fs::create_dir_all(&output_dir)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let report_file = output_dir.join(format!("{}_0day_news.md", timestamp));

    let mut report = String::new();
    report.push_str("# 0-Day Exploit Research Report\n\n");
    report.push_str(&format!(
        "**Generated:** {}\n",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));
    report.push_str(&format!("**Period:** Last {} days\n", args.days));
    report.push_str("**Purpose:** Manual template construction\n\n");
    report.push_str("---\n\n");
    report.push_str("## 📰 Recent Exploits\n\n");

    for exploit in &exploits {
        // Skip if no contract address
        if exploit.contract_address.is_none() {
            continue;
        }

        report.push_str(&format!("### {}\n\n", exploit.title));
        report.push_str(&format!(
            "**{}** | **{}**",
            exploit.date.format("%Y-%m-%d"),
            exploit.source
        ));

        if let Some(addr) = &exploit.contract_address {
            report.push_str(&format!(" | `{}`", addr));
        }

        if let Some(tx) = &exploit.tx_hash {
            report.push_str(&format!(" | `{}`", tx));
        }

        report.push_str("\n\n**Links:** ");

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
                report.push_str(&format!("[Contract]({}#code) | ", explorer));
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
                report.push_str(&format!("[TX]({}#eventlog) | ", tx_url));
            }
        }

        let search_query = exploit.title.replace(' ', "+");
        report.push_str(&format!(
            "[PoC](https://github.com/SunWeb3Sec/DeFiHackLabs/search?q={})\n\n",
            search_query
        ));
        report.push_str("**Vuln:** _TBD - analyze contract_\n\n");
        report.push_str("---\n\n");
    }

    // Add template creation guide
    report.push_str(TEMPLATE_GUIDE);

    // Add summary
    report.push_str("\n## 📊 Summary\n\n");
    report.push_str(&format!("**Total Exploits:** {}\n", exploits.len()));
    report.push_str(&format!(
        "**Report Location:** `{}`\n\n",
        report_file.display()
    ));
    report.push_str("**Workflow:**\n");
    report.push_str("1. ✅ News fetched\n");
    report.push_str("2. ⏳ Research exploits (manual)\n");
    report.push_str("3. ⏳ Create templates (manual)\n");
    report.push_str("4. ⏳ Test templates\n");
    report.push_str("5. ⏳ Deploy to production\n");

    fs::write(&report_file, report)?;

    println!();
    println!("{}", "═".repeat(50).cyan());
    println!(
        "{}  Report generated: {}",
        "✅".green(),
        report_file.display()
    );
    println!();
    println!("{}  Quick Links:", "🔗".cyan());
    println!("   • DeFiHackLabs: https://github.com/SunWeb3Sec/DeFiHackLabs");
    println!("   • Rekt News: https://rekt.news");
    println!("   • BlockSec: https://blocksec.com/blog");
    println!("   • PeckShield: https://twitter.com/peckshield");
    println!();
    println!("{}  Next Steps:", "📝".cyan());
    println!("   1. Open report: cat {}", report_file.display());
    println!("   2. Research each exploit using provided links");
    println!("   3. Create templates in templates/ directory");
    println!("   4. Test: scpf scan-recent --templates ./templates");
    println!();

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

const TEMPLATE_GUIDE: &str = r#"
## 🛠️ Quick Guide

1. Click **Contract** link → View source code
2. Click **PoC** link → Understand attack
3. Identify vulnerability pattern in contract
4. Create `templates/0day_YYYYMMDD_name.yaml`
5. Test: `scpf scan <address> --templates ./templates`
"#;
