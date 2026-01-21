#!/bin/bash
# Quick local test of contextual filtering

set -e

echo "=== Local Contextual Filtering Test ==="
echo

# Build
cargo build --release --quiet 2>&1 | grep -v "warning:" || true

# Create temp project
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
mkdir -p contracts

# Copy test file
cp /home/teycir/Repos/SmartContractPatternFinder/sol/realistic_defi.sol contracts/

# Copy templates
cp -r /home/teycir/Repos/SmartContractPatternFinder/templates .

echo "Test contract: realistic_defi.sol"
echo "Expected results:"
echo "  - 5 total call{value: patterns"
echo "  - 3 protected (nonReentrant or onlyOwner)"
echo "  - 2 vulnerable (no protection)"
echo "  - Expected findings: 2"
echo

# Scan
OUTPUT=$(/home/teycir/Repos/SmartContractPatternFinder/target/release/scpf scan 2>&1 || true)

echo "$OUTPUT"
echo

# Extract findings
FINDINGS=$(echo "$OUTPUT" | grep -oP "Total issues: \K\d+" || echo "0")

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Results:"
echo "  Total findings: $FINDINGS"
echo "  Expected: 2 (withdrawVulnerable, claimRewards)"
echo

if [ "$FINDINGS" -eq 2 ]; then
    echo "✓ TEST PASSED: Filtering working correctly!"
    exit 0
elif [ "$FINDINGS" -eq 5 ]; then
    echo "✗ TEST FAILED: No filtering applied (all 5 patterns reported)"
    exit 1
else
    echo "⚠ UNEXPECTED: Got $FINDINGS findings (expected 2)"
    exit 1
fi
