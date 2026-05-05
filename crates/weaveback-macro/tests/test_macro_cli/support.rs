// weaveback-macro/tests/test_macro_cli/support.rs
// I'd Really Rather You Didn't edit this generated file.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    let mut file = fs::File::create(&path).unwrap();
    write!(file, "{}", content).unwrap();
    path.canonicalize().unwrap()
}

pub(crate) fn cargo_weaveback_macro_cli() -> Result<escargot::CargoRun, Box<dyn std::error::Error>> {
    Ok(escargot::CargoBuild::new()
        .bin("weaveback-macro")
        .current_release()
        .current_target()
        .run()?)
}

