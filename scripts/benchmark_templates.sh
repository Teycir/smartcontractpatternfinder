#!/bin/bash
# Benchmark test suite for template validation

echo "🧪 SCPF Template Benchmark Suite"
echo "=================================="

SCPF="cargo run --release --bin scpf-cli --"
RESULTS_DIR="benchmark_results"
mkdir -p "$RESULTS_DIR"

# Test against vulnerable contracts
echo ""
echo "1️⃣  Testing Vulnerable Contracts..."
$SCPF scan sol/test_fee_accounting.sol --output json > "$RESULTS_DIR/vulnerable_fee.json"
$SCPF scan sol/test_reward_inflation.sol --output json > "$RESULTS_DIR/vulnerable_reward.json"
$SCPF scan sol/test_price_manipulation.sol --output json > "$RESULTS_DIR/vulnerable_price.json"
$SCPF scan sol/test_precision_loss_v2.sol --output json > "$RESULTS_DIR/vulnerable_precision.json"
$SCPF scan sol/test_access_control.sol --output json > "$RESULTS_DIR/vulnerable_access.json"

# Test against safe implementations (OpenZeppelin, Uniswap patterns)
echo ""
echo "2️⃣  Testing Safe Implementations..."
echo "   (Should have 0 or minimal findings)"

# Calculate metrics
echo ""
echo "📊 Benchmark Results:"
echo "===================="
echo "Vulnerable Contracts:"
grep -c "severity" "$RESULTS_DIR"/vulnerable_*.json | awk -F: '{sum+=$2} END {print "  Total Findings: " sum}'

echo ""
echo "✅ Benchmark complete. Results in $RESULTS_DIR/"
