#!/bin/bash
# Benchmark SCPF on 50 real production contracts

echo "=== SCPF 50-Contract Production Benchmark ==="
echo ""

# 50 verified, audited production contracts across different categories
CONTRACTS=(
    # Stablecoins (10)
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48:ethereum:USDC"
    "0x6B175474E89094C44Da98b954EedeAC495271d0F:ethereum:DAI"
    "0xdAC17F958D2ee523a2206206994597C13D831ec7:ethereum:USDT"
    "0x4Fabb145d64652a948d72533023f6E7A623C7C53:ethereum:BUSD"
    "0x8E870D67F660D95d5be530380D0eC0bd388289E1:ethereum:USDP"
    "0x056Fd409E1d7A124BD7017459dFEa2F387b6d5Cd:ethereum:GUSD"
    "0x0000000000085d4780B73119b644AE5ecd22b376:ethereum:TUSD"
    "0x57Ab1ec28D129707052df4dF418D58a2D46d5f51:ethereum:sUSD"
    "0x1aBaEA1f7C830BD89Acc67eC4af516284b1bC33c:ethereum:EUROC"
    "0xBC6DA0FE9aD5f3b0d58160288917AA56653660E9:ethereum:alUSD"
    
    # DEX (10)
    "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f:ethereum:UniV2Factory"
    "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D:ethereum:UniV2Router"
    "0x1F98431c8aD98523631AE4a59f267346ea31F984:ethereum:UniV3Factory"
    "0xE592427A0AEce92De3Edee1F18E0157C05861564:ethereum:UniV3Router"
    "0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac:ethereum:SushiFactory"
    "0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F:ethereum:SushiRouter"
    "0xBA12222222228d8Ba445958a75a0704d566BF2C8:ethereum:BalancerVault"
    "0x1111111254EEB25477B68fb85Ed929f73A960582:ethereum:1inchRouter"
    "0xDef1C0ded9bec7F1a1670819833240f027b25EfF:ethereum:0xExchange"
    "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45:ethereum:UniV3Router2"
    
    # Lending (10)
    "0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9:ethereum:AaveLending"
    "0x3d9819210A31b4961b30EF54bE2aeD79B9c9Cd3B:ethereum:CompoundComptroller"
    "0x5d3a536E4D6DbD6114cc1Ead35777bAB948E3643:ethereum:cDAI"
    "0x4Ddc2D193948926D02f9B1fE9e1daa0718270ED5:ethereum:cETH"
    "0xccF4429DB6322D5C611ee964527D42E5d685DD6a:ethereum:cWBTC"
    "0x39AA39c021dfbaE8faC545936693aC917d5E7563:ethereum:cUSDC"
    "0xf650C3d88D12dB855b8bf7D11Be6C55A4e07dCC9:ethereum:cUSDT"
    "0x5f98805A4E8be255a32880FDeC7F6728C6568bA0:ethereum:LUSDToken"
    "0x6c8c6b02e7b2be14d4fa6022dfd6d75921d90e4e:ethereum:cBAT"
    "0xC11b1268C1A384e55C48c2391d8d480264A3A7F4:ethereum:cWBTC2"
    
    # Tokens (10)
    "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984:ethereum:UNI"
    "0x514910771AF9Ca656af840dff83E8264EcF986CA:ethereum:LINK"
    "0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9:ethereum:AAVE"
    "0xC18360217D8F7Ab5e7c516566761Ea12Ce7F9D72:ethereum:ENS"
    "0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2:ethereum:MKR"
    "0xc00e94Cb662C3520282E6f5717214004A7f26888:ethereum:COMP"
    "0x0bc529c00C6401aEF6D220BE8C6Ea1667F6Ad93e:ethereum:YFI"
    "0xD533a949740bb3306d119CC777fa900bA034cd52:ethereum:CRV"
    "0x6B3595068778DD592e39A122f4f5a5cF09C90fE2:ethereum:SUSHI"
    "0xba100000625a3754423978a60c9317c58a424e3D:ethereum:BAL"
    
    # Wrapped Assets (10)
    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2:ethereum:WETH"
    "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599:ethereum:WBTC"
    "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0:ethereum:wstETH"
    "0xae78736Cd615f374D3085123A210448E74Fc6393:ethereum:rETH"
    "0xBe9895146f7AF43049ca1c1AE358B0541Ea49704:ethereum:cbETH"
    "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84:ethereum:stETH"
    "0x5A98FcBEA516Cf06857215779Fd812CA3beF1B32:ethereum:LDO"
    "0xD33526068D116cE69F19A9ee46F0bd304F21A51f:ethereum:RPL"
    "0x853d955aCEf822Db058eb8505911ED77F175b99e:ethereum:FRAX"
    "0x3432B6A60D23Ca0dFCa7761B7ab56459D9C964D0:ethereum:FXS"
)

