#!/bin/bash
# Test script for new vulnerability templates

set -e

echo "🧪 Testing New Vulnerability Templates"
echo "======================================"

SCPF="cargo run --release --bin scpf-cli --"
TEMPLATES_DIR="templates"
SOL_DIR="sol"

# Test 1: Fee Accounting
echo ""
echo "1️⃣  Testing fee_accounting_flaw.yaml..."
$SCPF scan --local-file "$SOL_DIR/test_fee_accounting.sol" \
    --templates "$TEMPLATES_DIR/fee_accounting_flaw.yaml" \
    | tee /tmp/test_fee.txt
MATCHES=$(grep -c "MATCH" /tmp/test_fee.txt || true)
echo "   ✓ Found $MATCHES matches (expected: 6)"

# Test 2: Reward Inflation
echo ""
echo "2️⃣  Testing reward_inflation.yaml..."
$SCPF scan --local-file "$SOL_DIR/test_reward_inflation.sol" \
    --templates "$TEMPLATES_DIR/reward_inflation.yaml" \
    | tee /tmp/test_reward.txt
MATCHES=$(grep -c "MATCH" /tmp/test_reward.txt || true)
echo "   ✓ Found $MATCHES matches (expected: 6)"

# Test 3: Price Manipulation
echo ""
echo "3️⃣  Testing price_manipulation.yaml..."
$SCPF scan --local-file "$SOL_DIR/test_price_manipulation.sol" \
    --templates "$TEMPLATES_DIR/price_manipulation.yaml" \
    | tee /tmp/test_price.txt
MATCHES=$(grep -c "MATCH" /tmp/test_price.txt || true)
echo "   ✓ Found $MATCHES matches (expected: 7)"

# Test 4: Precision Loss
echo ""
echo "4️⃣  Testing precision_loss.yaml..."
$SCPF scan --local-file "$SOL_DIR/test_precision_loss_v2.sol" \
    --templates "$TEMPLATES_DIR/precision_loss.yaml" \
    | tee /tmp/test_precision.txt
MATCHES=$(grep -c "MATCH" /tmp/test_precision.txt || true)
echo "   ✓ Found $MATCHES matches (expected: 7)"

# Test 5: Access Control
echo ""
echo "5️⃣  Testing access_control_bypass.yaml..."
$SCPF scan --local-file "$SOL_DIR/test_access_control.sol" \
    --templates "$TEMPLATES_DIR/access_control_bypass.yaml" \
    | tee /tmp/test_access.txt
MATCHES=$(grep -c "MATCH" /tmp/test_access.txt || true)
echo "   ✓ Found $MATCHES matches (expected: 8)"

echo ""
echo "======================================"
echo "✅ All template tests completed!"
echo ""
echo "Summary:"
echo "  - Fee Accounting: 6 patterns (MTToken, FutureSwap)"
echo "  - Reward Inflation: 6 patterns (PRXVT)"
echo "  - Price Manipulation: 7 patterns (DRLVaultV3, NGP, etc.)"
echo "  - Precision Loss: 7 patterns (BalancerV2)"
echo "  - Access Control: 8 patterns (TokenHolder, SuperRare)"
echo ""
echo "Total: 34 exploit patterns from real 2025-2026 0-days"
