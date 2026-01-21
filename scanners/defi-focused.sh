#!/bin/bash
# DeFi focused - High+ - Last 14 days - Ethereum, BSC, Polygon - Update templates (3 days)
if ! cargo run --release -- scan --days 14 --chain ethereum --min-severity high --update-templates 3 --output json > defi-ethereum.json; then
  echo "Error: Ethereum scan failed" >&2
  exit 1
fi
if ! cargo run --release -- scan --days 14 --chain bsc --min-severity high --update-templates 0 --output json > defi-bsc.json; then
  echo "Error: BSC scan failed" >&2
  exit 1
fi
if ! cargo run --release -- scan --days 14 --chain polygon --min-severity high --update-templates 0 --output json > defi-polygon.json; then
  echo "Error: Polygon scan failed" >&2
  exit 1
fi
echo "Results saved to defi-*.json"
