// weaveback-api/src/mcp/run/lsp.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

pub(crate) fn get_or_spawn_lsp<'a>(
    clients: &'a mut HashMap<String, LspClient>,
    ext: &str,
) -> Result<&'a mut LspClient, String> {
    let (lsp_cmd, lsp_lang) = weaveback_lsp::get_lsp_config(ext)
        .ok_or_else(|| format!("unsupported file extension: .{}", ext))?;

    let needs_spawn = match clients.get_mut(&lsp_lang) {
        Some(c) => !c.is_alive(),
        None => true,
    };

    if needs_spawn {
        let project_root = std::env::current_dir().map_err(|e| e.to_string())?;
        let mut c = LspClient::spawn(&lsp_cmd, &[], &project_root, lsp_lang.clone())
            .map_err(|e| format!("failed to spawn LSP '{}': {e}", lsp_cmd))?;
        c.initialize(&project_root)
            .map_err(|e| format!("failed to initialize LSP '{}': {e}", lsp_cmd))?;
        clients.insert(lsp_lang.clone(), c);
    }
    Ok(clients.get_mut(&lsp_lang).unwrap())
}
fn lsp_position_args(input: &serde_json::Map<String, Value>) -> Result<(&str, u32, u32), &'static str> {
    let out_file = input.get("out_file").and_then(|v| v.as_str()).unwrap_or("");
    let line = input.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let col = input.get("col").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    if out_file.is_empty() || line == 0 || col == 0 {
        Err("out_file, line, and col are required and must be > 0")
    } else {
        Ok((out_file, line, col))
    }
}

fn lsp_client_for_file<'a, W: Write>(
    writer: &mut W,
    id: Option<Value>,
    lsp_clients: &'a mut HashMap<String, LspClient>,
    out_file: &str,
) -> Option<&'a mut LspClient> {
    let ext = std::path::Path::new(out_file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match get_or_spawn_lsp(lsp_clients, ext) {
        Ok(client) => Some(client),
        Err(e) => {
            send_error(writer, id, &format!("LSP error: {e}"));
            None
        }
    }
}

fn open_lsp_mapping_db<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    db_path: &std::path::Path,
) -> Option<WeavebackDb> {
    if !db_path.exists() {
        send_error(writer, id, "Database not found");
        return None;
    }
    match WeavebackDb::open_read_only(db_path) {
        Ok(db) => Some(db),
        Err(e) => {
            send_error(writer, id, &format!("Database error: {e:?}"));
            None
        }
    }
}

pub(super) fn handle_lsp_definition<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    input: Option<&serde_json::Map<String, Value>>,
    lsp_clients: &mut HashMap<String, LspClient>,
    db_path: &std::path::Path,
    resolver: &PathResolver,
    eval_config: &EvalConfig,
) {
    let Some(input) = input else {
        send_error(writer, id, "Missing arguments");
        return;
    };
    let (out_file, line, col) = match lsp_position_args(input) {
        Ok(args) => args,
        Err(msg) => {
            send_error(writer, id, msg);
            return;
        }
    };
    let Some(client) = lsp_client_for_file(writer, id.clone(), lsp_clients, out_file) else {
        return;
    };

    match client.goto_definition(std::path::Path::new(out_file), line - 1, col - 1) {
        Ok(Some(loc)) => {
            if let Ok(target_path) = loc.uri.to_file_path() {
                let Some(db) = open_lsp_mapping_db(writer, id.clone(), db_path) else {
                    return;
                };
                match lookup::perform_trace(
                    target_path.to_string_lossy().as_ref(),
                    loc.range.start.line + 1,
                    loc.range.start.character + 1,
                    &db,
                    resolver,
                    eval_config.clone(),
                ) {
                    Ok(Some(res)) => {
                        send_text(writer, id, &serde_json::to_string_pretty(&res).unwrap())
                    }
                    Ok(None) => send_text(
                        writer,
                        id,
                        &serde_json::to_string_pretty(&json!({
                            "out_file": target_path.to_string_lossy(),
                            "out_line": loc.range.start.line + 1,
                            "out_col":  loc.range.start.character + 1,
                            "note": "LSP result could not be mapped to source"
                        }))
                        .unwrap(),
                    ),
                    Err(e) => send_error(writer, id, &format!("Mapping error: {e:?}")),
                }
            } else {
                send_error(writer, id, "LSP returned non-file URI");
            }
        }
        Ok(None) => send_text(writer, id, "No definition found."),
        Err(e) => send_error(writer, id, &format!("LSP call failed: {e}")),
    }
}

