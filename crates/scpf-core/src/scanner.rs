use crate::ast::{AstAnalyzer, ValidationResult};
use crate::contract_type::ContractTypeDetector;
use crate::regex_validator::RegexValidator;
use anyhow::Result;
use fancy_regex::RegexBuilder;
use rayon::prelude::*;
use scpf_types::{Match, Pattern, PatternKind, Template};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::warn;

/// Shared string type to avoid cloning template/pattern IDs for every match
type SharedStr = Arc<str>;

enum CompiledMatcher {
    Regex(regex::Regex),
    FancyRegex(fancy_regex::Regex),
    Literal(String),
}

struct CompiledPatternOptimized {
    matcher: CompiledMatcher,
    pattern_id: SharedStr,
    message: SharedStr,
    index: u32,
}

struct CompiledTemplateOptimized {
    template_id: SharedStr,
    severity: scpf_types::Severity,
    patterns: Vec<CompiledPatternOptimized>,
}

/// Context for parallel scanning - shared immutable data
struct ScanContext<'a> {
    source: &'a str,
    newlines: &'a [usize],
    lines: &'a [&'a str],
    file_path: &'a PathBuf,
    has_reentrancy_guard: bool,
    has_access_control: bool,
    has_pausable: bool,
    is_oz_library: bool,
}

pub struct Scanner {
    templates: Vec<CompiledTemplateOptimized>,
    reentrancy_guard_regex: Option<fancy_regex::Regex>,
    access_control_regex: Option<fancy_regex::Regex>,
    pausable_regex: Option<fancy_regex::Regex>,
    oz_address_lib_regex: Option<fancy_regex::Regex>,
    proxy_pattern_regex: Option<fancy_regex::Regex>,
    safe_nft_pattern_regex: Option<fancy_regex::Regex>,
    timestamp_pattern_regex: Option<fancy_regex::Regex>,
    ast_analyzer: Option<AstAnalyzer>,
}

impl Scanner {
    pub fn new(templates: Vec<Template>) -> Result<Self> {
        let mut compiled_templates = Vec::with_capacity(templates.len());
        let mut pattern_index = 0u32;

        for template in templates {
            if let Some(compiled) = compile_template_optimized(template, &mut pattern_index)? {
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
            ast_analyzer: Some(AstAnalyzer::new()),
        })
    }

