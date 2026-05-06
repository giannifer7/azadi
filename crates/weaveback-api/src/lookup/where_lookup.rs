// weaveback-api/src/lookup/where_lookup.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::lookup::{LookupError, PathResolver, WeavebackDb};
use serde_json::Value;

pub fn perform_where(
    out_file: &str,
    line: u32,
    db: &WeavebackDb,
    resolver: &PathResolver,
) -> Result<Option<Value>, LookupError> {
    use serde_json::json;
    use weaveback_tangle::lookup::find_best_noweb_entry;

    if line == 0 {
        return Err(LookupError::InvalidInput("Line number must be >= 1".to_string()));
    }
    let out_line_0 = line - 1;

    if let Some(entry) = find_best_noweb_entry(db, out_file, out_line_0, resolver)? {
        Ok(Some(json!({
            "out_file": out_file,
            "out_line": line,
            "chunk": entry.chunk_name,
            "expanded_file": entry.src_file,
            "expanded_line": entry.src_line + 1,
            "indent": entry.indent,
            "confidence": entry.confidence.as_str(),
        })))
    } else {
        Ok(None)
    }
}
