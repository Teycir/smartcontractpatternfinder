pub mod scanner;
pub mod semantic;
pub mod template;
pub mod fetcher;
pub mod cache;
pub mod regex_validator;
pub mod zeroday;
pub mod dataflow;

pub use scanner::Scanner;
pub use semantic::SemanticScanner;
pub use template::TemplateLoader;
pub use fetcher::ContractFetcher;
pub use cache::Cache;
pub use regex_validator::RegexValidator;
pub use zeroday::ZeroDayFetcher;
pub use dataflow::DataFlowAnalysis;

#[cfg(test)]
mod tests;
