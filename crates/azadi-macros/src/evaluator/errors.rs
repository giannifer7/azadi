// crates/azadi-macros/src/evaluator/errors.rs

use std::io;
use thiserror::Error;

/// Custom type to signal termination (not an error).
#[derive(Debug)]
pub struct Terminate;

#[derive(Error, Debug)]
pub enum PyEvalError {
    #[error("Python execution error: {0}")]
    Execution(String),
    #[error("Security violation: {0}")]
    Security(String),
    #[error("Environment error: {0}")]
    Environment(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Undefined macro: {0}")]
    UndefinedMacro(String),

    #[error("Builtin error: {0}")]
    BuiltinError(String),

    #[error("Include not found: {0}")]
    IncludeNotFound(String),

    #[error("Circular include: {0}")]
    CircularInclude(String),

    #[error("Invalid usage: {0}")]
    InvalidUsage(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Terminate execution")]
    Terminate(Terminate),

    #[error("Python error: {0}")]
    Python(#[from] PyEvalError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type EvalResult<T> = Result<T, EvalError>;

impl From<String> for EvalError {
    fn from(s: String) -> Self {
        EvalError::Runtime(s)
    }
}
