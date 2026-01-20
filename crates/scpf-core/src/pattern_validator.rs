use anyhow::{Context, Result};
use tree_sitter::{Query, QueryError};

pub struct PatternValidator {
    language: tree_sitter::Language,
}

impl PatternValidator {
    pub fn new() -> Self {
        Self {
            language: tree_sitter_solidity::LANGUAGE.into(),
        }
    }

    pub fn validate(&self, pattern: &str) -> Result<ValidationResult> {
        match Query::new(&self.language, pattern) {
            Ok(query) => Ok(ValidationResult {
                valid: true,
                capture_names: query
                    .capture_names()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                pattern_count: query.pattern_count(),
                error: None,
            }),
            Err(e) => Ok(ValidationResult {
                valid: false,
                capture_names: vec![],
                pattern_count: 0,
                error: Some(PatternError::from_query_error(&e, pattern)),
            }),
        }
    }

    pub fn validate_template(&self, yaml_content: &str) -> Result<TemplateValidation> {
        let template: TemplateFile =
            serde_yaml::from_str(yaml_content).context("Failed to parse template YAML")?;

        let mut results = Vec::new();

        for pattern in &template.patterns {
            if pattern.kind == "semantic" {
                let result = self.validate(&pattern.pattern)?;
                results.push((pattern.id.clone(), result));
            }
        }

        let passed = results.iter().filter(|(_, r)| r.valid).count();
        let failed = results.iter().filter(|(_, r)| !r.valid).count();

        Ok(TemplateValidation {
            name: template.name,
            total: results.len(),
            passed,
            failed,
            patterns: results,
        })
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub capture_names: Vec<String>,
    pub pattern_count: usize,
    pub error: Option<PatternError>,
}

#[derive(Debug)]
pub struct PatternError {
    pub kind: String,
    pub row: u32,
    pub column: u32,
    pub offset: usize,
    pub context: String,
    pub suggestion: Option<String>,
}

impl PatternError {
    fn from_query_error(e: &QueryError, pattern: &str) -> Self {
        let lines: Vec<&str> = pattern.lines().collect();
        let context = if (e.row as usize) < lines.len() {
            format!(
                "{}\n{}^",
                lines[e.row as usize],
                " ".repeat(e.column as usize)
            )
        } else {
            String::new()
        };

        let suggestion = Self::suggest_fix(e, pattern);

        Self {
            kind: format!("{:?}", e.kind),
            row: e.row as u32,
            column: e.column as u32,
            offset: e.offset,
            context,
            suggestion,
        }
    }

    fn suggest_fix(_e: &QueryError, pattern: &str) -> Option<String> {
        if pattern.contains("function_call") {
            return Some("Use 'call_expression' instead of 'function_call'".to_string());
        }
        if pattern.contains("member_access") {
            return Some("Use 'member_expression' instead of 'member_access'".to_string());
        }
        if pattern.contains("body:") {
            return Some("Remove 'body:' field - use direct nesting instead".to_string());
        }
        if pattern.contains("left:") || pattern.contains("right:") {
            return Some("Remove field names - use positional children".to_string());
        }
        None
    }
}

#[derive(Debug)]
pub struct TemplateValidation {
    pub name: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub patterns: Vec<(String, ValidationResult)>,
}

impl TemplateValidation {
    pub fn print_report(&self) {
        println!("\n=== Template: {} ===", self.name);
        println!(
            "Total: {} | Passed: {} ✅ | Failed: {} ❌\n",
            self.total, self.passed, self.failed
        );

        for (id, result) in &self.patterns {
            if result.valid {
                println!("  ✅ {}", id);
            } else {
                println!("  ❌ {}", id);
                if let Some(ref err) = result.error {
                    println!(
                        "     Error: {} at line {}, col {}",
                        err.kind,
                        err.row + 1,
                        err.column
                    );
                    if let Some(ref suggestion) = err.suggestion {
                        println!("     💡 {}", suggestion);
                    }
                }
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct TemplateFile {
    name: String,
    patterns: Vec<PatternDef>,
}

#[derive(serde::Deserialize)]
struct PatternDef {
    id: String,
    kind: String,
    pattern: String,
}
