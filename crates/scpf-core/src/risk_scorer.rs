use scpf_types::{Match, Severity};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RiskScore {
    pub total_score: f64,
    pub severity_breakdown: SeverityBreakdown,
    pub risk_factors: Vec<RiskFactor>,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SeverityBreakdown {
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
    pub informational: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
    Minimal,
}

#[derive(Debug, Clone)]
pub struct RiskFactor {
    pub category: String,
    pub weight: f64,
    pub score: f64,
    pub details: String,
}

pub struct RiskScorer {
    weights: HashMap<String, f64>,
}

impl RiskScorer {
    pub fn new() -> Self {
        let mut weights = HashMap::new();

        weights.insert("critical".to_string(), 25.0);
        weights.insert("high".to_string(), 15.0);
        weights.insert("medium".to_string(), 8.0);
        weights.insert("low".to_string(), 3.0);
        weights.insert("info".to_string(), 1.0);

        weights.insert("reentrancy".to_string(), 1.5);
        weights.insert("access-control".to_string(), 1.4);
        weights.insert("oracle".to_string(), 1.3);
        weights.insert("upgrade".to_string(), 1.3);
        weights.insert("cryptography".to_string(), 1.4);
        weights.insert("governance".to_string(), 1.3);

        Self { weights }
    }

    pub fn calculate(&self, findings: &[Match]) -> RiskScore {
        let severity_breakdown = self.count_severities(findings);
        let mut score = self.calculate_severity_score(&severity_breakdown);
        let mut factors = Vec::new();

        let category_factor = self.calculate_category_factor(findings);
        factors.push(RiskFactor {
            category: "Category Risk".to_string(),
            weight: category_factor,
            score: score * category_factor,
            details: format!(
                "High-risk categories detected (multiplier: {:.2}x)",
                category_factor
            ),
        });
        score *= category_factor;

        let concentration_factor = self.calculate_concentration_factor(findings);
        factors.push(RiskFactor {
            category: "Issue Concentration".to_string(),
            weight: concentration_factor,
            score: score * concentration_factor,
            details: format!(
                "Multiple similar issues (multiplier: {:.2}x)",
                concentration_factor
            ),
        });
        score *= concentration_factor;

        let total_score = score.min(100.0).max(0.0);
        let recommendations = self.generate_recommendations(&severity_breakdown, total_score);

        RiskScore {
            total_score,
            severity_breakdown,
            risk_factors: factors,
            risk_level: self.score_to_level(total_score),
            recommendations,
        }
    }

    fn count_severities(&self, findings: &[Match]) -> SeverityBreakdown {
        let mut breakdown = SeverityBreakdown {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            informational: 0,
        };

        for finding in findings {
            match finding.severity {
                Severity::Critical => breakdown.critical += 1,
                Severity::High => breakdown.high += 1,
                Severity::Medium => breakdown.medium += 1,
                Severity::Low => breakdown.low += 1,
                Severity::Info => breakdown.informational += 1,
            }
        }

        breakdown
    }

    fn calculate_severity_score(&self, breakdown: &SeverityBreakdown) -> f64 {
        let critical_score =
            breakdown.critical as f64 * self.weights.get("critical").unwrap_or(&25.0);
        let high_score = breakdown.high as f64 * self.weights.get("high").unwrap_or(&15.0);
        let medium_score = breakdown.medium as f64 * self.weights.get("medium").unwrap_or(&8.0);
        let low_score = breakdown.low as f64 * self.weights.get("low").unwrap_or(&3.0);
        let info_score = breakdown.informational as f64 * self.weights.get("info").unwrap_or(&1.0);

        critical_score + high_score + medium_score + low_score + info_score
    }

    fn calculate_category_factor(&self, findings: &[Match]) -> f64 {
        let mut max_multiplier = 1.0;

        for finding in findings {
            let pattern_id = &finding.pattern_id;
            for (key, &multiplier) in &self.weights {
                if pattern_id.contains(key) {
                    if multiplier > max_multiplier {
                        max_multiplier = multiplier;
                    }
                }
            }
        }

        max_multiplier
    }

    fn calculate_concentration_factor(&self, findings: &[Match]) -> f64 {
        if findings.is_empty() {
            return 1.0;
        }

        let mut pattern_counts: HashMap<String, u32> = HashMap::new();
        for finding in findings {
            *pattern_counts
                .entry(finding.pattern_id.clone())
                .or_insert(0) += 1;
        }

        let max_count = pattern_counts.values().max().unwrap_or(&1);

        if *max_count > 10 {
            1.3
        } else if *max_count > 5 {
            1.2
        } else if *max_count > 3 {
            1.1
        } else {
            1.0
        }
    }

    fn score_to_level(&self, score: f64) -> RiskLevel {
        match score as u32 {
            80..=100 => RiskLevel::Critical,
            60..=79 => RiskLevel::High,
            40..=59 => RiskLevel::Medium,
            20..=39 => RiskLevel::Low,
            _ => RiskLevel::Minimal,
        }
    }

    fn generate_recommendations(&self, breakdown: &SeverityBreakdown, score: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if breakdown.critical > 0 {
            recommendations.push(format!(
                "🚨 CRITICAL: {} critical vulnerabilities found - DO NOT DEPLOY until fixed",
                breakdown.critical
            ));
        }

        if breakdown.high > 0 {
            recommendations.push(format!(
                "⚠️  HIGH: {} high-severity issues require immediate attention",
                breakdown.high
            ));
        }

        if score >= 80.0 {
            recommendations
                .push("❌ Risk Level: CRITICAL - Contract is not safe for production".to_string());
            recommendations.push("→ Conduct full security audit before deployment".to_string());
        } else if score >= 60.0 {
            recommendations
                .push("⚠️  Risk Level: HIGH - Major security concerns present".to_string());
            recommendations.push("→ Fix critical and high-severity issues immediately".to_string());
        } else if score >= 40.0 {
            recommendations
                .push("⚡ Risk Level: MEDIUM - Security improvements needed".to_string());
            recommendations.push("→ Address high and medium severity findings".to_string());
        } else if score >= 20.0 {
            recommendations
                .push("✓ Risk Level: LOW - Minor security improvements recommended".to_string());
        } else {
            recommendations.push("✅ Risk Level: MINIMAL - Good security posture".to_string());
        }

        recommendations
    }
}

impl Default for RiskScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_calculation() {
        let scorer = RiskScorer::new();
        assert_eq!(scorer.score_to_level(90.0), RiskLevel::Critical);
        assert_eq!(scorer.score_to_level(70.0), RiskLevel::High);
        assert_eq!(scorer.score_to_level(50.0), RiskLevel::Medium);
        assert_eq!(scorer.score_to_level(30.0), RiskLevel::Low);
        assert_eq!(scorer.score_to_level(10.0), RiskLevel::Minimal);
    }
}
