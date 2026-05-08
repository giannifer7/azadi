// weaveback-docgen/src/render/markdown/diagrams.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarkdownDiagramKind {
    PlantUml,
    D2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownDiagramFence {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) body: String,
    pub(crate) kind: MarkdownDiagramKind,
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

pub(crate) fn collect_markdown_diagram_fences(source: &str) -> Vec<MarkdownDiagramFence> {
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

pub(crate) fn preprocess_markdown_diagrams(
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
