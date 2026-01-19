use anyhow::Result;
use futures::stream::{self, StreamExt};
use scpf_core::{Cache, ContractFetcher, Scanner, TemplateLoader};
use std::path::PathBuf;
use std::sync::Arc;
use crate::cli::ScanArgs;

pub async fn run(args: ScanArgs) -> Result<()> {
    let templates_dir = args.templates.unwrap_or_else(|| PathBuf::from("templates"));
    let templates = TemplateLoader::load_from_dir(&templates_dir)?;
    
    if templates.is_empty() {
        anyhow::bail!("No templates found in {:?}", templates_dir);
    }
    
    let scanner = Arc::new(Scanner::new(templates)?);
    let api_key = std::env::var("ETHERSCAN_API_KEY").ok();
    let fetcher = Arc::new(ContractFetcher::new(api_key)?);
    let cache = Arc::new(Cache::new(PathBuf::from(".cache")).await?);
    let chain = Arc::new(args.chain.clone());

    let results = stream::iter(args.addresses.iter())
        .map(|address| {
            let scanner = Arc::clone(&scanner);
            let fetcher = Arc::clone(&fetcher);
            let cache = Arc::clone(&cache);
            let chain = Arc::clone(&chain);
            let address = address.clone();
            
            async move {
                println!("Scanning {}...", address);
                
                let cache_key = format!("{}:{}", chain, address);
                let source = if let Some(cached) = cache.get(&cache_key).await {
                    cached
                } else {
                    let src = fetcher.fetch_source(&address, &chain).await?;
                    cache.set(&cache_key, &src).await?;
                    src
                };
                
                let matches = scanner.scan(&source, PathBuf::from(&address))?;
                
                Ok::<_, anyhow::Error>((address, matches))
            }
        })
        .buffer_unordered(args.concurrency)
        .collect::<Vec<_>>()
        .await;

    for result in results {
        match result {
            Ok((address, matches)) => {
                println!("\n{}: Found {} matches", address, matches.len());
                for m in matches {
                    println!("  [{}:{}] {}", m.line_number, m.column, m.message);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
