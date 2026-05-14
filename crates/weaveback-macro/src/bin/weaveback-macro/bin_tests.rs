// weaveback-macro/src/bin/weaveback-macro/bin_tests.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let unique = format!(
            "wb-macro-bin-tests-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn write(&self, name: &str, content: &str) -> PathBuf {
        let p = self.root.join(name);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, content).unwrap();
        p
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn default_args() -> Args {
    Args {
        output: PathBuf::from("-"),
        sigil: '%',
        include: ".".to_string(),
        pathsep: default_pathsep(),
        input_dir: PathBuf::from("."),
        allow_env: false,
        env_prefix: None,
        recursion_limit: 1000,
        define: vec![],
        inputs: vec![],
        directory: None,
        ext: vec!["md".to_string()],
        dump_ast: false,
    }
}

#[test]
fn test_bin_run_basic() {
    let ws = TestWorkspace::new();
    let input = ws.write("test.md", "hello %def(x,y)%x() world");
    let output = ws.root.join("out.txt");

    let mut args = default_args();
    args.inputs = vec![input];
    args.output = output.clone();

    run(args).unwrap();

    let body = std::fs::read_to_string(output).unwrap();
    assert_eq!(body.trim(), "hello y world");
}

#[test]
fn test_bin_run_dir_scan() {
    let ws = TestWorkspace::new();
    // Create a driver and a fragment
    ws.write("driver.md", "include %include(frag.md)");
    ws.write("frag.md", "fragment content");

    let output = ws.root.join("out.txt");
    let mut args = default_args();
    args.directory = Some(ws.root.clone());
    args.include = ws.root.to_string_lossy().to_string(); // Ensure includes are found
    args.output = output.clone();

    run(args).unwrap();

    let body = std::fs::read_to_string(output).unwrap();
    assert!(body.contains("fragment content"));
}

#[test]
fn test_bin_run_not_found() {
    let mut args = default_args();
    args.inputs = vec![PathBuf::from("nonexistent.md")];
    let res = run(args);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("does not exist"));
}

#[test]
fn test_bin_run_dump_ast() {
    let ws = TestWorkspace::new();
    let input = ws.write("test.md", "hello world");

    let mut args = default_args();
    args.inputs = vec![input.clone()];
    args.dump_ast = true;

    run(args).unwrap();

    let ast_file = input.with_extension("ast");
    assert!(ast_file.exists());
    let _ = std::fs::remove_file(ast_file);
}
