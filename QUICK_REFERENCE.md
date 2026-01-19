# Quick Reference: Chain Enum & ApiKeyConfig

## Chain Enum

### Usage

```rust
use scpf_types::Chain;

// Direct usage
let chain = Chain::Ethereum;
let chain = Chain::Bsc;
let chain = Chain::Polygon;

// From string (CLI parsing)
use std::str::FromStr;
let chain = Chain::from_str("ethereum")?;  // Ok(Chain::Ethereum)
let chain = Chain::from_str("eth")?;       // Ok(Chain::Ethereum) - alias
let chain = Chain::from_str("bsc")?;       // Ok(Chain::Bsc)
let chain = Chain::from_str("polygon")?;   // Ok(Chain::Polygon)
let chain = Chain::from_str("invalid")?;   // Err("Unsupported chain: invalid")

// Get API URL
let url = Chain::Ethereum.api_base_url();  // "https://api.etherscan.io/api"
let url = Chain::Bsc.api_base_url();       // "https://api.bscscan.com/api"
let url = Chain::Polygon.api_base_url();   // "https://api.polygonscan.com/api"

// Convert to string
let s = Chain::Ethereum.to_string();       // "ethereum"
let s = Chain::Ethereum.as_str();          // "ethereum"
```

### Supported Aliases

| Chain | Primary | Aliases |
|-------|---------|---------|
| Ethereum | `ethereum` | `eth` |
| BSC | `bsc` | `binance` |
| Polygon | `polygon` | `matic` |

---

## ApiKeyConfig

### Usage

```rust
use scpf_types::{ApiKeyConfig, Chain};

// Load from environment variables
let config = ApiKeyConfig::from_env();
// Reads: ETHERSCAN_API_KEY, BSCSCAN_API_KEY, POLYGONSCAN_API_KEY

// Build manually (for testing)
let config = ApiKeyConfig::new()
    .with_key(Chain::Ethereum, "eth-key".to_string())
    .with_key(Chain::Bsc, "bsc-key".to_string());

// Get API key for chain
if let Some(key) = config.get(Chain::Ethereum) {
    println!("Ethereum API key: {}", key);
}

// Use with ContractFetcher
use scpf_core::ContractFetcher;
let fetcher = ContractFetcher::new(config)?;
```

### Environment Variables

| Chain | Environment Variable |
|-------|---------------------|
| Ethereum | `ETHERSCAN_API_KEY` |
| BSC | `BSCSCAN_API_KEY` |
| Polygon | `POLYGONSCAN_API_KEY` |

---

## Migration Guide

### Before (Old API)

```rust
// String-based chains
let fetcher = ContractFetcher::new(Some("api-key".to_string()))?;
fetcher.fetch_source(address, "ethereum").await?;

// CLI
scpf scan 0x123... --chain ethereum
```

### After (New API)

```rust
// Type-safe chains
let config = ApiKeyConfig::from_env();
let fetcher = ContractFetcher::new(config)?;
fetcher.fetch_source(address, Chain::Ethereum).await?;

// CLI (unchanged for users)
scpf scan 0x123... --chain ethereum  // Still works
scpf scan 0x123... --chain eth       // Alias also works
```

---

## Testing Examples

### Test with Mock API Keys

```rust
#[test]
fn test_with_mock_keys() {
    let config = ApiKeyConfig::new()
        .with_key(Chain::Ethereum, "test-key".to_string());
    
    let fetcher = ContractFetcher::new(config).unwrap();
    // Test without setting environment variables
}
```

### Test Chain Parsing

```rust
#[test]
fn test_chain_parsing() {
    assert_eq!(Chain::from_str("ethereum").unwrap(), Chain::Ethereum);
    assert_eq!(Chain::from_str("eth").unwrap(), Chain::Ethereum);
    assert!(Chain::from_str("invalid").is_err());
}
```

---

## Benefits Summary

### Chain Enum
- ✅ Compile-time validation
- ✅ No typos in chain names
- ✅ Centralized URL configuration
- ✅ Better IDE support

### ApiKeyConfig
- ✅ Testable without environment
- ✅ Dependency injection
- ✅ Type-safe key mapping
- ✅ Clear configuration API
