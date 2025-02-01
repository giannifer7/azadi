#[cfg(test)]
mod tests {
    use azadi_cli::{run_pipeline, Args, PipelineError};
    use azadi_macros::evaluator::EvalError;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Setup helper to create a test environment
    fn setup_test_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    /// Helper to write a file with content
    fn write_test_file(dir: &TempDir, name: &str, content: &str) -> std::io::Result<PathBuf> {
        let path = dir.path().join(name);
        fs::write(&path, content)?;
        Ok(path)
    }

    #[test]
    fn test_normal_success() -> Result<(), Box<dyn std::error::Error>> {
        let temp = setup_test_dir();

        // Create input file with both macro and noweb chunks
        let test_file = write_test_file(
            &temp,
            "test.txt",
            r#"%def(x, hello)
<[@file test.txt]>=
%x()
$$"#,
        )?;

        let args = Args {
            files: vec![test_file],
            input_dir: temp.path().to_path_buf(),
            output_dir: temp.path().join("out"),
            work_dir: temp.path().join("work"),
            ..Default::default()
        };

        run_pipeline(args)?;

        let output_file = temp.path().join("out/test.txt");
        let content = fs::read_to_string(output_file)?;
        assert!(content.contains("hello"), "Expected expanded macro content");

        Ok(())
    }

    #[test]
    fn test_file_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let temp = setup_test_dir();
        let nonexistent = temp.path().join("does-not-exist.txt");

        let args = Args {
            files: vec![nonexistent.clone()],
            input_dir: temp.path().to_path_buf(),
            output_dir: temp.path().join("out"),
            work_dir: temp.path().join("work"),
            ..Default::default()
        };

        match run_pipeline(args) {
            Err(PipelineError::InputNotFound { path, .. }) => {
                assert_eq!(path, nonexistent);
                Ok(())
            }
            other => panic!("Expected InputNotFound error, got: {:?}", other),
        }
    }

    #[test]
    fn test_macro_processing_error() -> Result<(), Box<dyn std::error::Error>> {
        let temp = setup_test_dir();

        let test_file = write_test_file(
            &temp,
            "test.txt",
            "<[@file test.txt]>=\n%undefined_macro(this will fail)\n$$",
        )?;

        let args = Args {
            files: vec![test_file],
            input_dir: temp.path().to_path_buf(),
            output_dir: temp.path().join("out"),
            work_dir: temp.path().join("work"),
            ..Default::default()
        };

        match run_pipeline(args) {
            Err(PipelineError::MacroError { .. }) => Ok(()),
            other => panic!("Expected MacroError, got: {:?}", other),
        }
    }

    #[test]
    fn test_here_macro_termination() -> Result<(), Box<dyn std::error::Error>> {
        let temp = setup_test_dir();

        let test_file = write_test_file(
            &temp,
            "test.txt",
            r#"%def(content, replaced text)
<[@file test.txt]>=
%here(content)
$$"#,
        )?;

        let args = Args {
            files: vec![test_file],
            input_dir: temp.path().to_path_buf(),
            output_dir: temp.path().join("out"),
            work_dir: temp.path().join("work"),
            ..Default::default()
        };

        match run_pipeline(args) {
            Err(PipelineError::MacroError {
                source: EvalError::Terminate(_),
                ..
            }) => Ok(()),
            other => panic!("Expected Terminate error, got: {:?}", other),
        }
    }
}
