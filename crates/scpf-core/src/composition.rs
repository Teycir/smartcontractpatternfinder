use scpf_types::{Pattern, Severity, Template};
use std::collections::HashMap;

/// Template composition allows combining multiple templates
pub struct TemplateComposer {
    templates: HashMap<String, Template>,
    compositions: Vec<ComposedTemplate>,
}

#[derive(Debug, Clone)]
pub struct ComposedTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub base_templates: Vec<String>,
    pub composition_rules: Vec<CompositionRule>,
}

#[derive(Debug, Clone)]
pub enum CompositionRule {
    /// All base templates must match
    AllOf(Vec<String>),
    /// Any base template must match
    AnyOf(Vec<String>),
    /// Exactly N templates must match
    ExactlyN {
        count: usize,
        templates: Vec<String>,
    },
    /// Template A must match AND template B must NOT match
    AndNot { required: String, excluded: String },
    /// Sequential: patterns must appear in order
    Sequential {
        templates: Vec<String>,
        max_distance: usize,
    },
}

impl TemplateComposer {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            compositions: Vec::new(),
        }
    }

    pub fn add_template(&mut self, template: Template) {
        self.templates.insert(template.id.clone(), template);
    }

    pub fn add_composition(&mut self, composition: ComposedTemplate) {
        self.compositions.push(composition);
    }

    /// Create a composed template from YAML
    pub fn compose_from_yaml(&mut self, yaml: &str) -> Result<ComposedTemplate, String> {
        // Parse YAML and create composition
        // This is a simplified version - full implementation would use serde_yaml
        Ok(ComposedTemplate {
            id: "composed-example".to_string(),
            name: "Composed Template".to_string(),
            description: "Example composition".to_string(),
            severity: Severity::High,
            base_templates: vec![],
            composition_rules: vec![],
        })
    }

    /// Evaluate composition rules against matches
    pub fn evaluate_composition(
        &self,
        composition: &ComposedTemplate,
        matches: &HashMap<String, Vec<usize>>, // template_id -> line numbers
    ) -> bool {
        for rule in &composition.composition_rules {
            if !self.evaluate_rule(rule, matches) {
                return false;
            }
        }
        true
    }

    fn evaluate_rule(&self, rule: &CompositionRule, matches: &HashMap<String, Vec<usize>>) -> bool {
        match rule {
            CompositionRule::AllOf(templates) => templates
                .iter()
                .all(|t| matches.contains_key(t) && !matches[t].is_empty()),
            CompositionRule::AnyOf(templates) => templates
                .iter()
                .any(|t| matches.contains_key(t) && !matches[t].is_empty()),
            CompositionRule::ExactlyN { count, templates } => {
                let matched = templates
                    .iter()
                    .filter(|t| matches.contains_key(*t) && !matches[*t].is_empty())
                    .count();
                matched == *count
            }
            CompositionRule::AndNot { required, excluded } => {
                let has_required = matches.contains_key(required) && !matches[required].is_empty();
                let has_excluded = matches.contains_key(excluded) && !matches[excluded].is_empty();
                has_required && !has_excluded
            }
            CompositionRule::Sequential {
                templates,
                max_distance,
            } => self.check_sequential(templates, matches, *max_distance),
        }
    }

    fn check_sequential(
        &self,
        templates: &[String],
        matches: &HashMap<String, Vec<usize>>,
        max_distance: usize,
    ) -> bool {
        if templates.is_empty() {
            return true;
        }

        // Get first template matches
        let first_matches = match matches.get(&templates[0]) {
            Some(m) if !m.is_empty() => m,
            _ => return false,
        };

        // For each starting point, check if sequence exists
        for &start_line in first_matches {
            let mut current_line = start_line;
            let mut found_sequence = true;

            for template_id in &templates[1..] {
                let next_matches = match matches.get(template_id) {
                    Some(m) if !m.is_empty() => m,
                    _ => {
                        found_sequence = false;
                        break;
                    }
                };

                // Find next match within max_distance
                let next_line = next_matches
                    .iter()
                    .find(|&&line| line > current_line && line - current_line <= max_distance);

                match next_line {
                    Some(&line) => current_line = line,
                    None => {
                        found_sequence = false;
                        break;
                    }
                }
            }

            if found_sequence {
                return true;
            }
        }

        false
    }
}

/// Predefined composition templates
pub fn create_reentrancy_composition() -> ComposedTemplate {
    ComposedTemplate {
        id: "reentrancy-comprehensive".to_string(),
        name: "Comprehensive Reentrancy Detection".to_string(),
        description: "Combines multiple reentrancy detection methods".to_string(),
        severity: Severity::Critical,
        base_templates: vec![
            "reentrancy-state-change-v4".to_string(),
            "unchecked-return-value-v4".to_string(),
            "strict-balance-equality-v4".to_string(),
        ],
        composition_rules: vec![CompositionRule::Sequential {
            templates: vec!["external-call".to_string(), "state-mutation".to_string()],
            max_distance: 50,
        }],
    }
}

pub fn create_access_control_composition() -> ComposedTemplate {
    ComposedTemplate {
        id: "access-control-comprehensive".to_string(),
        name: "Comprehensive Access Control".to_string(),
        description: "Detects missing or weak access controls".to_string(),
        severity: Severity::Critical,
        base_templates: vec![
            "unprotected-selfdestruct-v4".to_string(),
            "tx-origin-authentication".to_string(),
        ],
        composition_rules: vec![CompositionRule::AndNot {
            required: "critical-function".to_string(),
            excluded: "access-modifier".to_string(),
        }],
    }
}

pub fn create_defi_composition() -> ComposedTemplate {
    ComposedTemplate {
        id: "defi-comprehensive".to_string(),
        name: "DeFi Vulnerability Suite".to_string(),
        description: "Comprehensive DeFi-specific vulnerability detection".to_string(),
        severity: Severity::High,
        base_templates: vec![
            "front-running-v4".to_string(),
            "strict-balance-equality-v4".to_string(),
            "reentrancy-state-change-v4".to_string(),
        ],
        composition_rules: vec![CompositionRule::AnyOf(vec![
            "price-manipulation".to_string(),
            "flash-loan-attack".to_string(),
            "sandwich-attack".to_string(),
        ])],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_of_rule() {
        let composer = TemplateComposer::new();
        let rule = CompositionRule::AllOf(vec!["t1".to_string(), "t2".to_string()]);

        let mut matches = HashMap::new();
        matches.insert("t1".to_string(), vec![10]);
        matches.insert("t2".to_string(), vec![20]);

        assert!(composer.evaluate_rule(&rule, &matches));

        matches.remove("t2");
        assert!(!composer.evaluate_rule(&rule, &matches));
    }

    #[test]
    fn test_sequential_rule() {
        let composer = TemplateComposer::new();
        let rule = CompositionRule::Sequential {
            templates: vec!["t1".to_string(), "t2".to_string()],
            max_distance: 10,
        };

        let mut matches = HashMap::new();
        matches.insert("t1".to_string(), vec![10]);
        matches.insert("t2".to_string(), vec![15]);

        assert!(composer.evaluate_rule(&rule, &matches));

        matches.insert("t2".to_string(), vec![25]);
        assert!(!composer.evaluate_rule(&rule, &matches));
    }
}
