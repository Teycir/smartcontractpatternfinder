#!/bin/bash
# SCPF Full Report Generator
#
# Usage:
#   ./scripts/run_full_report.sh [DAYS] [TOP_N]
#
# Examples:
#   ./scripts/run_full_report.sh              # Default: 10 days, top 10 contracts
#   ./scripts/run_full_report.sh 7            # 7 days, top 10 contracts
#   ./scripts/run_full_report.sh 10 5         # 10 days, top 5 contracts
#   ./scripts/run_full_report.sh 30 20        # 30 days, top 20 contracts

set -e

DAYS=${1:-10}
EXTRACT_TOP_N=${2:-10}
TIMESTAMP=$(date +%s)
REPORT_DIR="/home/teycir/smartcontractpatternfinderReports/report_${TIMESTAMP}"
LOG_FILE="$REPORT_DIR/run.log"
START_TIME=$(date '+%Y-%m-%d %H:%M:%S')

mkdir -p "$REPORT_DIR"

echo "╭────────────────────────────────────────────────────────────╮"
echo "│  SCPF - Smart Contract Pattern Finder                     │"
echo "╰────────────────────────────────────────────────────────────╯"
echo "📅 Period: Last $DAYS days"
echo "📄 Extract Top: $EXTRACT_TOP_N contracts"
echo "🕒 Start: $START_TIME"
echo "📂 Report: $REPORT_DIR"
echo ""

echo "Start: $START_TIME" > "$LOG_FILE"
echo "" >> "$LOG_FILE"

cd "$(dirname "$0")/.." || exit 1

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 STEP 1/2: Fetching 0-Day Vulnerability News"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

export SCPF_REPORT_DIR="$REPORT_DIR"
export SCPF_TIMESTAMP="$TIMESTAMP"
export SCPF_EXTRACT_TOP_N="$EXTRACT_TOP_N"
cargo run --release --bin scpf -- fetch-zero-day --days "$DAYS" 2>&1 | tee -a "$LOG_FILE"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 STEP 2/2: Scanning Contracts for Exploitable Vulnerabilities"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cargo run --release --bin scpf -- scan --days "$DAYS" --chain ethereum --min-severity high 2>&1 | while IFS= read -r line; do
    echo "$line"
    echo "$line" >> "$LOG_FILE"
done

END_TIME=$(date '+%Y-%m-%d %H:%M:%S')
DURATION=$(($(date +%s) - TIMESTAMP))

echo "" >> "$LOG_FILE"
echo "End: $END_TIME" >> "$LOG_FILE"
echo "Duration: ${DURATION}s" >> "$LOG_FILE"

echo ""
echo "╭────────────────────────────────────────────────────────────╮"
echo "│  ✅ FULL REPORT COMPLETE                                   │"
echo "╰────────────────────────────────────────────────────────────╯"
echo ""
echo "🕒 Start: $START_TIME"
echo "🏁 End: $END_TIME"
echo "⏱️  Duration: ${DURATION}s"
echo ""
echo "📂 Report Directory: $REPORT_DIR"
echo "   • 0-Day Summary: 0day_summary.md"
echo "   • Vulnerability Summary: vuln_summary.md"
echo "   • Execution Log: run.log"
echo ""
