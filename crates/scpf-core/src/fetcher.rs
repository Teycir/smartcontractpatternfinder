use anyhow::{Context, Result};
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

    pub fn new_without_keys() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_keys: ApiKeyConfig::new(),
            rate_limiter: Arc::new(Semaphore::new(5)),
        })
    }

    pub async fn fetch_source(&self, address: &str, chain: Chain) -> Result<String> {
        if !address.starts_with("0x") || address.len() != 42 {
            anyhow::bail!("Invalid address format: {}", address);
        }

        let keys = self.api_keys.get(chain).unwrap_or(&[]);
        if keys.is_empty() {
            anyhow::bail!(
                "No API keys configured for {}. Set {}_API_KEY environment variable.",
                chain.as_str().to_uppercase(),
                chain.as_str().to_uppercase()
            );
        }

        let mut last_error = None;

        for (idx, key) in keys.iter().enumerate() {
            let _permit = self
                .rate_limiter
                .acquire()
                .await
                .context("Failed to acquire rate limit permit")?;

            let url = self.build_url_with_key(address, chain, key)?;

            let response = match self.client.get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!("API key {} network error: {}", idx + 1, e);
                    last_error = Some(anyhow::anyhow!("Network error: {}", e));
                    continue;
                }
            };

            let json: Value = match response.json().await {
                Ok(j) => j,
                Err(e) => {
                    tracing::warn!("API key {} parse error: {}", idx + 1, e);
                    last_error = Some(anyhow::anyhow!("Parse error: {}", e));
                    continue;
                }
            };

            if json["status"].as_str() != Some("1") {
                let error_msg = json["message"]
                    .as_str()
                    .unwrap_or("Unknown error");
                tracing::warn!("API key {} failed: {}, trying next key", idx + 1, error_msg);
                last_error = Some(anyhow::anyhow!("API error: {}", error_msg));
                continue; // Try next key
            }

            let result = match json["result"].as_array().and_then(|arr| arr.first()) {
                Some(r) => r,
                None => {
                    tracing::warn!("API key {} returned no result, trying next key", idx + 1);
                    last_error = Some(anyhow::anyhow!("No result in API response"));
                    continue;
                }
            };

            let source_code = match result["SourceCode"].as_str() {
                Some(s) => s,
                None => {
                    tracing::warn!("API key {} returned no source code, trying next key", idx + 1);
                    last_error = Some(anyhow::anyhow!("Source code not found"));
                    continue;
                }
            };

            // Success! Return the parsed source code
            return Ok(Self::parse_source_code(source_code));
        }

        // All keys failed
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All {} API keys failed for {}", keys.len(), chain.as_str())))
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

    fn build_url_with_key(&self, address: &str, chain: Chain, key: &str) -> Result<String> {
        // All chains use Etherscan v2 API with chainid
        Ok(format!(
            "{}?chainid={}&module=contract&action=getsourcecode&address={}&apikey={}",
            chain.api_base_url(),
            chain.chain_id(),
            address,
            key
        ))
    }

    /// Fetch recently deployed contracts from last N days
    pub async fn fetch_recent_contracts(&self, chain: Chain, days: u64) -> Result<Vec<String>> {
        let keys = self.api_keys.get(chain).unwrap_or(&[]);
        if keys.is_empty() {
            anyhow::bail!("No API keys configured for {}", chain.as_str());
        }

        let cutoff_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            - (days * 24 * 60 * 60);

        let mut last_error = None;

        for (idx, key) in keys.iter().enumerate() {
            let block_url = format!(
                "{}?chainid={}&module=block&action=getblocknobytime&timestamp={}&closest=before&apikey={}",
                chain.api_base_url(),
                chain.chain_id(),
                cutoff_timestamp,
                key
            );

            let response = match self.client.get(&block_url).send().await {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Request failed: {}", e));
                    continue;
                }
            };

            let json: Value = match response.json().await {
                Ok(j) => j,
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Failed to decode response: {}", e));
                    continue;
                }
            };

            if json["status"].as_str() != Some("1") {
                last_error = Some(anyhow::anyhow!("Failed to get block number: {:?}", json["message"]));
                if idx < keys.len() - 1 {
                    tracing::warn!("API key {} failed for block fetch, trying next key", idx + 1);
                    continue;
                }
                break;
            }

            let from_block = match json["result"].as_str() {
                Some(b) => b,
                None => {
                    last_error = Some(anyhow::anyhow!("No block number in response"));
                    continue;
                }
            };

            let logs_url = format!(
                "{}?chainid={}&module=logs&action=getLogs&fromBlock={}&toBlock=latest&topic0=0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0&page=1&offset=100&apikey={}",
                chain.api_base_url(),
                chain.chain_id(),
                from_block,
                key
            );

            let response = match self.client.get(&logs_url).send().await {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Logs request failed: {}", e));
                    continue;
                }
            };

            let text = match response.text().await {
                Ok(t) => t,
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Failed to read response: {}", e));
                    continue;
                }
            };

            let json: Value = match serde_json::from_str(&text) {
                Ok(j) => j,
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Failed to parse JSON: {}", e));
                    continue;
                }
            };

            if json["status"].as_str() != Some("1") {
                last_error = Some(anyhow::anyhow!("Failed to get logs: {:?}", json["message"]));
                if idx < keys.len() - 1 {
                    tracing::warn!("API key {} failed for logs fetch, trying next key", idx + 1);
                    continue;
                }
                break;
            }

            let logs = match json["result"].as_array() {
                Some(l) => l,
                None => {
                    last_error = Some(anyhow::anyhow!("No logs in response"));
                    continue;
                }
            };

            let mut addresses = std::collections::HashSet::new();
            for log in logs {
                if let Some(addr) = log["address"].as_str() {
                    addresses.insert(addr.to_string());
                }
            }

            return Ok(addresses.into_iter().collect());
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All API keys failed for {}", chain.as_str())))
    }
}
