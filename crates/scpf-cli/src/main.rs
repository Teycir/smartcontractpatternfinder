use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod error_helper;
mod output;
mod pattern_builder;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists
    if dotenvy::dotenv().is_err() {
        eprintln!("Warning: No .env file found. API keys must be set via environment variables.");
        eprintln!("Create a .env file with your API keys or export them manually.");
    }

    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(match cli.verbose {
            0 => tracing::Level::WARN,
            1 => tracing::Level::INFO,
            2 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        })
        .init();

    let result = match cli.command {
        Commands::Audit(args) => {
            if args.addresses.is_empty() {
                eprintln!("Error: Audit command requires contract addresses");
                eprintln!("Usage: scpf audit 0x... 0x... --chain ethereum");
                std::process::exit(1);
            }
            commands::audit::run_full_audit(args.addresses.clone(), args).await
        }
        Commands::Scan(args) => commands::scan::run(args).await,
        Commands::Init(args) => commands::init::run(args).await,
        Commands::Templates(args) => match args.command {
            cli::TemplatesCommand::List { templates } => commands::templates::list(templates).await,
            cli::TemplatesCommand::Show { id, templates } => commands::templates::show(&id, templates).await,
            cli::TemplatesCommand::Install { collection, templates } => commands::templates::install(&collection, templates).await,
            cli::TemplatesCommand::Update { templates } => commands::templates::update(templates).await,
            cli::TemplatesCommand::Registry => commands::templates::registry().await,
        },
        Commands::FetchZeroDay(args) => commands::fetch_zeroday::run(args).await,
    };

    if let Err(e) = &result {
        eprintln!("{}", error_helper::format_error_with_help(e));
    }

    result
}
