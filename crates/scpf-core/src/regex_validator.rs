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

    /// Detect nested quantifiers like (a+)+ or (a*)* that can cause exponential backtracking.
    /// Uses escape-aware parsing to properly handle escaped characters and parentheses.
    fn check_nested_quantifiers(pattern: &str) -> Result<()> {
        Self::scan_for_nested_quantifiers(pattern)?;
        Ok(())
    }

    /// Escape-aware scanner to detect nested quantifiers in regex patterns.
    /// Properly handles escaped characters (\(, \), etc.) and tracks quantifier nesting.
    fn scan_for_nested_quantifiers(pattern: &str) -> Result<()> {
        let bytes = pattern.as_bytes();
        let mut i = 0;
        let mut paren_stack: Vec<(usize, bool)> = Vec::new(); // (depth, has_quantifier_inside)

        while i < bytes.len() {
            let ch = bytes[i];

            // Handle escape sequences
            if ch == b'\\' && i + 1 < bytes.len() {
                i += 2; // Skip the backslash and the next character
                continue;
            }

            // Track opening parenthesis (unescaped)
            if ch == b'(' {
                paren_stack.push((paren_stack.len(), false));
                i += 1;
                continue;
            }

            // Track closing parenthesis (unescaped)
            if ch == b')' {
                if let Some((_, has_quant)) = paren_stack.pop() {
                    // After closing ), check if there's a quantifier that applies to this group
                    if i + 1 < bytes.len() {
                        let next = bytes[i + 1];
                        // Check for outer quantifier: *, +, {, or ?
                        if next == b'*' || next == b'+' || next == b'{' || next == b'?' {
                            // If the group contained a quantifier, we have nested quantifiers
                            if has_quant {
                                anyhow::bail!(
                                    "Potentially dangerous nested quantifier detected in pattern: {}. \
                                    This can cause catastrophic backtracking.",
                                    pattern
                                );
                            }
                        }
                    }
                    // Propagate quantifier info up the stack
                    if has_quant && !paren_stack.is_empty() {
                        if let Some(last) = paren_stack.last_mut() {
                            last.1 = true;
                        }
                    }
                }
                i += 1;
                continue;
            }

            // Check for quantifiers at current level
            if (ch == b'*' || ch == b'+' || ch == b'{' || ch == b'?') && !paren_stack.is_empty() {
                if let Some(last) = paren_stack.last_mut() {
                    last.1 = true;
                }
            }

            i += 1;
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
        // Character classes don't allow * or + as quantifiers, so [x*]* is actually safe
        assert!(RegexValidator::validate_pattern(r"[a*]+").is_ok());
    }

    #[test]
    fn test_nested_quantifiers() {
        assert!(RegexValidator::validate_pattern(r"(a+)+").is_err());
        assert!(RegexValidator::validate_pattern(r"(a*)*").is_err());
        assert!(RegexValidator::validate_pattern(r"(a+){2,}").is_err());
        assert!(RegexValidator::validate_pattern(r"(a*)+").is_err());
    }

    #[test]
    fn test_escaped_parentheses_safe() {
        // Escaped parentheses should not be counted for grouping
        assert!(RegexValidator::validate_pattern(r"\(a+\)+").is_ok());
        assert!(RegexValidator::validate_pattern(r"\(a\)\+").is_ok());
    }

    #[test]
    fn test_pattern_too_long() {
        let long_pattern = "a".repeat(1001);
        assert!(RegexValidator::validate_pattern(&long_pattern).is_err());
    }

    #[test]
    fn test_too_many_alternations() {
        let pattern = (0..52)
            .map(|i| format!("a{}", i))
            .collect::<Vec<_>>()
            .join("|");
        assert!(RegexValidator::validate_pattern(&pattern).is_err());
    }

    #[test]
    fn test_too_many_groups() {
        let pattern = "(a)".repeat(21);
        assert!(RegexValidator::validate_pattern(&pattern).is_err());
    }
}
