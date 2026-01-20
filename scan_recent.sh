#!/bin/bash
# Scan contracts updated in last 30 days (configurable)
# All chains, high to critical vulnerabilities
# Templates updated for 0 days (no update)

DAYS=${1:-30}
MIN_SEVERITY=${2:-high}

echo "🔍 Smart Contract Pattern Finder - Recent Contracts Scan"
echo "=================================================="
echo "Configuration:"
echo "  • Days: $DAYS"
echo "  • Chains: All (ethereum, bsc, polygon, arbitrum, optimism, base)"
echo "  • Min Severity: $MIN_SEVERITY"
echo "  • Template Update: Disabled (0 days)"
echo ""

cargo run --release -- scan \
  --days "$DAYS" \
  --all-chains \
  --min-severity "$MIN_SEVERITY" \
  --output console
