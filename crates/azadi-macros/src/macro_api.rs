// crates/azadi-macros/src/macro_api.rs

use crate::evaluator::{EvalConfig, EvalError, Evaluator};
use crate::types::{ASTNode, NodeKind};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn process_string(
    source: &str,
    real_path: Option<&Path>,
    evaluator: &mut Evaluator,
) -> Result<Vec<u8>, EvalError> {
    let path_for_parsing = match real_path {
        Some(rp) => rp.to_path_buf(),
        None => PathBuf::from(format!("<string-{}>", evaluator.num_source_files())),
    };
    let ast = evaluator.parse_string(source, &path_for_parsing)?;
    if let Some(rp) = real_path {
        evaluator.set_current_file(rp.to_path_buf());
    }
    let output_string = evaluator.evaluate(&ast)?;
    Ok(output_string.into_bytes())
}

pub fn process_file_with_writer(
    input_file: &Path,
    writer: &mut dyn Write,
    evaluator: &mut Evaluator,
) -> Result<(), EvalError> {
    let content = fs::read_to_string(input_file)
        .map_err(|e| EvalError::Runtime(format!("Cannot read {input_file:?}: {e}")))?;
    let expanded = process_string(&content, Some(input_file), evaluator)?;
    writer
        .write_all(&expanded)
        .map_err(|e| EvalError::Runtime(format!("Cannot write to output: {e}")))?;
    Ok(())
}

pub fn process_file(
    input_file: &Path,
    output_file: &Path,
    evaluator: &mut Evaluator,
) -> Result<(), EvalError> {
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| EvalError::Runtime(format!("Cannot create dir {parent:?}: {e}")))?;
    }
    let mut file = fs::File::create(output_file)
        .map_err(|e| EvalError::Runtime(format!("Cannot create {output_file:?}: {e}")))?;
    process_file_with_writer(input_file, &mut file, evaluator)
}

pub fn process_files(
    inputs: &[PathBuf],
    output_path: &Path,
    evaluator: &mut Evaluator,
) -> Result<(), EvalError> {
    // Determine the appropriate writer based on output_path
    let mut stdout_handle;
    let mut file_handle;
    let writer: &mut dyn Write = if output_path.to_string_lossy() == "-" {
        stdout_handle = io::stdout();
        &mut stdout_handle
    } else {
        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| EvalError::Runtime(format!("Cannot create dir {parent:?}: {e}")))?;
        }

        // Open the output file
        file_handle = fs::File::create(output_path)
            .map_err(|e| EvalError::Runtime(format!("Cannot create {output_path:?}: {e}")))?;
        &mut file_handle
    };

    // Process all input files with the selected writer
    for input_path in inputs {
        process_file_with_writer(input_path, writer, evaluator)?;
    }

    Ok(())
}

pub fn process_files_from_config(
    inputs: &[PathBuf],
    output_dir: &Path,
    config: EvalConfig,
) -> Result<(), EvalError> {
    let mut evaluator = Evaluator::new(config);
    process_files(inputs, output_dir, &mut evaluator)
}

pub fn process_string_defaults(source: &str) -> Result<Vec<u8>, EvalError> {
    let mut evaluator = Evaluator::new(EvalConfig::default());
    process_string(source, None, &mut evaluator)
}

/// Parse `source` (without evaluating) and return the literal path argument of
/// every top-level `%include(...)` or `%import(...)` call found in the AST.
///
/// Paths that are computed via a macro call (e.g. `%include(%mypath())`) cannot
/// be statically resolved and are silently skipped.  On parse error the function
/// returns an empty vec rather than propagating the error, so callers can treat
/// a broken file as having no static includes.
pub fn collect_direct_includes(
    source: &str,
    file_path: Option<&Path>,
    config: &EvalConfig,
) -> Vec<String> {
    let mut evaluator = Evaluator::new(config.clone());
    let path = file_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("<string>"));
    let Ok(ast) = evaluator.parse_string(source, &path) else {
        return vec![];
    };
    let mut result = Vec::new();
    collect_includes_in_node(&ast, &evaluator, &mut result);
    result
}

fn collect_includes_in_node(node: &ASTNode, evaluator: &Evaluator, out: &mut Vec<String>) {
    if node.kind == NodeKind::Macro {
        let name = evaluator.node_text(node);
        if name == "include" || name == "import" {
            if let Some(param) = node.parts.first() {
                let path = literal_text(param, evaluator).trim().to_string();
                if !path.is_empty() {
                    out.push(path);
                }
            }
            return; // don't descend into the argument
        }
    }
    for child in &node.parts {
        collect_includes_in_node(child, evaluator, out);
    }
}

/// Concatenate literal text from a node, recursing into Composite/Param/Block.
/// Returns an empty string for any node that requires macro evaluation.
fn literal_text(node: &ASTNode, evaluator: &Evaluator) -> String {
    match node.kind {
        NodeKind::Text | NodeKind::Ident | NodeKind::Punct => evaluator.node_text(node),
        NodeKind::Space => " ".to_string(),
        NodeKind::Composite | NodeKind::Param | NodeKind::Block => node
            .parts
            .iter()
            .map(|c| literal_text(c, evaluator))
            .collect(),
        // Macro calls inside the path argument → cannot statically resolve.
        _ => String::new(),
    }
}
