// weaveback-api/src/apply_back/tests/runner/entry.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── run_apply_back entry point edge cases ──────────────────────────────

#[test]
fn run_apply_back_gen_dir_fallback() {
    let ws = TestWorkspace::new();
    let db = ws.open_db();
    // Set gen_dir in run_config.
    db.set_run_config("gen_dir", ws.root.join("alt_gen").to_str().unwrap()).unwrap();
    db.set_baseline("test.rs", b"content").unwrap();

    // Write file in alt_gen.
    ws.write_file("alt_gen/test.rs", b"MODIFIED");

    let opts = ApplyBackOptions {
        db_path: ws.root.join("weaveback.db"),
        gen_dir: std::path::PathBuf::from("gen"), // default doesn't exist
        files: vec![],
        dry_run: true,
        eval_config: None,
    };
    let mut out = Vec::new();

    // Should fall back to alt_gen from DB and find the MODIFIED file.
    run_apply_back(opts, &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("Processing test.rs"));
}

#[test]
fn run_apply_back_specific_files_non_existent_is_no_op() {
    let ws = TestWorkspace::new();
    let _db = ws.open_db(); // just creates it

    let opts = ApplyBackOptions {
        db_path: ws.root.join("weaveback.db"),
        gen_dir: ws.root.join("gen"),
        files: vec!["missing.rs".into()],
        dry_run: false,
        eval_config: None,
    };
    let mut out = Vec::new();
    run_apply_back(opts, &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    // Since missing.rs is not in baselines, it should say no modified files found.
    assert!(s.contains("No modified gen/ files found"));
}

#[test]
fn run_apply_back_diff_delete_is_detected() {
    let ws = TestWorkspace::new();
    let mut db = ws.open_db();
    db.set_baseline("out.rs", b"line1\nline2").unwrap();
    ws.write_file("gen/out.rs", b"line1\n"); // line2 deleted

    db.set_noweb_entries("out.rs", &[
        (0, NowebMapEntry { src_file: "src.adoc".into(), chunk_name: "c".into(), src_line: 0, indent: "".into(), confidence: Confidence::Exact }),
        (1, NowebMapEntry { src_file: "src.adoc".into(), chunk_name: "c".into(), src_line: 1, indent: "".into(), confidence: Confidence::Exact }),
    ]).unwrap();
    db.set_src_snapshot("src.adoc", b"line1\nline2\n").unwrap();
    ws.write_file("src.adoc", b"line1\nline2\n");

    let opts = ApplyBackOptions {
        db_path: ws.root.join("weaveback.db"),
        gen_dir: ws.root.join("gen"),
        files: vec![],
        dry_run: true,
        eval_config: None,
    };
    let mut out = Vec::new();
    run_apply_back(opts, &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    // Deletion of line 2 (out_line 1) should be detected.
    // It uses DiffOp::Delete logic.
    assert!(s.contains("Processing out.rs"));
}
