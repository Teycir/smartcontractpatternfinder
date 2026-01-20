use anyhow::Result;
use regex::RegexBuilder;
use scpf_types::{Match, Pattern, PatternKind, Template};
use std::path::PathBuf;
use tracing::warn;
use crate::regex_validator::RegexValidator;
use crate::semantic::SemanticScanner;

struct CompiledPattern {
    regex: regex::Regex,
    pattern: Pattern,
    index: u32,
}

struct CompiledTemplate {
    template: Template,
    patterns: Vec<CompiledPattern>,
}

pub struct Scanner {
    templates: Vec<CompiledTemplate>,
    semantic_scanner: Option<SemanticScanner>,
}

impl Scanner {
    pub fn new(templates: Vec<Template>) -> Result<Self> {
        let mut compiled_templates = Vec::with_capacity(templates.len());
        let mut pattern_index = 0u32;
        let mut needs_semantic = false;
        
        for template in templates {
            let pattern_count = template.patterns.len();
            let mut compiled_patterns = Vec::with_capacity(pattern_count);
            
            for pattern in &template.patterns {
                if pattern.kind == PatternKind::Semantic {
                    needs_semantic = true;
                    compiled_patterns.push(CompiledPattern {
                        regex: RegexBuilder::new(".*").build().unwrap(),
                        pattern: pattern.clone(),
                        index: pattern_index,
                    });
                    pattern_index += 1;
                    continue;
                }

                // Validate pattern for DoS vulnerabilities
                if let Err(e) = RegexValidator::validate_pattern(&pattern.pattern) {
                    warn!(
                        "Unsafe regex pattern in template '{}', pattern '{}': {}",
                        template.id, pattern.id, e
                    );
                    anyhow::bail!(
                        "Unsafe regex pattern in template '{}', pattern '{}': {}",
                        template.id, pattern.id, e
                    );
                }

                match RegexBuilder::new(&pattern.pattern)
                    .multi_line(true)
                    .dot_matches_new_line(true)
                    .build()
                {
                    Ok(regex) => {
                        compiled_patterns.push(CompiledPattern {
                            regex,
                            pattern: pattern.clone(),
                            index: pattern_index,
                        });
                        pattern_index += 1;
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
        
        let semantic_scanner = if needs_semantic {
            Some(SemanticScanner::new()?)
        } else {
            None
        };
        
        Ok(Self { 
            templates: compiled_templates,
            semantic_scanner,
        })
    }

    pub fn scan(&mut self, source: &str, file_path: PathBuf) -> Result<Vec<Match>> {
        let newlines: Vec<usize> = source.match_indices('\n').map(|(i, _)| i).collect();
        
        let mut matches = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for compiled_template in &self.templates {
            for compiled_pattern in &compiled_template.patterns {
                // Dispatch based on pattern kind
                if compiled_pattern.pattern.kind == PatternKind::Semantic {
                    if let Some(ref mut semantic_scanner) = self.semantic_scanner {
                        let semantic_matches = semantic_scanner.scan(
                            source,
                            &compiled_pattern.pattern,
                            &compiled_template.template.id,
                            compiled_template.template.severity,
                            file_path.clone(),
                        )?;
                        matches.extend(semantic_matches);
                    }
                    continue;
                }

                // Regex pattern matching
                for mat in compiled_pattern.regex.find_iter(source) {
                    let key = (mat.start(), mat.end(), compiled_pattern.index);
                    if !seen.insert(key) {
                        continue;
                    }

                    let line_number = newlines.partition_point(|&pos| pos < mat.start()) + 1;
                    let line_start = if line_number > 1 {
                        newlines[line_number - 2] + 1
                    } else {
                        0
                    };
                    
                    const MAX_CONTEXT_CHARS: usize = 200;
                    const CONTEXT_PADDING: usize = 50;
                    
                    let match_len = mat.end() - mat.start();
                    let context = if match_len > MAX_CONTEXT_CHARS {
                        let start = mat.start().saturating_sub(CONTEXT_PADDING);
                        let end = (mat.end() + CONTEXT_PADDING).min(source.len());
                        source[start..end].to_string()
                    } else {
                        let context_start = if line_number > 1 {
                            newlines[line_number - 2] + 1
                        } else {
                            0
                        };
                        let context_end = newlines.get(line_number - 1)
                            .copied()
                            .unwrap_or(source.len());
                        source[context_start..context_end].to_string()
                    };
                    
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
                        start_byte: None,
                        end_byte: None,
                    });
                }
            }
        }

        Ok(matches)
    }
}
