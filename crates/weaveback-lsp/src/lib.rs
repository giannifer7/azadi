// weaveback-lsp/src/lib.rs
// I'd Really Rather You Didn't edit this generated file.

mod client;
mod nav;
mod registry;

pub use client::{LspClient, LspError};
pub use registry::get_lsp_config;

#[cfg(test)]
mod tests;
