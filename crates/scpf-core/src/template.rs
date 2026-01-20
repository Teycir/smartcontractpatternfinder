use anyhow::{Context, Result};
use scpf_types::Template;
use std::path::Path;
use tokio::fs;

pub struct TemplateLoader;

impl TemplateLoader {
    pub async fn load_from_dir(dir: &Path) -> Result<Vec<Template>> {
        let mut templates = Vec::new();
        let mut entries = fs::read_dir(dir)
            .await
            .context("Failed to read templates directory")?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                let content = fs::read_to_string(&path)
                    .await
                    .context(format!("Failed to read template: {:?}", path))?;
                let template: Template = serde_yaml::from_str(&content)
                    .context(format!("Failed to parse template: {:?}", path))?;
                templates.push(template);
            }
        }

        Ok(templates)
    }
}
