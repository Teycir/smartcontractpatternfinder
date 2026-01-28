use fancy_regex::Regex;

/// Honeypot detection patterns - STRICT mode
pub struct HoneypotFilter {
    patterns: Vec<(String, Regex)>,
}

impl HoneypotFilter {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let patterns = vec![
            // STRICT: tx.origin in ERC20 core functions
            (
                "balance_tx_origin".to_string(),
                Regex::new(r"function\s+balanceOf[^}]{0,500}(tx\.origin|origin\(\))")?,
            ),
            (
                "allowance_tx_origin".to_string(),
                Regex::new(r"function\s+allowance[^}]{0,500}(tx\.origin|origin\(\))")?,
            ),
            (
                "transfer_tx_origin".to_string(),
                Regex::new(r"function\s+transfer[^}]{0,500}(tx\.origin|origin\(\))")?,
            ),
            // STRICT: Known honeypot helper functions
            (
                "hidden_fee_taxPayer".to_string(),
                Regex::new(r"function\s+_taxPayer[^}]{0,300}(tx\.origin|origin\(\))")?,
            ),
            (
                "isSuper_tx_origin".to_string(),
                Regex::new(r"function\s+_isSuper[^}]{0,200}(tx\.origin|origin\(\))")?,
            ),
        ];

        Ok(Self { patterns })
    }

    /// Check if source code contains honeypot patterns
    /// Returns (is_honeypot, matched_patterns)
    /// Requires 2+ patterns for high confidence filtering
    pub fn is_honeypot(&self, source: &str) -> (bool, Vec<String>) {
        let mut matches = Vec::new();

        for (name, pattern) in &self.patterns {
            if let Ok(true) = pattern.is_match(source) {
                matches.push(name.to_string());
            }
        }

        // Require 2+ patterns to avoid false positives
        (matches.len() >= 2, matches)
    }
}

impl Default for HoneypotFilter {
    fn default() -> Self {
        Self::new().expect("Failed to compile honeypot patterns")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_multiple_patterns() {
        let filter = HoneypotFilter::new().unwrap();
        // Single pattern should NOT flag as honeypot
        let source = r#"
            function balanceOf(address account) public view returns (uint256) {
                if (tx.origin != msg.sender) revert();
                return _balances[account];
            }
        "#;
        let (is_honeypot, _) = filter.is_honeypot(source);
        assert!(!is_honeypot);
    }

    #[test]
    fn test_detects_multiple_patterns() {
        let filter = HoneypotFilter::new().unwrap();
        // Multiple patterns = honeypot
        let source = r#"
            function balanceOf(address account) public view returns (uint256) {
                if (tx.origin != msg.sender) return 0;
                return _balances[account];
            }
            function _taxPayer() internal view returns (address) {
                return tx.origin;
            }
        "#;
        let (is_honeypot, matches) = filter.is_honeypot(source);
        assert!(is_honeypot);
        assert!(matches.len() >= 2);
    }

    #[test]
    fn test_clean_contract() {
        let filter = HoneypotFilter::new().unwrap();
        let source = r#"
            function transfer(address to, uint256 amount) public returns (bool) {
                _balances[msg.sender] -= amount;
                _balances[to] += amount;
                return true;
            }
        "#;
        let (is_honeypot, _) = filter.is_honeypot(source);
        assert!(!is_honeypot);
    }
}
