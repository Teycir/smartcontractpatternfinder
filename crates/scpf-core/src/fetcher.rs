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
            // Small delay between key attempts (50ms)
            if idx > 0 {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }

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
                let error_msg = json["message"].as_str().unwrap_or("Unknown error");
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
                    tracing::warn!(
                        "API key {} returned no source code, trying next key",
                        idx + 1
                    );
                    last_error = Some(anyhow::anyhow!("Source code not found"));
                    continue;
                }
            };

            // Success! Return the parsed source code
            return Ok(Self::parse_source_code(source_code));
        }

        // All keys failed
        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("All {} API keys failed for {}", keys.len(), chain.as_str())
        }))
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

    /// Fetch recently deployed contracts from last N pages (approximately last 7 days)
    pub async fn fetch_recent_contracts(&self, chain: Chain, pages: u64) -> Result<Vec<String>> {
        let keys = self.api_keys.get(chain).unwrap_or(&[]);
        if keys.is_empty() {
            anyhow::bail!("No API keys configured for {}", chain.as_str());
        }

        let max_pages = pages.min(100); // Cap at 100 pages

        // Calculate starting block from ~7 days ago to keep results recent
        let cutoff_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            - (7 * 24 * 60 * 60);

        for (idx, key) in keys.iter().enumerate() {
            // Get starting block number
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
                    if idx < keys.len() - 1 {
                        continue;
                    }
                    anyhow::bail!("Request failed: {}", e);
                }
            };

            let json: Value = match response.json().await {
                Ok(j) => j,
                Err(e) => {
                    if idx < keys.len() - 1 {
                        continue;
                    }
                    anyhow::bail!("Failed to decode response: {}", e);
                }
            };

            if json["status"].as_str() != Some("1") {
                if idx < keys.len() - 1 {
                    continue;
                }
                anyhow::bail!("Failed to get block number: {:?}", json["message"]);
            }

            let from_block = match json["result"].as_str() {
                Some(b) => b,
                None => {
                    if idx < keys.len() - 1 {
                        continue;
                    }
                    anyhow::bail!("No block number in response");
                }
            };
            // Paginate through results to get requested number of pages
            let mut all_addresses = std::collections::HashSet::new();
            let mut consecutive_failures = 0;
            let max_consecutive_failures = 5;
            let mut current_key_idx = idx;
            let mut last_page = 0;

            for page in 1..=max_pages {
                last_page = page;
                // Rate limiting: 5 calls/sec = 200ms between calls
                if page > 1 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                }

                // Rotate to next key if current one is failing
                let current_key = &keys[current_key_idx % keys.len()];

                let logs_url = format!(
                    "{}?chainid={}&module=logs&action=getLogs&fromBlock={}&toBlock=latest&topic0=0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0&page={}&offset=100&apikey={}",
                    chain.api_base_url(),
                    chain.chain_id(),
                    from_block,
                    page,
                    current_key
                );

                let mut retry_count = 0;
                let max_retries = 3;

                let (text, should_continue) = loop {
                    let response = match self.client.get(&logs_url).send().await {
                        Ok(r) => r,
                        Err(e) => {
                            retry_count += 1;
                            if retry_count >= max_retries {
                                tracing::warn!(
                                    "Page {} failed after {} retries: {}",
                                    page,
                                    max_retries,
                                    e
                                );
                                consecutive_failures += 1;
                                break (String::new(), false);
                            }
                            tracing::debug!(
                                "Page {} retry {}/{}: {}",
                                page,
                                retry_count,
                                max_retries,
                                e
                            );
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    let text = match response.text().await {
                        Ok(t) => t,
                        Err(e) => {
                            retry_count += 1;
                            if retry_count >= max_retries {
                                tracing::warn!(
                                    "Page {} read failed after {} retries: {}",
                                    page,
                                    max_retries,
                                    e
                                );
                                consecutive_failures += 1;
                                break (String::new(), false);
                            }
                            tracing::debug!(
                                "Page {} read retry {}/{}: {}",
                                page,
                                retry_count,
                                max_retries,
                                e
                            );
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    break (text, true);
                };

                if !should_continue {
                    if consecutive_failures >= max_consecutive_failures {
                        // Try rotating to next API key
                        if keys.len() > 1 {
                            current_key_idx += 1;
                            if current_key_idx - idx < keys.len() {
                                tracing::info!(
                                    "Rotating to API key {} after failures",
                                    (current_key_idx % keys.len()) + 1
                                );
                                consecutive_failures = 0;
                                continue; // Retry this page with new key
                            }
                        }
                        tracing::warn!(
                            "Stopping pagination after {} consecutive failures",
                            consecutive_failures
                        );
                        break;
                    }
                    continue; // Skip this page, try next
                }

                let json: Value = match serde_json::from_str(&text) {
                    Ok(j) => j,
                    Err(e) => {
                        tracing::warn!("Page {} parse failed: {}", page, e);
                        consecutive_failures += 1;
                        if consecutive_failures >= max_consecutive_failures {
                            break;
                        }
                        continue; // Skip this page, try next
                    }
                };

                if json["status"].as_str() != Some("1") {
                    // Check if it's an error or just no more results
                    if let Some(msg) = json["message"].as_str() {
                        if msg.contains("No records found") || msg.contains("No transactions found")
                        {
                            tracing::info!("No more results at page {}", page);
                            break;
                        }
                        tracing::warn!("Page {} API error: {}", page, msg);
                        consecutive_failures += 1;
                        if consecutive_failures >= max_consecutive_failures {
                            break;
                        }
                        continue;
                    }
                    break;
                }

                let logs = match json["result"].as_array() {
                    Some(l) if !l.is_empty() => l,
                    _ => {
                        tracing::info!("Empty results at page {}", page);
                        break;
                    }
                };

                // Reset consecutive failures on success
                consecutive_failures = 0;

                for log in logs {
                    if let Some(addr) = log["address"].as_str() {
                        all_addresses.insert(addr.to_string());
                    }
                }

                // If we got less than 100 results, we've reached the end
                if logs.len() < 100 {
                    tracing::info!(
                        "Pagination complete at page {} ({} total addresses)",
                        page,
                        all_addresses.len()
                    );
                    break;
                }
            }

            tracing::info!(
                "Fetched {} unique addresses from {} after {} pages",
                all_addresses.len(),
                chain.as_str(),
                last_page
            );
            return Ok(all_addresses.into_iter().collect());
        }

        anyhow::bail!("All API keys failed for {}", chain.as_str())
    }
}
