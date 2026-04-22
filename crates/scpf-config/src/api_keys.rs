use rand::seq::SliceRandom;
use scpf_types::{ApiKeyConfig, Chain};

pub fn load_api_keys_from_env() -> ApiKeyConfig {
    load_api_keys_from_lookup(|name| std::env::var(name).ok())
}

pub fn load_api_keys_from_lookup<F>(mut lookup: F) -> ApiKeyConfig
where
    F: FnMut(&str) -> Option<String>,
{
    let mut config = ApiKeyConfig::new();

    let mut etherscan_keys = collect_key_family("ETHERSCAN", &mut lookup);
    if !etherscan_keys.is_empty() {
        let mut rng = rand::thread_rng();
        etherscan_keys.shuffle(&mut rng);

        config = config.with_keys(Chain::Ethereum, etherscan_keys.clone());
        config = config.with_keys(Chain::Polygon, etherscan_keys.clone());
        config = config.with_keys(Chain::Arbitrum, etherscan_keys);
    }

    config
}

fn collect_key_family<F>(prefix: &str, lookup: &mut F) -> Vec<String>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut keys = Vec::new();

    for index in 1..=6 {
        let variable_name = if index == 1 {
            format!("{}_API_KEY", prefix)
        } else {
            format!("{}_API_KEY_{}", prefix, index)
        };

        if let Some(value) = lookup(&variable_name).filter(|value| !value.trim().is_empty()) {
            keys.push(value);
        }
    }

    keys
}
