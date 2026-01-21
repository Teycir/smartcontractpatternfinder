use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
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
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("SCPF/1.0")
            .build()?;
        Ok(Self { client })
    }

    pub async fn fetch_recent_exploits(&self, days: i64) -> Result<Vec<Exploit>> {
        let cutoff = Utc::now() - Duration::days(days);
        let mut exploits = Vec::new();

        info!("Fetching exploits from last {} days", days);

        // Fetch from all sources with graceful degradation
        match self.fetch_defillama_hacks(&cutoff).await {
            Ok(results) => exploits.extend(results),
            Err(e) => warn!("DeFiLlama fetch failed: {}", e),
        }

        match self.fetch_defihacklabs(&cutoff).await {
            Ok(results) => exploits.extend(results),
            Err(e) => warn!("DeFiHackLabs fetch failed: {}", e),
        }

        match self.fetch_github_solidity_advisories(&cutoff).await {
            Ok(results) => exploits.extend(results),
            Err(e) => warn!("GitHub Solidity fetch failed: {}", e),
        }

        match self.fetch_rss_feeds(&cutoff).await {
            Ok(results) => exploits.extend(results),
            Err(e) => warn!("RSS feeds fetch failed: {}", e),
        }

        info!("Found {} total exploits", exploits.len());
        Ok(exploits)
    }

    async fn fetch_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<Option<T>> {
        match self.client.get(url).send().await {
            Ok(resp) => {
                match resp.json::<T>().await {
                    Ok(data) => Ok(Some(data)),
                    Err(e) => {
                        eprintln!("Error: Failed to parse JSON from {}: {}", url, e);
                        Err(anyhow::anyhow!("JSON parsing failed for {}: {}", url, e))
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: Failed to fetch {}: {}", url, e);
                Err(anyhow::anyhow!("Network request failed for {}: {}", url, e))
            }
        }
    }

    async fn fetch_defillama_hacks(&self, cutoff: &DateTime<Utc>) -> Result<Vec<Exploit>> {
        info!("Fetching from DeFiLlama Hacks API...");

        if let Some(hacks) = self
            .fetch_json::<Vec<serde_json::Value>>("https://api.llama.fi/hacks")
            .await?
        {
            let exploits: Vec<Exploit> = hacks
                .into_iter()
                .filter_map(|hack| {
                    // Date is Unix timestamp
                    let timestamp = hack.get("date")?.as_i64()?;
                    let date = DateTime::from_timestamp(timestamp, 0)?.with_timezone(&Utc);

                    if date < *cutoff {
                        return None;
                    }

                    let technique = hack.get("technique").and_then(|t| t.as_str()).unwrap_or("");
                    let language = hack.get("language").and_then(|l| l.as_str()).unwrap_or("");

                    // Filter for Solidity/Vyper or include all if no language specified
                    if !language.is_empty()
                        && !language.contains("Solidity")
                        && !language.contains("Vyper")
                    {
                        return None;
                    }

                    let name = hack.get("name")?.as_str()?.to_string();
                    let loss = hack
                        .get("loss_amount")
                        .and_then(|l| l.as_f64())
                        .map(|l| (l * 1_000_000.0) as u64);

                    Some(Exploit {
                        source: "defillama".to_string(),
                        title: name,
                        date,
                        loss_usd: loss,
                        exploit_type: classify_by_text(technique),
                        description: format!("Technique: {} | Language: {}", technique, language),
                    })
                })
                .collect();

            info!("  Found {} from DeFiLlama", exploits.len());
            Ok(exploits)
        } else {
            Ok(Vec::new())
        }
    }

    async fn fetch_defihacklabs(&self, cutoff: &DateTime<Utc>) -> Result<Vec<Exploit>> {
        info!("Fetching from DeFiHackLabs GitHub...");

        let url = "https://api.github.com/repos/SunWeb3Sec/DeFiHackLabs/commits?per_page=30";

        if let Some(commits) = self.fetch_json::<Vec<serde_json::Value>>(url).await? {
            let exploits: Vec<Exploit> = commits
                .into_iter()
                .filter_map(|commit| {
                    let commit_data = commit.get("commit")?;
                    let author = commit_data.get("author")?;
                    let date_str = author.get("date")?.as_str()?;
                    let date = DateTime::parse_from_rfc3339(date_str)
                        .ok()?
                        .with_timezone(&Utc);

                    if date < *cutoff {
                        return None;
                    }

                    let message = commit_data.get("message")?.as_str()?;
                    let title = message.lines().next()?.to_string();

                    Some(Exploit {
                        source: "defihacklabs".to_string(),
                        title: title.clone(),
                        date,
                        loss_usd: extract_loss(&title),
                        exploit_type: classify_by_text(&title),
                        description: message.to_string(),
                    })
                })
                .collect();

            info!("  Found {} from DeFiHackLabs", exploits.len());
            Ok(exploits)
        } else {
            Ok(Vec::new())
        }
    }

    async fn fetch_github_solidity_advisories(
        &self,
        cutoff: &DateTime<Utc>,
    ) -> Result<Vec<Exploit>> {
        info!("Fetching from GitHub Solidity Advisories...");

        let url = "https://api.github.com/repos/ethereum/solidity/security/advisories";

        if let Some(advisories) = self.fetch_json::<Vec<serde_json::Value>>(url).await? {
            let exploits: Vec<Exploit> = advisories
                .into_iter()
                .filter_map(|adv| {
                    let date_str = adv.get("published_at")?.as_str()?;
                    let date = DateTime::parse_from_rfc3339(date_str)
                        .ok()?
                        .with_timezone(&Utc);

                    if date < *cutoff {
                        return None;
                    }

                    Some(Exploit {
                        source: "github_solidity".to_string(),
                        title: adv.get("summary")?.as_str()?.to_string(),
                        date,
                        loss_usd: None,
                        exploit_type: ExploitType::Unknown,
                        description: adv.get("description")?.as_str()?.to_string(),
                    })
                })
                .collect();

            info!("  Found {} from GitHub Solidity", exploits.len());
            Ok(exploits)
        } else {
            Ok(Vec::new())
        }
    }

    async fn fetch_rss_feeds(&self, cutoff: &DateTime<Utc>) -> Result<Vec<Exploit>> {
        info!("Fetching from global RSS feeds...");

        let feeds = vec![
            // Asian Security Firms (Early Detection)
            ("https://blog.chainlight.io/rss", "chainlight"),
            ("https://slowmist.medium.com/feed", "slowmist"),
            ("https://medium.com/feed/blocksec", "blocksec"),
            ("https://medium.com/feed/@peckshield", "peckshield"),
            ("https://medium.com/feed/@goplussecurity", "goplussecurity"),
            // Global Security Firms
            ("https://medium.com/feed/@Immunefi", "immunefi"),
            ("https://blog.trailofbits.com/feed/", "trailofbits"),
            ("https://web3isgoinggreat.com/feed.xml", "web3isgoinggreat"),
        ];

        let mut all_exploits = Vec::new();

        for (url, source) in feeds {
            match self.client.get(url).send().await {
                Ok(resp) => {
                    if let Ok(text) = resp.text().await {
                        let exploits = parse_rss(&text, cutoff, source);
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

    pub async fn generate_template(
        &self,
        exploits: Vec<Exploit>,
        output_path: &Path,
    ) -> Result<()> {
        use scpf_types::{Pattern, PatternKind, Severity, Template};

        let patterns: Vec<Pattern> = exploits
            .into_iter()
            .filter_map(|exploit| {
                let pattern_template = match exploit.exploit_type {
                    ExploitType::Reentrancy => REENTRANCY_PATTERN,
                    ExploitType::OracleManipulation => ORACLE_PATTERN,
                    ExploitType::AccessControl => ACCESS_CONTROL_PATTERN,
                    ExploitType::FlashLoan => FLASH_LOAN_PATTERN,
                    ExploitType::Unknown => return None,
                };

                Some(Pattern {
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
                    kind: PatternKind::Semantic,
                })
            })
            .collect();

        let template = Template {
            id: "zero-day-live".to_string(),
            name: "Live 0-Day Detection".to_string(),
            description: format!(
                "Auto-generated from security feeds (Updated: {})",
                Utc::now().format("%Y-%m-%d")
            ),
            severity: Severity::Critical,
            tags: vec!["zero-day".to_string(), "live".to_string()],
            patterns,
        };

        let yaml = serde_yaml::to_string(&template)?;
        std::fs::write(output_path, yaml)?;

        info!(
            "Generated template with {} patterns",
            template.patterns.len()
        );
        Ok(())
    }
}

fn parse_rss(xml: &str, cutoff: &DateTime<Utc>, source: &str) -> Vec<Exploit> {
    let mut exploits = Vec::new();

    // Simple XML parsing for RSS items
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
                        });
                    }
                }
            }
        }
    }

    exploits
}

