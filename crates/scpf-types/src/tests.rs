use super::*;

#[test]
fn test_severity_ordering() {
    assert!(Severity::Critical > Severity::High);
    assert!(Severity::High > Severity::Medium);
    assert!(Severity::Medium > Severity::Low);
    assert!(Severity::Low > Severity::Info);
}

#[test]
fn test_severity_serialization() {
    let severity = Severity::High;
    let json = serde_json::to_string(&severity).unwrap();
    assert_eq!(json, "\"high\"");

    let deserialized: Severity = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, Severity::High);
}

#[test]
fn test_template_deserialization() {
    let yaml = r#"
id: test
name: Test Template
description: Test description
severity: high
tags:
  - security
patterns:
  - id: p1
    pattern: "test"
    message: "Test message"
"#;

    let template: Template = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(template.id, "test");
    assert_eq!(template.severity, Severity::High);
    assert_eq!(template.patterns.len(), 1);
}

#[test]
fn test_config_default() {
    let config = Config::default();
    assert_eq!(config.concurrency, 10);
    assert_eq!(config.timeout_secs, 30);
}

#[test]
fn test_match_creation() {
    let m = Match {
        template_id: "t1".to_string(),
        pattern_id: "p1".to_string(),
        file_path: PathBuf::from("test.sol"),
        line_number: 42,
        column: 10,
        matched_text: "test".to_string(),
        context: "context".to_string(),
        severity: Severity::Medium,
        message: "Test message".to_string(),
        start_byte: None,
        end_byte: None,
        code_snippet: None,
    };

    assert_eq!(m.line_number, 42);
    assert_eq!(m.severity, Severity::Medium);
}

#[test]
fn test_scan_result_serialization() {
    let result = ScanResult {
        address: "0x1234567890123456789012345678901234567890".to_string(),
        chain: "ethereum".to_string(),
        matches: vec![],
        scan_time_ms: 100,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("0x1234567890123456789012345678901234567890"));
    assert!(json.contains("ethereum"));
}
