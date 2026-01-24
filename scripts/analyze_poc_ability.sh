#!/bin/bash
# Analyze PoC-ability of scan results
# Usage: ./analyze_poc_ability.sh <results_file.json>

set -e

RESULTS_FILE=${1:-$(find "$HOME/scpf-reports" -name "scan_*.json" -type f -printf '%T@ %p\n' 2>/dev/null | sort -rn | head -1 | cut -d' ' -f2-)}

if [ -z "$RESULTS_FILE" ] || [ ! -f "$RESULTS_FILE" ]; then
    echo "❌ No results file found"
    echo "Usage: ./analyze_poc_ability.sh <results_file.json>"
    exit 1
fi

echo "🎯 PoC-ABILITY ANALYSIS"
echo "================================"
echo "Analyzing: $RESULTS_FILE"
echo ""

python3 << PYEOF
import json
import sys

try:
    with open('$RESULTS_FILE') as f:
        results = json.load(f)
except FileNotFoundError:
    print(f"Error: File '$RESULTS_FILE' not found", file=sys.stderr)
    sys.exit(1)
except json.JSONDecodeError as e:
    print(f"Error: Invalid JSON in '$RESULTS_FILE': {e}", file=sys.stderr)
    sys.exit(1)
except Exception as e:
    print(f"Error: Failed to read results file: {e}", file=sys.stderr)
    sys.exit(1)

total = len(results)
with_findings = [r for r in results if r.get('matches', [])]

print(f"📊 SUMMARY:")
print(f"  Total Contracts: {total}")
print(f"  With Findings: {len(with_findings)}")
print()

# Analyze by exploitability
trivial_contracts = set()
easy_contracts = set()
medium_contracts = set()

trivial_findings = 0
easy_findings = 0
medium_findings = 0

for contract in with_findings:
    addr = contract.get('address', 'unknown')
    
    for match in contract.get('matches', []):
        pattern = match.get('pattern_id', '')
        
        # TRIVIAL: 95-100% PoC success
        if any(p in pattern for p in ['unprotected-selfdestruct', 'missing-access-control', 'reentrancy-pattern']):
            trivial_contracts.add(addr)
            trivial_findings += 1
        # EASY: 85-90% PoC success
        elif any(p in pattern for p in ['delegatecall-user-input', 'tx-origin', 'unchecked-call']):
            if addr not in trivial_contracts:
                easy_contracts.add(addr)
            easy_findings += 1
        # MEDIUM: 50-70% PoC success
        else:
            if addr not in trivial_contracts and addr not in easy_contracts:
                medium_contracts.add(addr)
            medium_findings += 1

print(f"🎯 PoC-ABILITY BY CONTRACT:")
print(f"  🟢 TRIVIAL (95-100% success): {len(trivial_contracts)} contracts")
print(f"  🟡 EASY (85-90% success): {len(easy_contracts)} contracts")
print(f"  🟠 MEDIUM (50-70% success): {len(medium_contracts)} contracts")
print()

print(f"🎯 PoC-ABILITY BY FINDING:")
print(f"  🟢 TRIVIAL: {trivial_findings} findings")
print(f"  🟡 EASY: {easy_findings} findings")
print(f"  🟠 MEDIUM: {medium_findings} findings")
print()

# Top 10 most PoC-able contracts
pocable = []
for contract in with_findings:
    addr = contract.get('address', 'unknown')
    matches = contract.get('matches', [])
    
    trivial_count = sum(1 for m in matches 
                       if any(p in m.get('pattern_id', '') for p in ['unprotected-selfdestruct', 'missing-access-control', 'reentrancy-pattern']))
    easy_count = sum(1 for m in matches
                    if any(p in m.get('pattern_id', '') for p in ['delegatecall-user-input', 'tx-origin', 'unchecked-call']))
    
    poc_score = (trivial_count * 3.0) + (easy_count * 2.0)
    
    if poc_score > 0:
        pocable.append({
            'address': addr,
            'poc_score': poc_score,
            'trivial': trivial_count,
            'easy': easy_count,
            'total': len(matches)
        })

pocable.sort(key=lambda x: x['poc_score'], reverse=True)

if pocable:
    print(f"🎯 TOP 10 MOST PoC-ABLE CONTRACTS:")
    print("-" * 70)
    for i, c in enumerate(pocable[:10], 1):
        print(f"{i:2d}. {c['address']}")
        print(f"    PoC Score: {c['poc_score']:.1f} | Trivial: {c['trivial']} | Easy: {c['easy']} | Total: {c['total']}")
    print()

print("=" * 70)
print(f"✅ RECOMMENDATION: {len(trivial_contracts)} contracts ready for immediate PoC generation")
print("=" * 70)

PYEOF
