#!/bin/bash
# Zero-day focused - Critical only - Last 3 days - All chains - Update templates (3 days)
cargo run --release -- scan --days 3 --all-chains --min-severity critical --update-templates 3 --output console
