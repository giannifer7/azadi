// weaveback-api/src/coverage/tests_coverage/cargo_extra/attribution.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

#[test]
fn collect_cargo_attributions_maps_generated_span_back_to_source() {
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
    let diagnostic = CargoDiagnostic {
        spans: vec![CargoDiagnosticSpan {
            file_name: "out.rs".to_string(),
            line_start: 1,
            column_start: 1,
            is_primary: true,
        }],
    };

    let records = collect_cargo_attributions(
        &diagnostic,
        Some(&db),
        Path::new("."),
        &resolver,
        &EvalConfig::default(),
    );
    assert_eq!(records.len(), 1);
    assert!(
        records[0]["src_file"]
            .as_str()
            .is_some_and(|path| path.ends_with("src/doc.adoc"))
    );
    assert_eq!(records[0]["src_line"], 4);
    assert_eq!(records[0]["chunk"], "main");
    assert_eq!(records[0]["source_section_breadcrumb"], json!(["Root", "Topic"]));
}

#[test]
fn collect_cargo_span_attributions_keeps_generated_span_context() {
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
    let diagnostic = CargoDiagnostic {
        spans: vec![
            CargoDiagnosticSpan {
                file_name: "out.rs".to_string(),
                line_start: 1,
                column_start: 1,
                is_primary: true,
            },
            CargoDiagnosticSpan {
                file_name: "out.rs".to_string(),
                line_start: 1,
                column_start: 5,
                is_primary: false,
            },
        ],
    };

    let records = collect_cargo_span_attributions(
        &diagnostic,
        Some(&db),
        Path::new("."),
        &resolver,
        &EvalConfig::default(),
    );
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["generated_file"], "out.rs");
    assert_eq!(records[0]["trace"]["chunk"], "main");
    assert_eq!(records[1]["is_primary"], false);
}

#[test]
fn build_cargo_attribution_summary_groups_by_source_file() {
    let summary = build_cargo_attribution_summary(&[
        json!({
            "generated_file": "out.rs",
            "generated_line": 1,
            "generated_col": 1,
            "is_primary": true,
            "trace": {
                "src_file": "src/a.adoc",
                "chunk": "alpha",
                "source_section_breadcrumb": ["Root", "Alpha"],
                "source_section_prose": "Alpha prose."
            }
        }),
        json!({
            "generated_file": "out.rs",
            "generated_line": 2,
            "generated_col": 1,
            "is_primary": false,
            "trace": {
                "src_file": "src/a.adoc",
                "chunk": "beta",
                "source_section_breadcrumb": ["Root", "Alpha"],
                "source_section_prose": "Alpha prose."
            }
        }),
        json!({
            "generated_file": "out2.rs",
            "generated_line": 1,
            "generated_col": 1,
            "is_primary": true,
            "trace": {
                "src_file": "src/b.adoc",
                "chunk": "gamma",
                "source_section_breadcrumb": ["Root", "Beta"],
                "source_section_prose": "Beta prose."
            }
        }),
    ]);
    assert_eq!(summary["count"], 3);
    assert_eq!(summary["sources"][0]["src_file"], "src/a.adoc");
    assert_eq!(summary["sources"][0]["count"], 2);
    assert_eq!(
        summary["sources"][0]["sections"][0]["source_section_breadcrumb"],
        json!(["Root", "Alpha"])
    );
    assert_eq!(
        summary["sources"][0]["sections"][0]["generated_spans"][0]["generated_file"],
        "out.rs"
    );
    assert_eq!(summary["sources"][1]["src_file"], "src/b.adoc");
}

#[test]
fn test_collect_cargo_attributions_with_mock() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("test.db");
    let mut db = WeavebackDb::open(&db_path).unwrap();
    let project_root = tmp.path().to_path_buf();
    let resolver = PathResolver::new(project_root.clone(), project_root.join("gen"));

    let src_file = "src/main.adoc";
    ws_write_file(&project_root, src_file, b"content");

    db.set_noweb_entries("main.rs", &[(0, weaveback_tangle::db::NowebMapEntry {
        src_file: src_file.to_string(),
        chunk_name: "main".to_string(),
        src_line: 0,
        indent: "".into(),
        confidence: Confidence::Exact,
    })]).unwrap();

    let diag = CargoDiagnostic {
        spans: vec![CargoDiagnosticSpan {
            file_name: "main.rs".to_string(),
            line_start: 1,
            column_start: 1,
            is_primary: true,
        }],
    };

    let attributions = collect_cargo_attributions(
        &diag,
        Some(&db),
        &project_root,
        &resolver,
        &EvalConfig::default(),
    );
    assert_eq!(attributions.len(), 1);
    assert_eq!(attributions[0]["src_file"].as_str(), Some(project_root.join(src_file).to_string_lossy().as_ref()));
}
