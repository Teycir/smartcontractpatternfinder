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

    // Load all Etherscan-family chains with 6 keys each
    if let Some(keys) = some_if_not_empty(load_keys("ETHERSCAN", 6)) {
        config = config.with_keys(Chain::Ethereum, keys);
    }

    if let Some(keys) = some_if_not_empty(load_keys("BSCSCAN", 6)) {
        config = config.with_keys(Chain::Bsc, keys);
    }

    if let Some(keys) = some_if_not_empty(load_keys("POLYGONSCAN", 6)) {
        config = config.with_keys(Chain::Polygon, keys);
    }

    if let Some(keys) = some_if_not_empty(load_keys("ARBISCAN", 6)) {
        config = config.with_keys(Chain::Arbitrum, keys);
    }

    if let Some(keys) = some_if_not_empty(load_keys("OPTIMISTIC_ETHERSCAN", 6)) {
        config = config.with_keys(Chain::Optimism, keys);
    }

    if let Some(keys) = some_if_not_empty(load_keys("BASESCAN", 6)) {
        config = config.with_keys(Chain::Base, keys);
    }

    if let Some(keys) = some_if_not_empty(load_keys("SNOWTRACE", 6)) {
        config = config.with_keys(Chain::Avalanche, keys);
    }

    if let Some(keys) = some_if_not_empty(load_keys("FTMSCAN", 6)) {
        config = config.with_keys(Chain::Fantom, keys);
    }

    // ZkSync doesn't require API keys (public API)
    config = config.with_keys(Chain::ZkSync, vec![String::new()]);

    // Linea with fallback keys
    if let Some(keys) = some_if_not_empty(load_keys("LINEASCAN", 6)) {
        config = config.with_keys(Chain::Linea, keys);
    }

    // Scroll with fallback keys
    if let Some(keys) = some_if_not_empty(load_keys("SCROLLSCAN", 6)) {
        config = config.with_keys(Chain::Scroll, keys);
    }

    // Zora with fallback keys (3 keys)
    if let Some(keys) = some_if_not_empty(load_keys("ZORASCAN", 3)) {
        config = config.with_keys(Chain::Zora, keys);
    }

    config
}

fn some_if_not_empty(v: Vec<String>) -> Option<Vec<String>> {
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}
