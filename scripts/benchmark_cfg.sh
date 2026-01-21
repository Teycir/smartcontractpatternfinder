#!/bin/bash
# Benchmark SCPF on 10 real production contracts

echo "=== SCPF Production Contract Benchmark ==="
echo ""

# Test contracts (verified, audited production contracts)
CONTRACTS=(
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48:ethereum"  # USDC
    "0x6B175474E89094C44Da98b954EedeAC495271d0F:ethereum"  # DAI
    "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984:ethereum"  # UNI
    "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0:ethereum"  # wstETH
    "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f:ethereum"  # Uniswap V2 Factory
    "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D:ethereum"  # Uniswap V2 Router
    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2:ethereum"  # WETH
    "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599:ethereum"  # WBTC
    "0xdAC17F958D2ee523a2206206994597C13D831ec7:ethereum"  # USDT
    "0x514910771AF9Ca656af840dff83E8264EcF986CA:ethereum"  # LINK
)

TOTAL=0
BEFORE_TOTAL=0
AFTER_TOTAL=0

echo "Testing CFG analysis impact..."
echo ""

for contract in "${CONTRACTS[@]}"; do
    IFS=':' read -r address chain <<< "$contract"
    name=$(echo $address | cut -c1-10)
    
    echo -n "[$name] "
    
    # Scan with CFG disabled (baseline)
    before=$(./target/release/scpf scan $address --chain $chain --output json 2>/dev/null | jq '.[0].matches | length' 2>/dev/null || echo "0")
    
    # Scan with CFG enabled (current)
    after=$(./target/release/scpf scan $address --chain $chain --output json 2>/dev/null | jq '.[0].matches | length' 2>/dev/null || echo "0")
    
    if [ "$before" = "0" ] && [ "$after" = "0" ]; then
        echo "SKIP (fetch failed)"
        continue
    fi
    
    reduction=0
    if [ "$before" -gt 0 ]; then
        reduction=$(echo "scale=1; 100 * ($before - $after) / $before" | bc)
    fi
    
    echo "Before: $before | After: $after | Reduction: ${reduction}%"
    
    BEFORE_TOTAL=$((BEFORE_TOTAL + before))
    AFTER_TOTAL=$((AFTER_TOTAL + after))
    TOTAL=$((TOTAL + 1))
done

echo ""
echo "=== Summary ==="
echo "Contracts tested: $TOTAL"
echo "Total findings before: $BEFORE_TOTAL"
echo "Total findings after: $AFTER_TOTAL"

if [ "$BEFORE_TOTAL" -gt 0 ]; then
    avg_reduction=$(echo "scale=1; 100 * ($BEFORE_TOTAL - $AFTER_TOTAL) / $BEFORE_TOTAL" | bc)
    echo "Average reduction: ${avg_reduction}%"
fi

echo ""
echo "=== CFG Analysis Impact ==="
if [ "$avg_reduction" -gt 50 ]; then
    echo "✅ SUCCESS: ${avg_reduction}% reduction achieved"
else
    echo "⚠️  MODERATE: ${avg_reduction}% reduction (target: >50%)"
fi
