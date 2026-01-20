use crate::Chain;
use rand::seq::SliceRandom;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ApiKeyConfig {
    keys: HashMap<Chain, Vec<String>>,
}

impl ApiKeyConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_key(mut self, chain: Chain, key: String) -> Self {
        self.keys.entry(chain).or_insert_with(Vec::new).push(key);
        self
    }

    pub fn from_env() -> Self {
        let mut config = Self::new();
        
        // Helper to load and shuffle keys
        let load_keys = |prefix: &str, count: usize| -> Vec<String> {
            let mut keys = Vec::new();
            for i in 1..=count {
                let var_name = if i == 1 {
                    format!("{}_API_KEY", prefix)
                } else {
                    format!("{}_API_KEY_{}", prefix, i)
                };
                if let Ok(key) = std::env::var(&var_name) {
                    keys.push(key);
                }
            }
            // Shuffle keys randomly
            let mut rng = rand::thread_rng();
            keys.shuffle(&mut rng);
            keys
        };
        
        // Load all Etherscan-family chains with 6 keys each
        let eth_keys = load_keys("ETHERSCAN", 6);
        if !eth_keys.is_empty() {
            config.keys.insert(Chain::Ethereum, eth_keys);
        }
        
        let bsc_keys = load_keys("BSCSCAN", 6);
        if !bsc_keys.is_empty() {
            config.keys.insert(Chain::Bsc, bsc_keys);
        }
        
        let polygon_keys = load_keys("POLYGONSCAN", 6);
        if !polygon_keys.is_empty() {
            config.keys.insert(Chain::Polygon, polygon_keys);
        }
        
        let arb_keys = load_keys("ARBISCAN", 6);
        if !arb_keys.is_empty() {
            config.keys.insert(Chain::Arbitrum, arb_keys);
        }
        
        let op_keys = load_keys("OPTIMISTIC_ETHERSCAN", 6);
        if !op_keys.is_empty() {
            config.keys.insert(Chain::Optimism, op_keys);
        }
        
        let base_keys = load_keys("BASESCAN", 6);
        if !base_keys.is_empty() {
            config.keys.insert(Chain::Base, base_keys);
        }
        
        let avax_keys = load_keys("SNOWTRACE", 6);
        if !avax_keys.is_empty() {
            config.keys.insert(Chain::Avalanche, avax_keys);
        }
        
        let ftm_keys = load_keys("FTMSCAN", 6);
        if !ftm_keys.is_empty() {
            config.keys.insert(Chain::Fantom, ftm_keys);
        }
        if let Ok(key) = std::env::var("ZKSYNC_API_KEY") {
            config.keys.insert(Chain::ZkSync, vec![key]);
        }
        if let Ok(key) = std::env::var("LINEASCAN_API_KEY") {
            config.keys.insert(Chain::Linea, vec![key]);
        }
        if let Ok(key) = std::env::var("SCROLLSCAN_API_KEY") {
            config.keys.insert(Chain::Scroll, vec![key]);
        }
        if let Ok(key) = std::env::var("ZORASCAN_API_KEY") {
            config.keys.insert(Chain::Zora, vec![key]);
        }
        config
    }

    pub fn get(&self, chain: Chain) -> Option<&[String]> {
        self.keys.get(&chain).map(|v| v.as_slice())
    }
}
