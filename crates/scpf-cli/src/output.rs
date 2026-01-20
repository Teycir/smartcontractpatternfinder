use anyhow::Result;
use scpf_types::ScanResult;

pub fn format_json(results: &[ScanResult]) -> Result<String> {
    Ok(serde_json::to_string_pretty(results)?)
}

pub fn format_sarif(results: &[ScanResult]) -> Result<String> {
    let runs: Vec<_> = results
        .iter()
        .map(|result| {
            let sarif_results: Vec<_> = result
                .matches
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "ruleId": m.template_id,
                        "level": match m.severity {
                            scpf_types::Severity::Critical | scpf_types::Severity::High => "error",
                            scpf_types::Severity::Medium => "warning",
                            _ => "note",
                        },
                        "message": {
                            "text": m.message
                        },
                        "locations": [{
                            "physicalLocation": {
                                "artifactLocation": {
                                    "uri": result.address
                                },
                                "region": {
                                    "startLine": m.line_number,
                                    "startColumn": m.column
                                }
                            }
                        }]
                    })
                })
                .collect();

            serde_json::json!({
                "tool": {
                    "driver": {
                        "name": "SCPF",
                        "version": "0.1.0"
                    }
                },
                "results": sarif_results
            })
        })
        .collect();

    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": runs
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}
