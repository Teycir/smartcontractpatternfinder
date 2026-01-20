use crate::cli::FetchZeroDayArgs;
use anyhow::Result;
use colored::Colorize;
use scpf_core::{zeroday::ExploitType, ZeroDayFetcher};
use std::path::PathBuf;

pub async fn run(args: FetchZeroDayArgs) -> Result<()> {
    println!("{}", "🔍 SCPF 0-Day Pattern Fetcher".cyan().bold());
    println!("{}", "═".repeat(50).cyan());
    println!();

    let fetcher = ZeroDayFetcher::new()?;

    println!(
        "{}  Fetching exploits from last {} days...",
        "📡".cyan(),
        args.days
    );
    let exploits = fetcher.fetch_recent_exploits(args.days).await?;

    if exploits.is_empty() {
        println!("{}  No recent exploits found", "⚠️".yellow());
        return Ok(());
    }

    println!("{}  Found {} recent exploits:", "✓".green(), exploits.len());
    println!();

    // Display exploits
    for exploit in &exploits {
        let severity_icon = match exploit.exploit_type {
            ExploitType::Reentrancy => "🔴",
            ExploitType::FlashLoan => "🔴",
            ExploitType::OracleManipulation => "🟠",
            ExploitType::AccessControl => "🟡",
            ExploitType::Unknown => "⚪",
        };

        println!(
            "  {} {} - {} ({})",
            severity_icon,
            exploit.title.bright_white(),
            exploit.source.dimmed(),
            exploit.date.format("%Y-%m-%d").to_string().dimmed()
        );

        if let Some(loss) = exploit.loss_usd {
            println!("     💰 Loss: ${}", format_loss(loss).red());
        }
    }

    println!();

    if args.dry_run {
        println!("{}  Dry run - no template generated", "ℹ️".blue());
        return Ok(());
    }

    // Generate template
    let output_path = args
        .output
        .unwrap_or_else(|| PathBuf::from("templates/zero_day_live.yaml"));

    println!("{}  Generating detection patterns...", "🔨".cyan());
    fetcher.generate_template(exploits, &output_path).await?;

    println!();
    println!(
        "{}  Template generated: {}",
        "✅".green(),
        output_path.display()
    );
    println!();
    println!("{}  Next steps:", "→".cyan().bold());
    println!("   1. Review patterns: cat {}", output_path.display());
    println!(
        "   2. Scan contracts: scpf scan <address> --templates {}",
        output_path.display()
    );
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
