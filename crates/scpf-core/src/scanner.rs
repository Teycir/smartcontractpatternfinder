use anyhow::Result;
use regex::Regex;
use scpf_types::{Match, Template};
use std::path::PathBuf;

pub struct Scanner {
    templates: Vec<(Template, Vec<Regex>)>,
}

impl Scanner {
    pub fn new(templates: Vec<Template>) -> Self {
        let compiled = templates
            .into_iter()
            .map(|t| {
                let regexes = t.patterns.iter()
                    .filter_map(|p| Regex::new(&p.pattern).ok())
                    .collect();
                (t, regexes)
            })
            .collect();
        Self { templates: compiled }
    }

    pub fn scan(&self, source: &str, file_path: PathBuf) -> Result<Vec<Match>> {
        let mut matches = Vec::new();

        for (template, regexes) in &self.templates {
            let mut pattern_idx = 0;
            for (regex_idx, regex) in regexes.iter().enumerate() {
                while pattern_idx < template.patterns.len() {
                    if Regex::new(&template.patterns[pattern_idx].pattern).is_ok() {
                        if pattern_idx == regex_idx {
                            break;
                        }
                        pattern_idx += 1;
                    } else {
                        pattern_idx += 1;
                    }
                }
                
                if pattern_idx >= template.patterns.len() {
                    break;
                }
                
                let pattern = &template.patterns[pattern_idx];
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
                pattern_idx += 1;
            }
        }

        Ok(matches)
    }
}
