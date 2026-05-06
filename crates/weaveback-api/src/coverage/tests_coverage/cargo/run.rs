// weaveback-api/src/coverage/tests_coverage/cargo/run.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;
use super::CARGO_TEST_MUTEX;

#[test]
fn run_cargo_annotated_to_writer_traces_real_generated_compile_error() {
    let _guard = CARGO_TEST_MUTEX.lock().unwrap();
    let temp = tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    std::fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "wb-fixture"
version = "0.1.0"
edition = "2024"
"#,
    )
    .expect("Cargo.toml");
    std::fs::write(
        root.join("src/main.rs"),
        "mod generated;\nfn main() { generated::broken(); }\n",
    )
    .expect("main");
    std::fs::write(
        root.join("src/generated.rs"),
        "pub fn broken() { let x = ; }\n",
    )
    .expect("generated");

    let db_path = root.join("weaveback.db");
    let mut db = WeavebackDb::open(&db_path).expect("db");
    db.set_noweb_entries(
        "src/generated.rs",
        &[(
            0,
            NowebMapEntry {
                src_file: "src/doc.adoc".to_string(),
                chunk_name: "generated".to_string(),
                src_line: 3,
                indent: String::new(),
                confidence: Confidence::Exact,
            },
        )],
    )
    .expect("noweb");
    db.set_src_snapshot("src/doc.adoc", b"= Root\n\n== Generated\nThe generated body.\n")
        .expect("snapshot");

    let mut out = Vec::new();
    let err = run_cargo_annotated_to_writer(
        vec!["check".to_string(), "--quiet".to_string()],
        true,
        db_path,
        root.join("gen"),
        EvalConfig::default(),
        root,
        &mut out,
    )
    .expect_err("cargo should fail on generated syntax error");
    let rendered = String::from_utf8(out).expect("utf8");
    let lines = rendered
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("json line"))
        .collect::<Vec<_>>();

    assert!(matches!(err, CoverageApiError::Io(_)));
    let compiler = lines
        .iter()
        .find(|value| value["reason"] == "compiler-message")
        .expect("compiler message");
    let span_attrs = compiler["weaveback_span_attributions"]
        .as_array()
        .expect("span attributions");
    assert!(!span_attrs.is_empty());
    assert!(span_attrs.iter().any(|record| {
        record["trace"]["src_file"]
            .as_str()
            .or_else(|| record["trace"]["expanded_file"].as_str())
            .is_some_and(|path| path.ends_with("src/doc.adoc"))
            && record["trace"]["source_section_breadcrumb"] == json!(["Root", "Generated"])
    }));

    let summary = lines
        .iter()
        .find(|value| value["reason"] == "weaveback-summary")
        .expect("summary");
    let sections = summary["weaveback_source_summary"]["sources"][0]["sections"]
        .as_array()
        .expect("sections");
    assert!(sections.iter().any(|section| {
        section["source_section_breadcrumb"] == json!(["Root", "Generated"])
            && section["generated_spans"]
                .as_array()
                .is_some_and(|spans| spans.iter().any(|span| {
                    span["generated_file"]
                        .as_str()
                        .is_some_and(|file| file.ends_with("src/generated.rs"))
                }))
    }));
}

#[test]
fn run_cargo_annotated_to_writer_emits_text_attribution_for_text_warning() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    std::fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "wb-fixture-warning"
version = "0.1.0"
edition = "2024"
build = "build.rs"
"#,
    )
    .expect("Cargo.toml");
    std::fs::write(
        root.join("build.rs"),
        "fn main() { println!(\"cargo:warning=src/generated.rs:1:27\"); }\n",
    )
    .expect("build");
    std::fs::write(
        root.join("src/main.rs"),
        "fn main() {}\n",
    )
    .expect("main");
    std::fs::write(
        root.join("src/generated.rs"),
        "pub fn generated() {}\n",
    )
    .expect("generated");

    let db_path = root.join("weaveback.db");
    let mut db = WeavebackDb::open(&db_path).expect("db");
    db.set_noweb_entries(
        "src/generated.rs",
        &[(
            0,
            NowebMapEntry {
                src_file: "src/doc.adoc".to_string(),
                chunk_name: "generated".to_string(),
                src_line: 3,
                indent: String::new(),
                confidence: Confidence::Exact,
            },
        )],
    )
    .expect("noweb");
    db.set_src_snapshot("src/doc.adoc", b"= Root\n\n== Generated\nThe generated body.\n")
        .expect("snapshot");

    let _guard = CARGO_TEST_MUTEX.lock().unwrap();
    let mut out = Vec::new();
    run_cargo_annotated_to_writer(
        vec![
            "check".to_string(),
        ],
        true,
        db_path,
        root.join("gen"),
        EvalConfig::default(),
        root,
        &mut out,
    )
    .expect("cargo check should succeed");
    let rendered = String::from_utf8(out).expect("utf8");
    let lines = rendered
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("json line"))
        .collect::<Vec<_>>();

    let text_attr = lines
        .iter()
        .find(|value| value["reason"] == "weaveback-text-attribution")
        .expect("text attribution");
    assert_eq!(text_attr["stream"], "stderr");
    assert!(
        text_attr["weaveback_attributions"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| {
                item["trace"]["source_section_breadcrumb"] == json!(["Root", "Generated"])
            }))
    );
}
