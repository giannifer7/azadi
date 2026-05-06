// weaveback-api/src/apply_back/tests/primitives/oracle.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── splice_line ────────────────────────────────────────────────────────

#[test]
fn splice_line_replaces_indexed_line() {
    let ls = lines("aaa\nbbb\nccc");
    let result = splice_line(&ls, 1, "BBB", false);
    assert_eq!(result, "aaa\nBBB\nccc");
}

#[test]
fn splice_line_preserves_trailing_newline() {
    let ls = lines("x\ny");
    let result = splice_line(&ls, 0, "X", true);
    assert!(result.ends_with('\n'));
}

// ── token_overlap_score ────────────────────────────────────────────────

#[test]
fn token_overlap_score_counts_shared_tokens() {
    // "hello world" shares "hello" with old and "world" with new
    let score = token_overlap_score("hello world", "hello foo", "world bar");
    assert!(score > 0, "expected positive score, got {score}");
}

#[test]
fn token_overlap_score_zero_when_no_overlap() {
    let score = token_overlap_score("abc", "xyz", "uvw");
    assert_eq!(score, 0);
}

// ── differing_token_pair ───────────────────────────────────────────────

#[test]
fn differing_token_pair_single_diff_returns_pair() {
    let result = differing_token_pair("foo bar", "foo baz");
    assert_eq!(result, Some(("bar".to_string(), "baz".to_string())));
}

#[test]
fn differing_token_pair_returns_none_when_multiple_diffs() {
    let result = differing_token_pair("foo bar", "qux baz");
    assert_eq!(result, None);
}

#[test]
fn differing_token_pair_returns_none_when_token_counts_differ() {
    let result = differing_token_pair("foo", "foo bar");
    assert_eq!(result, None);
}
