// weaveback-tangle/src/safe_writer/paths.rs
// I'd Really Rather You Didn't edit this generated file.

use super::SafeWriterError;
use std::path::Path;

pub(in crate::safe_writer) fn validate_filename(path: &Path) -> Result<(), SafeWriterError> {
    use std::path::Component;

    if path.is_absolute() {
        return Err(SafeWriterError::SecurityViolation(format!(
            "Absolute paths are not allowed: {}",
            path.display()
        )));
    }

    let filename = path.to_string_lossy();
    if filename.len() >= 2 {
        let mut chars = filename.chars();
        let first = chars.next().unwrap();
        let second = chars.next().unwrap();
        if second == ':' && first.is_ascii_alphabetic() {
            return Err(SafeWriterError::SecurityViolation(format!(
                "Windows-style absolute paths are not allowed: {}",
                filename
            )));
        }
    }

    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(SafeWriterError::SecurityViolation(format!(
            "Path traversal detected (..): {}",
            path.display()
        )));
    }

    Ok(())
}

