// crates/azadi-macros/src/evaluator/eval_api.rs

use std::fs;
use std::path::{Path, PathBuf};

use super::core::Evaluator;
use super::errors::{EvalError, EvalResult};
use super::state::EvalConfig;

pub fn eval_string(
    source: &str,
    real_path: Option<&Path>,
    evaluator: &mut Evaluator,
) -> Result<String, EvalError> {
    let path_for_parsing = match real_path {
        Some(rp) => rp.to_path_buf(),
        None => PathBuf::from(format!("<string-{}>", evaluator.num_source_files())),
    };
    let ast = evaluator.parse_string(source, &path_for_parsing)?;
    if let Some(rp) = real_path {
        evaluator.set_current_file(rp.to_path_buf());
    }
    evaluator.evaluate(&ast)
}

pub fn eval_file(
    input_file: &Path,
    output_file: &Path,
    evaluator: &mut Evaluator,
) -> EvalResult<()> {
    let content = fs::read_to_string(input_file)
        .map_err(|e| EvalError::Runtime(format!("Cannot read {input_file:?}: {e}")))?;

    let expanded = eval_string(&content, Some(input_file), evaluator)?;

    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| EvalError::Runtime(format!("Cannot create dir {parent:?}: {e}")))?;
    }

    fs::write(output_file, expanded.as_bytes())
        .map_err(|e| EvalError::Runtime(format!("Cannot write {output_file:?}: {e}")))?;

    Ok(())
}

pub fn eval_file_with_config(
    input_file: &Path,
    output_file: &Path,
    config: EvalConfig,
) -> EvalResult<()> {
    let mut evaluator = Evaluator::new(config);
    eval_file(input_file, output_file, &mut evaluator)
}

pub fn eval_files(
    inputs: &[PathBuf],
    output_dir: &Path,
    evaluator: &mut Evaluator,
) -> EvalResult<()> {
    fs::create_dir_all(output_dir)
        .map_err(|e| EvalError::Runtime(format!("Cannot create {output_dir:?}: {e}")))?;

    for input_path in inputs {
        let mut out_name = match input_path.file_name() {
            Some(n) => n.to_os_string(),
            None => "output".into(),
        };
        out_name.push(".txt");
        let out_file = output_dir.join(out_name);

        eval_file(input_path, &out_file, evaluator)?;
    }
    Ok(())
}

pub fn eval_files_with_config(
    inputs: &[PathBuf],
    output_dir: &Path,
    config: EvalConfig,
) -> EvalResult<()> {
    let mut evaluator = Evaluator::new(config);
    eval_files(inputs, output_dir, &mut evaluator)
}

pub fn eval_string_with_defaults(source: &str) -> EvalResult<String> {
    let mut evaluator = Evaluator::new(EvalConfig::default());
    eval_string(source, None, &mut evaluator)
}
