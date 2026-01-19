use anyhow::{Context, Result};
use scpf_types::Template;
use std::fs;
use std::path::Path;

pub struct TemplateLoader;

impl TemplateLoader {
    pub fn load_from_dir(dir: &Path) -> Result<Vec<Template>> {
        let mut templates = Vec::new();

        for entry in fs::read_dir(dir).context("Failed to read templates directory")? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                let content = fs::read_to_string(&path)
                    .context(format!("Failed to read template: {:?}", path))?;
                let template: Template = serde_yaml::from_str(&content)
                    .context(format!("Failed to parse template: {:?}", path))?;
                templates.push(template);
            }
        }

        Ok(templates)
    }
}
