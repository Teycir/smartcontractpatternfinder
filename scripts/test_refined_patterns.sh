#!/bin/bash
# Test refined patterns on top 5 contracts

echo "🧪 Testing Refined Patterns"
echo "=========================="
echo ""

cd /home/teycir/Repos/SmartContractPatternFinder

echo "📊 Before Refinement:"
echo "- Total findings: 1259"
echo "- False positive rate: 100%"
echo ""

echo "🔄 Testing refined patterns on top 5 contracts..."
echo ""

for i in {1..5}; do
    file="validation/top10_sources/${i}_*.sol"
    if [ -f $file ]; then
        echo "[$i/5] Scanning contract $i..."
        cargo run --release -- scan $file --min-severity critical 2>&1 | grep -E "(Findings:|clean|findings)" | head -3
    fi
done

echo ""
echo "✅ Refinement test complete"
echo ""
echo "Expected improvements:"
echo "- sqrt-price-no-bounds: Should not match NFT contracts"
echo "- arbitrary-call-no-check: Should skip functions with onlyOwner"
echo "- delegatecall-no-whitelist: Should check for access control"
echo "- Contract type detection: Skip irrelevant patterns"
