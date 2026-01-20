use scpf_types::{Match, Severity, Template};

pub struct ScanFilters {
    pub min_severity: Option<Severity>,
    pub tags: Vec<String>,
    pub exclude_templates: Vec<String>,
    pub only_templates: Vec<String>,
}

impl ScanFilters {
    pub fn from_args(
        min_severity: Option<String>,
        tags: Option<String>,
        exclude_templates: Option<String>,
        only_templates: Option<String>,
    ) -> Self {
        Self {
            min_severity: min_severity.and_then(|s| parse_severity(&s)),
            tags: tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            exclude_templates: exclude_templates
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            only_templates: only_templates
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
        }
    }

    pub fn filter_templates(&self, templates: Vec<Template>) -> Vec<Template> {
        templates
            .into_iter()
            .filter(|t| {
                // If only_templates specified, only include those
                if !self.only_templates.is_empty() {
                    if !self.only_templates.contains(&t.id) {
                        return false;
                    }
                }

                // Exclude specified templates
                if self.exclude_templates.contains(&t.id) {
                    return false;
                }

                // Filter by tags if specified
                if !self.tags.is_empty() {
                    let has_matching_tag = t.tags.iter().any(|tag| self.tags.contains(tag));
                    if !has_matching_tag {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    pub fn filter_matches(&self, matches: Vec<Match>) -> Vec<Match> {
        matches
            .into_iter()
            .filter(|m| {
                // Filter by minimum severity
                if let Some(min_sev) = &self.min_severity {
                    if m.severity < *min_sev {
                        return false;
                    }
                }

                true
            })
            .collect()
    }
}

fn parse_severity(s: &str) -> Option<Severity> {
    match s.to_lowercase().as_str() {
        "critical" => Some(Severity::Critical),
        "high" => Some(Severity::High),
        "medium" => Some(Severity::Medium),
        "low" => Some(Severity::Low),
        "info" => Some(Severity::Info),
        _ => None,
    }
}

pub fn detect_contract_type(source: &str) -> Vec<String> {
    let mut types = Vec::new();

    // ERC20
    if source.contains("function transfer(") && source.contains("function balanceOf(") {
        types.push("erc20".to_string());
    }

    // ERC721
    if source.contains("function ownerOf(") && source.contains("function safeTransferFrom(") {
        types.push("erc721".to_string());
    }

    // ERC1155
    if source.contains("function balanceOfBatch(") && source.contains("function safeBatchTransferFrom(") {
        types.push("erc1155".to_string());
    }

    // Proxy patterns
    if source.contains("delegatecall") || source.contains("implementation()") {
        types.push("proxy".to_string());
    }

    // DeFi patterns
    if source.contains("swap") || source.contains("liquidity") || source.contains("pool") {
        types.push("defi".to_string());
    }

    types
}
