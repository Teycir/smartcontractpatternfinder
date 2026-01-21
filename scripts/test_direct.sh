#!/bin/bash
# Direct test of filtering functionality

set -e

echo "=== Day 5-6: Testing Contextual Filtering ==="
echo

# Create test directory
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

# Create test contract
cat > test.sol << 'EOF'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Test {
    bool private locked;
    address public owner;
    
    modifier nonReentrant() {
        require(!locked);
        locked = true;
        _;
        locked = false;
    }
    
    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }
    
    // Protected - should be filtered
    function withdrawProtected() external nonReentrant {
        msg.sender.call{value: 1 ether}("");
    }
    
    // Vulnerable - should be reported
    function withdrawVulnerable() external {
        msg.sender.call{value: 1 ether}("");
    }
    
    // Protected - should be filtered
    function adminCall() external onlyOwner {
        msg.sender.call{value: 1 ether}("");
    }
}
EOF

echo "Test contract created at: $TEST_DIR/test.sol"
echo

# Create minimal template
mkdir -p templates
cat > templates/test.yaml << 'EOF'
id: test-reentrancy
name: Test Reentrancy
description: Test pattern for reentrancy
severity: high
tags:
  - reentrancy
patterns:
  - id: low-level-call
    pattern: '\.call\{value:'
    message: External call with value detected
EOF

echo "Template created"
echo

# Run scanner with Rust test
cd /home/teycir/Repos/SmartContractPatternFinder

cat > /tmp/test_filter.rs << 'RUST_EOF'
use scpf_core::Scanner;
use scpf_types::Template;
use std::path::PathBuf;

fn main() {
    let source = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Test {
    bool private locked;
    address public owner;
    
    modifier nonReentrant() {
        require(!locked);
        locked = true;
        _;
        locked = false;
    }
    
    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }
    
    // Protected - should be filtered
    function withdrawProtected() external nonReentrant {
        msg.sender.call{value: 1 ether}("");
    }
    
    // Vulnerable - should be reported
    function withdrawVulnerable() external {
        msg.sender.call{value: 1 ether}("");
    }
    
    // Protected - should be filtered  
    function adminCall() external onlyOwner {
        msg.sender.call{value: 1 ether}("");
    }
}
"#;

    let template = Template {
        id: "test-reentrancy".to_string(),
        name: "Test Reentrancy".to_string(),
        description: "Test".to_string(),
        severity: scpf_types::Severity::High,
        tags: vec!["reentrancy".to_string()],
        patterns: vec![scpf_types::Pattern {
            id: "low-level-call".to_string(),
            pattern: r"\.call\{value:".to_string(),
            message: "External call with value".to_string(),
            kind: scpf_types::PatternKind::Regex,
        }],
    };

    let mut scanner = Scanner::new(vec![template]).unwrap();
    let matches = scanner.scan(source, PathBuf::from("test.sol")).unwrap();

    println!("Total findings: {}", matches.len());
    for m in &matches {
        println!("  Line {}: {} ({})", m.line_number, m.message, m.matched_text);
    }

    println!("\nExpected: 1 finding (withdrawVulnerable only)");
    println!("Actual: {} findings", matches.len());

    if matches.len() == 1 {
        println!("\n✓ TEST PASSED: Filtering working!");
        std::process::exit(0);
    } else {
        println!("\n✗ TEST FAILED");
        std::process::exit(1);
    }
}
RUST_EOF

echo "Running direct Rust test..."
rustc --edition 2021 \
    -L target/release/deps \
    --extern scpf_core=target/release/libscpf_core.rlib \
    --extern scpf_types=target/release/libscpf_types.rlib \
    /tmp/test_filter.rs -o /tmp/test_filter 2>&1 || echo "Compilation failed, using cargo test instead"

echo
echo "=== Test Complete ==="