pub(super) fn handle_lsp_references<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    input: Option<&serde_json::Map<String, Value>>,
    lsp_clients: &mut HashMap<String, LspClient>,
    db_path: &std::path::Path,
    resolver: &PathResolver,
    eval_config: &EvalConfig,
) {
    let Some(input) = input else {
        send_error(writer, id, "Missing arguments");
        return;
    };
    let (out_file, line, col) = match lsp_position_args(input) {
        Ok(args) => args,
        Err(msg) => {
            send_error(writer, id, msg);
            return;
        }
    };
    let Some(client) = lsp_client_for_file(writer, id.clone(), lsp_clients, out_file) else {
        return;
    };

    match client.find_references(std::path::Path::new(out_file), line - 1, col - 1) {
        Ok(locs) => {
            let Some(db) = open_lsp_mapping_db(writer, id.clone(), db_path) else {
                return;
            };
            let mut results = Vec::new();
            for loc in locs {
                if let Ok(target_path) = loc.uri.to_file_path() {
                    match lookup::perform_trace(
                        target_path.to_string_lossy().as_ref(),
                        loc.range.start.line + 1,
                        loc.range.start.character + 1,
                        &db,
                        resolver,
                        eval_config.clone(),
                    ) {
                        Ok(Some(res)) => results.push(res),
                        _ => results.push(json!({
                            "out_file": target_path.to_string_lossy(),
                            "out_line": loc.range.start.line + 1,
                            "out_col":  loc.range.start.character + 1,
                            "note": "LSP result could not be mapped to source"
                        })),
                    }
                }
            }
            send_text(writer, id, &serde_json::to_string_pretty(&results).unwrap());
        }
        Err(e) => send_error(writer, id, &format!("LSP call failed: {e}")),
    }
}

pub(super) fn handle_lsp_hover<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    input: Option<&serde_json::Map<String, Value>>,
    lsp_clients: &mut HashMap<String, LspClient>,
    db_path: &std::path::Path,
    resolver: &PathResolver,
    eval_config: &EvalConfig,
) {
    let Some(input) = input else {
        send_error(writer, id, "Missing arguments");
        return;
    };
    let (out_file, line, col) = match lsp_position_args(input) {
        Ok(args) => args,
        Err(msg) => {
            send_error(writer, id, msg);
            return;
        }
    };
    let Some(client) = lsp_client_for_file(writer, id.clone(), lsp_clients, out_file) else {
        return;
    };

    match client.hover(std::path::Path::new(out_file), line - 1, col - 1) {
        Ok(Some(hover)) => {
            let Some(db) = open_lsp_mapping_db(writer, id.clone(), db_path) else {
                return;
            };
            let trace = lookup::perform_trace(out_file, line, col, &db, resolver, eval_config.clone())
                .ok()
                .flatten();

            let mut res = json!({ "hover": hover });
            if let Some(t) = trace {
                res.as_object_mut().unwrap().insert("source".into(), t);
            }
            send_text(writer, id, &serde_json::to_string_pretty(&res).unwrap());
        }
        Ok(None) => send_text(writer, id, "No hover info found."),
        Err(e) => send_error(writer, id, &format!("LSP call failed: {e}")),
    }
}

pub(super) fn handle_lsp_diagnostics<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    input: Option<&serde_json::Map<String, Value>>,
    lsp_clients: &mut HashMap<String, LspClient>,
    db_path: &std::path::Path,
    resolver: &PathResolver,
    eval_config: &EvalConfig,
) {
    let Some(input) = input else {
        send_error(writer, id, "Missing arguments");
        return;
    };
    let out_file = input.get("out_file").and_then(|v| v.as_str()).unwrap_or("");
    if out_file.is_empty() {
        send_error(writer, id, "out_file is required");
        return;
    }
    let Some(client) = lsp_client_for_file(writer, id.clone(), lsp_clients, out_file) else {
        return;
    };

    let diags = client.get_diagnostics(std::path::Path::new(out_file));
    let Some(db) = open_lsp_mapping_db(writer, id.clone(), db_path) else {
        return;
    };
    let mut mapped = Vec::new();
    for d in diags {
        let line = d.range.start.line + 1;
        let col = d.range.start.character + 1;
        let trace = lookup::perform_trace(out_file, line, col, &db, resolver, eval_config.clone())
            .ok()
            .flatten();
        mapped.push(json!({
            "diagnostic": d,
            "source": trace,
        }));
    }
    send_text(writer, id, &serde_json::to_string_pretty(&mapped).unwrap());
}

pub(super) fn handle_lsp_symbols<W: Write>(
    writer: &mut W,
    id: Option<Value>,
    input: Option<&serde_json::Map<String, Value>>,
    lsp_clients: &mut HashMap<String, LspClient>,
) {
    let Some(input) = input else {
        send_error(writer, id, "Missing arguments");
        return;
    };
    let out_file = input.get("out_file").and_then(|v| v.as_str()).unwrap_or("");
    if out_file.is_empty() {
        send_error(writer, id, "out_file is required");
        return;
    }
    let Some(client) = lsp_client_for_file(writer, id.clone(), lsp_clients, out_file) else {
        return;
    };

    match client.document_symbols(std::path::Path::new(out_file)) {
        Ok(symbols) => send_text(writer, id, &serde_json::to_string_pretty(&symbols).unwrap()),
        Err(e) => send_error(writer, id, &format!("LSP call failed: {e}")),
    }
}
