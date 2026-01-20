#!/bin/bash
# Full audit - Last 7 days contracts - All chains - Update templates (0 days)

TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Cross-platform report directory
if [ -n "$USERPROFILE" ]; then
    REPORT_DIR="$USERPROFILE/smartcontractpatternfinderReports"
else
    REPORT_DIR="$HOME/smartcontractpatternfinderReports"
fi

mkdir -p "$REPORT_DIR"

echo "🔍 Full Audit: Last 7 Days Contracts"
echo "   Chains: All (ethereum, bsc, polygon, arbitrum, optimism, base)"
echo "   Template Update: 0 days (latest exploits)"
echo "   Reports: $REPORT_DIR/"
echo ""

# Console output
export RUST_LOG=error
cargo run --release --bin scpf -- scan \
  --days 7 \
  --all-chains \
  --min-severity high \
  --update-templates 0 \
  --output console \
  2>/dev/null | tee "$REPORT_DIR/audit_7days_${TIMESTAMP}_console.txt"

# JSON report
cargo run --release --bin scpf -- scan \
  --days 7 \
  --all-chains \
  --min-severity high \
  --update-templates 0 \
  --output json \
  2>/dev/null > "$REPORT_DIR/audit_7days_${TIMESTAMP}.json"

# SARIF report
cargo run --release --bin scpf -- scan \
  --days 7 \
  --all-chains \
  --min-severity high \
  --update-templates 0 \
  --output sarif \
  2>/dev/null > "$REPORT_DIR/audit_7days_${TIMESTAMP}.sarif"

echo ""
echo "✅ Audit Complete"
echo "   📄 Console: $REPORT_DIR/audit_7days_${TIMESTAMP}_console.txt"
echo "   📊 JSON:    $REPORT_DIR/audit_7days_${TIMESTAMP}.json"
echo "   🔒 SARIF:   $REPORT_DIR/audit_7days_${TIMESTAMP}.sarif"
