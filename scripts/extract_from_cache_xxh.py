#!/usr/bin/env python3
import os

top_contracts = [
    ("0x226730cd50a9dc9b11e37528d74622683d6666cc", 1, 7),
    ("0x792c800606663cac54767d3d2a9c89dd32caca18", 2, 7),
    ("0xc2b9667d65de41904a3f504199ec4f9fa2692e46", 3, 157),
    ("0x0b9bd3eaac381a1a6731ff6598a50638e5cffd25", 4, 7),
    ("0xdec6e667212656677a89fb729cab386c35fba553", 5, 5),
]

cache_dir = os.path.expanduser("~/.cache/scpf/")
output_dir = "validation/top10_sources"
os.makedirs(output_dir, exist_ok=True)

for addr, rank, findings in top_contracts:
    found = False
    for cache_file in os.listdir(cache_dir):
        cache_path = os.path.join(cache_dir, cache_file)
        if os.path.isfile(cache_path):
            try:
                with open(cache_path, 'r', errors='ignore') as f:
                    content = f.read()
                    if 'pragma solidity' in content and len(content) > 10000:
                        output_path = os.path.join(output_dir, f"{rank}_{addr}.sol")
                        with open(output_path, 'w') as out:
                            out.write(content)
                        size = len(content)
                        print(f"✅ [{rank}/5] {addr} - {size/1024:.1f} KB - {findings} findings")
                        found = True
                        break
            except:
                pass
    if not found:
        print(f"❌ [{rank}/5] {addr} - Not found")

print(f"\n📁 Top 5 sources saved to {output_dir}")
