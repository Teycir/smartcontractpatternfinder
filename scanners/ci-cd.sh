#!/bin/bash
# CI/CD integration - High+ - Last 7 days - SARIF output - Update templates (1 day)
set -e
if ! cargo run --release -- scan --days 7 --all-chains --min-severity high --update-templates 1 --output sarif > security-report.sarif; then
  echo "Error: SCPF scan failed" >&2
  exit 1
fi
echo "SARIF report generated: security-report.sarif"
