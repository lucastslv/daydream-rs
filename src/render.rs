use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use minijinja::{Environment, context};

use crate::config::Config;
use crate::content::Lang;
use crate::site::{Post, SiteIndex};

pub struct Renderer {
    env: Environment<'static>,
}

impl Renderer {
    pub fn new(templates_dir: impl AsRef<Path>) -> Result<Self> {
        let templates_dir = templates_dir.as_ref();
        let mut env = Environment::new();

        for name in ["base.html", "blog_index.html", "post.html"] {
            let path = templates_dir.join(name);
            let source = fs::read_to_string(&path)
                .with_context(|| format!("failed to read template '{}'", path.display()))?;
            env.add_template_owned(name.to_string(), source)
                .with_context(|| format!("failed to compile template '{}'", path.display()))?;
        }

        Ok(Self { env })
    }

    pub fn render_blog_index(&self, cfg: &Config, lang: Lang, posts: &[Post]) -> Result<String> {
        let tmpl = self.env.get_template("blog_index.html")?;
        let is_pt = lang == Lang::PtBr;

        let (root_prefix, lang_prefix, language_switch_href, hreflang_en, hreflang_pt) = match lang
        {
            Lang::En => ("", "", "pt-br/blog.html", "blog.html", "pt-br/blog.html"),
            Lang::PtBr => ("../", "", "../blog.html", "../blog.html", "blog.html"),
        };

        let page_title = format!("Blog - {}", cfg.site_title);

        let html = tmpl.render(context! {
            lang_code => lang.code(),
            is_pt => is_pt,
            page_title => page_title,
            root_prefix => root_prefix,
            lang_prefix => lang_prefix,
            language_switch_href => language_switch_href,
            hreflang_en => hreflang_en,
            hreflang_pt => hreflang_pt,
            posts => posts,
        })?;

        Ok(html)
    }

    pub fn render_post(&self, cfg: &Config, post: &Post) -> Result<String> {
        let tmpl = self.env.get_template("post.html")?;
        let is_pt = post.lang == Lang::PtBr;
        let page_title = format!("{} - {}", post.meta.title, cfg.site_title);

        let html = tmpl.render(context! {
            lang_code => post.lang.code(),
            is_pt => is_pt,
            page_title => page_title,
            root_prefix => &post.root_prefix,
            lang_prefix => &post.lang_prefix,
            language_switch_href => &post.language_switch_href,
            hreflang_en => &post.hreflang_en,
            hreflang_pt => &post.hreflang_pt,
            post => post,
        })?;

        Ok(html)
    }

    pub fn write_site(&self, cfg: &Config, index: &SiteIndex) -> Result<()> {
        let out_dir = &cfg.output_dir;
        let en_blog_dir = out_dir.join("blog");
        let pt_dir = out_dir.join("pt-br");
        let pt_blog_dir = pt_dir.join("blog");

        fs::create_dir_all(&en_blog_dir)
            .with_context(|| format!("failed to create directory '{}'", en_blog_dir.display()))?;
        fs::create_dir_all(&pt_blog_dir)
            .with_context(|| format!("failed to create directory '{}'", pt_blog_dir.display()))?;

        // 1. Render EN blog index -> blog.html
        let en_index_html = self.render_blog_index(cfg, Lang::En, &index.en_posts)?;
        let en_index_path = out_dir.join("blog.html");
        fs::write(&en_index_path, en_index_html)
            .with_context(|| format!("failed to write '{}'", en_index_path.display()))?;

        // 2. Render PT-BR blog index -> pt-br/blog.html
        let pt_index_html = self.render_blog_index(cfg, Lang::PtBr, &index.pt_posts)?;
        let pt_index_path = pt_dir.join("blog.html");
        fs::write(&pt_index_path, pt_index_html)
            .with_context(|| format!("failed to write '{}'", pt_index_path.display()))?;

        // 3. Render EN posts -> blog/<slug>.html
        for post in &index.en_posts {
            let html = self.render_post(cfg, post)?;
            let dest = out_dir.join(&post.rel_path);
            fs::write(&dest, html)
                .with_context(|| format!("failed to write '{}'", dest.display()))?;
        }

        // 4. Render PT-BR posts -> pt-br/blog/<slug>.html
        for post in &index.pt_posts {
            let html = self.render_post(cfg, post)?;
            let dest = out_dir.join(&post.rel_path);
            fs::write(&dest, html)
                .with_context(|| format!("failed to write '{}'", dest.display()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{Frontmatter, RawPost};
    use crate::site::build_site_index;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    #[test]
    fn test_render_index_and_post() {
        let cfg = Config {
            site_title: "Lucas Telles da Silva".to_string(),
            author: "Lucas Telles da Silva".to_string(),
            base_url: "https://lucastslv.github.io".to_string(),
            content_dir: PathBuf::from("content"),
            templates_dir: PathBuf::from("templates"),
            output_dir: PathBuf::from("../simple-website"),
        };

        let renderer = Renderer::new(&cfg.templates_dir).unwrap();
        let raw_posts = vec![
            RawPost {
                meta: Frontmatter {
                    title: "Hello Rust".to_string(),
                    slug: "hello-rust".to_string(),
                    date: NaiveDate::from_ymd_opt(2026, 11, 12).unwrap(),
                    summary: "First post in English.".to_string(),
                    tags: vec!["rust".to_string()],
                    draft: false,
                },
                lang: Lang::En,
                source_path: PathBuf::from("en.md"),
                html_body: "<p>Hello world</p>".to_string(),
                reading_time_min: 1,
            },
            RawPost {
                meta: Frontmatter {
                    title: "Olá Rust".to_string(),
                    slug: "hello-rust".to_string(),
                    date: NaiveDate::from_ymd_opt(2026, 11, 12).unwrap(),
                    summary: "Primeiro post em Português.".to_string(),
                    tags: vec!["rust".to_string()],
                    draft: false,
                },
                lang: Lang::PtBr,
                source_path: PathBuf::from("pt.md"),
                html_body: "<p>Olá mundo</p>".to_string(),
                reading_time_min: 1,
            },
        ];

        let index = build_site_index(raw_posts, false);

        let en_index_html = renderer
            .render_blog_index(&cfg, Lang::En, &index.en_posts)
            .unwrap();
        assert!(en_index_html.contains("Hello Rust"));
        assert!(
            en_index_html.contains(r#"href="blog/hello-rust.html""#),
            "actual html:\n{}",
            en_index_html
        );
        assert!(en_index_html.contains(r#"href="pt-br/blog.html""#));

        let pt_post_html = renderer.render_post(&cfg, &index.pt_posts[0]).unwrap();
        assert!(pt_post_html.contains("Olá Rust"));
        assert!(pt_post_html.contains(r#"href="../../style.css""#));
        assert!(pt_post_html.contains(r#"href="../../blog/hello-rust.html""#));
        assert!(pt_post_html.contains(r#"href="../blog.html""#));
    }
}
