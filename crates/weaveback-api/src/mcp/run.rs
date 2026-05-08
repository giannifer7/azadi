// weaveback-api/src/mcp/run.rs
// I'd Really Rather You Didn't edit this generated file.

use crate::apply_back::{self, ApplyBackOptions};
use crate::lookup;
use weaveback_agent_core::{
    ChangePlan, ChangeTarget, PlannedEdit, Session as AgentSession, Workspace as AgentWorkspace,
    WorkspaceConfig as AgentWorkspaceConfig,
};
use weaveback_agent_core::change_plan::OutputAnchor;
use weaveback_macro::evaluator::EvalConfig;
use weaveback_tangle::db::WeavebackDb;
use weaveback_lsp::LspClient;
use weaveback_core::PathResolver;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;

mod data;
pub(crate) mod lsp;

use super::helpers::{send_error, send_response, send_text};
use super::tools::tools_list_result;
use data::{
    handle_chunk_context, handle_coverage, handle_find_chunk, handle_list_chunks,
    handle_list_tags, handle_search,
};
use lsp::{
    handle_lsp_definition, handle_lsp_diagnostics, handle_lsp_hover, handle_lsp_references,
    handle_lsp_symbols,
};

pub fn run_mcp<R: BufRead, W: Write>(
    reader: R,
    mut writer: W,
    db_path: PathBuf,
    gen_dir: PathBuf,
    project_root: PathBuf,
    eval_config: EvalConfig,
) -> Result<(), std::io::Error> {
    let mut lsp_clients: HashMap<String, LspClient> = HashMap::new();
    let agent_workspace = AgentWorkspace::open(AgentWorkspaceConfig {
        project_root: project_root.clone(),
        db_path: db_path.clone(),
        gen_dir: gen_dir.clone(),
    });
    let agent_session = agent_workspace.session();
    let resolver = PathResolver::new(project_root, gen_dir.clone());

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() { continue; }

        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let id = req.get("id").cloned();
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

        match method {
            "initialize" => {
                send_response(&mut writer, id, json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "Weaveback Trace Server", "version": "0.1.0" }
                }));
            }

            "tools/list" => {
                send_response(&mut writer, id, tools_list_result());
            }

            "tools/call" => {
                let params = req.get("params").and_then(|p| p.as_object());
                let tool_name = params.and_then(|p| p.get("name")).and_then(|n| n.as_str());
                let input = params.and_then(|p| p.get("arguments")).and_then(|a| a.as_object());

                match tool_name {
                    Some("weaveback_trace") => {
                        let Some(input) = input else {
                            send_error(&mut writer, id, "Missing arguments");
                            continue;
                        };
                        let out_file = input.get("out_file").and_then(|f| f.as_str()).unwrap_or("");
                        let out_line = input.get("out_line").and_then(|l| l.as_u64()).unwrap_or(0) as u32;
                        let out_col  = input.get("out_col") .and_then(|c| c.as_u64()).unwrap_or(0) as u32;

                        if !db_path.exists() {
                            send_error(&mut writer, id, "Database not found. Run weaveback on your source files first.");
                            continue;
                        }
                        match agent_session.trace(out_file, out_line, out_col) {
                            Ok(Some(res)) => {
                                let mut obj = serde_json::Map::new();
                                obj.insert("out_file".into(), json!(res.out_file));
                                obj.insert("out_line".into(), json!(res.out_line));
                                if let Some(v) = res.src_file { obj.insert("src_file".into(), json!(v)); }
                                if let Some(v) = res.src_line { obj.insert("src_line".into(), json!(v)); }
                                if let Some(v) = res.src_col { obj.insert("src_col".into(), json!(v)); }
                                if let Some(v) = res.kind { obj.insert("kind".into(), json!(v)); }
                                if let Some(v) = res.macro_name { obj.insert("macro_name".into(), json!(v)); }
                                if let Some(v) = res.param_name { obj.insert("param_name".into(), json!(v)); }
                                send_text(&mut writer, id, &serde_json::to_string(&Value::Object(obj)).unwrap())
                            }
                            Ok(None) => send_error(&mut writer, id, &format!("No mapping found for {}:{}", out_file, out_line)),
                            Err(e) => send_error(&mut writer, id, &format!("Lookup error: {e}")),
                        }
                    }

                    Some("weaveback_apply_back") => {
                        let input = input.cloned().unwrap_or_default();
                        let files: Vec<String> = input.get("files")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_default();
                        let dry_run = input.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false);

                        let opts = ApplyBackOptions {
                            db_path: db_path.clone(),
                            gen_dir: gen_dir.clone(),
                            dry_run,
                            files,
                            eval_config: Some(eval_config.clone()),
                        };
                        let mut buf: Vec<u8> = Vec::new();
                        match apply_back::run_apply_back(opts, &mut buf) {
                            Ok(()) => send_text(&mut writer, id, &String::from_utf8_lossy(&buf)),
                            Err(e) => send_error(&mut writer, id, &format!("{:?}", e)),
                        }
                    }

                    Some("weaveback_apply_fix") => {
                        let Some(input) = input else {
                            send_error(&mut writer, id, "Missing arguments");
                            continue;
                        };
                        let src_file   = input.get("src_file")       .and_then(|v| v.as_str()).unwrap_or("");
                        let src_line_1 = input.get("src_line")        .and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let src_line_end_1 = input.get("src_line_end").and_then(|v| v.as_u64())
                            .map(|v| v as usize).unwrap_or(src_line_1);
                        let new_lines: Vec<String> = if let Some(arr) = input.get("new_src_lines").and_then(|v| v.as_array()) {
                            arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()
                        } else {
                            let s = input.get("new_src_line").and_then(|v| v.as_str()).unwrap_or("");
                            vec![s.to_string()]
                        };
                        let out_file   = input.get("out_file")        .and_then(|v| v.as_str()).unwrap_or("");
                        let out_line_1 = input.get("out_line")        .and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        let expected   = input.get("expected_output") .and_then(|v| v.as_str()).unwrap_or("");

                        if src_line_1 == 0 {
                            send_error(&mut writer, id, "src_line must be >= 1");
                            continue;
                        }
                        if src_line_end_1 < src_line_1 {
                            send_error(&mut writer, id, "src_line_end must be >= src_line");
                            continue;
                        }

                        let plan = ChangePlan {
                            plan_id: "mcp-apply-fix".to_string(),
                            goal: "Apply a single oracle-verified fix".to_string(),
                            constraints: Vec::new(),
                            edits: vec![PlannedEdit {
                                edit_id: "edit-1".to_string(),
                                rationale: "MCP weaveback_apply_fix request".to_string(),
                                target: ChangeTarget {
                                    src_file: src_file.to_string(),
                                    src_line: src_line_1,
                                    src_line_end: src_line_end_1,
                                },
                                new_src_lines: new_lines,
                                anchor: OutputAnchor {
                                    out_file: out_file.to_string(),
                                    out_line: out_line_1,
                                    expected_output: expected.to_string(),
                                },
                            }],
                        };
                        match agent_session.apply_change_plan(&plan) {
                            Ok(result) if result.applied => send_text(&mut writer,
                                id,
                                &format!(
                                    "Applied ChangePlan {} with edits: {}",
                                    result.plan_id,
                                    result.applied_edit_ids.join(", ")
                                ),
                            ),
                            Ok(result) => send_error(&mut writer,
                                id,
                                &format!(
                                    "Failed ChangePlan {}. Failed edits: {}",
                                    result.plan_id,
                                    result.failed_edit_ids.join(", ")
                                ),
                            ),
                            Err(e)  => send_error(&mut writer, id, &e),
                        }
                    }

                    Some("weaveback_chunk_context") => {
                        handle_chunk_context(&mut writer, id, input, &agent_session);
                    }

                    Some("weaveback_list_chunks") => {
                        handle_list_chunks(&mut writer, id, input, &db_path);
                    }

                    Some("weaveback_find_chunk") => {
                        handle_find_chunk(&mut writer, id, input, &db_path);
                    }

                    Some("weaveback_lsp_definition") => {
                        handle_lsp_definition(
                            &mut writer,
                            id,
                            input,
                            &mut lsp_clients,
                            &db_path,
                            &resolver,
                            &eval_config,
                        );
                    }

                    Some("weaveback_lsp_references") => {
                        handle_lsp_references(
                            &mut writer,
                            id,
                            input,
                            &mut lsp_clients,
                            &db_path,
                            &resolver,
                            &eval_config,
                        );
                    }

                    Some("weaveback_lsp_hover") => {
                        handle_lsp_hover(
                            &mut writer,
                            id,
                            input,
                            &mut lsp_clients,
                            &db_path,
                            &resolver,
                            &eval_config,
                        );
                    }

                    Some("weaveback_lsp_diagnostics") => {
                        handle_lsp_diagnostics(
                            &mut writer,
                            id,
                            input,
                            &mut lsp_clients,
                            &db_path,
                            &resolver,
                            &eval_config,
                        );
                    }

                    Some("weaveback_lsp_symbols") => {
                        handle_lsp_symbols(&mut writer, id, input, &mut lsp_clients);
                    }

                    Some("weaveback_search") => {
                        handle_search(&mut writer, id, input, &db_path, &agent_session);
                    }

                    Some("weaveback_list_tags") => {
                        handle_list_tags(&mut writer, id, input, &db_path);
                    }

                    Some("weaveback_coverage") => {
                        handle_coverage(&mut writer, id, input, &db_path, &resolver);
                    }

                    other => send_error(&mut writer, id, &format!("Unknown tool: {:?}", other)),
                }
            }

            "resources/list" => {
                send_response(&mut writer, id, json!({ "resources": [] }));
            }
            "prompts/list" => {
                send_response(&mut writer, id, json!({ "prompts": [] }));
            }
            "notifications/initialized" => {}
            _ => {}
        }
    }
    Ok(())
}
