use crate::analysis::{classify_modifiers, SymbolCollector};
use crate::dataflow::{DataFlowRegistry, DataFlowSeverity};
use crate::regex_validator::RegexValidator;
use crate::semantic::SemanticScanner;
use anyhow::Result;
use regex::RegexBuilder;
use scpf_types::{ContractContext, Match, Pattern, PatternKind, Template};
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
    pub contextual_enabled: bool,
}

impl Scanner {
    pub fn new(templates: Vec<Template>) -> Result<Self> {
        let mut compiled_templates = Vec::with_capacity(templates.len());
        let mut pattern_index = 0u32;
        let mut needs_semantic = false;

        for template in templates {
            if let Some(compiled) =
                compile_template(template, &mut pattern_index, &mut needs_semantic)?
            {
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
            contextual_enabled: true,
        })
    }

    pub fn scan(&mut self, source: &str, file_path: PathBuf) -> Result<Vec<Match>> {
        let newlines: Vec<usize> = source.match_indices('\n').map(|(i, _)| i).collect();

        let mut matches = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut dedup_set = std::collections::HashSet::new();

        // Parse AST once for all semantic patterns and dataflow analysis
        let parsed_tree = if self.semantic_scanner.is_some() {
            self.semantic_scanner
                .as_mut()
                .and_then(|scanner| match scanner.parse(source) {
                    Ok(tree) => Some(tree),
                    Err(e) => {
                        eprintln!("Error: Failed to parse source for semantic analysis: {}", e);
                        None
                    }
                })
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
                    code_snippet: extract_code_snippet(source, &newlines, finding.line),
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
                                eprintln!(
                                    "Error: Semantic pattern '{}' in template '{}' failed: {}",
                                    compiled_pattern.pattern.id, compiled_template.template.id, e
                                );
                                // Continue scanning other patterns instead of failing entire scan
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

                    let context =
                        get_match_context(source, &newlines, mat.start(), mat.end(), line_number);

                    matches.push(Match {
                        template_id: compiled_template.template.id.clone(),
                        pattern_id: compiled_pattern.pattern.id.clone(),
                        file_path: file_path.clone(),
                        line_number,
                        column: mat.start() - line_start,
                        matched_text: mat.as_str().to_string(),
                        context,
                        code_snippet: extract_code_snippet(source, &newlines, line_number),
                        severity: compiled_template.template.severity,
                        message: compiled_pattern.pattern.message.clone(),
                        start_byte: None,
                        end_byte: None,
                    });
                }
            }
        }

        // Deduplicate matches by (file_path, line_number, pattern_id)
        matches.retain(|m| {
            let key = (
                m.file_path.clone(),
                m.line_number,
                m.column,
                m.pattern_id.clone(),
            );
            dedup_set.insert(key)
        });

        // Apply contextual filtering if enabled
        if self.contextual_enabled {
            // Parse AST for contextual analysis if not already parsed
            let tree_for_context = if let Some(ref tree) = parsed_tree {
                Some(tree.clone())
            } else if let Some(ref mut scanner) = self.semantic_scanner {
                scanner.parse(source).ok()
            } else {
                // Initialize semantic scanner just for contextual analysis
                SemanticScanner::new()
                    .ok()
                    .and_then(|mut s| s.parse(source).ok())
            };

            if let Some(ref tree) = tree_for_context {
                let ctx = self.build_context(source, tree);
                matches = self.filter_findings(matches, &ctx);
            }
        }

        Ok(matches)
    }

    /// Build semantic context from AST
    fn build_context(&self, source: &str, tree: &tree_sitter::Tree) -> ContractContext {
        let collector = SymbolCollector::new(source, tree);
        let mut ctx = collector.collect();
        classify_modifiers(&mut ctx);
        
        let function_data: Vec<(String, Vec<String>)> = ctx.functions
            .iter()
            .map(|(name, func)| (name.clone(), func.modifiers.clone()))
            .collect();
        
        let modifiers_map: std::collections::HashMap<_, _> = ctx.modifiers.clone().into_iter().collect();
        
        for (name, modifiers) in function_data {
            if let Some(func) = ctx.functions.get_mut(&name) {
                func.protections = Self::compute_protections_static(&modifiers, &modifiers_map);
            }
        }
        
        ctx
    }

    /// Filter findings based on semantic context
    fn filter_findings(&self, matches: Vec<Match>, ctx: &ContractContext) -> Vec<Match> {
        matches.into_iter().filter(|m| {
            self.should_report_finding(m, ctx)
        }).collect()
    }

    /// Determine if finding should be reported based on protections
    fn should_report_finding(&self, finding: &Match, ctx: &ContractContext) -> bool {
        let func = self.find_function_at_line(ctx, finding.line_number);
        
        if let Some(func) = func {
            // Filter reentrancy findings if function has reentrancy guard OR access control
            // (access-controlled functions are safe from untrusted reentrancy)
            if self.is_reentrancy_pattern(&finding.template_id) {
                if func.protections.has_reentrancy_guard || func.protections.has_access_control {
                    return false;
                }
            }
            
            // Filter access control findings if function has access control
            if self.is_access_control_pattern(&finding.template_id) && func.protections.has_access_control {
                return false;
            }
            
            // Filter if function is pausable
            if func.protections.has_pausable {
                return false;
            }
        }
        
        true
    }

    /// Find function containing the given line number
    fn find_function_at_line<'a>(&self, ctx: &'a ContractContext, line: usize) -> Option<&'a scpf_types::FunctionContext> {
        ctx.functions.values().find(|f| {
            f.start_line <= line && line <= f.end_line
        })
    }

    /// Check if template is reentrancy-related
    fn is_reentrancy_pattern(&self, template_id: &str) -> bool {
        template_id.contains("reentrancy") || 
        template_id.contains("external-call") ||
        template_id.contains("low-level-call")
    }

    /// Check if template is access-control-related
    fn is_access_control_pattern(&self, template_id: &str) -> bool {
        template_id.contains("access") || 
        template_id.contains("authorization") ||
        template_id.contains("permission")
    }

    fn compute_protections_static(
        modifiers: &[String],
        modifiers_map: &std::collections::HashMap<String, scpf_types::ModifierContext>,
    ) -> scpf_types::ProtectionSet {
        let mut protections = scpf_types::ProtectionSet::default();

        for mod_name in modifiers {
            if let Some(mod_ctx) = modifiers_map.get(mod_name) {
                match mod_ctx.modifier_type {
                    scpf_types::ModifierType::ReentrancyGuard => {
                        protections.has_reentrancy_guard = true;
                    }
                    scpf_types::ModifierType::AccessControl => {
                        protections.has_access_control = true;
                    }
                    scpf_types::ModifierType::Pausable => {
                        protections.has_pausable = true;
                    }
                    _ => {}
                }
            }
        }

        protections
    }
}

