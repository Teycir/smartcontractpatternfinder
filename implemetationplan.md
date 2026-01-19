The user wants me to create a full, production-ready Rust project called "SmartContractPatternFinder" that incorporates all my suggestions for improvements. They want it to be:

Modular - reusable code in other projects
Solid/robust - working and not breaking easily
Full implementation - not just snippets
Let me create a comprehensive, well-structured Rust project with:

Core library (scpf-core) - The reusable library component
CLI application (scpf-cli) - The command-line interface
Shared types (scpf-types) - Common types and traits
Key features to implement:

Subcommands (scan, init, new, test, watch, templates)
Persistent SQLite cache
Multi-source fetching (Etherscan, Sourcify, Blockscout)
Parallel template matching
Watch mode with notifications
Interactive template wizard
Rich progress display
Auto API key discovery
Profile-based scanning
Bytecode pre-filtering
Error handling with thiserror/anyhow
Comprehensive logging
Rate limiting
Retry logic with backoff
I'll structure this as a Cargo workspace with multiple crates for maximum modularity.

Let me write out the complete implementation:

SmartContractPatternFinder (SCPF)
A production-ready, modular smart contract pattern scanner with enterprise features.

Project Structure
text

smart-contract-pattern-finder/
├── Cargo.toml                          # Workspace root
├── README.md
├── LICENSE
├── .env.example
├── config.example.yaml
│
├── crates/
│   ├── scpf-core/                      # Core library (reusable)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── config/
│   │       │   ├── mod.rs
│   │       │   ├── chain.rs
│   │       │   └── settings.rs
│   │       ├── template/
│   │       │   ├── mod.rs
│   │       │   ├── types.rs
│   │       │   ├── parser.rs
│   │       │   ├── matcher.rs
│   │       │   ├── extractor.rs
│   │       │   ├── validator.rs
│   │       │   └── store.rs
│   │       ├── scanner/
│   │       │   ├── mod.rs
│   │       │   ├── engine.rs
│   │       │   ├── result.rs
│   │       │   └── parallel.rs
│   │       ├── source/
│   │       │   ├── mod.rs
│   │       │   ├── provider.rs
│   │       │   ├── etherscan.rs
│   │       │   ├── sourcify.rs
│   │       │   ├── blockscout.rs
│   │       │   └── multi.rs
│   │       ├── cache/
│   │       │   ├── mod.rs
│   │       │   ├── memory.rs
│   │       │   └── persistent.rs
│   │       └── utils/
│   │           ├── mod.rs
│   │           ├── rate_limiter.rs
│   │           ├── retry.rs
│   │           └── hash.rs
│   │
│   ├── scpf-cli/                       # CLI application
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── cli.rs
│   │       ├── commands/
│   │       │   ├── mod.rs
│   │       │   ├── scan.rs
│   │       │   ├── init.rs
│   │       │   ├── new_template.rs
│   │       │   ├── test.rs
│   │       │   ├── watch.rs
│   │       │   └── templates.rs
│   │       ├── output/
│   │       │   ├── mod.rs
│   │       │   ├── console.rs
│   │       │   ├── json.rs
│   │       │   ├── sarif.rs
│   │       │   └── progress.rs
│   │       └── notify/
│   │           ├── mod.rs
│   │           ├── slack.rs
│   │           ├── discord.rs
│   │           └── telegram.rs
│   │
│   └── scpf-types/                     # Shared types (for library consumers)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── severity.rs
│           ├── chain.rs
│           ├── contract.rs
│           └── match_result.rs
│
├── templates/                          # Default templates
│   ├── vulnerabilities/
│   ├── patterns/
│   ├── defi/
│   └── profiles/
│
└── tests/
    ├── integration/
    └── samples/
Workspace Cargo.toml
toml

[workspace]
resolver = "2"
members = [
    "crates/scpf-core",
    "crates/scpf-cli",
    "crates/scpf-types",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
authors = ["Your Team <team@example.com>"]
license = "MIT"
repository = "https://github.com/yourorg/smart-contract-pattern-finder"
rust-version = "1.75"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full", "parking_lot"] }
futures = "0.3"
async-trait = "0.1"

# HTTP
reqwest = { version = "0.11", features = ["json", "gzip", "rustls-tls", "cookies"], default-features = false }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# CLI
clap = { version = "4.4", features = ["derive", "env", "unicode", "wrap_help", "color"] }

# Regex
regex = "1.10"
fancy-regex = "0.13"

# Database
rusqlite = { version = "0.30", features = ["bundled"] }

# Caching
moka = { version = "0.12", features = ["future"] }

# Rate limiting
governor = "0.6"

# Retry
backoff = { version = "0.4", features = ["tokio"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# Console
colored = "2.1"
console = "0.15"
indicatif = { version = "0.17", features = ["tokio"] }
dialoguer = "0.11"
tabled = "0.15"

# HTML parsing
scraper = "0.18"

# Hashing
xxhash-rust = { version = "0.8", features = ["xxh3"] }
sha2 = "0.10"

# URL
url = "2.5"

# UUID
uuid = { version = "1.6", features = ["v4"] }

# Glob
glob = "0.3"
globset = "0.4"

# Parallel processing
rayon = "1.8"

# Bloom filter
bloomfilter = "1.0"

# File watching
notify = "6.1"

# Once cell
once_cell = "1.19"

# Semver
semver = "1.0"

# Diff
similar = "2.4"

# Bytes
bytes = "1.5"
hex = "0.4"

# Directory
dirs = "5.0"

# Zip for template downloads
zip = "0.6"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
opt-level = 3

[profile.dev]
opt-level = 1
crates/scpf-types/Cargo.toml
toml

[package]
name = "scpf-types"
description = "Shared types for Smart Contract Pattern Finder"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
serde = { workspace = true }
chrono = { workspace = true }
crates/scpf-types/src/lib.rs
Rust

//! Shared types for Smart Contract Pattern Finder
//! 
//! This crate provides common types that can be used by both the core library
//! and any external consumers who want to integrate with SCPF.

mod severity;
mod chain;
mod contract;
mod match_result;

pub use severity::Severity;
pub use chain::{Chain, ChainId};
pub use contract::{Contract, ContractInfo, SourceCode};
pub use match_result::{ScanMatch, MatchLocation, ExtractionResult};
crates/scpf-types/src/severity.rs
Rust

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Severity levels for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    #[default]
    Info,
    Unknown,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
            Severity::Info => "info",
            Severity::Unknown => "unknown",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🟢",
            Severity::Info => "🔵",
            Severity::Unknown => "⚪",
        }
    }

    pub fn all() -> &'static [Severity] {
        &[
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
            Severity::Info,
        ]
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "critical" | "crit" => Ok(Severity::Critical),
            "high" | "h" => Ok(Severity::High),
            "medium" | "med" | "m" => Ok(Severity::Medium),
            "low" | "l" => Ok(Severity::Low),
            "info" | "informational" | "i" => Ok(Severity::Info),
            _ => Err(format!("Unknown severity: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::Info);
    }

    #[test]
    fn test_severity_from_str() {
        assert_eq!(Severity::from_str("critical").unwrap(), Severity::Critical);
        assert_eq!(Severity::from_str("HIGH").unwrap(), Severity::High);
        assert_eq!(Severity::from_str("med").unwrap(), Severity::Medium);
    }
}
crates/scpf-types/src/chain.rs
Rust

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Ethereum,
    Polygon,
    Bsc,
    Arbitrum,
    Optimism,
    Base,
    Avalanche,
    Fantom,
    Gnosis,
    Custom(u64),
}

impl Chain {
    pub fn chain_id(&self) -> ChainId {
        ChainId(match self {
            Chain::Ethereum => 1,
            Chain::Polygon => 137,
            Chain::Bsc => 56,
            Chain::Arbitrum => 42161,
            Chain::Optimism => 10,
            Chain::Base => 8453,
            Chain::Avalanche => 43114,
            Chain::Fantom => 250,
            Chain::Gnosis => 100,
            Chain::Custom(id) => *id,
        })
    }

    pub fn name(&self) -> &str {
        match self {
            Chain::Ethereum => "Ethereum",
            Chain::Polygon => "Polygon",
            Chain::Bsc => "BNB Smart Chain",
            Chain::Arbitrum => "Arbitrum One",
            Chain::Optimism => "Optimism",
            Chain::Base => "Base",
            Chain::Avalanche => "Avalanche C-Chain",
            Chain::Fantom => "Fantom Opera",
            Chain::Gnosis => "Gnosis Chain",
            Chain::Custom(_) => "Custom Chain",
        }
    }

    pub fn explorer_url(&self) -> &str {
        match self {
            Chain::Ethereum => "https://etherscan.io",
            Chain::Polygon => "https://polygonscan.com",
            Chain::Bsc => "https://bscscan.com",
            Chain::Arbitrum => "https://arbiscan.io",
            Chain::Optimism => "https://optimistic.etherscan.io",
            Chain::Base => "https://basescan.org",
            Chain::Avalanche => "https://snowtrace.io",
            Chain::Fantom => "https://ftmscan.com",
            Chain::Gnosis => "https://gnosisscan.io",
            Chain::Custom(_) => "",
        }
    }

    pub fn api_url(&self) -> &str {
        match self {
            Chain::Ethereum => "https://api.etherscan.io/api",
            Chain::Polygon => "https://api.polygonscan.com/api",
            Chain::Bsc => "https://api.bscscan.com/api",
            Chain::Arbitrum => "https://api.arbiscan.io/api",
            Chain::Optimism => "https://api-optimistic.etherscan.io/api",
            Chain::Base => "https://api.basescan.org/api",
            Chain::Avalanche => "https://api.snowtrace.io/api",
            Chain::Fantom => "https://api.ftmscan.com/api",
            Chain::Gnosis => "https://api.gnosisscan.io/api",
            Chain::Custom(_) => "",
        }
    }

    pub fn all_supported() -> &'static [Chain] {
        &[
            Chain::Ethereum,
            Chain::Polygon,
            Chain::Bsc,
            Chain::Arbitrum,
            Chain::Optimism,
            Chain::Base,
            Chain::Avalanche,
            Chain::Fantom,
            Chain::Gnosis,
        ]
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for Chain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" | "mainnet" => Ok(Chain::Ethereum),
            "polygon" | "matic" => Ok(Chain::Polygon),
            "bsc" | "binance" | "bnb" => Ok(Chain::Bsc),
            "arbitrum" | "arb" => Ok(Chain::Arbitrum),
            "optimism" | "op" => Ok(Chain::Optimism),
            "base" => Ok(Chain::Base),
            "avalanche" | "avax" => Ok(Chain::Avalanche),
            "fantom" | "ftm" => Ok(Chain::Fantom),
            "gnosis" | "xdai" => Ok(Chain::Gnosis),
            _ => {
                // Try parsing as chain ID
                if let Ok(id) = s.parse::<u64>() {
                    Ok(Chain::Custom(id))
                } else {
                    Err(format!("Unknown chain: {}", s))
                }
            }
        }
    }
}

impl Default for Chain {
    fn default() -> Self {
        Chain::Ethereum
    }
}

/// Wrapper for chain ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainId(pub u64);

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
crates/scpf-types/src/contract.rs
Rust

use serde::{Deserialize, Serialize};
use crate::Chain;

/// Basic contract information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub address: String,
    pub name: Option<String>,
    pub chain: Chain,
}

impl Contract {
    pub fn new(address: impl Into<String>, chain: Chain) -> Self {
        Self {
            address: address.into(),
            name: None,
            chain,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Normalize address to checksummed format
    pub fn normalized_address(&self) -> String {
        // Simple lowercase normalization for now
        // Could implement EIP-55 checksum
        self.address.to_lowercase()
    }

    pub fn explorer_url(&self) -> String {
        format!("{}/address/{}", self.chain.explorer_url(), self.address)
    }
}

/// Extended contract information from explorer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub address: String,
    pub name: String,
    pub compiler_version: String,
    pub optimization_used: bool,
    pub runs: Option<u32>,
    pub constructor_arguments: Option<String>,
    pub evm_version: Option<String>,
    pub library: Option<String>,
    pub license_type: Option<String>,
    pub proxy: Option<String>,
    pub implementation: Option<String>,
}

/// Source code retrieved from explorer or sourcify
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCode {
    /// Full concatenated source code
    pub source: String,
    
    /// Individual source files (for multi-file contracts)
    pub files: Vec<SourceFile>,
    
    /// Contract name
    pub contract_name: String,
    
    /// Compiler version
    pub compiler_version: String,
    
    /// ABI (if available)
    pub abi: Option<String>,
    
    /// Bytecode (if available)
    pub bytecode: Option<String>,
    
    /// Source provider
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub content: String,
}

impl SourceCode {
    pub fn new(source: impl Into<String>, contract_name: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            files: Vec::new(),
            contract_name: contract_name.into(),
            compiler_version: String::new(),
            abi: None,
            bytecode: None,
            provider: String::new(),
        }
    }

    /// Get total source code length
    pub fn len(&self) -> usize {
        self.source.len()
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.source.lines().count()
    }
}
crates/scpf-types/src/match_result.rs
Rust

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::{Chain, Severity};

/// Complete scan match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMatch {
    /// Unique match ID
    pub id: String,
    
    /// Contract address
    pub contract_address: String,
    
    /// Contract name
    pub contract_name: String,
    
    /// Chain where contract was found
    pub chain: Chain,
    
    /// Template ID that matched
    pub template_id: String,
    
    /// Template name
    pub template_name: String,
    
    /// Severity level
    pub severity: Severity,
    
    /// Tags from template
    pub tags: Vec<String>,
    
    /// Match locations in source
    pub locations: Vec<MatchLocation>,
    
    /// Extracted values
    pub extracted: HashMap<String, Vec<String>>,
    
    /// Source code snippet with context
    pub snippet: Option<String>,
    
    /// When the match was found
    pub timestamp: DateTime<Utc>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ScanMatch {
    pub fn new(
        contract_address: impl Into<String>,
        contract_name: impl Into<String>,
        chain: Chain,
        template_id: impl Into<String>,
        template_name: impl Into<String>,
        severity: Severity,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            contract_address: contract_address.into(),
            contract_name: contract_name.into(),
            chain,
            template_id: template_id.into(),
            template_name: template_name.into(),
            severity,
            tags: Vec::new(),
            locations: Vec::new(),
            extracted: HashMap::new(),
            snippet: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_locations(mut self, locations: Vec<MatchLocation>) -> Self {
        self.locations = locations;
        self
    }

    pub fn with_extracted(mut self, extracted: HashMap<String, Vec<String>>) -> Self {
        self.extracted = extracted;
        self
    }

    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn explorer_url(&self) -> String {
        format!("{}/address/{}", self.chain.explorer_url(), self.contract_address)
    }
}

/// Location of a match within source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLocation {
    /// Pattern that matched
    pub pattern: String,
    
    /// Matched text
    pub matched_text: String,
    
    /// Line number (1-indexed)
    pub line: usize,
    
    /// Column number (1-indexed)
    pub column: usize,
    
    /// Byte offset start
    pub byte_start: usize,
    
    /// Byte offset end
    pub byte_end: usize,
    
    /// File path (for multi-file contracts)
    pub file: Option<String>,
}

