#!/bin/bash
# Get top 40 most PoC-able contracts from all chains
# Rust handles: 7 days | All chains | High/Critical only | Top 60 by risk → Top 40 by PoC

set -e

cd "$(dirname "$0")/.." || exit 1

cargo run --release --bin scpf -- scan \
  --days 7 \
  --all-chains \
  --min-severity high
