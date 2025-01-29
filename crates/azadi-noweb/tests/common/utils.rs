// <[@file tests/common/utils.rs]>=
// crates/azadi-noweb/tests/common/utils.rs

use azadi_noweb::safe_writer::SafeFileWriter;
use tempfile::TempDir;

pub fn create_test_writer() -> (TempDir, SafeFileWriter) {
    let temp = TempDir::new().unwrap();
    let gen_path = temp.path().join("gen");
    let work_dir = temp.path().join("work_dir");
    let writer = SafeFileWriter::new(gen_path, work_dir, true);
    (temp, writer)
}
// $$