impl MatchLocation {
    pub fn new(pattern: impl Into<String>, matched_text: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            pattern: pattern.into(),
            matched_text: matched_text.into(),
            line,
            column,
            byte_start: 0,
            byte_end: 0,
            file: None,
        }
    }
}

/// Result from an extractor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Name of the extraction
    pub name: String,
    
    /// Extracted values
    pub values: Vec<String>,
    
    /// Whether this is internal only
    pub internal: bool,
}

impl ExtractionResult {
    pub fn new(name: impl Into<String>, values: Vec<String>) -> Self {
        Self {
            name: name.into(),
            values,
            internal: false,
        }
    }

    pub fn internal(mut self) -> Self {
        self.internal = true;
        self
    }
}
crates/scpf-core/Cargo.toml
toml

[package]
name = "scpf-core"
description = "Core library for Smart Contract Pattern Finder"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[features]
default = ["sqlite", "multi-source"]
sqlite = ["rusqlite"]
multi-source = []
parallel = ["rayon"]

[dependencies]
# Internal crates
scpf-types = { path = "../scpf-types" }

# Async
tokio = { workspace = true }
futures = { workspace = true }
async-trait = { workspace = true }

# HTTP
reqwest = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }

# Error handling
thiserror = { workspace = true }
anyhow = { workspace = true }

# Logging
tracing = { workspace = true }

# Regex
regex = { workspace = true }
fancy-regex = { workspace = true }

# Database
rusqlite = { workspace = true, optional = true }

# Caching
moka = { workspace = true }

# Rate limiting
governor = { workspace = true }

# Retry
backoff = { workspace = true }

# Time
chrono = { workspace = true }

# HTML parsing
scraper = { workspace = true }

# Hashing
xxhash-rust = { workspace = true }
sha2 = { workspace = true }

# URL
url = { workspace = true }

# UUID
uuid = { workspace = true }

# Glob
glob = { workspace = true }
globset = { workspace = true }

# Parallel
rayon = { workspace = true, optional = true }

# Bloom filter
bloomfilter = { workspace = true }

# Once cell
once_cell = { workspace = true }

# Semver
semver = { workspace = true }

# Bytes
bytes = { workspace = true }
hex = { workspace = true }

# Dirs
dirs = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.9"
pretty_assertions = "1.4"
crates/scpf-core/src/lib.rs
Rust

//! # SCPF Core
//! 
//! Core library for Smart Contract Pattern Finder.
//! 
//! This crate provides the fundamental building blocks for scanning smart contracts
//! for patterns, vulnerabilities, and other characteristics.
//! 
//! ## Features
//! 
//! - **Template-based scanning**: Define patterns in YAML templates
//! - **Multi-source support**: Fetch contracts from Etherscan, Sourcify, Blockscout
//! - **Caching**: Memory and persistent caching for efficiency
//! - **Rate limiting**: Built-in rate limiting for API calls
//! - **Parallel processing**: Optional parallel template matching
//! 
//! ## Example
//! 
//! ```rust,no_run
//! use scpf_core::{Scanner, Config, TemplateStore};
//! 
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Load configuration
//!     let config = Config::from_file("config.yaml").await?;
//!     
//!     // Load templates
//!     let templates = TemplateStore::from_directory("templates/").await?;
//!     
//!     // Create scanner
//!     let scanner = Scanner::new(config, templates)?;
//!     
//!     // Scan a contract
//!     let matches = scanner.scan_address("0x1234...", Chain::Ethereum).await?;
//!     
//!     for m in matches {
//!         println!("Found: {} - {}", m.template_id, m.severity);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod config;
pub mod template;
pub mod scanner;
pub mod source;
pub mod cache;
pub mod utils;

// Re-export important types
pub use error::{Error, Result};
pub use config::{Config, ChainConfig, ScannerConfig};
pub use template::{Template, TemplateInfo, TemplateStore, Matcher, Extractor};
pub use scanner::{Scanner, ScanOptions, ScanStats};
pub use source::{SourceProvider, MultiSourceProvider};
pub use cache::{Cache, CacheConfig};

// Re-export types from scpf-types
pub use scpf_types::{
    Severity, Chain, ChainId,
    Contract, ContractInfo, SourceCode,
    ScanMatch, MatchLocation, ExtractionResult,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the library with default configuration
pub fn init() -> Result<()> {
    // Initialize tracing if not already done
    Ok(())
}
crates/scpf-core/src/error.rs
Rust

//! Error types for SCPF Core

use thiserror::Error;

/// Result type alias using SCPF Error
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for SCPF Core
#[derive(Error, Debug)]
pub enum Error {
    // Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Missing configuration: {0}")]
    MissingConfig(String),

    // Template errors
    #[error("Template parse error: {message}")]
    TemplateParse {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Template validation error: {0}")]
    TemplateValidation(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Invalid regex pattern '{pattern}': {message}")]
    InvalidRegex {
        pattern: String,
        message: String,
    },

    // Source errors
    #[error("Source fetch error for {address}: {message}")]
    SourceFetch {
        address: String,
        message: String,
    },

    #[error("Source not verified: {0}")]
    SourceNotVerified(String),

    #[error("Rate limited by {provider}")]
    RateLimited {
        provider: String,
    },

    // Cache errors
    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Database error: {0}")]
    Database(String),

    // Scanner errors
    #[error("Scan error: {0}")]
    Scan(String),

    #[error("Scanner not initialized")]
    NotInitialized,

    // Network errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Request timeout")]
    Timeout,

    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // Serialization errors
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    // Chain errors
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),

    #[error("Chain not configured: {0}")]
    ChainNotConfigured(String),

    #[error("Missing API key for chain: {0}")]
    MissingApiKey(String),

    // Generic errors
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("{0}")]
    Other(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl Error {
    pub fn config(msg: impl Into<String>) -> Self {
        Error::Config(msg.into())
    }

    pub fn template_parse(msg: impl Into<String>) -> Self {
        Error::TemplateParse {
            message: msg.into(),
            source: None,
        }
    }

    pub fn template_validation(msg: impl Into<String>) -> Self {
        Error::TemplateValidation(msg.into())
    }

    pub fn source_fetch(address: impl Into<String>, msg: impl Into<String>) -> Self {
        Error::SourceFetch {
            address: address.into(),
            message: msg.into(),
        }
    }

    pub fn invalid_regex(pattern: impl Into<String>, msg: impl Into<String>) -> Self {
        Error::InvalidRegex {
            pattern: pattern.into(),
            message: msg.into(),
        }
    }

    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Error::RateLimited { .. })
    }

    pub fn is_timeout(&self) -> bool {
        matches!(self, Error::Timeout)
    }

    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Error::RateLimited { .. }
                | Error::Timeout
                | Error::Network(_)
        )
    }
}

// Conversion from rusqlite errors
#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Database(e.to_string())
    }
}

impl From<regex::Error> for Error {
    fn from(e: regex::Error) -> Self {
        Error::InvalidRegex {
            pattern: String::new(),
            message: e.to_string(),
        }
    }
}

impl From<fancy_regex::Error> for Error {
    fn from(e: fancy_regex::Error) -> Self {
        Error::InvalidRegex {
            pattern: String::new(),
            message: e.to_string(),
        }
    }
}

impl From<glob::PatternError> for Error {
    fn from(e: glob::PatternError) -> Self {
        Error::Config(format!("Invalid glob pattern: {}", e))
    }
}
crates/scpf-core/src/config/mod.rs
Rust

//! Configuration management

mod chain;
mod settings;

pub use chain::ChainConfig;
pub use settings::{Config, ScannerConfig, CacheConfig as ConfigCacheSettings, OutputConfig};

use crate::error::{Error, Result};
use crate::Chain;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tokio::fs;

impl Config {
    /// Load configuration from file
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| Error::config(format!("Failed to read config file: {}", e)))?;
        
        let config: Self = serde_yaml::from_str(&content)?;
        config.validate()?;
        
        Ok(config)
    }

    /// Load configuration with environment variable overrides
    pub async fn from_file_with_env(path: impl AsRef<Path>) -> Result<Self> {
        let mut config = Self::from_file(path).await?;
        config.apply_env_overrides();
        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        // Apply chain-specific API keys from environment
        for (chain_name, chain_config) in &mut self.chains {
            if chain_config.api_key.is_none() {
                chain_config.api_key = discover_api_key(chain_name);
            }
        }

        // Apply other environment overrides
        if let Ok(val) = std::env::var("SCPF_CONCURRENCY") {
            if let Ok(v) = val.parse() {
                self.scanner.concurrency = v;
            }
        }

        if let Ok(val) = std::env::var("SCPF_RATE_LIMIT") {
            if let Ok(v) = val.parse() {
                self.scanner.rate_limit = v;
            }
        }

        if let Ok(val) = std::env::var("SCPF_TIMEOUT") {
            if let Ok(v) = val.parse() {
                self.scanner.timeout_seconds = v;
            }
        }
    }

    /// Validate configuration
    fn validate(&self) -> Result<()> {
        if self.scanner.concurrency == 0 {
            return Err(Error::config("Concurrency must be greater than 0"));
        }

        if self.scanner.rate_limit <= 0.0 {
            return Err(Error::config("Rate limit must be positive"));
        }

        if self.scanner.timeout_seconds == 0 {
            return Err(Error::config("Timeout must be greater than 0"));
        }

        Ok(())
    }

    /// Get chain configuration by name
    pub fn get_chain(&self, chain: &Chain) -> Option<&ChainConfig> {
        let name = match chain {
            Chain::Ethereum => "ethereum",
            Chain::Polygon => "polygon",
            Chain::Bsc => "bsc",
            Chain::Arbitrum => "arbitrum",
            Chain::Optimism => "optimism",
            Chain::Base => "base",
            Chain::Avalanche => "avalanche",
            Chain::Fantom => "fantom",
            Chain::Gnosis => "gnosis",
            Chain::Custom(_) => return None,
        };
        self.chains.get(name)
    }

    /// Get or create chain configuration
    pub fn get_or_create_chain(&mut self, chain: &Chain) -> &ChainConfig {
        let name = chain.to_string().to_lowercase();
        
        self.chains.entry(name).or_insert_with(|| {
            ChainConfig {
                name: chain.name().to_string(),
                api_url: chain.api_url().to_string(),
                explorer_url: chain.explorer_url().to_string(),
                verified_url: format!("{}/contractsVerified", chain.explorer_url()),
                api_key: discover_api_key(&chain.to_string().to_lowercase()),
                rate_limit: 5.0,
                enabled: true,
            }
        })
    }

    /// Get the data directory
    pub fn data_dir(&self) -> PathBuf {
        self.data_dir.clone().unwrap_or_else(|| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("scpf")
        })
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> PathBuf {
        self.data_dir().join("cache")
    }

    /// Save configuration to file
    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        fs::write(path, content).await?;
        Ok(())
    }
}

/// Discover API key from environment variables
fn discover_api_key(chain: &str) -> Option<String> {
    let env_vars = match chain.to_lowercase().as_str() {
        "ethereum" | "eth" => vec![
            "ETHERSCAN_API_KEY",
            "ETHERSCAN_KEY",
            "ETH_API_KEY",
        ],
        "polygon" | "matic" => vec![
            "POLYGONSCAN_API_KEY",
            "POLYGON_API_KEY",
        ],
        "bsc" | "binance" | "bnb" => vec![
            "BSCSCAN_API_KEY",
            "BSC_API_KEY",
        ],
        "arbitrum" | "arb" => vec![
            "ARBISCAN_API_KEY",
            "ARBITRUM_API_KEY",
        ],
        "optimism" | "op" => vec![
            "OPTIMISTIC_ETHERSCAN_API_KEY",
            "OPTIMISM_API_KEY",
        ],
        "base" => vec![
            "BASESCAN_API_KEY",
            "BASE_API_KEY",
        ],
        "avalanche" | "avax" => vec![
            "SNOWTRACE_API_KEY",
            "AVALANCHE_API_KEY",
        ],
        "fantom" | "ftm" => vec![
            "FTMSCAN_API_KEY",
            "FANTOM_API_KEY",
        ],
        "gnosis" | "xdai" => vec![
            "GNOSISSCAN_API_KEY",
            "GNOSIS_API_KEY",
        ],
        _ => vec![],
    };

    for var in env_vars {
        if let Ok(key) = std::env::var(var) {
            if !key.is_empty() {
                return Some(key);
            }
        }
    }

    // Try loading from keys file
    load_api_key_from_file(chain)
}

/// Load API key from ~/.scpf/keys.yaml
fn load_api_key_from_file(chain: &str) -> Option<String> {
    let keys_path = dirs::home_dir()?.join(".scpf").join("keys.yaml");
    
    if !keys_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&keys_path).ok()?;
    let keys: HashMap<String, String> = serde_yaml::from_str(&content).ok()?;
    
    keys.get(chain).cloned()
}
crates/scpf-core/src/config/chain.rs
Rust

use serde::{Deserialize, Serialize};

/// Configuration for a specific blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Display name
    pub name: String,
    
    /// API URL for source fetching
    pub api_url: String,
    
    /// Block explorer URL
    pub explorer_url: String,
    
    /// URL for verified contracts list
    pub verified_url: String,
    
    /// API key (optional, can be from environment)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    
    /// Rate limit in requests per second
    #[serde(default = "default_rate_limit")]
    pub rate_limit: f64,
    
    /// Whether this chain is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_rate_limit() -> f64 {
    5.0
}

fn default_true() -> bool {
    true
}

impl ChainConfig {
    pub fn has_api_key(&self) -> bool {
        self.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false)
    }

    pub fn api_key_or_err(&self) -> crate::Result<&str> {
        self.api_key
            .as_deref()
            .filter(|k| !k.is_empty())
            .ok_or_else(|| crate::Error::MissingApiKey(self.name.clone()))
    }
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            name: "Ethereum".to_string(),
            api_url: "https://api.etherscan.io/api".to_string(),
            explorer_url: "https://etherscan.io".to_string(),
            verified_url: "https://etherscan.io/contractsVerified".to_string(),
            api_key: None,
            rate_limit: 5.0,
            enabled: true,
        }
    }
}
crates/scpf-core/src/config/settings.rs
Rust

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::ChainConfig;
use crate::Chain;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Version of the config file format
    #[serde(default = "default_version")]
    pub version: String,

    /// Template directories to load
    #[serde(default)]
    pub template_paths: Vec<PathBuf>,

    /// Chain configurations
    #[serde(default = "default_chains")]
    pub chains: HashMap<String, ChainConfig>,

    /// Scanner settings
    #[serde(default)]
    pub scanner: ScannerConfig,

    /// Output settings
    #[serde(default)]
    pub output: OutputConfig,

    /// Cache settings
    #[serde(default)]
    pub cache: CacheConfig,

    /// Scan profiles
    #[serde(default)]
    pub profiles: HashMap<String, ScanProfile>,

    /// Data directory override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<PathBuf>,
}

