# Chain Support Status

## Current Implementation

### Ethereum Mainnet Only

SCPF currently supports **Ethereum mainnet only** via Etherscan API.

**API Endpoint:** `https://api.etherscan.io/api`

### Cascade API Key System

SCPF implements a **rolling cascade fallback system** for handling multiple Etherscan API keys:

**How it works:**
1. Tries `ETHERSCAN_API_KEY` first
2. If rate limited or failed, automatically rotates to `ETHERSCAN_API_KEY_2`
3. Continues through up to `ETHERSCAN_API_KEY_6`
4. 50ms delay between key rotation attempts
5. Built-in rate limiting (5 concurrent requests via semaphore)

**Configuration:**
```bash
export ETHERSCAN_API_KEY="key-1"
export ETHERSCAN_API_KEY_2="key-2"
export ETHERSCAN_API_KEY_3="key-3"
export ETHERSCAN_API_KEY_4="key-4"
export ETHERSCAN_API_KEY_5="key-5"
export ETHERSCAN_API_KEY_6="key-6"
```

**Benefits:**
- ✅ Automatic failover on rate limits
- ✅ Increased throughput (5 calls/sec per key = 30 calls/sec total)
- ✅ Zero downtime during API failures
- ✅ Simple configuration - just add more keys

**Implementation:** See `crates/scpf-core/src/fetcher.rs` - `fetch_source()` method

---

## Planned Multi-Chain Support

### Future Implementation

Each chain has its own separate API endpoint:

| Chain | Actual API Endpoint | Status |
|-------|-------------------|--------|
| Ethereum | `https://api.etherscan.io/api` | ✅ Active |
| BSC | `https://api.bscscan.com/api` | 🚧 Planned |
| Polygon | `https://api.polygonscan.com/api` | 🚧 Planned |
| Arbitrum | `https://api.arbiscan.io/api` | 🚧 Planned |
| Optimism | `https://api-optimistic.etherscan.io/api` | 🚧 Planned |
| Base | `https://api.basescan.org/api` | 🚧 Planned |
| Avalanche | `https://api.snowtrace.io/api` | 🚧 Planned |
| Fantom | `https://api.ftmscan.com/api` | 🚧 Planned |
| Linea | `https://api.lineascan.build/api` | 🚧 Planned |
| Scroll | `https://api.scrollscan.com/api` | 🚧 Planned |

### Required Changes for Multi-Chain

Update `chain.rs` to return chain-specific API endpoints:

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

Remove `chainid` parameter and use chain-specific endpoints:

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

### Multi-Chain API Key Configuration

Each chain will need its own set of API keys with cascade support:

```bash
# Ethereum (current)
export ETHERSCAN_API_KEY="ethereum-key-1"
export ETHERSCAN_API_KEY_2="ethereum-key-2"
# ... up to ETHERSCAN_API_KEY_6

# BSC (planned)
export BSCSCAN_API_KEY="bsc-key-1"
export BSCSCAN_API_KEY_2="bsc-key-2"
# ... up to BSCSCAN_API_KEY_6

# Polygon (planned)
export POLYGONSCAN_API_KEY="polygon-key-1"
export POLYGONSCAN_API_KEY_2="polygon-key-2"
# ... up to POLYGONSCAN_API_KEY_6

# And so on for each chain...
```

**Each chain will support up to 6 keys with the same cascade fallback system.**

---

## Summary

**Current Status:** ✅ Ethereum mainnet fully supported with cascade API key system (up to 6 keys)

**Planned:** Multi-chain support with chain-specific API endpoints and per-chain cascade fallback
