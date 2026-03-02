#!/bin/bash
# Run 20-page scan on Etherscan with all patterns + AST validation

set -e

echo "🚀 Starting 20-page scan on Etherscan with AST validation..."
echo "📋 Using all available templates"
echo "🧬 AST validation: ENABLED"
echo "⚡ Concurrency: 2 (safe rate limiting)"
echo ""

# Run scan with 20 pages, ethereum only, critical severity
cargo run --release --bin scpf -- scan \
  --pages 20 \
  --chains ethereum \
  --min-severity critical \
  --concurrency 2

echo ""
echo "✅ Scan complete!"
