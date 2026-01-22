#!/bin/bash
# Rank contracts using BOTH metrics: Risk Score + PoC Exploitability
# Uses SCPF's built-in ranking system with proper report storage

set -e

DAYS=${1:-7}
CHAIN=${2:-ethereum}
MIN_SEVERITY=${3:-high}
OUTPUT_DIR="results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="${OUTPUT_DIR}/scan_${CHAIN}_${DAYS}days_${TIMESTAMP}.json"

echo "🎯 SCPF Dual-Metric Ranking System"
echo "=================================="
echo "Metric 1: Risk Score (CRITICAL×100 + HIGH×10 + MEDIUM×3)"
echo "Metric 2: PoC Exploitability (Pattern-based success probability)"
echo ""
echo "Scanning: $DAYS days | Chain: $CHAIN | Min Severity: $MIN_SEVERITY"
echo ""

mkdir -p "$OUTPUT_DIR"

# Run scan with exploitability sorting AND save JSON
if ! cargo run --release --bin scpf -- scan \
  --days "$DAYS" \
  --chain "$CHAIN" \
  --min-severity "$MIN_SEVERITY" \
  --sort-by-exploitability \
  --output json > "$OUTPUT_FILE"; then
    echo "Error: SCPF scan failed" >&2
    exit 1
fi

echo ""
echo "✅ Scan complete!"
echo "📊 Results: $OUTPUT_FILE"
echo ""
echo "💡 The scan used BOTH ranking metrics:"
echo "   1. PoC Exploitability - Pattern-based success probability"
echo "   2. Risk Score - CRITICAL×100 + HIGH×10 + MEDIUM×3"
echo ""
echo "📋 To view ranked results:"
echo "   • Console: cargo run --release --bin scpf -- scan \\"
echo "              --days $DAYS --chain $CHAIN --sort-by-exploitability"
echo "   • JSON: cat $OUTPUT_FILE | jq"
echo "   • SARIF: cargo run --release --bin scpf -- scan \\"
echo "            --days $DAYS --chain $CHAIN --output sarif"