fn default_version() -> String {
    "1.0".to_string()
}

fn default_chains() -> HashMap<String, ChainConfig> {
    let mut chains = HashMap::new();
    
    for chain in Chain::all_supported() {
        let name = match chain {
            Chain::Ethereum => "ethereum",
            Chain::Polygon => "polygon",
            Chain::Bsc => "bsc",
            Chain::Arbitrum => "arbitrum",
            Chain::Optimism => "optimism",
            Chain::Base => "base",
            Chain::Avalanche => "avalanche",
            Chain::Fantom => "fantom",
            Chain::Gnosis => "gnosis",
            Chain::Custom(_) => continue,
        };
        
        chains.insert(name.to_string(), ChainConfig {
            name: chain.name().to_string(),
            api_url: chain.api_url().to_string(),
            explorer_url: chain.explorer_url().to_string(),
            verified_url: format!("{}/contractsVerified", chain.explorer_url()),
            api_key: None,
            rate_limit: 5.0,
            enabled: true,
        });
    }
    
    chains
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: default_version(),
            template_paths: vec![PathBuf::from("templates")],
            chains: default_chains(),
            scanner: ScannerConfig::default(),
            output: OutputConfig::default(),
            cache: CacheConfig::default(),
            profiles: default_profiles(),
            data_dir: None,
        }
    }
}

/// Scanner settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    /// Number of concurrent scans
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

    /// Rate limit (requests per second)
    #[serde(default = "default_rate_limit")]
    pub rate_limit: f64,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Maximum retries for failed requests
    #[serde(default = "default_retries")]
    pub max_retries: u32,

    /// Delay between retries in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,

    /// Maximum contracts to scan per session
    #[serde(default)]
    pub max_contracts: Option<usize>,

    /// Stop after first match
    #[serde(default)]
    pub stop_at_first: bool,

    /// Maximum matches per template
    #[serde(default)]
    pub max_matches_per_template: Option<usize>,

    /// Use bytecode pre-filtering
    #[serde(default = "default_true")]
    pub prefilter: bool,
}

fn default_concurrency() -> usize { 10 }
fn default_rate_limit() -> f64 { 5.0 }
fn default_timeout() -> u64 { 30 }
fn default_retries() -> u32 { 3 }
fn default_retry_delay() -> u64 { 1000 }
fn default_true() -> bool { true }

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            concurrency: default_concurrency(),
            rate_limit: default_rate_limit(),
            timeout_seconds: default_timeout(),
            max_retries: default_retries(),
            retry_delay_ms: default_retry_delay(),
            max_contracts: None,
            stop_at_first: false,
            max_matches_per_template: None,
            prefilter: true,
        }
    }
}

/// Output settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format
    #[serde(default = "default_format")]
    pub format: String,

    /// Include source code in output
    #[serde(default)]
    pub include_source: bool,

    /// Number of context lines around matches
    #[serde(default = "default_context")]
    pub context_lines: usize,

    /// Colorize output
    #[serde(default = "default_true")]
    pub colorize: bool,

    /// Show progress bar
    #[serde(default = "default_true")]
    pub progress: bool,

    /// Verbosity level (0=quiet, 1=normal, 2=verbose, 3=debug)
    #[serde(default = "default_verbosity")]
    pub verbosity: u8,
}

fn default_format() -> String { "console".to_string() }
fn default_context() -> usize { 3 }
fn default_verbosity() -> u8 { 1 }

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            include_source: false,
            context_lines: default_context(),
            colorize: true,
            progress: true,
            verbosity: default_verbosity(),
        }
    }
}

/// Cache settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Use persistent (SQLite) cache
    #[serde(default = "default_true")]
    pub persistent: bool,

    /// Maximum entries in memory cache
    #[serde(default = "default_max_entries")]
    pub max_entries: u64,

    /// TTL in seconds
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,

    /// Cache source code
    #[serde(default = "default_true")]
    pub cache_source: bool,

    /// Cache scan results
    #[serde(default = "default_true")]
    pub cache_results: bool,
}

fn default_max_entries() -> u64 { 100_000 }
fn default_ttl() -> u64 { 86400 } // 24 hours

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            persistent: true,
            max_entries: default_max_entries(),
            ttl_seconds: default_ttl(),
            cache_source: true,
            cache_results: true,
        }
    }
}

/// Scan profile for quick selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProfile {
    /// Profile description
    pub description: String,

    /// Tags to include
    #[serde(default)]
    pub include_tags: Vec<String>,

    /// Tags to exclude
    #[serde(default)]
    pub exclude_tags: Vec<String>,

    /// Severities to include
    #[serde(default)]
    pub severities: Vec<String>,

    /// Maximum templates to use
    #[serde(default)]
    pub max_templates: Option<usize>,
}

fn default_profiles() -> HashMap<String, ScanProfile> {
    let mut profiles = HashMap::new();

    profiles.insert("security-audit".to_string(), ScanProfile {
        description: "Full security audit scan".to_string(),
        include_tags: vec![
            "vulnerability".to_string(),
            "security".to_string(),
        ],
        exclude_tags: vec!["informational".to_string()],
        severities: vec![
            "critical".to_string(),
            "high".to_string(),
            "medium".to_string(),
        ],
        max_templates: None,
    });

    profiles.insert("quick-check".to_string(), ScanProfile {
        description: "Fast critical issues only".to_string(),
        include_tags: vec!["critical".to_string()],
        exclude_tags: vec![],
        severities: vec!["critical".to_string(), "high".to_string()],
        max_templates: Some(10),
    });

    profiles.insert("token-analysis".to_string(), ScanProfile {
        description: "Token contract analysis".to_string(),
        include_tags: vec![
            "erc20".to_string(),
            "erc721".to_string(),
            "token".to_string(),
        ],
        exclude_tags: vec![],
        severities: vec![],
        max_templates: None,
    });

    profiles.insert("defi-scan".to_string(), ScanProfile {
        description: "DeFi protocol patterns".to_string(),
        include_tags: vec![
            "defi".to_string(),
            "lending".to_string(),
            "dex".to_string(),
            "staking".to_string(),
        ],
        exclude_tags: vec![],
        severities: vec![],
        max_templates: None,
    });

    profiles
}
crates/scpf-core/src/template/mod.rs
Rust

//! Template system for pattern matching

mod types;
mod parser;
mod matcher;
mod extractor;
mod validator;
mod store;

pub use types::{Template, TemplateInfo, Classification, MatchCondition};
pub use parser::TemplateParser;
pub use matcher::{Matcher, MatcherType, MatchPart, MatcherResult};
pub use extractor::{Extractor, ExtractorType};
pub use validator::TemplateValidator;
pub use store::TemplateStore;
crates/scpf-core/src/template/types.rs
Rust

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::Severity;

use super::{Matcher, Extractor};

/// Complete template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Unique template identifier
    pub id: String,

    /// Template metadata
    pub info: TemplateInfo,

    /// Matchers to apply
    #[serde(default)]
    pub matchers: Vec<Matcher>,

    /// Condition between matchers (and/or)
    #[serde(default)]
    pub matchers_condition: MatchCondition,

    /// Extractors for capturing values
    #[serde(default)]
    pub extractors: Vec<Extractor>,

    /// Template variables
    #[serde(default)]
    pub variables: HashMap<String, String>,

    /// Required function selectors (for bytecode prefiltering)
    #[serde(default)]
    pub required_selectors: Vec<String>,

    /// File path (set during loading)
    #[serde(skip)]
    pub file_path: Option<PathBuf>,

    /// Whether template is compiled
    #[serde(skip)]
    pub compiled: bool,
}

impl Template {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.info.name
    }

    pub fn severity(&self) -> Severity {
        self.info.severity
    }

    pub fn tags(&self) -> &[String] {
        &self.info.tags
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.info.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
    }

    pub fn has_any_tag(&self, tags: &[String]) -> bool {
        tags.iter().any(|t| self.has_tag(t))
    }
}

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Human-readable name
    pub name: String,

    /// Author
    #[serde(default)]
    pub author: String,

    /// Description
    #[serde(default)]
    pub description: String,

    /// Severity level
    #[serde(default)]
    pub severity: Severity,

    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// References (URLs)
    #[serde(default)]
    pub reference: Vec<String>,

    /// Classification (CWE, SWC, etc.)
    #[serde(default)]
    pub classification: Option<Classification>,

    /// Remediation advice
    #[serde(default)]
    pub remediation: Option<String>,

    /// Template version
    #[serde(default)]
    pub version: Option<String>,
}

/// Classification information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Classification {
    /// CWE ID
    #[serde(rename = "cwe-id", default)]
    pub cwe_id: Option<String>,

    /// CVSS score
    #[serde(rename = "cvss-score", default)]
    pub cvss_score: Option<f64>,

    /// SWC ID (Smart Contract Weakness Classification)
    #[serde(default)]
    pub swc: Option<String>,

    /// CVE ID
    #[serde(default)]
    pub cve: Option<String>,
}

/// Condition for combining multiple matchers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchCondition {
    #[default]
    Or,
    And,
}

