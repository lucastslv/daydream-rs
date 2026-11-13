use pulldown_cmark::{Options, Parser, html};

pub struct RenderedMarkdown {
    pub html: String,
    pub reading_time_min: usize,
}

pub fn render_markdown(markdown_input: &str) -> RenderedMarkdown {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown_input, options);
    let mut html_output = String::with_capacity(markdown_input.len() * 3 / 2);
    html::push_html(&mut html_output, parser);

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
        let rendered = render_markdown(md);
        assert!(rendered.html.contains("<h1>Hello</h1>"));
        assert!(rendered.html.contains("<strong>test</strong>"));
        assert!(rendered.html.contains("<code>code</code>"));
        assert!(rendered.html.contains("<del>strikethrough</del>"));
        assert_eq!(rendered.reading_time_min, 1);
    }
}
