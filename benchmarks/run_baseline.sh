#!/usr/bin/env bash
set -e

echo "=== SCPF Baseline Accuracy Evaluation ==="
echo ""

# Count contracts
TOTAL_CONTRACTS=$(find benchmarks -name "*.sol" 2>/dev/null | wc -l)
if [ ! -f benchmarks/ground-truth.json ]; then
    echo "Error: benchmarks/ground-truth.json not found" >&2
    exit 1
fi
VULNERABLE=$(jq '.contracts | map(select(.vulnerabilities | length > 0)) | length' benchmarks/ground-truth.json 2>/dev/null)
if [ $? -ne 0 ]; then
    echo "Error: Failed to parse ground-truth.json for vulnerable count" >&2
    exit 1
fi
SAFE=$(jq '.contracts | map(select(.vulnerabilities | length == 0)) | length' benchmarks/ground-truth.json 2>/dev/null)
if [ $? -ne 0 ]; then
    echo "Error: Failed to parse ground-truth.json for safe count" >&2
    exit 1
fi
TOTAL_VULNS=$(jq '[.contracts[].vulnerabilities | length] | add' benchmarks/ground-truth.json 2>/dev/null)
if [ $? -ne 0 ]; then
    echo "Error: Failed to parse ground-truth.json for total vulnerabilities" >&2
    exit 1
fi

echo "Benchmark Corpus:"
echo "  Total contracts: $TOTAL_CONTRACTS"
echo "  Vulnerable: $VULNERABLE"
echo "  Safe: $SAFE"
echo "  Total vulnerabilities labeled: $TOTAL_VULNS"
echo ""

# Run scan on benchmarks
echo "Scanning benchmark contracts..."
if ! cd benchmarks; then
    echo "Error: benchmarks directory not found" >&2
    exit 1
fi

if ! cargo run --release --bin scpf -- scan --output json > /tmp/scpf_findings.json 2>&1; then
    echo "Error: SCPF scan failed" >&2
    cd ..
    exit 1
fi
cd ..

# Count findings
if [ ! -f /tmp/scpf_findings.json ]; then
    echo "Error: Scan output file not found" >&2
    exit 1
fi
FINDINGS=$(jq '[.[].matches | length] | add // 0' /tmp/scpf_findings.json 2>/dev/null)
if [ $? -ne 0 ]; then
    echo "Error: Failed to parse scan results" >&2
    exit 1
fi
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
