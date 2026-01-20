use scpf_types::Match;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PocCandidate {
    pub id: String,
    pub pattern_id: String,
    pub confidence: f64,
    pub exploitability: f64,
    pub validation_score: f64,
    pub context: PocContext,
    pub priority: PocPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PocContext {
    pub source_code: String,
    pub vulnerable_function: String,
    pub line_number: usize,
    pub matched_code: String,
    pub contract_name: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PocPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

pub struct PocStager {
    min_confidence: f64,
    min_exploitability: f64,
}

impl PocStager {
    pub fn new() -> Self {
        Self {
            min_confidence: 0.6,
            min_exploitability: 0.5,
        }
    }

    pub fn stage_for_poc(
        &self,
        findings: &[Match],
        source_code: &str,
        contract_name: &str,
    ) -> Vec<PocCandidate> {
        findings
            .iter()
            .filter_map(|f| self.evaluate_candidate(f, source_code, contract_name))
            .collect()
    }

    fn evaluate_candidate(
        &self,
        finding: &Match,
        source_code: &str,
        contract_name: &str,
    ) -> Option<PocCandidate> {
        let confidence = self.calculate_confidence(finding, source_code);
        let exploitability = self.calculate_exploitability(finding);
        let validation_score = self.validate_finding(finding, source_code);

        if confidence < self.min_confidence || exploitability < self.min_exploitability {
            return None;
        }

        let context = self.extract_context(finding, source_code, contract_name);
        let priority = self.determine_priority(confidence, exploitability, validation_score);

        Some(PocCandidate {
            id: format!("{}:{}", finding.pattern_id, finding.line_number),
            pattern_id: finding.pattern_id.clone(),
            confidence,
            exploitability,
            validation_score,
            context,
            priority,
        })
    }

    fn calculate_confidence(&self, finding: &Match, source_code: &str) -> f64 {
        let mut score = 0.5_f64;

        if finding.code_snippet.is_some() {
            score += 0.1;
        }

        if self.has_vulnerable_pattern(finding, source_code) {
            score += 0.2;
        }

        if self.lacks_protection(finding, source_code) {
            score += 0.2;
        }

        score.min(1.0)
    }

    fn calculate_exploitability(&self, finding: &Match) -> f64 {
        match finding.pattern_id.as_str() {
            id if id.contains("unprotected") || id.contains("missing-access") => 0.9,
            id if id.contains("reentrancy") || id.contains("delegatecall") => 0.8,
            id if id.contains("tx-origin") || id.contains("unchecked-call") => 0.7,
            id if id.contains("timestamp") || id.contains("overflow") => 0.6,
            _ => 0.5,
        }
    }

    fn validate_finding(&self, finding: &Match, source_code: &str) -> f64 {
        let mut score = 0.0;

        if self.has_state_change(finding, source_code) {
            score += 0.3;
        }

        if self.has_external_call(finding, source_code) {
            score += 0.3;
        }

        if self.has_value_transfer(finding, source_code) {
            score += 0.4;
        }

        score
    }

    fn has_vulnerable_pattern(&self, finding: &Match, source_code: &str) -> bool {
        let patterns = [
            r"\.call\{value:",
            r"\.delegatecall\(",
            r"tx\.origin",
            r"selfdestruct\(",
            r"suicide\(",
        ];

        let context = self.get_function_context(finding, source_code);
        patterns.iter().any(|p| context.contains(p))
    }

    fn lacks_protection(&self, finding: &Match, source_code: &str) -> bool {
        let context = self.get_function_context(finding, source_code);
        let protections = ["onlyOwner", "require(", "assert(", "nonReentrant"];

        !protections.iter().any(|p| context.contains(p))
    }

    fn has_state_change(&self, finding: &Match, source_code: &str) -> bool {
        let context = self.get_function_context(finding, source_code);
        context.contains("=") && !context.contains("==")
    }

    fn has_external_call(&self, finding: &Match, source_code: &str) -> bool {
        let context = self.get_function_context(finding, source_code);
        context.contains(".call")
            || context.contains(".delegatecall")
            || context.contains(".transfer")
    }

    fn has_value_transfer(&self, finding: &Match, source_code: &str) -> bool {
        let context = self.get_function_context(finding, source_code);
        context.contains("value:") || context.contains(".transfer(") || context.contains(".send(")
    }

    fn get_function_context(&self, finding: &Match, source_code: &str) -> String {
        let lines: Vec<&str> = source_code.lines().collect();
        let start = finding.line_number.saturating_sub(5);
        let end = (finding.line_number + 5).min(lines.len());

        lines[start..end].join("\n")
    }

    fn extract_context(
        &self,
        finding: &Match,
        source_code: &str,
        contract_name: &str,
    ) -> PocContext {
        let function_name = self.extract_function_name(finding, source_code);
        let matched_code = self.get_function_context(finding, source_code);

        PocContext {
            source_code: source_code.to_string(),
            vulnerable_function: function_name,
            line_number: finding.line_number,
            matched_code,
            contract_name: contract_name.to_string(),
            dependencies: vec![],
        }
    }

    fn extract_function_name(&self, finding: &Match, source_code: &str) -> String {
        let lines: Vec<&str> = source_code.lines().collect();

        for i in (0..finding.line_number).rev() {
            if i >= lines.len() {
                continue;
            }
            let line = lines[i];
            if line.contains("function ") {
                if let Some(start) = line.find("function ") {
                    let rest = &line[start + 9..];
                    if let Some(end) = rest.find('(') {
                        return rest[..end].trim().to_string();
                    }
                }
            }
        }

        "unknown".to_string()
    }

    fn determine_priority(
        &self,
        confidence: f64,
        exploitability: f64,
        validation: f64,
    ) -> PocPriority {
        let combined = (confidence + exploitability + validation) / 3.0;

        if combined >= 0.8 {
            PocPriority::Critical
        } else if combined >= 0.7 {
            PocPriority::High
        } else if combined >= 0.6 {
            PocPriority::Medium
        } else {
            PocPriority::Low
        }
    }

    pub fn export_for_ai(&self, candidates: &[PocCandidate]) -> String {
        serde_json::to_string_pretty(candidates).unwrap_or_default()
    }
}

impl Default for PocStager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scpf_types::Severity;
    use std::path::PathBuf;

    #[test]
    fn test_poc_staging() {
        let stager = PocStager::new();
        let finding = Match {
            template_id: "test".to_string(),
            pattern_id: "reentrancy-unprotected".to_string(),
            file_path: PathBuf::from("test.sol"),
            line_number: 10,
            column: 0,
            matched_text: "call{value:".to_string(),
            context: "test".to_string(),
            severity: Severity::High,
            message: "Reentrancy".to_string(),
            start_byte: None,
            end_byte: None,
            code_snippet: Some("call{value: amount}".to_string()),
        };

        let source = "function withdraw() public { msg.sender.call{value: balance}(\"\"); }";
        let candidates = stager.stage_for_poc(&[finding], source, "Test");

        assert!(!candidates.is_empty());
        assert!(candidates[0].confidence >= 0.6);
    }
}
