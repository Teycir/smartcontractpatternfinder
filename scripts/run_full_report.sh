#!/bin/bash
# SCPF Full Report - Run both scanners
# Usage: ./run_full_report.sh [days]

set -e

DAYS=${1:-10}
REPORT_DIR="/home/teycir/smartcontractpatternfinderReports"
LOG_FILE="$REPORT_DIR/$(date +%Y%m%d_%H%M%S)_run.log"

mkdir -p "$REPORT_DIR"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  SCPF - Smart Contract Pattern Finder                     ║"
echo "║  Full Security Report Generator                           ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "📅 Period: Last $DAYS days"
echo "📝 Log: $LOG_FILE"
echo ""

cd "$(dirname "$0")/.." || exit 1

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 STEP 1/2: Fetching 0-Day Vulnerability News"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cargo run --release --bin scpf -- fetch-zero-day --days "$DAYS" 2>&1 | tee -a "$LOG_FILE"

echo ""
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 STEP 2/2: Scanning Contracts for Exploitable Vulnerabilities"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cargo run --release --bin scpf -- scan --days "$DAYS" --chain ethereum --min-severity high 2>&1 | tee -a "$LOG_FILE"

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  ✅ FULL REPORT COMPLETE                                   ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "📂 Reports:"
echo "   • 0-Day News: $REPORT_DIR/0days/"
echo "   • Vulnerability Scans: $REPORT_DIR/scans/"
echo "   • Execution Log: $LOG_FILE"
echo ""
