// weaveback-api/src/coverage/cargo/attribution.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

pub fn collect_cargo_attributions(
    diagnostic: &CargoDiagnostic,
    db: Option<&weaveback_tangle::db::WeavebackDb>,
    project_root: &Path,
    resolver: &PathResolver,
    eval_config: &EvalConfig,
) -> Vec<serde_json::Value> {
    let Some(db) = db else {
        return Vec::new();
    };
    let mut records = Vec::new();
    let mut seen = HashSet::new();

    for span in diagnostic.spans.iter().filter(|span| span.is_primary) {
        let Some(trace) = trace_generated_location(
            &span.file_name,
            span.line_start,
            span.column_start,
            db,
            project_root,
            resolver,
            eval_config,
        ) else {
            continue;
        };

        let dedupe_key = serde_json::to_string(&trace).unwrap_or_default();
        if seen.insert(dedupe_key) {
            records.push(trace);
        }
    }

    records
}

pub fn collect_cargo_span_attributions(
    diagnostic: &CargoDiagnostic,
    db: Option<&weaveback_tangle::db::WeavebackDb>,
    project_root: &Path,
    resolver: &PathResolver,
    eval_config: &EvalConfig,
) -> Vec<serde_json::Value> {
    let Some(db) = db else {
        return Vec::new();
    };
    let mut records = Vec::new();
    let mut seen = HashSet::new();

    for span in &diagnostic.spans {
        let Some(trace) = trace_generated_location(
            &span.file_name,
            span.line_start,
            span.column_start,
            db,
            project_root,
            resolver,
            eval_config,
        ) else {
            continue;
        };

        let record = json!({
            "generated_file": span.file_name,
            "generated_line": span.line_start,
            "generated_col": span.column_start,
            "is_primary": span.is_primary,
            "trace": trace,
        });
        let dedupe_key = serde_json::to_string(&record).unwrap_or_default();
        if seen.insert(dedupe_key) {
            records.push(record);
        }
    }

    records
}

pub(in crate::coverage) fn trace_generated_location(
    file_name: &str,
    line: u32,
    col: u32,
    db: &weaveback_tangle::db::WeavebackDb,
    project_root: &Path,
    resolver: &PathResolver,
    eval_config: &EvalConfig,
) -> Option<serde_json::Value> {
    if let Ok(Some(value)) =
        lookup::perform_trace(file_name, line, col, db, resolver, eval_config.clone())
    {
        return Some(value);
    }

    let file_path = Path::new(file_name);
    let rel = file_path
        .strip_prefix(project_root)
        .ok()
        .map(|path| path.to_string_lossy().replace('\\', "/"))?;
    lookup::perform_trace(&rel, line, col, db, resolver, eval_config.clone())
        .ok()
        .flatten()
}