    /// Scan source code for vulnerabilities using parallel pattern matching with rayon
    pub fn scan(&self, source: &str, file_path: PathBuf) -> Result<Vec<Match>> {
        const MAX_FILE_SIZE: usize = 1_572_864; // 1.5MB hard limit (1536 KB)
        const CHUNK_THRESHOLD: usize = 700_000; // 700KB - use chunking above this
        const CHUNK_SIZE: usize = 200_000; // 200KB chunks

        // Skip files > 1.5MB
        if source.len() > MAX_FILE_SIZE {
            warn!(
                "File {} size {} exceeds 1.5MB limit, skipping",
                file_path.display(),
                source.len()
            );
            return Ok(Vec::new());
        }

        // Use chunking for files 700KB-1.5MB
        if source.len() > CHUNK_THRESHOLD {
            warn!(
                "File {} size {} exceeds 700KB, scanning in chunks",
                file_path.display(),
                source.len()
            );
            return self.scan_chunked(source, file_path, CHUNK_SIZE);
        }

        // Pre-compute newline positions once
        let newlines: Vec<usize> = source.match_indices('\n').map(|(i, _)| i).collect();

        // Cache lines for snippet extraction (avoids re-parsing)
        let lines: Vec<&str> = source.lines().collect();

        let solidity_version = extract_solidity_version(source);
        let is_modern_solidity = is_version_gte_0_8(&solidity_version);

        // Fast regex-based protection detection (compiled once at init)
        let has_reentrancy_guard = self
            .reentrancy_guard_regex
            .as_ref()
            .map(|r| r.is_match(source).unwrap_or(false))
            .unwrap_or(false);
        let has_access_control = self
            .access_control_regex
            .as_ref()
            .map(|r| r.is_match(source).unwrap_or(false))
            .unwrap_or(false);
        let has_pausable = self
            .pausable_regex
            .as_ref()
            .map(|r| r.is_match(source).unwrap_or(false))
            .unwrap_or(false);

        // Pre-check if source is OZ library (avoid repeated checks)
        let is_oz_library = is_openzeppelin_library(source);
        
        // Detect contract type for pattern filtering
        let contract_type = ContractTypeDetector::detect(source);

        // Create shared context for parallel processing
        let scan_ctx = ScanContext {
            source,
            newlines: &newlines,
            lines: &lines,
            file_path: &file_path,
            has_reentrancy_guard,
            has_access_control,
            has_pausable,
            is_oz_library,
        };

        // Use rayon to parallelize across templates
        let all_matches: Vec<Vec<Match>> = self
            .templates
            .par_iter()
            .filter(|t| {
                // Skip integer overflow for modern Solidity
                if is_modern_solidity && t.template_id.contains("integer_overflow") {
                    return false;
                }
                // Skip patterns not applicable to contract type
                !t.patterns.iter().any(|p| {
                    ContractTypeDetector::should_skip_pattern(contract_type, &p.pattern_id)
                })
            })
            .map(|compiled_template| self.scan_template(compiled_template, &scan_ctx))
            .collect();

        // Flatten and deduplicate results
        let mut matches: Vec<Match> = all_matches.into_iter().flatten().collect();

        // AST-based validation (second pass)
        if let Some(ref analyzer) = self.ast_analyzer {
            matches.retain(|m| {
                let result = analyzer.validate(source, &m.pattern_id, m.line_number);
                match result {
                    ValidationResult::Vulnerable => true,
                    ValidationResult::Protected(_) => false,
                    ValidationResult::NotApplicable => true,
                    ValidationResult::ParseError => true,
                }
            });
        }

        // Global deduplication
        let mut seen = std::collections::HashSet::new();
        matches.retain(|m| {
            let key = (m.line_number, m.column, m.pattern_id.clone());
            seen.insert(key)
        });

        // Filter out interface declarations (functions ending with semicolon)
        matches.retain(|m| {
            // Check if matched text or context contains interface pattern
            let is_interface = m.context.contains(";")
                && !m.context.contains("{")
                && (m.matched_text.contains("function") || m.context.contains("function"));
            !is_interface
        });

        // Apply global limit
        const MAX_PATTERNS_PER_FILE: usize = 10000;
        if matches.len() > MAX_PATTERNS_PER_FILE {
            warn!(
                "Pattern limit reached for {}, truncating to {}",
                file_path.display(),
                MAX_PATTERNS_PER_FILE
            );
            matches.truncate(MAX_PATTERNS_PER_FILE);
        }

        Ok(matches)
    }

    /// Scan a single template (called in parallel by rayon)
    fn scan_template(
        &self,
        compiled_template: &CompiledTemplateOptimized,
        ctx: &ScanContext,
    ) -> Vec<Match> {
        let is_reentrancy_template = self.is_reentrancy_pattern(&compiled_template.template_id);
        let mut template_matches = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for compiled_pattern in &compiled_template.patterns {
            // Match based on pattern type
            let matches: Vec<(usize, usize, String)> = match &compiled_pattern.matcher {
                CompiledMatcher::Regex(re) => {
                    re.find_iter(ctx.source)
                        .map(|m| (m.start(), m.end(), m.as_str().to_string()))
                        .collect()
                }
                CompiledMatcher::FancyRegex(re) => {
                    re.find_iter(ctx.source)
                        .filter_map(|m| m.ok())
                        .map(|m| (m.start(), m.end(), m.as_str().to_string()))
                        .collect()
                }
                CompiledMatcher::Literal(literal) => {
                    ctx.source
                        .match_indices(literal.as_str())
                        .map(|(start, matched)| (start, start + matched.len(), matched.to_string()))
                        .collect()
                }
            };

            for (start, end, matched_text) in matches {
                let key = (start, end, compiled_pattern.index);
                if !seen.insert(key) {
                    continue;
                }

                let line_number = ctx.newlines.partition_point(|&pos| pos < start) + 1;
                let line_start = if line_number > 1 {
                    ctx.newlines[line_number - 2] + 1
                } else {
                    0
                };

                let context = get_match_context(
                    ctx.source,
                    ctx.newlines,
                    start,
                    end,
                    line_number,
                );

                // Fast regex-based false positive filtering
                if is_reentrancy_template {
                    if ctx.has_reentrancy_guard {
                        if let Some(ref regex) = self.reentrancy_guard_regex {
                            if regex.is_match(&context).unwrap_or(false) {
                                continue;
                            }
                        }
                    }
                    if ctx.has_access_control {
                        if let Some(ref regex) = self.access_control_regex {
                            if regex.is_match(&context).unwrap_or(false) {
                                continue;
                            }
                        }
                    }
                }

                if ctx.has_pausable {
                    if let Some(ref regex) = self.pausable_regex {
                        if regex.is_match(&context).unwrap_or(false) {
                            continue;
                        }
                    }
                }

                if ctx.is_oz_library
                    && is_openzeppelin_safe_pattern(ctx.source, &context, &matched_text)
                {
                    continue;
                }

                let filtered = self.check_filtered(&context);

                template_matches.push(Match {
                    template_id: compiled_template.template_id.to_string(),
                    pattern_id: compiled_pattern.pattern_id.to_string(),
                    file_path: ctx.file_path.clone(),
                    line_number,
                    column: start - line_start,
                    matched_text,
                    context,
                    code_snippet: extract_code_snippet_cached(ctx.lines, line_number),
                    severity: compiled_template.severity,
                    message: compiled_pattern.message.to_string(),
                    start_byte: None,
                    end_byte: None,
                    function_context: None,
                    protections: None,
                    filtered,
                });
            }
        }
        template_matches
    }

