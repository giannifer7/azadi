// crates/azadi-macros/src/evaluator/tests/test_utils.rs

use crate::evaluator::python::{PythonConfig, SubprocessEvaluator};

pub fn create_test_evaluator() -> SubprocessEvaluator {
    SubprocessEvaluator::new(PythonConfig::default())
}
