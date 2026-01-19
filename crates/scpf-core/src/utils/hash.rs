use sha2::{Digest, Sha256};
use xxhash_rust::xxh3::xxh3_64;

pub fn hash_source(source: &str) -> u64 {
    xxh3_64(source.as_bytes())
}

pub fn hash_address(address: &str, chain_id: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(chain_id.to_le_bytes());
    hasher.update(address.to_lowercase().as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..16])
}

pub fn cache_key(address: &str, chain_id: u64) -> String {
    format!("{}:{}", chain_id, address.to_lowercase())
}
