#!/bin/bash
# Test script for contextual filtering

set -e

echo "=== Testing Contextual Filtering ==="
echo

# Build project
echo "Building project..."
cargo build --release --quiet
echo "✓ Build complete"
echo

# Create test contract
cat > sol/test_protected.sol << 'EOF'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract TestProtected {
    bool private locked;
    address public owner;
    
    modifier nonReentrant() {
        require(!locked, "No reentrancy");
        locked = true;
        _;
        locked = false;
    }
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    // Should NOT be flagged (has nonReentrant)
    function withdrawProtected() external nonReentrant {
        (bool success, ) = msg.sender.call{value: address(this).balance}("");
        require(success);
    }
    
    // SHOULD be flagged (no protection)
    function withdrawVulnerable() external {
        (bool success, ) = msg.sender.call{value: address(this).balance}("");
        require(success);
    }
    
    // Should NOT be flagged (has onlyOwner)
    function adminWithdraw() external onlyOwner {
        (bool success, ) = msg.sender.call{value: address(this).balance}("");
        require(success);
    }
}
EOF

echo "✓ Test contract created"
echo

# Scan and count findings
echo "Scanning test contract..."
OUTPUT=$(cargo run --release --bin scpf -- scan 2>&1 || true)

# Count findings
FINDINGS=$(echo "$OUTPUT" | grep -c "call{value:" || true)

echo "$OUTPUT"
echo
echo "=== Results ==="
echo "Total findings: $FINDINGS"
echo
echo "Expected behavior:"
echo "  - withdrawProtected (line 22): FILTERED (has nonReentrant)"
echo "  - withdrawVulnerable (line 28): REPORTED (no protection)"
echo "  - adminWithdraw (line 34): FILTERED (has onlyOwner)"
echo
echo "Expected: 1 finding (withdrawVulnerable only)"
echo "Actual: $FINDINGS findings"
echo

if [ "$FINDINGS" -eq 1 ]; then
    echo "✓ TEST PASSED: Filtering working correctly!"
    exit 0
else
    echo "✗ TEST FAILED: Expected 1 finding, got $FINDINGS"
    exit 1
fi
