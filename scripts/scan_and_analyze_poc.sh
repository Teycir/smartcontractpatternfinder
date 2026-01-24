#!/bin/bash
# Clean scan with PoC analysis
# Usage: ./scan_and_analyze_poc.sh [days] [chain] [severity]

set -e

DAYS=${1:-7}
CHAIN=${2:-ethereum}
MIN_SEVERITY=${3:-high}
OUTPUT_DIR="$HOME/scpf-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="${OUTPUT_DIR}/scan_${CHAIN}_${DAYS}days_${TIMESTAMP}_scan_and_analyze_poc.json"

echo "🔍 SCPF - Scan & PoC Analysis"
echo "================================"
echo "Days: $DAYS | Chain: $CHAIN | Min Severity: $MIN_SEVERITY"
echo ""

cd "$(dirname "$0")/.." || exit 1
mkdir -p "$OUTPUT_DIR"

echo "📡 Scanning contracts..."

# Run scan and extract only JSON (skip console output lines)
cargo run --release --bin scpf -- scan \
  --days "$DAYS" \
  --chain "$CHAIN" \
  --min-severity "$MIN_SEVERITY" \
  --output json 2>&1 | \
  sed -n '/^\[/,/^\]/p' > "$OUTPUT_FILE"

# Check if JSON is valid
if ! python3 -c "import json; json.load(open('$OUTPUT_FILE'))" 2>/dev/null; then
    echo "❌ Failed to generate valid JSON"
    exit 1
fi

echo "✅ Scan complete!"
echo "📊 Results: $OUTPUT_FILE"
echo ""

# Analyze PoC-ability
if ! ./scripts/analyze_poc_ability.sh "$OUTPUT_FILE"; then
    echo "Error: PoC analysis failed" >&2
    exit 1
fi
