// weaveback-api/src/coverage/tests_coverage/cargo_extra/text.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

#[test]
fn test_collect_text_attributions_scans_locations() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("test.db");
    let mut db = WeavebackDb::open(&db_path).unwrap();
    let project_root = tmp.path().to_path_buf();
    let resolver = PathResolver::new(project_root.clone(), project_root.join("gen"));

    ws_write_file(&project_root, "src/a.adoc", b"content");
    db.set_noweb_entries("out.rs", &[(9, weaveback_tangle::db::NowebMapEntry {
        src_file: "src/a.adoc".to_string(),
        chunk_name: "main".to_string(),
        src_line: 5,
        indent: "".into(),
        confidence: Confidence::Exact,
    })]).unwrap();

    let text = "Error at out.rs:10:1 and some other text";
    let attributions = collect_text_attributions(
        text,
        Some(&db),
        &project_root,
        &resolver,
        &EvalConfig::default(),
    );

    assert_eq!(attributions.len(), 1);
    assert_eq!(attributions[0]["location"], "out.rs:10:1");
    assert_eq!(attributions[0]["ok"], true);
}

#[test]
fn test_emit_text_attribution_message() {
    let mut out = Vec::new();
    let attributions = vec![json!({"location": "out.rs:1:1", "ok": false})];

    emit_text_attribution_message("stdout", "some test line", attributions, &mut out).unwrap();
    let result = String::from_utf8(out).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["reason"], "weaveback-text-attribution");
    assert_eq!(parsed["stream"], "stdout");
    assert_eq!(parsed["text"], "some test line");
}

