pub mod config;
pub mod content;
pub mod markdown;
pub mod render;
pub mod site;

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
    }

    Ok(())
}
