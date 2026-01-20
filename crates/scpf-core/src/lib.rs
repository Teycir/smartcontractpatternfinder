pub mod scanner;
pub mod semantic;
pub mod template;
pub mod fetcher;
pub mod cache;
pub mod regex_validator;
pub mod zeroday;
pub mod dataflow;
pub mod composition;
pub mod risk_scoring;
pub mod risk_scorer;
pub mod dependency_analyzer;
pub mod pattern_validator;

pub use scanner::Scanner;
pub use semantic::SemanticScanner;
pub use template::TemplateLoader;
pub use fetcher::ContractFetcher;
pub use cache::Cache;
pub use regex_validator::RegexValidator;
pub use zeroday::ZeroDayFetcher;
pub use dataflow::DataFlowAnalysis;
pub use composition::TemplateComposer;
pub use risk_scoring::{RiskScorer as OldRiskScorer, RiskAssessment, RiskLevel as OldRiskLevel, RiskConfig};
pub use risk_scorer::{RiskScorer, RiskScore, RiskLevel, SeverityBreakdown, RiskFactor};
pub use dependency_analyzer::{DependencyAnalyzer, DependencyGraph, AttackSurface};
pub use pattern_validator::PatternValidator;

#[cfg(test)]
mod tests;
