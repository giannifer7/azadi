// weaveback-docgen/src/args.rs
// I'd Really Rather You Didn't edit this generated file.

use std::path::PathBuf;

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
pub(super) struct Args {
    pub(super) specials: Vec<char>,
    pub(super) xref_cmd: Option<String>,
    pub(super) no_xref: bool,
    pub(super) ai_xref: bool,
    pub(super) out_dir: Option<PathBuf>,
    pub(super) md_root: Option<PathBuf>,
    pub(super) md_out_dir: Option<PathBuf>,
    pub(super) no_md: bool,
    pub(super) no_adoc: bool,
    pub(super) theme_dir: Option<PathBuf>,
    pub(super) plantuml_jar: Option<PathBuf>,
    pub(super) d2_theme: u32,
    pub(super) d2_layout: String,
}
pub(super) fn parse_args_from(raw: &[String]) -> Args {
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

pub(super) fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().collect();
    parse_args_from(&raw)
}
