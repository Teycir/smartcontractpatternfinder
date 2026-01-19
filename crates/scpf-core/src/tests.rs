use super::*;
use std::path::PathBuf;
use tempfile::tempdir;
use scpf_types::{Pattern, Template, Severity};
use anyhow::Result;

#[tokio::test]
async fn test_cache_operations() -> Result<()> {
    let dir = tempdir()?;
    let cache = Cache::new(dir.path().to_path_buf()).await?;
    
    // Test set and get
    cache.set("test-key", "test-value").await?;
    let value = cache.get("test-key").await;
    assert_eq!(value, Some("test-value".to_string()));
    
    // Test miss
    let missing = cache.get("non-existent").await;
    assert!(missing.is_none());
    
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
