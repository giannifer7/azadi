// crates/azadi-macros/tests/test_macro_cli.rs

use escargot;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Helper function to create a file with content
fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    let mut file = fs::File::create(&path).unwrap();
    write!(file, "{}", content).unwrap();
    path.canonicalize().unwrap()
}

// Helper to build and get command
fn cargo_azadi_macro_cli() -> Result<escargot::CargoRun, Box<dyn std::error::Error>> {
    Ok(escargot::CargoBuild::new()
        .bin("azadi-macro")
        .current_release()
        .current_target()
        .run()?)
}

#[test]
fn test_basic_macro_processing() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;
    println!("Canonicalized temp dir: {}", temp_path.display());

    // Create input file with simple content - explicit macro with argument
    let input = create_test_file(
        &temp_path,
        "input.txt",
        r#"%def(hello, World)
Hello %hello()!"#,
    );
    println!("Canonicalized input file: {}", input.display());
    println!("Input file content:");
    println!("{}", fs::read_to_string(&input)?);
    assert!(input.exists(), "Input file should exist");

    // Set up an output file instead of a directory
    let out_file = temp_path.join("output.txt");
    let work_dir = temp_path.join("work");

    println!("Output file will be: {}", out_file.display());
    println!("Work dir will be: {}", work_dir.display());

    // Run the command
    let run = cargo_azadi_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--out-file") // Changed from --out-dir to --out-file
        .arg(&out_file)
        .arg("--work-dir")
        .arg(&work_dir)
        .arg(&input);

    println!("Running command: {:?}", cmd);

    let output = cmd.output()?;
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());

    // Check the output file directly
    println!("Looking for output file: {}", out_file.display());
    assert!(out_file.exists(), "Output file should exist");

    let output_content = fs::read_to_string(&out_file)?;
    println!("Output content: {:?}", output_content); // Debug print to show exact content
    assert_eq!(output_content.trim(), "Hello World!");

    Ok(())
}

// 1) Test the help message
#[test]
fn test_cli_help() -> Result<(), Box<dyn std::error::Error>> {
    let run = cargo_azadi_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--help");

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "Expected 'azadi-macro --help' to succeed."
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("(help) stdout:\n{stdout}");
    println!("(help) stderr:\n{stderr}");

    // Check that it has some known usage text:
    assert!(
        stdout.contains("azadi-macro"),
        "Help output did not mention 'azadi-macro'"
    );
    assert!(
        stdout.contains("--out-file"), // Changed from --out-dir to --out-file
        "Help output did not mention '--out-file'"
    );

    Ok(())
}

// 2) Test passing a non-existent input file
#[test]
fn test_missing_input_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    // We intentionally do NOT create the file
    let missing_input = temp_path.join("not_real.txt");

    let out_file = temp_path.join("output.txt");
    let work_dir = temp_path.join("work");

    let run = cargo_azadi_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--out-file") // Changed from --out-dir to --out-file
        .arg(&out_file)
        .arg("--work-dir")
        .arg(&work_dir)
        .arg(missing_input.to_string_lossy().to_string());

    let output = cmd.output()?;
    // We expect a failure
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

    // Create two input files
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
    let work_dir = temp_path.join("work");

    let run = cargo_azadi_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--out-file") // Changed from --out-dir to --out-file
        .arg(&out_file)
        .arg("--work-dir")
        .arg(&work_dir)
        .arg(input1.to_string_lossy().to_string())
        .arg(input2.to_string_lossy().to_string());

    let output = cmd.output()?;
    println!("(multiple_inputs) status: {:?}", output.status);
    println!(
        "(multiple_inputs) stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "(multiple_inputs) stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success());

    // Check the single output file with combined content
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

// 4) Test a custom special char
#[test]
fn test_custom_special_char() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    // Use '@' as the special character
    let input = create_test_file(
        &temp_path,
        "input_at.txt",
        "@def(test_macro, Hello from custom char)\n@test_macro()",
    );
    let out_file = temp_path.join("output_at.txt");
    let work_dir = temp_path.join("work");

    let run = cargo_azadi_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--special")
        .arg("@")
        .arg("--out-file") // Changed from --out-dir to --out-file
        .arg(&out_file)
        .arg("--work-dir")
        .arg(&work_dir)
        .arg(&input);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI run with custom special char should succeed."
    );

    let content = fs::read_to_string(&out_file)?;
    println!("(custom_special_char) output content:\n{content}");
    assert!(
        content.contains("Hello from custom char"),
        "Expected to see expansion with '@' as the macro char."
    );

    Ok(())
}

