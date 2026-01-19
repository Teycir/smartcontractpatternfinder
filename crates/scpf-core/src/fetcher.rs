use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use scpf_types::{ApiKeyConfig, Chain};

pub struct ContractFetcher {
    client: Client,
    api_keys: ApiKeyConfig,
    rate_limiter: Arc<Semaphore>,
}

impl ContractFetcher {
    pub fn new(api_keys: ApiKeyConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_keys,
            rate_limiter: Arc::new(Semaphore::new(5)),
        })
    }

    pub async fn fetch_source(&self, address: &str, chain: Chain) -> Result<String> {
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

    fn build_url(&self, address: &str, chain: Chain) -> Result<String> {
        let mut url = format!(
            "{}?module=contract&action=getsourcecode&address={}",
            chain.api_base_url(), address
        );

        if let Some(key) = self.api_keys.get(chain) {
            url.push_str(&format!("&apikey={}", key));
        }

        Ok(url)
    }
}
