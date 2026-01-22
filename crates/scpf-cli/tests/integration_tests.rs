use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_init_command_creates_structure() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    let mut cmd = cargo_bin_cmd!("scpf");
    cmd.arg("init")
        .arg(project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized SCPF project"));

    assert!(project_path.join("templates").exists());
}

#[test]
fn test_templates_list_no_templates() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = cargo_bin_cmd!("scpf");
    cmd.arg("templates")
        .arg("list")
        .arg("--templates")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No templates found"));
}

#[test]
fn test_templates_show_missing_template_fails() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = cargo_bin_cmd!("scpf");
    cmd.arg("templates")
        .arg("show")
        .arg("does-not-exist")
        .arg("--templates")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Template 'does-not-exist' not found"));
}

#[test]
fn test_templates_list_with_one_template() {
    let temp_dir = TempDir::new().unwrap();
    let templates_dir = temp_dir.path().join("templates");
    fs::create_dir_all(&templates_dir).unwrap();

    let template_content = r#"
id: test-template
name: Test Template
description: Test pattern
severity: high
tags:
  - test
patterns:
  - id: test-pattern
    pattern: "function"
    message: "Found function keyword"
"#;
    fs::write(templates_dir.join("test.yaml"), template_content).unwrap();

    let mut cmd = cargo_bin_cmd!("scpf");
    cmd.arg("templates")
        .arg("list")
        .arg("--templates")
        .arg(&templates_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Available templates"))
        .stdout(predicate::str::contains("test-template"));
}
