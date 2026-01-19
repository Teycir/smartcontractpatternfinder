use anyhow::Result;
use scpf_core::{Cache, ContractFetcher, Scanner, TemplateLoader};
use std::path::PathBuf;
use crate::cli::ScanArgs;

pub async fn run(args: ScanArgs) -> Result<()> {
    let templates_dir = args.templates.unwrap_or_else(|| PathBuf::from("templates"));
    let templates = TemplateLoader::load_from_dir(&templates_dir)?;
    
    if templates.is_empty() {
        anyhow::bail!("No templates found in {:?}", templates_dir);
    }
    
    let scanner = Scanner::new(templates);
    let api_key = std::env::var("ETHERSCAN_API_KEY").ok();
    let fetcher = ContractFetcher::new(api_key)?;
    let cache = Cache::new(PathBuf::from(".cache"))?;

    for address in &args.addresses {
        println!("Scanning {}...", address);
        
        let cache_key = format!("{}:{}", args.chain, address);
        let source = if let Some(cached) = cache.get(&cache_key) {
            cached
        } else {
            let src = fetcher.fetch_source(address, &args.chain).await?;
            cache.set(&cache_key, &src)?;
            src
        };
        
        let matches = scanner.scan(&source, PathBuf::from(address))?;
        
        println!("Found {} matches", matches.len());
        for m in matches {
            println!("  [{}:{}] {}", m.line_number, m.column, m.message);
        }
    }

    Ok(())
}
