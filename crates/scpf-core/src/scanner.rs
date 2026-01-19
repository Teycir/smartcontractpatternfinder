use anyhow::Result;
use regex::Regex;
use scpf_types::{Match, Template};
use std::path::PathBuf;

pub struct Scanner {
    templates: Vec<Template>,
}

impl Scanner {
    pub fn new(templates: Vec<Template>) -> Self {
        Self { templates }
    }

    pub fn scan(&self, source: &str, file_path: PathBuf) -> Result<Vec<Match>> {
        let mut matches = Vec::new();

        for template in &self.templates {
            for pattern in &template.patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    for (line_num, line) in source.lines().enumerate() {
                        if let Some(mat) = regex.find(line) {
                            matches.push(Match {
                                template_id: template.id.clone(),
                                pattern_id: pattern.id.clone(),
                                file_path: file_path.clone(),
                                line_number: line_num + 1,
                                column: mat.start(),
                                matched_text: mat.as_str().to_string(),
                                context: line.to_string(),
                                severity: template.severity,
                                message: pattern.message.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(matches)
    }
}
