// crates/azadi-macros/src/evaluator/tests/test_utils.rs
/*
use crate::evaluator::python::{PythonConfig, SubprocessEvaluator};

pub fn create_test_evaluator() -> SubprocessEvaluator {
    SubprocessEvaluator::new(PythonConfig::default())
}
*/
use crate::evaluator::python::{PyO3Evaluator, PythonConfig};

pub fn create_test_evaluator() -> PyO3Evaluator {
    PyO3Evaluator::new(PythonConfig::default()).expect("Failed to create PyO3Evaluator")
}
