// crates/azadi-noweb/tests/common/mod.rs
use azadi_noweb::noweb::Clip;
use azadi_noweb::safe_writer::SafeFileWriter;
use std::fs;
use tempfile::TempDir;

mod test_data;
mod utils;

pub use test_data::*;
pub use utils::*;

pub struct TestSetup {
    pub _temp_dir: TempDir,
    pub clip: Clip,
}

impl TestSetup {
    pub fn new(comment_markers: &[&str]) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let gen_path = temp_dir.path().join("gen");
        fs::create_dir_all(&gen_path).unwrap();
        let safe_writer = SafeFileWriter::new(gen_path);

        let comment_markers = comment_markers
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let clip = Clip::new(safe_writer, "<<", ">>", "@", &comment_markers);

        TestSetup {
            _temp_dir: temp_dir,
            clip,
        }
    }
}
