#!/bin/bash
# DeFi focused - High+ - Last 14 days - Ethereum, BSC, Polygon - Update templates (3 days)
cargo run --release -- scan --days 14 --chain ethereum --min-severity high --update-templates 3 --output json > defi-ethereum.json
cargo run --release -- scan --days 14 --chain bsc --min-severity high --update-templates 0 --output json > defi-bsc.json
cargo run --release -- scan --days 14 --chain polygon --min-severity high --update-templates 0 --output json > defi-polygon.json
echo "Results saved to defi-*.json"
