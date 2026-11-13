use std::collections::HashSet;

use chrono::{Datelike, NaiveDate};
use serde::Serialize;

use crate::content::{Frontmatter, Lang, RawPost};

#[derive(Debug, Clone, Serialize)]
pub struct Post {
    pub meta: Frontmatter,
    pub lang: Lang,
    pub formatted_date: String,
    pub iso_date: String,
    pub html_body: String,
    pub reading_time_min: usize,
    /// Relative path from site root (e.g. "blog/my-post.html" or "pt-br/blog/my-post.html")
    pub rel_path: String,
    /// Relative URL from the language blog index ("blog/my-post.html")
    pub index_href: String,
    /// Prefix to reach the site root from this post page ("../" for EN, "../../" for PT-BR)
    pub root_prefix: String,
    /// Prefix to reach the current language root from this post page ("../" for both EN and PT-BR)
    pub lang_prefix: String,
    /// Relative URL from this post page to its translation (or fallback to the other language's blog index)
    pub language_switch_href: String,
    /// Relative URL from site root to the English version of this page (for hreflang)
    pub hreflang_en: String,
    /// Relative URL from site root to the Portuguese version of this page (for hreflang)
    pub hreflang_pt: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SiteIndex {
    pub en_posts: Vec<Post>,
    pub pt_posts: Vec<Post>,
}

pub fn format_date(date: NaiveDate, lang: Lang) -> String {
    match lang {
        Lang::En => {
            let month = match date.month() {
                1 => "Jan",
                2 => "Feb",
                3 => "Mar",
                4 => "Apr",
                5 => "May",
                6 => "Jun",
                7 => "Jul",
                8 => "Aug",
                9 => "Sep",
                10 => "Oct",
                11 => "Nov",
                12 => "Dec",
                _ => "",
            };
            format!("{} {:02}, {}", month, date.day(), date.year())
        }
        Lang::PtBr => {
            let month = match date.month() {
                1 => "jan.",
                2 => "fev.",
                3 => "mar.",
                4 => "abr.",
                5 => "mai.",
                6 => "jun.",
                7 => "jul.",
                8 => "ago.",
                9 => "set.",
                10 => "out.",
                11 => "nov.",
                12 => "dez.",
                _ => "",
            };
            format!("{:02} de {} de {}", date.day(), month, date.year())
        }
    }
}

pub fn build_site_index(raw_posts: Vec<RawPost>, include_drafts: bool) -> SiteIndex {
    let filtered: Vec<RawPost> = raw_posts
        .into_iter()
        .filter(|p| include_drafts || !p.meta.draft)
        .collect();

    let en_slugs: HashSet<String> = filtered
        .iter()
        .filter(|p| p.lang == Lang::En)
        .map(|p| p.meta.slug.clone())
        .collect();

    let pt_slugs: HashSet<String> = filtered
        .iter()
        .filter(|p| p.lang == Lang::PtBr)
        .map(|p| p.meta.slug.clone())
        .collect();

    let mut en_posts = Vec::new();
    let mut pt_posts = Vec::new();

    for raw in filtered {
        let slug = &raw.meta.slug;
        let has_en = en_slugs.contains(slug);
        let has_pt = pt_slugs.contains(slug);

        let formatted_date = format_date(raw.meta.date, raw.lang);
        let iso_date = raw.meta.date.format("%Y-%m-%d").to_string();

        match raw.lang {
            Lang::En => {
                // Output file: blog/<slug>.html (depth 1 from root)
                let root_prefix = raw.lang.post_root_prefix().to_string();
                let lang_prefix = "../".to_string();
                let rel_path = format!("blog/{}.html", slug);
                let index_href = format!("blog/{}.html", slug);

                let pt_target = if has_pt {
                    format!("pt-br/blog/{}.html", slug)
                } else {
                    "pt-br/blog.html".to_string()
                };
                let language_switch_href = format!("{}{}", root_prefix, pt_target);
                let hreflang_en = format!("{}{}", root_prefix, rel_path);
                let hreflang_pt = language_switch_href.clone();

                en_posts.push(Post {
                    meta: raw.meta,
                    lang: Lang::En,
                    formatted_date,
                    iso_date,
                    html_body: raw.html_body,
                    reading_time_min: raw.reading_time_min,
                    rel_path,
                    index_href,
                    root_prefix,
                    lang_prefix,
                    language_switch_href,
                    hreflang_en,
                    hreflang_pt,
                });
            }
            Lang::PtBr => {
                // Output file: pt-br/blog/<slug>.html (depth 2 from root, depth 1 from pt-br/)
                let root_prefix = raw.lang.post_root_prefix().to_string();
                let lang_prefix = "../".to_string();
                let rel_path = format!("pt-br/blog/{}.html", slug);
                let index_href = format!("blog/{}.html", slug);

                let en_target = if has_en {
                    format!("blog/{}.html", slug)
                } else {
                    "blog.html".to_string()
                };
                let language_switch_href = format!("{}{}", root_prefix, en_target);
                let hreflang_en = language_switch_href.clone();
                let hreflang_pt = format!("{}.html", slug);

                pt_posts.push(Post {
                    meta: raw.meta,
                    lang: Lang::PtBr,
                    formatted_date,
                    iso_date,
                    html_body: raw.html_body,
                    reading_time_min: raw.reading_time_min,
                    rel_path,
                    index_href,
                    root_prefix,
                    lang_prefix,
                    language_switch_href,
                    hreflang_en,
                    hreflang_pt,
                });
            }
        }
    }

    en_posts.sort_by(|a, b| {
        b.meta
            .date
            .cmp(&a.meta.date)
            .then_with(|| a.meta.slug.cmp(&b.meta.slug))
    });
    pt_posts.sort_by(|a, b| {
        b.meta
            .date
            .cmp(&a.meta.date)
            .then_with(|| a.meta.slug.cmp(&b.meta.slug))
    });

    SiteIndex { en_posts, pt_posts }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_raw_post(slug: &str, date: &str, lang: Lang, draft: bool) -> RawPost {
        RawPost {
            meta: Frontmatter {
                title: format!("Title {}", slug),
                slug: slug.to_string(),
                date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
                summary: "Summary".to_string(),
                tags: vec!["rust".to_string()],
                draft,
            },
            lang,
            source_path: PathBuf::from("test.md"),
            html_body: "<p>Body</p>".to_string(),
            reading_time_min: 2,
        }
    }

    #[test]
    fn test_bilingual_linking_and_sorting() {
        let posts = vec![
            sample_raw_post("older", "2026-01-10", Lang::En, false),
            sample_raw_post("newer", "2026-11-12", Lang::En, false),
            sample_raw_post("newer", "2026-11-12", Lang::PtBr, false),
            sample_raw_post("secret", "2026-12-01", Lang::En, true),
        ];

        let index = build_site_index(posts, false);
        assert_eq!(index.en_posts.len(), 2);
        assert_eq!(index.pt_posts.len(), 1);

        // Sorted descending by date
        assert_eq!(index.en_posts[0].meta.slug, "newer");
        assert_eq!(index.en_posts[1].meta.slug, "older");

        // "newer" exists in both languages, so it links directly to the translated post
        assert_eq!(
            index.en_posts[0].language_switch_href,
            "../pt-br/blog/newer.html"
        );
        assert_eq!(
            index.pt_posts[0].language_switch_href,
            "../../blog/newer.html"
        );

        // "older" only exists in EN, so its language switch falls back to pt-br/blog.html
        assert_eq!(index.en_posts[1].language_switch_href, "../pt-br/blog.html");
    }
}
