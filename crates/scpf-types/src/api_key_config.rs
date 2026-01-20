use crate::Chain;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ApiKeyConfig {
    keys: HashMap<Chain, String>,
}

impl ApiKeyConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_key(mut self, chain: Chain, key: String) -> Self {
        self.keys.insert(chain, key);
        self
    }

    pub fn from_env() -> Self {
        let mut config = Self::new();
        if let Ok(key) = std::env::var("ETHERSCAN_API_KEY") {
            config.keys.insert(Chain::Ethereum, key);
        }
        if let Ok(key) = std::env::var("BSCSCAN_API_KEY") {
            config.keys.insert(Chain::Bsc, key);
        }
        if let Ok(key) = std::env::var("POLYGONSCAN_API_KEY") {
            config.keys.insert(Chain::Polygon, key);
        }
        if let Ok(key) = std::env::var("ARBISCAN_API_KEY") {
            config.keys.insert(Chain::Arbitrum, key);
        }
        if let Ok(key) = std::env::var("OPTIMISTIC_ETHERSCAN_API_KEY") {
            config.keys.insert(Chain::Optimism, key);
        }
        if let Ok(key) = std::env::var("BASESCAN_API_KEY") {
            config.keys.insert(Chain::Base, key);
        }
        if let Ok(key) = std::env::var("SNOWTRACE_API_KEY") {
            config.keys.insert(Chain::Avalanche, key);
        }
        if let Ok(key) = std::env::var("FTMSCAN_API_KEY") {
            config.keys.insert(Chain::Fantom, key);
        }
        if let Ok(key) = std::env::var("ZKSYNC_API_KEY") {
            config.keys.insert(Chain::ZkSync, key);
        }
        if let Ok(key) = std::env::var("LINEASCAN_API_KEY") {
            config.keys.insert(Chain::Linea, key);
        }
        if let Ok(key) = std::env::var("SCROLLSCAN_API_KEY") {
            config.keys.insert(Chain::Scroll, key);
        }
        if let Ok(key) = std::env::var("ZORASCAN_API_KEY") {
            config.keys.insert(Chain::Zora, key);
        }
        config
    }

    pub fn get(&self, chain: Chain) -> Option<&str> {
        self.keys.get(&chain).map(|s| s.as_str())
    }
}
