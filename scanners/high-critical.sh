#!/bin/bash
# High to Critical vulnerabilities - Last 30 days - All chains - No template update
cargo run --release -- scan --days 30 --all-chains --min-severity high --update-templates 0 --output console
