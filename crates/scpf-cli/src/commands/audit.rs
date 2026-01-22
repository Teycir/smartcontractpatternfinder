use crate::cli::ScanArgs;
use anyhow::Result;
use colored::Colorize;

pub async fn run_full_audit(_addresses: Vec<String>, args: ScanArgs) -> Result<()> {
    println!("{}", "🔍 SCPF Full Security Audit".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();

    // Run both scans in parallel: static templates + 0-day templates
    println!("Running parallel scans: static + 0-day...");
    println!();

    let templates_clone = args.templates.clone();
    let static_handle = tokio::spawn(async move {
        crate::commands::scan_recent::scan_recent_contracts(10, "high", &templates_clone).await
    });

    let zeroday_handle = tokio::spawn(async move {
        crate::commands::scan_recent_0day::scan_recent_0day_contracts(10, "high", &args.templates)
            .await
    });

    let (static_result, zeroday_result) = tokio::try_join!(static_handle, zeroday_handle)?;
    static_result?;
    zeroday_result?;

    println!();
    println!("{}", "═".repeat(60).cyan());
    println!("{}", "✅ Audit Complete".cyan().bold());
    println!("{}", "═".repeat(60).cyan());

    Ok(())
}
