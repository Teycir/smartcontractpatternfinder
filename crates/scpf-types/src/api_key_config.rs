use crate::Chain;

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
        self.keys.entry(chain).or_default().push(key);
        self
    }

    pub fn with_keys(mut self, chain: Chain, keys: Vec<String>) -> Self {
        self.keys.insert(chain, keys);
        self
    }



    pub fn get(&self, chain: Chain) -> Option<&[String]> {
        self.keys.get(&chain).map(|v| v.as_slice())
    }
}
