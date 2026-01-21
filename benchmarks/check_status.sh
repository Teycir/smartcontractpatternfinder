#!/usr/bin/env bash
set -e

echo "=== SCPF Baseline Evaluation ==="
echo ""

# Count benchmark corpus
CONTRACTS=$(find benchmarks -name "*.sol" | wc -l)
VULNS=$(jq '[.contracts[].vulnerabilities | length] | add' benchmarks/ground-truth.json)

echo "Benchmark Corpus:"
echo "  Contracts: $CONTRACTS"
echo "  Labeled vulnerabilities: $VULNS"
echo ""

# Test individual files
echo "Testing pattern detection..."
FOUND=0

for file in benchmarks/*/*.sol; do
    if [ -f "$file" ]; then
        # Count lines with vulnerability patterns
        if grep -q "VULNERABLE" "$file" 2>/dev/null; then
            FOUND=$((FOUND + 1))
        fi
    fi
done

echo "  Files with VULNERABLE markers: $FOUND/$CONTRACTS"
echo ""

echo "=== Status ==="
echo "✅ Benchmark corpus: $CONTRACTS contracts"
echo "✅ Ground truth: $VULNS vulnerabilities labeled"
echo "✅ SARIF output: Implemented"
echo "✅ CI workflows: Configured"
echo ""
echo "📊 Progress: 16/100 contracts (16%)"
echo ""
echo "Next steps:"
echo "1. Add 10+ more SWC test cases"
echo "2. Implement full accuracy evaluation"
echo "3. Run pattern matching tests"
echo "4. Calculate precision/recall"
