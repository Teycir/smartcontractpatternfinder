
#[test]
fn test_scanner_utf8_context() {
    let template = scpf_types::Template {
        id: "test".to_string(),
        patterns: vec![scpf_types::Pattern {
            id: "test-pattern".to_string(),
            kind: scpf_types::PatternKind::Regex,
            pattern: "test".to_string(),
            severity: scpf_types::Severity::High,
            message: "Test".to_string(),
        }],
        severity: scpf_types::Severity::High,
    };
    
    let mut scanner = Scanner::new(vec![template]).unwrap();
    // String with 3-byte char '你' (E4 BD A0) at start
    // "你" is bytes 0,1,2. "test" starts at 3.
    // If we try to get context padding that lands on byte 1 or 2, it should panic if not handled.
    // Let's force a match where padding logic might be tricky.
    // We need CONTEXT_PADDING (50) to reach back into invalid chars.
    
    // Construct string: [20 chars of 3-bytes] + "test"
    // 20 * 3 = 60 bytes. 
    // Match "test" at index 60.
    // Padding 50. Start = 60 - 50 = 10.
    // 10 is not a multiple of 3 (0, 3, 6, 9, 12). So 10 is inside the 4th char (bytes 9,10,11).
    // This should panic.
    let prefix = "你".repeat(20); 
    let source = format!("{}test", prefix);
    
    let findings = scanner.scan(&source, PathBuf::from("test.sol")).unwrap();
    assert_eq!(findings.len(), 1);
}
