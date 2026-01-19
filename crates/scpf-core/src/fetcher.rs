use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

pub struct ContractFetcher {
    client: Client,
    api_keys: HashMap<String, String>,
    rate_limiter: Arc<Semaphore>,
}

impl ContractFetcher {
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let mut api_keys = HashMap::new();
        if let Some(key) = api_key {
            api_keys.insert("ethereum".to_string(), key);
        }
        if let Ok(key) = std::env::var("ETHERSCAN_API_KEY") {
            api_keys.insert("ethereum".to_string(), key);
        }
        if let Ok(key) = std::env::var("BSCSCAN_API_KEY") {
            api_keys.insert("bsc".to_string(), key);
        }
        if let Ok(key) = std::env::var("POLYGONSCAN_API_KEY") {
            api_keys.insert("polygon".to_string(), key);
        }

        Ok(Self {
            client,
            api_keys,
            rate_limiter: Arc::new(Semaphore::new(5)),
        })
    }

    pub async fn fetch_source(&self, address: &str, chain: &str) -> Result<String> {
        if !address.starts_with("0x") || address.len() != 42 {
            anyhow::bail!("Invalid address format: {}", address);
        }

        let _permit = self.rate_limiter.acquire().await
            .context("Failed to acquire rate limit permit")?;
        
        let url = self.build_url(address, chain)?;
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch contract source")?;

        let json: Value = response
            .json()
            .await
            .context("Failed to parse response")?;

        if json["status"].as_str() != Some("1") {
            let error_msg = json["message"].as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Unknown API error. Response: {:?}", json));
            anyhow::bail!("API error: {}", error_msg);
        }

        let result = json["result"]
            .as_array()
            .and_then(|arr| arr.first())
            .context("No result in API response")?;

        let source = result["SourceCode"]
            .as_str()
            .context("Source code not found")?
            .to_string();

        Ok(source)
    }

    fn build_url(&self, address: &str, chain: &str) -> Result<String> {
        let base_url = match chain {
            "ethereum" => "https://api.etherscan.io/api",
            "bsc" => "https://api.bscscan.com/api",
            "polygon" => "https://api.polygonscan.com/api",
            _ => anyhow::bail!("Unsupported chain: {}", chain),
        };

        let mut url = format!(
            "{}?module=contract&action=getsourcecode&address={}",
            base_url, address
        );

        if let Some(key) = self.api_keys.get(chain) {
            url.push_str(&format!("&apikey={}", key));
        }

        Ok(url)
    }
}
