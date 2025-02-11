// azadi/crates/azadi-macros/src/evaluator/mod.rs

// The top-level module for the “evaluator” folder, integrating everything.

pub mod builtins;
pub mod case_conversion;
pub mod evaluator;
pub mod lexer_parser;
pub mod source_utils;

#[cfg(test)]
mod tests; // holds multiple small test files

pub use builtins::default_builtins;
pub use evaluator::{EvalError, EvalResult, Evaluator, MacroDefinition};
pub use lexer_parser::lex_parse_content;
pub use source_utils::{backup_source_file, modify_source};
