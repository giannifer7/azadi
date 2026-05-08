// weaveback-api/src/apply_back/tests/runner/success.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── successful reconciliation paths ────────────────────────────────────

#[test]
fn test_run_apply_back_success_literal() {
    let ws = TestWorkspace::new();
    let mut db = ws.open_db();

    let src_rel = "src/main.adoc";
    let gen_rel = "main.rs";
    let src_abs = ws.root.join(src_rel);

    // Initial setup: source file has a literal line.
    let src_content = "= File\n\n<<main>>=\noriginal line\n@\n";
    ws.write_file(src_rel, src_content.as_bytes());

    // Seed DB with baseline and source map
    db.set_baseline(gen_rel, b"original line\n").unwrap();
    db.set_noweb_entries(gen_rel, &[(0, weaveback_tangle::db::NowebMapEntry {
        src_file: src_rel.to_string(),
        chunk_name: "main".to_string(),
        src_line: 3, // 0-indexed "original line" is on line 3
        indent: "".into(),
        confidence: weaveback_tangle::db::Confidence::Exact,
    })]).unwrap();
    db.set_src_snapshot(src_rel, src_content.as_bytes()).unwrap();

    // Modify generated file
    ws.write_file(&format!("gen/{}", gen_rel), b"modified line\n");

    let opts = ApplyBackOptions {
        db_path: ws.root.join("weaveback.db"),
        gen_dir: ws.root.join("gen"),
        files: vec![],
        dry_run: false,
        eval_config: None,
    };
    let mut out = Vec::new();
    run_apply_back(opts, &mut out).unwrap();

    // Verify output message
    let msg = String::from_utf8(out).unwrap();
    assert!(msg.contains("patched"), "expected 'patched' in: {msg}");

    // Verify source file was actually updated
    let updated_src = std::fs::read_to_string(src_abs).unwrap();
    assert!(updated_src.contains("modified line"), "source file not updated: {updated_src}");
}

#[test]
fn test_run_apply_back_macro_edit() {
    let ws = TestWorkspace::new();
    let mut db = ws.open_db();

    let driver_rel = "src/driver.adoc";
    let macro_rel = "src/macros.adoc";
    let gen_rel = "out.txt";

    // Setup: Driver includes macros and calls a macro.
    let driver_content = "= Driver\n<<include macros.adoc>>\n<<@file out.txt>>=\n<<the-macro>>\n@\n";
    let macro_content = "<<the-macro>>=\noriginal macro body\n@\n";

    ws.write_file(driver_rel, driver_content.as_bytes());
    ws.write_file(macro_rel, macro_content.as_bytes());

    // Seed DB
    db.set_baseline(gen_rel, b"original macro body\n").unwrap();
    db.set_noweb_entries(gen_rel, &[(0, weaveback_tangle::db::NowebMapEntry {
        src_file: macro_rel.to_string(),
        chunk_name: "the-macro".to_string(),
        src_line: 1, // line 1 of macros.adoc
        indent: "".into(),
        confidence: weaveback_tangle::db::Confidence::Exact,
    })]).unwrap();
    db.set_src_snapshot(driver_rel, driver_content.as_bytes()).unwrap();
    db.set_src_snapshot(macro_rel, macro_content.as_bytes()).unwrap();

    // Modify generated file
    ws.write_file(&format!("gen/{}", gen_rel), b"modified macro body\n");

    let opts = ApplyBackOptions {
        db_path: ws.root.join("weaveback.db"),
        gen_dir: ws.root.join("gen"),
        files: vec![],
        dry_run: false,
        eval_config: None,
    };
    let mut out = Vec::new();
    run_apply_back(opts, &mut out).unwrap();

    // Verify macro source file was updated, not the driver.
    let updated_macro = std::fs::read_to_string(ws.root.join(macro_rel)).unwrap();
    assert!(updated_macro.contains("modified macro body"), "macro source not updated: {updated_macro}");

    let updated_driver = std::fs::read_to_string(ws.root.join(driver_rel)).unwrap();
    assert!(updated_driver.contains("<<the-macro>>"), "driver source should not be updated: {updated_driver}");
}