impl MatchCondition {
    pub fn evaluate(&self, results: &[bool]) -> bool {
        if results.is_empty() {
            return false;
        }

        match self {
            MatchCondition::Or => results.iter().any(|&r| r),
            MatchCondition::And => results.iter().all(|&r| r),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_condition() {
        assert!(MatchCondition::Or.evaluate(&[true, false]));
        assert!(!MatchCondition::Or.evaluate(&[false, false]));
        assert!(MatchCondition::And.evaluate(&[true, true]));
        assert!(!MatchCondition::And.evaluate(&[true, false]));
    }
}
crates/scpf-core/src/template/matcher.rs
Rust

use std::collections::HashMap;
use std::sync::Arc;

use fancy_regex::Regex as FancyRegex;
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::MatchLocation;

/// Matcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Matcher {
    /// Matcher type
    #[serde(rename = "type")]
    pub matcher_type: MatcherType,

    /// Name for identification
    #[serde(default)]
    pub name: Option<String>,

    /// Words to match (for word matcher)
    #[serde(default)]
    pub words: Vec<String>,

    /// Regex patterns
    #[serde(default)]
    pub regex: Vec<String>,

    /// DSL expressions
    #[serde(default)]
    pub dsl: Vec<String>,

    /// Binary patterns (hex)
    #[serde(default)]
    pub binary: Vec<String>,

    /// Part of source to match
    #[serde(default)]
    pub part: MatchPart,

    /// Condition for multiple patterns
    #[serde(default)]
    pub condition: MatchCondition,

    /// Case insensitive
    #[serde(default)]
    pub case_insensitive: bool,

    /// Negative match (must NOT match)
    #[serde(default)]
    pub negative: bool,

    /// Internal (don't include in output)
    #[serde(default)]
    pub internal: bool,

    /// Compiled patterns
    #[serde(skip)]
    compiled: OnceCell<CompiledMatcher>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatcherType {
    #[default]
    Word,
    Regex,
    Dsl,
    Binary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchPart {
    #[default]
    Body,
    All,
    Functions,
    Events,
    Modifiers,
    Variables,
    Imports,
    Comments,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchCondition {
    #[default]
    Or,
    And,
}

/// Compiled matcher patterns for performance
#[derive(Debug)]
struct CompiledMatcher {
    regexes: Vec<CompiledRegex>,
    word_patterns: Vec<String>,
}

#[derive(Debug)]
enum CompiledRegex {
    Standard(Regex),
    Fancy(FancyRegex),
}

impl CompiledRegex {
    fn is_match(&self, text: &str) -> bool {
        match self {
            CompiledRegex::Standard(r) => r.is_match(text),
            CompiledRegex::Fancy(r) => r.is_match(text).unwrap_or(false),
        }
    }

    fn find_iter<'a>(&'a self, text: &'a str) -> Vec<(usize, usize, String)> {
        match self {
            CompiledRegex::Standard(r) => r
                .find_iter(text)
                .map(|m| (m.start(), m.end(), m.as_str().to_string()))
                .collect(),
            CompiledRegex::Fancy(r) => r
                .find_iter(text)
                .filter_map(|m| m.ok())
                .map(|m| (m.start(), m.end(), m.as_str().to_string()))
                .collect(),
        }
    }
}

/// Result of a match operation
#[derive(Debug, Clone)]
pub struct MatcherResult {
    pub matched: bool,
    pub matcher_name: Option<String>,
    pub locations: Vec<MatchLocation>,
}

impl Matcher {
    /// Compile the matcher patterns
    pub fn compile(&self) -> Result<()> {
        self.compiled.get_or_try_init(|| {
            let mut regexes = Vec::new();
            
            for pattern in &self.regex {
                let compiled = self.compile_regex(pattern)?;
                regexes.push(compiled);
            }

            let word_patterns = if self.case_insensitive {
                self.words.iter().map(|w| w.to_lowercase()).collect()
            } else {
                self.words.clone()
            };

            Ok(CompiledMatcher { regexes, word_patterns })
        })?;

        Ok(())
    }

    fn compile_regex(&self, pattern: &str) -> Result<CompiledRegex> {
        let needs_fancy = pattern.contains("(?=")
            || pattern.contains("(?!")
            || pattern.contains("(?<=")
            || pattern.contains("(?<!");

        let pattern = if self.case_insensitive && !pattern.starts_with("(?i)") {
            format!("(?i){}", pattern)
        } else {
            pattern.to_string()
        };

        if needs_fancy {
            FancyRegex::new(&pattern)
                .map(CompiledRegex::Fancy)
                .map_err(|e| Error::invalid_regex(pattern, e.to_string()))
        } else {
            Regex::new(&pattern)
                .map(CompiledRegex::Standard)
                .map_err(|e| Error::invalid_regex(pattern, e.to_string()))
        }
    }

    /// Execute the matcher against source code
    pub fn execute(&self, source: &str, metadata: &HashMap<String, String>) -> Result<MatcherResult> {
        // Ensure compiled
        self.compile()?;

        let compiled = self.compiled.get().unwrap();
        let target = self.get_match_part(source);

        let (matched, locations) = match self.matcher_type {
            MatcherType::Word => self.match_words(&compiled.word_patterns, target),
            MatcherType::Regex => self.match_regex(&compiled.regexes, target),
            MatcherType::Dsl => self.match_dsl(target, metadata),
            MatcherType::Binary => self.match_binary(target),
        };

        let final_matched = if self.negative { !matched } else { matched };

        Ok(MatcherResult {
            matched: final_matched,
            matcher_name: self.name.clone(),
            locations,
        })
    }

    fn get_match_part<'a>(&self, source: &'a str) -> &'a str {
        // For now, return full source
        // Could implement Solidity-aware parsing for specific parts
        source
    }

    fn match_words(&self, patterns: &[String], source: &str) -> (bool, Vec<MatchLocation>) {
        let mut locations = Vec::new();
        let search_source = if self.case_insensitive {
            source.to_lowercase()
        } else {
            source.to_string()
        };

        for word in patterns {
            let mut start = 0;
            while let Some(pos) = search_source[start..].find(word.as_str()) {
                let byte_start = start + pos;
                let byte_end = byte_start + word.len();
                let (line, col) = byte_to_line_col(source, byte_start);

                locations.push(MatchLocation {
                    pattern: word.clone(),
                    matched_text: source[byte_start..byte_end].to_string(),
                    line,
                    column: col,
                    byte_start,
                    byte_end,
                    file: None,
                });

                start = byte_start + 1;
            }
        }

        let matched = match self.condition {
            MatchCondition::Or => !locations.is_empty(),
            MatchCondition::And => {
                let found: std::collections::HashSet<_> =
                    locations.iter().map(|l| &l.pattern).collect();
                patterns.iter().all(|w| found.contains(w))
            }
        };

        (matched, locations)
    }

    fn match_regex(&self, regexes: &[CompiledRegex], source: &str) -> (bool, Vec<MatchLocation>) {
        let mut locations = Vec::new();

        for (i, regex) in regexes.iter().enumerate() {
            for (start, end, text) in regex.find_iter(source) {
                let (line, col) = byte_to_line_col(source, start);
                let pattern = self.regex.get(i).cloned().unwrap_or_default();

                locations.push(MatchLocation {
                    pattern,
                    matched_text: text,
                    line,
                    column: col,
                    byte_start: start,
                    byte_end: end,
                    file: None,
                });
            }
        }

        let matched = match self.condition {
            MatchCondition::Or => !locations.is_empty(),
            MatchCondition::And => regexes.iter().all(|r| r.is_match(source)),
        };

        (matched, locations)
    }

    fn match_dsl(&self, source: &str, metadata: &HashMap<String, String>) -> (bool, Vec<MatchLocation>) {
        let mut all_matched = true;
        let mut any_matched = false;

        for expr in &self.dsl {
            let result = self.evaluate_dsl(expr, source, metadata);
            if result {
                any_matched = true;
            } else {
                all_matched = false;
            }
        }

        let matched = match self.condition {
            MatchCondition::Or => any_matched,
            MatchCondition::And => all_matched,
        };

        (matched, vec![])
    }

    fn evaluate_dsl(&self, expr: &str, source: &str, metadata: &HashMap<String, String>) -> bool {
        let expr = expr.trim();

        // contains(body, "text")
        if expr.starts_with("contains(") && expr.ends_with(')') {
            let inner = &expr[9..expr.len() - 1];
            let parts: Vec<&str> = inner.splitn(2, ',').collect();
            if parts.len() == 2 {
                let haystack = self.resolve_var(parts[0].trim(), source, metadata);
                Continuing from the matcher.rs file and completing the full implementation:

crates/scpf-core/src/template/matcher.rs (continued)
Rust

                let needle = parts[1].trim().trim_matches('"');
                return haystack.contains(needle);
            }
        }

        // starts_with(body, "text")
        if expr.starts_with("starts_with(") && expr.ends_with(')') {
            let inner = &expr[12..expr.len() - 1];
            let parts: Vec<&str> = inner.splitn(2, ',').collect();
            if parts.len() == 2 {
                let haystack = self.resolve_var(parts[0].trim(), source, metadata);
                let needle = parts[1].trim().trim_matches('"');
                return haystack.starts_with(needle);
            }
        }

        // len(body) > 1000
        if expr.contains("len(") {
            if let Some(result) = self.evaluate_len_expr(expr, source, metadata) {
                return result;
            }
        }

        // regex_match(body, "pattern")
        if expr.starts_with("regex_match(") && expr.ends_with(')') {
            let inner = &expr[12..expr.len() - 1];
            let parts: Vec<&str> = inner.splitn(2, ',').collect();
            if parts.len() == 2 {
                let haystack = self.resolve_var(parts[0].trim(), source, metadata);
                let pattern = parts[1].trim().trim_matches('"');
                if let Ok(re) = Regex::new(pattern) {
                    return re.is_match(&haystack);
                }
            }
        }

        // count(body, "pattern") > N
        if expr.contains("count(") {
            if let Some(result) = self.evaluate_count_expr(expr, source, metadata) {
                return result;
            }
        }

        false
    }

    fn resolve_var(&self, var: &str, source: &str, metadata: &HashMap<String, String>) -> String {
        match var {
            "body" | "source" => source.to_string(),
            _ => metadata.get(var).cloned().unwrap_or_default(),
        }
    }

    fn evaluate_len_expr(&self, expr: &str, source: &str, metadata: &HashMap<String, String>) -> Option<bool> {
        let re = Regex::new(r"len\((\w+)\)\s*(>|<|>=|<=|==|!=)\s*(\d+)").ok()?;
        let caps = re.captures(expr)?;
        
        let var = caps.get(1)?.as_str();
        let op = caps.get(2)?.as_str();
        let value: usize = caps.get(3)?.as_str().parse().ok()?;
        
        let haystack = self.resolve_var(var, source, metadata);
        let len = haystack.len();
        
        Some(match op {
            ">" => len > value,
            "<" => len < value,
            ">=" => len >= value,
            "<=" => len <= value,
            "==" => len == value,
            "!=" => len != value,
            _ => false,
        })
    }

    fn evaluate_count_expr(&self, expr: &str, source: &str, metadata: &HashMap<String, String>) -> Option<bool> {
        let re = Regex::new(r#"count\((\w+),\s*"([^"]+)"\)\s*(>|<|>=|<=|==|!=)\s*(\d+)"#).ok()?;
        let caps = re.captures(expr)?;
        
        let var = caps.get(1)?.as_str();
        let pattern = caps.get(2)?.as_str();
        let op = caps.get(3)?.as_str();
        let value: usize = caps.get(4)?.as_str().parse().ok()?;
        
        let haystack = self.resolve_var(var, source, metadata);
        let count = haystack.matches(pattern).count();
        
        Some(match op {
            ">" => count > value,
            "<" => count < value,
            ">=" => count >= value,
            "<=" => count <= value,
            "==" => count == value,
            "!=" => count != value,
            _ => false,
        })
    }

    fn match_binary(&self, source: &str) -> (bool, Vec<MatchLocation>) {
        let mut locations = Vec::new();

        for hex_pattern in &self.binary {
            let pattern = hex_pattern.replace(" ", "").replace("0x", "");
            if let Ok(bytes) = hex::decode(&pattern) {
                let pattern_str = String::from_utf8_lossy(&bytes);
                if source.contains(&*pattern_str) {
                    locations.push(MatchLocation {
                        pattern: hex_pattern.clone(),
                        matched_text: pattern_str.to_string(),
                        line: 0,
                        column: 0,
                        byte_start: 0,
                        byte_end: 0,
                        file: None,
                    });
                }
            }
        }

        let matched = !locations.is_empty();
        (matched, locations)
    }
}

/// Convert byte offset to line and column numbers
fn byte_to_line_col(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    
    for (i, ch) in source.char_indices() {
        if i >= byte_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    
    (line, col)
}

impl Default for Matcher {
    fn default() -> Self {
        Self {
            matcher_type: MatcherType::default(),
            name: None,
            words: Vec::new(),
            regex: Vec::new(),
            dsl: Vec::new(),
            binary: Vec::new(),
            part: MatchPart::default(),
            condition: MatchCondition::default(),
            case_insensitive: false,
            negative: false,
            internal: false,
            compiled: OnceCell::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_matcher() {
        let matcher = Matcher {
            matcher_type: MatcherType::Word,
            words: vec!["transfer".to_string(), "approve".to_string()],
            ..Default::default()
        };

        let source = "function transfer(address to, uint amount) public {}";
        let result = matcher.execute(source, &HashMap::new()).unwrap();
        
        assert!(result.matched);
        assert!(!result.locations.is_empty());
    }

    #[test]
    fn test_regex_matcher() {
        let matcher = Matcher {
            matcher_type: MatcherType::Regex,
            regex: vec![r"function\s+\w+\s*\(".to_string()],
            ..Default::default()
        };

        let source = "function transfer(address to) public {}";
        let result = matcher.execute(source, &HashMap::new()).unwrap();
        
        assert!(result.matched);
    }

    #[test]
    fn test_negative_matcher() {
        let matcher = Matcher {
            matcher_type: MatcherType::Word,
            words: vec!["selfdestruct".to_string()],
            negative: true,
            ..Default::default()
        };

        let source = "function transfer(address to) public {}";
        let result = matcher.execute(source, &HashMap::new()).unwrap();
        
        assert!(result.matched); // No selfdestruct = match (negative)
    }

    #[test]
    fn test_case_insensitive() {
        let matcher = Matcher {
            matcher_type: MatcherType::Word,
            words: vec!["TRANSFER".to_string()],
            case_insensitive: true,
            ..Default::default()
        };

        let source = "function transfer() {}";
        let result = matcher.execute(source, &HashMap::new()).unwrap();
        
        assert!(result.matched);
    }
}
crates/scpf-core/src/template/extractor.rs
Rust

use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use once_cell::sync::OnceCell;

use crate::error::{Error, Result};
use crate::ExtractionResult;

/// Extractor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extractor {
    /// Extractor type
    #[serde(rename = "type")]
    pub extractor_type: ExtractorType,

    /// Name for the extraction
    pub name: String,

    /// Regex patterns for extraction
    #[serde(default)]
    pub regex: Vec<String>,

    /// Capture group to use (0 = full match)
    #[serde(default)]
    pub group: usize,

    /// JSON path for JSON extraction
    #[serde(default)]
    pub json: Option<String>,

    /// XPath for HTML/XML extraction
    #[serde(default)]
    pub xpath: Option<String>,

    /// DSL expression for extraction
    #[serde(default)]
    pub dsl: Option<String>,

    /// Part of source to extract from
    #[serde(default)]
    pub part: ExtractorPart,

    /// Internal only (don't show in output)
    #[serde(default)]
    pub internal: bool,

    /// Compiled patterns
    #[serde(skip)]
    compiled: OnceCell<Vec<Regex>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtractorType {
    #[default]
    Regex,
    Json,
    Xpath,
    Dsl,
    Kval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtractorPart {
    #[default]
    Body,
    All,
    Functions,
    Events,
    Variables,
}

impl Extractor {
    /// Compile extractor patterns
    pub fn compile(&self) -> Result<()> {
        self.compiled.get_or_try_init(|| {
            let mut compiled = Vec::new();
            for pattern in &self.regex {
                let re = Regex::new(pattern)
                    .map_err(|e| Error::invalid_regex(pattern, e.to_string()))?;
                compiled.push(re);
            }
            Ok(compiled)
        })?;
        Ok(())
    }

    /// Execute the extractor against source code
    pub fn execute(&self, source: &str, _metadata: &HashMap<String, String>) -> Result<ExtractionResult> {
        self.compile()?;

        let values = match self.extractor_type {
            ExtractorType::Regex => self.extract_regex(source),
            ExtractorType::Dsl => self.extract_dsl(source),
            ExtractorType::Json => self.extract_json(source),
            ExtractorType::Kval => self.extract_kval(source),
            ExtractorType::Xpath => vec![], // Not typically used for Solidity
        };

        let mut result = ExtractionResult::new(&self.name, values);
        if self.internal {
            result = result.internal();
        }

        Ok(result)
    }

    fn extract_regex(&self, source: &str) -> Vec<String> {
        let compiled = self.compiled.get().expect("Not compiled");
        let mut values = Vec::new();

        for re in compiled {
            for caps in re.captures_iter(source) {
                if let Some(m) = caps.get(self.group) {
                    values.push(m.as_str().to_string());
                } else if let Some(m) = caps.get(0) {
                    values.push(m.as_str().to_string());
                }
            }
        }

        values
    }

    fn extract_dsl(&self, source: &str) -> Vec<String> {
        let mut values = Vec::new();

        if let Some(dsl) = &self.dsl {
            // Extract functions: functions()
            if dsl == "functions()" || dsl == "function_names()" {
                let re = Regex::new(r"function\s+(\w+)\s*\(").unwrap();
                for caps in re.captures_iter(source) {
                    if let Some(m) = caps.get(1) {
                        values.push(m.as_str().to_string());
                    }
                }
            }
            // Extract events: events()
            else if dsl == "events()" || dsl == "event_names()" {
                let re = Regex::new(r"event\s+(\w+)\s*\(").unwrap();
                for caps in re.captures_iter(source) {
                    if let Some(m) = caps.get(1) {
                        values.push(m.as_str().to_string());
                    }
                }
            }
            // Extract modifiers: modifiers()
            else if dsl == "modifiers()" || dsl == "modifier_names()" {
                let re = Regex::new(r"modifier\s+(\w+)\s*[\(\{]").unwrap();
                for caps in re.captures_iter(source) {
                    if let Some(m) = caps.get(1) {
                        values.push(m.as_str().to_string());
                    }
                }
            }
            // Extract state variables: state_variables()
            else if dsl == "state_variables()" {
                let re = Regex::new(r"^\s*(mapping|uint|int|address|bool|string|bytes)\S*\s+(?:public\s+|private\s+|internal\s+)?(\w+)").unwrap();
                for caps in re.captures_iter(source) {
                    if let Some(m) = caps.get(2) {
                        values.push(m.as_str().to_string());
                    }
                }
            }
            // Extract imports: imports()
            else if dsl == "imports()" {
                let re = Regex::new(r#"import\s+["']([^"']+)["']"#).unwrap();
                for caps in re.captures_iter(source) {
                    if let Some(m) = caps.get(1) {
                        values.push(m.as_str().to_string());
                    }
                }
            }
            // Extract contract name: contract_name()
            else if dsl == "contract_name()" {
                let re = Regex::new(r"(?:contract|interface|library|abstract\s+contract)\s+(\w+)").unwrap();
                for caps in re.captures_iter(source) {
                    if let Some(m) = caps.get(1) {
                        values.push(m.as_str().to_string());
                    }
                }
            }
            // Extract pragma version: pragma_version()
            else if dsl == "pragma_version()" {
                let re = Regex::new(r"pragma\s+solidity\s+([^;]+)").unwrap();
                for caps in re.captures_iter(source) {
                    if let Some(m) = caps.get(1) {
                        values.push(m.as_str().trim().to_string());
                    }
                }
            }
            // Extract external calls: external_calls()
            else if dsl == "external_calls()" {
                let re = Regex::new(r"(\w+)\.(call|delegatecall|staticcall|transfer|send)\s*[\(\{]").unwrap();
                for caps in re.captures_iter(source) {
                    if let Some(m) = caps.get(0) {
                        values.push(m.as_str().to_string());
                    }
                }
            }
        }

        values
    }

    fn extract_json(&self, source: &str) -> Vec<String> {
        // Try to parse as JSON and extract using path
        if let Some(path) = &self.json {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(source) {
                if let Some(val) = self.json_path_extract(&json, path) {
                    return vec![val];
                }
            }
        }
        vec![]
    }

    fn json_path_extract(&self, value: &serde_json::Value, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.trim_start_matches('.').split('.').collect();
        let mut current = value;

        for part in parts {
            if let Some(index) = part.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Ok(i) = index.parse::<usize>() {
                    current = current.get(i)?;
                }
            } else {
                current = current.get(part)?;
            }
        }

        match current {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            _ => Some(current.to_string()),
        }
    }

    fn extract_kval(&self, source: &str) -> Vec<String> {
        // Key-value extraction from comments or annotations
        let mut values = Vec::new();
        
        // Pattern: @name value or name: value
        let patterns = [
            format!(r"@{}\s+([^\n]+)", regex::escape(&self.name)),
            format!(r"{}\s*:\s*([^\n]+)", regex::escape(&self.name)),
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                for caps in re.captures_iter(source) {
                    if let Some(m) = caps.get(1) {
                        values.push(m.as_str().trim().to_string());
                    }
                }
            }
        }

        values
    }
}

impl Default for Extractor {
    fn default() -> Self {
        Self {
            extractor_type: ExtractorType::default(),
            name: String::new(),
            regex: Vec::new(),
            group: 0,
            json: None,
            xpath: None,
            dsl: None,
            part: ExtractorPart::default(),
            internal: false,
            compiled: OnceCell::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_extractor() {
        let extractor = Extractor {
            name: "functions".to_string(),
            regex: vec![r"function\s+(\w+)".to_string()],
            group: 1,
            ..Default::default()
        };

        let source = "function transfer() {} function approve() {}";
        let result = extractor.execute(source, &HashMap::new()).unwrap();
        
        assert_eq!(result.values, vec!["transfer", "approve"]);
    }

    #[test]
    fn test_dsl_extractor() {
        let extractor = Extractor {
            extractor_type: ExtractorType::Dsl,
            name: "functions".to_string(),
            dsl: Some("functions()".to_string()),
            ..Default::default()
        };

        let source = "function transfer(address to) public {} function approve() external {}";
        let result = extractor.execute(source, &HashMap::new()).unwrap();
        
        assert!(result.values.contains(&"transfer".to_string()));
        assert!(result.values.contains(&"approve".to_string()));
    }
}
crates/scpf-core/src/template/parser.rs
Rust

use std::path::Path;
use tokio::fs;

use crate::error::{Error, Result};
use super::{Template, TemplateValidator};

/// Parser for template files
pub struct TemplateParser {
    validator: TemplateValidator,
    strict: bool,
}

impl TemplateParser {
    pub fn new() -> Self {
        Self {
            validator: TemplateValidator::new(),
            strict: false,
        }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Parse a template from a YAML string
    pub fn parse_str(&self, content: &str) -> Result<Template> {
        let mut template: Template = serde_yaml::from_str(content)
            .map_err(|e| Error::template_parse(format!("YAML parse error: {}", e)))?;

        // Validate
        if let Err(errors) = self.validator.validate(&template) {
            if self.strict || errors.iter().any(|e| e.is_error) {
                return Err(Error::template_validation(
                    errors.iter()
                        .map(|e| e.message.as_str())
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }
        }

        // Compile matchers
        for matcher in &template.matchers {
            matcher.compile()?;
        }

        // Compile extractors
        for extractor in &template.extractors {
            extractor.compile()?;
        }

        template.compiled = true;
        Ok(template)
    }

    /// Parse a template from a file
    pub async fn parse_file(&self, path: impl AsRef<Path>) -> Result<Template> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).await?;
        
        let mut template = self.parse_str(&content)?;
        template.file_path = Some(path.to_path_buf());

        Ok(template)
    }

    /// Parse multiple templates from a directory
    pub async fn parse_directory(&self, path: impl AsRef<Path>) -> Result<Vec<Template>> {
        let path = path.as_ref();
        let mut templates = Vec::new();

        let mut entries = fs::read_dir(path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                // Recursively process subdirectories
                let sub_templates = Box::pin(self.parse_directory(&entry_path)).await?;
                templates.extend(sub_templates);
            } else if Self::is_template_file(&entry_path) {
                match self.parse_file(&entry_path).await {
                    Ok(template) => templates.push(template),
                    Err(e) => {
                        tracing::warn!("Failed to parse template {:?}: {}", entry_path, e);
                        if self.strict {
                            return Err(e);
                        }
                    }
                }
            }
        }

        Ok(templates)
    }

    fn is_template_file(path: &Path) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => true,
            _ => false,
        }
    }
}

impl Default for TemplateParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_template() {
        let yaml = r#"
id: test-template
info:
  name: Test Template
  author: Test
  severity: high
  tags:
    - test
matchers:
  - type: word
    words:
      - "selfdestruct"
"#;
        
        let parser = TemplateParser::new();
        let template = parser.parse_str(yaml).unwrap();
        
        assert_eq!(template.id, "test-template");
        assert_eq!(template.info.name, "Test Template");
        assert!(!template.matchers.is_empty());
    }

    #[test]
    fn test_parse_with_extractors() {
        let yaml = r#"
id: extractor-test
info:
  name: Extractor Test
  severity: info
matchers:
  - type: regex
    regex:
      - "function\\s+\\w+"
extractors:
  - type: regex
    name: functions
    regex:
      - "function\\s+(\\w+)"
    group: 1
"#;
        
        let parser = TemplateParser::new();
        let template = parser.parse_str(yaml).unwrap();
        
        assert!(!template.extractors.is_empty());
    }
}
crates/scpf-core/src/template/validator.rs
Rust

use super::Template;

/// Validation result
pub struct ValidationResult {
    pub errors: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        !self.errors.iter().any(|e| e.is_error)
    }
}

/// A single validation issue
#[derive(Debug)]
pub struct ValidationIssue {
    pub message: String,
    pub is_error: bool,
    pub field: Option<String>,
}

impl ValidationIssue {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_error: true,
            field: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_error: false,
            field: None,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }
}

/// Template validator
pub struct TemplateValidator {
    require_id: bool,
    require_name: bool,
    require_matchers: bool,
}

impl TemplateValidator {
    pub fn new() -> Self {
        Self {
            require_id: true,
            require_name: true,
            require_matchers: true,
        }
    }

    pub fn validate(&self, template: &Template) -> Result<(), Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Required fields
        if self.require_id && template.id.is_empty() {
            issues.push(ValidationIssue::error("Template ID is required").with_field("id"));
        }

        if self.require_name && template.info.name.is_empty() {
            issues.push(ValidationIssue::error("Template name is required").with_field("info.name"));
        }

        if self.require_matchers && template.matchers.is_empty() {
            issues.push(ValidationIssue::error("At least one matcher is required").with_field("matchers"));
        }

        // Validate matchers
        for (i, matcher) in template.matchers.iter().enumerate() {
            let field = format!("matchers[{}]", i);

            match matcher.matcher_type {
                super::MatcherType::Word => {
                    if matcher.words.is_empty() {
                        issues.push(
                            ValidationIssue::error("Word matcher requires 'words' field")
                                .with_field(&field)
                        );
                    }
                }
                super::MatcherType::Regex => {
                    if matcher.regex.is_empty() {
                        issues.push(
                            ValidationIssue::error("Regex matcher requires 'regex' field")
                                .with_field(&field)
                        );
                    }

                    // Validate regex patterns
                    for (j, pattern) in matcher.regex.iter().enumerate() {
                        if regex::Regex::new(pattern).is_err() && fancy_regex::Regex::new(pattern).is_err() {
                            issues.push(
                                ValidationIssue::error(format!("Invalid regex pattern: {}", pattern))
                                    .with_field(format!("{}regex[{}]", field, j))
                            );
                        }
                    }
                }
                super::MatcherType::Dsl => {
                    if matcher.dsl.is_empty() {
                        issues.push(
                            ValidationIssue::error("DSL matcher requires 'dsl' field")
                                .with_field(&field)
                        );
                    }
                }
                super::MatcherType::Binary => {
                    if matcher.binary.is_empty() {
                        issues.push(
                            ValidationIssue::error("Binary matcher requires 'binary' field")
                                .with_field(&field)
                        );
                    }
                }
            }
        }

        // Validate extractors
        for (i, extractor) in template.extractors.iter().enumerate() {
            let field = format!("extractors[{}]", i);

            if extractor.name.is_empty() {
                issues.push(
                    ValidationIssue::error("Extractor requires 'name' field")
                        .with_field(&field)
                );
            }

            if matches!(extractor.extractor_type, super::ExtractorType::Regex) && extractor.regex.is_empty() {
                issues.push(
                    ValidationIssue::error("Regex extractor requires 'regex' field")
                        .with_field(&field)
                );
            }
        }

        // Warnings for best practices
        if template.info.description.is_empty() {
            issues.push(ValidationIssue::warning("Template should have a description"));
        }

        if template.info.tags.is_empty() {
            issues.push(ValidationIssue::warning("Template should have tags for filtering"));
        }

        if template.info.reference.is_empty() {
            issues.push(ValidationIssue::warning("Template should have references"));
        }

        if issues.iter().any(|i| i.is_error) {
            Err(issues)
        } else {
            Ok(())
        }
    }
}

impl Default for TemplateValidator {
    fn default() -> Self {
        Self::new()
    }
}
crates/scpf-core/src/template/store.rs
Rust

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{Error, Result};
use crate::Severity;
use super::{Template, TemplateParser};

/// Thread-safe template store
pub struct TemplateStore {
    templates: Arc<RwLock<HashMap<String, Template>>>,
    loaded_paths: Arc<RwLock<Vec<PathBuf>>>,
}

impl TemplateStore {
    pub fn new() -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            loaded_paths: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Load templates from a directory
    pub async fn from_directory(path: impl AsRef<Path>) -> Result<Self> {
        let store = Self::new();
        store.load_directory(path).await?;
        Ok(store)
    }

