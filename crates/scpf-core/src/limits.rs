/// Security limits for pattern matching to prevent ReDoS and resource exhaustion
#[derive(Debug, Clone, Copy)]
pub struct MatcherLimits {
    pub max_patterns: usize,
    pub max_regex_length: usize,
    pub max_matches: usize,
    pub max_file_size: u64,
}

impl Default for MatcherLimits {
    fn default() -> Self {
        Self {
            max_patterns: 1000,
            max_regex_length: 500,
            max_matches: 10_000,
            max_file_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Security limits for template loading to prevent YAML bombs
#[derive(Debug, Clone, Copy)]
pub struct LoaderLimits {
    pub max_file_size: u64,
    pub max_lines: usize,
    pub max_templates: usize,
}

impl Default for LoaderLimits {
    fn default() -> Self {
        Self {
            max_file_size: 1024 * 1024, // 1MB
            max_lines: 10_000,
            max_templates: 100,
        }
    }
}
