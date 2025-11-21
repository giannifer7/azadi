// src/tests/utils.rs
use std::{fs, io::Write, path::PathBuf};
use tempfile::TempDir;
use crate::{AzadiError, SafeFileWriter};

pub(crate) fn create_test_writer() -> (TempDir, SafeFileWriter) {
    let temp = TempDir::new().unwrap();
    let writer = SafeFileWriter::new(temp.path().join("gen"), temp.path().join("private"));
    (temp, writer)
}

pub(crate) fn write_file(
    writer: &mut SafeFileWriter,
    path: &PathBuf,
    content: &str,
) -> Result<(), AzadiError> {
    let private_path = writer.before_write(path)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", content)?;
    }
    Ok(writer.after_write(path)?)
}
