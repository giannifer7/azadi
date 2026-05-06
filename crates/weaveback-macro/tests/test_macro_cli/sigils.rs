// weaveback-macro/tests/test_macro_cli/sigils.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::support::{cargo_weaveback_macro_cli, create_test_file};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_custom_sigil() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let input = create_test_file(
        &temp_path,
        "input_at.txt",
        "@def(test_macro, Hello from custom char)\n@test_macro()",
    );
    let out_file = temp_path.join("output_at.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--sigil")
        .arg("@")
        .arg("--output")
        .arg(&out_file)
        .arg(&input);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI run with custom sigil should succeed."
    );

    let content = fs::read_to_string(&out_file)?;
    assert!(
        content.contains("Hello from custom char"),
        "Expected to see expansion with '@' as the macro char."
    );

    Ok(())
}

#[test]
fn test_unicode_sigil() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let input = create_test_file(
        &temp_path,
        "input_section.txt",
        "§def(test_macro, Hello from unicode char)\n§test_macro()",
    );
    let out_file = temp_path.join("output_section.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--sigil")
        .arg("§")
        .arg("--output")
        .arg(&out_file)
        .arg(&input);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI run with unicode sigil should succeed."
    );

    let content = fs::read_to_string(&out_file)?;
    assert!(
        content.contains("Hello from unicode char"),
        "Expected to see expansion with '§' as the macro char."
    );

    Ok(())
}
