use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub tags: Vec<String>,
    pub patterns: Vec<Pattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub pattern: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub template_id: String,
    pub pattern_id: String,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub column: usize,
    pub matched_text: String,
    pub context: String,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub address: String,
    pub chain: String,
    pub matches: Vec<Match>,
    pub scan_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub templates_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub concurrency: usize,
    pub timeout_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            templates_dir: PathBuf::from("templates"),
            cache_dir: PathBuf::from(".cache"),
            concurrency: 10,
            timeout_secs: 30,
        }
    }
}
