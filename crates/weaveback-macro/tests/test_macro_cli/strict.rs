// weaveback-macro/tests/test_macro_cli/strict.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::support::{cargo_weaveback_macro_cli, create_test_file};
use tempfile::TempDir;

#[test]
fn test_undefined_variable_is_strict_by_default_cli() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let input = create_test_file(&temp_path, "strict_vars.txt", "before%(missing)after");
    let out_file = temp_path.join("strict_vars_out.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--output")
        .arg(&out_file)
        .arg(&input);

    let output = cmd.output()?;
    assert!(!output.status.success(), "undefined variable should fail by default");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Undefined variable: missing"),
        "expected undefined-variable error, got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_unbound_params_are_strict_by_default_cli() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let input = create_test_file(
        &temp_path,
        "strict_params.txt",
        "%def(greet, name, msg, Hello %(name)%(msg)!)\n%greet(Alice)\n",
    );
    let out_file = temp_path.join("strict_params_out.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--output")
        .arg(&out_file)
        .arg(&input);

    let output = cmd.output()?;
    assert!(!output.status.success(), "missing args should fail by default");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unbound parameter 'msg' in macro 'greet'"),
        "expected unbound-parameter error, got: {stderr}"
    );

    Ok(())
}
