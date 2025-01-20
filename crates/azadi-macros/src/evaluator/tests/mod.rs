// crates/azadi-macros/src/evaluator/tests/mod.rs
// mod test_comment_spaces;
mod test_def;
// mod test_equal;
// mod test_eval;
// mod test_here;
// mod test_if;
// mod test_include;
mod test_macros;
// mod test_param_builder;
// mod test_position_and_length;
// mod test_token_lexing;
// mod test_var;
//mod test_ast_builder;

/*
use crate::evaluator::evaluator::EvalConfig;
use std::fs;
//use std::path::PathBuf;
use tempfile::TempDir;

fn setup() -> (EvalConfig, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_path = temp_dir.path();

    let config = EvalConfig {
        special_char: '%',
        pydef: false,
        include_paths: vec![temp_path.join("includes")],
        backup_dir: temp_path.join("_azadi_work"),
    };

    fs::create_dir_all(&config.include_paths[0]).unwrap();
    fs::create_dir_all(&config.backup_dir).unwrap();

    (config, temp_dir)
}
fn setup() -> (EvalConfig, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_path = temp_dir.path();

    let config = EvalConfig {
        special_char: '%',
        pydef: false,
        include_paths: vec![temp_path.join("includes")],
        backup_dir: temp_path.join("_azadi_work"),
    };

    fs::create_dir_all(&config.include_paths[0]).unwrap_or_else(|e| {
        panic!(
            "Failed to create includes directory {}: {}",
            config.include_paths[0].display(),
            e
        );
    });
    fs::create_dir_all(&config.backup_dir).unwrap_or_else(|e| {
        panic!(
            "Failed to create backup directory {}: {}",
            config.backup_dir.display(),
            e
        );
    });

    (config, temp_dir)
}

*/
