// weaveback-api/src/apply_back/tests/runner/bulk.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── multi-file and direct apply edge cases ──────────────────────────────

#[test]
fn run_apply_back_bulk_reconciliation() {
    let ws = TestWorkspace::new();
    let mut db = ws.open_db();

    let a_rel = "a.rs";
    let b_rel = "b.rs";
    let src_a = "src/a.adoc";
    let src_b = "src/b.adoc";

    // Setup two files
    db.set_baseline(a_rel, b"line A\n").unwrap();
    db.set_baseline(b_rel, b"line B\n").unwrap();
    ws.write_file(&format!("gen/{}", a_rel), b"line A modified\n");
    ws.write_file(&format!("gen/{}", b_rel), b"line B modified\n");

    // Mock source mappings
    db.set_noweb_entries(a_rel, &[(0, weaveback_tangle::db::NowebMapEntry {
        src_file: src_a.to_string(),
        chunk_name: "main".to_string(),
        src_line: 1,
        indent: "".into(),
        confidence: Confidence::Exact,
    })]).unwrap();
    db.set_noweb_entries(b_rel, &[(0, weaveback_tangle::db::NowebMapEntry {
        src_file: src_b.to_string(),
        chunk_name: "main".to_string(),
        src_line: 1,
        indent: "".into(),
        confidence: Confidence::Exact,
    })]).unwrap();

    ws.write_file(src_a, b"<<main>>=\nline A\n@\n");
    ws.write_file(src_b, b"<<main>>=\nline B\n@\n");

    let opts = ApplyBackOptions {
        db_path: ws.root.join("weaveback.db"),
        gen_dir: ws.root.join("gen"),
        files: vec![],
        dry_run: false,
        eval_config: None,
    };
    let mut out = Vec::new();
    run_apply_back(opts, &mut out).unwrap();

    // Verify both sources patched
    assert!(fs::read_to_string(ws.root.join(src_a)).unwrap().contains("line A modified"));
    assert!(fs::read_to_string(ws.root.join(src_b)).unwrap().contains("line B modified"));
}

#[test]
fn apply_patches_to_file_missing_source_errors() {
    let ws = TestWorkspace::new();
    let db = ws.open_db();
    let ctx = FilePatchContext {
        src_file: "nonexistent.adoc",
        src_root: &ws.root,
        db: &db,
        patches: &[],
        dry_run: false,
        sigil: '%',
        eval_config: None,
        snapshot: None,
    };
    let mut skipped = 0;
    let mut out = Vec::new();
    let res = apply_patches_to_file(ctx, &mut skipped, &mut out);
    assert!(res.is_err());
}

#[test]
fn run_apply_back_with_restricted_files() {
    let ws = TestWorkspace::new();
    let db = ws.open_db();
    db.set_baseline("a.rs", b"line A\n").unwrap();
    db.set_baseline("b.rs", b"line B\n").unwrap();
    ws.write_file("gen/a.rs", b"mod A\n");
    ws.write_file("gen/b.rs", b"mod B\n");

    let opts = ApplyBackOptions {
        db_path: ws.root.join("weaveback.db"),
        gen_dir: ws.root.join("gen"),
        files: vec!["a.rs".to_string()], // ONLY a.rs
        dry_run: true,
        eval_config: None,
    };
    let mut out = Vec::new();
    run_apply_back(opts, &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("Processing a.rs"));
    assert!(!s.contains("Processing b.rs"));
}
