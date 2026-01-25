#!/bin/bash
# Scan local .sol files (no API keys needed)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

[ $# -eq 0 ] && echo "Usage: $0 <file.sol> [...]" && exit 1

cd "$PROJECT_ROOT"
[ ! -f target/release/scpf ] && cargo build --release

exec target/release/scpf scan --local "$@" --templates templates
