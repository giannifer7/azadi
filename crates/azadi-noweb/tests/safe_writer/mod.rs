// crates/azadi-noweb/tests/safe_writer.rs
use super::common::{create_test_writer, write_file};
use azadi_noweb::AzadiError;
use std::{fs, io::Write, path::PathBuf, thread, time::Duration};

#[test]
fn test_modification_detection() -> Result<(), AzadiError> {
    let (_temp, mut writer) = create_test_writer();

    let test_file = PathBuf::from("test.txt");
    write_file(&mut writer, &test_file, "Initial content")?;

    // externally modify
    thread::sleep(Duration::from_millis(10));
    let finalp = writer.get_gen_base().join(&test_file);
    {
        let mut f = fs::File::create(&finalp)?;
        write!(f, "Externally modified")?;
    }

    // rewriting => expect ModifiedExternally
    let result = write_file(&mut writer, &test_file, "New content");
    match result {
        Err(AzadiError::ModifiedExternally(msg)) => {
            let read_back = fs::read_to_string(&finalp)?;
            assert_eq!(read_back, "Externally modified");
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
    write_file(&mut writer, &test_file, "Test content")?;

    // Verify file exists in final location
    let final_path = writer.get_gen_base().join(&test_file);
    assert!(final_path.exists(), "File should exist in final location");

    let content = fs::read_to_string(final_path)?;
    assert_eq!(content, "Test content");

    Ok(())
}
