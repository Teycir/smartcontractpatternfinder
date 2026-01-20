#!/bin/bash
# Medium and above - Last 60 days - All chains - No template update
cargo run --release -- scan --days 60 --all-chains --min-severity medium --update-templates 0 --output console
