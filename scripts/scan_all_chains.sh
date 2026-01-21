#!/bin/bash
# Scan all chains for recent contracts
# Usage: ./scan_all_chains.sh [days] [severity]

set -e

DAYS=${1:-7}
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
cargo run --release --bin scpf -- scan \
  --days "$DAYS" \
  --all-chains \
  --min-severity "$MIN_SEVERITY" \
  --output json > "$OUTPUT_FILE"

echo ""
echo "✅ Scan complete!"
echo "📊 Results: $OUTPUT_FILE"
echo ""

# Risk analysis
echo "📈 Risk Analysis:"
echo "=========================="

python3 << PYEOF
import json
from collections import defaultdict

with open('$OUTPUT_FILE') as f:
    results = json.load(f)

total_contracts = len(results)
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
    chain = r['chain']
    chain_stats[chain]['contracts'] += 1
    chain_stats[chain]['findings'] += len(r['matches'])
    chain_stats[chain]['risk'] += sum(100 if m['severity'] == 'critical' else 10 if m['severity'] == 'high' else 3 for m in r['matches'])

print("📊 Per-Chain Breakdown:")
for chain, stats in sorted(chain_stats.items(), key=lambda x: x[1]['risk'], reverse=True):
    print(f"  {chain}: {stats['contracts']} contracts, {stats['findings']} findings, Risk: {stats['risk']}")
PYEOF

echo ""
echo "💾 Full results: $OUTPUT_FILE"
