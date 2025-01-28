// <[@file tests/safe_writer/mod.rs]>=
// crates/azadi-noweb/tests/safe_writer.rs
use super::common::create_test_writer;
use azadi_noweb::AzadiError;
use std::{fs, io::Write, path::PathBuf, thread, time::Duration};

#[test]
fn test_modification_detection() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();

    let test_file = PathBuf::from("test.txt");
    let private_path = writer.before_write(&test_file)?;
    fs::write(&private_path, "Initial content")?;
    writer.after_write(&test_file)?;

    // Introduce a delay (still a good practice)
    thread::sleep(Duration::from_millis(1500));

    // Externally modify by appending
    let finalp = writer.get_gen_base().join(&test_file);
    {
        let mut f = fs::OpenOptions::new().append(true).open(&finalp)?;
        writeln!(f, "External modification")?;
    }

    // Get the content of the externally modified file
    let modified_content = fs::read_to_string(&finalp)?;

    // Attempt to rewrite the file
    let private_path = writer.before_write(&test_file)?;
    fs::write(&private_path, "New content")?;
    let result = writer.after_write(&test_file);

    // Check for ModifiedExternally error
    match result {
        Err(AzadiError::ModifiedExternally(msg)) => {
            // Verify the content of the final file (should be the modified content)
            let read_back = fs::read_to_string(&finalp)?;
            assert_eq!(read_back, modified_content);
            assert!(msg.contains("test.txt"), "Should mention the file");
            Ok(())
        }
        Ok(_) => panic!("Should fail with ModifiedExternally"),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn test_nested_directory_creation() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();

    // Try to write to a deeply nested path
    let test_file = PathBuf::from("deep/nested/path/test.txt");
    let private_path = writer.before_write(&test_file)?;
    fs::write(&private_path, "Test content")?;
    writer.after_write(&test_file)?;

    // Verify file exists in final location
    let final_path = writer.get_gen_base().join(&test_file);
    assert!(final_path.exists(), "File should exist in final location");

    let content = fs::read_to_string(final_path)?;
    assert_eq!(content, "Test content");

    Ok(())
}

#[test]
fn test_backup() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();

    let test_file = PathBuf::from("test_backup.txt");
    let private_path = writer.before_write(&test_file)?;
    fs::write(&private_path, "Initial content")?;
    writer.after_write(&test_file)?;

    // Simulate an external modification
    let final_path = writer.get_gen_base().join(&test_file);
    fs::write(&final_path, "Modified content")?;

    // Write again
    let private_path = writer.before_write(&test_file)?;
    fs::write(&private_path, "New content")?;
    writer.after_write(&test_file)?;

    // Verify that the backup exists
    let backup_path = writer.get_work_dir().join("__old__").join(&test_file);
    assert!(backup_path.exists());

    // Verify the content of the backup
    let backup_content = fs::read_to_string(backup_path)?;
    assert_eq!(backup_content, "Modified content");

    Ok(())
}

#[test]
fn test_no_backup() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();

    // Disable backup
    let mut config = writer.get_config().clone();
    config.backup_enabled = false;
    writer.set_config(config);

    let test_file = PathBuf::from("test_no_backup.txt");
    let private_path = writer.before_write(&test_file)?;
    fs::write(&private_path, "Initial content")?;
    writer.after_write(&test_file)?;

    // Simulate an external modification
    let final_path = writer.get_gen_base().join(&test_file);
    fs::write(&final_path, "Modified content")?;

    // Write again
    let private_path = writer.before_write(&test_file)?;
    fs::write(&private_path, "New content")?;
    writer.after_write(&test_file)?;

    // Verify that no backup was created
    let backup_path = writer.get_work_dir().join("__old__").join(&test_file);
    assert!(!backup_path.exists());

    Ok(())
}

#[test]
fn test_allow_overwrite() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();

    // Allow overwrites
    let mut config = writer.get_config().clone();
    config.allow_overwrites = true;
    writer.set_config(config);

    let test_file = PathBuf::from("test_overwrite.txt");
    let private_path = writer.before_write(&test_file)?;
    fs::write(&private_path, "Initial content")?;
    writer.after_write(&test_file)?;

    // Introduce a delay to ensure a timestamp difference
    thread::sleep(Duration::from_millis(100));

    // Simulate an external modification
    let final_path = writer.get_gen_base().join(&test_file);
    fs::write(&final_path, "Modified content")?;

    // Write again - should not fail
    let private_path = writer.before_write(&test_file)?;
    fs::write(&private_path, "New content")?;
    writer.after_write(&test_file)?;

    // Verify content
    let content = fs::read_to_string(final_path)?;
    assert_eq!(content, "New content");

    Ok(())
}
// $$
