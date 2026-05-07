// weaveback-docgen/src/main.rs
// I'd Really Rather You Didn't edit this generated file.

mod adoc_scan;
mod d2;
mod inject;
mod literate_index;
mod plantuml;
mod render;
mod xref;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use xref::XrefEntry;

fn find_project_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("cannot determine cwd");
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();
            if content.contains("[workspace]") {
                return dir;
            }
        }
        if !dir.pop() {
            break;
        }
    }
    std::env::current_dir().unwrap()
}
const HELP: &str = "\
Usage: weaveback-docgen [OPTIONS]

Renders all .adoc files to HTML, post-processes them with chunk IDs and
a literate-source index, and (for Rust workspaces) injects cross-reference
data linking modules by their import graph.

Options:
  --out-dir   <path>   Output directory for rendered HTML
                       (default: <project-root>/docs/html)
  --md-root   <path>   Directory containing generated Markdown projection files
                       (default: <project-root>/expanded-md)
  --md-out-dir <path>  Output directory for rendered Markdown HTML
                       (default: <project-root>/docs/html-md)
  --no-md              Skip Markdown HTML rendering.
  --no-adoc            Skip AsciiDoc HTML rendering.
  --theme-dir <path>   Directory containing docinfo.html / docinfo-footer.html
                       (default: <project-root>/scripts/asciidoc-theme)
  --special   <char>   De-duplicate doubled occurrences of CHAR before
                       passing .adoc files to the acdc renderer (repeatable)
  --xref-cmd  <cmd>    External command to supply cross-reference data.
                       Receives the project root as its first argument and
                       must print a JSON object matching HashMap<key, XrefEntry>
                       to stdout.  Replaces the built-in Rust scanner.
  --no-xref            Skip cross-reference analysis entirely.
  --ai-xref            Use LSP (rust-analyzer) to build precise cross-references.
  --plantuml-jar <path>  Path to plantuml.jar; renders [plantuml] blocks directly
                         (SVGs cached by BLAKE3).
  --help               Print this message and exit.

Cross-reference notes:
  The built-in scanner is Rust-specific: it parses .rs files with syn and
  expects workspace members under <project-root>/crates/.  Workspaces that
  place members elsewhere (root-level, packages/, libs/, ...) will not get
  automatic xref -- use --xref-cmd to supply data from an external tool, or
  --no-xref to skip it.
";
struct Args {
    specials: Vec<char>,
    xref_cmd: Option<String>,
    no_xref: bool,
    ai_xref: bool,
    out_dir: Option<PathBuf>,
    md_root: Option<PathBuf>,
    md_out_dir: Option<PathBuf>,
    no_md: bool,
    no_adoc: bool,
    theme_dir: Option<PathBuf>,
    plantuml_jar: Option<PathBuf>,
    d2_theme: u32,
    d2_layout: String,
}

#[derive(serde::Deserialize, Default)]
struct DocsConfig {
    d2_theme: Option<u32>,
    d2_layout: Option<String>,
}

#[derive(serde::Deserialize, Default)]
struct WeavebackConfig {
    docs: Option<DocsConfig>,
}

fn read_config(root: &Path) -> WeavebackConfig {
    let path = root.join("weaveback.toml");
    if let Ok(content) = std::fs::read_to_string(&path) {
        toml::from_str(&content).unwrap_or_default()
    } else {
        WeavebackConfig::default()
    }
}

