use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};

pub struct RenderedMarkdown {
    pub html: String,
    pub reading_time_min: usize,
}

struct ImageState<'a> {
    dest_url: String,
    title: String,
    inner_events: Vec<Event<'a>>,
    plain_text: String,
}

fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn resolve_image_url(url: &str, root_prefix: &str) -> String {
    if url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("data:")
        || url.starts_with('/')
        || url.starts_with("./")
        || url.starts_with("../")
        || url.starts_with('#')
    {
        url.to_string()
    } else {
        format!("{}{}", root_prefix, url)
    }
}

pub fn render_markdown(markdown_input: &str, root_prefix: &str) -> RenderedMarkdown {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let mut parser = Parser::new_ext(markdown_input, options).peekable();
    let mut events = Vec::new();
    let mut in_image: Option<ImageState> = None;

    while let Some(event) = parser.next() {
        if let Some(state) = in_image.as_mut() {
            match event {
                Event::End(TagEnd::Image) => {
                    let finished = in_image.take().unwrap();
                    let resolved_url = resolve_image_url(&finished.dest_url, root_prefix);

                    let mut caption_html = String::new();
                    html::push_html(&mut caption_html, finished.inner_events.into_iter());
                    let caption_html = caption_html.trim();

                    // Inverted order: ![figcaption](url "alt")
                    // If "alt" (Markdown title) is omitted, fall back to the plain-text caption.
                    let alt_text = if !finished.title.is_empty() {
                        &finished.title
                    } else {
                        finished.plain_text.trim()
                    };

                    let figure_html = if caption_html.is_empty() {
                        format!(
                            r#"<figure class="post-figure"><img src="{}" alt="{}" loading="lazy"></figure>"#,
                            escape_html_attr(&resolved_url),
                            escape_html_attr(alt_text),
                        )
                    } else {
                        format!(
                            r#"<figure class="post-figure"><img src="{}" alt="{}" loading="lazy"><figcaption>{}</figcaption></figure>"#,
                            escape_html_attr(&resolved_url),
                            escape_html_attr(alt_text),
                            caption_html,
                        )
                    };

                    // If the image is the only element in a paragraph, unwrap the surrounding <p>...</p>
                    // so that the block-level <figure> is not nested inside a <p>.
                    if matches!(events.last(), Some(Event::Start(Tag::Paragraph)))
                        && matches!(parser.peek(), Some(Event::End(TagEnd::Paragraph)))
                    {
                        events.pop();
                        parser.next();
                        events.push(Event::Html(format!("{}\n", figure_html).into()));
                    } else {
                        events.push(Event::Html(figure_html.into()));
                    }
                }
                Event::Text(ref t) | Event::Code(ref t) => {
                    state.plain_text.push_str(t);
                    state.inner_events.push(event);
                }
                Event::SoftBreak | Event::HardBreak => {
                    state.plain_text.push(' ');
                    state.inner_events.push(event);
                }
                other => {
                    state.inner_events.push(other);
                }
            }
            continue;
        }

        match event {
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                in_image = Some(ImageState {
                    dest_url: dest_url.into_string(),
                    title: title.into_string(),
                    inner_events: Vec::new(),
                    plain_text: String::new(),
                });
            }
            other => events.push(other),
        }
    }

    let mut html_output = String::with_capacity(markdown_input.len() * 3 / 2);
    html::push_html(&mut html_output, events.into_iter());

    let word_count = markdown_input.split_whitespace().count();
    let reading_time_min = (word_count / 200).max(1);

    RenderedMarkdown {
        html: html_output,
        reading_time_min,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html_and_reading_time() {
        let md = "# Hello\n\nThis is a **test** with `code` and ~strikethrough~.";
        let rendered = render_markdown(md, "../");
        assert!(rendered.html.contains("<h1>Hello</h1>"));
        assert!(rendered.html.contains("<strong>test</strong>"));
        assert!(rendered.html.contains("<code>code</code>"));
        assert!(rendered.html.contains("<del>strikethrough</del>"));
        assert_eq!(rendered.reading_time_min, 1);
    }

    #[test]
    fn test_image_with_caption_and_alt_inverted_order() {
        let md = r#"![Figura 1: Mesmo sem cache, o tempo caiu pela metade.](img/blog/meu-grafico.png "Gráfico de barras comparando o tempo de build em segundos")"#;
        let rendered_en = render_markdown(md, "../");
        assert_eq!(
            rendered_en.html.trim(),
            r#"<figure class="post-figure"><img src="../img/blog/meu-grafico.png" alt="Gráfico de barras comparando o tempo de build em segundos" loading="lazy"><figcaption>Figura 1: Mesmo sem cache, o tempo caiu pela metade.</figcaption></figure>"#
        );

        let rendered_pt = render_markdown(md, "../../");
        assert!(
            rendered_pt
                .html
                .contains(r#"src="../../img/blog/meu-grafico.png""#)
        );
    }

    #[test]
    fn test_image_without_caption_only_alt() {
        let md = r#"![](img/blog/setup.jpg "Foto da minha mesa")"#;
        let rendered = render_markdown(md, "../");
        assert_eq!(
            rendered.html.trim(),
            r#"<figure class="post-figure"><img src="../img/blog/setup.jpg" alt="Foto da minha mesa" loading="lazy"></figure>"#
        );
    }

    #[test]
    fn test_image_with_caption_only_falls_back_alt() {
        let md = r#"![Legenda com *ênfase*](https://example.com/pic.png)"#;
        let rendered = render_markdown(md, "../");
        assert_eq!(
            rendered.html.trim(),
            r#"<figure class="post-figure"><img src="https://example.com/pic.png" alt="Legenda com ênfase" loading="lazy"><figcaption>Legenda com <em>ênfase</em></figcaption></figure>"#
        );
    }
}
