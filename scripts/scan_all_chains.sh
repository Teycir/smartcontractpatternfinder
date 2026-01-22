#!/bin/bash
# Scan all chains for recent contracts
# Usage: ./scan_all_chains.sh [days] [severity]

set -e

DAYS=${1:-10}
MIN_SEVERITY=${2:-high}
OUTPUT_DIR="results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="${OUTPUT_DIR}/scan_all_chains_${DAYS}days_${TIMESTAMP}.json"

echo "🔍 SCPF - Multi-Chain Scan"
echo "=========================="
echo "Days: $DAYS | Min Severity: $MIN_SEVERITY"
echo "Chains: ethereum, bsc, polygon, arbitrum, optimism, base"
echo ""

cd "$(dirname "$0")/.." || exit 1

mkdir -p "$OUTPUT_DIR"

echo "📡 Fetching and scanning all chains..."
if ! cargo run --release --bin scpf -- scan \
  --days "$DAYS" \
  --all-chains \
  --min-severity "$MIN_SEVERITY" \
  --output json > "$OUTPUT_FILE"; then
    echo "Error: SCPF scan failed" >&2
    exit 1
fi

echo ""
echo "✅ Scan complete!"
echo "📊 Results: $OUTPUT_FILE"
echo ""

# Risk analysis
echo "📈 Risk Analysis:"
echo "=========================="

if ! python3 << PYEOF
import json
import sys
from collections import defaultdict

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

critical = sum(1 for r in results for m in r.get('matches', []) if m.get('severity') == 'critical')
high = sum(1 for r in results for m in r.get('matches', []) if m.get('severity') == 'high')
medium = sum(1 for r in results for m in r.get('matches', []) if m.get('severity') == 'medium')

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

print(f"Contracts: {total_contracts} scanned, {with_issues} with issues")
print(f"Findings: {total_findings} total")
print(f"  • Critical: {critical}")
print(f"  • High: {high}")
print(f"  • Medium: {medium}")
print()
print(f"Risk Score: {total_risk} ({risk_level})")
print(f"  Formula: {critical}×100 + {high}×10 + {medium}×3 = {total_risk}")
print()

# Per-chain breakdown
chain_stats = defaultdict(lambda: {'contracts': 0, 'findings': 0, 'risk': 0})
for r in results:
    chain = r.get('chain', 'unknown')
    matches = r.get('matches', [])
    chain_stats[chain]['contracts'] += 1
    chain_stats[chain]['findings'] += len(matches)
    chain_stats[chain]['risk'] += sum(100 if m.get('severity') == 'critical' else 10 if m.get('severity') == 'high' else 3 for m in matches)

print("📊 Per-Chain Breakdown:")
for chain, stats in sorted(chain_stats.items(), key=lambda x: x[1]['risk'], reverse=True):
    print(f"  {chain}: {stats['contracts']} contracts, {stats['findings']} findings, Risk: {stats['risk']}")
PYEOF
then
    echo "Error: Risk analysis failed" >&2
    exit 1
fi

echo ""
echo "💾 Full results: $OUTPUT_FILE"
