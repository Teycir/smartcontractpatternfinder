use anyhow::Result;
use std::path::PathBuf;

#[allow(dead_code)]
pub async fn scan_recent_0day_contracts(
    days: u64,
    _min_severity: &str,
    _templates_path: &Option<PathBuf>,
) -> Result<()> {
    eprintln!("⚠️  0-Day scanning disabled - use manual workflow");
    eprintln!("   1. Run: scpf fetch-zero-day --days {}", days);
    eprintln!("   2. Research exploits and create templates manually");
    eprintln!("   3. Run: scpf scan-recent --templates ./templates");
    Ok(())
}
