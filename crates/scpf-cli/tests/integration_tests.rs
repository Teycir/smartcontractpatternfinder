use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_init_command_creates_structure() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    let mut cmd = Command::cargo_bin("scpf").unwrap();
    cmd.arg("init")
        .arg(project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized SCPF project"));

    assert!(project_path.join("templates").exists());
}

#[test]
fn test_scan_command_no_templates() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("scpf").unwrap();
    cmd.arg("scan")
        .arg("0x1234567890123456789012345678901234567890")
        .arg("--templates")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No templates found"));
}

#[test]
fn test_scan_command_invalid_address() {
    let mut cmd = Command::cargo_bin("scpf").unwrap();
    cmd.arg("scan")
        .arg("invalid-address")
        .assert()
        .failure();
}

#[test]
fn test_scan_with_mock_template() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    fs::create_dir_all(&templates_dir).unwrap();

    let template_content = r#"
id: test-template
name: Test Template
description: Test pattern
severity: low
tags:
  - test
patterns:
  - id: test-pattern
    pattern: "function"
    message: "Found function keyword"
"#;
    fs::write(templates_dir.join("test.yaml"), template_content).unwrap();

    let mut cmd = Command::cargo_bin("scpf").unwrap();
    cmd.arg("scan")
        .arg("0x1234567890123456789012345678901234567890")
        .arg("--templates")
        .arg(&templates_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanning"));
}

#[test]
fn test_output_format_json() {
    let mut cmd = Command::cargo_bin("scpf").unwrap();
    cmd.arg("scan")
        .arg("0x1234567890123456789012345678901234567890")
        .arg("--output")
        .arg("json")
        .assert()
        .failure();
}

#[test]
fn test_chain_argument() {
    let mut cmd = Command::cargo_bin("scpf").unwrap();
    cmd.arg("scan")
        .arg("0x1234567890123456789012345678901234567890")
        .arg("--chain")
        .arg("bsc")
        .assert()
        .failure();
}
