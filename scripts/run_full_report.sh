#!/bin/bash
set -e

DAYS=${1:-10}
TIMESTAMP=$(date +%s)
REPORT_DIR="/home/teycir/smartcontractpatternfinderReports/report_${TIMESTAMP}"
LOG_FILE="$REPORT_DIR/run.log"

mkdir -p "$REPORT_DIR"

echo "╭────────────────────────────────────────────────────────────╮"
echo "│  SCPF - Smart Contract Pattern Finder                     │"
echo "╰────────────────────────────────────────────────────────────╯"
echo "📅 Period: Last $DAYS days"
echo "📂 Report: $REPORT_DIR"
echo ""

cd "$(dirname "$0")/.." || exit 1

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 STEP 1/2: Fetching 0-Day Vulnerability News"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

export SCPF_REPORT_DIR="$REPORT_DIR"
export SCPF_TIMESTAMP="$TIMESTAMP"
cargo run --release --bin scpf -- fetch-zero-day --days "$DAYS" 2>&1 | tee -a "$LOG_FILE"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 STEP 2/2: Scanning Contracts for Exploitable Vulnerabilities"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cargo run --release --bin scpf -- scan --days "$DAYS" --chain ethereum --min-severity high 2>&1 | tee -a "$LOG_FILE"

echo ""
echo "╭────────────────────────────────────────────────────────────╮"
echo "│  ✅ FULL REPORT COMPLETE                                   │"
echo "╰────────────────────────────────────────────────────────────╯"
echo ""
echo "📂 Report Directory: $REPORT_DIR"
echo "   • 0-Day Summary: 0day_summary.md"
echo "   • Vulnerability Summary: vuln_summary.md"
echo "   • Execution Log: run.log"
echo ""
