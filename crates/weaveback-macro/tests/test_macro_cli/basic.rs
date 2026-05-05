// weaveback-macro/tests/test_macro_cli/basic.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::support::{cargo_weaveback_macro_cli, create_test_file};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_basic_macro_processing() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let input = create_test_file(
        &temp_path,
        "input.txt",
        r#"%def(hello, World)
Hello %hello()!"#,
    );
    assert!(input.exists(), "Input file should exist");

    let out_file = temp_path.join("output.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--output")
        .arg(&out_file)
        .arg(&input);

    let output = cmd.output()?;
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());
    assert!(out_file.exists(), "Output file should exist");

    let output_content = fs::read_to_string(&out_file)?;
    assert_eq!(output_content.trim(), "Hello World!");

    Ok(())
}

// 1) Test the help message
#[test]
fn test_cli_help() -> Result<(), Box<dyn std::error::Error>> {
    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--help");

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "Expected 'weaveback-macro --help' to succeed."
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("weaveback-macro"),
        "Help output did not mention 'weaveback-macro'"
    );
    assert!(
        stdout.contains("--output"),
        "Help output did not mention '--output'"
    );

    Ok(())
}

// 2) Test passing a non-existent input file
#[test]
fn test_missing_input_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let missing_input = temp_path.join("not_real.txt");
    let out_file = temp_path.join("output.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--output")
        .arg(&out_file)
        .arg(&missing_input);

    let output = cmd.output()?;
    assert!(
        !output.status.success(),
        "CLI was expected to fail on missing file."
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("(missing_input) stderr:\n{stderr}");
    assert!(
        stderr.contains("Input file does not exist"),
        "Should mention 'Input file does not exist' in error."
    );

    Ok(())
}

// 3) Test multiple input files in a single run
#[test]
fn test_multiple_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let input1 = create_test_file(
        &temp_path,
        "file1.txt",
        "%def(macro1, MACRO_ONE)\n%macro1()",
    );
    let input2 = create_test_file(
        &temp_path,
        "file2.txt",
        "%def(macro2, MACRO_TWO)\n%macro2()",
    );

    let out_file = temp_path.join("combined_output.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--output")
        .arg(&out_file)
        .arg(&input1)
        .arg(&input2);

    let output = cmd.output()?;
    assert!(output.status.success());

    let content = fs::read_to_string(&out_file)?;
    assert!(
        content.contains("MACRO_ONE"),
        "Expected 'MACRO_ONE' in combined output file."
    );
    assert!(
        content.contains("MACRO_TWO"),
        "Expected 'MACRO_TWO' in combined output file."
    );

    Ok(())
}

