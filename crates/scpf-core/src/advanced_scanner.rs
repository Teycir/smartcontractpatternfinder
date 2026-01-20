use crate::dependency_analyzer::DependencyAnalyzer;
use crate::exploit_gen::{ExploitGenerator, ExploitTemplate, VulnerabilityInfo};
use crate::risk_scorer::{RiskScore, RiskScorer};
use crate::state_analysis::{StateAnalyzer, StateViolation};
use crate::taint::{TaintAnalyzer, TaintFlow};
use crate::value_flow::{ValueExtractionPath, ValueFlowAnalyzer};
use scpf_types::Match;

#[derive(Debug, Clone)]
pub struct AdvancedReport {
    pub vulnerabilities: Vec<CombinedVulnerability>,
    pub exploits: Vec<ExploitTemplate>,
    pub risk_score: RiskScore,
    pub taint_summary: TaintSummary,
    pub value_flow_summary: ValueFlowSummary,
    pub state_violations: Vec<StateViolation>,
}

#[derive(Debug, Clone)]
pub struct CombinedVulnerability {
    pub id: String,
    pub severity: String,
    pub confidence: f64,
    pub sources: Vec<String>,
    pub description: String,
    pub exploit_scenario: String,
}

#[derive(Debug, Clone)]
pub struct TaintSummary {
    pub total_flows: usize,
    pub high_risk_flows: usize,
    pub critical_sinks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValueFlowSummary {
    pub total_paths: usize,
    pub extraction_types: Vec<String>,
    pub max_profit_path: Option<String>,
}

pub struct AdvancedScanner {
    taint_analyzer: TaintAnalyzer,
    value_flow_analyzer: ValueFlowAnalyzer,
    state_analyzer: StateAnalyzer,
    exploit_generator: ExploitGenerator,
    risk_scorer: RiskScorer,
    dependency_analyzer: DependencyAnalyzer,
}

impl AdvancedScanner {
    pub fn new() -> Self {
        Self {
            taint_analyzer: TaintAnalyzer::new(),
            value_flow_analyzer: ValueFlowAnalyzer::new(),
            state_analyzer: StateAnalyzer::new(),
            exploit_generator: ExploitGenerator::new(),
            risk_scorer: RiskScorer::new(),
            dependency_analyzer: DependencyAnalyzer::new(),
        }
    }

    pub fn deep_analysis(&mut self, findings: &[Match]) -> AdvancedReport {
        let taint_flows = self.taint_analyzer.analyze();
        let value_paths = self.value_flow_analyzer.analyze();
        let state_violations = self.state_analyzer.get_violations().to_vec();

        let vulnerabilities = self.combine_findings(findings, &taint_flows, &value_paths);

        let exploits: Vec<_> = vulnerabilities
            .iter()
            .filter(|v| v.confidence > 0.7)
            .map(|v| {
                self.exploit_generator.generate(&VulnerabilityInfo {
                    pattern_id: v.id.clone(),
                    contract: "Target".to_string(),
                    function: "vulnerable".to_string(),
                    vuln_type: v.severity.clone(),
                })
            })
            .collect();

        let risk_score = self.risk_scorer.calculate(findings);

        AdvancedReport {
            vulnerabilities,
            exploits,
            risk_score,
            taint_summary: self.summarize_taint(&taint_flows),
            value_flow_summary: self.summarize_value_flow(&value_paths),
            state_violations,
        }
    }

    fn combine_findings(
        &self,
        findings: &[Match],
        taint_flows: &[TaintFlow],
        value_paths: &[ValueExtractionPath],
    ) -> Vec<CombinedVulnerability> {
        let mut vulnerabilities = Vec::new();

        for finding in findings {
            let mut confidence = 0.5;
            let mut sources = vec!["Pattern Match".to_string()];

            for flow in taint_flows {
                if self.matches_taint_flow(finding, flow) {
                    confidence += 0.2;
                    sources.push("Taint Analysis".to_string());
                    break;
                }
            }

            for path in value_paths {
                if self.matches_value_path(finding, path) {
                    confidence += 0.2;
                    sources.push("Value Flow Analysis".to_string());
                    break;
                }
            }

            vulnerabilities.push(CombinedVulnerability {
                id: finding.pattern_id.clone(),
                severity: format!("{:?}", finding.severity),
                confidence: (confidence as f64).min(1.0),
                sources,
                description: finding.message.clone(),
                exploit_scenario: self.generate_scenario(finding),
            });
        }

        vulnerabilities.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        vulnerabilities
    }

