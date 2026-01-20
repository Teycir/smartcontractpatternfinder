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

        // Real working sources
        exploits.extend(self.fetch_defihacklabs(&cutoff).await?);
        exploits.extend(self.fetch_github_solidity_advisories(&cutoff).await?);
        exploits.extend(self.fetch_rekt_rss(&cutoff).await?);

        info!("Found {} total exploits", exploits.len());
        Ok(exploits)
    }

    async fn fetch_defihacklabs(&self, cutoff: &DateTime<Utc>) -> Result<Vec<Exploit>> {
        info!("Fetching from DeFiHackLabs GitHub...");
        
        let url = "https://api.github.com/repos/SunWeb3Sec/DeFiHackLabs/commits?per_page=30";
        
        match self.client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(commits) = resp.json::<Vec<serde_json::Value>>().await {
                    let exploits: Vec<Exploit> = commits
                        .into_iter()
                        .filter_map(|commit| {
                            let commit_data = commit.get("commit")?;
                            let author = commit_data.get("author")?;
                            let date_str = author.get("date")?.as_str()?;
                            let date = DateTime::parse_from_rfc3339(date_str).ok()?.with_timezone(&Utc);
                            
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
                                exploit_type: classify_exploit(&title),
                                description: message.to_string(),
                            })
                        })
                        .collect();
                    
                    info!("  Found {} from DeFiHackLabs", exploits.len());
                    return Ok(exploits);
                }
            }
            Err(e) => warn!("Failed to fetch DeFiHackLabs: {}", e),
        }
        Ok(Vec::new())
    }

    async fn fetch_github_solidity_advisories(&self, cutoff: &DateTime<Utc>) -> Result<Vec<Exploit>> {
        info!("Fetching from GitHub Solidity Advisories...");
        
        let url = "https://api.github.com/repos/ethereum/solidity/security/advisories";
        
        match self.client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(advisories) = resp.json::<Vec<serde_json::Value>>().await {
                    let exploits: Vec<Exploit> = advisories
                        .into_iter()
                        .filter_map(|adv| {
                            let date_str = adv.get("published_at")?.as_str()?;
                            let date = DateTime::parse_from_rfc3339(date_str).ok()?.with_timezone(&Utc);
                            
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
                    return Ok(exploits);
                }
            }
            Err(e) => warn!("Failed to fetch GitHub Solidity: {}", e),
        }
        Ok(Vec::new())
    }

    async fn fetch_rekt_rss(&self, cutoff: &DateTime<Utc>) -> Result<Vec<Exploit>> {
        info!("Fetching from Rekt News RSS...");
        
        match self.client
            .get("https://rekt.news/feed.xml")
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(text) = resp.text().await {
                    let exploits = parse_rss_simple(&text, cutoff);
                    info!("  Found {} from Rekt News", exploits.len());
                    return Ok(exploits);
                }
            }
            Err(e) => warn!("Failed to fetch Rekt RSS: {}", e),
        }
        Ok(Vec::new())
    }

    pub async fn generate_template(&self, exploits: Vec<Exploit>, output_path: &Path) -> Result<()> {
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
                    id: format!("{}_{}", 
                        exploit.exploit_type.to_string().to_lowercase(),
                        exploit.date.format("%Y%m%d")
                    ),
                    pattern: pattern_template.to_string(),
                    message: format!("{} - {} ({})", 
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
            description: format!("Auto-generated from security feeds (Updated: {})", Utc::now().format("%Y-%m-%d")),
            severity: Severity::Critical,
            tags: vec!["zero-day".to_string(), "live".to_string()],
            patterns,
        };

        let yaml = serde_yaml::to_string(&template)?;
        std::fs::write(output_path, yaml)?;

        info!("Generated template with {} patterns", template.patterns.len());
        Ok(())
    }
}

fn parse_rss_simple(xml: &str, cutoff: &DateTime<Utc>) -> Vec<Exploit> {
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
                            source: "rekt".to_string(),
                            title: title.clone(),
                            date: date_utc,
                            loss_usd: extract_loss(&title),
                            exploit_type: classify_exploit(&title),
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
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    
    let start = content.find(&start_tag)? + start_tag.len();
    let end = content.find(&end_tag)?;
    
    Some(content[start..end].trim().to_string())
}

fn extract_loss(text: &str) -> Option<u64> {
    // Extract loss amounts like "$15M", "$5.2M", "$500K"
    let text_lower = text.to_lowercase();
    
    if let Some(pos) = text_lower.find('$') {
        let after_dollar = &text_lower[pos+1..];
        let num_str: String = after_dollar.chars()
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

fn classify_exploit(content: &str) -> ExploitType {
    let content_lower = content.to_lowercase();
    
    if content_lower.contains("reentrancy") || content_lower.contains("reentrant") {
        ExploitType::Reentrancy
    } else if content_lower.contains("oracle") || content_lower.contains("price manipulation") {
        ExploitType::OracleManipulation
    } else if content_lower.contains("access control") || content_lower.contains("unauthorized") {
        ExploitType::AccessControl
    } else if content_lower.contains("flash loan") || content_lower.contains("flashloan") {
        ExploitType::FlashLoan
    } else {
        ExploitType::Unknown
    }
}

impl ToString for ExploitType {
    fn to_string(&self) -> String {
        match self {
            ExploitType::Reentrancy => "reentrancy".to_string(),
            ExploitType::OracleManipulation => "oracle_manipulation".to_string(),
            ExploitType::AccessControl => "access_control".to_string(),
            ExploitType::FlashLoan => "flash_loan".to_string(),
            ExploitType::Unknown => "unknown".to_string(),
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
