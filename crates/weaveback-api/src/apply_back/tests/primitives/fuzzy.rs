// weaveback-api/src/apply_back/tests/primitives/fuzzy.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── fuzzy_find_line ────────────────────────────────────────────────────

#[test]
fn fuzzy_find_line_finds_unique_match() {
    let ls = lines("foo\nbar baz\nqux");
    assert_eq!(fuzzy_find_line(&ls, 1, "bar baz", 5), Some(1));
}

#[test]
fn fuzzy_find_line_returns_none_when_ambiguous() {
    let ls = lines("foo\nfoo\nfoo");
    assert_eq!(fuzzy_find_line(&ls, 1, "foo", 5), None);
}

#[test]
fn fuzzy_find_line_returns_none_outside_window() {
    let ls = lines("match\nother\nother\nother\nother\nother\nother\nother\nother\nother");
    // center=9, window=0 — "match" is at index 0, distance 9 > window 0
    assert_eq!(fuzzy_find_line(&ls, 9, "match", 0), None);
}

#[test]
fn fuzzy_find_line_tolerates_leading_whitespace() {
    // The pattern is anchored with ^\s* and \s*$, so leading/trailing
    // spaces in the source line are ignored.
    let ls = lines("   bar baz   ");
    assert_eq!(fuzzy_find_line(&ls, 0, "bar baz", 0), Some(0));
}
