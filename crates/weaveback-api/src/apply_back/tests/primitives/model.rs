// weaveback-api/src/apply_back/tests/primitives/model.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── ApplyBackError Display ─────────────────────────────────────────────

#[test]
fn apply_back_error_display_io_variant() {
    let e = ApplyBackError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
    let s = format!("{e}");
    assert!(s.contains("I/O error"), "got: {s}");
}

// ── patch_source_rank ──────────────────────────────────────────────────

#[test]
fn patch_source_rank_macro_arg_outranks_literal() {
    let arg = PatchSource::MacroArg {
        src_file: "f".into(), src_line: 1, src_col: 0,
        macro_name: "m".into(), param_name: "p".into(),
    };
    let lit = PatchSource::Literal { src_file: "f".into(), src_line: 1, len: 1 };
    assert!(patch_source_rank(&arg) > patch_source_rank(&lit));
}

#[test]
fn patch_source_rank_unpatchable_is_lowest() {
    let unp = PatchSource::Unpatchable { src_file: "f".into(), src_line: 1, kind_label: "x".into() };
    let nw = PatchSource::Noweb { src_file: "f".into(), src_line: 1, len: 1 };
    assert!(patch_source_rank(&unp) < patch_source_rank(&nw));
}
// ── patch_source_location ──────────────────────────────────────────────

#[test]
fn patch_source_location_returns_file_and_line() {
    let lit = PatchSource::Literal { src_file: "src/foo.adoc".into(), src_line: 42, len: 1 };
    let (file, line) = patch_source_location(&lit);
    assert_eq!(file, "src/foo.adoc");
    assert_eq!(line, 42);
}

// ── strip_indent ────────────────────────────────────────────────────────

#[test]
fn strip_indent_removes_prefix() {
    let result = strip_indent("    hello world", "    ");
    assert_eq!(result, "hello world");
}

#[test]
fn strip_indent_returns_original_when_no_match() {
    let result = strip_indent("hello", "    ");
    assert_eq!(result, "hello");
}
