// crates/azadi-macros/src/evaluator/tests/test_utils.rs

use crate::evaluator::{EvalConfig, Evaluator};
use std::path::Path;

/// Create an EvalConfig whose include path and backup dir both live inside
/// `temp_dir`.  Callers must keep the `TempDir` alive for the test duration;
/// on drop, Rust cleans up the directory automatically.
pub fn config_in_temp_dir(temp_dir: &Path) -> EvalConfig {
    EvalConfig {
        include_paths: vec![temp_dir.to_path_buf()],
        backup_dir: temp_dir.join("_azadi_work"),
        ..Default::default()
    }
}

/// Convenience wrapper: evaluator whose working files stay inside `temp_dir`.
pub fn evaluator_in_temp_dir(temp_dir: &Path) -> Evaluator {
    Evaluator::new(config_in_temp_dir(temp_dir))
}
