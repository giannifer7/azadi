// weaveback-docgen/src/render/markdown.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

mod diagrams;
mod page;

pub(crate) use diagrams::{
    collect_markdown_diagram_fences, preprocess_markdown_diagrams, MarkdownDiagramKind,
};
pub(crate) use page::render_markdown_page;

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
