// weaveback-api/src/apply_back/tests/primitives/do_patch.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── do_patch ─────────────────────────────────────────────────────────────

#[test]
fn do_patch_applies_exact_match() {
    let mut lines = lines("aaa\nbbb\nccc");
    let mut out = Vec::new();
    let mut skipped = 0;
    let mut applied = 0;
    let mut conflicts = 0;
    do_patch("f.adoc", 1, 1, "bbb", "BBB", &mut lines, false,
             &mut skipped, &mut applied, &mut conflicts, None, &mut out);
    assert_eq!(applied, 1);
    assert_eq!(lines[1], "BBB");
    let msg = String::from_utf8(out).unwrap();
    assert!(msg.contains("patched"));
}

#[test]
fn do_patch_detects_already_applied() {
    let mut lines = lines("aaa\nBBB\nccc");
    let mut out = Vec::new();
    let mut skipped = 0;
    let mut applied = 0;
    let mut conflicts = 0;
    do_patch("f.adoc", 1, 1, "bbb", "BBB", &mut lines, false,
             &mut skipped, &mut applied, &mut conflicts, None, &mut out);
    let msg = String::from_utf8(out).unwrap();
    assert!(msg.contains("already applied"));
}

#[test]
fn do_patch_records_conflict_when_no_match() {
    let mut lines = lines("aaa\nzzzz\nccc");
    let mut out = Vec::new();
    let mut skipped = 0;
    let mut applied = 0;
    let mut conflicts = 0;
    do_patch("f.adoc", 1, 1, "bbb", "BBB", &mut lines, false,
             &mut skipped, &mut applied, &mut conflicts, None, &mut out);
    assert_eq!(conflicts, 1);
    let msg = String::from_utf8(out).unwrap();
    assert!(msg.contains("CONFLICT"));
}

#[test]
fn do_patch_dry_run_does_not_modify_lines() {
    let mut lines = lines("aaa\nbbb\nccc");
    let mut out = Vec::new();
    let mut skipped = 0;
    let mut applied = 0;
    let mut conflicts = 0;
    do_patch("f.adoc", 1, 1, "bbb", "BBB", &mut lines, true,
             &mut skipped, &mut applied, &mut conflicts, None, &mut out);
    assert_eq!(lines[1], "bbb", "dry-run must not modify content");
    let msg = String::from_utf8(out).unwrap();
    assert!(msg.contains("dry-run"));
}
