use clap::{Parser, Subcommand, Args, ValueEnum};
use std::path::PathBuf;
use scpf_types::Chain;
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "scpf")]
#[command(author, version, about = "Smart Contract Pattern Finder")]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Scan(ScanArgs),
    Init(InitArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Console,
    Json,
    Sarif,
}

#[derive(Args)]
pub struct ScanArgs {
    pub addresses: Vec<String>,

    #[arg(short = 'n', long, default_value = "ethereum", value_parser = parse_chain)]
    pub chain: Chain,

    #[arg(short, long)]
    pub templates: Option<PathBuf>,

    #[arg(short, long, default_value = "console")]
    pub output: OutputFormat,

    #[arg(long, default_value = "10")]
    pub concurrency: usize,
}

fn parse_chain(s: &str) -> Result<Chain, String> {
    Chain::from_str(s)
}

#[derive(Args)]
pub struct InitArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,

    #[arg(short, long)]
    pub yes: bool,
}