    /// Check if match should be filtered
    #[inline]
    fn check_filtered(&self, context: &str) -> bool {
        // OpenZeppelin Address.sendValue pattern
        if context.contains("function sendValue(address payable recipient, uint256 amount) internal") {
            return true;
        }
        if context.contains("Address: insufficient balance") || context.contains("Address: unable to send value") {
            return true;
        }
        
        // Chainlink oracle patterns
        if context.contains("interface AggregatorV3Interface") 
            || context.contains("EACAggregatorProxy")
            || context.contains("function latestRoundData()") {
            return true;
        }
        
        // Diamond proxy (EIP-2535) patterns
        if context.contains("ds.selectorToFacetAndPosition") 
            || context.contains("LibDiamond")
            || context.contains("IDiamondCut") {
            return true;
        }
        
        // Existing filters
        if let Some(ref regex) = self.oz_address_lib_regex {
            if regex.is_match(context).unwrap_or(false) {
                return true;
            }
        }
        if let Some(ref regex) = self.proxy_pattern_regex {
            if regex.is_match(context).unwrap_or(false) {
                return true;
            }
        }
        if let Some(ref regex) = self.safe_nft_pattern_regex {
            if regex.is_match(context).unwrap_or(false) {
                if context.contains("_mint")
                    || context.contains("_burn")
                    || context.contains("_transfer")
                {
                    return true;
                }
            }
        }
        if let Some(ref regex) = self.timestamp_pattern_regex {
            if regex.is_match(context).unwrap_or(false) {
                return true;
            }
        }
        false
    }

    /// Check if template is reentrancy-related
    fn is_reentrancy_pattern(&self, template_id: &str) -> bool {
        template_id.contains("reentrancy")
            || template_id.contains("external-call")
            || template_id.contains("low-level-call")
    }

    /// Scan large files in overlapping chunks to avoid memory issues
    fn scan_chunked(
        &self,
        source: &str,
        file_path: PathBuf,
        chunk_size: usize,
    ) -> Result<Vec<Match>> {
        use crate::chunking::ChunkProcessor;

        const OVERLAP: usize = 50_000;
        let processor = ChunkProcessor::new(chunk_size, OVERLAP);
        let mut seen_keys = std::collections::HashSet::new();

        let all_matches = processor.process(source, |chunk, line_offset| {
            self.scan_chunk_direct(chunk, &file_path, line_offset)
        })?;

        // Deduplicate
        let mut deduped = Vec::new();
        for m in all_matches {
            let key = (m.line_number, m.column, m.pattern_id.clone());
            if seen_keys.insert(key) {
                deduped.push(m);
            }
        }

        tracing::info!(
            "Scanned {} in chunks, found {} matches",
            file_path.display(),
            deduped.len()
        );
        Ok(deduped)
    }

