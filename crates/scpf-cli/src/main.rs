use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(match cli.verbose {
            0 => tracing::Level::WARN,
            1 => tracing::Level::INFO,
            2 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        })
        .init();

    match cli.command {
        Commands::Scan(args) => commands::scan::run(args).await,
        Commands::Init(args) => commands::init::run(args).await,
    }
}
