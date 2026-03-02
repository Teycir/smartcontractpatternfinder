use anyhow::{Context, Result};
use scpf_types::Template;
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;
use crate::limits::LoaderLimits;

pub struct TemplateLoader;

impl TemplateLoader {
    pub async fn load_from_dir(dir: &Path) -> Result<Vec<Template>> {
        Self::load_from_dir_with_limits(dir, LoaderLimits::default()).await
    }

    pub async fn load_from_dir_with_limits(dir: &Path, limits: LoaderLimits) -> Result<Vec<Template>> {
        let mut templates = Vec::new();
        let mut seen_ids = HashSet::new();
        let mut entries = fs::read_dir(dir)
            .await
            .context("Failed to read templates directory")?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                // Check file size limit
                let metadata = fs::metadata(&path).await?;
                if metadata.len() > limits.max_file_size {
                    anyhow::bail!(
                        "Template file too large: {} bytes (max {}): {:?}",
                        metadata.len(),
                        limits.max_file_size,
                        path
                    );
                }

                let content = fs::read_to_string(&path)
                    .await
                    .context(format!("Failed to read template: {:?}", path))?;

                // Check line count limit (prevent YAML bombs)
                let line_count = content.lines().count();
                if line_count > limits.max_lines {
                    anyhow::bail!(
                        "Template file too complex: {} lines (max {}): {:?}",
                        line_count,
                        limits.max_lines,
                        path
                    );
                }

                let template: Template = serde_yaml::from_str(&content)
                    .context(format!("Failed to parse template: {:?}", path))?;

                // Check for duplicate template IDs
                if !seen_ids.insert(template.id.clone()) {
                    anyhow::bail!("Duplicate template id detected: {}", template.id);
                }

                templates.push(template);

                // Check template count limit
                if templates.len() > limits.max_templates {
                    anyhow::bail!(
                        "Too many templates: {} (max {})",
                        templates.len(),
                        limits.max_templates
                    );
                }
            }
        }

        Ok(templates)
    }
}
