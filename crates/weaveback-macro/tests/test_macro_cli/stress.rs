// weaveback-macro/tests/test_macro_cli/stress.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::support::{cargo_weaveback_macro_cli, create_test_file};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_large_input() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let mut big_content = String::new();
    big_content.push_str("%def(say, HELLO)\n");
    for _ in 0..10_000 {
        big_content.push_str("%say()");
        big_content.push('\n');
    }

    let big_file = create_test_file(&temp_path, "big_file.txt", &big_content);
    let out_file = temp_path.join("output_big.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--output")
        .arg(&out_file)
        .arg(&big_file);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI should handle a large input file."
    );

    let out_content = fs::read_to_string(&out_file)?;
    let line_count = out_content.matches("HELLO").count();
    assert_eq!(
        line_count, 10_000,
        "Expected 10,000 expansions of HELLO in the large output."
    );

    Ok(())
}