    fn matches_taint_flow(&self, finding: &Match, _flow: &TaintFlow) -> bool {
        finding.pattern_id.contains("reentrancy")
            || finding.pattern_id.contains("delegatecall")
            || finding.pattern_id.contains("tx-origin")
    }

    fn matches_value_path(&self, finding: &Match, _path: &ValueExtractionPath) -> bool {
        finding.pattern_id.contains("transfer")
            || finding.pattern_id.contains("balance")
            || finding.pattern_id.contains("withdraw")
    }

    fn generate_scenario(&self, finding: &Match) -> String {
        format!(
            "Vulnerability detected at {}:{}. {}",
            finding.file_path.display(),
            finding.line_number,
            finding.message
        )
    }

    fn summarize_taint(&self, flows: &[TaintFlow]) -> TaintSummary {
        let high_risk_flows = flows
            .iter()
            .filter(|f| f.exploitability.score >= 0.7)
            .count();

        let critical_sinks: Vec<String> = flows
            .iter()
            .filter(|f| f.exploitability.score >= 0.8)
            .map(|f| format!("{:?}", f.sink))
            .collect();

        TaintSummary {
            total_flows: flows.len(),
            high_risk_flows,
            critical_sinks,
        }
    }

    fn summarize_value_flow(&self, paths: &[ValueExtractionPath]) -> ValueFlowSummary {
        let extraction_types: Vec<String> = paths
            .iter()
            .map(|p| format!("{:?}", p.extraction_type))
            .collect();

        let max_profit_path = paths
            .iter()
            .max_by_key(|p| p.exploit_steps.len())
            .map(|p| p.profit_calculation.clone());

        ValueFlowSummary {
            total_paths: paths.len(),
            extraction_types,
            max_profit_path,
        }
    }

    pub fn export_report(&self, report: &AdvancedReport) -> String {
        let mut output = String::from("# Advanced Security Analysis Report\n\n");

        output.push_str(&format!(
            "## Risk Score: {:.1}/100\n",
            report.risk_score.total_score
        ));
        output.push_str(&format!(
            "**Risk Level**: {:?}\n\n",
            report.risk_score.risk_level
        ));

        output.push_str(&format!(
            "## Vulnerabilities Found: {}\n\n",
            report.vulnerabilities.len()
        ));

        for (i, vuln) in report.vulnerabilities.iter().enumerate() {
            output.push_str(&format!(
                "### {}. {} (Confidence: {:.0}%)\n",
                i + 1,
                vuln.id,
                vuln.confidence * 100.0
            ));
            output.push_str(&format!("**Severity**: {}\n", vuln.severity));
            output.push_str(&format!("**Sources**: {}\n", vuln.sources.join(", ")));
            output.push_str(&format!("**Description**: {}\n\n", vuln.description));
        }

        output.push_str(&format!("\n## Taint Analysis\n"));
        output.push_str(&format!(
            "- Total flows: {}\n",
            report.taint_summary.total_flows
        ));
        output.push_str(&format!(
            "- High-risk flows: {}\n",
            report.taint_summary.high_risk_flows
        ));

        output.push_str(&format!("\n## Value Flow Analysis\n"));
        output.push_str(&format!(
            "- Total paths: {}\n",
            report.value_flow_summary.total_paths
        ));

        output.push_str(&format!(
            "\n## Exploits Generated: {}\n",
            report.exploits.len()
        ));
        for exploit in &report.exploits {
            output.push_str(&format!(
                "- {} ({})\n",
                exploit.name, exploit.vulnerability_type
            ));
        }

        output.push_str("\n## Recommendations\n");
        for rec in &report.risk_score.recommendations {
            output.push_str(&format!("- {}\n", rec));
        }

        output
    }
}

impl Default for AdvancedScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scpf_types::{Match, Severity};
    use std::path::PathBuf;

    #[test]
    fn test_advanced_scanner() {
        let mut scanner = AdvancedScanner::new();

        let findings = vec![Match {
            template_id: "test".to_string(),
            pattern_id: "reentrancy-pattern".to_string(),
            file_path: PathBuf::from("test.sol"),
            line_number: 10,
            column: 5,
            matched_text: "call".to_string(),
            context: "vulnerable code".to_string(),
            code_snippet: None,
            severity: Severity::High,
            message: "Reentrancy detected".to_string(),
            start_byte: None,
            end_byte: None,
        }];

        let report = scanner.deep_analysis(&findings);
        assert!(!report.vulnerabilities.is_empty());
    }
}
