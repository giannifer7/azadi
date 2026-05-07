// weaveback-docgen/src/render.rs
// I'd Really Rather You Didn't edit this generated file.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum RenderError {
    #[error("PlantUML batch render failed")]
    #[diagnostic(code(weaveback::docgen::plantuml_batch))]
    PlantUmlBatch {
        #[source]
        source: crate::plantuml::PlantUmlError,
    },
    #[error("PlantUML preprocessing failed for {path}")]
    #[diagnostic(code(weaveback::docgen::plantuml))]
    PlantUml {
        path: String,
        #[source]
        source: crate::plantuml::PlantUmlError,
    },
    #[error("D2 preprocessing failed for {path}")]
    #[diagnostic(code(weaveback::docgen::d2))]
    D2 {
        path: String,
        #[source]
        source: crate::d2::D2Error,
    },
    #[error("failed to read source document {path}")]
    #[diagnostic(code(weaveback::docgen::read))]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write rendered HTML file {path}")]
    #[diagnostic(code(weaveback::docgen::write))]
    Write {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
const EXCLUDE_DIRS: &[&str] = &["target", ".git", "node_modules", ".venv"];
fn mtime(path: &Path) -> SystemTime {
    path.metadata()
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

fn theme_max_mtime(theme_dir: &Path) -> SystemTime {
    walkdir::WalkDir::new(theme_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| mtime(e.path()))
        .max()
        .unwrap_or(SystemTime::UNIX_EPOCH)
}
fn dedup_specials(content: &str, specials: &[char]) -> Option<String> {
    let mut out = content.to_owned();
    let mut changed = false;
    for &s in specials {
        let doubled = format!("{s}{s}");
        if out.contains(&doubled) {
            out = out.replace(&doubled, &s.to_string());
            changed = true;
        }
    }
    if changed { Some(out) } else { None }
}
fn copy_theme_assets(theme_dir: &Path, out_dir: &Path) {
    for name in &["wb-theme.css", "wb-theme.js"] {
        let src = theme_dir.join(name);
        let dst = out_dir.join(name);
        if src.exists() {
            std::fs::copy(&src, &dst).ok();
        }
    }
}

fn read_docinfo(theme_dir: &Path) -> Option<String> {
    let path = theme_dir.join("docinfo.html");
    std::fs::read_to_string(&path).ok()
}

fn read_footer(theme_dir: &Path) -> Option<String> {
    let path = theme_dir.join("docinfo-footer.html");
    std::fs::read_to_string(&path).ok()
}

fn inject_docinfo(mut html: String, docinfo: &str) -> String {
    if let Some(pos) = html.find("</head>") {
        html.insert_str(pos, docinfo);
    }
    html
}

fn inject_footer(mut html: String, footer: &str) -> String {
    if let Some(pos) = html.find("</body>") {
        html.insert_str(pos, footer);
    }
    html
}
pub fn render_docs(
    project_root: &Path,
    theme_dir: &Path,
    out_dir: &Path,
    specials: &[char],
    plantuml_jar: Option<&Path>,
    d2_theme: u32,
    d2_layout: &str,
) -> Result<Vec<PathBuf>, RenderError> {
    use acdc_converters_core::Converter as _;
    use rayon::prelude::*;

    std::fs::create_dir_all(out_dir).ok();
    copy_theme_assets(theme_dir, out_dir);

    let docinfo = read_docinfo(theme_dir);
    let footer = read_footer(theme_dir);
    let theme_mtime = theme_max_mtime(theme_dir);
    let adoc_files = find_adoc_files(project_root);

    // SVG cache lives outside out_dir so `rm -rf <out_dir>` doesn't blow it away.
    let svg_cache_dir = out_dir.parent().unwrap_or(out_dir).join(".plantuml-cache");
    std::fs::create_dir_all(&svg_cache_dir).ok();

    // Phase 1: collect uncached PlantUML diagrams from all stale files, then
    // batch-render them in a single JVM invocation.
    if let Some(jar) = plantuml_jar {
        let mut to_render: Vec<(String, PathBuf)> = Vec::new();
        let mut seen: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

        for adoc in &adoc_files {
            let rel = adoc.strip_prefix(project_root).unwrap_or(adoc);
            let out_file = out_dir.join(rel).with_extension("html");
            if out_file.exists()
                && mtime(&out_file) >= mtime(adoc)
                && mtime(&out_file) >= theme_mtime
            {
                continue;
            }
            let source = std::fs::read_to_string(adoc).unwrap_or_default();
            let label = adoc.strip_prefix(project_root).unwrap_or(adoc).to_string_lossy();
            for (diagram_src, cache_path) in crate::plantuml::collect_uncached_plantuml_diagrams(
                &source, &svg_cache_dir, &label,
            ) {
                if seen.insert(cache_path.clone()) {
                    to_render.push((diagram_src, cache_path));
                }
            }
        }

        if !to_render.is_empty()
            && let Err(e) = crate::plantuml::batch_render_plantuml(&to_render, jar) {
                return Err(RenderError::PlantUmlBatch { source: e });
        }
    }

    // Phase 2: parallel acdc render; plantuml preprocessing only copies from cache.
    let results: Vec<Result<(PathBuf, bool), RenderError>> = adoc_files
        .par_iter()
        .map(|adoc| {
            let rel = adoc.strip_prefix(project_root).unwrap_or(adoc);
            let out_file = out_dir.join(rel).with_extension("html");
            std::fs::create_dir_all(out_file.parent().unwrap()).ok();

            if out_file.exists()
                && mtime(&out_file) >= mtime(adoc)
                && mtime(&out_file) >= theme_mtime
            {
                return Ok((out_file, false));
            }

            let source = std::fs::read_to_string(adoc).unwrap_or_default();

            // 1. PlantUML pre-processing.
            let after_plantuml: Option<String> = if let Some(jar) = plantuml_jar {
                let images_dir = out_file.parent().unwrap_or(out_dir);
                let label = adoc.strip_prefix(project_root).unwrap_or(adoc).to_string_lossy();
                match crate::plantuml::preprocess_plantuml(
                    &source, jar, images_dir, &svg_cache_dir, &label,
                ) {
                    Ok(opt) => opt,
                    Err(e) => return Err(RenderError::PlantUml {
                        path: adoc.display().to_string(),
                        source: e,
                    }),
                }
            } else {
                None
            };

            // 1.5. D2 pre-processing.
            let base_before_d2 = after_plantuml.as_deref().unwrap_or(&source);
            let after_d2: Option<String> = {
                let images_dir = out_file.parent().unwrap_or(out_dir);
                let label = adoc.strip_prefix(project_root).unwrap_or(adoc).to_string_lossy();
                match crate::d2::preprocess_d2(
                    base_before_d2, images_dir, &svg_cache_dir, &label, d2_theme, d2_layout,
                ) {
                    Ok(opt) => opt,
                    Err(e) => return Err(RenderError::D2 {
                        path: adoc.display().to_string(),
                        source: e,
                    }),
                }
            };

            // 2. Special-char deduplication.
            let base = after_d2.as_deref().or(after_plantuml.as_deref()).unwrap_or(&source);
            let processed: String = dedup_specials(base, specials)
                .unwrap_or_else(|| base.to_owned());

            // 3. Build per-file acdc options.
            let source_dir = adoc.parent().unwrap_or(project_root);
            let docname = adoc
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_owned();
            let images_dir = out_file
                .parent()
                .unwrap_or(out_dir)
                .to_string_lossy()
                .into_owned();

            let parse_options = acdc_parser::Options::builder()
                .with_attribute("imagesdir", images_dir)
                .with_attribute("source-highlighter", "syntect")
                .with_attribute("syntect-css", "class")
                .build();

            // 4. Parse — catch panics from parser bugs in experimental acdc.
            let parse_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                acdc_parser::parse(&processed, &parse_options)
            }));
            let doc = match parse_result {
                Ok(Ok(d)) => d,
                Ok(Err(e)) => {
                    eprintln!("acdc parse: {}: {}", adoc.display(), e);
                    return Ok((out_file, false));
                }
                Err(_) => {
                    eprintln!("acdc parse: {}: parser panicked, skipping", adoc.display());
                    return Ok((out_file, false));
                }
            };

            // 5. Render to HTML — catch panics from renderer bugs in experimental acdc.
            let conv_options = acdc_converters_core::Options::builder().build();
            let processor = acdc_converters_html::Processor::new(
                conv_options,
                doc.attributes.clone(),
            );
            let render_opts = acdc_converters_html::RenderOptions {
                embedded: false,
                source_dir: Some(source_dir.to_path_buf()),
                docname: Some(docname),
                ..Default::default()
            };
            let render_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                processor.convert_to_string(&doc, &render_opts)
            }));
            let mut html = match render_result {
                Ok(Ok(h)) => h,
                Ok(Err(e)) => {
                    eprintln!("acdc render: {}: {}", adoc.display(), e);
                    return Ok((out_file, false));
                }
                Err(_) => {
                    eprintln!("acdc render: {}: renderer panicked, skipping", adoc.display());
                    return Ok((out_file, false));
                }
            };

            // 6. Inject head fragment (link tag) and footer script tag.
            // Rewrite absolute asset paths (href="/…", src="/…") to relative
            // so the page works when served from a subdirectory (e.g. GitHub Pages).
            let depth = out_file
                .strip_prefix(out_dir)
                .map(|rel| rel.components().count().saturating_sub(1))
                .unwrap_or(0);
            let prefix = "../".repeat(depth);
            if let Some(ref di) = docinfo {
                let patched = di
                    .replace("href=\"/", &format!("href=\"{prefix}"))
                    .replace("src=\"/", &format!("src=\"{prefix}"));
                html = inject_docinfo(html, &patched);
            }
            if let Some(ref f) = footer {
                let patched = f
                    .replace("href=\"/", &format!("href=\"{prefix}"))
                    .replace("src=\"/", &format!("src=\"{prefix}"));
                html = inject_footer(html, &patched);
            }

            if let Err(e) = std::fs::write(&out_file, &html) {
                return Err(RenderError::Write {
                    path: out_file.display().to_string(),
                    source: e,
                });
            }

            Ok((out_file, true))
        })
        .collect();

    let results: Vec<(PathBuf, bool)> = results.into_iter().collect::<Result<_, _>>()?;
    let all_html: Vec<PathBuf> = results.iter().map(|(p, _)| p.clone()).collect();
    let rendered = results.iter().filter(|(_, r)| *r).count();

    if rendered == 0 {
        println!("docs: nothing to do");
    } else {
        println!("docs: rendered {rendered} file(s)");
    }

    Ok(all_html)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkdownDiagramKind {
    PlantUml,
    D2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownDiagramFence {
    start: usize,
    end: usize,
    body: String,
    kind: MarkdownDiagramKind,
}

fn strip_yaml_front_matter(source: &str) -> &str {
    let Some(rest) = source.strip_prefix("---\n") else {
        return source;
    };
    let Some(end) = rest.find("\n---\n") else {
        return source;
    };
    &rest[end + "\n---\n".len()..]
}

fn markdown_fence_kind(info: &str) -> Option<MarkdownDiagramKind> {
    let lang = info
        .trim()
        .trim_start_matches('{')
        .split(|ch: char| ch.is_whitespace() || ch == ',' || ch == '}')
        .next()
        .unwrap_or("");
    match lang {
        "plantuml" => Some(MarkdownDiagramKind::PlantUml),
        "d2" => Some(MarkdownDiagramKind::D2),
        _ => None,
    }
}

fn opening_fence(line: &str) -> Option<(char, usize, &str)> {
    let trimmed = line.trim_start();
    let first = trimmed.as_bytes().first().copied()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let marker = first as char;
    let len = trimmed.chars().take_while(|&ch| ch == marker).count();
    if len < 3 {
        return None;
    }
    Some((marker, len, &trimmed[len..]))
}

fn closing_fence(line: &str, marker: char, len: usize) -> bool {
    let trimmed = line.trim_start();
    let count = trimmed.chars().take_while(|&ch| ch == marker).count();
    count >= len && trimmed[count..].trim().is_empty()
}

fn markdown_graph_comment_start(source: &str, fence_start: usize) -> Option<usize> {
    let before = &source[..fence_start];
    let comment_end = before.strip_suffix('\n')?.rfind('\n').map_or(0, |idx| idx + 1);
    let line = &before[comment_end..before.len() - 1];
    let trimmed = line.trim();
    if trimmed.starts_with("<!-- graph:") && trimmed.ends_with("-->") {
        Some(comment_end)
    } else {
        None
    }
}

fn collect_markdown_diagram_fences(source: &str) -> Vec<MarkdownDiagramFence> {
    let mut fences = Vec::new();
    let mut offset = 0usize;
    let mut lines = source.split_inclusive('\n').peekable();

    while let Some(line) = lines.next() {
        let line_start = offset;
        offset += line.len();

        let Some((marker, len, info)) = opening_fence(line) else {
            continue;
        };
        let kind = markdown_fence_kind(info);
        let body_start = offset;

        let mut body_end = offset;
        let mut end = offset;
        let mut closed = false;

        for inner in lines.by_ref() {
            let inner_start = offset;
            offset += inner.len();
            if closing_fence(inner, marker, len) {
                body_end = inner_start;
                end = offset;
                closed = true;
                break;
            }
        }

        if closed && let Some(kind) = kind {
            fences.push(MarkdownDiagramFence {
                start: markdown_graph_comment_start(source, line_start).unwrap_or(line_start),
                end,
                body: source[body_start..body_end].to_string(),
                kind,
            });
        }
    }

    fences
}

fn markdown_diagram_alt(kind: MarkdownDiagramKind) -> &'static str {
    match kind {
        MarkdownDiagramKind::PlantUml => "PlantUML diagram",
        MarkdownDiagramKind::D2 => "D2 diagram",
    }
}

