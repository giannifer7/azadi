// weaveback-api/src/coverage/tests_coverage/cargo_extra/run.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;
use crate::coverage::tests_coverage::CARGO_TEST_MUTEX;

#[test]
fn test_run_cargo_annotated_to_writer_mega_mock() {
    let _guard = CARGO_TEST_MUTEX.lock().unwrap();
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("test.db");
    let gen_dir = tmp.path().join("gen");
    std::fs::create_dir(&gen_dir).unwrap();

    let mut out = Vec::new();

    // Mock shell script that outputs:
    // 1. A compiler-message (JSON)
    // 2. A plain text line (stderr-like)
    // 3. A build-finished message (JSON)
    let mock_script = r#"
        echo '{"reason":"compiler-message","message":{"spans":[{"file_name":"src/main.rs","line_start":1,"column_start":1,"is_primary":true}]}}'
        echo "plain text stderr line that looks like a location out.rs:1:1" >&2
        echo '{"reason":"build-finished","success":true}'
    "#;

    unsafe { std::env::set_var("WEAVEBACK_CARGO_BIN", "sh"); }
    let res = run_cargo_annotated_to_writer(
        vec!["-c".to_string(), mock_script.to_string()],
        false,
        db_path,
        gen_dir,
        EvalConfig::default(),
        tmp.path(),
        &mut out,
    );
    unsafe { std::env::remove_var("WEAVEBACK_CARGO_BIN"); }

    assert!(res.is_ok());
    let output = String::from_utf8(out).unwrap();
    assert!(output.contains("compiler-message"));
    assert!(output.contains("build-finished"));
    // The stderr line ("plain text stderr line") is currently piped to out in the loop too.
}

#[test]
fn test_run_cargo_annotated_to_writer_diagnostics_only() {
    let _guard = CARGO_TEST_MUTEX.lock().unwrap();
    let tmp = tempdir().unwrap();
    let mut out = Vec::new();
    let mock_script = "echo '{\"reason\":\"compiler-message\",\"message\":{\"spans\":[]}}'; echo '{\"reason\":\"other\"}'";

    unsafe { std::env::set_var("WEAVEBACK_CARGO_BIN", "sh"); }
    run_cargo_annotated_to_writer(
        vec!["-c".to_string(), mock_script.to_string()],
        true, // diagnostics_only = true
        tmp.path().join("db"),
        tmp.path().join("gen"),
        EvalConfig::default(),
        tmp.path(),
        &mut out,
    ).unwrap();
    unsafe { std::env::remove_var("WEAVEBACK_CARGO_BIN"); }

    let output = String::from_utf8(out).unwrap();
    assert!(output.contains("compiler-message"));
    assert!(!output.contains("other"), "expected 'other' message to be filtered out");
}
