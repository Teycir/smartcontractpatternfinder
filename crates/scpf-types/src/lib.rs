use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod chain;
mod api_key_config;
mod language;
mod semantic;

pub use chain::Chain;
pub use api_key_config::ApiKeyConfig;
pub use language::Language;
pub use semantic::*;

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
    #[serde(default)]
    pub kind: PatternKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PatternKind {
    #[default]
    Regex,
    Semantic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Exploitability {
    Trivial,
    Easy,
    Medium,
    Hard,
    Impossible,
}

impl Exploitability {
    pub fn multiplier(&self) -> f32 {
        match self {
            Exploitability::Trivial => 3.0,
            Exploitability::Easy => 2.0,
            Exploitability::Medium => 1.5,
            Exploitability::Hard => 1.0,
            Exploitability::Impossible => 0.5,
        }
    }

    pub fn from_pattern(pattern_id: &str) -> Self {
        match pattern_id {
            // TRIVIAL - 100% PoC success
            "unprotected-selfdestruct" | "unprotected-selfdestruct-fixed" => Self::Trivial,
            "missing-access-control" | "missing-access-control-fixed" => Self::Trivial,
            "reentrancy-pattern" | "reentrancy-pattern-fixed" | "state-after-call" => Self::Trivial,
            
            // EASY - 85-90% PoC success
            "tx-origin-auth" | "tx-origin-auth-fixed" | "tx-origin-regex" => Self::Easy,
            "delegatecall-user-input" => Self::Easy,
            "unchecked-call" | "unchecked-call-fixed" | "unchecked-call-return" => Self::Easy,
            
            // MEDIUM - 50-70% PoC success
            "timestamp-dependence" | "timestamp-dependence-fixed" => Self::Medium,
            "integer-overflow" | "overflow-mul-div" => Self::Medium,
            "frontrun-vulnerable" => Self::Medium,
            
            // HARD - 30-40% PoC success
            "single-source-price" => Self::Hard,
            "flash-loan-vulnerable" => Self::Hard,
            "oracle-manipulation" => Self::Hard,
            
            // IMPOSSIBLE - Cannot generate reliable PoC
            _ => Self::Impossible,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Exploitability::Trivial => "TRIVIAL",
            Exploitability::Easy => "EASY",
            Exploitability::Medium => "MEDIUM",
            Exploitability::Hard => "HARD",
            Exploitability::Impossible => "IMPOSSIBLE",
        }
    }

    pub fn success_rate(&self) -> &'static str {
        match self {
            Exploitability::Trivial => "95-100%",
            Exploitability::Easy => "85-90%",
            Exploitability::Medium => "50-70%",
            Exploitability::Hard => "30-40%",
            Exploitability::Impossible => "0-10%",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Match {
    pub template_id: String,
    pub pattern_id: String,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub column: usize,
    pub matched_text: String,
    pub context: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_snippet: Option<CodeSnippet>,
    pub severity: Severity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_byte: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_byte: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeSnippet {
    pub before: String,
    pub vulnerable_line: String,
    pub after: String,
    pub line_start: usize,
}

impl Match {
    /// Calculate risk score for this match
    /// 
    /// Formula:
    /// - CRITICAL: 100 points
    /// - HIGH: 10 points
    /// - MEDIUM: 3 points
    /// - LOW: 1 point
    /// - INFO: 0 points
    pub fn risk_score(&self) -> u32 {
        match self.severity {
            Severity::Critical => 100,
            Severity::High => 10,
            Severity::Medium => 3,
            Severity::Low => 1,
            Severity::Info => 0,
        }
    }

    /// Calculate exploitability score for PoC generation priority
    /// Formula: Base Severity × PoC Difficulty Multiplier
    pub fn exploitability_score(&self) -> f32 {
        let base = self.risk_score() as f32;
        let exploitability = Exploitability::from_pattern(&self.pattern_id);
        base * exploitability.multiplier()
    }

    /// Get exploitability level for this vulnerability
    pub fn exploitability(&self) -> Exploitability {
        Exploitability::from_pattern(&self.pattern_id)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub address: String,
    pub chain: String,
    pub matches: Vec<Match>,
    pub scan_time_ms: u64,
}

impl ScanResult {
    /// Calculate total risk score for all matches
    /// 
    /// Formula: Σ(severity_weight × count)
    /// Weights: CRITICAL=100, HIGH=10, MEDIUM=3, LOW=1, INFO=0
    pub fn total_risk_score(&self) -> u32 {
        self.matches.iter().map(|m| m.risk_score()).sum()
    }

    /// Get risk level based on total score
    /// 
    /// Thresholds:
    /// - 0: None ✅
    /// - 1-100: Low ✅
    /// - 101-500: Medium ⚠️
    /// - 501-2000: High 🔴
    /// - 2000+: Critical 🚨
    pub fn risk_level(&self) -> &'static str {
        match self.total_risk_score() {
            0 => "None",
            1..=100 => "Low",
            101..=500 => "Medium",
            501..=2000 => "High",
            _ => "Critical",
        }
    }

    /// Get risk level emoji
    pub fn risk_emoji(&self) -> &'static str {
        match self.total_risk_score() {
            0 => "✅",
            1..=100 => "✅",
            101..=500 => "⚠️",
            501..=2000 => "🔴",
            _ => "🚨",
        }
    }

    /// Get severity breakdown for risk calculation
    pub fn severity_breakdown(&self) -> SeverityBreakdown {
        let mut breakdown = SeverityBreakdown::default();
        for m in &self.matches {
            match m.severity {
                Severity::Critical => breakdown.critical += 1,
                Severity::High => breakdown.high += 1,
                Severity::Medium => breakdown.medium += 1,
                Severity::Low => breakdown.low += 1,
                Severity::Info => breakdown.info += 1,
            }
        }
        breakdown
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SeverityBreakdown {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
}

impl SeverityBreakdown {
    /// Calculate risk score from breakdown
    pub fn risk_score(&self) -> u32 {
        (self.critical as u32 * 100)
            + (self.high as u32 * 10)
            + (self.medium as u32 * 3)
            + (self.low as u32)
    }

    /// Format breakdown as string
    pub fn format(&self) -> String {
        format!(
            "CRITICAL: {} × 100 = {}\n  HIGH: {} × 10 = {}\n  MEDIUM: {} × 3 = {}",
            self.critical,
            self.critical * 100,
            self.high,
            self.high * 10,
            self.medium,
            self.medium * 3
        )
    }
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

#[cfg(test)]
mod tests;
