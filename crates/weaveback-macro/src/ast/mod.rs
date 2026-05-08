// weaveback-macro/src/ast/mod.rs
// I'd Really Rather You Didn't edit this generated file.

// crates/weaveback-macro/src/ast/mod.rs — generated from ast.adoc
use thiserror::Error;
mod build;
mod strip;
pub mod serialization;

pub use serialization::{dump_macro_ast, serialize_ast_nodes};
#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
pub enum ASTError {
    #[error("Parser error: {0}")]
    Parser(String),
    #[error("Node not found: {0}")]
    NodeNotFound(usize),
    #[error("Processing error: {0}")]
    Other(String),
}

impl From<String> for ASTError {
    fn from(error: String) -> Self {
        ASTError::Other(error)
    }
}

pub use build::build_ast;
pub use strip::strip_space_before_comments;

#[cfg(test)]
pub(crate) use build::analyze_param;
#[cfg(test)]
pub(crate) use crate::types::ASTNode;
