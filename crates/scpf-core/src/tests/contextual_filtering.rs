#[cfg(test)]
mod contextual_filtering_tests {
    use crate::Scanner;
    use scpf_types::{Pattern, PatternKind, Severity, Template};
    use std::path::PathBuf;

    #[test]
    fn test_reentrancy_guard_filtering() {
        let source = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Test {
    uint256 private balance;
    
    // Vulnerable - no reentrancy guard in contract
    function withdrawVulnerable() external {
        msg.sender.call{value: 1 ether}("");
        balance = 0;
    }
}
"#;

        let template = Template {
            id: "test-reentrancy".to_string(),
            name: "Test Reentrancy".to_string(),
            description: "Test".to_string(),
            severity: Severity::High,
            tags: vec!["reentrancy".to_string()],
            patterns: vec![Pattern {
                id: "low-level-call".to_string(),
                pattern: r"\.call\{value:".to_string(),
                message: "External call with value".to_string(),
                kind: PatternKind::Regex,
            }],
        };

        let mut scanner = Scanner::new(vec![template]).unwrap();
        let matches = scanner.scan(source, PathBuf::from("test.sol")).unwrap();

        assert_eq!(
            matches.len(),
            1,
            "Expected 1 finding, got {}",
            matches.len()
        );
    }

    #[test]
    fn test_access_control_filtering() {
        let source = r#"
pragma solidity ^0.8.0;

contract Test {
    // Vulnerable - no access control in contract
    function publicCall() external {
        msg.sender.call{value: 1 ether}("");
    }
}
"#;

        let template = Template {
            id: "test-access-control".to_string(),
            name: "Test Access Control".to_string(),
            description: "Test".to_string(),
            severity: Severity::High,
            tags: vec!["access".to_string()],
            patterns: vec![Pattern {
                id: "low-level-call".to_string(),
                pattern: r"\.call\{value:".to_string(),
                message: "External call with value".to_string(),
                kind: PatternKind::Regex,
            }],
        };

        let mut scanner = Scanner::new(vec![template]).unwrap();
        let matches = scanner.scan(source, PathBuf::from("test.sol")).unwrap();

        assert_eq!(
            matches.len(),
            1,
            "Expected 1 finding, got {}",
            matches.len()
        );
    }
}
