use anyhow::Result;
use colored::Colorize;
use scpf_core::TemplateLoader;
use std::path::PathBuf;

pub async fn list(templates_dir: Option<PathBuf>) -> Result<()> {
    let dir = templates_dir.unwrap_or_else(|| PathBuf::from("templates"));
    let templates = TemplateLoader::load_from_dir(&dir).await?;

    if templates.is_empty() {
        println!("{} No templates found in {:?}", "!".yellow(), dir);
        println!("\n{} Run: scpf init", "→".cyan());
        return Ok(());
    }

    println!("{} Available templates:\n", "📋".cyan());

    for template in &templates {
        let severity_str = match template.severity {
            scpf_types::Severity::Critical => "CRITICAL".red().bold(),
            scpf_types::Severity::High => "HIGH".red(),
            scpf_types::Severity::Medium => "MEDIUM".yellow(),
            scpf_types::Severity::Low => "LOW".blue(),
            scpf_types::Severity::Info => "INFO".cyan(),
        };

        println!("  {} [{}]", template.id.bold(), severity_str);
        println!("    {}", template.description.dimmed());
        println!(
            "    Patterns: {} | Tags: {}",
            template.patterns.len(),
            template.tags.join(", ")
        );
        println!();
    }

    println!("Total: {} templates", templates.len());
    println!("\n{} View details: scpf templates show <id>", "→".cyan());

    Ok(())
}

pub async fn show(id: &str, templates_dir: Option<PathBuf>) -> Result<()> {
    let dir = templates_dir.unwrap_or_else(|| PathBuf::from("templates"));
    let templates = TemplateLoader::load_from_dir(&dir).await?;

    let template = templates
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", id))?;

    println!("{}", "═".repeat(60).cyan());
    println!("{} {}", "📄".cyan(), template.name.bold());
    println!("{}", "═".repeat(60).cyan());

    let severity_str = match template.severity {
        scpf_types::Severity::Critical => "CRITICAL".red().bold(),
        scpf_types::Severity::High => "HIGH".red(),
        scpf_types::Severity::Medium => "MEDIUM".yellow(),
        scpf_types::Severity::Low => "LOW".blue(),
        scpf_types::Severity::Info => "INFO".cyan(),
    };

    println!("\n{}: {}", "ID".bold(), template.id);
    println!("{}: {}", "Severity".bold(), severity_str);
    println!("{}: {}", "Description".bold(), template.description);
    println!("{}: {}", "Tags".bold(), template.tags.join(", "));

    println!("\n{} Patterns ({}):", "🔍".cyan(), template.patterns.len());
    for (i, pattern) in template.patterns.iter().enumerate() {
        println!(
            "\n  {}. {} ({})",
            i + 1,
            pattern.id.bold(),
            "Pattern".dimmed()
        );
        println!("     Message: {}", pattern.message);
        println!("     Regex: {}", pattern.pattern.dimmed());
    }

    println!("\n{}", "═".repeat(60).cyan());

    Ok(())
}
