use anyhow::Result;
use colored::Colorize;
use scpf_core::TemplateLoader;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Registry {
    collections: std::collections::HashMap<String, Collection>,
    #[serde(default)]
    aliases: std::collections::HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Collection {
    description: String,
    templates: Vec<String>,
}

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

pub async fn registry() -> Result<()> {
    let registry_path = PathBuf::from("registry.yaml");
    if !registry_path.exists() {
        anyhow::bail!("Registry file not found. Run 'scpf init' first.");
    }

    let content = tokio::fs::read_to_string(&registry_path).await?;
    let registry: Registry = serde_yaml::from_str(&content)?;

    println!("{} Template Collections:\n", "📦".cyan());

    for (name, collection) in &registry.collections {
        println!("  {} {}", "•".cyan(), name.bold());
        println!("    {}", collection.description.dimmed());
        println!("    Templates: {}", collection.templates.len());
        println!();
    }

    println!("\n{} Aliases:\n", "🔗".cyan());
    for (alias, collections) in &registry.aliases {
        println!("  {} {} → {}", "•".cyan(), alias.bold(), collections.join(", "));
    }

    println!("\n{} Install: scpf templates install <collection>", "→".cyan());
    Ok(())
}

pub async fn install(collection: &str, templates_dir: Option<PathBuf>) -> Result<()> {
    let dir = templates_dir.unwrap_or_else(|| PathBuf::from("templates"));
    
    let registry_path = PathBuf::from("registry.yaml");
    if !registry_path.exists() {
        anyhow::bail!("Registry file not found. Run 'scpf init' first.");
    }

    let content = tokio::fs::read_to_string(&registry_path).await?;
    let registry: Registry = serde_yaml::from_str(&content)?;

    let collections_to_install = if let Some(alias_collections) = registry.aliases.get(collection) {
        alias_collections.clone()
    } else if registry.collections.contains_key(collection) {
        vec![collection.to_string()]
    } else {
        anyhow::bail!("Collection '{}' not found. Run 'scpf templates registry' to see available collections.", collection);
    };

    println!("{}  Installing {} collection(s)...", "📦".cyan(), collections_to_install.len());

    for coll_name in &collections_to_install {
        let coll = registry.collections.get(coll_name)
            .ok_or_else(|| anyhow::anyhow!("Collection '{}' not found", coll_name))?;
        
        println!("\n{}  {} ({} templates)", "→".cyan(), coll_name.bold(), coll.templates.len());
        
        for template_name in &coll.templates {
            let template_path = dir.join(template_name);
            if template_path.exists() {
                println!("  {}  {} (already exists)", "✓".green(), template_name);
            } else {
                println!("  {}  {} (would download)", "⬇".yellow(), template_name);
            }
        }
    }

    println!("\n{}  Note: Template download from remote registries coming soon!", "ℹ".blue());
    println!("{}  Currently using local templates from {:?}", "→".cyan(), dir);
    
    Ok(())
}

pub async fn update(templates_dir: Option<PathBuf>) -> Result<()> {
    let dir = templates_dir.unwrap_or_else(|| PathBuf::from("templates"));
    
    println!("{}  Checking for template updates...", "🔄".cyan());
    
    let templates = TemplateLoader::load_from_dir(&dir).await?;
    println!("{}  Found {} local templates", "✓".green(), templates.len());
    
    println!("\n{}  Note: Automatic updates from remote registries coming soon!", "ℹ".blue());
    println!("{}  For now, manually update templates in {:?}", "→".cyan(), dir);
    
    Ok(())
}
