// crates/azadi-macros/src/macro_api.rs

use crate::evaluator::evaluator::EvalConfig;
use crate::evaluator::evaluator::EvalError;
use crate::evaluator::evaluator::Evaluator;
use std::fs;
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

pub fn process_file(
    input_file: &Path,
    output_file: &Path,
    evaluator: &mut Evaluator,
) -> Result<(), EvalError> {
    let content = fs::read_to_string(input_file)
        .map_err(|e| EvalError::Runtime(format!("Cannot read {input_file:?}: {e}")))?;
    let expanded = process_string(&content, Some(input_file), evaluator)?;
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| EvalError::Runtime(format!("Cannot create dir {parent:?}: {e}")))?;
    }
    fs::write(output_file, expanded)
        .map_err(|e| EvalError::Runtime(format!("Cannot write {output_file:?}: {e}")))?;
    Ok(())
}

pub fn process_file_from_config(
    input_file: &Path,
    output_file: &Path,
    config: EvalConfig,
) -> Result<(), EvalError> {
    let mut evaluator = Evaluator::new(config);
    process_file(input_file, output_file, &mut evaluator)
}

pub fn process_files(
    inputs: &[PathBuf],
    output_dir: &Path,
    evaluator: &mut Evaluator,
) -> Result<(), EvalError> {
    fs::create_dir_all(output_dir)
        .map_err(|e| EvalError::Runtime(format!("Cannot create {output_dir:?}: {e}")))?;

    for input_path in inputs {
        let mut out_name = match input_path.file_name() {
            Some(n) => n.to_os_string(),
            None => "output".into(),
        };
        out_name.push(".txt");
        let out_file = output_dir.join(out_name);

        process_file(input_path, &out_file, evaluator)?;
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