    /// Load templates from multiple directories
    pub async fn from_directories(paths: &[PathBuf]) -> Result<Self> {
        let store = Self::new();
        for path in paths {
            if path.exists() {
                store.load_directory(path).await?;
            }
        }
        Ok(store)
    }

    /// Load templates from a directory
    pub async fn load_directory(&self, path: impl AsRef<Path>) -> Result<usize> {
        let path = path.as_ref();
        let parser = TemplateParser::new();
        let templates = parser.parse_directory(path).await?;
        
        let count = templates.len();
        
        let mut store = self.templates.write().await;
        for template in templates {
            store.insert(template.id.clone(), template);
        }

        let mut paths = self.loaded_paths.write().await;
        paths.push(path.to_path_buf());

        Ok(count)
    }

    /// Add a single template
    pub async fn add(&self, template: Template) {
        let mut store = self.templates.write().await;
        store.insert(template.id.clone(), template);
    }

    /// Get a template by ID
    pub async fn get(&self, id: &str) -> Option<Template> {
        let store = self.templates.read().await;
        store.get(id).cloned()
    }

    /// Get all templates
    pub async fn all(&self) -> Vec<Template> {
        let store = self.templates.read().await;
        store.values().cloned().collect()
    }

    /// Get templates filtered by tags
    pub async fn by_tags(&self, include_tags: &[String], exclude_tags: &[String]) -> Vec<Template> {
        let store = self.templates.read().await;
        store.values()
            .filter(|t| {
                // Must have at least one include tag (if specified)
                let includes_match = include_tags.is_empty() || t.has_any_tag(include_tags);
                
                // Must not have any exclude tag
                let excludes_match = !exclude_tags.iter().any(|tag| t.has_tag(tag));
                
                includes_match && excludes_match
            })
            .cloned()
            .collect()
    }

    /// Get templates filtered by severity
    pub async fn by_severity(&self, min_severity: Severity) -> Vec<Template> {
        let store = self.templates.read().await;
        store.values()
            .filter(|t| t.severity() >= min_severity)
            .cloned()
            .collect()
    }

    /// Get templates filtered by tags and severity
    pub async fn filtered(
        &self,
        include_tags: &[String],
        exclude_tags: &[String],
        min_severity: Option<Severity>,
        max_count: Option<usize>,
    ) -> Vec<Template> {
        let store = self.templates.read().await;
        let mut templates: Vec<_> = store.values()
            .filter(|t| {
                let includes_match = include_tags.is_empty() || t.has_any_tag(include_tags);
                let excludes_match = !exclude_tags.iter().any(|tag| t.has_tag(tag));
                let severity_match = min_severity.map(|s| t.severity() >= s).unwrap_or(true);
                
                includes_match && excludes_match && severity_match
            })
            .cloned()
            .collect();

        // Sort by severity (highest first)
        templates.sort_by(|a, b| b.severity().cmp(&a.severity()));

        if let Some(max) = max_count {
            templates.truncate(max);
        }

        templates
    }

    /// Count templates
    pub async fn len(&self) -> usize {
        let store = self.templates.read().await;
        store.len()
    }

    /// Check if empty
    pub async fn is_empty(&self) -> bool {
        let store = self.templates.read().await;
        store.is_empty()
    }

    /// Get all unique tags
    pub async fn all_tags(&self) -> Vec<String> {
        let store = self.templates.read().await;
        let mut tags: Vec<String> = store.values()
            .flat_map(|t| t.tags().iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Remove a template
    pub async fn remove(&self, id: &str) -> Option<Template> {
        let mut store = self.templates.write().await;
        store.remove(id)
    }

    /// Clear all templates
    pub async fn clear(&self) {
        let mut store = self.templates.write().await;
        store.clear();
    }

    /// Reload templates from loaded paths
    pub async fn reload(&self) -> Result<usize> {
        let paths: Vec<PathBuf> = {
            let p = self.loaded_paths.read().await;
            p.clone()
        };

        self.clear().await;

        let mut total = 0;
        for path in paths {
            total += self.load_directory(&path).await?;
        }

        Ok(total)
    }
}

impl Default for TemplateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TemplateStore {
    fn clone(&self) -> Self {
        Self {
            templates: Arc::clone(&self.templates),
            loaded_paths: Arc::clone(&self.loaded_paths),
        }
    }
}
crates/scpf-core/src/scanner/mod.rs
Rust

//! Scanner module for contract analysis

mod engine;
mod result;
mod parallel;

pub use engine::{Scanner, ScanOptions};
pub use result::ScanStats;
pub use parallel::ParallelScanner;
crates/scpf-core/src/scanner/engine.rs
Rust

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use chrono::Utc;

use crate::error::{Error, Result};
use crate::config::Config;
use crate::template::{TemplateStore, Template, MatchCondition};
use crate::source::MultiSourceProvider;
use crate::cache::Cache;
use crate::{Chain, ScanMatch, MatchLocation, SourceCode};

use super::ScanStats;

/// Scan options
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Stop after first match
    pub stop_at_first: bool,
    
    /// Maximum matches per template
    pub max_matches_per_template: Option<usize>,
    
    /// Context lines around matches
    pub context_lines: usize,
    
    /// Include source code in results
    pub include_source: bool,
    
    /// Template filter tags
    pub include_tags: Vec<String>,
    
    /// Template exclude tags
    pub exclude_tags: Vec<String>,
    
    /// Profile name
    pub profile: Option<String>,
    
    /// Use bytecode prefiltering
    pub prefilter: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            stop_at_first: false,
            max_matches_per_template: None,
            context_lines: 3,
            include_source: false,
            include_tags: Vec::new(),
            exclude_tags: Vec::new(),
            profile: None,
            prefilter: true,
        }
    }
}