fn compile_pattern(pattern: &Pattern, template_id: &str, index: u32) -> Result<CompiledPattern> {
    if pattern.kind == PatternKind::Semantic {
        return Ok(CompiledPattern {
            regex: RegexBuilder::new(".*").build().unwrap(),
            pattern: pattern.clone(),
            index,
        });
    }

    RegexValidator::validate_pattern(&pattern.pattern).map_err(|e| {
        warn!(
            "Unsafe regex pattern in template '{}', pattern '{}': {}",
            template_id, pattern.id, e
        );
        anyhow::anyhow!(
            "Unsafe regex pattern in template '{}', pattern '{}': {}",
            template_id,
            pattern.id,
            e
        )
    })?;

    let regex = RegexBuilder::new(&pattern.pattern)
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .map_err(|e| {
            warn!(
                "Invalid regex in template '{}', pattern '{}': {}",
                template_id, pattern.id, e
            );
            anyhow::anyhow!(
                "Invalid regex in template '{}', pattern '{}': {}",
                template_id,
                pattern.id,
                e
            )
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

fn extract_code_snippet(
    source: &str,
    _newlines: &[usize],
    line_number: usize,
) -> Option<scpf_types::CodeSnippet> {
    let lines: Vec<&str> = source.lines().collect();
    if line_number == 0 || line_number > lines.len() {
        return None;
    }

    let idx = line_number - 1;
    let before = if idx > 0 {
        lines[idx - 1].to_string()
    } else {
        String::new()
    };

    let vulnerable_line = lines[idx].to_string();

    let after = if idx + 1 < lines.len() {
        lines[idx + 1].to_string()
    } else {
        String::new()
    };

    Some(scpf_types::CodeSnippet {
        before,
        vulnerable_line,
        after,
        line_start: line_number.saturating_sub(1),
    })
}
