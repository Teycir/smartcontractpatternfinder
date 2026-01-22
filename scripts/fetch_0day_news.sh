#!/bin/bash
# Quick wrapper for SCPF 0-day news fetcher
# Usage: ./scripts/fetch_0day_news.sh [days]

DAYS=${1:-7}

echo "Fetching 0-day news for last ${DAYS} days..."
scpf fetch-zero-day --days "${DAYS}"