/// Main scanner engine
pub struct Scanner {
    config: Config,
    templates: TemplateStore,
    source_provider: Arc<MultiSourceProvider>,
    cache: Arc<Cache>,
    concurrency: Semaphore,
    stats: Arc<RwLock<ScanStats>>,
}

impl Scanner {
    /// Create a new scanner
    pub fn new(config: Config, templates: TemplateStore) -> Result<Self> {
        let cache = Cache::new(&config.cache)?;
        let source_provider = MultiSourceProvider::new(&config)?;
        let concurrency = Semaphore::new(config.scanner.concurrency);

        Ok(Self {
            config,
            templates,
            source_provider: Arc::new(source_provider),
            cache: Arc::new(cache),
            concurrency,
            stats: Arc::new(RwLock::new(ScanStats::default())),
        })
    }

    /// Scan a single address
    pub async fn scan_address(
        &self,
        address: &str,
        chain: Chain,
    ) -> Result<Vec<ScanMatch>> {
        self.scan_address_with_options(address, chain, &ScanOptions::default()).await
    }

    /// Scan a single address with options
    pub async fn scan_address_with_options(
        &self,
        address: &str,
        chain: Chain,
        options: &ScanOptions,
    ) -> Result<Vec<ScanMatch>> {
        // Check cache first
        let cache_key = format!("{}:{}", chain.chain_id(), address.to_lowercase());
        if let Some(cached) = self.cache.get_scan_result(&cache_key).await? {
            let mut stats = self.stats.write().await;
            stats.cache_hits += 1;
            return Ok(cached);
        }

        // Acquire concurrency permit
        let _permit = self.concurrency.acquire().await
            .map_err(|_| Error::Cancelled)?;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.contracts_scanned += 1;
        }

        // Fetch source code
        let source = self.source_provider.fetch_source(address, chain).await?;

        // Get filtered templates
        let templates = self.get_filtered_templates(options).await;

        // Scan with templates
        let matches = self.scan_source_with_templates(&source, chain, address, &templates, options).await?;

        // Cache results
        if self.config.cache.cache_results {
            self.cache.set_scan_result(&cache_key, &matches).await?;
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.matches_found += matches.len();
        }

        Ok(matches)
    }

    /// Scan multiple addresses
    pub async fn scan_addresses(
        &self,
        addresses: &[(String, Chain)],
        options: &ScanOptions,
    ) -> Result<Vec<ScanMatch>> {
        let mut all_matches = Vec::new();

        for (address, chain) in addresses {
            match self.scan_address_with_options(address, *chain, options).await {
                Ok(matches) => {
                    all_matches.extend(matches);
                    
                    if options.stop_at_first && !all_matches.is_empty() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to scan {}: {}", address, e);
                    let mut stats = self.stats.write().await;
                    stats.errors += 1;
                }
            }
        }

        Ok(all_matches)
    }

    /// Scan source code directly
    pub async fn scan_source(
        &self,
        source: &str,
        contract_name: &str,
        options: &ScanOptions,
    ) -> Result<Vec<ScanMatch>> {
        let source_code = SourceCode::new(source, contract_name);
        let templates = self.get_filtered_templates(options).await;
        
        self.scan_source_with_templates(
            &source_code,
            Chain::Ethereum,
            "local",
            &templates,
            options,
        ).await
    }

    async fn get_filtered_templates(&self, options: &ScanOptions) -> Vec<Template> {
        // Apply profile if specified
        let (include_tags, exclude_tags) = if let Some(profile_name) = &options.profile {
            if let Some(profile) = self.config.profiles.get(profile_name) {
                (profile.include_tags.clone(), profile.exclude_tags.clone())
            } else {
                (options.include_tags.clone(), options.exclude_tags.clone())
            }
        } else {
            (options.include_tags.clone(), options.exclude_tags.clone())
        };

        self.templates.filtered(&include_tags, &exclude_tags, None, None).await
    }

    async fn scan_source_with_templates(
        &self,
        source: &SourceCode,
        chain: Chain,
        address: &str,
        templates: &[Template],
        options: &ScanOptions,
    ) -> Result<Vec<ScanMatch>> {
        let mut matches = Vec::new();
        let metadata = self.build_metadata(source, chain, address);

        for template in templates {
            if let Some(template_matches) = self.apply_template(template, source, chain, address, &metadata, options).await? {
                matches.extend(template_matches);

                if options.stop_at_first && !matches.is_empty() {
                    break;
                }
            }
        }

        Ok(matches)
    }

    async fn apply_template(
        &self,
        template: &Template,
        source: &SourceCode,
        chain: Chain,
        address: &str,
        metadata: &HashMap<String, String>,
        options: &ScanOptions,
    ) -> Result<Option<Vec<ScanMatch>>> {
        // Execute matchers
        let mut matcher_results = Vec::new();
        let mut all_locations = Vec::new();

        for matcher in &template.matchers {
            let result = matcher.execute(&source.source, metadata)?;
            
            if !matcher.internal {
                all_locations.extend(result.locations);
            }
            
            matcher_results.push(result.matched);
        }

        // Check if template matches based on condition
        let matched = template.matchers_condition.evaluate(&matcher_results);

        if !matched {
            return Ok(None);
        }

        // Execute extractors
        let mut extracted = HashMap::new();
        for extractor in &template.extractors {
            let result = extractor.execute(&source.source, metadata)?;
            if !result.internal && !result.values.is_empty() {
                extracted.insert(result.name.clone(), result.values);
            }
        }

        // Build match result
        let mut scan_match = ScanMatch::new(
            address,
            &source.contract_name,
            chain,
            &template.id,
            &template.info.name,
            template.severity(),
        )
        .with_locations(all_locations)
        .with_extracted(extracted)
        .with_tags(template.tags().to_vec());

        // Add snippet if requested
        if options.include_source {
            if let Some(first_loc) = scan_match.locations.first() {
                let snippet = self.extract_snippet(&source.source, first_loc.line, options.context_lines);
                scan_match = scan_match.with_snippet(snippet);
            }
        }

        Ok(Some(vec![scan_match]))
    }

    fn build_metadata(&self, source: &SourceCode, chain: Chain, address: &str) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("address".to_string(), address.to_string());
        metadata.insert("chain".to_string(), chain.to_string());
        metadata.insert("chain_id".to_string(), chain.chain_id().to_string());
        metadata.insert("contract_name".to_string(), source.contract_name.clone());
        metadata.insert("compiler_version".to_string(), source.compiler_version.clone());
        metadata.insert("source_len".to_string(), source.len().to_string());
        metadata.insert("line_count".to_string(), source.line_count().to_string());
        metadata
    }

    fn extract_snippet(&self, source: &str, line: usize, context: usize) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let start = line.saturating_sub(context + 1);
        let end = (line + context).min(lines.len());

        lines[start..end].join("\n")
    }

    /// Get current stats
    pub async fn stats(&self) -> ScanStats {
        self.stats.read().await.clone()
    }

    /// Reset stats
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ScanStats::default();
    }

    /// Get the template store
    pub fn templates(&self) -> &TemplateStore {
        &self.templates
    }

    /// Get the cache
    pub fn cache(&self) -> &Arc<Cache> {
        &self.cache
    }
}
crates/scpf-core/src/scanner/result.rs
Rust

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Statistics from scanning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanStats {
    /// Number of contracts scanned
    pub contracts_scanned: usize,
    
    /// Number of matches found
    pub matches_found: usize,
    
    /// Number of templates used
    pub templates_used: usize,
    
    /// Cache hits
    pub cache_hits: usize,
    
    /// Cache misses
    pub cache_misses: usize,
    
    /// Errors encountered
    pub errors: usize,
    
    /// Start time
    pub started_at: Option<DateTime<Utc>>,
    
    /// End time
    pub ended_at: Option<DateTime<Utc>>,
}

impl ScanStats {
    pub fn new() -> Self {
        Self {
            started_at: Some(Utc::now()),
            ..Default::default()
        }
    }

    pub fn finish(&mut self) {
        self.ended_at = Some(Utc::now());
    }

    pub fn duration_seconds(&self) -> f64 {
        match (self.started_at, self.ended_at) {
            (Some(start), Some(end)) => (end - start).num_milliseconds() as f64 / 1000.0,
            (Some(start), None) => (Utc::now() - start).num_milliseconds() as f64 / 1000.0,
            _ => 0.0,
        }
    }

    pub fn contracts_per_second(&self) -> f64 {
        let duration = self.duration_seconds();
        if duration > 0.0 {
            self.contracts_scanned as f64 / duration
        } else {
            0.0
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }
}
crates/scpf-core/src/scanner/parallel.rs
Rust

use std::sync::Arc;
use tokio::sync::mpsc;
use futures::stream::{self, StreamExt};

use crate::error::Result;
use crate::{Chain, ScanMatch};
use super::{Scanner, ScanOptions};

/// Parallel scanner for high-throughput scanning
pub struct ParallelScanner {
    scanner: Arc<Scanner>,
    concurrency: usize,
}

impl ParallelScanner {
    pub fn new(scanner: Scanner, concurrency: usize) -> Self {
        Self {
            scanner: Arc::new(scanner),
            concurrency,
        }
    }

    /// Scan addresses in parallel with streaming results
    pub async fn scan_stream(
        &self,
        addresses: Vec<(String, Chain)>,
        options: ScanOptions,
    ) -> mpsc::Receiver<Result<(String, Vec<ScanMatch>)>> {
        let (tx, rx) = mpsc::channel(self.concurrency * 2);
        let scanner = Arc::clone(&self.scanner);
        let options = Arc::new(options);
        let concurrency = self.concurrency;

        tokio::spawn(async move {
            stream::iter(addresses)
                .map(|(address, chain)| {
                    let scanner = Arc::clone(&scanner);
                    let options = Arc::clone(&options);
                    let tx = tx.clone();
                    
                    async move {
                        let result = scanner.scan_address_with_options(&address, chain, &options).await;
                        let _ = tx.send(result.map(|matches| (address, matches))).await;
                    }
                })
                .buffer_unordered(concurrency)
                .collect::<Vec<_>>()
                .await;
        });

        rx
    }

    /// Scan all addresses and collect results
    pub async fn scan_all(
        &self,
        addresses: Vec<(String, Chain)>,
        options: ScanOptions,
    ) -> Result<Vec<ScanMatch>> {
        let mut rx = self.scan_stream(addresses, options).await;
        let mut all_matches = Vec::new();

        while let Some(result) = rx.recv().await {
            match result {
                Ok((_address, matches)) => {
                    all_matches.extend(matches);
                }
                Err(e) => {
                    tracing::warn!("Scan error: {}", e);
                }
            }
        }

        Ok(all_matches)
    }
}
crates/scpf-core/src/source/mod.rs
Rust

//! Source code fetching from block explorers

mod provider;
mod etherscan;
mod sourcify;
mod blockscout;
mod multi;

pub use provider::SourceProvider;
pub use etherscan::EtherscanProvider;
pub use sourcify::SourcifyProvider;
pub use blockscout::BlockscoutProvider;
pub use multi::MultiSourceProvider;
crates/scpf-core/src/source/provider.rs
Rust

use async_trait::async_trait;
use crate::error::Result;
use crate::{Chain, SourceCode};

/// Trait for source code providers
#[async_trait]
pub trait SourceProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &'static str;

    /// Check if provider supports this chain
    fn supports_chain(&self, chain: Chain) -> bool;

    /// Fetch source code for an address
    async fn fetch_source(&self, address: &str, chain: Chain) -> Result<SourceCode>;

    /// Check if contract is verified
    async fn is_verified(&self, address: &str, chain: Chain) -> Result<bool>;
}
crates/scpf-core/src/source/etherscan.rs
Rust

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::error::{Error, Result};
use crate::config::Config;
use crate::utils::{RateLimiter, RetryPolicy};
use crate::{Chain, SourceCode, SourceFile};

use super::SourceProvider;

pub struct EtherscanProvider {
    client: Client,
    rate_limiter: RateLimiter,
    retry_policy: RetryPolicy,
    api_keys: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct EtherscanResponse {
    status: String,
    message: String,
    result: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContractSource {
    source_code: String,
    contract_name: String,
    compiler_version: String,
    #[serde(rename = "ABI")]
    abi: String,
    optimization_used: String,
    runs: String,
    #[serde(rename = "EVMVersion")]
    evm_version: Option<String>,
}

impl EtherscanProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.scanner.timeout_seconds))
            .user_agent("SCPF/1.0")
            .build()?;

        let mut api_keys = std::collections::HashMap::new();
        for (name, chain_config) in &config.chains {
            if let Some(key) = &chain_config.api_key {
                api_keys.insert(name.clone(), key.clone());
            }
        }

        let rate_limiter = RateLimiter::new(config.scanner.rate_limit);
        let retry_policy = RetryPolicy::new(
            config.scanner.max_retries,
            Duration::from_millis(config.scanner.retry_delay_ms),
        );

        Ok(Self {
            client,
            rate_limiter,
            retry_policy,
            api_keys,
        })
    }

    fn get_api_url(&self, chain: Chain) -> &'static str {
        chain.api_url()
    }

    fn get_api_key(&self, chain: Chain) -> Option<&str> {
        let name = match chain {
            Chain::Ethereum => "ethereum",
            Chain::Polygon => "polygon",
            Chain::Bsc => "bsc",
            Chain::Arbitrum => "arbitrum",
            Chain::Optimism => "optimism",
            Chain::Base => "base",
            Chain::Avalanche => "avalanche",
            Chain::Fantom => "fantom",
            Chain::Gnosis => "gnosis",
            Chain::Custom(_) => return None,
        };
        self.api_keys.get(name).map(|s| s.as_str())
    }

    async fn fetch_with_retry(&self, address: &str, chain: Chain) -> Result<SourceCode> {
        self.retry_policy.retry(|| async {
            self.fetch_source_internal(address, chain).await
        }).await
    }

    async fn fetch_source_internal(&self, address: &str, chain: Chain) -> Result<SourceCode> {
        self.rate_limiter.acquire().await;

        let api_url = self.get_api_url(chain);
        let api_key = self.get_api_key(chain);

        let mut url = format!(
            "{}?module=contract&action=getsourcecode&address={}",
            api_url, address
        );

        if let Some(key) = api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        let response = self.client.get(&url).send().await?;
        let data: EtherscanResponse = response.json().await?;

        if data.status != "1" {
            return Err(Error::source_fetch(address, data.message));
        }

        let results: Vec<ContractSource> = serde_json::from_value(data.result)
            .map_err(|e| Error::source_fetch(address, e.to_string()))?;

        let contract = results.first()
            .ok_or_else(|| Error::SourceNotVerified(address.to_string()))?;

        if contract.source_code.is_empty() || contract.source_code == "Contract source code not verified" {
            return Err(Error::SourceNotVerified(address.to_string()));
        }

        // Handle multi-file contracts (JSON format)
        let (source, files) = if contract.source_code.starts_with("{{") {
            self.parse_multi_file_source(&contract.source_code)?
        } else if contract.source_code.starts_with('{') {
            // Standard JSON input
            self.parse_standard_json(&contract.source_code)?
        } else {
            (contract.source_code.clone(), vec![])
        };

        Ok(SourceCode {
            source,
            files,
            contract_name: contract.contract_name.clone(),
            compiler_version: contract.compiler_version.clone(),
            abi: Some(contract.abi.clone()),
            bytecode: None,
            provider: "etherscan".to_string(),
        })
    }

    fn parse_multi_file_source(&self, source: &str) -> Result<(String, Vec<SourceFile>)> {
        // Remove outer braces {{ and }}
        let inner = source.trim_start_matches('{').trim_end_matches('}');
        
        let json: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(inner)
            .map_err(|e| Error::source_fetch("", format!("Failed to parse multi-file: {}", e)))?;

        let mut files = Vec::new();
        let mut combined = String::new();

        if let Some(sources) = json.get("sources") {
            if let Some(obj) = sources.as_object() {
                for (path, value) in obj {
                    if let Some(content) = value.get("content").and_then(|c| c.as_str()) {
                        files.push(SourceFile {
                            path: path.clone(),
                            content: content.to_string(),
                        });
                        combined.push_str(&format!("// File: {}\n{}\n\n", path, content));
                    }
                }
            }
        }

        Ok((combined, files))
    }

    fn parse_standard_json(&self, source: &str) -> Result<(String, Vec<SourceFile>)> {
        let json: serde_json::Value = serde_json::from_str(source)
            .map_err(|e| Error::source_fetch("", format!("Failed to parse JSON: {}", e)))?;

        let mut files = Vec::new();
        let mut combined = String::new();

        if let Some(sources) = json.get("sources").and_then(|s| s.as_object()) {
            for (path, value) in sources {
                if let Some(content) = value.get("content").and_then(|c| c.as_str()) {
                    files.push(SourceFile {
                        path: path.clone(),
                        content: content.to_string(),
                    });
                    combined.push_str(&format!("// File: {}\n{}\n\n", path, content));
                }
            }
        }

        if combined.is_empty() {
            combined = source.to_string();
        }

        Ok((combined, files))
    }
}

