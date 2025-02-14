// crates/azadi-macros/src/evaluator/mod.rs

mod builtins;
mod case_conversion;
mod core;
mod errors;
mod eval_api;
pub mod lexer_parser; // Make this public
mod python;
mod source_utils;
mod state;

#[cfg(test)]
mod tests;

// Re-export everything needed by the rest of the crate
pub use crate::types::ASTNode;
pub use core::Evaluator;
pub use errors::{EvalError, EvalResult, Terminate};
pub use eval_api::{
    eval_file, eval_file_with_config, eval_files, eval_files_with_config, eval_string,
    eval_string_with_defaults,
};
pub use lexer_parser::lex_parse_content;
pub use state::{EvalConfig, MacroDefinition};

#[cfg(feature = "pyo3")]
pub use python::pyo3_evaluator::PyO3Evaluator;

pub use python::{PythonConfig, PythonEvaluator, SecurityLevel, SubprocessEvaluator};
