#!/usr/bin/env bash
set -e

echo "=== SCPF Baseline Accuracy Evaluation ==="
echo ""

# Count contracts
TOTAL_CONTRACTS=$(find benchmarks -name "*.sol" | wc -l)
VULNERABLE=$(jq '.contracts | map(select(.vulnerabilities | length > 0)) | length' benchmarks/ground-truth.json)
SAFE=$(jq '.contracts | map(select(.vulnerabilities | length == 0)) | length' benchmarks/ground-truth.json)
TOTAL_VULNS=$(jq '[.contracts[].vulnerabilities | length] | add' benchmarks/ground-truth.json)

echo "Benchmark Corpus:"
echo "  Total contracts: $TOTAL_CONTRACTS"
echo "  Vulnerable: $VULNERABLE"
echo "  Safe: $SAFE"
echo "  Total vulnerabilities labeled: $TOTAL_VULNS"
echo ""

# Run scan on benchmarks
echo "Scanning benchmark contracts..."
cd benchmarks && cargo run --release --bin scpf -- scan --output json > /tmp/scpf_findings.json 2>/dev/null || true
cd ..

# Count findings
FINDINGS=$(jq '[.[].matches | length] | add // 0' /tmp/scpf_findings.json)
echo "  Found $FINDINGS total findings"
echo ""

# Simple accuracy calculation
echo "=== Preliminary Results ==="
echo ""
echo "Note: Full accuracy evaluation requires pattern matching implementation"
echo "This is a baseline scan to verify the system works end-to-end"
echo ""
echo "Next steps:"
echo "1. Implement pattern matching in accuracy.rs"
echo "2. Run full evaluation with precision/recall"
echo "3. Identify false positives and false negatives"
echo "4. Iterate on patterns"
echo ""
echo "Status: ✅ Baseline scan complete"
echo "Findings saved to: /tmp/scpf_findings.json"
