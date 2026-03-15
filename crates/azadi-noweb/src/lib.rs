pub mod db;
pub mod noweb;
pub mod safe_writer;

#[cfg(test)]
mod tests;

pub use noweb::ChunkError;

use db::DbError;
use safe_writer::SafeWriterError;
use std::fmt;

#[derive(Debug)]
pub enum AzadiError {
    Chunk(ChunkError),
    SafeWriter(SafeWriterError),
    Db(DbError),
}

impl fmt::Display for AzadiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AzadiError::Chunk(e) => write!(f, "Chunk error: {}", e),
            AzadiError::SafeWriter(e) => write!(f, "Safe writer error: {}", e),
            AzadiError::Db(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for AzadiError {}

impl From<ChunkError> for AzadiError {
    fn from(err: ChunkError) -> Self {
        AzadiError::Chunk(err)
    }
}

impl From<SafeWriterError> for AzadiError {
    fn from(err: SafeWriterError) -> Self {
        AzadiError::SafeWriter(err)
    }
}

impl From<DbError> for AzadiError {
    fn from(err: DbError) -> Self {
        AzadiError::Db(err)
    }
}

impl From<std::io::Error> for AzadiError {
    fn from(err: std::io::Error) -> Self {
        AzadiError::SafeWriter(SafeWriterError::IoError(err))
    }
}

pub use crate::db::{AzadiDb, NowebMapEntry};
pub use crate::noweb::Clip;
pub use crate::safe_writer::SafeFileWriter;
pub use crate::safe_writer::SafeWriterConfig;
