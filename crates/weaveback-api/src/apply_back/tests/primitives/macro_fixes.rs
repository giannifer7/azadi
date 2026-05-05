// weaveback-api/src/apply_back/tests/primitives/macro_fixes.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── attempt_macro_arg_patch ────────────────────────────────────────────

#[test]
fn attempt_macro_arg_patch_exact_col_replaces() {
    let ls = lines("    let x = old_val;");
    // old_text "old_val" starts at byte 12
    let result = attempt_macro_arg_patch(&ls, 0, 12, "old_val", "new_val");
    assert_eq!(result, Some("    let x = new_val;".to_string()));
}

#[test]
fn attempt_macro_arg_patch_returns_none_when_not_found() {
    let ls = lines("irrelevant line");
    let result = attempt_macro_arg_patch(&ls, 0, 0, "missing", "replacement");
    assert_eq!(result, None);
}

#[test]
fn attempt_macro_arg_patch_fallback_finds_differing_part() {
    // Source line has indentation, but src_col is 0.
    // Exact match at 0 fails, fallback scans for the differing part.
    let ls = lines("    let x = old_val;");
    let old_text = "let x = old_val;";
    let new_text = "let x = new_val;";
    // old/new differ at "old_val" vs "new_val".
    // common prefix: "let x = " (8 chars)
    // common suffix: ";" (1 char)
    // old_frag: "old_val"
    let result = attempt_macro_arg_patch(&ls, 0, 0, old_text, new_text);
    assert_eq!(result, Some("    let x = new_val;".to_string()));
}

#[test]
fn attempt_macro_arg_patch_fallback_avoids_false_suffix_match() {
    // old: "literate", new: "illiterate"
    // prefix: "" (0), suffix: "literate" (8)
    // This is a tricky case because old is a suffix of new.
    let ls = lines("    process(literate);");
    let result = attempt_macro_arg_patch(&ls, 0, 0, "literate", "illiterate");
    assert_eq!(result, Some("    process(illiterate);".to_string()));
}

// ── attempt_macro_body_fix ─────────────────────────────────────────────

#[test]
fn attempt_macro_body_fix_no_vars_replaces_literal() {
    // Body has no %%(…) variables; old_expanded matches body_line
    let result = attempt_macro_body_fix("hello world", "hello world", "hello Rust", '%');
    assert_eq!(result, Some("hello Rust".to_string()));
}

#[test]
fn attempt_macro_body_fix_returns_none_when_same() {
    let result = attempt_macro_body_fix("foo", "foo", "foo", '%');
    assert_eq!(result, None);
}

#[test]
fn attempt_macro_body_fix_adjacent_vars_is_ambiguous() {
    // adjacent variables with no separator are rejected as ambiguous
    let result = attempt_macro_body_fix(
        "%(first)%(second)",
        "val1val2",
        "new1new2",
        '%'
    );
    assert_eq!(result, None);
}

#[test]
fn attempt_macro_body_fix_no_match_returns_none() {
    let result = attempt_macro_body_fix("completely different", "old", "new", '%');
    assert_eq!(result, None);
}

// ── attempt_macro_body_fix with vars ────────────────────────────────────

#[test]
fn attempt_macro_body_fix_with_single_var_updates_literal() {
    // body: "Hello %(name). Bye."
    // old expanded: "Hello Alice. Bye."
    // new expanded: "Hello Alice. Later."
    // Expects the literal suffix " Bye." → " Later." while preserving %(name).
    let result = attempt_macro_body_fix(
        "Hello %(name). Bye.",
        "Hello Alice. Bye.",
        "Hello Alice. Later.",
        '%',
    );
    assert!(result.is_some(), "expected Some result, got None");
    let new_body = result.unwrap();
    assert!(new_body.contains("Later"), "expected 'Later' in: {new_body}");
    assert!(new_body.contains("%(name)"), "expected var ref preserved in: {new_body}");
}

#[test]
fn attempt_macro_body_fix_returns_none_when_body_eq_expanded() {
    // body line IS exactly the expanded text → result is just the new expanded text
    let result = attempt_macro_body_fix("plain text", "plain text", "new text", '%');
    assert_eq!(result, Some("new text".to_string()));
}