    /// Scan a chunk directly without size checks (internal use only)
    fn scan_chunk_direct(
        &self,
        source: &str,
        file_path: &PathBuf,
        line_offset: usize,
    ) -> Result<Vec<Match>> {
        // Pre-compute newline positions once
        let newlines: Vec<usize> = source.match_indices('\n').map(|(i, _)| i).collect();

        // Cache lines for snippet extraction
        let lines: Vec<&str> = source.lines().collect();

        let solidity_version = extract_solidity_version(source);
        let is_modern_solidity = is_version_gte_0_8(&solidity_version);

        let has_reentrancy_guard = self
            .reentrancy_guard_regex
            .as_ref()
            .map(|r| r.is_match(source).unwrap_or(false))
            .unwrap_or(false);
        let has_access_control = self
            .access_control_regex
            .as_ref()
            .map(|r| r.is_match(source).unwrap_or(false))
            .unwrap_or(false);
        let has_pausable = self
            .pausable_regex
            .as_ref()
            .map(|r| r.is_match(source).unwrap_or(false))
            .unwrap_or(false);
        let is_oz_library = is_openzeppelin_library(source);

        let scan_ctx = ScanContext {
            source,
            newlines: &newlines,
            lines: &lines,
            file_path,
            has_reentrancy_guard,
            has_access_control,
            has_pausable,
            is_oz_library,
        };

        let all_matches: Vec<Vec<Match>> = self
            .templates
            .iter()
            .filter(|t| !(is_modern_solidity && t.template_id.contains("integer_overflow")))
            .map(|compiled_template| self.scan_template(compiled_template, &scan_ctx))
            .collect();

        let mut matches: Vec<Match> = all_matches.into_iter().flatten().collect();

        // Adjust line numbers based on line_offset
        for m in &mut matches {
            m.line_number += line_offset;
        }

        Ok(matches)
    }
}

fn compile_pattern_optimized(
    pattern: &Pattern,
    template_id: &str,
    index: u32,
) -> Result<CompiledPatternOptimized> {
    let matcher = match pattern.kind {
        PatternKind::Regex => {
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

            let multiline_pattern = format!("(?m){}", pattern.pattern);
            let re = regex::Regex::new(&multiline_pattern).map_err(|e| {
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
            CompiledMatcher::Regex(re)
        }
        PatternKind::FancyRegex => {
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

            let multiline_pattern = format!("(?m){}", pattern.pattern);
            let re = RegexBuilder::new(&multiline_pattern).build().map_err(|e| {
                warn!(
                    "Invalid fancy-regex in template '{}', pattern '{}': {}",
                    template_id, pattern.id, e
                );
                anyhow::anyhow!(
                    "Invalid fancy-regex in template '{}', pattern '{}': {}",
                    template_id,
                    pattern.id,
                    e
                )
            })?;
            CompiledMatcher::FancyRegex(re)
        }
        PatternKind::Literal => {
            if pattern.pattern.is_empty() {
                anyhow::bail!(
                    "Empty literal pattern in template '{}', pattern '{}'",
                    template_id,
                    pattern.id
                );
            }
            CompiledMatcher::Literal(pattern.pattern.clone())
        }
        PatternKind::Semantic => {
            // Semantic patterns are handled separately, skip compilation
            return Err(anyhow::anyhow!(
                "Semantic patterns should not be compiled in scanner"
            ));
        }
    };

    Ok(CompiledPatternOptimized {
        matcher,
        pattern_id: Arc::from(pattern.id.as_str()),
        message: Arc::from(pattern.message.as_str()),
        index,
    })
}

fn compile_template_optimized(
    template: Template,
    pattern_index: &mut u32,
) -> Result<Option<CompiledTemplateOptimized>> {
    let mut compiled_patterns = Vec::with_capacity(template.patterns.len());

    for pattern in &template.patterns {
        // Skip semantic patterns - they're handled separately
        if pattern.kind == PatternKind::Semantic {
            continue;
        }
        
        match compile_pattern_optimized(pattern, &template.id, *pattern_index) {
            Ok(compiled) => {
                compiled_patterns.push(compiled);
                *pattern_index += 1;
            }
            Err(e) => {
                warn!("Skipping pattern '{}' in template '{}': {}", pattern.id, template.id, e);
            }
        }
    }

    if compiled_patterns.is_empty() {
        return Ok(None);
    }

    Ok(Some(CompiledTemplateOptimized {
        template_id: Arc::from(template.id.as_str()),
        severity: template.severity,
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

/// Optimized code snippet extraction using pre-cached lines
fn extract_code_snippet_cached(
    lines: &[&str],
    line_number: usize,
) -> Option<scpf_types::CodeSnippet> {
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
        .captures(source)
        .ok()?
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
    let has_library_indicators =
        source.contains("library ") || source.contains("abstract contract");
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
        if context.contains("target.delegatecall(data)") || context.contains("functionDelegateCall")
        {
            return true;
        }
        // Proxy-related contexts
        if context.contains("_implementation")
            || context.contains("Proxy")
            || context.contains("@dev This abstract contract")
        {
            return true;
        }
    }

    false
}
