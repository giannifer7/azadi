// weaveback-docgen/src/render/markdown/page.rs
// I'd Really Rather You Didn't edit this generated file.

fn strip_yaml_front_matter(source: &str) -> &str {
    let Some(rest) = source.strip_prefix("---\n") else {
        return source;
    };
    let Some(end) = rest.find("\n---\n") else {
        return source;
    };
    &rest[end + "\n---\n".len()..]
}

fn html_escape_text(text: &str) -> String {
    text.chars()
        .flat_map(|ch| match ch {
            '&' => "&amp;".chars().collect::<Vec<_>>(),
            '<' => "&lt;".chars().collect::<Vec<_>>(),
            '>' => "&gt;".chars().collect::<Vec<_>>(),
            '"' => "&quot;".chars().collect::<Vec<_>>(),
            '\'' => "&#39;".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn markdown_code_language(kind: &pulldown_cmark::CodeBlockKind<'_>) -> Option<String> {
    match kind {
        pulldown_cmark::CodeBlockKind::Fenced(info) => info
            .split(|ch: char| ch.is_whitespace() || ch == ',' || ch == '}')
            .next()
            .map(|s| s.trim_start_matches('{').to_string())
            .filter(|s| !s.is_empty()),
        pulldown_cmark::CodeBlockKind::Indented => None,
    }
}

static MARKDOWN_SYNTAX_SET: std::sync::LazyLock<syntect::parsing::SyntaxSet> =
    std::sync::LazyLock::new(syntect::parsing::SyntaxSet::load_defaults_newlines);

fn highlight_markdown_code(code: &str, lang: Option<&str>) -> String {
    let lang = lang.unwrap_or("text");
    let syntax_set = &*MARKDOWN_SYNTAX_SET;
    let syntax = syntax_set
        .find_syntax_by_token(lang)
        .or_else(|| syntax_set.find_syntax_by_extension(lang))
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let mut generator = syntect::html::ClassedHTMLGenerator::new_with_class_style(
        syntax,
        syntax_set,
        syntect::html::ClassStyle::SpacedPrefixed { prefix: "syntax-" },
    );
    for line in syntect::util::LinesWithEndings::from(code) {
        if generator.parse_html_for_line_which_includes_newline(line).is_err() {
            return html_escape_text(code);
        }
    }
    generator.finalize()
}

fn render_markdown_code_block(code: &str, lang: Option<&str>) -> String {
    let lang_attr = lang.unwrap_or("text");
    let highlighted = highlight_markdown_code(code, lang);
    format!(
        "<div class=\"listingblock\"><div class=\"content\"><pre class=\"highlight\"><code class=\"language-{lang_attr}\" data-lang=\"{lang_attr}\">{highlighted}</code></pre></div></div>\n",
        lang_attr = html_escape_text(lang_attr),
    )
}

fn render_markdown_body(source: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);

    let mut events = Vec::new();
    let mut iter = pulldown_cmark::Parser::new_ext(source, options).peekable();
    while let Some(event) = iter.next() {
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::CodeBlock(kind)) => {
                let lang = markdown_code_language(&kind);
                let mut code = String::new();
                for inner in iter.by_ref() {
                    match inner {
                        pulldown_cmark::Event::End(pulldown_cmark::TagEnd::CodeBlock) => break,
                        pulldown_cmark::Event::Text(text) => code.push_str(&text),
                        pulldown_cmark::Event::Code(text) => code.push_str(&text),
                        pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                            code.push('\n');
                        }
                        pulldown_cmark::Event::Html(text) | pulldown_cmark::Event::InlineHtml(text) => {
                            code.push_str(&text);
                        }
                        _ => {}
                    }
                }
                events.push(pulldown_cmark::Event::Html(
                    render_markdown_code_block(&code, lang.as_deref()).into(),
                ));
            }
            other => events.push(other),
        }
    }

    let mut body = String::new();
    pulldown_cmark::html::push_html(&mut body, events.into_iter());
    body
}

pub(crate) fn render_markdown_page(source: &str, title: &str) -> String {
    let source = strip_yaml_front_matter(source);
    let body = render_markdown_body(source);
    let title = html_escape_text(title);
    format!(
        "<!doctype html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>{title}</title>\n</head>\n<body>\n<div id=\"content\">\n{body}</div>\n</body>\n</html>\n"
    )
}
