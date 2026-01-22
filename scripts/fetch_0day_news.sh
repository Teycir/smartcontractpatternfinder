#!/bin/bash
# Quick wrapper for SCPF 0-day news fetcher
# Usage: ./scripts/fetch_0day_news.sh [days]

DAYS=${1:-7}

echo "Fetching 0-day news for last ${DAYS} days..."
if ! scpf fetch-zero-day --days "${DAYS}"; then
    echo "Error: Failed to fetch 0-day news" >&2
    exit 1
fi
