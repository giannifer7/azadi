// weaveback-api/src/lookup/span.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::lookup::context::append_source_context;
use serde_json::Value;
use weaveback_macro::evaluator::Evaluator;
use weaveback_macro::evaluator::output::{PreciseTracingOutput, SourceSpan, SpanKind, SpanRange};
use weaveback_tangle::db::WeavebackDb;

/// Find the `SourceSpan` covering `col_char_0` (0-indexed character position)
/// of 0-indexed `line_0` in the given expanded text and span ranges.
pub(in crate::lookup) fn span_at_line<'a>(
    expanded: &str,
    ranges: &'a [SpanRange],
    line_0: u32,
    col_char_0: u32,
) -> Option<&'a SourceSpan> {
    let line_start = if line_0 == 0 {
        0usize
    } else {
        let mut count = 0u32;
        let mut found = None;
        for (i, b) in expanded.bytes().enumerate() {
            if b == b'\n' {
                count += 1;
                if count == line_0 {
                    found = Some(i + 1);
                    break;
                }
            }
        }
        found?
    };
    // Convert 0-indexed char position to byte offset within the line.
    let line_text = &expanded[line_start..];
    let byte_col = line_text
        .char_indices()
        .nth(col_char_0 as usize)
        .map(|(i, _)| i)
        .unwrap_or(line_text.len());
    PreciseTracingOutput::span_at_byte(ranges, line_start + byte_col)
}

/// Append macro-level fields to `result` from `span`.
pub(in crate::lookup) fn append_span_fields(
    result: &mut Value,
    span: &SourceSpan,
    sources: &Evaluator,
) {
    use weaveback_tangle::lookup::find_line_col;

    let src_manager = sources.sources();
    let Some(src_path) = src_manager.source_files().get(span.src as usize) else {
        return;
    };
    let Some(src_bytes) = src_manager.get_source(span.src) else {
        return;
    };
    let src_content = String::from_utf8_lossy(src_bytes);
    let (src_line_1, src_col_1) = find_line_col(&src_content, span.pos);

    let obj = result.as_object_mut().unwrap();
    obj.insert("src_file".into(), Value::String(src_path.to_string_lossy().into_owned()));
    obj.insert("src_line".into(), Value::Number(src_line_1.into()));
    obj.insert("src_col".into(), Value::Number(src_col_1.into()));
    append_source_context(obj, &src_content, src_line_1 as usize);

    let kind_str = match &span.kind {
        SpanKind::Literal => "Literal",
        SpanKind::MacroBody { .. } => "MacroBody",
        SpanKind::MacroArg { .. } => "MacroArg",
        SpanKind::VarBinding { .. } => "VarBinding",
        SpanKind::Computed => "Computed",
    };
    obj.insert("kind".into(), Value::String(kind_str.to_string()));

    match &span.kind {
        SpanKind::MacroBody { macro_name } => {
            obj.insert("macro_name".into(), Value::String(macro_name.clone()));
        }
        SpanKind::MacroArg { macro_name, param_name } => {
            obj.insert("macro_name".into(), Value::String(macro_name.clone()));
            obj.insert("param_name".into(), Value::String(param_name.clone()));
        }
        SpanKind::VarBinding { var_name } => {
            obj.insert("var_name".into(), Value::String(var_name.clone()));
        }
        _ => {}
    }
}

/// Look up definition sites from the db and append them to `obj` as a JSON array.
/// Each entry has `file`, `line` (1-indexed), and `col` (1-indexed character position).
/// `use_var_defs`: true → query VAR_DEFS, false → query MACRO_DEFS.
pub(in crate::lookup) fn append_def_locations(
    obj: &mut serde_json::Map<String, Value>,
    field: &str,
    name: &str,
    db: &WeavebackDb,
    use_var_defs: bool,
) {
    use serde_json::json;
    use weaveback_tangle::lookup::find_line_col;

    let entries = if use_var_defs {
        db.query_var_defs(name)
    } else {
        db.query_macro_defs(name)
    };
    let Ok(entries) = entries else { return };
    if entries.is_empty() { return }
    let locations: Vec<Value> = entries.into_iter().filter_map(|(src_file, pos, _length)| {
        // Resolve position → (line, col) using the stored snapshot.
        let bytes = db.get_src_snapshot(&src_file).ok()??;
        let text = String::from_utf8_lossy(&bytes);
        let (line_1, col_1) = find_line_col(&text, pos as usize);
        Some(json!({
            "file": src_file,
            "line": line_1,
            "col":  col_1,
        }))
    }).collect();
    if !locations.is_empty() {
        obj.insert(field.into(), Value::Array(locations));
    }
}
