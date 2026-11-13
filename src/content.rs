use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::markdown::render_markdown;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Lang {
    En,
    PtBr,
}

impl Lang {
    pub fn code(&self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::PtBr => "pt-BR",
        }
    }

    pub fn dir_name(&self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::PtBr => "pt-br",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Frontmatter {
    pub title: String,
    pub slug: String,
    pub date: NaiveDate,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub draft: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RawPost {
    pub meta: Frontmatter,
    pub lang: Lang,
    pub source_path: PathBuf,
    pub html_body: String,
    pub reading_time_min: usize,
}

pub fn split_frontmatter(input: &str) -> Result<(&str, &str)> {
    let trimmed = input.trim_start();
    let Some(rest) = trimmed.strip_prefix("+++") else {
        bail!("missing opening '+++' TOML frontmatter delimiter");
    };

    let Some(end_idx) = rest.find("\n+++") else {
        bail!("missing closing '+++' TOML frontmatter delimiter");
    };

    let fm_str = rest[..end_idx].trim();
    let after_delim = &rest[end_idx + 4..];
    let body = after_delim.strip_prefix('\r').unwrap_or(after_delim);
    let body = body.strip_prefix('\n').unwrap_or(body);

    Ok((fm_str, body))
}

pub fn parse_post_str(input: &str, lang: Lang, source_path: PathBuf) -> Result<RawPost> {
    let (fm_str, markdown_body) = split_frontmatter(input)
        .with_context(|| format!("invalid frontmatter in '{}'", source_path.display()))?;

    let meta: Frontmatter = toml::from_str(fm_str)
        .with_context(|| format!("failed to parse TOML frontmatter in '{}'", source_path.display()))?;

    let rendered = render_markdown(markdown_body);

    Ok(RawPost {
        meta,
        lang,
        source_path,
        html_body: rendered.html,
        reading_time_min: rendered.reading_time_min,
    })
}

pub fn load_post_file(path: impl AsRef<Path>, lang: Lang) -> Result<RawPost> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read post file '{}'", path.display()))?;
    parse_post_str(&raw, lang, path.to_path_buf())
}

pub fn load_all_posts(content_dir: impl AsRef<Path>) -> Result<Vec<RawPost>> {
    let content_dir = content_dir.as_ref();
    let mut posts = Vec::new();

    for lang in [Lang::En, Lang::PtBr] {
        let lang_dir = content_dir.join(lang.dir_name());
        if !lang_dir.exists() {
            continue;
        }
        let entries = fs::read_dir(&lang_dir)
            .with_context(|| format!("failed to read content directory '{}'", lang_dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                let post = load_post_file(&path, lang)?;
                posts.push(post);
            }
        }
    }

    Ok(posts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_post_frontmatter_and_body() {
        let raw = r#"+++
title = "Writing an SSG in Rust"
slug = "writing-an-ssg-in-rust"
date = "2026-11-12"
summary = "Minimalist static site generator."
tags = ["rust", "ssg"]
draft = false
+++

Hello **world**!
"#;
        let post = parse_post_str(raw, Lang::En, PathBuf::from("test.md")).unwrap();
        assert_eq!(post.meta.title, "Writing an SSG in Rust");
        assert_eq!(post.meta.slug, "writing-an-ssg-in-rust");
        assert_eq!(
            post.meta.date,
            NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()
        );
        assert_eq!(post.meta.tags, vec!["rust", "ssg"]);
        assert!(!post.meta.draft);
        assert!(post.html_body.contains("<p>Hello <strong>world</strong>!</p>"));
    }
}
