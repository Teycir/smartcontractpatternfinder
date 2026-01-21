use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod error_helper;
mod pattern_builder;
pub mod keys;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists, ignore if missing
    dotenvy::dotenv().ok();

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
                anyhow::bail!("Audit command requires contract addresses\nUsage: scpf audit 0x... 0x... --chain ethereum");
            }
            commands::audit::run_full_audit(args.addresses.clone(), args).await
        }
        Commands::Scan(args) => commands::scan_recent::scan_recent_contracts(args.days, &args.min_severity, &args.templates).await,
        Commands::Init(args) => commands::init::run(args).await,
        Commands::Templates(args) => match args.command {
            cli::TemplatesCommand::List { templates } => commands::templates::list(templates).await,
            cli::TemplatesCommand::Show { id, templates } => commands::templates::show(&id, templates).await,
            cli::TemplatesCommand::Install { collection, templates } => commands::templates::install(&collection, templates).await,
            cli::TemplatesCommand::Update { templates } => commands::templates::update(templates).await,
            cli::TemplatesCommand::Registry => commands::templates::registry().await,
        },
        Commands::FetchZeroDay(args) => commands::fetch_zeroday::run(args).await,
        Commands::PatternBuilder(args) => {
            pattern_builder::cmd_pattern_builder(args.file.as_deref(), args.pattern.as_deref())
                .map_err(|e| anyhow::anyhow!(e))
        }
    };

    if let Err(e) = &result {
        eprintln!("{}", error_helper::format_error_with_help(e));
    }

    result
}
