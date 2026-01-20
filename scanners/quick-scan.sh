#!/bin/bash
# Quick scan - Critical only - Last 24 hours - Ethereum only - Update templates (1 day)
cargo run --release -- scan --days 1 --chain ethereum --min-severity critical --update-templates 1 --output console
