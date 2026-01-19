use anyhow::Result;
use scpf_core::{ContractFetcher, Scanner, TemplateLoader};
use std::path::PathBuf;
use crate::cli::ScanArgs;

pub async fn run(args: ScanArgs) -> Result<()> {
    let templates_dir = args.templates.unwrap_or_else(|| PathBuf::from("templates"));
    let templates = TemplateLoader::load_from_dir(&templates_dir)?;
    
    let scanner = Scanner::new(templates);
    let fetcher = ContractFetcher::new(None)?;

    for address in &args.addresses {
        println!("Scanning {}...", address);
        
        let source = fetcher.fetch_source(address, &args.chain).await?;
        let matches = scanner.scan(&source, PathBuf::from(address))?;
        
        println!("Found {} matches", matches.len());
        for m in matches {
            println!("  [{}:{}] {}", m.line_number, m.column, m.message);
        }
    }

    Ok(())
}
