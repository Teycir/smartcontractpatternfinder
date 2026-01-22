#!/bin/bash
# Scan recently updated contracts from blockchain
# Usage: ./scan_recent.sh [days] [chain] [severity]

set -e

DAYS=${1:-10}
CHAIN=${2:-ethereum}
MIN_SEVERITY=${3:-high}
OUTPUT_DIR="results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="${OUTPUT_DIR}/scan_${CHAIN}_${DAYS}days_${TIMESTAMP}.json"

echo "🔍 SCPF - Recent Contracts Scan"
echo "================================"
echo "Days: $DAYS | Chain: $CHAIN | Min Severity: $MIN_SEVERITY"
echo ""

cd "$(dirname "$0")/.." || exit 1

mkdir -p "$OUTPUT_DIR"

echo "📡 Fetching and scanning contracts..."
cargo run --release --bin scpf -- scan \
  --days "$DAYS" \
  --chain "$CHAIN" \
  --min-severity "$MIN_SEVERITY" \
  --output json 2>/dev/null > "$OUTPUT_FILE"

echo ""
echo "✅ Scan complete!"
echo "📊 Results: $OUTPUT_FILE"
echo ""

# Calculate risk scores and generate report
echo "📈 Risk Analysis:"
echo "================================"

python3 << PYEOF
import json
import sys

with open('$OUTPUT_FILE') as f:
    results = json.load(f)

total_contracts = len(results)
with_issues = sum(1 for r in results if r['matches'])
total_findings = sum(len(r['matches']) for r in results)

# Severity breakdown
critical = sum(1 for r in results for m in r['matches'] if m['severity'] == 'critical')
high = sum(1 for r in results for m in r['matches'] if m['severity'] == 'high')
medium = sum(1 for r in results for m in r['matches'] if m['severity'] == 'medium')

# Risk scoring (CRITICAL×100 + HIGH×10 + MEDIUM×3)
total_risk = (critical * 100) + (high * 10) + (medium * 3)
avg_risk = total_risk // total_contracts if total_contracts > 0 else 0

# Risk level
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

print(f"Contracts: {total_contracts} scanned, {with_issues} with issues")
print(f"Findings: {total_findings} total")
print(f"  • Critical: {critical}")
print(f"  • High: {high}")
print(f"  • Medium: {medium}")
print()
print(f"Risk Score: {total_risk} ({risk_level})")
print(f"  Formula: {critical}×100 + {high}×10 + {medium}×3 = {total_risk}")
print(f"  Average: {avg_risk} per contract")
print()
print("Thresholds: 0-100=Low | 101-500=Medium | 501-2000=High | 2000+=Critical")

# Top risky contracts
if results:
    print()
    print("🔴 Top 5 Risky Contracts:")
    sorted_results = sorted(results, key=lambda r: sum(m.get('severity') == 'critical' for m in r['matches']) * 100 + sum(m.get('severity') == 'high' for m in r['matches']) * 10, reverse=True)
    for i, r in enumerate(sorted_results[:5], 1):
        addr = r['address'][:10] + '...'
        risk = sum(100 if m['severity'] == 'critical' else 10 if m['severity'] == 'high' else 3 for m in r['matches'])
        count = len(r['matches'])
        print(f"  {i}. {addr} - Risk: {risk}, Findings: {count}")
PYEOF

echo ""
echo "💾 Full results: $OUTPUT_FILE"
