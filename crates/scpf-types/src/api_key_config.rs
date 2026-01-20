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
        config
    }

    pub fn get(&self, chain: Chain) -> Option<&str> {
        self.keys.get(&chain).map(|s| s.as_str())
    }
}
