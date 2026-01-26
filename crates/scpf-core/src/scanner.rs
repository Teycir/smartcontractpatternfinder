use crate::regex_validator::RegexValidator;
use anyhow::Result;
use fancy_regex::RegexBuilder;
use scpf_types::{Match, Pattern, Template};
use std::path::PathBuf;
use tracing::warn;

struct CompiledPattern {
    regex: fancy_regex::Regex,
    pattern: Pattern,
    index: u32,
}

struct CompiledTemplate {
    template: Template,
    patterns: Vec<CompiledPattern>,
}

pub struct Scanner {
    templates: Vec<CompiledTemplate>,
    reentrancy_guard_regex: Option<fancy_regex::Regex>,
    access_control_regex: Option<fancy_regex::Regex>,
    pausable_regex: Option<fancy_regex::Regex>,
    oz_address_lib_regex: Option<fancy_regex::Regex>,
    proxy_pattern_regex: Option<fancy_regex::Regex>,
    safe_nft_pattern_regex: Option<fancy_regex::Regex>,
    timestamp_pattern_regex: Option<fancy_regex::Regex>,
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
            reentrancy_guard_regex: fancy_regex::Regex::new(r"\b(nonReentrant|ReentrancyGuard|noReentrancy|lock|mutex)\b").ok(),
            access_control_regex: fancy_regex::Regex::new(r"\b(onlyOwner|onlyRole|onlyAdmin|requireOwner|requireRole|AccessControl|Ownable|auth|authorized)\b").ok(),
            pausable_regex: fancy_regex::Regex::new(r"\b(whenNotPaused|whenPaused|Pausable|notPaused)\b").ok(),
            oz_address_lib_regex: fancy_regex::Regex::new(r"(sendValue|functionCall|functionCallWithValue|functionStaticCall|functionDelegateCall)\s*\(").ok(),
            proxy_pattern_regex: fancy_regex::Regex::new(r"\b(Proxy|ERC1967|TransparentUpgradeable|BeaconProxy|_implementation|_delegate|_fallback)\b").ok(),
            safe_nft_pattern_regex: fancy_regex::Regex::new(r"\b(ERC721|ERC1155|_mint|_burn|_transfer|_safeMint|tokenId|balanceOf)\b").ok(),
            timestamp_pattern_regex: fancy_regex::Regex::new(r"\b(block\.timestamp|block\.number|now)\b").ok(),
        })
    }

    pub fn scan(&mut self, source: &str, file_path: PathBuf) -> Result<Vec<Match>> {
        let newlines: Vec<usize> = source.match_indices('\n').map(|(i, _)| i).collect();

        let solidity_version = extract_solidity_version(source);
        let is_modern_solidity = is_version_gte_0_8(&solidity_version);

        let mut matches = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut dedup_set = std::collections::HashSet::new();

        // Fast regex-based protection detection (compiled once at init)
        let has_reentrancy_guard = self.reentrancy_guard_regex.as_ref().map(|r| r.is_match(source).unwrap_or(false)).unwrap_or(false);
        let has_access_control = self.access_control_regex.as_ref().map(|r| r.is_match(source).unwrap_or(false)).unwrap_or(false);
        let has_pausable = self.pausable_regex.as_ref().map(|r| r.is_match(source).unwrap_or(false)).unwrap_or(false);

        for compiled_template in &self.templates {
            if is_modern_solidity && compiled_template.template.id.contains("integer_overflow") {
                continue;
            }

            let _is_zeroday_template = compiled_template.template.id.contains("zero-day");

            for compiled_pattern in &compiled_template.patterns {
                // All patterns are now regex-based
                for mat in compiled_pattern.regex.find_iter(source) {
                    let mat = match mat {
                        Ok(m) => m,
                        Err(_) => continue,
                    };
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

                    // Fast regex-based false positive filtering
                    if self.is_reentrancy_pattern(&compiled_template.template.id) {
                        // Check for reentrancy guards in function context
                        if has_reentrancy_guard {
                            if let Some(ref regex) = self.reentrancy_guard_regex {
                                if regex.is_match(&context).unwrap_or(false) {
                                    continue;
                                }
                            }
                        }
                        // Check for access control modifiers
                        if has_access_control {
                            if let Some(ref regex) = self.access_control_regex {
                                if regex.is_match(&context).unwrap_or(false) {
                                    continue;
                                }
                            }
                        }
                        // REMOVED: require check - too aggressive, causes false negatives
                        // The presence of require() doesn't guarantee safety
                    }
                    
                    // Check for pausable modifier
                    if has_pausable {
                        if let Some(ref regex) = self.pausable_regex {
                            if regex.is_match(&context).unwrap_or(false) {
                                continue;
                            }
                        }
                    }

                    // OpenZeppelin whitelist for library code
                    if is_openzeppelin_library(source) && is_openzeppelin_safe_pattern(source, &context, mat.as_str()) {
                        continue;
                    }

                    let mut filtered = false;

                    // Filter OpenZeppelin Address library functions
                    if let Some(ref regex) = self.oz_address_lib_regex {
                        if regex.is_match(&context).unwrap_or(false) {
                            filtered = true;
                        }
                    }

                    // Filter standard proxy patterns
                    if let Some(ref regex) = self.proxy_pattern_regex {
                        if regex.is_match(&context).unwrap_or(false) {
                            filtered = true;
                        }
                    }

                    // Filter safe NFT patterns (ERC721/ERC1155 standard functions)
                    if let Some(ref regex) = self.safe_nft_pattern_regex {
                        if regex.is_match(&context).unwrap_or(false) {
                            // Additional check: ensure it's in a standard NFT function
                            if context.contains("_mint") || context.contains("_burn") || context.contains("_transfer") {
                                filtered = true;
                            }
                        }
                    }

                    // Filter timestamp dependence patterns (low severity)
                    if let Some(ref regex) = self.timestamp_pattern_regex {
                        if regex.is_match(&context).unwrap_or(false) {
                            filtered = true;
                        }
                    }

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
                        filtered,
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

        Ok(matches)
    }

    /// Check if template is reentrancy-related
    fn is_reentrancy_pattern(&self, template_id: &str) -> bool {
        template_id.contains("reentrancy")
            || template_id.contains("external-call")
            || template_id.contains("low-level-call")
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
    const MAX_CONTEXT_CHARS: usize = 1500;
    const CONTEXT_LINES: usize = 15;

    let match_len = match_end - match_start;

    if match_len > MAX_CONTEXT_CHARS {
        let raw_start = match_start.saturating_sub(150);
        let raw_end = (match_end + 150).min(source.len());
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
    let pragma_regex = fancy_regex::Regex::new(r"pragma\s+solidity\s+([^;]+);").ok()?;
    pragma_regex
        .captures(source).ok()?
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
    let version_regex = fancy_regex::Regex::new(r"(\d+)\.(\d+)").ok();
    if let Some(regex) = version_regex {
        if let Some(cap) = regex.captures(version).ok().flatten() {
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
