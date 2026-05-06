// weaveback-api/src/coverage/tests_coverage/cargo/emission.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

#[test]
fn emit_augmented_cargo_message_attaches_full_trace_json() {
    let line = r#"{"reason":"compiler-message","message":{"spans":[]}}"#;
    let records = vec![json!({
        "out_file": "gen/out.rs",
        "out_line": 17,
        "out_col": 9,
        "src_file": "src/doc.adoc",
        "src_line": 42,
        "src_col": 3,
        "chunk": "main",
        "kind": "Literal",
        "source_section_breadcrumb": ["Root", "Topic"],
        "source_section_prose": "Explain."
    })];
    let span_records = vec![json!({
        "generated_file": "gen/out.rs",
        "generated_line": 17,
        "generated_col": 9,
        "is_primary": true,
        "trace": records[0].clone(),
    })];
    let mut out = Vec::new();
    emit_augmented_cargo_message(line, records, span_records, &mut out).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let attrs = value["weaveback_attributions"].as_array().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0]["chunk"], "main");
    assert_eq!(attrs[0]["source_section_breadcrumb"], json!(["Root", "Topic"]));
    assert_eq!(attrs[0]["source_section_prose"], "Explain.");
    let span_attrs = value["weaveback_span_attributions"].as_array().unwrap();
    assert_eq!(span_attrs[0]["generated_file"], "gen/out.rs");
    assert_eq!(span_attrs[0]["trace"]["chunk"], "main");
    assert_eq!(value["weaveback_source_summary"]["sources"][0]["src_file"], "src/doc.adoc");
    assert_eq!(
        value["weaveback_source_summary"]["sources"][0]["sections"][0]["source_section_breadcrumb"],
        json!(["Root", "Topic"])
    );
    assert_eq!(
        value["weaveback_source_summary"]["sources"][0]["sections"][0]["generated_spans"][0]["generated_file"],
        "gen/out.rs"
    );
}

#[test]
fn emit_text_attribution_message_wraps_plain_text_line() {
    let mut out = Vec::new();
    emit_text_attribution_message(
        "stderr",
        "panic at src/generated.rs:1:27",
        vec![json!({
            "location": "src/generated.rs:1:27",
            "ok": true,
            "trace": {"expanded_file": "src/doc.adoc", "chunk": "generated"},
        })],
        &mut out,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(value["reason"], "weaveback-text-attribution");
    assert_eq!(value["stream"], "stderr");
    assert_eq!(value["text"], "panic at src/generated.rs:1:27");
    assert_eq!(value["weaveback_attributions"][0]["location"], "src/generated.rs:1:27");
    assert_eq!(
        value["weaveback_source_summary"]["sources"][0]["src_file"],
        "src/doc.adoc"
    );
}

#[test]
fn collect_text_attributions_scans_and_traces_locations() {
    let mut db = WeavebackDb::open_temp().expect("db");
    db.set_noweb_entries(
        "out.rs",
        &[(
            0,
            NowebMapEntry {
                src_file: "src/doc.adoc".to_string(),
                chunk_name: "main".to_string(),
                src_line: 3,
                indent: String::new(),
                confidence: Confidence::Exact,
            },
        )],
    )
    .expect("noweb");
    db.set_src_snapshot("src/doc.adoc", b"= Root\n\n== Topic\nalpha\n")
        .expect("snapshot");
    let resolver = PathResolver::new(PathBuf::from("."), PathBuf::from("gen"));
    let records = collect_text_attributions(
        "panic at out.rs:1 and out.rs:1",
        Some(&db),
        Path::new("."),
        &resolver,
        &EvalConfig::default(),
    );
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["location"], "out.rs:1");
    assert_eq!(records[0]["ok"], true);
    assert_eq!(records[0]["trace"]["chunk"], "main");
}

#[test]
fn build_location_attribution_summary_groups_successful_records() {
    let summary = build_location_attribution_summary(&[
        json!({
            "location": "out.rs:1",
            "ok": true,
            "trace": {
                "src_file": "src/doc.adoc",
                "chunk": "main",
                "source_section_breadcrumb": ["Root", "Topic"],
                "source_section_prose": "Explain."
            },
        }),
        json!({
            "location": "out.rs:2",
            "ok": false,
            "trace": serde_json::Value::Null,
        }),
    ]);
    assert_eq!(summary["count"], 1);
    assert_eq!(summary["sources"][0]["src_file"], "src/doc.adoc");
    assert_eq!(
        summary["sources"][0]["sections"][0]["locations"],
        json!(["out.rs:1"])
    );
}

#[test]
fn emit_cargo_summary_message_emits_final_grouped_json() {
    let span_records = vec![
        json!({
            "generated_file": "gen/out.rs",
            "generated_line": 17,
            "generated_col": 9,
            "is_primary": true,
            "trace": {
                "src_file": "src/doc.adoc",
                "chunk": "main",
                "source_section_breadcrumb": ["Root", "Topic"],
                "source_section_prose": "Explain."
            },
        }),
        json!({
            "generated_file": "gen/out.rs",
            "generated_line": 20,
            "generated_col": 1,
            "is_primary": false,
            "trace": {
                "src_file": "src/doc.adoc",
                "chunk": "helper",
                "source_section_breadcrumb": ["Root", "Topic"],
                "source_section_prose": "Explain."
            },
        }),
    ];
    let mut out = Vec::new();
    emit_cargo_summary_message(3, &span_records, &mut out).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(value["reason"], "weaveback-summary");
    assert_eq!(value["compiler_message_count"], 3);
    assert_eq!(value["generated_span_count"], 2);
    assert_eq!(value["weaveback_source_summary"]["sources"][0]["src_file"], "src/doc.adoc");
    assert_eq!(
        value["weaveback_source_summary"]["sources"][0]["sections"][0]["chunks"],
        json!(["helper", "main"])
    );
}

