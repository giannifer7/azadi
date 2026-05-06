// weaveback-macro/tests/test_macro_cli/config.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::support::{cargo_weaveback_macro_cli, create_test_file};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_define_cli() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let input = create_test_file(&temp_path, "define.txt", "before%(name)after");
    let out_file = temp_path.join("define_out.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("-D")
        .arg("name=value")
        .arg("--output")
        .arg(&out_file)
        .arg(&input);

    let output = cmd.output()?;
    assert!(output.status.success(), "define should seed a top-level variable");

    let body = fs::read_to_string(&out_file)?;
    assert_eq!(body, "beforevalueafter");

    Ok(())
}

#[test]
fn test_env_prefix_cli() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let input = create_test_file(&temp_path, "env_prefix.txt", "%env(DEMO)");
    let out_file = temp_path.join("env_prefix_out.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.env("WB_DEMO", "scoped")
        .arg("--allow-env")
        .arg("--env-prefix")
        .arg("WB_")
        .arg("--output")
        .arg(&out_file)
        .arg(&input);

    let output = cmd.output()?;
    assert!(output.status.success(), "env-prefix should map to prefixed environment variables");

    let body = fs::read_to_string(&out_file)?;
    assert_eq!(body, "scoped");

    Ok(())
}

#[test]
fn test_recursion_limit_cli() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let input = create_test_file(
        &temp_path,
        "recursion_limit.txt",
        "%def(loop, %loop())\n%loop()",
    );
    let out_file = temp_path.join("recursion_limit_out.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--recursion-limit")
        .arg("4")
        .arg("--output")
        .arg(&out_file)
        .arg(&input);

    let output = cmd.output()?;
    assert!(
        !output.status.success(),
        "CLI run with a self-recursive macro should fail once the configured recursion limit is reached."
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("maximum recursion depth (4) exceeded"),
        "expected configured recursion limit in stderr, got: {stderr}"
    );

    Ok(())
}
