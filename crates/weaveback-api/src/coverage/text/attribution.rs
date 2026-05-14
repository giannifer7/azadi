// weaveback-api/src/coverage/text/attribution.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

pub fn collect_text_attributions(
    text: &str,
    db: Option<&weaveback_tangle::db::WeavebackDb>,
    project_root: &Path,
    resolver: &PathResolver,
    eval_config: &EvalConfig,
) -> Vec<serde_json::Value> {
    let Some(db) = db else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for location in scan_generated_locations(text) {
        let Ok((out_file, line, col)) = parse_generated_location(&location) else {
            continue;
        };
        let Some(trace) = trace_generated_location(
            &out_file,
            line,
            col,
            db,
            project_root,
            resolver,
            eval_config,
        ) else {
            out.push(json!({
                "location": location,
                "ok": false,
                "trace": serde_json::Value::Null,
            }));
            continue;
        };
        out.push(json!({
            "location": location,
            "ok": true,
            "trace": trace,
        }));
    }
    out
}

pub fn emit_text_attribution_message(
    stream: &str,
    line: &str,
    attributions: Vec<serde_json::Value>,
    mut out: impl Write,
) -> std::io::Result<()> {
    serde_json::to_writer(
        &mut out,
        &json!({
            "reason": "weaveback-text-attribution",
            "stream": stream,
            "text": line,
            "weaveback_attributions": attributions,
            "weaveback_source_summary": build_location_attribution_summary(
                &attributions
            ),
        }),
    )?;
    writeln!(out)?;
    Ok(())
}
