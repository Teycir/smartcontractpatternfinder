use super::*;
use std::path::PathBuf;
use tempfile::tempdir;
use scpf_types::{Pattern, Template, Severity};
use anyhow::Result;

#[tokio::test]
async fn test_cache_operations() -> Result<()> {
    let dir = tempdir()?;
    let cache = Cache::new(dir.path().to_path_buf()).await?;
    
    cache.set("test-key", "test-value").await?;
    let value = cache.get("test-key").await;
    assert_eq!(value, Some("test-value".to_string()));
    
    let missing = cache.get("non-existent").await;
    assert!(missing.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_cache_atomic_write() -> Result<()> {
    let dir = tempdir()?;
    let cache = Cache::new(dir.path().to_path_buf()).await?;
    
    cache.set("key1", "value1").await?;
    cache.set("key1", "value2").await?;
    
    let value = cache.get("key1").await;
    assert_eq!(value, Some("value2".to_string()));
    
    Ok(())
}

#[test]
fn test_scanner_basic_match() -> Result<()> {
    let templates = vec![Template {
        id: "test-template".to_string(),
        name: "Test Template".to_string(),
        description: "A test template".to_string(),
        tags: vec!["test".to_string()],
        patterns: vec![Pattern {
            id: "test-pattern".to_string(),
            pattern: "eval\\(".to_string(),
            message: "Avoid eval".to_string(),
        }],
        severity: Severity::High,
    }];
    
    let scanner = Scanner::new(templates)?;
    let source = "function test() { eval(input); }";
    let matches = scanner.scan(source, PathBuf::from("test.sol"))?;
    
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern_id, "test-pattern");
    assert_eq!(matches[0].line_number, 1);
    
    Ok(())
}

#[test]
fn test_scanner_multiline() -> Result<()> {
    let templates = vec![Template {
        id: "multiline".to_string(),
        name: "Multiline Template".to_string(),
        description: "A multiline template".to_string(),
        tags: vec!["test".to_string()],
        patterns: vec![Pattern {
            id: "multi".to_string(),
            pattern: "A.*B".to_string(),
            message: "Multiline match".to_string(),
        }],
        severity: Severity::Medium,
    }];
    
    let scanner = Scanner::new(templates)?;
    let source = "A\n\nB";
    let matches = scanner.scan(source, PathBuf::from("test.sol"))?;
    
    assert_eq!(matches.len(), 1);
    Ok(())
}

#[test]
fn test_scanner_no_match() -> Result<()> {
    let templates = vec![Template {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: "Test".to_string(),
        tags: vec![],
        patterns: vec![Pattern {
            id: "pattern".to_string(),
            pattern: "nonexistent".to_string(),
            message: "Should not match".to_string(),
        }],
        severity: Severity::Low,
    }];
    
    let scanner = Scanner::new(templates)?;
    let source = "function test() { return 42; }";
    let matches = scanner.scan(source, PathBuf::from("test.sol"))?;
    
    assert_eq!(matches.len(), 0);
    Ok(())
}

#[test]
fn test_scanner_invalid_regex() {
    let templates = vec![Template {
        id: "invalid".to_string(),
        name: "Invalid".to_string(),
        description: "Invalid regex".to_string(),
        tags: vec![],
        patterns: vec![Pattern {
            id: "bad".to_string(),
            pattern: "[".to_string(),
            message: "Invalid".to_string(),
        }],
        severity: Severity::Low,
    }];
    
    let result = Scanner::new(templates);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_template_loader() -> Result<()> {
    let dir = tempdir()?;
    let template_path = dir.path().join("test.yaml");
    
    let yaml = r#"
id: test
name: Test Template
description: Test
severity: high
tags:
  - test
patterns:
  - id: p1
    pattern: "test"
    message: "Test pattern"
"#;
    
    tokio::fs::write(&template_path, yaml).await?;
    
    let templates = TemplateLoader::load_from_dir(dir.path()).await?;
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].id, "test");
    
    Ok(())
}

#[test]
fn test_fetcher_invalid_address() {
    let fetcher = ContractFetcher::new(None).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let result = rt.block_on(fetcher.fetch_source("invalid", "ethereum"));
    assert!(result.is_err());
    
    let result = rt.block_on(fetcher.fetch_source("0x123", "ethereum"));
    assert!(result.is_err());
}

#[test]
fn test_fetcher_unsupported_chain() {
    let fetcher = ContractFetcher::new(None).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let result = rt.block_on(fetcher.fetch_source("0x1234567890123456789012345678901234567890", "unsupported"));
    assert!(result.is_err());
}
