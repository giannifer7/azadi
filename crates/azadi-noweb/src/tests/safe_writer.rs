// src/tests/safe_writer.rs
use super::*;
use crate::SafeWriterError;
use crate::AzadiError;
use std::{fs, io::Write, path::PathBuf, thread, time::Duration};

#[test]
fn test_basic_file_writing() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();
    let test_file = PathBuf::from("test.txt");
    let test_content = "Hello, World!";

    write_file(&mut writer, &test_file, test_content)?;

    let final_path = writer.get_gen_base().join(&test_file);
    let content = fs::read_to_string(&final_path)?;
    assert_eq!(content, test_content);
    Ok(())
}

#[test]
fn test_multiple_file_generation() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();
    let test_file1 = PathBuf::from("file1.txt");
    let test_file2 = PathBuf::from("file2.txt");

    write_file(&mut writer, &test_file1, "Content 1")?;
    write_file(&mut writer, &test_file2, "Content 2")?;

    let content1 = fs::read_to_string(writer.get_gen_base().join(&test_file1))?;
    let content2 = fs::read_to_string(writer.get_gen_base().join(&test_file2))?;

    assert_eq!(content1.trim(), "Content 1");
    assert_eq!(content2.trim(), "Content 2");
    Ok(())
}

#[test]
fn test_unmodified_file_update() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();

    let mut config = writer.get_config().clone();
    config.modification_check = false;
    writer.set_config(config);

    let test_file = PathBuf::from("test.txt");

    write_file(&mut writer, &test_file, "Initial content")?;
    write_file(&mut writer, &test_file, "New content")?;

    let content = fs::read_to_string(writer.get_gen_base().join(&test_file))?;
    assert_eq!(content, "New content", "New content should be written");
    Ok(())
}

#[test]
fn test_backup_creation() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();
    let test_file = PathBuf::from("test.txt");
    let content = "Test content";

    write_file(&mut writer, &test_file, content)?;

    let backup_path = writer.get_old_dir().join(&test_file);
    assert!(backup_path.exists(), "Backup file should exist");

    let backup_content = fs::read_to_string(backup_path)?;
    assert_eq!(backup_content, content, "Backup content should match original");
    Ok(())
}

#[test]
fn test_nested_directory_creation() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();
    let nested_path = PathBuf::from("dir1/dir2/test.txt");

    write_file(&mut writer, &nested_path, "Nested content")?;

    let gen_dir = writer.get_gen_base().join("dir1").join("dir2");
    let old_dir = writer.get_old_dir().join("dir1").join("dir2");
    let private_dir = writer.get_private_dir().join("dir1").join("dir2");

    assert!(gen_dir.exists(), "Generated directory structure should exist");
    assert!(old_dir.exists(), "Backup directory structure should exist");
    assert!(private_dir.exists(), "Private directory structure should exist");
    Ok(())
}

#[test]
fn test_modification_detection() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();
    let test_file = PathBuf::from("test.txt");
    let modified_content = "Modified content";

    write_file(&mut writer, &test_file, "Initial content")?;

    thread::sleep(Duration::from_millis(10));
    let final_path = writer.get_gen_base().join(&test_file);
    {
        let mut file = fs::File::create(&final_path)?;
        write!(file, "{}", modified_content)?;
    }

    let result = write_file(&mut writer, &test_file, "New content");
    match result {
        Err(AzadiError::SafeWriter(SafeWriterError::ModifiedExternally(_))) => {
            let content = fs::read_to_string(&final_path)?;
            assert_eq!(content, modified_content, "Modified content should be preserved");
            Ok(())
        }
        Ok(_) => panic!("Expected ModifiedExternally error"),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_concurrent_modifications() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();
    let test_file = PathBuf::from("test.txt");
    let modified_content_2 = "Modified 2";

    write_file(&mut writer, &test_file, "Initial")?;

    let final_path = writer.get_gen_base().join(&test_file);
    thread::sleep(Duration::from_millis(10));
    {
        let mut file = fs::File::create(&final_path)?;
        write!(file, "Modified 1")?;
    }
    thread::sleep(Duration::from_millis(10));
    {
        let mut file = fs::File::create(&final_path)?;
        write!(file, "{}", modified_content_2)?;
    }

    let result = write_file(&mut writer, &test_file, "New content");
    match result {
        Err(AzadiError::SafeWriter(SafeWriterError::ModifiedExternally(_))) => {
            let content = fs::read_to_string(&final_path)?;
            assert_eq!(content, modified_content_2, "Latest modification should be preserved");
            Ok(())
        }
        Ok(_) => panic!("Expected ModifiedExternally error"),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_backup_disabled() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();
    let test_file = PathBuf::from("test.txt");

    let mut config = writer.get_config().clone();
    config.backup_enabled = false;
    writer.set_config(config);

    write_file(&mut writer, &test_file, "Test content")?;

    let backup_path = writer.get_old_dir().join(&test_file);
    assert!(!backup_path.exists(), "Backup file should not exist when backups are disabled");
    Ok(())
}

#[test]
fn test_validate_filename_relative_path() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();
    let test_file = PathBuf::from("simple.txt");
    write_file(&mut writer, &test_file, "Allowed")?;
    let final_path = writer.get_gen_base().join(&test_file);
    let content = fs::read_to_string(&final_path)?;
    assert_eq!(content, "Allowed");
    Ok(())
}

#[test]
fn test_path_safety() {
    let (_temp, mut writer) = create_test_writer();
    
    let test_cases = [
        (PathBuf::from("../outside.txt"), "Path traversal detected (..)"),
        (PathBuf::from("/absolute/path.txt"), "Absolute paths are not allowed"),
        (PathBuf::from("C:/windows/path.txt"), "Windows-style absolute paths are not allowed"),
        (PathBuf::from("C:test.txt"), "Windows-style absolute paths are not allowed"),
    ];

    for (path, expected_msg) in test_cases {
        let result = write_file(&mut writer, &path, "Should fail");
        match result {
            Err(AzadiError::SafeWriter(SafeWriterError::SecurityViolation(msg))) => {
                assert!(msg.contains(expected_msg), 
                    "Expected message '{}' for path {}", expected_msg, path.display());
            }
            _ => panic!("Expected SecurityViolation for path: {}", path.display()),
        }
    }
}
