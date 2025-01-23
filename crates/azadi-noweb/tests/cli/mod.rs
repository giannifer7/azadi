// tests/main_tests.rs
use assert_cmd::assert::OutputAssertExt;
use assert_cmd::prelude::CommandCargoExt;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_no_arguments_fails() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("azadi-noweb")?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
    Ok(())
}

#[test]
fn test_basic_chunk_extraction() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let input_file = dir.path().join("input.nw");
    {
        let mut file = fs::File::create(&input_file)?;
        writeln!(file, "<<@file test.txt>>=")?;
        writeln!(file, "Hello, world!")?;
        writeln!(file, "@")?;
    }

    // run the command => writes test.txt to gen by default
    let mut cmd = Command::cargo_bin("azadi-noweb")?;
    cmd.arg("--gen").arg(dir.path().join("gen")).arg(input_file);
    cmd.assert().success();

    let output_path = dir.path().join("gen/test.txt");
    let output_content = fs::read_to_string(output_path)?;
    assert_eq!(output_content, "Hello, world!\n");
    Ok(())
}

#[test]
fn test_extract_specific_chunk_to_stdout() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let input_file = dir.path().join("input.nw");
    {
        let mut f = fs::File::create(&input_file)?;
        writeln!(f, "<<chunk1>>=")?;
        writeln!(f, "Chunk 1 content")?;
        writeln!(f, "@")?;
        writeln!(f, "<<chunk2>>=")?;
        writeln!(f, "Chunk 2 content")?;
        writeln!(f, "@")?;
    }

    let mut cmd = Command::cargo_bin("azadi-noweb")?;
    cmd.arg("--gen")
        .arg(dir.path().join("gen"))
        .arg("--chunks")
        .arg("chunk2")
        .arg(input_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Chunk 2 content"));
    Ok(())
}
