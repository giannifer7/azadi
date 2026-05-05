// weaveback-api/src/coverage/cargo/emit.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

pub fn emit_augmented_cargo_message(
    original_line: &str,
    attributions: Vec<serde_json::Value>,
    span_attributions: Vec<serde_json::Value>,
    mut out: impl Write,
) -> std::io::Result<()> {
    let mut value: serde_json::Value = match serde_json::from_str(original_line) {
        Ok(value) => value,
        Err(_) => {
            writeln!(out, "{original_line}")?;
            return Ok(());
        }
    };
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "weaveback_attributions".to_string(),
            serde_json::Value::Array(attributions),
        );
        obj.insert(
            "weaveback_span_attributions".to_string(),
            serde_json::Value::Array(span_attributions.clone()),
        );
        obj.insert(
            "weaveback_source_summary".to_string(),
            build_cargo_attribution_summary(&span_attributions),
        );
    }
    serde_json::to_writer(&mut out, &value)?;
    writeln!(out)?;
    Ok(())
}

pub fn emit_cargo_summary_message(
    compiler_message_count: usize,
    span_attributions: &[serde_json::Value],
    mut out: impl Write,
) -> std::io::Result<()> {
    serde_json::to_writer(
        &mut out,
        &json!({
            "reason": "weaveback-summary",
            "compiler_message_count": compiler_message_count,
            "generated_span_count": span_attributions.len(),
            "weaveback_source_summary": build_cargo_attribution_summary(span_attributions),
        }),
    )?;
    writeln!(out)?;
    Ok(())
}

