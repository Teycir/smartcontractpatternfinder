use crate::analysis::{classify_modifiers, is_vulnerable_reentrancy, SymbolCollector};
use crate::dataflow::{DataFlowRegistry, DataFlowSeverity};
use crate::regex_validator::RegexValidator;
use anyhow::Result;
use regex::RegexBuilder;
use scpf_types::{ContractContext, Match, Pattern, Template};
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
    dataflow_registry: DataFlowRegistry,
    pub contextual_enabled: bool,
}

impl Scanner {
    pub fn new(templates: Vec<Template>) -> Result<Self> {
        let mut compiled_templates = Vec::with_capacity(templates.len());
        let mut pattern_index = 0u32;

        for template in templates {
            if let Some(compiled) = compile_template(template, &mut pattern_index)? {
                compiled_templates.push(compiled);
            }
        }

        Ok(Self {
            templates: compiled_templates,
            dataflow_registry: DataFlowRegistry::with_default_analyzers(),
            contextual_enabled: true,
        })
    }

    pub fn scan(&mut self, source: &str, file_path: PathBuf) -> Result<Vec<Match>> {
        let newlines: Vec<usize> = source.match_indices('\n').map(|(i, _)| i).collect();

        let solidity_version = extract_solidity_version(source);
        let is_modern_solidity = is_version_gte_0_8(&solidity_version);

        let mut matches = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut dedup_set = std::collections::HashSet::new();

        // Parse AST once for dataflow analysis
        let parsed_tree = crate::semantic::SemanticScanner::new()
            .ok()
            .and_then(|mut s| s.parse(source).ok());

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
                    function_context: None,
                    protections: None,
                });
            }
        }

        for compiled_template in &self.templates {
            if is_modern_solidity && compiled_template.template.id.contains("integer_overflow") {
                continue;
            }

            let _is_zeroday_template = compiled_template.template.id.contains("zero-day");

            for compiled_pattern in &compiled_template.patterns {
                // All patterns are now regex-based
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

                    // OpenZeppelin whitelist disabled - was filtering too aggressively
                    // if is_zeroday_template && is_openzeppelin_safe_pattern(source, &context, mat.as_str()) {
                    //     continue;
                    // }

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
                        function_context: None,
                        protections: None,
                    });
                }
            }
        }

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
            let tree_for_context = parsed_tree.or_else(|| {
                crate::semantic::SemanticScanner::new()
                    .ok()
                    .and_then(|mut s| s.parse(source).ok())
            });

            if let Some(ref tree) = tree_for_context {
                let ctx = self.build_context(source, tree);

                matches.retain(|m| {
                    if self.is_reentrancy_pattern(&m.template_id) {
                        is_vulnerable_reentrancy(tree, source, m.line_number)
                    } else {
                        true
                    }
                });

                matches = self.filter_findings(matches, &ctx);
                matches = self.enrich_with_context(matches, &ctx);
            }
        }

        Ok(matches)
    }

    /// Build semantic context from AST
    fn build_context(&self, source: &str, tree: &tree_sitter::Tree) -> ContractContext {
        let collector = SymbolCollector::new(source, tree);
        let mut ctx = collector.collect();
        classify_modifiers(&mut ctx);

        let function_data: Vec<(String, Vec<String>)> = ctx
            .functions
            .iter()
            .map(|(name, func)| (name.clone(), func.modifiers.clone()))
            .collect();

        let modifiers_map: std::collections::HashMap<_, _> =
            ctx.modifiers.clone().into_iter().collect();

        for (name, modifiers) in function_data {
            if let Some(func) = ctx.functions.get_mut(&name) {
                func.protections = Self::compute_protections_static(&modifiers, &modifiers_map);
            }
        }

        ctx
    }

    /// Filter findings based on semantic context
    fn filter_findings(&self, matches: Vec<Match>, ctx: &ContractContext) -> Vec<Match> {
        matches
            .into_iter()
            .filter(|m| self.should_report_finding(m, ctx))
            .collect()
    }

    /// Enrich matches with function context for Opus analysis
    fn enrich_with_context(&self, mut matches: Vec<Match>, ctx: &ContractContext) -> Vec<Match> {
        for m in &mut matches {
            if let Some(func) = self.find_function_at_line(ctx, m.line_number) {
                m.function_context = Some(func.clone());
                m.protections = Some(func.protections.clone());
            }
        }
        matches
    }

    /// Determine if finding should be reported based on protections
    fn should_report_finding(&self, finding: &Match, ctx: &ContractContext) -> bool {
        let func = self.find_function_at_line(ctx, finding.line_number);

        if let Some(func) = func {
            // Filter reentrancy findings if function has reentrancy guard OR access control
            if self.is_reentrancy_pattern(&finding.template_id)
                && (func.protections.has_reentrancy_guard || func.protections.has_access_control)
            {
                return false;
            }

            // Filter access control findings if function has access control
            if self.is_access_control_pattern(&finding.template_id)
                && func.protections.has_access_control
            {
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
    fn find_function_at_line<'a>(
        &self,
        ctx: &'a ContractContext,
        line: usize,
    ) -> Option<&'a scpf_types::FunctionContext> {
        ctx.functions
            .values()
            .find(|f| f.start_line <= line && line <= f.end_line)
    }

    /// Check if template is reentrancy-related
    fn is_reentrancy_pattern(&self, template_id: &str) -> bool {
        template_id.contains("reentrancy")
            || template_id.contains("external-call")
            || template_id.contains("low-level-call")
    }

    /// Check if template is access-control-related
    fn is_access_control_pattern(&self, template_id: &str) -> bool {
        template_id.contains("access")
            || template_id.contains("authorization")
            || template_id.contains("permission")
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
) -> Result<Option<CompiledTemplate>> {
    let mut compiled_patterns = Vec::with_capacity(template.patterns.len());

    for pattern in &template.patterns {
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

/// Ensures the given index is at a valid UTF-8 character boundary by walking backward if needed.
/// If the index is already at a boundary, returns it unchanged.
/// If the index is beyond the string length, returns the string length.
fn adjust_to_char_boundary_start(source: &str, index: usize) -> usize {
    if index >= source.len() {
        return source.len();
    }

    // If already at a valid boundary, return as-is
    if source.is_char_boundary(index) {
        return index;
    }

    // Walk backward to find a valid boundary
    for i in (0..index).rev() {
        if source.is_char_boundary(i) {
            return i;
        }
    }

    // If nothing found walking backward, return 0
    0
}

/// Ensures the given index is at a valid UTF-8 character boundary by walking forward if needed.
/// If the index is already at a boundary, returns it unchanged.
/// If the index is beyond the string length, returns the string length.
fn adjust_to_char_boundary_end(source: &str, index: usize) -> usize {
    if index >= source.len() {
        return source.len();
    }

    // If already at a valid boundary, return as-is
    if source.is_char_boundary(index) {
        return index;
    }

    // Walk forward to find a valid boundary
    for i in (index + 1)..=source.len() {
        if source.is_char_boundary(i) {
            return i;
        }
    }

    // If nothing found walking forward, return string length
    source.len()
}

fn get_match_context(
    source: &str,
    newlines: &[usize],
    match_start: usize,
    match_end: usize,
    line_number: usize,
) -> String {
    const MAX_CONTEXT_CHARS: usize = 1000;
    const CONTEXT_LINES: usize = 10;

    let match_len = match_end - match_start;

    if match_len > MAX_CONTEXT_CHARS {
        let raw_start = match_start.saturating_sub(100);
        let raw_end = (match_end + 100).min(source.len());
        let start = adjust_to_char_boundary_start(source, raw_start);
        let end = adjust_to_char_boundary_end(source, raw_end);
        source[start..end].to_string()
    } else {
        // Include CONTEXT_LINES before and after for better detection
        let start_line = line_number.saturating_sub(CONTEXT_LINES);
        let end_line = (line_number + CONTEXT_LINES).min(newlines.len());
        
        let context_start = if start_line > 1 {
            newlines.get(start_line - 2).copied().unwrap_or(0) + 1
        } else {
            0
        };
        let context_end = newlines
            .get(end_line.saturating_sub(1))
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

/// Extract Solidity version from pragma statement
fn extract_solidity_version(source: &str) -> Option<String> {
    let pragma_regex = regex::Regex::new(r"pragma\s+solidity\s+([^;]+);").ok()?;
    pragma_regex
        .captures(source)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// Check if version is >= 0.8.0 (has built-in overflow protection)
fn is_version_gte_0_8(version: &Option<String>) -> bool {
    let version = match version {
        Some(v) => v,
        None => return false,
    };

    // Extract major.minor from version string (e.g., "^0.8.0" -> "0.8")
    let version_regex = regex::Regex::new(r"(\d+)\.(\d+)").ok();
    if let Some(regex) = version_regex {
        if let Some(cap) = regex.captures(version) {
            if let (Some(major), Some(minor)) = (cap.get(1), cap.get(2)) {
                if let (Ok(maj), Ok(min)) =
                    (major.as_str().parse::<u32>(), minor.as_str().parse::<u32>())
                {
                    return maj > 0 || (maj == 0 && min >= 8);
                }
            }
        }
    }
    false
}

/// Detect if source code is OpenZeppelin library (not just imports it)
#[allow(dead_code)]
fn is_openzeppelin_library(source: &str) -> bool {
    // Only consider it library code if it's in a library file path or has library/abstract contract
    let has_library_indicators = source.contains("library ") || source.contains("abstract contract");
    let has_oz_attribution = source.contains("@openzeppelin") || source.contains("OpenZeppelin");
    let has_solady_attribution = source.contains("@solady") || source.contains("Solady");
    
    // Must have both library indicators AND attribution to be considered library code
    has_library_indicators && (has_oz_attribution || has_solady_attribution)
}

/// Check if matched pattern is a safe OpenZeppelin pattern (only for 0-day scans)
#[allow(dead_code)]
fn is_openzeppelin_safe_pattern(source: &str, context: &str, matched_text: &str) -> bool {
    // First check if this is OpenZeppelin library code
    if !is_openzeppelin_library(source) {
        return false;
    }

    // Skip comments and documentation
    let trimmed = context.trim_start();
    if trimmed.starts_with("//") || trimmed.starts_with("*") {
        return true;
    }

    // Safe unchecked blocks (OpenZeppelin uses these with overflow protection)
    if matched_text == "unchecked" && context.contains("unchecked {") {
        return true;
    }

    // Safe delegatecall in proxy patterns
    if matched_text == "delegatecall" {
        // Assembly delegatecall in proxy _delegate function
        if context.contains("let result := delegatecall") {
            return true;
        }
        // High-level delegatecall wrapper functions
        if context.contains("target.delegatecall(data)") 
            || context.contains("functionDelegateCall") {
            return true;
        }
        // Proxy-related contexts
        if context.contains("_implementation") 
            || context.contains("Proxy") 
            || context.contains("@dev This abstract contract") {
            return true;
        }
    }

    false
}
