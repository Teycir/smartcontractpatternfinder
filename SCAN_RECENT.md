# Recent Contracts Scan

Scan contracts updated in the last N days across all chains for high to critical vulnerabilities.

## Quick Start

```bash
# Default: 30 days, high+ severity
./scan_recent.sh

# Custom: 60 days, medium+ severity
./scan_recent.sh 60 medium

# Custom: 7 days, critical only
./scan_recent.sh 7 critical
```

## Direct CLI Usage

```bash
# Scan contracts from last 30 days, all chains, high+ severity
cargo run --release -- scan --days 30 --all-chains --min-severity high

# Scan contracts from last 60 days, all chains, medium+ severity
cargo run --release -- scan --days 60 --all-chains --min-severity medium

# Export to JSON
cargo run --release -- scan --days 30 --all-chains --min-severity high --output json > results.json

# Export to SARIF
cargo run --release -- scan --days 30 --all-chains --min-severity high --output sarif > results.sarif
```

## Parameters

- `--days <N>`: Scan contracts from last N days (default: 30)
- `--all-chains`: Scan all supported chains (ethereum, bsc, polygon, arbitrum, optimism, base)
- `--min-severity <LEVEL>`: Minimum severity to report (info, low, medium, high, critical)
- `--output <FORMAT>`: Output format (console, json, sarif)

## Features

✅ Scans contracts updated in last N days  
✅ All chains supported  
✅ Filters by severity (high to critical by default)  
✅ No template updates (uses existing templates)  
✅ Smart caching to avoid redundant API calls  
✅ Concurrent scanning for performance  

## Requirements

- API keys configured for each chain (see main README)
- Rust toolchain installed
- Templates directory with vulnerability patterns
