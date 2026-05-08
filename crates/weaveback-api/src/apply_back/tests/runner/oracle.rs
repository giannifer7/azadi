// weaveback-api/src/apply_back/tests/runner/oracle.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── oracle rejection paths ──────────────────────────────────────────────

#[test]
fn test_apply_back_oracle_rejection_on_mismatch() {
    let ws = TestWorkspace::new();
    let mut db = ws.open_db();

    let src_rel = "src/test.adoc";
    let gen_rel = "test.rs";

    ws.write_file(src_rel, "<<main>>=\noriginal\n@\n".as_bytes());
    db.set_baseline(gen_rel, b"original\n").unwrap();
    db.set_noweb_entries(gen_rel, &[(0, weaveback_tangle::db::NowebMapEntry {
        src_file: src_rel.to_string(),
        chunk_name: "main".to_string(),
        src_line: 1,
        indent: "".into(),
        confidence: weaveback_tangle::db::Confidence::Exact,
    })]).unwrap();
    db.set_src_snapshot(src_rel, b"<<main>>=\noriginal\n@\n").unwrap();

    // Target edit: change "original" to "new"
    ws.write_file(&format!("gen/{}", gen_rel), b"new\n");

    // Now, manually trigger a scenario where reconstruction fails.
    // We'll use apply_patches_to_file with a patch that doesn't match the source exactly
    // or ensure the oracle re-evaluates and finds a mismatch.

    let _opts = ApplyBackOptions {
        db_path: ws.root.join("weaveback.db"),
        gen_dir: ws.root.join("gen"),
        files: vec![],
        dry_run: false,
        eval_config: None,
    };

    // We'll simulate a failure by providing an incorrect expected_output in the oracle check if possible,
    // or just rely on the fact that if re-evaluation yields different text, it rejects.
    // Actually, the easiest way is to mock a Patch that target a wrong line.

    let ctx = FilePatchContext {
        src_file: src_rel,
        src_root: &ws.root,
        db: &db,
        patches: &[Patch {
            source: PatchSource::MacroBodyWithVars {
                src_file: src_rel.into(),
                src_line: 1,
                macro_name: "main".into(),
            },
            old_text: "original".into(),
            new_text: "new".into(),
            expanded_line: 0,
        }],
        dry_run: false,
        sigil: '<',
        eval_config: Some(EvalConfig::default()),
        snapshot: None,
    };

    let mut skipped = 0;
    let mut out = Vec::new();
    apply_patches_to_file(ctx, &mut skipped, &mut out).unwrap();

    let msg = String::from_utf8(out).unwrap();
    // The oracle will fail because the patched source (src_rel) will actually contain
    // a different result when re-evaluated.
    assert!(msg.contains("manual") || msg.contains("rejected"), "expected rejection in: {msg}");
    assert_eq!(skipped, 1);
}
