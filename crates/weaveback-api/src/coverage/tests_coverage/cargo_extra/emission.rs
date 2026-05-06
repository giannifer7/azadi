// weaveback-api/src/coverage/tests_coverage/cargo_extra/emission.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

#[test]
fn emit_cargo_summary_message_outputs_json_with_reason() {
    let mut out = Vec::new();
    emit_cargo_summary_message(5, &[], &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    let val: serde_json::Value = serde_json::from_str(s.trim()).unwrap();
    assert_eq!(val["reason"], "weaveback-summary");
    assert_eq!(val["compiler_message_count"], 5);
    assert_eq!(val["generated_span_count"], 0);
}

#[test]
fn test_emit_augmented_cargo_message() {
    let mut out = Vec::new();
    let diag_json = json!({"reason": "compiler-message", "message": {"spans": []}});
    let original = serde_json::to_string(&diag_json).unwrap();

    emit_augmented_cargo_message(&original, vec![json!({"ok":true})], vec![], &mut out).unwrap();
    let result = String::from_utf8(out).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["reason"], "compiler-message");
    assert!(parsed.get("weaveback_attributions").is_some());
}

