// weaveback-docgen/src/render.rs
// I'd Really Rather You Didn't edit this generated file.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

mod adoc;
mod markdown;

pub use adoc::render_docs;
pub use markdown::render_markdown_docs;

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
pub(super) use markdown::{
    collect_markdown_diagram_fences, preprocess_markdown_diagrams, render_markdown_page,
    MarkdownDiagramKind,
};

#[cfg(test)]
mod tests;
