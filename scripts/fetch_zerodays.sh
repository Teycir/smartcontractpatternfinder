#!/bin/bash
set -e

# Fetch latest 0-day patterns and store in timestamped folder

TIMESTAMP=$(date +%s)
ZERODAY_DIR="templates-zeroday/${TIMESTAMP}"

echo "🔍 Fetching 0-day patterns from last 7 days..."
echo "📁 Output directory: ${ZERODAY_DIR}"
echo ""

# Create timestamped directory
mkdir -p "${ZERODAY_DIR}"

# Fetch 0-day patterns (7 days, all sources: GitHub + RSS)
cargo run --release --bin scpf -- fetch-zero-day --days 7 --output "${ZERODAY_DIR}/zeroday.yaml"

if [ -f "${ZERODAY_DIR}/zeroday.yaml" ]; then
    echo ""
    echo "✅ 0-day patterns saved to: ${ZERODAY_DIR}/zeroday.yaml"
    
    # Count patterns
    PATTERN_COUNT=$(grep -c '^  - id:' "${ZERODAY_DIR}/zeroday.yaml" 2>/dev/null || echo 0)
    echo "📊 Pattern count: ${PATTERN_COUNT}"
    
    # Create symlink to latest
    rm -f templates-zeroday/latest
    ln -s "${TIMESTAMP}" templates-zeroday/latest
    echo "🔗 Symlink: templates-zeroday/latest -> ${TIMESTAMP}"
    
    # List all 0-day template versions
    echo ""
    echo "📚 Available 0-day template versions:"
    ls -lt templates-zeroday/ | grep '^d' | head -5 | awk '{print "   " $9 " (" $6 " " $7 " " $8 ")"}'
else
    echo "⚠️  No 0-day patterns found or generation failed"
    rmdir "${ZERODAY_DIR}" 2>/dev/null || true
    exit 1
fi
