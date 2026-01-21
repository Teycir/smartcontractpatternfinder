#!/bin/bash
# Day 7: Validation Script - Test contextual filtering on real contracts

set -e

echo "=== Day 7: Contextual Filtering Validation ==="
echo

# Build project
echo "Building project..."
cargo build --release --quiet
echo "✓ Build complete"
echo

# Test contracts
CONTRACTS=(
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48:ethereum:USDC"
    "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f:ethereum:UniswapV2Factory"
)

echo "Testing contextual filtering on production contracts..."
echo

for contract_info in "${CONTRACTS[@]}"; do
    IFS=':' read -r address chain name <<< "$contract_info"
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Contract: $name"
    echo "Address: $address"
    echo "Chain: $chain"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo
    
    # Scan contract
    OUTPUT=$(cargo run --release --bin scpf -- scan "$address" --chain "$chain" 2>&1 || true)
    
    # Extract findings count
    FINDINGS=$(echo "$OUTPUT" | grep -oP "Total issues: \K\d+" || echo "0")
    
    echo "$OUTPUT" | tail -30
    echo
    echo "Results for $name:"
    echo "  Total findings: $FINDINGS"
    echo
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Validation Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
