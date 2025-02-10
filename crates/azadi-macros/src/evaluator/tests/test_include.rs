use crate::evaluator::evaluator::EvalConfig;
use crate::evaluator::{EvalError, Evaluator};
use crate::macro_api::process_string;
use crate::macro_api::process_string_defaults;
use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::path::Path;
use tempfile::NamedTempFile;
use tempfile::TempDir;

/// Helper function to create an Evaluator with a temporary directory as the include path
fn create_evaluator_with_temp_dir(temp_dir: &Path) -> Evaluator {
    let config = EvalConfig {
        include_paths: vec![temp_dir.to_path_buf()],
        ..Default::default()
    };
    Evaluator::new(config)
}

#[test]
fn test_include_basic() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.path();

    // Create a test file `header.txt` in the temporary directory
    let header_path = temp_dir_path.join("header.txt");
    fs::write(&header_path, "Hello from header.txt").expect("Failed to write header.txt");

    let mut evaluator = create_evaluator_with_temp_dir(temp_dir_path);

    // Test including a file
    let result = process_string("%include(header.txt)", None, &mut evaluator).unwrap();
    assert_eq!(String::from_utf8(result).unwrap(), "Hello from header.txt");
}

#[test]
fn test_include_with_macros() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.path();

    // Create a test file `macros.txt` in the temporary directory
    let macros_path = temp_dir_path.join("macros.txt");
    fs::write(
        &macros_path,
        r#"
        %def(greet, name, %{
            Hello, %(name)!
        %})
        %def(farewell, name, %{
            Goodbye, %(name)!
        %})
    "#,
    )
    .expect("Failed to write macros.txt");

    let mut evaluator = create_evaluator_with_temp_dir(temp_dir_path);

    // Test including a file with macros
    let result = process_string(
        r#"
        %include(macros.txt)
        %greet(World)
        %farewell(Friend)
        "#,
        None,
        &mut evaluator,
    )
    .unwrap();

    // Trim the result to remove extra whitespace and newlines
    let trimmed_result = String::from_utf8(result).unwrap().trim().to_string();

    assert_eq!(
        trimmed_result,
        "Hello, World!\n        \n        \n            Goodbye, Friend!"
    );
}

#[test]
fn test_include_missing_file() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.path();

    let mut evaluator = create_evaluator_with_temp_dir(temp_dir_path);

    // Test including a non-existent file
    let result = process_string("%include(missing.txt)", None, &mut evaluator);
    assert!(matches!(result, Err(EvalError::IncludeNotFound(_))));
}

#[test]
fn test_include_self_inclusion() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.path();

    // Create a test file `self_include.txt` that includes itself
    let self_include_path = temp_dir_path.join("self_include.txt");
    fs::write(&self_include_path, "%include(self_include.txt)")
        .expect("Failed to write self_include.txt");

    let mut evaluator = create_evaluator_with_temp_dir(temp_dir_path);

    // Test self-inclusion
    let result = process_string("%include(self_include.txt)", None, &mut evaluator);
    assert!(matches!(result, Err(EvalError::CircularInclude(_))));
}

#[test]
fn test_include_mutual_inclusion() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.path();

    // Create two files that include each other
    let file_a_path = temp_dir_path.join("file_a.txt");
    fs::write(&file_a_path, "%include(file_b.txt)").expect("Failed to write file_a.txt");

    let file_b_path = temp_dir_path.join("file_b.txt");
    fs::write(&file_b_path, "%include(file_a.txt)").expect("Failed to write file_b.txt");

    let mut evaluator = create_evaluator_with_temp_dir(temp_dir_path);

    // Test mutual inclusion
    let result = process_string("%include(file_a.txt)", None, &mut evaluator);
    assert!(matches!(result, Err(EvalError::CircularInclude(_))));
}

#[test]
fn test_include_with_symlink() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.path();

    // Create a target file and a symlink to it
    let target_path = temp_dir_path.join("target.txt");
    fs::write(&target_path, "Hello from target.txt").expect("Failed to write target.txt");

    let symlink_path = temp_dir_path.join("symlink.txt");
    symlink(&target_path, &symlink_path).expect("Failed to create symlink");

    let mut evaluator = create_evaluator_with_temp_dir(temp_dir_path);

    // Test including a symlink
    let result = process_string("%include(symlink.txt)", None, &mut evaluator).unwrap();
    assert_eq!(String::from_utf8(result).unwrap(), "Hello from target.txt");
}

#[test]
/// Test that an %include inside a macro correctly scopes the included %defs.
///
/// The test simulates the following source:
///
/// ```text
/// %def(macro_with_include, param, %{
///     %include(TEMP_PATH)
///     %included_macro(%(param))
/// %})
/// %macro_with_include(test)
///
/// %included_macro(outside)
/// ```
///
/// Where the file at TEMP_PATH contains:
///
///     %def(included_macro, x, Included says: %(x)!)
///
/// The expected expansion is:
///
///     Included says: test
///
/// (The call to `%included_macro(outside)` outside the macro should produce nothing.)
fn test_include_scope() {
    // Create a temporary file that will act as our included definitions.
    let mut tmp = NamedTempFile::new().expect("Failed to create temp file");
    // Write a macro definition into the temporary file.
    // This defines a macro 'included_macro' that takes one parameter `x`.
    writeln!(tmp, "%def(included_macro, x, Included says: %(x)!)")
        .expect("Failed to write to temp file");
    // Get the path to the temporary file.
    let tmp_path = tmp.path().to_str().expect("Invalid temp file path");

    // Build a source string that defines a macro which includes our temporary file.
    let source = format!(
        r#"
%def(macro_with_include, param, %{{
    %include({tmp_path})
    %included_macro(%(param))
%}})
%macro_with_include(test)
%included_macro(outside)
"#
    );
    let result = process_string_defaults(&source);
    match result {
        Err(EvalError::UndefinedMacro(m)) => {
            assert_eq!(
                m, "included_macro",
                "Expected UndefinedMacro error for 'included_macro'"
            );
        }
        Err(e) => {
            panic!(
                "Expected UndefinedMacro error, but got a different error: {:?}",
                e
            );
        }
        Ok(output) => {
            panic!(
                    "Expected an error due to undefined macro, but processing succeeded with output: {:?}",
                    String::from_utf8_lossy(&output)
                );
        }
    }
}