fn parse_args_from(raw: &[String]) -> Args {
    let mut specials = Vec::new();
    let mut xref_cmd = None;
    let mut no_xref = false;
    let mut ai_xref = false;
    let mut out_dir = None;
    let mut md_root = None;
    let mut md_out_dir = None;
    let mut no_md = false;
    let mut no_adoc = false;
    let mut theme_dir = None;
    let mut plantuml_jar = None;
    let mut i = 1;
    while i < raw.len() {
        match raw[i].as_str() {
            "--help" | "-h" => {
                print!("{HELP}");
                std::process::exit(0);
            }
            "--special" => {
                if let Some(s) = raw.get(i + 1) {
                    let mut chars = s.chars();
                    if let (Some(c), None) = (chars.next(), chars.next()) {
                        specials.push(c);
                    }
                    i += 2;
                    continue;
                }
            }
            "--xref-cmd" => {
                if let Some(cmd) = raw.get(i + 1) {
                    xref_cmd = Some(cmd.clone());
                    i += 2;
                    continue;
                }
            }
            "--out-dir" => {
                if let Some(p) = raw.get(i + 1) {
                    out_dir = Some(PathBuf::from(p));
                    i += 2;
                    continue;
                }
            }
            "--md-root" => {
                if let Some(p) = raw.get(i + 1) {
                    md_root = Some(PathBuf::from(p));
                    i += 2;
                    continue;
                }
            }
            "--md-out-dir" => {
                if let Some(p) = raw.get(i + 1) {
                    md_out_dir = Some(PathBuf::from(p));
                    i += 2;
                    continue;
                }
            }
            "--theme-dir" => {
                if let Some(p) = raw.get(i + 1) {
                    theme_dir = Some(PathBuf::from(p));
                    i += 2;
                    continue;
                }
            }
            "--plantuml-jar" => {
                if let Some(p) = raw.get(i + 1) {
                    plantuml_jar = Some(PathBuf::from(p));
                    i += 2;
                    continue;
                }
            }
            "--no-xref" => {
                no_xref = true;
            }
            "--no-md" => {
                no_md = true;
            }
            "--no-adoc" => {
                no_adoc = true;
            }
            "--ai-xref" => {
                ai_xref = true;
            }
            _ => {}
        }
        i += 1;
    }
    Args {
        specials,
        xref_cmd,
        no_xref,
        ai_xref,
        out_dir,
        md_root,
        md_out_dir,
        no_md,
        no_adoc,
        theme_dir,
        plantuml_jar,
        d2_theme: 200,
        d2_layout: "elk".to_string(),
    }
}

fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().collect();
    parse_args_from(&raw)
}
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
enum Error {
    #[error("documentation rendering failed")]
    #[diagnostic(code(weaveback::docgen::render))]
    Render {
        #[from]
        #[source]
        source: render::RenderError,
    },
    #[error("failed to run xref command '{cmd}'")]
    #[diagnostic(code(weaveback::docgen::xref_cmd_spawn))]
    XrefCommandSpawn {
        cmd: String,
        #[source]
        source: std::io::Error,
    },
    #[error("xref command '{cmd}' exited with status {code}")]
    #[diagnostic(code(weaveback::docgen::xref_cmd_status))]
    XrefCommandStatus { cmd: String, code: i32 },
    #[error("failed to parse JSON from xref command '{cmd}'")]
    #[diagnostic(code(weaveback::docgen::xref_cmd_json))]
    XrefCommandJson {
        cmd: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to serialise xref data")]
    #[diagnostic(code(weaveback::docgen::xref_json))]
    XrefJson {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to write xref data to {path}")]
    #[diagnostic(code(weaveback::docgen::xref_write))]
    XrefWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
fn run_xref_cmd(cmd: &str, project_root: &Path) -> Result<HashMap<String, XrefEntry>, Error> {
    let output = Command::new(cmd)
        .arg(project_root)
        .output()
        .map_err(|source| Error::XrefCommandSpawn {
            cmd: cmd.to_string(),
            source,
        });
    let output = output?;
    if !output.status.success() {
        let code = output.status.code().unwrap_or(1);
        return Err(Error::XrefCommandStatus { cmd: cmd.to_string(), code });
    }
    serde_json::from_slice(&output.stdout).map_err(|source| Error::XrefCommandJson {
        cmd: cmd.to_string(),
        source,
    })
}
fn run() -> Result<(), Error> {
    let root = find_project_root();
    let config = read_config(&root);
    let docs_cfg = config.docs.unwrap_or_default();

    let mut args = parse_args();
    if let Some(theme) = docs_cfg.d2_theme {
        args.d2_theme = theme;
    }
    if let Some(layout) = docs_cfg.d2_layout {
        args.d2_layout = layout;
    }

    let out_dir = args.out_dir.clone().unwrap_or_else(|| root.join("docs").join("html"));
    let md_root = args.md_root.clone().unwrap_or_else(|| root.join("expanded-md"));
    let md_out_dir = args.md_out_dir.clone().unwrap_or_else(|| root.join("docs").join("html-md"));
    let theme_dir = args.theme_dir.clone().unwrap_or_else(|| root.join("scripts").join("asciidoc-theme"));

    let all_html = if args.no_adoc {
        Vec::new()
    } else {
        render::render_docs(
            &root,
            &theme_dir,
            &out_dir,
            &args.specials,
            args.plantuml_jar.as_deref(),
            args.d2_theme,
            &args.d2_layout,
        )?
    };
    let existing_html: std::collections::HashSet<String> = all_html
        .iter()
        .filter_map(|p| p.strip_prefix(&out_dir).ok())
        .map(|r| r.to_string_lossy().replace('\\', "/"))
        .collect();

    let md_html = if !args.no_md && md_root.exists() {
        render::render_markdown_docs(
            &md_root,
            &theme_dir,
            &md_out_dir,
            args.plantuml_jar.as_deref(),
            args.d2_theme,
            &args.d2_layout,
        )?
    } else {
        Vec::new()
    };
    let existing_md_html: std::collections::HashSet<String> = md_html
        .iter()
        .filter_map(|p| p.strip_prefix(&md_out_dir).ok())
        .map(|r| r.to_string_lossy().replace('\\', "/"))
        .collect();

    let crates_dir = root.join("crates");

    let (xref_data, adoc_map) = if args.no_xref {
        (HashMap::new(), HashMap::new())
    } else if let Some(ref cmd) = args.xref_cmd {
        println!("xref: running '{cmd}'...");
        let data = run_xref_cmd(cmd, &root)?;
        println!("xref: {} entries", data.len());
        (data, HashMap::new())
    } else if crates_dir.exists() {
        println!("xref: analysing crates...");
        let data = xref::build_xref(&root, args.ai_xref);
        let adoc_map = xref::scan_adoc_file_declarations(&root, &crates_dir);
        println!("xref: {} modules indexed, {} adoc overrides", data.len(), adoc_map.len());
        (data, adoc_map)
    } else {
        (HashMap::new(), HashMap::new())
    };

    let xref_json_base = if args.no_adoc { &md_out_dir } else { &out_dir };
    let xref_json_path = xref_json_base.join("xref.json");
    let json = serde_json::to_string_pretty(&xref_data)
        .map_err(|source| Error::XrefJson { source })?;
    std::fs::write(&xref_json_path, &json).map_err(|source| Error::XrefWrite {
        path: xref_json_path.display().to_string(),
        source,
    })?;
    println!("xref: wrote {}", xref_json_path.display());

    if !args.no_adoc {
        inject::rewrite_doc_links(&out_dir);
        inject::inject_xref(&out_dir, &xref_data, &existing_html, &adoc_map);
        literate_index::generate_and_inject_all(&out_dir);
        inject::inject_chunk_ids(&out_dir);
    }
    if !md_html.is_empty() {
        let empty_adoc_map = HashMap::new();
        inject::rewrite_doc_links(&md_out_dir);
        inject::inject_xref(&md_out_dir, &xref_data, &existing_md_html, &empty_adoc_map);
        literate_index::generate_and_inject_all(&md_out_dir);
        inject::inject_chunk_ids(&md_out_dir);
    }
    Ok(())
}

fn main() -> miette::Result<()> {
    run()?;
    Ok(())
}
#[cfg(test)]
mod tests;
