// weaveback-api/src/lookup/trace.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::lookup::{LookupError, PathResolver, WeavebackDb};
use serde_json::Value;
use weaveback_macro::evaluator::EvalConfig;

use crate::lookup::context::build_source_context_value;
use serde_json::json;
use weaveback_tangle::lookup::{find_best_noweb_entry, find_best_source_config};

fn trace_warnings_enabled() -> bool {
    std::env::var_os("WB_TRACE_WARNINGS").is_some()
}

pub fn load_source_text(
    src_file: &str,
    db: &WeavebackDb,
    resolver: &PathResolver,
) -> Result<String, LookupError> {
    let src_path = resolver.resolve_src(src_file);
    if let Ok(Some(bytes)) = db.get_src_snapshot(src_file) {
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    } else {
        std::fs::read_to_string(&src_path).map_err(LookupError::Io)
    }
}

pub fn perform_trace_coarse(
    out_file: &str,
    line: u32,
    db: &WeavebackDb,
    resolver: &PathResolver,
) -> Result<Option<Value>, LookupError> {
    if line == 0 {
        return Err(LookupError::InvalidInput("Line number must be >= 1".to_string()));
    }
    let out_line_0 = line - 1;

    let nw_entry = match find_best_noweb_entry(db, out_file, out_line_0, resolver)? {
        None => return Ok(None),
        Some(e) => e,
    };

    let mut result = json!({
        "out_file": out_file,
        "out_line": line,
        "chunk": nw_entry.chunk_name,
        "expanded_file": nw_entry.src_file,
        "expanded_line": nw_entry.src_line + 1,
        "indent": nw_entry.indent,
        "confidence": nw_entry.confidence.as_str(),
    });

    match load_source_text(&nw_entry.src_file, db, resolver) {
        Ok(src_content) => {
            let context = build_source_context_value(&src_content, (nw_entry.src_line + 1) as usize);
            if let Some(ctx) = context.as_object() {
                result.as_object_mut().unwrap().extend(ctx.clone());
            }
        }
        Err(e) => {
            if trace_warnings_enabled() {
                eprintln!("Warning: cannot read {} for trace: {:?}", nw_entry.src_file, e);
            }
        }
    }

    Ok(Some(result))
}

pub fn perform_trace(
    out_file: &str,
    line: u32,
    col: u32,
    db: &WeavebackDb,
    resolver: &PathResolver,
    eval_config: EvalConfig,
) -> Result<Option<Value>, LookupError> {
    use crate::lookup::span::{append_def_locations, append_span_fields, span_at_line};
    use weaveback_macro::evaluator::output::SpanKind;
    use weaveback_macro::evaluator::Evaluator;
    use weaveback_macro::macro_api::process_string_precise;

    let mut result = match perform_trace_coarse(out_file, line, db, resolver)? {
        None => return Ok(None),
        Some(value) => value,
    };

    let Some(nw_entry) = find_best_noweb_entry(db, out_file, line - 1, resolver)? else {
        return Ok(Some(result));
    };
    let src_path = resolver.resolve_src(&nw_entry.src_file);
    let src_content = match load_source_text(&nw_entry.src_file, db, resolver) {
        Ok(s) => s,
        Err(e) => {
            if trace_warnings_enabled() {
                eprintln!("Warning: cannot read {} for trace: {:?}", nw_entry.src_file, e);
            }
            return Ok(Some(result));
        }
    };
    let mut effective_eval_config = eval_config.clone();
    if let Ok(Some(cfg)) = find_best_source_config(db, &nw_entry.src_file) {
        effective_eval_config.sigil = cfg.sigil;
    }

    let mut evaluator = Evaluator::new(effective_eval_config);
    match process_string_precise(&src_content, Some(&src_path), &mut evaluator) {
        Ok((expanded, ranges)) => {
            let expanded_line_0 = nw_entry.src_line;
            // `col` is a 1-indexed character position in the *output* file line,
            // which has `nw_entry.indent` prepended by noweb.  Subtract the
            // indent char count, then convert to 0-indexed before querying the
            // span map.  col=0 is treated as col=1 (default: start of line).
            let indent_char_len = nw_entry.indent.chars().count() as u32;
            let col_1 = col.max(1);
            if col_1 > indent_char_len {
                let adjusted_col_0 = col_1 - 1 - indent_char_len;
                if let Some(span) = span_at_line(&expanded, &ranges, expanded_line_0, adjusted_col_0) {
                    append_span_fields(&mut result, span, &evaluator);
                    let obj = result.as_object_mut().unwrap();
                    match &span.kind {
                        SpanKind::VarBinding { var_name } => {
                            append_def_locations(obj, "set_locations", var_name, db, true);
                        }
                        SpanKind::MacroBody { macro_name } => {
                            append_def_locations(obj, "def_locations", macro_name, db, false);
                        }
                        SpanKind::Computed => {}
                        _ => {}
                    }
                }
            }
        }
        Err(e) => {
            if trace_warnings_enabled() {
                eprintln!("Warning: re-evaluation for trace failed: {e}");
            }
        }
    }

    Ok(Some(result))
}
