use anyhow::Result;
use clap::Parser;
use scpf_config::load_process_env;

mod cli;
mod commands;
mod error_helper;
mod pattern_builder;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    load_process_env();

    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(match cli.verbose {
            0 => tracing::Level::WARN,
            1 => tracing::Level::INFO,
            2 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        })
        .with_ansi(false)
        .init();

    let result = match cli.command {
        Commands::Audit(args) => commands::audit::run_full_audit(args).await,
        Commands::Scan(args) => commands::scan::scan_vulnerabilities(args).await,
        Commands::Init(args) => commands::init::run(args).await,
        Commands::Templates(args) => match args.command {
            cli::TemplatesCommand::List { templates } => commands::templates::list(templates).await,
            cli::TemplatesCommand::Show { id, templates } => {
                commands::templates::show(&id, templates).await
            }
            cli::TemplatesCommand::Install {
                collection,
                templates,
            } => commands::templates::install(&collection, templates).await,
            cli::TemplatesCommand::Update { templates } => {
                commands::templates::update(templates).await
            }
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
        std::process::exit(1);
    }

    Ok(())
}
