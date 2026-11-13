pub mod config;
pub mod content;
pub mod markdown;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "daydream", about = "Minimalist bilingual SSG for simple-website")]
struct Cli {
    /// Path to daydream.toml configuration file
    #[arg(short, long, default_value = "daydream.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build blog pages into output directory
    Build {
        /// Include draft posts in the output
        #[arg(long)]
        drafts: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::Config::load(&cli.config)?;

    match cli.command.unwrap_or(Command::Build { drafts: false }) {
        Command::Build { drafts } => {
            let posts = content::load_all_posts(&cfg.content_dir)?;
            let count = posts.iter().filter(|p| drafts || !p.meta.draft).count();
            println!(
                "Loaded {} post(s) from '{}' (site: {})",
                count,
                cfg.content_dir.display(),
                cfg.site_title
            );
        }
    }

    Ok(())
}
