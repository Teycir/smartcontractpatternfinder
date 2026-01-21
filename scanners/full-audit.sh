#!/bin/bash
# Full audit - All severities - Last 90 days - All chains - Update templates (7 days)
if ! cargo run --release -- scan --days 90 --all-chains --min-severity info --update-templates 7 --output console; then
  echo "Error: Full audit scan failed" >&2
  exit 1
fi
