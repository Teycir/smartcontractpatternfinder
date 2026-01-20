use std::fs;
use std::io::{self, Write};

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub line: usize,
    pub column: usize,
    pub text_preview: String,
}

pub struct PatternBuilder {
    pattern: String,
    test_code: String,
    error: Option<String>,
}

impl PatternBuilder {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            test_code: String::new(),
            error: None,
        }
    }

    pub fn load_test_code(&mut self, file: &str) -> io::Result<()> {
        self.test_code = fs::read_to_string(file)?;
        Ok(())
    }

    pub fn set_pattern(&mut self, pattern: &str) {
        self.pattern = pattern.to_string();
    }

    pub fn run_interactive(&mut self) -> io::Result<()> {
        println!("🔍 SCPF Pattern Builder");
        println!("======================\n");

        loop {
            println!("\nOptions:");
            println!("  1. Enter/Edit pattern");
            println!("  2. Load test code from file");
            println!("  3. Save pattern to template");
            println!("  4. Exit");
            print!("\nChoice: ");
            io::stdout().flush()?;

            let mut choice = String::new();
            io::stdin().read_line(&mut choice)?;

            match choice.trim() {
                "1" => self.edit_pattern()?,
                "2" => self.load_code_interactive()?,
                "3" => self.save_pattern()?,
                "4" => break,
                _ => println!("Invalid choice"),
            }
        }

        Ok(())
    }

    fn edit_pattern(&mut self) -> io::Result<()> {
        println!("\nCurrent pattern:");
        if !self.pattern.is_empty() {
            println!("{}", self.pattern);
        } else {
            println!("(empty)");
        }

        println!("\nEnter new pattern (or press Enter to keep current):");
        println!("Example: (function_definition name: (identifier) @name)");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().is_empty() {
            self.pattern = input.trim().to_string();
            println!("✓ Pattern updated");
        }

        Ok(())
    }

    fn load_code_interactive(&mut self) -> io::Result<()> {
        print!("\nEnter path to Solidity file: ");
        io::stdout().flush()?;

        let mut path = String::new();
        io::stdin().read_line(&mut path)?;

        match self.load_test_code(path.trim()) {
            Ok(_) => println!("✓ Test code loaded ({} bytes)", self.test_code.len()),
            Err(e) => println!("✗ Error loading file: {}", e),
        }

        Ok(())
    }

    fn save_pattern(&self) -> io::Result<()> {
        if self.pattern.is_empty() {
            println!("✗ No pattern to save");
            return Ok(());
        }

        print!("\nEnter template ID: ");
        io::stdout().flush()?;
        let mut id = String::new();
        io::stdin().read_line(&mut id)?;

        print!("Enter pattern ID: ");
        io::stdout().flush()?;
        let mut pattern_id = String::new();
        io::stdin().read_line(&mut pattern_id)?;

        print!("Enter message: ");
        io::stdout().flush()?;
        let mut message = String::new();
        io::stdin().read_line(&mut message)?;

        let yaml = format!(
            r#"  - id: {}
    kind: semantic
    pattern: |
{}
    message: "{}"
"#,
            pattern_id.trim(),
            self.pattern
                .lines()
                .map(|l| format!("      {}", l))
                .collect::<Vec<_>>()
                .join("\n"),
            message.trim()
        );

        let filename = format!("templates/{}.yaml", id.trim());
        println!("\n📝 Pattern YAML:\n{}", yaml);
        println!("\nSave to {}? (y/n): ", filename);
        io::stdout().flush()?;

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;

        if confirm.trim().eq_ignore_ascii_case("y") {
            fs::write(&filename, yaml)?;
            println!("✓ Saved to {}", filename);
        } else {
            println!("✗ Not saved");
        }

        Ok(())
    }
}

impl Default for PatternBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn cmd_pattern_builder(
    test_file: Option<&str>,
    initial_pattern: Option<&str>,
) -> io::Result<()> {
    let mut builder = PatternBuilder::new();

    if let Some(file) = test_file {
        builder.load_test_code(file)?;
        println!("✓ Loaded test code from {}", file);
    }

    if let Some(pattern) = initial_pattern {
        builder.set_pattern(pattern);
        println!("✓ Set initial pattern");
    }

    builder.run_interactive()
}
