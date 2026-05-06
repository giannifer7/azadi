// weaveback-api/src/apply_back/tests/primitives/verify.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── verify_candidate ────────────────────────────────────────────────────

#[test]
fn verify_candidate_returns_true_for_matching_line() {
    // A simple inline macro chunk: "Hello world!" → expanded line 0 = "Hello world!"
    let src = "Hello world!\n";
    let config = EvalConfig::default();
    let path = std::path::Path::new("test.adoc");
    assert!(verify_candidate(src, path, &config, 0, "Hello world!"));
}

#[test]
fn verify_candidate_returns_false_for_mismatched_line() {
    let src = "Hello world!\n";
    let config = EvalConfig::default();
    let path = std::path::Path::new("test.adoc");
    assert!(!verify_candidate(src, path, &config, 0, "Goodbye world!"));
}

#[test]
fn verify_candidate_returns_false_when_line_out_of_range() {
    let src = "only one line\n";
    let config = EvalConfig::default();
    let path = std::path::Path::new("test.adoc");
    assert!(!verify_candidate(src, path, &config, 99, "only one line"));
}
