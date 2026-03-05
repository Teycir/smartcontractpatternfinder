use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundTruth {
    pub contracts: Vec<ContractLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractLabel {
    pub file_path: String,
    pub description: String,
    pub vulnerabilities: Vec<VulnerabilityLabel>,
    pub safe_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityLabel {
    pub pattern_id: String,
    pub line_number: usize,
    pub severity: String,
    pub description: String,
    pub exploitable: bool,
}

#[derive(Debug, Clone)]
pub struct AccuracyMetrics {
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub per_category: HashMap<String, CategoryMetrics>,
}

#[derive(Debug, Clone)]
pub struct CategoryMetrics {
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub tp: usize,
    pub fp: usize,
    pub fn_: usize,
}

pub struct AccuracyEvaluator {
    ground_truth: GroundTruth,
}

impl AccuracyEvaluator {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let ground_truth: GroundTruth = serde_json::from_str(&content)?;
        Ok(Self { ground_truth })
    }

    pub fn evaluate(&self, findings: &[scpf_types::Match]) -> AccuracyMetrics {
        let mut tp = 0;
        let mut fp = 0;
        let mut fn_ = 0;
        let mut per_category: HashMap<String, (usize, usize, usize)> = HashMap::new();

        let mut matched_labels = std::collections::HashSet::new();

        for finding in findings {
            let file_path = finding.file_path.to_string_lossy().to_string();

            if let Some(contract) = self
                .ground_truth
                .contracts
                .iter()
                .find(|c| file_path.contains(&c.file_path))
            {
                if let Some((idx, _)) =
                    contract.vulnerabilities.iter().enumerate().find(|(_, v)| {
                        v.pattern_id == finding.pattern_id
                            && (v.line_number as isize - finding.line_number as isize).abs() <= 2
                    })
                {
                    tp += 1;
                    matched_labels.insert((file_path.clone(), idx));

                    let entry = per_category
                        .entry(finding.pattern_id.clone())
                        .or_insert((0, 0, 0));
                    entry.0 += 1;
                } else {
                    fp += 1;
                    let entry = per_category
                        .entry(finding.pattern_id.clone())
                        .or_insert((0, 0, 0));
                    entry.1 += 1;
                }
            } else {
                fp += 1;
            }
        }

        for contract in &self.ground_truth.contracts {
            for (idx, vuln) in contract.vulnerabilities.iter().enumerate() {
                if !matched_labels.contains(&(contract.file_path.clone(), idx)) {
                    fn_ += 1;
                    let entry = per_category
                        .entry(vuln.pattern_id.clone())
                        .or_insert((0, 0, 0));
                    entry.2 += 1;
                }
            }
        }

        let precision = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            0.0
        };

        let recall = if tp + fn_ > 0 {
            tp as f64 / (tp + fn_) as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        let per_category_metrics = per_category
            .into_iter()
            .map(|(cat, (tp, fp, fn_))| {
                let p = if tp + fp > 0 {
                    tp as f64 / (tp + fp) as f64
                } else {
                    0.0
                };
                let r = if tp + fn_ > 0 {
                    tp as f64 / (tp + fn_) as f64
                } else {
                    0.0
                };
                let f1 = if p + r > 0.0 {
                    2.0 * (p * r) / (p + r)
                } else {
                    0.0
                };
                (
                    cat,
                    CategoryMetrics {
                        precision: p,
                        recall: r,
                        f1_score: f1,
                        tp,
                        fp,
                        fn_,
                    },
                )
            })
            .collect();

        AccuracyMetrics {
            precision,
            recall,
            f1_score,
            true_positives: tp,
            false_positives: fp,
            false_negatives: fn_,
            per_category: per_category_metrics,
        }
    }

    pub fn print_report(&self, metrics: &AccuracyMetrics) {
        println!("\n=== Accuracy Report ===\n");
        println!("Overall Metrics:");
        println!("  Precision: {:.2}%", metrics.precision * 100.0);
        println!("  Recall:    {:.2}%", metrics.recall * 100.0);
        println!("  F1 Score:  {:.2}%", metrics.f1_score * 100.0);
        println!("\nConfusion Matrix:");
        println!("  True Positives:  {}", metrics.true_positives);
        println!("  False Positives: {}", metrics.false_positives);
        println!("  False Negatives: {}", metrics.false_negatives);

        println!("\nPer-Category Metrics:");
        let mut categories: Vec<_> = metrics.per_category.iter().collect();
        categories.sort_by_key(|(name, _)| *name);

        for (category, cat_metrics) in categories {
            println!("\n  {}:", category);
            println!("    Precision: {:.2}%", cat_metrics.precision * 100.0);
            println!("    Recall:    {:.2}%", cat_metrics.recall * 100.0);
            println!("    F1 Score:  {:.2}%", cat_metrics.f1_score * 100.0);
            println!(
                "    TP: {}, FP: {}, FN: {}",
                cat_metrics.tp, cat_metrics.fp, cat_metrics.fn_
            );
        }

        println!("\n=== Quality Grade ===");
        let grade = if metrics.f1_score >= 0.90 {
            "A (Excellent)"
        } else if metrics.f1_score >= 0.80 {
            "B (Good)"
        } else if metrics.f1_score >= 0.70 {
            "C (Acceptable)"
        } else if metrics.f1_score >= 0.60 {
            "D (Needs Improvement)"
        } else {
            "F (Unacceptable)"
        };
        println!("{}\n", grade);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accuracy_calculation() {
        let ground_truth = GroundTruth {
            contracts: vec![ContractLabel {
                file_path: "test.sol".to_string(),
                description: "Test".to_string(),
                vulnerabilities: vec![VulnerabilityLabel {
                    pattern_id: "reentrancy".to_string(),
                    line_number: 10,
                    severity: "high".to_string(),
                    description: "Test".to_string(),
                    exploitable: true,
                }],
                safe_patterns: vec![],
            }],
        };

        let evaluator = AccuracyEvaluator { ground_truth };

        let findings = vec![scpf_types::Match {
            template_id: "test".to_string(),
            pattern_id: "reentrancy".to_string(),
            file_path: PathBuf::from("test.sol"),
            line_number: 10,
            column: 0,
            matched_text: "test".to_string(),
            context: "test".to_string(),
            severity: scpf_types::Severity::High,
            message: "test".to_string(),
            start_byte: None,
            end_byte: None,
            code_snippet: None,
            function_context: None,
            protections: None,
            filtered: false,
        }];

        let metrics = evaluator.evaluate(&findings);
        assert_eq!(metrics.true_positives, 1);
        assert_eq!(metrics.false_positives, 0);
        assert_eq!(metrics.false_negatives, 0);
        assert_eq!(metrics.precision, 1.0);
        assert_eq!(metrics.recall, 1.0);
    }
}
