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
    bool private locked;
    
    modifier nonReentrant() {
        require(!locked);
        locked = true;
        _;
        locked = false;
    }
    
    // Protected - should be filtered
    function withdrawProtected() external nonReentrant {
        msg.sender.call{value: 1 ether}("");
    }
    
    // Vulnerable - should be reported
    function withdrawVulnerable() external {
        msg.sender.call{value: 1 ether}("");
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
            "Expected 1 finding (withdrawVulnerable only), got {}",
            matches.len()
        );
        assert_eq!(matches[0].line_number, 22, "Expected finding on line 22");
    }

    #[test]
    fn test_access_control_filtering() {
        let source = r#"
pragma solidity ^0.8.0;

contract Test {
    address public owner;
    
    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }
    
    // Protected - should be filtered
    function adminCall() external onlyOwner {
        msg.sender.call{value: 1 ether}("");
    }
    
    // Vulnerable - should be reported
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
            "Expected 1 finding (publicCall only), got {}",
            matches.len()
        );
        assert_eq!(matches[0].line_number, 19, "Expected finding on line 19");
    }
}
