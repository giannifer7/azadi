// weaveback-docgen/src/main.rs
// I'd Really Rather You Didn't edit this generated file.

mod adoc_scan;
mod args;
mod config;
mod d2;
mod error;
mod inject;
mod literate_index;
mod plantuml;
mod render;
mod xref;
mod xref_cmd;

use std::collections::HashMap;
use std::path::PathBuf;

use args::parse_args;
use config::read_config;
use error::Error;
use xref_cmd::run_xref_cmd;

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
