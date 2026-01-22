#!/bin/bash
# Get top 40 most PoC-able contracts from all chains
# Rust handles: 7 days | All chains | High/Critical only | Top 60 by risk → Top 40 by PoC
# Usage: ./get_top40_pocable.sh
# Output: Scans contracts from the last 7 days across all supported chains and identifies
#         the top 40 contracts most suitable for proof-of-concept exploit development

set -e

cd "$(dirname "$0")/.." || exit 1

if ! cargo run --release --bin scpf -- scan \
  --days 7 \
  --all-chains \
  --min-severity high; then
    echo "Error: SCPF scan failed" >&2
    exit 1
fi
