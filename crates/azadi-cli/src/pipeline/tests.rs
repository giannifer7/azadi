// src/pipeline/tests.rs

#[cfg(test)]
mod tests {
    use crate::pipeline::{run_pipeline, Args};
    use std::fs;
    use std::io;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    fn write_test_file(temp_dir: &TempDir, name: &str, content: &str) -> io::Result<()> {
        let path = temp_dir.path().join(name);
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, content)
    }

    #[test]
    fn test_basic_macro_pipeline() -> Result<(), Box<dyn std::error::Error>> {
        let temp = setup_test_dir();
        let test_file = "test.txt";
        write_test_file(&temp, test_file, "%def(x, hello)\n%x()")?;

        let args = Args {
            files: vec![test_file.into()],
            input_dir: temp.path().into(),
            output_dir: temp.path().join("out"),
            work_dir: temp.path().join("work"),
            ..Default::default()
        };

        run_pipeline(args).map_err(|e| {
            eprintln!("Error: {} ({:?})", e, e); // Print the error and its source
            e
        })?;

        let macro_output = fs::read_to_string(temp.path().join("work/macro_out/test.txt.txt"))?;
        assert!(macro_output.contains("hello"));
        Ok(())
    }

    #[test]
    fn test_full_pipeline() -> Result<(), Box<dyn std::error::Error>> {
        let temp = setup_test_dir();
        let test_file = "test.txt";
        let full_path = temp.path().join(test_file);
        fs::write(
            &full_path,
            "%def(code, <[test.rs]>=\nfn main() {}\n$$)\n%code()",
        )?;

        let args = Args {
            files: vec![test_file.into()],
            input_dir: temp.path().into(),
            output_dir: temp.path().join("out"),
            work_dir: temp.path().join("work"),
            ..Default::default()
        };

        // Run the pipeline
        run_pipeline(args).map_err(|e| {
            eprintln!("Error: {} ({:?})", e, e); // Print the error and its source
            e
        })?;

        // Verify the output
        let output_file = temp.path().join("out/test.rs");
        let noweb_output = fs::read_to_string(output_file)?;
        assert!(noweb_output.contains("fn main()"));
        Ok(())
    }

    #[test]
    fn test_dump_ast() -> Result<(), Box<dyn std::error::Error>> {
        let temp = setup_test_dir();
        let test_file = "test.txt";
        let full_path = temp.path().join(test_file);
        fs::write(&full_path, "%def(x, hello)\n%x()")?;

        let args = Args {
            files: vec![test_file.into()],
            input_dir: temp.path().into(),
            output_dir: temp.path().join("out"),
            work_dir: temp.path().join("work"),
            dump_ast: true,
            ..Default::default()
        };

        run_pipeline(args)?;
        let ast_output = fs::read_to_string(full_path.with_extension("ast"))?;
        assert!(!ast_output.is_empty());
        Ok(())
    }

    #[test]
    fn test_specific_chunks() -> Result<(), Box<dyn std::error::Error>> {
        let temp = setup_test_dir();
        let test_file = "test.txt";
        let full_path = temp.path().join(test_file);
        fs::write(
            &full_path,
            "<[chunk1]>=\nContent 1\n$$\n<[chunk2]>=\nContent 2\n$$\n",
        )?;

        let args = Args {
            files: vec![test_file.into()],
            input_dir: temp.path().into(),
            output_dir: temp.path().join("out"),
            work_dir: temp.path().join("work"),
            noweb_only: true,
            chunks: Some("chunk1".into()),
            ..Default::default()
        };

        // Run the pipeline
        run_pipeline(args).map_err(|e| {
            eprintln!("Error: {} ({:?})", e, e); // Print the error and its source
            e
        })?;

        // Verify the output
        let output_file = temp.path().join("out/chunk1");
        let content = fs::read_to_string(output_file)?;
        assert!(content.contains("Content 1"));
        Ok(())
    }
}
