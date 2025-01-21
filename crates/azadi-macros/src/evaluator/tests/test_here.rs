use crate::evaluator::evaluator::EvalConfig;
use crate::evaluator::Evaluator;
use crate::macro_api::process_string;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper function to create an Evaluator with a temporary directory as the include path
fn create_evaluator_with_temp_dir(temp_dir: &Path) -> Evaluator {
    let config = EvalConfig {
        include_paths: vec![temp_dir.to_path_buf()],
        ..Default::default()
    };
    Evaluator::new(config)
}

#[test]
fn test_here_with_macros() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.path();

    // Create a test file `test.txt` in the temporary directory
    let test_file_path = temp_dir_path.join("test.txt");
    fs::write(
        &test_file_path,
        r#"
        %def(insert_content, greeting, %{
            Inserted content, %(greeting)!
        %})
        Before %here(insert_content, Hello)
        After
        "#,
    )
    .expect("Failed to write test file");

    // Create an Evaluator with the temporary directory as the include path
    let mut evaluator = create_evaluator_with_temp_dir(temp_dir_path);

    // Process the file with the %here macro
    let result = process_string(
        &fs::read_to_string(&test_file_path).unwrap(),
        Some(&test_file_path),
        &mut evaluator,
    );

    // Verify that the file was modified correctly
    let modified_content = fs::read_to_string(&test_file_path).unwrap();
    assert_eq!(
        modified_content.trim(),
        "%def(insert_content, greeting, %{\n            Inserted content, %(greeting)!\n        %})\n        Before %%here(insert_content, Hello)\n            Inserted content, Hello!\n                After"
    );

    // Verify that the result indicates termination
    assert!(result.is_err()); // %here terminates execution
}
