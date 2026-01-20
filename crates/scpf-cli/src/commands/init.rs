use crate::cli::InitArgs;
use anyhow::Result;
use std::fs;

pub async fn run(args: InitArgs) -> Result<()> {
    let templates_dir = args.path.join("templates");
    fs::create_dir_all(&templates_dir)?;

    println!("Initialized SCPF project at {:?}", args.path);
    println!("Templates directory: {:?}", templates_dir);

    Ok(())
}
