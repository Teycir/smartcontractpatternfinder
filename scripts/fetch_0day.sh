#!/bin/bash
# Fetch 0-day exploits and generate research report
# Usage: ./fetch_0day.sh [days]

set -e

DAYS=${1:-7}

echo "🔍 SCPF - 0-Day Exploit Fetcher"
echo "================================"
echo "Days: $DAYS"
echo ""

cd "$(dirname "$0")/.." || exit 1

cargo run --release --bin scpf -- fetch-zero-day --days "$DAYS"
