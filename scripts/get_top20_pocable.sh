#!/bin/bash
# Get top 20 most PoC-able contracts from all chains
# Rust handles: 7 days | All chains | Top 40 by risk → Top 20 by PoC | Output format | Folder structure

set -e

cd "$(dirname "$0")/.." || exit 1

cargo run --release --bin scpf -- scan \
  --days 7 \
  --all-chains \
  --min-severity high
