use anyhow::Result;
use scpf_types::{Match, ScanResult};

pub fn format_json(results: &[ScanResult]) -> Result<String> {
    Ok(serde_json::to_string_pretty(results)?)
}

pub fn format_sarif(results: &[ScanResult]) -> Result<String> {
    let all_findings: Vec<&Match> = results.iter().flat_map(|r| r.matches.iter()).collect();
    export_sarif(&all_findings)
}

use scpf_types::Severity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SarifReport {
    version: String,
    #[serde(rename = "$schema")]
    schema: String,
    runs: Vec<SarifRun>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifDriver {
    name: String,
    version: String,
    #[serde(rename = "informationUri")]
    information_uri: String,
    rules: Vec<SarifRule>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifRule {
    id: String,
    #[serde(rename = "shortDescription")]
    short_description: SarifMessage,
    #[serde(rename = "defaultConfiguration")]
    default_configuration: SarifConfiguration,
    properties: SarifRuleProperties,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifMessage {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifConfiguration {
    level: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifRuleProperties {
    tags: Vec<String>,
    #[serde(rename = "security-severity")]
    security_severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: usize,
}

fn export_sarif(findings: &[&Match]) -> Result<String> {
    let mut unique_rules = std::collections::HashMap::new();

    for finding in findings {
        unique_rules
            .entry(finding.pattern_id.clone())
            .or_insert_with(|| (finding.severity, finding.message.clone()));
    }

    let rules: Vec<SarifRule> = unique_rules
        .into_iter()
        .map(|(id, (severity, message))| SarifRule {
            id: id.clone(),
            short_description: SarifMessage { text: message },
            default_configuration: SarifConfiguration {
                level: severity_to_sarif_level(&severity),
            },
            properties: SarifRuleProperties {
                tags: vec!["security".to_string()],
                security_severity: severity_to_score(&severity),
            },
        })
        .collect();

    let results: Vec<SarifResult> = findings
        .iter()
        .map(|finding| SarifResult {
            rule_id: finding.pattern_id.clone(),
            level: severity_to_sarif_level(&finding.severity),
            message: SarifMessage {
                text: finding.message.clone(),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: finding.file_path.to_string_lossy().to_string(),
                    },
                    region: SarifRegion {
                        start_line: finding.line_number,
                    },
                },
            }],
        })
        .collect();

    let report = SarifReport {
        version: "2.1.0".to_string(),
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json".to_string(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "SCPF".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    information_uri: "https://github.com/Teycir/smartcontractpatternfinder".to_string(),
                    rules,
                },
            },
            results,
        }],
    };

    Ok(serde_json::to_string_pretty(&report)?)
}

fn severity_to_sarif_level(severity: &Severity) -> String {
    match severity {
        Severity::Critical | Severity::High => "error".to_string(),
        Severity::Medium => "warning".to_string(),
        Severity::Low | Severity::Info => "note".to_string(),
    }
}

fn severity_to_score(severity: &Severity) -> String {
    match severity {
        Severity::Critical => "10.0".to_string(),
        Severity::High => "8.0".to_string(),
        Severity::Medium => "5.0".to_string(),
        Severity::Low => "3.0".to_string(),
        Severity::Info => "1.0".to_string(),
    }
}