OUTPUT_FILE="/tmp/scpf_50_benchmark_$(date +%Y%m%d_%H%M%S).json"
echo "{\"contracts\": [" > $OUTPUT_FILE

TOTAL=0
SUCCESS=0
FAILED=0

for i in "${!CONTRACTS[@]}"; do
    IFS=':' read -r address chain name <<< "${CONTRACTS[$i]}"
    
    echo -n "[$((i+1))/50] $name... "
    
    # Scan contract
    result=$(./target/release/scpf scan $address --chain $chain --output json 2>/dev/null)
    
    if [ $? -eq 0 ] && [ ! -z "$result" ]; then
        findings=$(echo "$result" | grep -v "WARN" | jq '.[0].matches | length' 2>/dev/null || echo "0")
        
        if [ "$findings" != "0" ] && [ ! -z "$findings" ]; then
            echo "$findings findings"
            
            # Add to JSON output
            if [ $SUCCESS -gt 0 ]; then
                echo "," >> $OUTPUT_FILE
            fi
            echo "  {" >> $OUTPUT_FILE
            echo "    \"name\": \"$name\"," >> $OUTPUT_FILE
            echo "    \"address\": \"$address\"," >> $OUTPUT_FILE
            echo "    \"chain\": \"$chain\"," >> $OUTPUT_FILE
            echo "    \"findings\": $findings," >> $OUTPUT_FILE
            echo "    \"data\": $(echo "$result" | grep -v "WARN")" >> $OUTPUT_FILE
            echo "  }" >> $OUTPUT_FILE
            
            SUCCESS=$((SUCCESS + 1))
            TOTAL=$((TOTAL + findings))
        else
            echo "0 findings"
            SUCCESS=$((SUCCESS + 1))
        fi
    else
        echo "FAILED"
        FAILED=$((FAILED + 1))
    fi
done

echo "]}" >> $OUTPUT_FILE

echo ""
echo "=== Benchmark Complete ==="
echo "Contracts scanned: $SUCCESS"
echo "Failed: $FAILED"
echo "Total findings: $TOTAL"
echo "Average per contract: $(echo "scale=1; $TOTAL / $SUCCESS" | bc)"
echo ""
echo "Results saved to: $OUTPUT_FILE"
echo ""
echo "=== Analysis ==="
python3 << PYEOF
import json

with open('$OUTPUT_FILE') as f:
    data = json.load(f)

contracts = data['contracts']
print(f"Contracts analyzed: {len(contracts)}")

# Group by findings count
by_findings = {}
for c in contracts:
    count = c['findings']
    by_findings.setdefault(count, []).append(c['name'])

print("\nFindings distribution:")
for count in sorted(by_findings.keys()):
    names = by_findings[count]
    print(f"  {count} findings: {len(names)} contracts")
    if len(names) <= 3:
        for name in names:
            print(f"    - {name}")

# Calculate statistics
findings = [c['findings'] for c in contracts]
total = sum(findings)
avg = total / len(findings) if findings else 0
median = sorted(findings)[len(findings)//2] if findings else 0

print(f"\nStatistics:")
print(f"  Total findings: {total}")
print(f"  Average: {avg:.1f}")
print(f"  Median: {median}")
print(f"  Min: {min(findings) if findings else 0}")
print(f"  Max: {max(findings) if findings else 0}")

# Identify high-finding contracts
high = [(c['name'], c['findings']) for c in contracts if c['findings'] > avg]
if high:
    print(f"\nHigh-finding contracts (>{avg:.0f}):")
    for name, count in sorted(high, key=lambda x: -x[1])[:10]:
        print(f"  {name}: {count}")
PYEOF
