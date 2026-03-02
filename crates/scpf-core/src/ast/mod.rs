mod validators;

pub use validators::{AstValidator, ValidationResult};

use solang_parser::parse;

pub struct AstAnalyzer {
    validator: AstValidator,
}

impl AstAnalyzer {
    pub fn new() -> Self {
        Self {
            validator: AstValidator::new(),
        }
    }

    pub fn validate(&self, source: &str, pattern_id: &str, line: usize) -> ValidationResult {
        let ast = match parse(source, 0) {
            Ok((ast, _)) => ast,
            Err(_) => return ValidationResult::ParseError,
        };

        self.validator.validate(&ast, pattern_id, line)
    }
}

impl Default for AstAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
