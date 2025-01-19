// crates/azadi-noweb/tests/common/utils.rs

use azadi_noweb::safe_writer::SafeFileWriter;
use azadi_noweb::AzadiError;
use std::{fs, io::Write, path::PathBuf};
use tempfile::TempDir;

pub fn create_test_writer() -> (TempDir, SafeFileWriter) {
    let temp = TempDir::new().unwrap();
    let gen_path = temp.path().join("gen");
    fs::create_dir_all(&gen_path).unwrap();
    let writer = SafeFileWriter::new(gen_path);
    (temp, writer)
}

pub fn write_file(
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
