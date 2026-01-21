#!/bin/bash
# Critical vulnerabilities only - Last 7 days - All chains - No template update
if ! cargo run --release -- scan --days 7 --all-chains --min-severity critical --update-templates 0 --output console; then
  echo "Error: Critical scan failed" >&2
  exit 1
fi
