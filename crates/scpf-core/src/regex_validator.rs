use anyhow::Result;

/// Validates regex patterns for potential DoS vulnerabilities
pub struct RegexValidator;

impl RegexValidator {
    /// Check if a regex pattern is safe from catastrophic backtracking
    pub fn validate_pattern(pattern: &str) -> Result<()> {
        Self::check_nested_quantifiers(pattern)?;
        Self::check_pattern_complexity(pattern)?;
        Ok(())
    }

    /// Detect nested quantifiers like (a+)+ or (a*)* that can cause exponential backtracking
    fn check_nested_quantifiers(pattern: &str) -> Result<()> {
        let dangerous_patterns = [
            r"\([^)]*[*+]\)[*+]",           // (x*)* or (x+)+
            r"\([^)]*[*+]\)\{",             // (x*){n,m}
            r"\[[^\]]*[*+]\][*+]",          // [x*]* or [x+]+
        ];

        for dangerous in &dangerous_patterns {
            if let Ok(re) = regex::Regex::new(dangerous) {
                if re.is_match(pattern) {
                    anyhow::bail!(
                        "Potentially dangerous nested quantifier detected in pattern: {}. \
                        This can cause catastrophic backtracking.",
                        pattern
                    );
                }
            }
        }

        Ok(())
    }

    /// Check if pattern complexity is within reasonable limits
    fn check_pattern_complexity(pattern: &str) -> Result<()> {
        const MAX_PATTERN_LENGTH: usize = 1000;
        const MAX_ALTERNATIONS: usize = 50;
        const MAX_GROUPS: usize = 20;

        if pattern.len() > MAX_PATTERN_LENGTH {
            anyhow::bail!(
                "Pattern too long ({} chars). Maximum allowed: {}",
                pattern.len(),
                MAX_PATTERN_LENGTH
            );
        }

        let alternation_count = pattern.matches('|').count();
        if alternation_count > MAX_ALTERNATIONS {
            anyhow::bail!(
                "Too many alternations ({}) in pattern. Maximum allowed: {}",
                alternation_count,
                MAX_ALTERNATIONS
            );
        }

        let group_count = pattern.matches('(').count();
        if group_count > MAX_GROUPS {
            anyhow::bail!(
                "Too many groups ({}) in pattern. Maximum allowed: {}",
                group_count,
                MAX_GROUPS
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_patterns() {
        assert!(RegexValidator::validate_pattern(r"\.call\{value:").is_ok());
        assert!(RegexValidator::validate_pattern(r"function\s+\w+").is_ok());
        assert!(RegexValidator::validate_pattern(r"[a-zA-Z]+").is_ok());
    }

    #[test]
    fn test_nested_quantifiers() {
        assert!(RegexValidator::validate_pattern(r"(a+)+").is_err());
        assert!(RegexValidator::validate_pattern(r"(a*)*").is_err());
        assert!(RegexValidator::validate_pattern(r"(a+){2,}").is_err());
    }

    #[test]
    fn test_pattern_too_long() {
        let long_pattern = "a".repeat(1001);
        assert!(RegexValidator::validate_pattern(&long_pattern).is_err());
    }

    #[test]
    fn test_too_many_alternations() {
        let pattern = (0..52).map(|i| format!("a{}", i)).collect::<Vec<_>>().join("|");
        assert!(RegexValidator::validate_pattern(&pattern).is_err());
    }

    #[test]
    fn test_too_many_groups() {
        let pattern = "(a)".repeat(21);
        assert!(RegexValidator::validate_pattern(&pattern).is_err());
    }
}
