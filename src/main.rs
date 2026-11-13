pub mod config;
pub mod content;
pub mod markdown;
pub mod render;
pub mod site;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::Local;
use clap::{Parser, Subcommand, ValueEnum};

use crate::content::Lang;

#[derive(Debug, Parser)]
#[command(name = "daydream", about = "Minimalist bilingual SSG for simple-website")]
struct Cli {
    /// Path to daydream.toml configuration file
    #[arg(short, long, default_value = "daydream.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum NewLang {
    En,
    PtBr,
    Both,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build blog pages into output directory
    Build {
        /// Include draft posts in the output
        #[arg(long)]
        drafts: bool,
    },
    /// Create a new markdown post skeleton
    New {
        /// Post slug (e.g. writing-an-ssg-in-rust)
        slug: String,
        /// Post title (defaults to capitalized slug)
        #[arg(short, long)]
        title: Option<String>,
        /// Target language(s) to create
        #[arg(short, long, value_enum, default_value_t = NewLang::Both)]
        lang: NewLang,
    },
}

fn create_post_skeleton(
    content_dir: &Path,
    lang: Lang,
    slug: &str,
    title: &str,
    date_str: &str,
) -> Result<PathBuf> {
    let dir = content_dir.join(lang.dir_name());
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create directory '{}'", dir.display()))?;

    let filename = format!("{}-{}.md", date_str, slug);
    let path = dir.join(filename);
    if path.exists() {
        bail!("post file '{}' already exists", path.display());
    }

    let summary_placeholder = match lang {
        Lang::En => "Short summary of the post.",
        Lang::PtBr => "Breve resumo do artigo.",
    };

    let template = format!(
        r#"+++
title = "{title}"
slug = "{slug}"
date = "{date_str}"
summary = "{summary_placeholder}"
tags = []
draft = false
+++

Write your post here...
"#
    );

    fs::write(&path, template)
        .with_context(|| format!("failed to write post skeleton to '{}'", path.display()))?;

    Ok(path)
}

fn humanize_slug(slug: &str) -> String {
    let mut words: Vec<String> = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .map(|w| w.to_string())
        .collect();
    if let Some(first) = words.first_mut() {
        let mut chars = first.chars();
        if let Some(c) = chars.next() {
            *first = c.to_uppercase().collect::<String>() + chars.as_str();
        }
    }
    words.join(" ")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::Config::load(&cli.config)?;

    match cli.command.unwrap_or(Command::Build { drafts: false }) {
        Command::Build { drafts } => {
            let raw_posts = content::load_all_posts(&cfg.content_dir)?;
            let index = site::build_site_index(raw_posts, drafts);
            let renderer = render::Renderer::new(&cfg.templates_dir)?;
            renderer.write_site(&cfg, &index)?;

            println!(
                "Built {} EN post(s) and {} PT-BR post(s) into '{}'",
                index.en_posts.len(),
                index.pt_posts.len(),
                cfg.output_dir.display()
            );
        }
        Command::New { slug, title, lang } => {
            let date_str = Local::now().format("%Y-%m-%d").to_string();
            let title = title.unwrap_or_else(|| humanize_slug(&slug));

            let langs: &[Lang] = match lang {
                NewLang::En => &[Lang::En],
                NewLang::PtBr => &[Lang::PtBr],
                NewLang::Both => &[Lang::En, Lang::PtBr],
            };

            for &l in langs {
                let created =
                    create_post_skeleton(&cfg.content_dir, l, &slug, &title, &date_str)?;
                println!("Created {}", created.display());
            }
        }
    }

    Ok(())
}
