use anyhow::Result;
use scpf_core::{Scanner, TemplateLoader};
use scpf_types::{Match, Severity};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::Path;
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> Result<()> {
    let templates = TemplateLoader::load_from_dir(Path::new("templates")).await?;
    let scanner = Scanner::new(templates)?;

    let mut findings = Vec::new();
    for entry in WalkDir::new("benchmarks")
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "sol"))
    {
        let path = entry.path();
        let content = std::fs::read_to_string(path)?;
        findings.extend(scanner.scan(&content, path.to_path_buf())?);
    }

    let sarif = build_sarif(&findings);
    println!("{}", serde_json::to_string_pretty(&sarif)?);
    Ok(())
}

fn build_sarif(findings: &[Match]) -> serde_json::Value {
    let mut rules = BTreeMap::new();
    let results: Vec<_> = findings
        .iter()
        .map(|finding| {
            rules.entry(finding.pattern_id.clone()).or_insert_with(|| {
                json!({
                    "id": finding.pattern_id,
                    "name": finding.pattern_id,
                    "shortDescription": { "text": finding.message },
                    "fullDescription": { "text": format!("Template: {}", finding.template_id) },
                    "defaultConfiguration": { "level": sarif_level(finding.severity) },
                })
            });

            json!({
                "ruleId": finding.pattern_id,
                "level": sarif_level(finding.severity),
                "message": { "text": finding.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": path_uri(&finding.file_path),
                        },
                        "region": {
                            "startLine": finding.line_number,
                            "startColumn": finding.column.max(1),
                        }
                    }
                }]
            })
        })
        .collect();

    json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "SCPF",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/Teycir/smartcontractpatternfinder",
                    "rules": rules.into_values().collect::<Vec<_>>(),
                }
            },
            "results": results,
        }]
    })
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical => "error",
        Severity::High => "warning",
    }
}

fn path_uri(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
