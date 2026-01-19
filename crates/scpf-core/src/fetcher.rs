use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

pub struct ContractFetcher {
    client: Client,
    api_key: Option<String>,
}

impl ContractFetcher {
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, api_key })
    }

    pub async fn fetch_source(&self, address: &str, chain: &str) -> Result<String> {
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

        let source = json["result"][0]["SourceCode"]
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

        if let Some(key) = &self.api_key {
            url.push_str(&format!("&apikey={}", key));
        }

        Ok(url)
    }
}
