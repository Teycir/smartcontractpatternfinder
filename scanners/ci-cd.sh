#!/bin/bash
# CI/CD integration - High+ - Last 7 days - SARIF output - Update templates (1 day)
cargo run --release -- scan --days 7 --all-chains --min-severity high --update-templates 1 --output sarif > security-report.sarif
echo "SARIF report generated: security-report.sarif"
