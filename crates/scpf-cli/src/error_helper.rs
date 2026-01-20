use colored::Colorize;

pub fn format_error_with_help(error: &anyhow::Error) -> String {
    let error_str = error.to_string();

    let (tip, fix) = if error_str.contains("Invalid address format") {
        (
            "Ethereum addresses must start with '0x' and be 42 characters long",
            vec![
                "Example: 0x1234567890123456789012345678901234567890",
                "Verify address at: https://etherscan.io",
            ],
        )
    } else if error_str.contains("API error") || error_str.contains("rate limit") {
        (
            "API key required or rate limit exceeded",
            vec![
                "Set key: export ETHERSCAN_API_KEY=your_key",
                "Or run: scpf init (interactive setup)",
                "Get keys: https://etherscan.io/apis",
            ],
        )
    } else if error_str.contains("No templates found") {
        (
            "Templates directory is empty or missing",
            vec![
                "Initialize: scpf init",
                "Or specify: scpf scan --templates ./my-templates",
                "Docs: https://github.com/Teycir/smartcontractpatternfinder#templates",
            ],
        )
    } else if error_str.contains("Invalid regex") {
        (
            "Template contains invalid regex pattern",
            vec![
                "Test regex: https://regex101.com",
                "Docs: https://docs.rs/regex",
                "Check template YAML syntax",
            ],
        )
    } else if error_str.contains("Failed to fetch") || error_str.contains("network") {
        (
            "Network connection issue",
            vec![
                "Check internet connection",
                "Retry with: --concurrency 1",
                "Check API status: https://etherscan.io/apis",
            ],
        )
    } else {
        return format!("{} {}", "✗".red().bold(), error_str);
    };

    let mut output = format!("{} {}\n", "✗".red().bold(), error_str);
    output.push_str(&format!("\n{} {}\n", "💡".yellow(), tip.yellow()));
    output.push_str(&format!("{}\n", "Fix:".yellow().bold()));
    for (i, step) in fix.iter().enumerate() {
        output.push_str(&format!("   {}. {}\n", i + 1, step.yellow()));
    }

    output
}