#[async_trait]
impl SourceProvider for EtherscanProvider {
    fn name(&self) -> &'static str {
        "etherscan"
    }

    fn supports_chain(&self, chain: Chain) -> bool {
        !matches!(chain, Chain::Custom(_))
    }

    async fn fetch_source(&self, address: &str, chain: Chain) -> Result<SourceCode> {
        self.fetch_with_retry(address, chain).await
    }

    async fn is_verified(&self, address: &str, chain: Chain) -> Result<bool> {
        match self.fetch_source(address, chain).await {
            Ok(_) => Ok(true),
            Err(Error::SourceNotVerified(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
crates/scpf-core/src/source/sourcify.rs
Rust

use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

use crate::error::{Error, Result};
use crate::config::Config;
use crate::utils::{RateLimiter, RetryPolicy};
use crate::{Chain, SourceCode, SourceFile};

use super::SourceProvider;

pub struct SourcifyProvider {
    client: Client,
    rate_limiter: RateLimiter,
    retry_policy: RetryPolicy,
}

impl SourcifyProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.scanner.timeout_seconds))
            .user_agent("SCPF/1.0")
            .build()?;

        let rate_limiter = RateLimiter::new(10.0); // Sourcify has higher limits
        let retry_policy = RetryPolicy::new(
            config.scanner.max_retries,
            Duration::from_millis(config.scanner.retry_delay_ms),
        );

        Ok(Self {
            client,
            rate_limiter,
            retry_policy,
        })
    }

    fn get_base_url(&self) -> &'static str {
        "https://sourcify.dev/server"
    }

    async fn fetch_with_retry(&self, address: &str, chain: Chain) -> Result<SourceCode> {
        self.retry_policy.retry(|| async {
            self.fetch_source_internal(address, chain).await
        }).await
    }

    async fn fetch_source_internal(&self, address: &str, chain: Chain) -> Result<SourceCode> {
        self.rate_limiter.acquire().await;

        let chain_id = chain.chain_id().0;
        
        // Try full match first, then partial
        for match_type in &["full_match", "partial_match"] {
            let url = format!(
                "{}/files/{}/{}/{}",
                self.get_base_url(),
                match_type,
                chain_id,
                address
            );

            let response = self.client.get(&url).send().await?;

            if response.status().is_success() {
                let files: Vec<SourcifyFile> = response.json().await?;
                return self.process_files(files, address);
            }
        }

        Err(Error::SourceNotVerified(address.to_string()))
    }

    fn process_files(&self, files: Vec<SourcifyFile>, address: &str) -> Result<SourceCode> {
        let mut source_files = Vec::new();
        let mut combined = String::new();
        let mut contract_name = String::new();
        let mut compiler_version = String::new();
        let mut abi = None;

        for file in files {
            if file.name == "metadata.json" {
                if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&file.content) {
                    if let Some(version) = metadata.get("compiler").and_then(|c| c.get("version")).and_then(|v| v.as_str()) {
                        compiler_version = version.to_string();
                    }
                    if let Some(output) = metadata.get("output").and_then(|o| o.get("abi")) {
                        abi = Some(output.to_string());
                    }
                }
                continue;
            }

            if file.name.ends_with(".sol") {
                if contract_name.is_empty() {
                    contract_name = file.name.trim_end_matches(".sol").to_string();
                }
                
                source_files.push(SourceFile {
                    path: file.name.clone(),
                    content: file.content.clone(),
                });
                
                combined.push_str(&format!("// File: {}\n{}\n\n", file.name, file.content));
            }
        }

        if combined.is_empty() {
            return Err(Error::SourceNotVerified(address.to_string()));
        }

        Ok(SourceCode {
            source: combined,
            files: source_files,
            contract_name,
            compiler_version,
            abi,
            bytecode: None,
            provider: "sourcify".to_string(),
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct SourcifyFile {
    name: String,
    content: String,
}

#[async_trait]
impl SourceProvider for SourcifyProvider {
    fn name(&self) -> &'static str {
        "sourcify"
    }

    fn supports_chain(&self, _chain: Chain) -> bool {
        true // Sourcify supports many chains
    }

    async fn fetch_source(&self, address: &str, chain: Chain) -> Result<SourceCode> {
        self.fetch_with_retry(address, chain).await
    }

    async fn is_verified(&self, address: &str, chain: Chain) -> Result<bool> {
        self.rate_limiter.acquire().await;
        
        let chain_id = chain.chain_id().0;
        let url = format!(
            "{}/check-by-addresses?addresses={}&chainIds={}",
            self.get_base_url(),
            address,
            chain_id
        );

        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let data: Vec<serde_json::Value> = response.json().await?;
            Ok(data.first()
                .and_then(|v| v.get("status"))
                .and_then(|s| s.as_str())
                .map(|s| s == "perfect" || s == "partial")
                .unwrap_or(false))
        } else {
            Ok(false)
        }
    }
}
crates/scpf-core/src/source/blockscout.rs
Rust

use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

use crate::error::{Error, Result};
use crate::config::Config;
use crate::utils::RateLimiter;
use crate::{Chain, SourceCode};

use super::SourceProvider;

pub struct BlockscoutProvider {
    client: Client,
    rate_limiter: RateLimiter,
}

impl BlockscoutProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.scanner.timeout_seconds))
            .user_agent("SCPF/1.0")
            .build()?;

        let rate_limiter = RateLimiter::new(5.0);

        Ok(Self {
            client,
            rate_limiter,
        })
    }

    fn get_api_url(&self, chain: Chain) -> Option<&'static str> {
        match chain {
            Chain::Ethereum => Some("https://eth.blockscout.com/api"),
            Chain::Gnosis => Some("https://gnosis.blockscout.com/api"),
            Chain::Optimism => Some("https://optimism.blockscout.com/api"),
            Chain::Base => Some("https://base.blockscout.com/api"),
            Chain::Polygon => Some("https://polygon.blockscout.com/api"),
            _ => None,
        }
    }
}

