// azadi-core/src/lib.rs
use std::path::Path;
use thiserror::Error;

pub mod error;
pub mod macro_;
pub mod noweb;
pub mod pipeline;

pub use error::Error;

/// High-level configuration for all operations
#[derive(Debug, Clone)]
pub struct Config {
    pub input_files: Vec<String>,
    pub output_dir: String,
    pub work_dir: String,
    pub syntax: SyntaxConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone)]
pub struct SyntaxConfig {
    pub special_char: char,
    pub open_delim: String,
    pub close_delim: String,
    pub chunk_end: String,
    pub comment_markers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FeatureFlags {
    pub pydef: bool,
    pub save_intermediates: bool,
    pub dump_ast: bool,
}

/// Main entry point for the core API
pub struct Azadi {
    config: Config,
}

impl Azadi {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Process files through macro expansion only
    pub fn process_macros(&self, inputs: &[impl AsRef<Path>]) -> Result<Vec<String>, Error> {
        // Implement macro processing
        todo!()
    }

    /// Process files through noweb only
    pub fn process_noweb(&self, inputs: &[impl AsRef<Path>]) -> Result<Vec<String>, Error> {
        // Implement noweb processing
        todo!()
    }

    /// Process files through complete pipeline
    pub fn process(&self, inputs: &[impl AsRef<Path>]) -> Result<Vec<String>, Error> {
        // Implement full pipeline
        todo!()
    }
}

// Re-exports for convenience
pub mod prelude {
    pub use super::{Azadi, Config, Error, FeatureFlags, SyntaxConfig};
}