// 5) Test the --pydef flag
#[test]
fn test_pydef_flag() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    // If your code treats pydef macros distinctly, you can define one
    // Here we just define "pydef" as if it were a normal macro for demonstration
    let input = create_test_file(
        &temp_path,
        "pydef_test.txt",
        "%pydef(test_python, %{print('Hello Python')\n%})\n%test_python()",
    );

    let out_file = temp_path.join("output_py.txt");
    let work_dir = temp_path.join("work");

    let run = cargo_azadi_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--pydef")
        .arg("--out-file") // Changed from --out-dir to --out-file
        .arg(&out_file)
        .arg("--work-dir")
        .arg(&work_dir)
        .arg(&input);

    let output = cmd.output()?;
    println!(
        "(pydef_flag) stdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "(pydef_flag) stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "CLI should succeed with the pydef argument set to true."
    );

    // Check output
    let content = fs::read_to_string(&out_file)?;
    println!("(pydef_flag) final content:\n{content}");
    assert!(
        content.contains("Hello Python"),
        "Expected expansion from a pydef macro."
    );

    Ok(())
}

// 6) Test using a colon-separated include path
#[test]
fn test_colon_separated_includes() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    // Make a subdirectory 'includes' with a file 'my_include.txt'
    let includes_dir = temp_path.join("includes");
    fs::create_dir_all(&includes_dir)?;
    let _inc_file = create_test_file(&includes_dir, "my_include.txt", "From includes dir");

    // Now create a main file that tries to include 'my_include.txt'
    let main_file = create_test_file(&temp_path, "main.txt", "%include(my_include.txt)");

    let out_file = temp_path.join("output_inc.txt");
    let work_dir = temp_path.join("work");

    let run = cargo_azadi_macro_cli()?;
    let mut cmd = run.command();

    // We'll pass two include paths: . and ./includes
    // On Unix, pathsep is ":" by default. If you want
    // to force a different pathsep, you can do so too.
    let includes_str = format!(".:{}", includes_dir.to_string_lossy().to_string());

    cmd.arg("--include")
        .arg(&includes_str)
        .arg("--out-file") // Changed from --out-dir to --out-file
        .arg(&out_file)
        .arg("--work-dir")
        .arg(&work_dir)
        .arg(&main_file);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI should succeed with colon-separated includes."
    );

    let content = fs::read_to_string(&out_file)?;
    println!("(colon_separated_includes) content:\n{content}");
    assert!(
        content.contains("From includes dir"),
        "Expected the included content from includes/my_include.txt."
    );

    Ok(())
}

// 7) Test forcing a custom --pathsep
#[test]
fn test_custom_pathsep_includes() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        // On Windows, the default pathsep is `;`.
        // This test might be more relevant on a Unix machine,
        // but let's include an example for completeness.
    }

    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    let includes_dir = temp_path.join("my_includes");
    fs::create_dir_all(&includes_dir)?;
    create_test_file(&includes_dir, "m_incl.txt", "Inside custom pathsep dir");

    let main_file = create_test_file(&temp_path, "custom_sep_main.txt", "%include(m_incl.txt)");

    let out_file = temp_path.join("output_sep.txt");
    let work_dir = temp_path.join("work");

    // Suppose we want to use '|' as the path separator
    let includes_str = format!(".|{}", includes_dir.display());

    let run = cargo_azadi_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--include")
        .arg(&includes_str)
        .arg("--pathsep")
        .arg("|")
        .arg("--out-file") // Changed from --out-dir to --out-file
        .arg(&out_file)
        .arg("--work-dir")
        .arg(&work_dir)
        .arg(&main_file);

    let output = cmd.output()?;
    println!(
        "(custom_pathsep) stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "(custom_pathsep) stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.status.success());

    let content = fs::read_to_string(&out_file)?;
    assert!(
        content.contains("Inside custom pathsep dir"),
        "Expected custom pathsep to locate includes dir."
    );

    Ok(())
}

// 8) Test that the CLI can handle a large input file (smoke test)
#[test]
fn test_large_input() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let temp_path = temp.path().canonicalize()?;

    // Let's create a large file with repeated macros
    let mut big_content = String::new();
    big_content.push_str("%def(say, HELLO)\n");
    for _ in 0..10_000 {
        big_content.push_str("%say()");
        big_content.push('\n');
    }

    let big_file = create_test_file(&temp_path, "big_file.txt", &big_content);

    let out_file = temp_path.join("output_big.txt");
    let work_dir = temp_path.join("work");

    let run = cargo_azadi_macro_cli()?;
    let mut cmd = run.command();
    cmd.arg("--out-file") // Changed from --out-dir to --out-file
        .arg(&out_file)
        .arg("--work-dir")
        .arg(&work_dir)
        .arg(&big_file);

    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "CLI should handle a large input file."
    );

    let out_content = fs::read_to_string(&out_file)?;
    // We expect 10,000 lines of "HELLO"
    let line_count = out_content.matches("HELLO").count();
    assert_eq!(
        line_count, 10_000,
        "Expected 10,000 expansions of HELLO in the large output."
    );

    Ok(())
}
