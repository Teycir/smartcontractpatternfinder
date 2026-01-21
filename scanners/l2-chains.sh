#!/bin/bash
# L2 chains - High+ - Last 30 days - Arbitrum, Optimism, Base - No template update
if ! cargo run --release -- scan --days 30 --chain arbitrum --min-severity high --update-templates 0 --output console; then
  echo "Error: Arbitrum scan failed" >&2
  exit 1
fi
if ! cargo run --release -- scan --days 30 --chain optimism --min-severity high --update-templates 0 --output console; then
  echo "Error: Optimism scan failed" >&2
  exit 1
fi
if ! cargo run --release -- scan --days 30 --chain base --min-severity high --update-templates 0 --output console; then
  echo "Error: Base scan failed" >&2
  exit 1
fi
