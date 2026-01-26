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
    long_about = "A high-performance tool for detecting security vulnerabilities and patterns in smart contracts.\n\nExamples:\n  scpf scan --chains ethereum,polygon,arbitrum --days 10\n  scpf scan 0x1234... --chains ethereum\n  scpf scan 0x1234... 0x5678... --concurrency 20\n  scpf scan 0x1234... --output json > results.json\n  scpf init --yes\n\nMore: https://github.com/Teycir/smartcontractpatternfinder"
)]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count, global = true, help = "Increase verbosity (-v, -vv, -vvv)")]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Full security audit: update templates + fetch + scan + report")]
    Audit(ScanArgs),
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
    #[command(about = "Interactive pattern builder helper")]
    PatternBuilder(PatternBuilderArgs),
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

    #[arg(short = 'n', long, value_delimiter = ',', value_parser = parse_chain, help = "Blockchain networks (comma-separated: ethereum,polygon,arbitrum)")]
    pub chains: Vec<Chain>,

    #[arg(short, long, help = "Path to templates directory")]
    pub templates: Option<PathBuf>,

    #[arg(
        short,
        long,
        default_value = "console",
        help = "Output format (console, json, sarif)"
    )]
    pub output: OutputFormat,

    #[arg(long, default_value = "3", help = "Number of concurrent requests")]
    pub concurrency: usize,

    #[arg(long, help = "Bypass cache and fetch fresh data")]
    pub no_cache: bool,

    #[arg(long, help = "Only scan files changed in git diff (e.g., main..HEAD)")]
    pub diff: Option<String>,

    #[arg(
        long,
        help = "Update templates with 0-day exploits from last N days (0 = no update)"
    )]
    pub update_templates: Option<i64>,

    // Filtering options
    #[arg(long, default_value = "10", help = "Scan contracts from last N days")]
    pub days: u64,

    #[arg(
        long,
        default_value = "true",
        help = "Scan all available chains (fixed: true)"
    )]
    pub all_chains: bool,
    #[arg(
        long,
        help = "Filter by contract type (erc20, erc721, erc1155, proxy, defi)"
    )]
    pub contract_type: Option<String>,

    #[arg(
        long,
        help = "Only scan contracts updated after this date (YYYY-MM-DD)"
    )]
    pub updated_after: Option<String>,

    #[arg(
        long,
        help = "Only scan contracts updated before this date (YYYY-MM-DD)"
    )]
    pub updated_before: Option<String>,

    #[arg(
        long,
        default_value = "high",
        help = "Minimum severity to report (info, low, medium, high, critical)"
    )]
    pub min_severity: String,

    #[arg(
        long,
        help = "Filter by specific vulnerability tags (comma-separated: reentrancy,access-control)"
    )]
    pub tags: Option<String>,

    #[arg(long, help = "Exclude specific templates by ID (comma-separated)")]
    pub exclude_templates: Option<String>,

    #[arg(long, help = "Only use specific templates by ID (comma-separated)")]
    pub only_templates: Option<String>,

    #[arg(long, help = "Enable fast mode (skip semantic analysis for speed)")]
    pub fast: bool,

    #[arg(
        long,
        help = "Sort vulnerabilities by exploitability score (PoC success probability)"
    )]
    pub sort_by_exploitability: bool,
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
    pub days: u32,

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
    #[command(about = "Install template collection from registry")]
    Install {
        #[arg(help = "Collection name (e.g., defi, erc, mev)")]
        collection: String,
        #[arg(short, long, help = "Path to templates directory")]
        templates: Option<PathBuf>,
    },
    #[command(about = "Update all installed templates")]
    Update {
        #[arg(short, long, help = "Path to templates directory")]
        templates: Option<PathBuf>,
    },
    #[command(about = "List available template collections")]
    Registry,
}

#[derive(Args)]
pub struct PatternBuilderArgs {
    #[arg(short, long, help = "Load test code from file")]
    pub file: Option<String>,

    #[arg(short, long, help = "Initial pattern string")]
    pub pattern: Option<String>,
}
