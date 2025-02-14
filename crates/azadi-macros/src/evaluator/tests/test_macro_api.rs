// crates/azadi-macros/src/evaluator/tests/test_macro_api.rs

use crate::evaluator::{EvalConfig, Evaluator};
use crate::macro_api::{
    process_file, process_file_from_config, process_files_from_config, process_string,
    process_string_defaults,
};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    let mut file = fs::File::create(&path).unwrap();
    write!(file, "{}", content).unwrap();
    path
}

#[test]
fn test_process_string_basic() {
    let result = process_string_defaults("Hello %def(test, World) %test()").unwrap();
    assert_eq!(String::from_utf8(result).unwrap(), "Hello  World");
}

#[test]
fn test_include_basic() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create an included file
    let _include_file = create_temp_file(&temp_dir, "include.txt", "test");

    // Create our test source that includes it
    let main_file = create_temp_file(&temp_dir, "main.txt", "%include(include.txt)");

    // Set up config with include path
    let mut config = EvalConfig::default();
    config.include_paths = vec![temp_dir.path().to_path_buf()];
    let mut evaluator = Evaluator::new(config);

    let output_file = temp_dir.path().join("output.txt");

    // Process using the file API
    process_file(&main_file, &output_file, &mut evaluator)?;

    let result = fs::read_to_string(output_file)?;
    assert_eq!(result.trim(), "test");

    Ok(())
}

#[test]
fn test_process_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create input file
    let input_path = create_temp_file(
        &temp_dir,
        "input.txt",
        "%def(hello, name, Hello %(name)!)\n%hello(World)",
    );

    // Create output path
    let output_path = temp_dir.path().join("output.txt");

    // Process file
    let mut config = EvalConfig::default();
    config.backup_dir = temp_dir.path().join("backup");
    process_file_from_config(&input_path, &output_path, config).unwrap();

    // Verify output
    let output_content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(output_content.trim(), "Hello World!");
}

#[test]
fn test_process_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create input files
    let input1 = create_temp_file(&temp_dir, "input1.txt", "%def(msg, Hello)\n%msg()");
    let input2 = create_temp_file(&temp_dir, "input2.txt", "%def(msg, Goodbye)\n%msg()");

    // Set up output directory
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    // Process files
    let mut config = EvalConfig::default();
    config.backup_dir = temp_dir.path().join("backup");
    process_files_from_config(&[input1, input2], &output_dir, config).unwrap();

    // Verify outputs
    let output1 = fs::read_to_string(output_dir.join("input1.txt.txt")).unwrap();
    let output2 = fs::read_to_string(output_dir.join("input2.txt.txt")).unwrap();

    assert_eq!(output1.trim(), "Hello");
    assert_eq!(output2.trim(), "Goodbye");
}

#[test]
fn test_process_string_with_error() {
    let result = process_string_defaults("%undefined_macro()");
    assert!(result.is_err());
}

#[test]
fn test_process_string_with_nested_macros() {
    let source = r#"
        %def(inner, value, Inside: %(value))
        %def(outer, arg, Outside: %inner(%(arg)))
        %outer(test)
    "#;

    let result = process_string_defaults(source).unwrap();
    let output = String::from_utf8(result).unwrap();
    assert!(output.contains("Outside: Inside: test"));
}

#[test]
fn test_process_string_with_special_chars() {
    let mut config = EvalConfig::default();
    config.special_char = '@';
    let mut evaluator = Evaluator::new(config);

    let result = process_string(
        "@def(test, value, Result: @(value))@test(works)",
        None,
        &mut evaluator,
    )
    .unwrap();

    assert_eq!(String::from_utf8(result).unwrap().trim(), "Result: works");
}

#[test]
fn test_process_files_with_shared_macros() {
    let temp_dir = TempDir::new().unwrap();

    // First file defines a macro
    let file1 = create_temp_file(&temp_dir, "file1.txt", "%def(shared, Shared content)");

    // Second file uses the macro
    let file2 = create_temp_file(&temp_dir, "file2.txt", "%shared()");

    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    // Process files
    let mut config = EvalConfig::default();
    config.backup_dir = temp_dir.path().join("backup");
    process_files_from_config(&[file1, file2], &output_dir, config).unwrap();

    // Verify outputs
    let output1 = fs::read_to_string(output_dir.join("file1.txt.txt")).unwrap();
    let output2 = fs::read_to_string(output_dir.join("file2.txt.txt")).unwrap();

    assert!(output1.is_empty()); // First file just defines the macro
    assert_eq!(output2.trim(), "Shared content"); // Second file uses it
}
