use scpf_types::{Match, ScanResult, Severity};
use std::collections::HashMap;

/// Risk scoring weights and thresholds
#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub severity_weights: HashMap<Severity, u32>,
    pub pattern_multipliers: HashMap<String, f32>,
    pub composition_bonus: u32,
    pub thresholds: RiskThresholds,
}

#[derive(Debug, Clone)]
pub struct RiskThresholds {
    pub low: u32,
    pub medium: u32,
    pub high: u32,
    pub critical: u32,
}

impl Default for RiskConfig {
    fn default() -> Self {
        let mut severity_weights = HashMap::new();
        severity_weights.insert(Severity::Info, 1);
        severity_weights.insert(Severity::Low, 3);
        severity_weights.insert(Severity::Medium, 7);
        severity_weights.insert(Severity::High, 15);
        severity_weights.insert(Severity::Critical, 30);

        Self {
            severity_weights,
            pattern_multipliers: HashMap::new(),
            composition_bonus: 10,
            thresholds: RiskThresholds {
                low: 5,
                medium: 15,
                high: 30,
                critical: 50,
            },
        }
    }
}

/// Detailed risk assessment
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub total_score: u32,
    pub risk_level: RiskLevel,
    pub severity_breakdown: HashMap<Severity, u32>,
    pub pattern_breakdown: HashMap<String, u32>,
    pub composition_score: u32,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::None => "None",
            RiskLevel::Low => "Low",
            RiskLevel::Medium => "Medium",
            RiskLevel::High => "High",
            RiskLevel::Critical => "Critical",
        }
    }
}

/// Risk scorer calculates weighted vulnerability scores
pub struct RiskScorer {
    config: RiskConfig,
}

impl RiskScorer {
    pub fn new(config: RiskConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(RiskConfig::default())
    }

    /// Calculate risk assessment for scan results
    pub fn assess(&self, result: &ScanResult) -> RiskAssessment {
        let mut severity_breakdown = HashMap::new();
        let mut pattern_breakdown = HashMap::new();
        let mut total_score = 0u32;

        for m in &result.matches {
            let base_score = self
                .config
                .severity_weights
                .get(&m.severity)
                .copied()
                .unwrap_or(1);
            let multiplier = self
                .config
                .pattern_multipliers
                .get(&m.pattern_id)
                .copied()
                .unwrap_or(1.0);
            let score = (base_score as f32 * multiplier) as u32;

            total_score += score;
            *severity_breakdown.entry(m.severity).or_insert(0) += score;
            *pattern_breakdown.entry(m.pattern_id.clone()).or_insert(0) += score;
        }

        let composition_score = self.calculate_composition_bonus(&result.matches);
        total_score += composition_score;

        let risk_level = self.determine_risk_level(total_score);
        let recommendations =
            self.generate_recommendations(&severity_breakdown, &pattern_breakdown);

        RiskAssessment {
            total_score,
            risk_level,
            severity_breakdown,
            pattern_breakdown,
            composition_score,
            recommendations,
        }
    }

    fn calculate_composition_bonus(&self, matches: &[Match]) -> u32 {
        let mut bonus = 0u32;

        // Reentrancy composition: external call + state change
        if self.has_pattern(matches, "external-call") && self.has_pattern(matches, "state-mutation")
        {
            bonus += self.config.composition_bonus;
        }

        // Access control: critical function without modifier
        if self.has_pattern(matches, "critical-function")
            && !self.has_pattern(matches, "access-modifier")
        {
            bonus += self.config.composition_bonus;
        }

        bonus
    }

    fn has_pattern(&self, matches: &[Match], pattern_id: &str) -> bool {
        matches.iter().any(|m| m.pattern_id.contains(pattern_id))
    }

    fn determine_risk_level(&self, score: u32) -> RiskLevel {
        if score == 0 {
            RiskLevel::None
        } else if score < self.config.thresholds.low {
            RiskLevel::Low
        } else if score < self.config.thresholds.medium {
            RiskLevel::Medium
        } else if score < self.config.thresholds.high {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        }
    }

    fn generate_recommendations(
        &self,
        severity: &HashMap<Severity, u32>,
        patterns: &HashMap<String, u32>,
    ) -> Vec<String> {
        let mut recs = Vec::new();

        if severity.get(&Severity::Critical).copied().unwrap_or(0) > 0 {
            recs.push("URGENT: Critical vulnerabilities detected. Do not deploy.".to_string());
        }

        if severity.get(&Severity::High).copied().unwrap_or(0) > 0 {
            recs.push("High severity issues require immediate attention.".to_string());
        }

        if patterns.contains_key("reentrancy") {
            recs.push("Implement checks-effects-interactions pattern.".to_string());
        }

        if patterns.contains_key("access-control") {
            recs.push("Add proper access control modifiers.".to_string());
        }

        recs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_match(severity: Severity, pattern_id: &str) -> Match {
        Match {
            template_id: "test".to_string(),
            pattern_id: pattern_id.to_string(),
            file_path: PathBuf::from("test.sol"),
            line_number: 1,
            column: 0,
            matched_text: "test".to_string(),
            context: "test".to_string(),
            severity,
            message: "test".to_string(),
            start_byte: None,
            end_byte: None,
        }
    }

    #[test]
    fn test_risk_scoring() {
        let scorer = RiskScorer::with_defaults();
        let result = ScanResult {
            address: "0x123".to_string(),
            chain: "ethereum".to_string(),
            matches: vec![
                create_test_match(Severity::Critical, "reentrancy"),
                create_test_match(Severity::High, "access-control"),
            ],
            scan_time_ms: 100,
        };

        let assessment = scorer.assess(&result);
        assert!(assessment.total_score > 0);
        assert!(assessment.risk_level >= RiskLevel::High);
    }

    #[test]
    fn test_composition_bonus() {
        let scorer = RiskScorer::with_defaults();
        let matches = vec![
            create_test_match(Severity::Medium, "external-call"),
            create_test_match(Severity::Medium, "state-mutation"),
        ];

        let bonus = scorer.calculate_composition_bonus(&matches);
        assert_eq!(bonus, scorer.config.composition_bonus);
    }
}