fn extract_xml_tag(content: &str, tag: &str) -> Option<String> {
    // Handle tags with attributes: <tag attr="value">content</tag>
    let start_pattern = format!("<{}", tag);
    let end_tag = format!("</{}>", tag);

    let start_pos = content.find(&start_pattern)?;
    let content_after_tag = &content[start_pos..];
    
    // Find the end of opening tag (could be <tag> or <tag attr="val">)
    let content_start = content_after_tag.find('>')? + 1;
    let full_start = start_pos + content_start;
    
    let end = content[full_start..].find(&end_tag)? + full_start;

    Some(content[full_start..end].trim().to_string())
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

const REENTRANCY_PATTERN: &str = r#"(function_definition
  body: (block
    (expression_statement
      (call_expression
        function: (member_expression
          property: (identifier) @call (#match? @call "^(call|transfer|send)$"))))
    (expression_statement
      (assignment_expression))))"#;

const ORACLE_PATTERN: &str = r#"(call_expression
  function: (member_expression
    property: (identifier) @method (#match? @method "^(getPrice|latestAnswer|latestRoundData)$")))"#;

const ACCESS_CONTROL_PATTERN: &str = r#"(function_definition
  name: (identifier) @name
  visibility: (visibility) @vis (#match? @vis "^(public|external)$"))"#;

const FLASH_LOAN_PATTERN: &str = r#"(function_definition
  body: (block
    (expression_statement
      (binary_expression
        (member_expression
          property: (identifier) @balance (#eq? @balance "balance"))))))"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_tag_extraction_simple() {
        let xml = r#"<item><title>Test Title</title><description>Test Desc</description></item>"#;
        
        assert_eq!(extract_xml_tag(xml, "title"), Some("Test Title".to_string()));
        assert_eq!(extract_xml_tag(xml, "description"), Some("Test Desc".to_string()));
    }

    #[test]
    fn test_xml_tag_extraction_with_attributes() {
        let xml = r#"<item><title type="text">Test Title</title><description type="html">Test Desc</description></item>"#;
        
        assert_eq!(extract_xml_tag(xml, "title"), Some("Test Title".to_string()));
        assert_eq!(extract_xml_tag(xml, "description"), Some("Test Desc".to_string()));
    }

    #[test]
    fn test_xml_tag_extraction_with_namespace() {
        let xml = r#"<item><pubDate ns:attr="value">Mon, 01 Jan 2024 12:00:00 GMT</pubDate></item>"#;
        
        assert_eq!(extract_xml_tag(xml, "pubDate"), Some("Mon, 01 Jan 2024 12:00:00 GMT".to_string()));
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
        assert!(matches!(classify_by_text("Reentrancy attack"), ExploitType::Reentrancy));
        assert!(matches!(classify_by_text("Flash loan exploit"), ExploitType::FlashLoan));
        assert!(matches!(classify_by_text("Oracle manipulation"), ExploitType::OracleManipulation));
        assert!(matches!(classify_by_text("Access control bypass"), ExploitType::AccessControl));
        assert!(matches!(classify_by_text("Unknown issue"), ExploitType::Unknown));
    }
}
