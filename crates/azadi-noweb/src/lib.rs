// src/lib.rs
//
// A single AzadiError merges chunk expansions + safe-writer logic.

pub mod noweb;
pub mod safe_writer;

use std::{error::Error, io};

#[derive(Debug)]
pub enum AzadiError {
    RecursionLimit {
        chunk: String,
        file_name: String,
        line: usize,
    },
    RecursiveReference {
        chunk: String,
        file_name: String,
        line: usize,
    },
    UndefinedChunk {
        chunk: String,
        file_name: String,
        line: usize,
    },
    FileChunkRedefinition {
        file_chunk: String,
        file_name: String,
        line: usize,
    },
    IoError(io::Error),
    SecurityViolation(String),
    ModifiedExternally(String),
}

impl std::fmt::Display for AzadiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzadiError::RecursionLimit {
                chunk,
                file_name,
                line,
            } => {
                write!(
                    f,
                    "Error: {} line {}: recursion limit in '{}'",
                    file_name,
                    line + 1,
                    chunk
                )
            }
            AzadiError::RecursiveReference {
                chunk,
                file_name,
                line,
            } => {
                write!(
                    f,
                    "Error: {} line {}: recursive reference in '{}'",
                    file_name,
                    line + 1,
                    chunk
                )
            }
            AzadiError::UndefinedChunk {
                chunk,
                file_name,
                line,
            } => {
                write!(
                    f,
                    "Error: {} line {}: chunk '{}' is undefined",
                    file_name,
                    line + 1,
                    chunk
                )
            }
            AzadiError::FileChunkRedefinition {
                file_chunk,
                file_name,
                line,
            } => {
                write!(
                    f,
                    "Error: {} line {}: file chunk '{}' is already defined (use @replace)",
                    file_name,
                    line + 1,
                    file_chunk
                )
            }
            AzadiError::IoError(e) => {
                write!(f, "IO error: {}", e)
            }
            AzadiError::SecurityViolation(msg) => {
                write!(f, "Security violation: {}", msg)
            }
            AzadiError::ModifiedExternally(msg) => {
                write!(f, "File was modified externally: {}", msg)
            }
        }
    }
}

impl Error for AzadiError {}

impl From<io::Error> for AzadiError {
    fn from(e: io::Error) -> Self {
        AzadiError::IoError(e)
    }
}
