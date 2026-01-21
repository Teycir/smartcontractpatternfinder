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

        let keys = self.api_keys.get(chain).unwrap_or(&[]);
        if keys.is_empty() {
            anyhow::bail!(
                "No API keys configured for {}. Set {}_API_KEY environment variable.",
                chain.as_str().to_uppercase(),
                chain.as_str().to_uppercase()
            );
        }

        let mut last_error = None;
        let mut invalid_key_found = false;
        let mut rate_limited_found = false;

        for (idx, key) in keys.iter().enumerate() {
            let fetch_fn = || async {
                let _permit = self
                    .rate_limiter
                    .acquire()
                    .await
                    .context("Failed to acquire rate limit permit")?;

                let url = self.build_url_with_key(address, chain, key)?;

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

                    // Categorize API errors
                    if error_msg.contains("Invalid API Key")
                        || error_msg.contains("Missing/Invalid")
                    {
                        anyhow::bail!("INVALID_KEY: {}", error_msg);
                    } else if error_msg.contains("rate limit")
                        || error_msg.contains("Max rate limit")
                    {
                        anyhow::bail!("RATE_LIMITED: {}", error_msg);
                    } else if error_msg.contains("NOTOK") {
                        anyhow::bail!("API_ERROR: {}", error_msg);
                    }

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

            match fetch_fn
                .retry(
                    ExponentialBuilder::default()
                        .with_max_times(3)
                        .with_min_delay(Duration::from_millis(500))
                        .with_max_delay(Duration::from_secs(5)),
                )
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let err_msg = e.to_string();
                    if err_msg.contains("INVALID_KEY") {
                        invalid_key_found = true;
                        tracing::error!(
                            "API key {} is invalid or expired for {}",
                            idx + 1,
                            chain.as_str()
                        );
                    } else if err_msg.contains("RATE_LIMITED") {
                        rate_limited_found = true;
                        tracing::warn!(
                            "API key {} is rate limited for {}",
                            idx + 1,
                            chain.as_str()
                        );
                    } else if idx < keys.len() - 1 {
                        tracing::warn!("API key {} failed, trying next key", idx + 1);
                    }
                    last_error = Some(e);
                }
            }
        }

        // Analyze error categories across all attempts to provide helpful message
        if invalid_key_found {
            return Err(anyhow::anyhow!(
                "All API keys for {} are invalid or expired. Please update your API keys.",
                chain.as_str()
            ));
        }

        if rate_limited_found {
            return Err(anyhow::anyhow!(
                "All API keys for {} have reached their rate limit. Please wait or add more keys.",
                chain.as_str()
            ));
        }

        Err(last_error
            .unwrap_or_else(|| anyhow::anyhow!("All API keys failed for {}", chain.as_str())))
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
        // ZkSync and Zora don't use v2 API
        if matches!(chain, Chain::ZkSync | Chain::Zora) {
            Ok(format!(
                "{}?module=contract&action=getsourcecode&address={}&apikey={}",
                chain.api_base_url(),
                address,
                key
            ))
        } else {
            // All Etherscan-based chains use v2 API with chainid
            Ok(format!(
                "{}?chainid={}&module=contract&action=getsourcecode&address={}&apikey={}",
                chain.api_base_url(),
                chain.chain_id(),
                address,
                key
            ))
        }
    }

    /// Fetch recently deployed contracts from last N days
    pub async fn fetch_recent_contracts(&self, chain: Chain, days: u64) -> Result<Vec<String>> {
        let keys = self.api_keys.get(chain).unwrap_or(&[]);
        if keys.is_empty() {
            anyhow::bail!("No API keys configured for {}", chain.as_str());
        }

        let key = &keys[0];
        let cutoff_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            - (days * 24 * 60 * 60);

        // Get block number from N days ago
        let block_url = if matches!(chain, Chain::ZkSync | Chain::Zora) {
            format!(
                "{}?module=block&action=getblocknobytime&timestamp={}&closest=before&apikey={}",
                chain.api_base_url(),
                cutoff_timestamp,
                key
            )
        } else {
            format!(
                "{}?chainid={}&module=block&action=getblocknobytime&timestamp={}&closest=before&apikey={}",
                chain.api_base_url(),
                chain.chain_id(),
                cutoff_timestamp,
                key
            )
        };

        let response = self.client.get(&block_url).send().await?;
        let json: Value = response
            .json()
            .await
            .context("Failed to decode block response")?;

        if json["status"].as_str() != Some("1") {
            anyhow::bail!("Failed to get block number: {:?}", json["message"]);
        }

        let from_block = json["result"]
            .as_str()
            .context("No block number in response")?;

        // Fetch contract creation events (OwnershipTransferred topic0)
        let logs_url = if matches!(chain, Chain::ZkSync | Chain::Zora) {
            format!(
                "{}?module=logs&action=getLogs&fromBlock={}&toBlock=latest&topic0=0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0&page=1&offset=100&apikey={}",
                chain.api_base_url(),
                from_block,
                key
            )
        } else {
            format!(
                "{}?chainid={}&module=logs&action=getLogs&fromBlock={}&toBlock=latest&topic0=0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0&page=1&offset=100&apikey={}",
                chain.api_base_url(),
                chain.chain_id(),
                from_block,
                key
            )
        };

        let response = self.client.get(&logs_url).send().await?;
        let text = response.text().await?;
        let json: Value = serde_json::from_str(&text).with_context(|| {
            format!(
                "Failed to decode logs response. Body: {}",
                &text[..text.len().min(500)]
            )
        })?;

        if json["status"].as_str() != Some("1") {
            anyhow::bail!("Failed to get logs: {:?}", json["message"]);
        }

        let logs = json["result"].as_array().context("No logs in response")?;

        let mut addresses = std::collections::HashSet::new();
        for log in logs {
            if let Some(addr) = log["address"].as_str() {
                addresses.insert(addr.to_string());
            }
        }

        Ok(addresses.into_iter().collect())
    }
}
