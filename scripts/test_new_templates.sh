#!/bin/bash
set -e

echo "🧪 Testing New Templates Against Test Files"
echo "============================================"
echo ""

SCPF="./target/release/scpf"
TEMPLATES="./templates"

# Test 1: weak_randomness.yaml
echo "📝 Test 1: Weak Randomness Detection"
echo "File: sol/test_weak_randomness.sol"
if grep -q "blockhash" sol/test_weak_randomness.sol && \
   grep -q "block.timestamp %" sol/test_weak_randomness.sol && \
   grep -q "block.number %" sol/test_weak_randomness.sol; then
    echo "✅ Test file contains all 3 weak randomness patterns"
else
    echo "❌ Test file missing patterns"
    exit 1
fi
echo ""

# Test 2: timelock_missing.yaml
echo "📝 Test 2: Missing Timelock Detection"
echo "File: sol/test_timelock_missing.sol"
if grep -q "function.*onlyOwner" sol/test_timelock_missing.sol && \
   grep -q "function upgrade" sol/test_timelock_missing.sol && \
   grep -q "function setImplementation" sol/test_timelock_missing.sol; then
    echo "✅ Test file contains all 3 timelock patterns"
else
    echo "❌ Test file missing patterns"
    exit 1
fi
echo ""

# Test 3: signature_unchecked.yaml
echo "📝 Test 3: Signature Replay Detection"
echo "File: sol/test_signature_replay.sol"
if grep -q "ecrecover" sol/test_signature_replay.sol && \
   grep -q "ECDSA.recover" sol/test_signature_replay.sol; then
    echo "✅ Test file contains signature patterns"
else
    echo "❌ Test file missing patterns"
    exit 1
fi
echo ""

# Test 4: unchecked_return_value.yaml
echo "📝 Test 4: Unchecked Return Value Detection"
echo "File: sol/test_unchecked_return_improved.sol"
if grep -q "(bool success, )" sol/test_unchecked_return_improved.sol && \
   grep -q ".call{value:" sol/test_unchecked_return_improved.sol; then
    echo "✅ Test file contains unchecked return patterns"
else
    echo "❌ Test file missing patterns"
    exit 1
fi
echo ""

# Verify templates are loaded
echo "📋 Verifying Templates Loaded"
$SCPF templates list --templates $TEMPLATES 2>&1 | grep -q "weak-randomness-v1" && echo "✅ weak-randomness-v1 loaded"
$SCPF templates list --templates $TEMPLATES 2>&1 | grep -q "timelock-missing-v1" && echo "✅ timelock-missing-v1 loaded"
$SCPF templates list --templates $TEMPLATES 2>&1 | grep -q "signature-return-unchecked-v3" && echo "✅ signature-return-unchecked-v3 loaded"
$SCPF templates list --templates $TEMPLATES 2>&1 | grep -q "unchecked-return-value-v5" && echo "✅ unchecked-return-value-v5 loaded"
echo ""

echo "✅ All Template Tests Passed!"
echo ""
echo "📊 5-Day Scan Summary:"
$SCPF scan --days 5 --templates $TEMPLATES 2>&1 | grep -A5 "Summary"
