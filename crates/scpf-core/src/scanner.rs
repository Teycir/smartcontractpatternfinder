use crate::dataflow::{DataFlowRegistry, DataFlowSeverity};
use crate::regex_validator::RegexValidator;
use crate::semantic::SemanticScanner;
use anyhow::Result;
use regex::RegexBuilder;
use scpf_types::{Match, Pattern, PatternKind, Template};
use std::path::PathBuf;
use tracing::warn;

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
    dataflow_registry: DataFlowRegistry,
}

impl Scanner {
    pub fn new(templates: Vec<Template>) -> Result<Self> {
        let mut compiled_templates = Vec::with_capacity(templates.len());
        let mut pattern_index = 0u32;
        let mut needs_semantic = false;

        for template in templates {
            if let Some(compiled) = compile_template(template, &mut pattern_index, &mut needs_semantic)? {
                compiled_templates.push(compiled);
            }
        }

        let semantic_scanner = if needs_semantic {
            match SemanticScanner::new() {
                Ok(scanner) => {
                    warn!("Semantic scanning enabled but may have compatibility issues with current tree-sitter-solidity grammar.");
                    Some(scanner)
                }
                Err(e) => {
                    warn!("Failed to initialize semantic scanner: {}. Only regex patterns will be used.", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            templates: compiled_templates,
            semantic_scanner,
            dataflow_registry: DataFlowRegistry::with_default_analyzers(),
        })
    }

    pub fn scan(&mut self, source: &str, file_path: PathBuf) -> Result<Vec<Match>> {
        let newlines: Vec<usize> = source.match_indices('\n').map(|(i, _)| i).collect();

        let mut matches = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Parse AST once for all semantic patterns and dataflow analysis
        let parsed_tree = if self.semantic_scanner.is_some() {
            self.semantic_scanner
                .as_mut()
                .and_then(|scanner| scanner.parse(source).ok())
        } else {
            None
        };

        // Run all registered dataflow analyzers
        if let Some(ref tree) = parsed_tree {
            let findings = self.dataflow_registry.analyze_all(tree, source);

            for finding in findings {
                let line_start = if finding.line > 1 {
                    newlines.get(finding.line - 2).copied().unwrap_or(0) + 1
                } else {
                    0
                };
                let line_end = newlines
                    .get(finding.line - 1)
                    .copied()
                    .unwrap_or(source.len());
                let context = source[line_start..line_end].to_string();

                let severity = match finding.severity {
                    DataFlowSeverity::Critical => scpf_types::Severity::Critical,
                    DataFlowSeverity::High => scpf_types::Severity::High,
                    DataFlowSeverity::Medium => scpf_types::Severity::Medium,
                    DataFlowSeverity::Low => scpf_types::Severity::Low,
                };

                matches.push(Match {
                    template_id: finding.analyzer_id,
                    pattern_id: finding.pattern_id,
                    file_path: file_path.clone(),
                    line_number: finding.line,
                    column: 0,
                    matched_text: finding.context,
                    context,
                    severity,
                    message: finding.message,
                    start_byte: None,
                    end_byte: None,
                });
            }
        }

        for compiled_template in &self.templates {
            for compiled_pattern in &compiled_template.patterns {
                // Dispatch based on pattern kind
                if compiled_pattern.pattern.kind == PatternKind::Semantic {
                    if let (Some(ref mut semantic_scanner), Some(ref tree)) =
                        (&mut self.semantic_scanner, &parsed_tree)
                    {
                        match semantic_scanner.scan_with_tree(
                            source,
                            tree,
                            &compiled_pattern.pattern,
                            &compiled_template.template.id,
                            compiled_template.template.severity,
                            file_path.clone(),
                        ) {
                            Ok(semantic_matches) => matches.extend(semantic_matches),
                            Err(e) => {
                                warn!(
                                    "Skipping semantic pattern '{}' in template '{}': {}",
                                    compiled_pattern.pattern.id, compiled_template.template.id, e
                                );
                            }
                        }
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

                    let context = get_match_context(source, &newlines, mat.start(), mat.end(), line_number);

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

fn compile_pattern(
    pattern: &Pattern,
    template_id: &str,
    index: u32,
) -> Result<CompiledPattern> {
    if pattern.kind == PatternKind::Semantic {
        return Ok(CompiledPattern {
            regex: RegexBuilder::new(".*").build().unwrap(),
            pattern: pattern.clone(),
            index,
        });
    }

    RegexValidator::validate_pattern(&pattern.pattern).map_err(|e| {
        warn!("Unsafe regex pattern in template '{}', pattern '{}': {}", template_id, pattern.id, e);
        anyhow::anyhow!("Unsafe regex pattern in template '{}', pattern '{}': {}", template_id, pattern.id, e)
    })?;

    let regex = RegexBuilder::new(&pattern.pattern)
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .map_err(|e| {
            warn!("Invalid regex in template '{}', pattern '{}': {}", template_id, pattern.id, e);
            anyhow::anyhow!("Invalid regex in template '{}', pattern '{}': {}", template_id, pattern.id, e)
        })?;

    Ok(CompiledPattern {
        regex,
        pattern: pattern.clone(),
        index,
    })
}

fn compile_template(
    template: Template,
    pattern_index: &mut u32,
    needs_semantic: &mut bool,
) -> Result<Option<CompiledTemplate>> {
    let mut compiled_patterns = Vec::with_capacity(template.patterns.len());

    for pattern in &template.patterns {
        if pattern.kind == PatternKind::Semantic {
            *needs_semantic = true;
        }
        let compiled = compile_pattern(pattern, &template.id, *pattern_index)?;
        compiled_patterns.push(compiled);
        *pattern_index += 1;
    }

    if compiled_patterns.is_empty() {
        return Ok(None);
    }

    Ok(Some(CompiledTemplate {
        template,
        patterns: compiled_patterns,
    }))
}

fn get_match_context(
    source: &str,
    newlines: &[usize],
    match_start: usize,
    match_end: usize,
    line_number: usize,
) -> String {
    const MAX_CONTEXT_CHARS: usize = 200;
    const CONTEXT_PADDING: usize = 50;

    let match_len = match_end - match_start;

    if match_len > MAX_CONTEXT_CHARS {
        let start = match_start.saturating_sub(CONTEXT_PADDING);
        let end = (match_end + CONTEXT_PADDING).min(source.len());
        source[start..end].to_string()
    } else {
        let context_start = if line_number > 1 {
            newlines[line_number - 2] + 1
        } else {
            0
        };
        let context_end = newlines
            .get(line_number - 1)
            .copied()
            .unwrap_or(source.len());
        source[context_start..context_end].to_string()
    }
}
