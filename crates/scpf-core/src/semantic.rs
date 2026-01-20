use anyhow::{Context, Result};
use scpf_types::{Match, Pattern, Severity};
use std::path::PathBuf;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator, Tree};

pub struct SemanticScanner {
    parser: Parser,
}

impl SemanticScanner {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_solidity::LANGUAGE.into())
            .context("Failed to set Solidity language")?;

        Ok(Self { parser })
    }

    pub fn parse(&mut self, source: &str) -> Result<Tree> {
        self.parser
            .parse(source, None)
            .context("Failed to parse source code")
    }

    pub fn scan_with_tree(
        &mut self,
        source: &str,
        tree: &Tree,
        pattern: &Pattern,
        template_id: &str,
        severity: Severity,
        file_path: PathBuf,
    ) -> Result<Vec<Match>> {
        let language = tree_sitter_solidity::LANGUAGE.into();

        // DETAILED ERROR REPORTING
        let query = match Query::new(&language, &pattern.pattern) {
            Ok(q) => q,
            Err(e) => {
                tracing::error!(
                    "Query compilation failed for pattern '{}' in template '{}':\n\
                     Error: {:?}\n\
                     At row: {}, column: {}\n\
                     Offset: {}\n\
                     Query text:\n{}\n\
                     ---",
                    pattern.id,
                    template_id,
                    e.kind,
                    e.row,
                    e.column,
                    e.offset,
                    &pattern.pattern
                );

                let lines: Vec<&str> = pattern.pattern.lines().collect();
                if (e.row as usize) < lines.len() {
                    tracing::error!("Error line: {}", lines[e.row as usize]);
                    tracing::error!("Position:   {}^", " ".repeat(e.column as usize));
                }

                return Err(anyhow::anyhow!(
                    "Query error at {}:{} - {:?}",
                    e.row,
                    e.column,
                    e.kind
                ));
            }
        };

        let mut cursor = QueryCursor::new();
        let mut matches_iter = cursor.matches(&query, tree.root_node(), source.as_bytes());

        let newlines: Vec<usize> = source.match_indices('\n').map(|(i, _)| i).collect();
        let mut results = Vec::new();

        while let Some(query_match) = matches_iter.next() {
            for capture in query_match.captures {
                let node = capture.node;
                let start_byte = node.start_byte();
                let end_byte = node.end_byte();

                let line_number = newlines.partition_point(|&pos| pos < start_byte) + 1;
                let line_start = if line_number > 1 {
                    newlines[line_number - 2] + 1
                } else {
                    0
                };

                let context_start = line_start;
                let context_end = newlines
                    .get(line_number - 1)
                    .copied()
                    .unwrap_or(source.len());
                let context = source[context_start..context_end].to_string();

                let matched_text = &source[start_byte..end_byte];

                results.push(Match {
                    code_snippet: None,
                    template_id: template_id.to_string(),
                    pattern_id: pattern.id.clone(),
                    file_path: file_path.clone(),
                    line_number,
                    column: start_byte - line_start,
                    matched_text: matched_text.to_string(),
                    context,
                    severity,
                    message: pattern.message.clone(),
                    start_byte: Some(start_byte),
                    end_byte: Some(end_byte),
                });
            }
        }

        Ok(results)
    }

    pub fn scan(
        &mut self,
        source: &str,
        pattern: &Pattern,
        template_id: &str,
        severity: Severity,
        file_path: PathBuf,
    ) -> Result<Vec<Match>> {
        let tree = self
            .parser
            .parse(source, None)
            .context("Failed to parse source code")?;

        self.scan_with_tree(source, &tree, pattern, template_id, severity, file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scpf_types::{PatternKind, Severity};

    #[test]
    fn test_semantic_scanner_initialization() {
        let scanner = SemanticScanner::new();
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_reentrancy_detection() {
        let mut scanner = SemanticScanner::new().unwrap();
        let code = r#"contract Test {
    function withdraw() public {
        msg.sender.call{value: balance}("");
        balance = 0;
    }
}"#;

        let pattern = Pattern {
            id: "call-value".to_string(),
            pattern: r#"(call_expression) @call"#.to_string(),
            message: "External call detected".to_string(),
            kind: PatternKind::Semantic,
        };

        let result = scanner.scan(
            code,
            &pattern,
            "reentrancy",
            Severity::High,
            PathBuf::from("test.sol"),
        );

        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let matches = result.unwrap();
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_tx_origin_detection() {
        let mut scanner = SemanticScanner::new().unwrap();
        let code = r#"contract Test {
    function auth() public {
        require(tx.origin == owner);
    }
}"#;

        let pattern = Pattern {
            id: "tx-origin".to_string(),
            pattern: r#"(member_expression
    object: (identifier) @tx
    property: (identifier) @origin
    (#eq? @tx "tx")
    (#eq? @origin "origin")) @usage"#
                .to_string(),
            message: "tx.origin usage detected".to_string(),
            kind: PatternKind::Semantic,
        };

        let result = scanner.scan(
            code,
            &pattern,
            "tx-origin",
            Severity::High,
            PathBuf::from("test.sol"),
        );

        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let matches = result.unwrap();
        assert!(!matches.is_empty());
    }
}
