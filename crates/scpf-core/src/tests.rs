use super::*;
use anyhow::Result;
use scpf_types::{ApiKeyConfig, Chain, Pattern, PatternKind, Severity, Template};
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_etherscan_v2_api() {
    if let Ok(key) = std::env::var("ETHERSCAN_API_KEY") {
        let config = ApiKeyConfig::new().with_key(Chain::Ethereum, key);
        
        let fetcher = ContractFetcher::new(config).unwrap();
        let result = fetcher
            .fetch_source("0xdac17f958d2ee523a2206206994597c13d831ec7", Chain::Ethereum)
            .await;
        
        assert!(result.is_ok(), "Etherscan v2 API should work");
        assert!(!result.unwrap().is_empty());
    }
}

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
            kind: PatternKind::Regex,
        }],
        severity: Severity::High,
    }];

    let mut scanner = Scanner::new(templates)?;
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
            kind: PatternKind::Regex,
        }],
        severity: Severity::Medium,
    }];

    let mut scanner = Scanner::new(templates)?;
    let source = "A\n\nB";
    let matches = scanner.scan(source, PathBuf::from("test.sol"))?;

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].line_number, 1);
    Ok(())
}

#[test]
fn test_scanner_line_numbers() -> Result<()> {
    let templates = vec![Template {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: "Test".to_string(),
        tags: vec![],
        patterns: vec![Pattern {
            id: "pattern".to_string(),
            pattern: "test".to_string(),
            message: "Found test".to_string(),
            kind: PatternKind::Regex,
        }],
        severity: Severity::Low,
    }];

    let mut scanner = Scanner::new(templates)?;
    let source = "line1\nline2 test\nline3 test";
    let matches = scanner.scan(source, PathBuf::from("test.sol"))?;

    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].line_number, 2);
    assert_eq!(matches[1].line_number, 3);
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
            kind: PatternKind::Regex,
        }],
        severity: Severity::Low,
    }];

    let mut scanner = Scanner::new(templates)?;
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
            kind: PatternKind::Regex,
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
    let fetcher = ContractFetcher::new(ApiKeyConfig::new()).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let result = rt.block_on(fetcher.fetch_source("invalid", Chain::Ethereum));
    assert!(result.is_err());

    let result = rt.block_on(fetcher.fetch_source("0x123", Chain::Ethereum));
    assert!(result.is_err());
}

#[test]
fn test_fetcher_unsupported_chain() {
    let fetcher = ContractFetcher::new(ApiKeyConfig::new()).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let result = rt.block_on(fetcher.fetch_source(
        "0x1234567890123456789012345678901234567890",
        Chain::Ethereum,
    ));
    assert!(result.is_err());
}

#[test]
fn test_parse_single_file_source() {
    let source = "contract Test {}";
    let parsed = ContractFetcher::parse_source_code(source);
    assert_eq!(parsed, "contract Test {}");
}

#[test]
fn test_parse_multi_file_source() {
    let source = r##"{"sources":{"Contract.sol":{"content":"contract A {}"},"Library.sol":{"content":"library B {}"}}}"##;
    let parsed = ContractFetcher::parse_source_code(source);
    assert!(parsed.contains("// File: Contract.sol"));
    assert!(parsed.contains("contract A {}"));
    assert!(parsed.contains("// File: Library.sol"));
    assert!(parsed.contains("library B {}"));
}

#[test]
fn test_parse_double_braced_json() {
    let source = r##"{{"sources":{"Contract.sol":{"content":"contract A {}"},"Library.sol":{"content":"library B {}"}}}}"##;
    let parsed = ContractFetcher::parse_source_code(source);
    assert!(parsed.contains("// File: Contract.sol"));
    assert!(parsed.contains("contract A {}"));
    assert!(parsed.contains("// File: Library.sol"));
    assert!(parsed.contains("library B {}"));
}

#[test]
fn test_parse_invalid_json_fallback() {
    let source = "{invalid json";
    let parsed = ContractFetcher::parse_source_code(source);
    assert_eq!(parsed, "{invalid json");
}

#[test]
fn test_scanner_deduplication() -> Result<()> {
    let templates = vec![Template {
        id: "template1".to_string(),
        name: "Template 1".to_string(),
        description: "Test".to_string(),
        tags: vec![],
        patterns: vec![
            Pattern {
                id: "pattern1".to_string(),
                pattern: "test".to_string(),
                message: "Found test".to_string(),
                kind: PatternKind::Regex,
            },
            Pattern {
                id: "pattern2".to_string(),
                pattern: "test".to_string(),
                message: "Found test again".to_string(),
                kind: PatternKind::Regex,
            },
        ],
        severity: Severity::Low,
    }];

    let mut scanner = Scanner::new(templates)?;
    let source = "test";
    let matches = scanner.scan(source, PathBuf::from("test.sol"))?;

    assert_eq!(matches.len(), 2, "Different patterns should both match");

    let source_multi = "test test";
    let matches_multi = scanner.scan(source_multi, PathBuf::from("test.sol"))?;
    assert_eq!(
        matches_multi.len(),
        4,
        "Each pattern should match each occurrence"
    );
    Ok(())
}

#[test]
fn test_scanner_large_match_context() -> Result<()> {
    let templates = vec![Template {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: "Test".to_string(),
        tags: vec![],
        patterns: vec![Pattern {
            id: "pattern".to_string(),
            pattern: "A+".to_string(),
            message: "Long match".to_string(),
            kind: PatternKind::Regex,
        }],
        severity: Severity::Low,
    }];

    let mut scanner = Scanner::new(templates)?;
    let long_match = "A".repeat(250);
    let source = format!("prefix {}suffix", long_match);
    let matches = scanner.scan(&source, PathBuf::from("test.sol"))?;

    assert_eq!(matches.len(), 1);
    assert!(
        matches[0].context.len() <= 300,
        "Context should be limited for large matches"
    );
    assert!(
        matches[0].context.contains("prefix"),
        "Should include prefix padding"
    );
    assert!(
        matches[0].context.contains("suffix"),
        "Should include suffix padding"
    );
    Ok(())
}

#[test]
fn debug_dump_ast() {
    let code = r#"
    contract Test {
        uint public lastTime;
        function withdraw(uint amount) public {
            msg.sender.call.value(amount)("");
            lastTime += 10;
        }
        function check() public {
            if (block.timestamp > lastTime) {
                lastTime = now;
            }
            if (tx.origin == msg.sender) {
                // ok
            }
            uint bal = address(this).balance;
        }
    }
    "#;
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_solidity::LANGUAGE.into())
        .unwrap();
    let tree = parser.parse(code, None).unwrap();
    println!("AST: {}", tree.root_node().to_sexp());
}
