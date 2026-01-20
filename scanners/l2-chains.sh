#!/bin/bash
# L2 chains - High+ - Last 30 days - Arbitrum, Optimism, Base - No template update
cargo run --release -- scan --days 30 --chain arbitrum --min-severity high --update-templates 0 --output console
cargo run --release -- scan --days 30 --chain optimism --min-severity high --update-templates 0 --output console
cargo run --release -- scan --days 30 --chain base --min-severity high --update-templates 0 --output console
