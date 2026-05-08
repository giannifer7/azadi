// weaveback-api/src/mcp/run/data.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

pub(super) fn handle_chunk_context<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    input: Option<&serde_json::Map<String, Value>>,
    agent_session: &AgentSession,
) {
    let Some(input) = input else {
        send_error(writer, id, "Missing arguments");
        return;
    };
    let file = input.get("file").and_then(|v| v.as_str()).unwrap_or("");
    let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let nth = input.get("nth").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    if file.is_empty() || name.is_empty() {
        send_error(writer, id, "file and name are required");
        return;
    }
    match agent_session.chunk_context(file, name, nth) {
        Ok(ctx) => {
            let obj = json!({
                "file": ctx.file,
                "name": ctx.name,
                "nth": ctx.nth,
                "body": ctx.body,
                "section_title_chain": ctx.section_breadcrumb,
                "section_prose": ctx.prose,
                "dependencies": ctx.direct_dependencies,
                "output_files": ctx.outputs,
            });
            send_text(writer, id, &serde_json::to_string_pretty(&obj).unwrap());
        }
        Err(_) => send_error(writer, id, &format!("Chunk not found: {}#{}[{}]", file, name, nth)),
    }
}

pub(super) fn handle_list_chunks<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    input: Option<&serde_json::Map<String, Value>>,
    db_path: &std::path::Path,
) {
    let file_filter = input
        .and_then(|i| i.get("file"))
        .and_then(|v| v.as_str());
    if !db_path.exists() {
        send_error(writer, id, "Database not found. Run weaveback on your source files first.");
        return;
    }
    match WeavebackDb::open_read_only(db_path) {
        Err(e) => send_error(writer, id, &format!("Database error: {e:?}")),
        Ok(db) => match db.list_chunk_defs(file_filter) {
            Err(e) => send_error(writer, id, &format!("Query error: {e:?}")),
            Ok(defs) => {
                let arr: Vec<Value> = defs.iter().map(|d| json!({
                    "file":      d.src_file,
                    "name":      d.chunk_name,
                    "nth":       d.nth,
                    "def_start": d.def_start,
                    "def_end":   d.def_end,
                })).collect();
                send_text(writer, id, &serde_json::to_string_pretty(&arr).unwrap());
            }
        },
    }
}

pub(super) fn handle_find_chunk<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    input: Option<&serde_json::Map<String, Value>>,
    db_path: &std::path::Path,
) {
    let Some(input) = input else {
        send_error(writer, id, "Missing arguments");
        return;
    };
    let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("");
    if name.is_empty() {
        send_error(writer, id, "name is required");
        return;
    }
    if !db_path.exists() {
        send_error(writer, id, "Database not found. Run weaveback on your source files first.");
        return;
    }
    match WeavebackDb::open_read_only(db_path) {
        Err(e) => send_error(writer, id, &format!("Database error: {e:?}")),
        Ok(db) => match db.find_chunk_defs_by_name(name) {
            Err(e) => send_error(writer, id, &format!("Query error: {e:?}")),
            Ok(defs) => {
                let arr: Vec<Value> = defs.iter().map(|d| json!({
                    "file":      d.src_file,
                    "nth":       d.nth,
                    "def_start": d.def_start,
                    "def_end":   d.def_end,
                })).collect();
                send_text(writer, id, &serde_json::to_string_pretty(&arr).unwrap());
            }
        },
    }
}

pub(super) fn handle_search<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    input: Option<&serde_json::Map<String, Value>>,
    db_path: &std::path::Path,
    agent_session: &AgentSession,
) {
    let query = input
        .and_then(|v| v.get("query"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if query.is_empty() {
        send_error(writer, id, "query is required");
        return;
    }
    let limit = input
        .and_then(|v| v.get("limit"))
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    if !db_path.exists() {
        send_error(writer, id, "Database not found. Run weaveback on your source files first.");
        return;
    }
    match agent_session.search(query, limit) {
        Err(e) => send_error(writer, id, &format!("Search error: {e}")),
        Ok(results) => {
            let arr: Vec<Value> = results.iter().map(|r| {
                let mut obj = json!({
                    "src_file":   r.src_file,
                    "block_type": r.block_type,
                    "line_start": r.line_start,
                    "line_end":   r.line_end,
                    "snippet":    r.snippet,
                    "score":      r.score,
                    "channels":   r.channels,
                });
                if !r.tags.is_empty() {
                    obj["tags"] = json!(r.tags);
                }
                obj
            }).collect();
            send_text(writer, id, &serde_json::to_string_pretty(&arr).unwrap());
        }
    }
}

pub(super) fn handle_list_tags<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    input: Option<&serde_json::Map<String, Value>>,
    db_path: &std::path::Path,
) {
    let file_filter = input
        .and_then(|v| v.get("file"))
        .and_then(|v| v.as_str());
    if !db_path.exists() {
        send_error(writer, id, "Database not found. Run weaveback on your source files first.");
        return;
    }
    match WeavebackDb::open_read_only(db_path) {
        Err(e) => send_error(writer, id, &format!("Database error: {e:?}")),
        Ok(db) => match db.list_block_tags(file_filter) {
            Err(e) => send_error(writer, id, &format!("Tag list error: {e:?}")),
            Ok(blocks) => {
                let arr: Vec<Value> = blocks.iter().map(|b| json!({
                    "src_file":    b.src_file,
                    "block_index": b.block_index,
                    "block_type":  b.block_type,
                    "line_start":  b.line_start,
                    "tags":        b.tags,
                })).collect();
                send_text(writer, id, &serde_json::to_string_pretty(&arr).unwrap());
            }
        },
    }
}

pub(super) fn handle_coverage<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    input: Option<&serde_json::Map<String, Value>>,
    db_path: &std::path::Path,
    resolver: &PathResolver,
) {
    let lcov_path = input
        .and_then(|v| v.get("lcov_path"))
        .and_then(|v| v.as_str())
        .unwrap_or("lcov.info");
    let path = std::path::Path::new(lcov_path);
    if !path.exists() {
        send_error(writer, id, &format!("lcov file not found at {}", path.display()));
        return;
    }
    if !db_path.exists() {
        send_error(writer, id, "Database not found. Run weaveback on your source files first.");
        return;
    }
    match (std::fs::read_to_string(path), WeavebackDb::open_read_only(db_path)) {
        (Ok(lcov_text), Ok(db)) => {
            let records = crate::coverage::parse_lcov_records(&lcov_text);
            let prj_root = std::env::current_dir().unwrap_or_default();
            let summary = crate::coverage::build_coverage_summary(&records, &db, &prj_root, resolver);
            send_text(writer, id, &serde_json::to_string_pretty(&summary).unwrap());
        }
        (Err(e), _) => send_error(writer, id, &format!("Error reading {lcov_path}: {e}")),
        (_, Err(e)) => send_error(writer, id, &format!("Database error: {e:?}")),
    }
}
