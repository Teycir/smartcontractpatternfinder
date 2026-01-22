use rand::seq::SliceRandom;
use scpf_types::{ApiKeyConfig, Chain};

pub fn load_api_keys_from_env() -> ApiKeyConfig {
    let mut config = ApiKeyConfig::new();

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

    // Etherscan V2 Unified API - free plan supports: Ethereum, Polygon, Arbitrum
    let etherscan_keys = load_keys("ETHERSCAN", 6);
    if !etherscan_keys.is_empty() {
        config = config.with_keys(Chain::Ethereum, etherscan_keys.clone());
        config = config.with_keys(Chain::Polygon, etherscan_keys.clone());
        config = config.with_keys(Chain::Arbitrum, etherscan_keys);
    }

    config
}


