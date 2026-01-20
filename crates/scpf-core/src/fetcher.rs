use anyhow::{Context, Result};
use backon::{ExponentialBuilder, Retryable};
use reqwest::Client;
use scpf_types::{ApiKeyConfig, Chain};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

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

        let fetch_fn = || async {
            let _permit = self
                .rate_limiter
                .acquire()
                .await
                .context("Failed to acquire rate limit permit")?;

            let url = self.build_url(address, chain)?;

            let response = self
                .client
                .get(&url)
                .send()
                .await
                .context("Failed to fetch contract source")?;

            let json: Value = response.json().await.context("Failed to parse response")?;

            if json["status"].as_str() != Some("1") {
                let error_msg = json["message"]
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("Unknown API error. Response: {:?}", json));
                anyhow::bail!("API error: {}", error_msg);
            }

            let result = json["result"]
                .as_array()
                .and_then(|arr| arr.first())
                .context("No result in API response")?;

            let source_code = result["SourceCode"]
                .as_str()
                .context("Source code not found")?;

            Ok(Self::parse_source_code(source_code))
        };

        fetch_fn
            .retry(
                ExponentialBuilder::default()
                    .with_max_times(3)
                    .with_min_delay(Duration::from_millis(500))
                    .with_max_delay(Duration::from_secs(5)),
            )
            .await
    }

    pub fn parse_source_code(source_code: &str) -> String {
        // Trim whitespace and normalize double-braced JSON
        let normalized = source_code.trim();
        let normalized = if normalized.starts_with("{{") && normalized.ends_with("}}") {
            &normalized[1..normalized.len() - 1]
        } else {
            normalized
        };

        if normalized.starts_with('{') {
            if let Ok(json) = serde_json::from_str::<Value>(normalized) {
                // Check for Etherscan-style multi-file JSON structure
                if json.get("language").is_some() && json.get("sources").is_some() {
                    if let Some(sources) = json["sources"].as_object() {
                        let mut combined = String::new();
                        for (filename, file_obj) in sources {
                            if let Some(content) = file_obj["content"].as_str() {
                                combined.push_str(&format!("// File: {}\n", filename));
                                combined.push_str(content);
                                combined.push_str("\n\n");
                            }
                        }
                        if !combined.is_empty() {
                            return combined;
                        }
                    }
                } else if let Some(sources) = json["sources"].as_object() {
                    // Fallback for simpler multi-file JSON without language field
                    let mut combined = String::new();
                    for (filename, file_obj) in sources {
                        if let Some(content) = file_obj["content"].as_str() {
                            combined.push_str(&format!("// File: {}\n", filename));
                            combined.push_str(content);
                            combined.push_str("\n\n");
                        }
                    }
                    if !combined.is_empty() {
                        return combined;
                    }
                }
            }
        }
        source_code.to_string()
    }

    fn build_url(&self, address: &str, chain: Chain) -> Result<String> {
        let mut url = format!(
            "{}?module=contract&action=getsourcecode&address={}",
            chain.api_base_url(),
            address
        );

        if let Some(key) = self.api_keys.get(chain) {
            url.push_str(&format!("&apikey={}", key));
        }

        Ok(url)
    }
}