fn preprocess_markdown_diagrams(
    source: &str,
    images_out_dir: &Path,
    svg_cache_dir: &Path,
    plantuml_jar: Option<&Path>,
    d2_theme: u32,
    d2_layout: &str,
) -> Result<Option<String>, RenderError> {
    let fences = collect_markdown_diagram_fences(source);
    if fences.is_empty() {
        return Ok(None);
    }

    std::fs::create_dir_all(images_out_dir).ok();
    std::fs::create_dir_all(svg_cache_dir).ok();

    let mut replacements: Vec<(usize, usize, String)> = Vec::with_capacity(fences.len());
    for (index, fence) in fences.into_iter().enumerate() {
        let svg_name = match fence.kind {
            MarkdownDiagramKind::PlantUml => {
                if plantuml_jar.is_none() {
                    continue;
                }
                let hash = blake3::hash(fence.body.as_bytes());
                let svg_name = format!("{}.svg", hash.to_hex());
                let svg_cache_path = svg_cache_dir.join(&svg_name);
                let svg_out_path = images_out_dir.join(&svg_name);
                std::fs::copy(&svg_cache_path, &svg_out_path).map_err(|source| {
                    RenderError::Write {
                        path: svg_out_path.display().to_string(),
                        source,
                    }
                })?;
                svg_name
            }
            MarkdownDiagramKind::D2 => {
                let hash = blake3::hash(fence.body.as_bytes());
                let svg_name = format!("d2-{}.svg", hash.to_hex());
                let svg_cache_path = svg_cache_dir.join(&svg_name);
                if !svg_cache_path.exists() {
                    let svg_bytes = crate::d2::render_d2_diagram(
                        &fence.body,
                        index,
                        d2_theme,
                        d2_layout,
                    )
                    .map_err(|source| RenderError::D2 {
                        path: "markdown diagram".to_string(),
                        source,
                    })?;
                    std::fs::write(&svg_cache_path, svg_bytes).map_err(|source| {
                        RenderError::Write {
                            path: svg_cache_path.display().to_string(),
                            source,
                        }
                    })?;
                }
                let svg_out_path = images_out_dir.join(&svg_name);
                std::fs::copy(&svg_cache_path, &svg_out_path).map_err(|source| {
                    RenderError::Write {
                        path: svg_out_path.display().to_string(),
                        source,
                    }
                })?;
                svg_name
            }
        };

        let replacement = format!("![{}]({})\n", markdown_diagram_alt(fence.kind), svg_name);
        replacements.push((fence.start, fence.end, replacement));
    }

    if replacements.is_empty() {
        return Ok(None);
    }

    let mut result = source.to_owned();
    for (start, end, replacement) in replacements.into_iter().rev() {
        result.replace_range(start..end, &replacement);
    }

    Ok(Some(result))
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

fn render_markdown_page(source: &str, title: &str) -> String {
    let source = strip_yaml_front_matter(source);
    let body = render_markdown_body(source);
    let title = html_escape_text(title);
    format!(
        "<!doctype html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>{title}</title>\n</head>\n<body>\n<div id=\"content\">\n{body}</div>\n</body>\n</html>\n"
    )
}

pub fn render_markdown_docs(
    markdown_root: &Path,
    theme_dir: &Path,
    out_dir: &Path,
    plantuml_jar: Option<&Path>,
    d2_theme: u32,
    d2_layout: &str,
) -> Result<Vec<PathBuf>, RenderError> {
    use rayon::prelude::*;

    std::fs::create_dir_all(out_dir).ok();
    copy_theme_assets(theme_dir, out_dir);

    let docinfo = read_docinfo(theme_dir);
    let footer = read_footer(theme_dir);
    let theme_mtime = theme_max_mtime(theme_dir);
    let md_files = find_md_files(markdown_root);
    let svg_cache_dir = out_dir.parent().unwrap_or(out_dir).join(".plantuml-cache");
    std::fs::create_dir_all(&svg_cache_dir).ok();

    if let Some(jar) = plantuml_jar {
        let mut to_render: Vec<(String, PathBuf)> = Vec::new();
        let mut seen: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

        for md in &md_files {
            let rel = md.strip_prefix(markdown_root).unwrap_or(md);
            let out_file = out_dir.join(rel).with_extension("html");
            if out_file.exists()
                && mtime(&out_file) >= mtime(md)
                && mtime(&out_file) >= theme_mtime
            {
                continue;
            }
            let source = std::fs::read_to_string(md).unwrap_or_default();
            for fence in collect_markdown_diagram_fences(&source) {
                if fence.kind != MarkdownDiagramKind::PlantUml {
                    continue;
                }
                let hash = blake3::hash(fence.body.as_bytes());
                let cache_path = svg_cache_dir.join(format!("{}.svg", hash.to_hex()));
                if !cache_path.exists() && seen.insert(cache_path.clone()) {
                    to_render.push((fence.body, cache_path));
                }
            }
        }

        if !to_render.is_empty()
            && let Err(source) = crate::plantuml::batch_render_plantuml(&to_render, jar) {
                return Err(RenderError::PlantUmlBatch { source });
        }
    }

    let results: Vec<Result<(PathBuf, bool), RenderError>> = md_files
        .par_iter()
        .map(|md| {
            let rel = md.strip_prefix(markdown_root).unwrap_or(md);
            let out_file = out_dir.join(rel).with_extension("html");
            std::fs::create_dir_all(out_file.parent().unwrap()).ok();

            if out_file.exists()
                && mtime(&out_file) >= mtime(md)
                && mtime(&out_file) >= theme_mtime
            {
                return Ok((out_file, false));
            }

            let source = std::fs::read_to_string(md).map_err(|source| RenderError::Read {
                path: md.display().to_string(),
                source,
            })?;
            let title = md
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document");
            let images_dir = out_file.parent().unwrap_or(out_dir);
            let processed = preprocess_markdown_diagrams(
                &source,
                images_dir,
                &svg_cache_dir,
                plantuml_jar,
                d2_theme,
                d2_layout,
            )?;
            let base = processed.as_deref().unwrap_or(&source);
            let mut html = render_markdown_page(base, title);

            let depth = out_file
                .strip_prefix(out_dir)
                .map(|rel| rel.components().count().saturating_sub(1))
                .unwrap_or(0);
            let prefix = "../".repeat(depth);
            if let Some(ref di) = docinfo {
                let patched = di
                    .replace("href=\"/", &format!("href=\"{prefix}"))
                    .replace("src=\"/", &format!("src=\"{prefix}"));
                html = inject_docinfo(html, &patched);
            }
            if let Some(ref f) = footer {
                let patched = f
                    .replace("href=\"/", &format!("href=\"{prefix}"))
                    .replace("src=\"/", &format!("src=\"{prefix}"));
                html = inject_footer(html, &patched);
            }

            std::fs::write(&out_file, &html).map_err(|source| RenderError::Write {
                path: out_file.display().to_string(),
                source,
            })?;

            Ok((out_file, true))
        })
        .collect();

    let results: Vec<(PathBuf, bool)> = results.into_iter().collect::<Result<_, _>>()?;
    let all_html: Vec<PathBuf> = results.iter().map(|(p, _)| p.clone()).collect();
    let rendered = results.iter().filter(|(_, r)| *r).count();

    if rendered == 0 {
        println!("docs-md: nothing to do");
    } else {
        println!("docs-md: rendered {rendered} file(s)");
    }

    Ok(all_html)
}
fn find_adoc_files(root: &Path) -> Vec<PathBuf> {
    find_files_with_extension(root, "adoc")
}

fn find_md_files(root: &Path) -> Vec<PathBuf> {
    find_files_with_extension(root, "md")
}

fn find_files_with_extension(root: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDE_DIRS.iter().any(|ex| name == *ex)
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == extension)
        })
        .map(|e| e.into_path())
        .collect();
    files.sort();
    files
}
#[cfg(test)]
mod tests;
