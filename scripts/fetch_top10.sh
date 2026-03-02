#!/bin/bash
# Fetch source code for top 10 contracts

TOP_CONTRACTS=(
    "0x226730cd50a9dc9b11e37528d74622683d6666cc"
    "0x792c800606663cac54767d3d2a9c89dd32caca18"
    "0xc2b9667d65de41904a3f504199ec4f9fa2692e46"
    "0x0b9bd3eaac381a1a6731ff6598a50638e5cffd25"
    "0xdec6e667212656677a89fb729cab386c35fba553"
    "0xff327cba9ce268ab83216b4b2fc2261b607d242a"
    "0xefb111931cabf7c0861341d6191ea8891c73252b"
    "0x1e768b662a9606abd3a58122d23bc0ef4cbc878b"
    "0x7fdfd70d73de8a59ff3366bdaa5bc6dc72a66cd4"
    "0xbc89d1ef7c721f1a28323e2a7753e7e715863bca"
)

OUTPUT_DIR="validation/top10_sources"
mkdir -p "$OUTPUT_DIR"

for i in "${!TOP_CONTRACTS[@]}"; do
    addr="${TOP_CONTRACTS[$i]}"
    rank=$((i + 1))
    echo "[$rank/10] Fetching $addr..."
    
    curl -s "https://api.etherscan.io/api?module=contract&action=getsourcecode&address=$addr&apikey=$ETHERSCAN_API_KEY" \
        | jq -r '.result[0].SourceCode' > "$OUTPUT_DIR/${rank}_${addr}.sol"
    
    sleep 0.3
done

echo "✅ Sources saved to $OUTPUT_DIR"
