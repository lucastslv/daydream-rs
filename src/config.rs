use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub site_title: String,
    pub author: String,
    pub base_url: String,
    #[serde(default = "default_content_dir")]
    pub content_dir: PathBuf,
    #[serde(default = "default_templates_dir")]
    pub templates_dir: PathBuf,
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
}

fn default_content_dir() -> PathBuf {
    PathBuf::from("content")
}

fn default_templates_dir() -> PathBuf {
    PathBuf::from("templates")
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("../simple-website")
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file at '{}'", path.display()))?;
        let config: Self = toml::from_str(&raw)
            .with_context(|| format!("failed to parse TOML config at '{}'", path.display()))?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let raw = r#"
            site_title = "Lucas Telles da Silva"
            author = "Lucas Telles da Silva"
            base_url = "https://lucastslv.github.io"
        "#;
        let cfg: Config = toml::from_str(raw).unwrap();
        assert_eq!(cfg.site_title, "Lucas Telles da Silva");
        assert_eq!(cfg.content_dir, PathBuf::from("content"));
        assert_eq!(cfg.output_dir, PathBuf::from("../simple-website"));
    }
}
