#!/bin/bash
# Scan local Solidity project
# Usage: ./scan_local.sh [severity]

set -e

MIN_SEVERITY=${1:-high}
OUTPUT_DIR="results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="${OUTPUT_DIR}/scan_local_${TIMESTAMP}.json"

echo "🔍 SCPF - Local Project Scan"
echo "============================="
echo "Min Severity: $MIN_SEVERITY"
echo ""

cd "$(dirname "$0")/.." || exit 1

mkdir -p "$OUTPUT_DIR"

echo "📂 Scanning local .sol files..."
cargo run --release --bin scpf -- scan \
  --min-severity "$MIN_SEVERITY" \
  --output json > "$OUTPUT_FILE"

echo ""
echo "✅ Scan complete!"
echo "📊 Results: $OUTPUT_FILE"
echo ""

# Risk analysis
echo "📈 Risk Analysis:"
echo "============================="

python3 << PYEOF
import json

with open('$OUTPUT_FILE') as f:
    results = json.load(f)

total_files = len(results)
with_issues = sum(1 for r in results if r['matches'])
total_findings = sum(len(r['matches']) for r in results)

critical = sum(1 for r in results for m in r['matches'] if m['severity'] == 'critical')
high = sum(1 for r in results for m in r['matches'] if m['severity'] == 'high')
medium = sum(1 for r in results for m in r['matches'] if m['severity'] == 'medium')

total_risk = (critical * 100) + (high * 10) + (medium * 3)

if total_risk == 0:
    risk_level = "None ✅"
elif total_risk <= 100:
    risk_level = "Low ✅"
elif total_risk <= 500:
    risk_level = "Medium ⚠️"
elif total_risk <= 2000:
    risk_level = "High 🔴"
else:
    risk_level = "Critical 🚨"

print(f"Files: {total_files} scanned, {with_issues} with issues")
print(f"Findings: {total_findings} total")
print(f"  • Critical: {critical}")
print(f"  • High: {high}")
print(f"  • Medium: {medium}")
print()
print(f"Risk Score: {total_risk} ({risk_level})")
print(f"  Formula: {critical}×100 + {high}×10 + {medium}×3 = {total_risk}")
print()

if total_risk > 0:
    print("🔴 Top Risky Files:")
    sorted_results = sorted(results, key=lambda r: sum(100 if m['severity'] == 'critical' else 10 if m['severity'] == 'high' else 3 for m in r['matches']), reverse=True)
    for i, r in enumerate(sorted_results[:5], 1):
        if r['matches']:
            risk = sum(100 if m['severity'] == 'critical' else 10 if m['severity'] == 'high' else 3 for m in r['matches'])
            print(f"  {i}. {r['address']} - Risk: {risk}")
PYEOF

echo ""
echo "💾 Full results: $OUTPUT_FILE"
