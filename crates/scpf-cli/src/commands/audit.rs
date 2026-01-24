use crate::cli::ScanArgs;
use anyhow::Result;
use colored::Colorize;

pub async fn run_full_audit(args: ScanArgs) -> Result<()> {
    println!("{}", "🔍 SCPF Security Audit (Static Only)".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();

    println!("Running static template scan...");
    crate::commands::scan::scan_vulnerabilities(args).await?;

    println!();
    println!("{}", "═".repeat(60).cyan());
    println!("{}", "✅ Audit Complete".cyan().bold());
    println!();
    println!("💡 For 0-day research:");
    println!("   1. Run: scpf fetch-zero-day --days 10");
    println!("   2. Manually research and create templates");
    println!("   3. Re-run: scpf audit");
    println!("{}", "═".repeat(60).cyan());

    Ok(())
}
