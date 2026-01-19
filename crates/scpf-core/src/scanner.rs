use anyhow::Result;
use regex::RegexBuilder;
use scpf_types::{Match, Pattern, Template};
use std::path::PathBuf;
use tracing::warn;

struct CompiledPattern {
    regex: regex::Regex,
    pattern: Pattern,
}

struct CompiledTemplate {
    template: Template,
    patterns: Vec<CompiledPattern>,
}

pub struct Scanner {
    templates: Vec<CompiledTemplate>,
}

impl Scanner {
    pub fn new(templates: Vec<Template>) -> Result<Self> {
        let mut compiled_templates = Vec::new();
        
        for template in templates {
            let mut compiled_patterns = Vec::new();
            
            for pattern in &template.patterns {
                match RegexBuilder::new(&pattern.pattern)
                    .multi_line(true)
                    .dot_matches_new_line(true)
                    .build()
                {
                    Ok(regex) => {
                        compiled_patterns.push(CompiledPattern {
                            regex,
                            pattern: pattern.clone(),
                        });
                    }
                    Err(e) => {
                        warn!(
                            "Invalid regex in template '{}', pattern '{}': {}",
                            template.id, pattern.id, e
                        );
                        anyhow::bail!(
                            "Invalid regex in template '{}', pattern '{}': {}",
                            template.id, pattern.id, e
                        );
                    }
                }
            }
            
            if !compiled_patterns.is_empty() {
                compiled_templates.push(CompiledTemplate {
                    template,
                    patterns: compiled_patterns,
                });
            }
        }
        
        Ok(Self { templates: compiled_templates })
    }

    pub fn scan(&self, source: &str, file_path: PathBuf) -> Result<Vec<Match>> {
        let mut matches = Vec::new();

        for compiled_template in &self.templates {
            for compiled_pattern in &compiled_template.patterns {
                for mat in compiled_pattern.regex.find_iter(source) {
                    let line_number = source[..mat.start()].lines().count();
                    let line_start = source[..mat.start()].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let line_end = source[mat.end()..].find('\n').map(|i| mat.end() + i).unwrap_or(source.len());
                    let context = source[line_start..line_end].to_string();
                    
                    matches.push(Match {
                        template_id: compiled_template.template.id.clone(),
                        pattern_id: compiled_pattern.pattern.id.clone(),
                        file_path: file_path.clone(),
                        line_number,
                        column: mat.start() - line_start,
                        matched_text: mat.as_str().to_string(),
                        context,
                        severity: compiled_template.template.severity,
                        message: compiled_pattern.pattern.message.clone(),
                    });
                }
            }
        }

        Ok(matches)
    }
}
