#!/bin/bash
# Extract top 20 most vulnerable contracts by risk score
# Usage: ./extract_top20.sh <results_file.json>

set -e

RESULTS_FILE=${1:-$(ls -t results/scan_*.json 2>/dev/null | head -1)}
OUTPUT_FILE="${RESULTS_FILE%.json}_top20.json"

if [ -z "$RESULTS_FILE" ] || [ ! -f "$RESULTS_FILE" ]; then
    echo "❌ No results file found"
    echo "Usage: ./extract_top20.sh <results_file.json>"
    exit 1
fi

echo "🎯 Extracting Top 20 Most Vulnerable Contracts"
echo "=============================================="
echo "Input: $RESULTS_FILE"
echo ""

python3 << PYEOF
import json

with open('$RESULTS_FILE') as f:
    results = json.load(f)

# Calculate risk score for each contract
scored = []
for contract in results:
    if not contract['matches']:
        continue
    
    critical = sum(1 for m in contract['matches'] if m['severity'] == 'critical')
    high = sum(1 for m in contract['matches'] if m['severity'] == 'high')
    medium = sum(1 for m in contract['matches'] if m['severity'] == 'medium')
    
    risk_score = (critical * 100) + (high * 10) + (medium * 3)
    
    scored.append({
        'contract': contract,
        'risk_score': risk_score,
        'critical': critical,
        'high': high,
        'medium': medium
    })

# Sort by risk score descending
scored.sort(key=lambda x: x['risk_score'], reverse=True)

# Take top 20
top20 = [s['contract'] for s in scored[:20]]

# Save to file
with open('$OUTPUT_FILE', 'w') as f:
    json.dump(top20, f, indent=2)

# Print summary
print(f"✅ Extracted top 20 contracts")
print(f"📊 Output: $OUTPUT_FILE")
print()
print("🎯 TOP 20 BY RISK SCORE:")
print("-" * 70)
for i, s in enumerate(scored[:20], 1):
    addr = s['contract']['address']
    print(f"{i:2d}. {addr}")
    print(f"    Risk: {s['risk_score']:,} | C:{s['critical']} H:{s['high']} M:{s['medium']} | Total: {len(s['contract']['matches'])}")

PYEOF

echo ""
echo "💾 Top 20 saved to: $OUTPUT_FILE"
