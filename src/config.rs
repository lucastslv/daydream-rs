use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub site_title: String,
    pub author: String,
    pub base_url: String,
    #[serde(default = "default_subtitle_en")]
    pub subtitle_en: String,
    #[serde(default = "default_subtitle_pt")]
    pub subtitle_pt: String,
    #[serde(default = "default_footer_text")]
    pub footer_text: String,
    #[serde(default = "default_content_dir")]
    pub content_dir: PathBuf,
    #[serde(default = "default_templates_dir")]
    pub templates_dir: PathBuf,
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
}

fn default_subtitle_en() -> String {
    "Personal Website & Blog".to_string()
}

fn default_subtitle_pt() -> String {
    "Site Pessoal & Blog".to_string()
}

fn default_footer_text() -> String {
    "Generated with daydream-rs.".to_string()
}

fn default_content_dir() -> PathBuf {
    PathBuf::from("content")
}

fn default_templates_dir() -> PathBuf {
    PathBuf::from("templates")
}

fn default_output_dir() -> PathBuf {
    PathBuf::from(".")
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file at '{}'", path.display()))?;
        let mut config: Self = toml::from_str(&raw)
            .with_context(|| format!("failed to parse TOML config at '{}'", path.display()))?;

        // Resolve relative directories against the directory containing daydream.toml
        if let Some(base_dir) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            if config.content_dir.is_relative() {
                config.content_dir = base_dir.join(&config.content_dir);
            }
            if config.templates_dir.is_relative() {
                config.templates_dir = base_dir.join(&config.templates_dir);
            }
            if config.output_dir.is_relative() {
                config.output_dir = base_dir.join(&config.output_dir);
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let raw = r#"
            site_title = "Example Site"
            author = "Jane Doe"
            base_url = "https://example.com"
        "#;
        let cfg: Config = toml::from_str(raw).unwrap();
        assert_eq!(cfg.site_title, "Example Site");
        assert_eq!(cfg.content_dir, PathBuf::from("content"));
        assert_eq!(cfg.output_dir, PathBuf::from("."));
    }
}
