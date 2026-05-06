// weaveback-tangle/src/safe_writer.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::db::{WeavebackDb, DbError};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

mod accessors;
mod helpers;
mod paths;
mod write_flow;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SafeWriterError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Failed to create directory: {0}")]
    DirectoryCreationFailed(PathBuf),
    #[error("Failed to create backup for: {0}")]
    BackupFailed(PathBuf),
    #[error("File was modified externally: {0}")]
    ModifiedExternally(PathBuf),
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("Formatter error: {0}")]
    FormatterError(String),
    #[error("Database error: {0}")]
    DbError(#[from] DbError),
}
#[derive(Debug, Clone)]
pub struct SafeWriterConfig {
    pub buffer_size: usize,
    pub formatters: HashMap<String, String>, // file-extension → shell command
    /// Allow `@file ~/...` chunks to write outside the gen/ sandbox.
    /// Default `false`: tilde-expanded (absolute) paths are rejected unless
    /// the user explicitly passes `--allow-home`.
    pub allow_home: bool,
    /// Override modification detection for generated files and always rewrite
    /// them from the current literate source.
    pub force_generated: bool,
}

impl Default for SafeWriterConfig {
    fn default() -> Self {
        SafeWriterConfig {
            buffer_size: 8192,
            formatters: HashMap::new(),
            allow_home: false,
            force_generated: false,
        }
    }
}
pub struct SafeFileWriter {
    gen_base: PathBuf,
    db: WeavebackDb,
    config: SafeWriterConfig,
    /// Staging area: logical file name → temp file on disk.
    /// The NamedTempFile is kept alive here until after_write consumes it.
    staging: HashMap<String, NamedTempFile>,
}

impl SafeFileWriter {
    pub fn new<P: AsRef<Path>>(gen_base: P) -> Result<Self, SafeWriterError> {
        Self::with_config(gen_base, SafeWriterConfig::default())
    }

    pub fn with_config<P: AsRef<Path>>(
        gen_base: P,
        config: SafeWriterConfig,
    ) -> Result<Self, SafeWriterError> {
        fs::create_dir_all(gen_base.as_ref())
            .map_err(|_| SafeWriterError::DirectoryCreationFailed(gen_base.as_ref().to_path_buf()))?;
        let gen_base = gen_base
            .as_ref()
            .canonicalize()
            .map_err(SafeWriterError::IoError)?;

        let db = WeavebackDb::open_temp().map_err(SafeWriterError::DbError)?;

        Ok(SafeFileWriter {
            gen_base,
            db,
            config,
            staging: HashMap::new(),
        })
    }
}
