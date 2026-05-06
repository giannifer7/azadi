// weaveback-macro/tests/test_macro_cli/includes.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::support::{cargo_weaveback_macro_cli, create_test_file};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_colon_separated_includes() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let includes_dir = temp_path.join("includes");
    fs::create_dir_all(&includes_dir)?;
    let _inc_file = create_test_file(&includes_dir, "my_include.txt", "From includes dir");

    let main_file = create_test_file(&temp_path, "main.txt", "%include(my_include.txt)");

    let out_file = temp_path.join("output_inc.txt");

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    let includes_str = format!(".:{}", includes_dir.to_string_lossy());

    cmd.arg("--include")
        .arg(&includes_str)
        .arg("--output")
        .arg(&out_file)
        .arg(&main_file);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI should succeed with colon-separated includes."
    );

    let content = fs::read_to_string(&out_file)?;
    assert!(
        content.contains("From includes dir"),
        "Expected the included content from includes/my_include.txt."
    );

    Ok(())
}

// 7) Test forcing a custom --pathsep
#[test]
fn test_custom_pathsep_includes() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let includes_dir = temp_path.join("my_includes");
    fs::create_dir_all(&includes_dir)?;
    create_test_file(&includes_dir, "m_incl.txt", "Inside custom pathsep dir");

    let main_file = create_test_file(&temp_path, "custom_sep_main.txt", "%include(m_incl.txt)");

    let out_file = temp_path.join("output_sep.txt");
    let includes_str = format!(".|{}", includes_dir.display());

    let run = cargo_weaveback_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--include")
        .arg(&includes_str)
        .arg("--pathsep")
        .arg("|")
        .arg("--output")
        .arg(&out_file)
        .arg(&main_file);

    let output = cmd.output()?;
    assert!(output.status.success());

    let content = fs::read_to_string(&out_file)?;
    assert!(
        content.contains("Inside custom pathsep dir"),
        "Expected custom pathsep to locate includes dir."
    );

    Ok(())
}
