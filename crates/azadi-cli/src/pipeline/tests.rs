#[cfg(test)]
mod tests {
    use crate::pipeline::{run_pipeline, Args};
    use std::fs;
    use std::io;
    use std::str;
    use tempfile::TempDir;

    /// Create a new temporary directory for each test
    fn setup_test_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    /// Write a file into the temp directory, **without** creating subdirectories—
    /// so the program must do the actual subdir creation if needed.
    fn write_test_file(temp_dir: &TempDir, name: &str, content: &str) -> io::Result<()> {
        let path = temp_dir.path().join(name);
        // We do NOT call `create_dir_all(path.parent())`; the pipeline is responsible
        fs::write(path, content)
    }

    /// Basic macro-only test
    #[test]
    fn test_basic_macro_pipeline() -> Result<(), Box<dyn std::error::Error>> {
        let temp = setup_test_dir();

        let test_file = temp.path().join("test.txt");
        write_test_file(&temp, "test.txt", "%def(x, hello)\n%x()")?;

        let args = Args {
            files: vec![test_file.clone()],
            input_dir: temp.path().into(),
            output_dir: temp.path().join("out"),
            work_dir: temp.path().join("work"),
            macro_only: true,
            ..Default::default()
        };

        run_pipeline(args)?;

        // Expect macro output in work/macro_out/test.txt.txt
        let out_name = "test.txt.txt";
        let macro_output_path = temp.path().join("work/macro_out").join(out_name);
        let macro_output = fs::read_to_string(&macro_output_path)?;
        assert!(
            macro_output.contains("hello"),
            "Expected expanded macro to contain 'hello'; got:\n{macro_output}"
        );

        Ok(())
    }

    /// Macros + noweb: define a macro that writes `test.rs`
    #[test]
    fn test_full_pipeline() -> Result<(), Box<dyn std::error::Error>> {
        let temp = setup_test_dir();

        let test_file = temp.path().join("test.txt");
        write_test_file(
            &temp,
            "test.txt",
            "%def(code, %{<[@file test.rs]>=\nfn main() {}\n$$%})\n%code()",
        )?;

        let args = Args {
            files: vec![test_file.clone()],
            input_dir: temp.path().into(),
            output_dir: temp.path().join("out"),
            work_dir: temp.path().join("work"),
            ..Default::default()
        };

        run_pipeline(args)?;

        let output_file = temp.path().join("out/test.rs");
        let noweb_output = fs::read_to_string(&output_file)?;
        assert!(
            noweb_output.contains("fn main()"),
            "Expected 'fn main()' in final noweb output"
        );

        Ok(())
    }

    /// Dump AST test
    #[test]
    fn test_dump_ast() -> Result<(), Box<dyn std::error::Error>> {
        let temp = setup_test_dir();

        let test_file = temp.path().join("test.txt");
        write_test_file(&temp, "test.txt", "%def(x, hello)\n%x()")?;

        let args = Args {
            files: vec![test_file.clone()],
            input_dir: temp.path().into(),
            output_dir: temp.path().join("out"),
            work_dir: temp.path().join("work"),
            dump_ast: true,
            ..Default::default()
        };

        run_pipeline(args)?;

        // The pipeline writes `<input>.ast`
        let ast_file = test_file.with_extension("ast");
        let ast_output = fs::read_to_string(ast_file)?;
        assert!(
            !ast_output.is_empty(),
            "AST output file is empty, expected some JSON lines"
        );

        Ok(())
    }

    #[test]
    fn test_specific_chunks() -> Result<(), Box<dyn std::error::Error>> {
        use escargot::CargoBuild;
        let temp = setup_test_dir();

        let test_file = temp.path().join("test.txt");
        write_test_file(
            &temp,
            "test.txt",
            "<[chunk1]>=\nContent 1\n$$\n<[chunk2]>=\nContent 2\n$$\n",
        )?;

        // First build the binary
        let binary = CargoBuild::new()
            .bin("azadi")
            .current_release()
            .current_target()
            .run()?;

        // Now run it
        let output = binary
            .command()
            .arg("--noweb-only")
            .arg("--chunks")
            .arg("chunk1")
            .arg("--input-dir")
            .arg(temp.path())
            .arg("--output-dir")
            .arg(temp.path().join("out"))
            .arg("--work-dir")
            .arg(temp.path().join("work"))
            .arg(&test_file)
            .output()?;

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout)?;
        assert!(stdout.contains("Content 1"));

        Ok(())
    }
}
