# Chain Support Status

## Current Implementation Issue

The codebase claims to support 10 chains, but the implementation is **incorrect**.

### Problem

In `crates/scpf-types/src/chain.rs`, all chains return the same API endpoint:

```rust
pub fn api_base_url(&self) -> &'static str {
    // Etherscan V2 Unified API - single endpoint for all supported chains
    "https://api.etherscan.io/v2/api"
}
```

The fetcher then uses: `https://api.etherscan.io/v2/api?chainid={chain_id}&...`

**This is incorrect.** Etherscan does NOT have a unified V2 API with `chainid` parameter.

### Reality

Each chain has its own separate API endpoint:

| Chain | Actual API Endpoint | Status |
|-------|-------------------|--------|
| Ethereum | `https://api.etherscan.io/api` | ✅ Works |
| BSC | `https://api.bscscan.com/api` | ❌ Not implemented |
| Polygon | `https://api.polygonscan.com/api` | ❌ Not implemented |
| Arbitrum | `https://api.arbiscan.io/api` | ❌ Not implemented |
| Optimism | `https://api-optimistic.etherscan.io/api` | ❌ Not implemented |
| Base | `https://api.basescan.org/api` | ❌ Not implemented |
| Avalanche | `https://api.snowtrace.io/api` | ❌ Not implemented |
| Fantom | `https://api.ftmscan.com/api` | ❌ Not implemented |
| Linea | `https://api.lineascan.build/api` | ❌ Not implemented |
| Scroll | `https://api.scrollscan.com/api` | ❌ Not implemented |

### Fix Required

Update `chain.rs`:

```rust
pub fn api_base_url(&self) -> &'static str {
    match self {
        Chain::Ethereum => "https://api.etherscan.io/api",
        Chain::Bsc => "https://api.bscscan.com/api",
        Chain::Polygon => "https://api.polygonscan.com/api",
        Chain::Arbitrum => "https://api.arbiscan.io/api",
        Chain::Optimism => "https://api-optimistic.etherscan.io/api",
        Chain::Base => "https://api.basescan.org/api",
        Chain::Avalanche => "https://api.snowtrace.io/api",
        Chain::Fantom => "https://api.ftmscan.com/api",
        Chain::Linea => "https://api.lineascan.build/api",
        Chain::Scroll => "https://api.scrollscan.com/api",
    }
}
```

Remove `chainid` parameter from `fetcher.rs`:

```rust
fn build_url_with_key(&self, address: &str, chain: Chain, key: &str) -> Result<String> {
    Ok(format!(
        "{}?module=contract&action=getsourcecode&address={}&apikey={}",
        chain.api_base_url(),
        address,
        key
    ))
}
```

### API Key Configuration

Each chain needs its own API key environment variable:

```bash
export ETHERSCAN_API_KEY="your-ethereum-key"

```

### Documentation Updates Needed

1. **README.md** - Update "Supported Chains" table to show only Ethereum as active
2. **Configuration section** - Document chain-specific API keys
3. **Examples** - Remove multi-chain examples until fixed

### Current Status

**Only Ethereum mainnet is properly supported.**

All other chains will fail because they're hitting the wrong API endpoint.
