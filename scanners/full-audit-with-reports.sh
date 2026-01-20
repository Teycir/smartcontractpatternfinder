#!/bin/bash
# Full audit with saved reports - All severities - Last 90 days - All chains - Update templates (7 days)

TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Cross-platform report directory
if [ -n "$USERPROFILE" ]; then
    # Windows
    REPORT_DIR="$USERPROFILE/smartcontractpatternfinderReports"
else
    # Unix/Linux/macOS
    REPORT_DIR="$HOME/smartcontractpatternfinderReports"
fi

mkdir -p "$REPORT_DIR"

echo "🔍 Running full audit with report generation..."
echo "Reports will be saved to: $REPORT_DIR/"
echo ""

# Console output (suppress warnings)
export RUST_LOG=error
cargo run --release -- scan --days 90 --all-chains --min-severity info --update-templates 7 --output console 2>/dev/null | tee "$REPORT_DIR/audit_${TIMESTAMP}_console.txt"

# JSON report
cargo run --release -- scan --days 90 --all-chains --min-severity info --update-templates 0 --output json 2>/dev/null > "$REPORT_DIR/audit_${TIMESTAMP}.json"

# SARIF report (for CI/CD)
cargo run --release -- scan --days 90 --all-chains --min-severity info --update-templates 0 --output sarif 2>/dev/null > "$REPORT_DIR/audit_${TIMESTAMP}.sarif"

echo ""
echo "✅ Reports generated:"
echo "   📄 Console: $REPORT_DIR/audit_${TIMESTAMP}_console.txt"
echo "   📊 JSON:    $REPORT_DIR/audit_${TIMESTAMP}.json"
echo "   🔒 SARIF:   $REPORT_DIR/audit_${TIMESTAMP}.sarif"
