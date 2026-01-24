#!/bin/bash
# Extract top N most vulnerable contracts from cache

set -e

N=${1:-10}
REPORT_DIR=${SCPF_REPORT_DIR:-"/home/teycir/smartcontractpatternfinderReports"}
CACHE_DIR="$HOME/.cache/scpf"
VULN_SUMMARY="$REPORT_DIR/vuln_summary.md"

if [ ! -f "$VULN_SUMMARY" ]; then
    echo "❌ Error: vuln_summary.md not found at $VULN_SUMMARY"
    exit 1
fi

if [ ! -d "$CACHE_DIR" ]; then
    echo "❌ Error: Cache directory not found at $CACHE_DIR"
    exit 1
fi

echo "🔍 Extracting top $N vulnerable contracts..."

# Extract contract addresses from vuln_summary.md (skip header lines)
addresses=$(grep -oP '0x[a-fA-F0-9]{40}' "$VULN_SUMMARY" | head -n "$N")

count=0
for addr in $addresses; do
    count=$((count + 1))
    
    # Find contract in cache
    cached_file=$(find "$CACHE_DIR" -type f -name "*${addr}*" 2>/dev/null | head -n 1)
    
    if [ -n "$cached_file" ]; then
        output_file="$REPORT_DIR/contract_${addr}.sol"
        cp "$cached_file" "$output_file"
        
        size=$(du -h "$output_file" | cut -f1)
        lines=$(wc -l < "$output_file")
        
        echo "✅ [$count/$N] $addr ($size, $lines lines)"
    else
        echo "⚠️  [$count/$N] $addr (not in cache)"
    fi
done

echo ""
echo "✅ Extraction complete: $count contracts saved to $REPORT_DIR"
