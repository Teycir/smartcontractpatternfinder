use clap::{Args, Parser, Subcommand, ValueEnum};
use scpf_types::Chain;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "scpf")]
#[command(
    author,
    version,
    about = "Smart Contract Pattern Finder - Detect vulnerabilities in smart contracts"
)]
#[command(
    long_about = "A high-performance tool for detecting security vulnerabilities and patterns in smart contracts.\n\nExamples:\n  scpf scan 0x1234... --chain ethereum\n  scpf scan 0x1234... 0x5678... --concurrency 20\n  scpf scan 0x1234... --output json > results.json\n  scpf init --yes\n\nMore: https://github.com/Teycir/smartcontractpatternfinder"
)]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count, global = true, help = "Increase verbosity (-v, -vv, -vvv)")]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Scan smart contracts for vulnerabilities")]
    #[command(after_help = "More: https://github.com/Teycir/smartcontractpatternfinder#scanning")]
    Scan(ScanArgs),
    #[command(about = "Initialize a new SCPF project with templates")]
    #[command(
        after_help = "More: https://github.com/Teycir/smartcontractpatternfinder#initialization"
    )]
    Init(InitArgs),
    #[command(about = "Manage vulnerability detection templates")]
    Templates(TemplatesArgs),
    #[command(about = "Fetch latest 0-day patterns from security feeds")]
    #[command(after_help = "Updates templates with vulnerabilities disclosed in last 7 days")]
    FetchZeroDay(FetchZeroDayArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Console,
    Json,
    Sarif,
}

#[derive(Args)]
pub struct ScanArgs {
    #[arg(
        help = "Contract addresses to scan (0x...). If empty, auto-detects from current directory"
    )]
    pub addresses: Vec<String>,

    #[arg(short = 'n', long, default_value = "ethereum", value_parser = parse_chain, help = "Blockchain network (ethereum, bsc, polygon)")]
    pub chain: Chain,

    #[arg(short, long, help = "Path to templates directory")]
    pub templates: Option<PathBuf>,

    #[arg(
        short,
        long,
        default_value = "console",
        help = "Output format (console, json, sarif)"
    )]
    pub output: OutputFormat,

    #[arg(long, default_value = "10", help = "Number of concurrent requests")]
    pub concurrency: usize,

    #[arg(long, help = "Bypass cache and fetch fresh data")]
    pub no_cache: bool,
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

#[derive(Args)]
pub struct TemplatesArgs {
    #[command(subcommand)]
    pub command: TemplatesCommand,
}

#[derive(Args)]
pub struct FetchZeroDayArgs {
    #[arg(
        short,
        long,
        default_value = "7",
        help = "Fetch exploits from last N days"
    )]
    pub days: i64,

    #[arg(short, long, help = "Output path for generated template")]
    pub output: Option<PathBuf>,

    #[arg(long, help = "Show exploits without generating template")]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum TemplatesCommand {
    #[command(about = "List all available templates")]
    List {
        #[arg(short, long, help = "Path to templates directory")]
        templates: Option<PathBuf>,
    },
    #[command(about = "Show details of a specific template")]
    Show {
        #[arg(help = "Template ID to display")]
        id: String,
        #[arg(short, long, help = "Path to templates directory")]
        templates: Option<PathBuf>,
    },
}