#[async_trait]
impl SourceProvider for BlockscoutProvider {
    fn name(&self) -> &'static str {
        "blockscout"
    }

    fn supports_chain(&self, chain: Chain) -> bool {
        self.get_api_url(chain).is_some()
    }

    async fn fetch_source(&self, address: &str, chain: Chain) -> Result<SourceCode> {
        let api_url = self.get_api_url(chain)
            .ok_or_else(|| Error::UnsupportedChain(chain.to_string()))?;

        self.rate_limiter.acquire().await;

        let url = format!(
            "{}?module=contract&action=getsourcecode&address={}",
            api_url, address
        );

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(Error::source_fetch(address, "Request failed"));
        }

        let data: serde_json::Value = response.json().await?;
        
        let result = data.get("result")
            .and_then(|r| r.get(0))
            .ok_or_else(|| Error::SourceNotVerified(address.to_string()))?;

        let source = result.get("SourceCode")
            .and_then(|s| s.as_str())
            .ok_or_else(|| Error::SourceNotVerified(address.to_string()))?;

        if source.is_empty() {
            return Err(Error::SourceNotVerified(address.to_string()));
        }

        let contract_name = result.get("ContractName")
            .and_then(|s| s.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let compiler_version = result.get("CompilerVersion")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        let abi = result.get("ABI")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        Ok(SourceCode {
            source: source.to_string(),
            files: vec![],
            contract_name,
            compiler_version,
            abi,
            bytecode: None,
            provider: "blockscout".to_string(),
        })
    }

    async fn is_verified(&self, address: &str, chain: Chain) -> Result<bool> {
        match self.fetch_source(address, chain).await {
            Ok(_) => Ok(true),
            Err(Error::SourceNotVerified(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
crates/scpf-core/src/source/multi.rs
Rust

use std::sync::Arc;
use async_trait::async_trait;

use crate::error::{Error, Result};
use crate::config::Config;
use crate::{Chain, SourceCode};

use super::{SourceProvider, EtherscanProvider, SourcifyProvider, BlockscoutProvider};

/// Multi-source provider that tries multiple providers in order
pub struct MultiSourceProvider {
    providers: Vec<Arc<dyn SourceProvider>>,
}

impl MultiSourceProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let mut providers: Vec<Arc<dyn SourceProvider>> = Vec::new();

        // Add Etherscan first (most reliable for most chains)
        providers.push(Arc::new(EtherscanProvider::new(config)?));

        // Add Sourcify (good for open source verification)
        providers.push(Arc::new(SourcifyProvider::new(config)?));

        // Add Blockscout
        providers.push(Arc::new(BlockscoutProvider::new(config)?));

        Ok(Self { providers })
    }

    /// Create with specific providers
    pub fn with_providers(providers: Vec<Arc<dyn SourceProvider>>) -> Self {
        Self { providers }
    }

    /// Fetch source, trying each provider in order
    pub async fn fetch_source(&self, address: &str, chain: Chain) -> Result<SourceCode> {
        let mut last_error = None;

        for provider in &self.providers {
            if !provider.supports_chain(chain) {
                continue;
            }

            match provider.fetch_source(address, chain).await {
                Ok(source) => return Ok(source),
                Err(e) => {
                    tracing::debug!(
                        "Provider {} failed for {}: {}",
                        provider.name(),
                        address,
                        e
                    );
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::SourceNotVerified(address.to_string())))
    }

    /// Check if verified on any provider
    pub async fn is_verified(&self, address: &str, chain: Chain) -> Result<bool> {
        for provider in &self.providers {
            if !provider.supports_chain(chain) {
                continue;
            }

            if provider.is_verified(address, chain).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[async_trait]
impl SourceProvider for MultiSourceProvider {
    fn name(&self) -> &'static str {
        "multi"
    }

    fn supports_chain(&self, chain: Chain) -> bool {
        self.providers.iter().any(|p| p.supports_chain(chain))
    }

    async fn fetch_source(&self, address: &str, chain: Chain) -> Result<SourceCode> {
        MultiSourceProvider::fetch_source(self, address, chain).await
    }

    async fn is_verified(&self, address: &str, chain: Chain) -> Result<bool> {
        MultiSourceProvider::is_verified(self, address, chain).await
    }
}
crates/scpf-core/src/cache/mod.rs
Rust

//! Caching layer

mod memory;
#[cfg(feature = "sqlite")]
mod persistent;

pub use memory::MemoryCache;
#[cfg(feature = "sqlite")]
pub use persistent::PersistentCache;

use crate::error::Result;
use crate::config::CacheConfig;
use crate::{SourceCode, ScanMatch};

/// Cache configuration from config
pub use crate::config::CacheConfig as CacheSettings;

/// Unified cache interface
pub struct Cache {
    memory: MemoryCache,
    #[cfg(feature = "sqlite")]
    persistent: Option<PersistentCache>,
}

impl Cache {
    pub fn new(config: &CacheConfig) -> Result<Self> {
        let memory = MemoryCache::new(config.max_entries, config.ttl_seconds);
        
        #[cfg(feature = "sqlite")]
        let persistent = if config.persistent {
            Some(PersistentCache::new()?)
        } else {
            None
        };

        Ok(Self {
            memory,
            #[cfg(feature = "sqlite")]
            persistent,
        })
    }

    /// Get source code from cache
    pub async fn get_source(&self, key: &str) -> Result<Option<SourceCode>> {
        // Check memory first
        if let Some(source) = self.memory.get_source(key).await {
            return Ok(Some(source));
        }

        // Check persistent cache
        #[cfg(feature = "sqlite")]
        if let Some(ref persistent) = self.persistent {
            if let Some(source) = persistent.get_source(key).await? {
                // Backfill memory cache
                self.memory.set_source(key, &source).await;
                return Ok(Some(source));
            }
        }

        Ok(None)
    }

    /// Store source code in cache
    pub async fn set_source(&self, key: &str, source: &SourceCode) -> Result<()> {
        self.memory.set_source(key, source).await;
        
        #[cfg(feature = "sqlite")]
        if let Some(ref persistent) = self.persistent {
            persistent.set_source(key, source).await?;
        }

        Ok(())
    }

    /// Get scan results from cache
    pub async fn get_scan_result(&self, key: &str) -> Result<Option<Vec<ScanMatch>>> {
        if let Some(result) = self.memory.get_scan_result(key).await {
            return Ok(Some(result));
        }

        #[cfg(feature = "sqlite")]
        if let Some(ref persistent) = self.persistent {
            if let Some(result) = persistent.get_scan_result(key).await? {
                self.memory.set_scan_result(key, &result).await;
                return Ok(Some(result));
            }
        }

        Ok(None)
    }

    /// Store scan results in cache
    pub async fn set_scan_result(&self, key: &str, matches: &[ScanMatch]) -> Result<()> {
        self.memory.set_scan_result(key, matches).await;

        #[cfg(feature = "sqlite")]
        if let Some(ref persistent) = self.persistent {
            persistent.set_scan_result(key, matches).await?;
        }

        Ok(())
    }

    /// Clear all caches
    pub async fn clear(&self) -> Result<()> {
        self.memory.clear().await;

        #[cfg(feature = "sqlite")]
        if let Some(ref persistent) = self.persistent {
            persistent.clear().await?;
        }

        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let memory_stats = self.memory.stats().await;
        
        CacheStats {
            memory_entries: memory_stats.entries,
            memory_hits: memory_stats.hits,
            memory_misses: memory_stats.misses,
            persistent_enabled: self.persistent.is_some(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub memory_entries: u64,
    pub memory_hits: u64,
    pub memory_misses: u64,
    pub persistent_enabled: bool,
}
crates/scpf-core/src/cache/memory.rs
Rust

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use moka::future::Cache as MokaCache;

use crate::{SourceCode, ScanMatch};

pub struct MemoryCache {
    sources: MokaCache<String, SourceCode>,
    scan_results: MokaCache<String, Vec<ScanMatch>>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl MemoryCache {
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        let ttl = Duration::from_secs(ttl_seconds);

        let sources = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .build();

        let scan_results = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .build();

        Self {
            sources,
            scan_results,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    pub async fn get_source(&self, key: &str) -> Option<SourceCode> {
        let result = self.sources.get(key).await;
        if result.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    pub async fn set_source(&self, key: &str, source: &SourceCode) {
        self.sources.insert(key.to_string(), source.clone()).await;
    }

    pub async fn get_scan_result(&self, key: &str) -> Option<Vec<ScanMatch>> {
        let result = self.scan_results.get(key).await;
        if result.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    pub async fn set_scan_result(&self, key: &str, matches: &[ScanMatch]) {
        self.scan_results.insert(key.to_string(), matches.to_vec()).await;
    }

    pub async fn clear(&self) {
        self.sources.invalidate_all();
        self.scan_results.invalidate_all();
    }

    pub async fn stats(&self) -> MemoryCacheStats {
        MemoryCacheStats {
            entries: self.sources.entry_count() + self.scan_results.entry_count(),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Default)]
pub struct MemoryCacheStats {
    pub entries: u64,
    pub hits: u64,
    pub misses: u64,
}
crates/scpf-core/src/cache/persistent.rs
Rust

use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::Result;
use crate::{SourceCode, ScanMatch};

pub struct PersistentCache {
    conn: Mutex<Connection>,
}

impl PersistentCache {
    pub fn new() -> Result<Self> {
        let path = Self::default_path();
        Self::with_path(path)
    }

    pub fn with_path(path: PathBuf) -> Result<Self> {
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;
        
        // Initialize schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sources (
                key TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                contract_name TEXT,
                compiler_version TEXT,
                abi TEXT,
                provider TEXT,
                created_at INTEGER DEFAULT (unixepoch())
            );
            
            CREATE TABLE IF NOT EXISTS scan_results (
                key TEXT PRIMARY KEY,
                results TEXT NOT NULL,
                created_at INTEGER DEFAULT (unixepoch())
            );
            
            CREATE INDEX IF NOT EXISTS idx_sources_created ON sources(created_at);
            CREATE INDEX IF NOT EXISTS idx_results_created ON scan_results(created_at);
            "
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn default_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("scpf")
            .join("cache.db")
    }

    pub async fn get_source(&self, key: &str) -> Result<Option<SourceCode>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare_cached(
            "SELECT source, contract_name, compiler_version, abi, provider FROM sources WHERE key = ?"
        )?;

        let result = stmt.query_row(params![key], |row| {
            Ok(SourceCode {
                source: row.get(0)?,
                files: vec![],
                contract_name: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                compiler_version: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                abi: row.get(3)?,
                bytecode: None,
                provider: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
            })
        });

        match result {
            Ok(source) => Ok(Some(source)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn set_source(&self, key: &str, source: &SourceCode) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO sources (key, source, contract_name, compiler_version, abi, provider)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                key,
                &source.source,
                &source.contract_name,
                &source.compiler_version,
                &source.abi,
                &source.provider,
            ],
        )?;

        Ok(())
    }

    pub async fn get_scan_result(&self, key: &str) -> Result<Option<Vec<ScanMatch>>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare_cached(
            "SELECT results FROM scan_results WHERE key = ?"
        )?;

        let result: std::result::Result<String, _> = stmt.query_row(params![key], |row| row.get(0));

        match result {
            Ok(json) => {
                let matches: Vec<ScanMatch> = serde_json::from_str(&json)?;
                Ok(Some(matches))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn set_scan_result(&self, key: &str, matches: &[ScanMatch]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let json = serde_json::to_string(matches)?;
        
        conn.execute(
            "INSERT OR REPLACE INTO scan_results (key, results) VALUES (?, ?)",
            params![key, json],
        )?;

        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("DELETE FROM sources; DELETE FROM scan_results;")?;
        Ok(())
    }

    pub async fn vacuum(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("VACUUM;")?;
        Ok(())
    }

    pub async fn prune(&self, max_age_seconds: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let threshold = chrono::Utc::now().timestamp() - max_age_seconds;
        
        let deleted_sources = conn.execute(
            "DELETE FROM sources WHERE created_at < ?",
            params![threshold],
        )?;
        
        let deleted_results = conn.execute(
            "DELETE FROM scan_results WHERE created_at < ?",
            params![threshold],
        )?;

        Ok(deleted_sources + deleted_results)
    }
}
crates/scpf-core/src/utils/mod.rs
Rust

//! Utility modules

mod rate_limiter;
mod retry;
mod hash;

pub use rate_limiter::RateLimiter;
pub use retry::RetryPolicy;
pub use hash::{hash_source, hash_address};
crates/scpf-core/src/utils/rate_limiter.rs
Rust

use std::num::NonZeroU32;
use governor::{Quota, RateLimiter as Governor, state::{NotKeyed, InMemoryState}, clock::DefaultClock, middleware::NoOpMiddleware};
use std::sync::Arc;

pub struct RateLimiter {
    limiter: Arc<Governor<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>,
}

impl RateLimiter {
    pub fn new(requests_per_second: f64) -> Self {
        let rps = requests_per_second.max(0.1) as u32;
        let quota = Quota::per_second(NonZeroU32::new(rps).unwrap_or(NonZeroU32::new(1).unwrap()));
        let limiter = Arc::new(Governor::direct(quota));

        Self { limiter }
    }

    pub async fn acquire(&self) {
        self.limiter.until_ready().await;
    }

    pub fn try_acquire(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            limiter: Arc::clone(&self.limiter),
        }
    }
}
crates/scpf-core/src/utils/retry.rs
Rust

use std::future::Future;
use std::time::Duration;
use backoff::{ExponentialBackoff, Error as BackoffError};

use crate::error::{Error, Result};

pub struct RetryPolicy {
    max_retries: u32,
    initial_delay: Duration,
}

impl RetryPolicy {
    pub fn new(max_retries: u32, initial_delay: Duration) -> Self {
        Self {
            max_retries,
            initial_delay,
        }
    }

    pub async fn retry<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let backoff = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(60)),
            initial_interval: self.initial_delay,
            max_interval: Duration::from_secs(30),
            multiplier: 2.0,
            ..Default::default()
        };

        let mut attempts = 0;

        backoff::future::retry(backoff, || async {
            attempts += 1;
            
            match operation().await {
                Ok(result) => Ok(result),
                Err(e) if e.is_retriable() && attempts <= self.max_retries => {
                    tracing::debug!("Retry attempt {} after error: {}", attempts, e);
                    Err(BackoffError::transient(e))
                }
                Err(e) => Err(BackoffError::permanent(e)),
            }
        })
        .await
        .map_err(|e| match e {
            BackoffError::Permanent(e) => e,
            BackoffError::Transient { err, .. } => err,
        })
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(3, Duration::from_secs(1))
    }
}
crates/scpf-core/src/utils/hash.rs
Rust

use sha2::{Sha256, Digest};
use xxhash_rust::xxh3::xxh3_64;

/// Fast hash for source code (for deduplication)
pub fn hash_source(source: &str) -> u64 {
    xxh3_64(source.as_bytes())
}

/// Hash for address normalization
pub fn hash_address(address: &str, chain_id: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(chain_id.to_le_bytes());
    hasher.update(address.to_lowercase().as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..16])
}

/// Create a cache key
pub fn cache_key(address: &str, chain_id: u64) -> String {
    format!("{}:{}", chain_id, address.to_lowercase())
}
Now let's create the CLI crate:

crates/scpf-cli/Cargo.toml
toml

[package]
name = "scpf-cli"
description = "CLI for Smart Contract Pattern Finder"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[[bin]]
name = "scpf"
path = "src/main.rs"

[dependencies]
# Internal crates
scpf-core = { path = "../scpf-core" }
scpf-types = { path = "../scpf-types" }

# Async
tokio = { workspace = true }
futures = { workspace = true }

# CLI
clap = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }

# Error handling
anyhow = { workspace = true }

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

# Time
chrono = { workspace = true }

# Console output
colored = { workspace = true }
console = { workspace = true }
indicatif = { workspace = true }
dialoguer = { workspace = true }
tabled = { workspace = true }

# File watching
notify = { workspace = true }

# Dirs
dirs = { workspace = true }

[dev-dependencies]
tempfile = "3.9"
crates/scpf-cli/src/main.rs
Rust

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod cli;
mod commands;
mod output;
mod notify;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI
    let cli = Cli::parse();

    // Setup logging
    setup_logging(cli.verbose);

    // Run command
    match cli.command {
        Commands::Scan(args) => commands::scan::run(args).await,
        Commands::Init(args) => commands::init::run(args).await,
        Commands::New(args) => commands::new_template::run(args).await,
        Commands::Test(args) => commands::test::run(args).await,
        Commands::Watch(args) => commands::watch::run(args).await,
        Commands::Templates(args) => commands::templates::run(args).await,
    }
}

fn setup_logging(verbose: u8) {
    let filter = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(env_filter)
        .init();
}
crates/scpf-cli/src/cli.rs
Rust

use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "scpf")]
#[command(author, version, about = "Smart Contract Pattern Finder", long_about = None)]
pub struct Cli {
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Config file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan contracts for patterns
    Scan(ScanArgs),
    
    /// Initialize a new project
    Init(InitArgs),
    
    /// Create a new template
    New(NewTemplateArgs),
    
    /// Test templates against sample code
    Test(TestArgs),
    
    /// Watch for new verified contracts
    Watch(WatchArgs),
    
    /// List and manage templates
    Templates(TemplatesArgs),
}

#[derive(Args)]
pub struct ScanArgs {
    /// Contract addresses to scan
    #[arg(required_unless_present = "file")]
    pub addresses: Vec<String>,

    /// File containing addresses (one per line)
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    /// Blockchain network
    #[arg(short = 'n', long, default_value = "ethereum")]
    pub chain: String,

    /// Template directory
    #[arg(short, long)]
    pub templates: Option<PathBuf>,

    /// Scan profile to use
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Filter by tags (comma-separated)
    #[arg(long)]
    pub tags: Option<String>,

    /// Exclude tags (comma-separated)
    #[arg(long)]
    pub exclude_tags: Option<String>,

    /// Minimum severity to report
    #[arg(long)]
    pub min_severity: Option<String>,

    /// Output format (console, json, sarif)
    #[arg(short, long, default_value = "console")]
    pub output: String,

    /// Output file path
    #[arg(short = 'O', long)]
    pub output_file: Option<PathBuf>,

    /// Context lines around matches
    #[arg(long, default_value = "3")]
    pub context: usize,

    /// Include source code in output
    #[arg(long)]
    pub include_source: bool,

    /// Stop after first match
    #[arg(long)]
    pub first: bool,

    /// Concurrency level
    #[arg(long, default_value = "10")]
    pub concurrency: usize,

    /// Disable cache
    #[arg(long)]
    pub no_cache: bool,

    /// Silent mode (no progress)
    #[arg(short, long)]
    pub silent: bool,
}

#[derive(Args)]
pub struct InitArgs {
    /// Project directory (default: current)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Skip interactive prompts
    #[arg(short, long)]
    pub yes: bool,

    /// Include example templates
    #[arg(long)]
    pub with_examples: bool,
}

#[derive(Args)]
pub struct NewTemplateArgs {
    /// Template ID
    pub id: String,

    /// Output directory
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Interactive wizard
    #[arg(short, long)]
    pub interactive: bool,

    /// Template severity
    #[arg(long, default_value = "info")]
    pub severity: String,

    /// Template tags (comma-separated)
    #[arg(long)]
    pub tags: Option<String>,
}

#[derive(Args)]
pub struct TestArgs {
    /// Template files to test
    pub templates: Vec<PathBuf>,

    /// Sample Solidity file to test against
    #[arg(short, long)]
    pub sample: Option<PathBuf>,

    /// Sample code inline
    #[arg(long)]
    pub code: Option<String>,

    /// Run all templates in directory
    #[arg(short, long)]
    pub all: bool,

    /// Validate only (don't run)
    #[arg(long)]
    pub validate_only: bool,
}

#[derive(Args)]
pub struct WatchArgs {
    /// Chains to watch (comma-separated)
    #[arg(short, long, default_value = "ethereum")]
    pub chains: String,

    /// Check interval in seconds
    #[arg(short, long, default_value = "60")]
    pub interval: u64,

    /// Number of contracts per check
    #[arg(long, default_value = "10")]
    pub batch_size: usize,

    /// Discord webhook URL
    #[arg(long, env = "DISCORD_WEBHOOK_URL")]
    pub discord: Option<String>,

    /// Slack webhook URL
    #[arg(long, env = "SLACK_WEBHOOK_URL")]
    pub slack: Option<String>,

    /// Telegram bot token
    #[arg(long, env = "TELEGRAM_BOT_TOKEN")]
    pub telegram_token: Option<String>,

    /// Telegram chat ID
    #[arg(long, env = "TELEGRAM_CHAT_ID")]
    pub telegram_chat: Option<String>,

    /// Template directory
    #[arg(short, long)]
    pub templates: Option<PathBuf>,

    /// Profile to use
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Output directory for results
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Args)]
pub struct TemplatesArgs {
    #[command(subcommand)]
    pub command: TemplatesCommands,
}

#[derive(Subcommand)]
pub enum TemplatesCommands {
    /// List all templates
    List {
        /// Template directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Filter by severity
        #[arg(long)]
        severity: Option<String>,
    },

    /// Show template details
    Show {
        /// Template ID
        id: String,

        /// Template directory
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Validate templates
    Validate {
        /// Template directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Strict validation
        #[arg(long)]
        strict: boo