#!/usr/bin/env python3
import os
import hashlib

top_contracts = [
    ("0x226730cd50a9dc9b11e37528d74622683d6666cc", 1),
    ("0x792c800606663cac54767d3d2a9c89dd32caca18", 2),
    ("0xc2b9667d65de41904a3f504199ec4f9fa2692e46", 3),
    ("0x0b9bd3eaac381a1a6731ff6598a50638e5cffd25", 4),
    ("0xdec6e667212656677a89fb729cab386c35fba553", 5),
    ("0xff327cba9ce268ab83216b4b2fc2261b607d242a", 6),
    ("0xefb111931cabf7c0861341d6191ea8891c73252b", 7),
    ("0x1e768b662a9606abd3a58122d23bc0ef4cbc878b", 8),
    ("0x7fdfd70d73de8a59ff3366bdaa5bc6dc72a66cd4", 9),
    ("0xbc89d1ef7c721f1a28323e2a7753e7e715863bca", 10),
]

cache_dir = os.path.expanduser("~/.cache/scpf/")
output_dir = "validation/top10_sources"
os.makedirs(output_dir, exist_ok=True)

for addr, rank in top_contracts:
    cache_key = f"ethereum:{addr}"
    hash_obj = hashlib.blake2b(cache_key.encode(), digest_size=8)
    cache_file = hash_obj.hexdigest()
    
    cache_path = os.path.join(cache_dir, cache_file)
    if os.path.exists(cache_path):
        output_path = os.path.join(output_dir, f"{rank}_{addr}.sol")
        os.system(f"cp {cache_path} {output_path}")
        size = os.path.getsize(output_path)
        print(f"✅ [{rank}/10] {addr} - {size/1024:.1f} KB")
    else:
        print(f"❌ [{rank}/10] {addr} - Not in cache")

print(f"\n📁 Sources saved to {output_dir}")
