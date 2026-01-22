use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exploit {
    pub source: String,
    pub title: String,
    pub date: DateTime<Utc>,
    pub loss_usd: Option<u64>,
    pub exploit_type: ExploitType,
    pub description: String,
    pub contract_address: Option<String>,
    pub tx_hash: Option<String>,
    pub chain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExploitType {
    Reentrancy,
    OracleManipulation,
    AccessControl,
    FlashLoan,
    Unknown,
}

pub struct ZeroDayFetcher {
    client: Client,
}

impl ZeroDayFetcher {
    pub fn new() -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            let auth_value = format!("Bearer {}", token);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&auth_value)?,
            );
        }
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("SCPF/1.0")
            .default_headers(headers)
            .build()?;
        Ok(Self { client })
    }

    pub async fn fetch_recent_exploits(&self, days: i64) -> Result<Vec<Exploit>> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let mut exploits = Vec::new();

        info!("Fetching exploits from last {} days", days);

        // Fetch from ALL sources with 7-day time filter
        match self.fetch_defihacklabs(&cutoff).await {
            Ok(results) => {
                info!("✓ DeFiHackLabs: {} exploits", results.len());
                exploits.extend(results);
            }
            Err(e) => warn!("DeFiHackLabs fetch failed: {}", e),
        }

        match self.fetch_github_security_repos(&cutoff).await {
            Ok(results) => {
                info!("✓ GitHub security repos: {} exploits", results.len());
                exploits.extend(results);
            }
            Err(e) => warn!("GitHub security repos fetch failed: {}", e),
        }

        match self.fetch_immunefi_attackathons(&cutoff).await {
            Ok(results) => {
                info!("✓ Immunefi attackathons: {} exploits", results.len());
                exploits.extend(results);
            }
            Err(e) => warn!("Immunefi attackathons fetch failed: {}", e),
        }

        match self.fetch_rss_feeds().await {
            Ok(results) => {
                info!("✓ RSS feeds: {} exploits", results.len());
                exploits.extend(results);
            }
            Err(e) => warn!("RSS feeds fetch failed: {}", e),
        }

        info!("Found {} total exploits from last {} days", exploits.len(), days);
        Ok(exploits)
    }

    async fn fetch_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<Option<T>> {
        match self.client.get(url).send().await {
            Ok(resp) => match resp.json::<T>().await {
                Ok(data) => Ok(Some(data)),
                Err(e) => {
                    eprintln!("Error: Failed to parse JSON from {}: {}", url, e);
                    Err(anyhow::anyhow!("JSON parsing failed for {}: {}", url, e))
                }
            },
            Err(e) => {
                eprintln!("Error: Failed to fetch {}: {}", url, e);
                Err(anyhow::anyhow!("Network request failed for {}: {}", url, e))
            }
        }
    }

    async fn fetch_defihacklabs(&self, cutoff: &DateTime<Utc>) -> Result<Vec<Exploit>> {
        info!("Fetching from DeFiHackLabs GitHub...");

        let commits_url = "https://api.github.com/repos/SunWeb3Sec/DeFiHackLabs/commits?per_page=100";
        let mut exploits = Vec::new();

        if let Some(commits) = self.fetch_json::<Vec<serde_json::Value>>(commits_url).await? {
            for commit in commits {
                let commit_data = match commit.get("commit") {
                    Some(c) => c,
                    None => continue,
                };
                let author = match commit_data.get("author") {
                    Some(a) => a,
                    None => continue,
                };
                let date_str = match author.get("date").and_then(|d| d.as_str()) {
                    Some(d) => d,
                    None => continue,
                };
                let date = match DateTime::parse_from_rfc3339(date_str) {
                    Ok(d) => d.with_timezone(&Utc),
                    Err(_) => continue,
                };

                if date < *cutoff {
                    continue;
                }

                let message = match commit_data.get("message").and_then(|m| m.as_str()) {
                    Some(m) => m,
                    None => continue,
                };
                let title = message.lines().next().unwrap_or("").to_string();

                // Try to get file path from commit to extract addresses from PoC
                let files = commit.get("files").and_then(|f| f.as_array());
                let mut contract_address = extract_address(&title);
                let mut tx_hash = extract_tx_hash(&title);

                // If commit has files, try to fetch content for addresses
                if let Some(files_arr) = files {
                    for file in files_arr {
                        if let Some(filename) = file.get("filename").and_then(|f| f.as_str()) {
                            if filename.ends_with(".sol") || filename.ends_with(".t.sol") {
                                // Try to fetch file content
                                if let Some(raw_url) = file.get("raw_url").and_then(|u| u.as_str()) {
                                    if let Ok(resp) = self.client.get(raw_url).send().await {
                                        if let Ok(content) = resp.text().await {
                                            // Extract addresses from PoC code
                                            if contract_address.is_none() {
                                                contract_address = extract_address(&content);
                                            }
                                            if tx_hash.is_none() {
                                                tx_hash = extract_tx_hash(&content);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                exploits.push(Exploit {
                    source: "defihacklabs".to_string(),
                    title: title.clone(),
                    date,
                    loss_usd: extract_loss(&title),
                    exploit_type: classify_by_text(&title),
                    description: message.to_string(),
                    contract_address,
                    tx_hash,
                    chain: extract_chain(&title),
                });
            }
        }

        info!("  Found {} from DeFiHackLabs", exploits.len());
        Ok(exploits)
    }

    async fn fetch_github_security_repos(&self, cutoff: &DateTime<Utc>) -> Result<Vec<Exploit>> {
        info!("Fetching from GitHub security repositories...");

        let repos = vec![
            ("immunefi-team", "attackathon"),  // Immunefi bug bounty reports
            ("pcaversaccio", "reentrancy-attacks"),  // Reentrancy attack collection
            ("crytic", "not-so-smart-contracts"),  // Trail of Bits examples
            ("securing", "SCSVS"),  // Smart Contract Security Verification Standard
            ("ConsenSys", "smart-contract-best-practices"),  // Known vulnerabilities
        ];

        let mut all_exploits = Vec::new();

        for (owner, repo_prefix) in repos {
            // Fetch repos matching prefix
            let repos_url = format!("https://api.github.com/users/{}/repos?per_page=20", owner);
            
            if let Some(repos_list) = self.fetch_json::<Vec<serde_json::Value>>(&repos_url).await? {
                for repo in repos_list {
                    let repo_name = match repo.get("name").and_then(|n| n.as_str()) {
                        Some(n) if n.contains(repo_prefix) => n,
                        _ => continue,
                    };

                    let full_name = format!("{}/{}", owner, repo_name);
                    
                    let commits_url = format!("https://api.github.com/repos/{}/commits?per_page=100", full_name);
                    
                    if let Some(commits) = self.fetch_json::<Vec<serde_json::Value>>(&commits_url).await? {
                        let mut poc_count = 0;
                        for commit in commits {
                            if poc_count >= 5 {
                                break;
                            }
                            let commit_data = match commit.get("commit") {
                                Some(c) => c,
                                None => continue,
                            };
                            
                            let date_str = match commit_data.get("author")
                                .and_then(|a| a.get("date"))
                                .and_then(|d| d.as_str()) {
                                Some(d) => d,
                                None => continue,
                            };
                            
                            let date = match DateTime::parse_from_rfc3339(date_str) {
                                Ok(d) => d.with_timezone(&Utc),
                                Err(_) => continue,
                            };

                            if date < *cutoff {
                                continue;
                            }

                            let message = match commit_data.get("message").and_then(|m| m.as_str()) {
                                Some(m) => m,
                                None => continue,
                            };
                            
                            // Only process commits with .sol files
                            let files = commit.get("files").and_then(|f| f.as_array());
                            let has_sol_file = files.as_ref().map_or(false, |f| {
                                f.iter().any(|file| {
                                    file.get("filename")
                                        .and_then(|n| n.as_str())
                                        .map_or(false, |n| n.ends_with(".sol"))
                                })
                            });

                            if !has_sol_file {
                                continue;
                            }

                            poc_count += 1;

                            let title = message.lines().next().unwrap_or("").to_string();
                            let mut contract_address = None;
                            let mut tx_hash = None;

                            if let Some(files_arr) = files {
                                for file in files_arr {
                                    if let Some(filename) = file.get("filename").and_then(|f| f.as_str()) {
                                        if filename.ends_with(".sol") {
                                            if let Some(raw_url) = file.get("raw_url").and_then(|u| u.as_str()) {
                                                if let Ok(resp) = self.client.get(raw_url).send().await {
                                                    if let Ok(content) = resp.text().await {
                                                        contract_address = extract_address(&content);
                                                        tx_hash = extract_tx_hash(&content);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            all_exploits.push(Exploit {
                                source: format!("github_{}", owner),
                                title: title.clone(),
                                date,
                                loss_usd: extract_loss(&title),
                                exploit_type: classify_by_text(&title),
                                description: message.to_string(),
                                contract_address,
                                tx_hash,
                                chain: extract_chain(&title),
                            });
                        }
                    }
                }
            }
        }

        info!("  Found {} from GitHub security repos", all_exploits.len());
        Ok(all_exploits)
    }

    async fn fetch_immunefi_attackathons(&self, cutoff: &DateTime<Utc>) -> Result<Vec<Exploit>> {
        info!("Fetching from Immunefi attackathon repositories...");

        let repos_url = "https://api.github.com/users/immunefi-team/repos?per_page=50";
        let mut all_exploits = Vec::new();

        if let Some(repos) = self.fetch_json::<Vec<serde_json::Value>>(repos_url).await? {
            for repo in repos {
                let repo_name = match repo.get("name").and_then(|n| n.as_str()) {
                    Some(n) if n.contains("attackathon") => n,
                    _ => continue,
                };

                let full_name = format!("immunefi-team/{}", repo_name);
                let commits_url = format!("https://api.github.com/repos/{}/commits?per_page=100", full_name);

                if let Some(commits) = self.fetch_json::<Vec<serde_json::Value>>(&commits_url).await? {
                    let mut poc_count = 0;
                    for commit in commits {
                        if poc_count >= 10 {
                            break;
                        }
                        let commit_data = match commit.get("commit") {
                            Some(c) => c,
                            None => continue,
                        };

                        let date_str = match commit_data.get("author")
                            .and_then(|a| a.get("date"))
                            .and_then(|d| d.as_str()) {
                            Some(d) => d,
                            None => continue,
                        };

                        let date = match DateTime::parse_from_rfc3339(date_str) {
                            Ok(d) => d.with_timezone(&Utc),
                            Err(_) => continue,
                        };

                        if date < *cutoff {
                            continue;
                        }

                        let message = match commit_data.get("message").and_then(|m| m.as_str()) {
                            Some(m) => m,
                            None => continue,
                        };

                        // Only process commits with .sol or PoC files
                        let files = commit.get("files").and_then(|f| f.as_array());
                        let has_poc_file = files.as_ref().map_or(false, |f| {
                            f.iter().any(|file| {
                                file.get("filename")
                                    .and_then(|n| n.as_str())
                                    .map_or(false, |n| {
                                        n.ends_with(".sol") || n.contains("poc") || n.contains("exploit")
                                    })
                            })
                        });

                        if !has_poc_file {
                            continue;
                        }

                        poc_count += 1;

                        let title = message.lines().next().unwrap_or("").to_string();
                        let mut contract_address = None;
                        let mut tx_hash = None;

                        if let Some(files_arr) = files {
                            for file in files_arr {
                                if let Some(filename) = file.get("filename").and_then(|f| f.as_str()) {
                                    if filename.ends_with(".sol") || filename.contains("poc") || filename.contains("exploit") {
                                        if let Some(raw_url) = file.get("raw_url").and_then(|u| u.as_str()) {
                                            if let Ok(resp) = self.client.get(raw_url).send().await {
                                                if let Ok(content) = resp.text().await {
                                                    contract_address = extract_address(&content);
                                                    tx_hash = extract_tx_hash(&content);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        all_exploits.push(Exploit {
                            source: "immunefi".to_string(),
                            title: format!("{} - {}", repo_name, title),
                            date,
                            loss_usd: extract_loss(&title),
                            exploit_type: classify_by_text(&title),
                            description: message.to_string(),
                            contract_address,
                            tx_hash,
                            chain: extract_chain(&title),
                        });
                    }
                }
            }
        }

        info!("  Found {} from Immunefi attackathons", all_exploits.len());
        Ok(all_exploits)
    }

    pub async fn generate_template(
        &self,
        exploits: Vec<Exploit>,
        output_path: &Path,
    ) -> Result<()> {
        use scpf_types::{Pattern, Severity, Template};

        let mut patterns = Vec::new();

        for exploit in exploits {
            let pattern_template = match exploit.exploit_type {
                ExploitType::Reentrancy => REENTRANCY_PATTERN,
                ExploitType::OracleManipulation => ORACLE_PATTERN,
                ExploitType::AccessControl => ACCESS_CONTROL_PATTERN,
                ExploitType::FlashLoan => FLASH_LOAN_PATTERN,
                ExploitType::Unknown => GENERIC_VULNERABILITY_PATTERN,
            };

            patterns.push(Pattern {
                id: format!(
                    "{}_{}",
                    exploit.exploit_type.to_string().to_lowercase(),
                    exploit.date.format("%Y%m%d")
                ),
                pattern: pattern_template.to_string(),
                message: format!(
                    "{} - {} ({})",
                    exploit.title,
                    exploit.source,
                    exploit.date.format("%Y-%m-%d")
                ),
                kind: scpf_types::PatternKind::Regex,
            });
        }

        let template = Template {
            id: "zero-day-live".to_string(),
            name: "Live 0-Day Detection".to_string(),
            description: format!(
                "Generic patterns from last 7 days (Updated: {})",
                Utc::now().format("%Y-%m-%d")
            ),
            severity: Severity::Critical,
            tags: vec!["zero-day".to_string(), "live".to_string()],
            patterns,
        };

        let yaml = serde_yaml::to_string(&template)?;
        std::fs::write(output_path, yaml)?;

        info!("Generated 0-day template with {} patterns", template.patterns.len());
        Ok(())
    }

    async fn fetch_rss_feeds(&self) -> Result<Vec<Exploit>> {
        info!("Fetching from RSS feeds (last 7 days)...");

        let cutoff = Utc::now() - chrono::Duration::days(7);
        let feeds = vec![
            ("https://blog.chainlight.io/rss", "chainlight"),
            ("https://slowmist.medium.com/feed", "slowmist"),
            ("https://medium.com/feed/@peckshield", "peckshield"),
            ("https://medium.com/feed/@Immunefi", "immunefi"),
            ("https://blog.trailofbits.com/feed/", "trailofbits"),
        ];

        let mut all_exploits = Vec::new();

        for (url, source) in feeds {
            match self.client.get(url).send().await {
                Ok(resp) => {
                    if let Ok(text) = resp.text().await {
                        let exploits = parse_rss(&text, &cutoff, source);
                        if !exploits.is_empty() {
                            info!("  Found {} from {}", exploits.len(), source);
                        }
                        all_exploits.extend(exploits);
                    }
                }
                Err(e) => warn!("Failed to fetch {}: {}", source, e),
            }
        }

        Ok(all_exploits)
    }
}

fn parse_rss(xml: &str, cutoff: &DateTime<Utc>, source: &str) -> Vec<Exploit> {
    let mut exploits = Vec::new();

    for item in xml.split("<item>").skip(1) {
        if let Some(end) = item.find("</item>") {
            let item_content = &item[..end];

            let title = extract_xml_tag(item_content, "title");
            let pub_date = extract_xml_tag(item_content, "pubDate");
            let description = extract_xml_tag(item_content, "description");

            if let (Some(title), Some(pub_date)) = (title, pub_date) {
                if let Ok(date) = DateTime::parse_from_rfc2822(&pub_date) {
                    let date_utc = date.with_timezone(&Utc);
                    if date_utc >= *cutoff {
                        exploits.push(Exploit {
                            source: source.to_string(),
                            title: title.clone(),
                            date: date_utc,
                            loss_usd: extract_loss(&title),
                            exploit_type: classify_by_text(&title),
                            description: description.unwrap_or_default(),
                            contract_address: extract_address(&title),
                            tx_hash: extract_tx_hash(&title),
                            chain: extract_chain(&title),
                        });
                    }
                }
            }
        }
    }

    exploits
}

fn extract_xml_tag(content: &str, tag: &str) -> Option<String> {
    let start_pattern = format!("<{}", tag);
    let end_tag = format!("</{}>", tag);

    let start_pos = content.find(&start_pattern)?;
    let content_after_tag = &content[start_pos..];

    let content_start = content_after_tag.find('>')? + 1;
    let full_start = start_pos + content_start;

    let end = content[full_start..].find(&end_tag)? + full_start;

    Some(content[full_start..end].trim().to_string())
}

fn extract_address(text: &str) -> Option<String> {
    // Extract Ethereum address: 0x followed by 40 hex chars
    let re = regex::Regex::new(r"0x[a-fA-F0-9]{40}").ok()?;
    re.find(text).map(|m| m.as_str().to_string())
}

fn extract_tx_hash(text: &str) -> Option<String> {
    // Extract transaction hash: 0x followed by 64 hex chars
    let re = regex::Regex::new(r"0x[a-fA-F0-9]{64}").ok()?;
    re.find(text).map(|m| m.as_str().to_string())
}

fn extract_chain(text: &str) -> Option<String> {
    let text_lower = text.to_lowercase();
    if text_lower.contains("ethereum") || text_lower.contains("eth") {
        Some("ethereum".to_string())
    } else if text_lower.contains("bsc") || text_lower.contains("binance") {
        Some("bsc".to_string())
    } else if text_lower.contains("polygon") {
        Some("polygon".to_string())
    } else if text_lower.contains("arbitrum") {
        Some("arbitrum".to_string())
    } else if text_lower.contains("optimism") {
        Some("optimism".to_string())
    } else if text_lower.contains("base") {
        Some("base".to_string())
    } else {
        None
    }
}

fn extract_loss(text: &str) -> Option<u64> {
    // Extract loss amounts like "$15M", "$5.2M", "$500K"
    let text_lower = text.to_lowercase();

    if let Some(pos) = text_lower.find('$') {
        let after_dollar = &text_lower[pos + 1..];
        let num_str: String = after_dollar
            .chars()
            .take_while(|c| c.is_numeric() || *c == '.')
            .collect();

        if let Ok(num) = num_str.parse::<f64>() {
            if after_dollar.contains('m') {
                return Some((num * 1_000_000.0) as u64);
            } else if after_dollar.contains('k') {
                return Some((num * 1_000.0) as u64);
            }
        }
    }

    None
}

fn classify_by_text(text: &str) -> ExploitType {
    let text_lower = text.to_lowercase();

    if text_lower.contains("reentrancy") || text_lower.contains("reentrant") {
        ExploitType::Reentrancy
    } else if text_lower.contains("flash loan") || text_lower.contains("flashloan") {
        ExploitType::FlashLoan
    } else if text_lower.contains("oracle") || text_lower.contains("price manipulation") {
        ExploitType::OracleManipulation
    } else if text_lower.contains("access control") || text_lower.contains("unauthorized") {
        ExploitType::AccessControl
    } else {
        ExploitType::Unknown
    }
}

impl std::fmt::Display for ExploitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExploitType::Reentrancy => write!(f, "reentrancy"),
            ExploitType::OracleManipulation => write!(f, "oracle_manipulation"),
            ExploitType::AccessControl => write!(f, "access_control"),
            ExploitType::FlashLoan => write!(f, "flash_loan"),
            ExploitType::Unknown => write!(f, "unknown"),
        }
    }
}

const REENTRANCY_PATTERN: &str = r#"\.call\{value:"#;

const ORACLE_PATTERN: &str = r#"(getPrice|latestAnswer|latestRoundData)\("#;

const ACCESS_CONTROL_PATTERN: &str = r#"function\s+(withdraw|mint|burn|transferOwnership|setOwner)\s*\([^)]*\)\s+(public|external)"#;

const FLASH_LOAN_PATTERN: &str = r#"flashLoan|borrow.*repay|address\(this\)\.balance"#;

const GENERIC_VULNERABILITY_PATTERN: &str = r#"(delegatecall|selfdestruct|suicide|tx\.origin)|(unchecked\s*\{)"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_tag_extraction_simple() {
        let xml = r#"<item><title>Test Title</title><description>Test Desc</description></item>"#;

        assert_eq!(
            extract_xml_tag(xml, "title"),
            Some("Test Title".to_string())
        );
        assert_eq!(
            extract_xml_tag(xml, "description"),
            Some("Test Desc".to_string())
        );
    }

    #[test]
    fn test_xml_tag_extraction_with_attributes() {
        let xml = r#"<item><title type="text">Test Title</title><description type="html">Test Desc</description></item>"#;

        assert_eq!(
            extract_xml_tag(xml, "title"),
            Some("Test Title".to_string())
        );
        assert_eq!(
            extract_xml_tag(xml, "description"),
            Some("Test Desc".to_string())
        );
    }

    #[test]
    fn test_xml_tag_extraction_with_namespace() {
        let xml =
            r#"<item><pubDate ns:attr="value">Mon, 01 Jan 2024 12:00:00 GMT</pubDate></item>"#;

        assert_eq!(
            extract_xml_tag(xml, "pubDate"),
            Some("Mon, 01 Jan 2024 12:00:00 GMT".to_string())
        );
    }

    #[test]
    fn test_extract_loss() {
        assert_eq!(extract_loss("Hack: $15M stolen"), Some(15_000_000));
        assert_eq!(extract_loss("Loss of $5.2M"), Some(5_200_000));
        assert_eq!(extract_loss("$500K exploit"), Some(500_000));
        assert_eq!(extract_loss("No loss mentioned"), None);
    }

    #[test]
    fn test_classify_by_text() {
        assert!(matches!(
            classify_by_text("Reentrancy attack"),
            ExploitType::Reentrancy
        ));
        assert!(matches!(
            classify_by_text("Flash loan exploit"),
            ExploitType::FlashLoan
        ));
        assert!(matches!(
            classify_by_text("Oracle manipulation"),
            ExploitType::OracleManipulation
        ));
        assert!(matches!(
            classify_by_text("Access control bypass"),
            ExploitType::AccessControl
        ));
        assert!(matches!(
            classify_by_text("Unknown issue"),
            ExploitType::Unknown
        ));
    }
}
