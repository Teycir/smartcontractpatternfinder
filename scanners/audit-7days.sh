#!/bin/bash
# Full audit - Last 7 days contracts - All chains - Update templates (0 days)

TIMESTAMP=$(date +%Y%m%d_%H%M%S)

if [ -n "$USERPROFILE" ]; then
    REPORT_DIR="$USERPROFILE/smartcontractpatternfinderReports"
else
    REPORT_DIR="$HOME/smartcontractpatternfinderReports"
fi

mkdir -p "$REPORT_DIR"

echo "🔍 Full Audit: Last 7 Days Contracts"
echo "   Chains: All"
echo "   Template Update: 0 days"
echo ""

export RUST_LOG=error
cargo run --release --bin scpf -- scan --days 7 --all-chains --min-severity high --update-templates 0 --output console 2>/dev/null | tee "$REPORT_DIR/audit_7days_${TIMESTAMP}_console.txt"

cargo run --release --bin scpf -- scan --days 7 --all-chains --min-severity high --update-templates 0 --output json 2>/dev/null > "$REPORT_DIR/audit_7days_${TIMESTAMP}.json"

cargo run --release --bin scpf -- scan --days 7 --all-chains --min-severity high --update-templates 0 --output sarif 2>/dev/null > "$REPORT_DIR/audit_7days_${TIMESTAMP}.sarif"

echo ""
echo "✅ Reports: $REPORT_DIR/audit_7days_${TIMESTAMP}.*"
