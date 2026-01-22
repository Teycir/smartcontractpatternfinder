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
if ! cargo run --release --bin scpf -- scan \
  --days "$DAYS" \
  --chain "$CHAIN" \
  --min-severity "$MIN_SEVERITY" \
  --output json > "$OUTPUT_FILE" 2>&1; then
    echo "Error: SCPF scan failed" >&2
    exit 1
fi

echo ""
echo "✅ Scan complete!"
echo "📊 Results: $OUTPUT_FILE"
echo ""

# Calculate risk scores and generate report
echo "📈 Risk Analysis:"
echo "================================"

if ! python3 << PYEOF
import json
import sys

try:
    with open('$OUTPUT_FILE') as f:
        results = json.load(f)
except FileNotFoundError:
    print(f"Error: File '$OUTPUT_FILE' not found", file=sys.stderr)
    sys.exit(1)
except json.JSONDecodeError as e:
    print(f"Error: Invalid JSON in '$OUTPUT_FILE': {e}", file=sys.stderr)
    sys.exit(1)
except Exception as e:
    print(f"Error: Failed to read results file: {e}", file=sys.stderr)
    sys.exit(1)

total_contracts = len(results)
with_issues = sum(1 for r in results if r.get('matches', []))
total_findings = sum(len(r.get('matches', [])) for r in results)

# Severity breakdown
critical = sum(1 for r in results for m in r.get('matches', []) if m.get('severity') == 'critical')
high = sum(1 for r in results for m in r.get('matches', []) if m.get('severity') == 'high')
medium = sum(1 for r in results for m in r.get('matches', []) if m.get('severity') == 'medium')

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
    sorted_results = sorted(results, key=lambda r: sum(m.get('severity') == 'critical' for m in r.get('matches', [])) * 100 + sum(m.get('severity') == 'high' for m in r.get('matches', [])) * 10, reverse=True)
    for i, r in enumerate(sorted_results[:5], 1):
        addr = r.get('address', 'unknown')[:10] + '...'
        matches = r.get('matches', [])
        risk = sum(100 if m.get('severity') == 'critical' else 10 if m.get('severity') == 'high' else 3 for m in matches)
        count = len(matches)
        print(f"  {i}. {addr} - Risk: {risk}, Findings: {count}")
PYEOF
then
    echo "Error: Risk analysis failed" >&2
    exit 1
fi

echo ""
echo "💾 Full results: $OUTPUT_FILE"
