// weaveback-lsp/src/client.rs
// I'd Really Rather You Didn't edit this generated file.

use std::process::{Child, ChildStdin, Command, Stdio};
use std::io::{BufRead, BufReader, Write, Read};
use std::path::Path;
use serde_json::{json, Value};
use lsp_types::*;
use url::Url;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LspError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("LSP error: {0}")]
    Protocol(String),
    #[error("Server exited")]
    Exited,
}

use std::collections::HashMap;

pub struct LspClient {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: i64,
    language_id: String,
    diagnostics: HashMap<Url, Vec<Diagnostic>>,
}

impl LspClient {
    pub fn spawn(
        cmd: &str,
        args: &[&str],
        root_dir: &Path,
        language_id: String,
    ) -> Result<Self, LspError> {
        let cmd_parts: Vec<&str> = cmd.split_whitespace().collect();
        if cmd_parts.is_empty() {
             return Err(LspError::Protocol("empty command string".into()));
        }

        let mut child = Command::new(cmd_parts[0])
            .args(&cmd_parts[1..])
            .args(args)
            .current_dir(root_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| LspError::Protocol("failed to open stdin".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| LspError::Protocol("failed to open stdout".into()))?;
        let reader = BufReader::new(stdout);

        Ok(Self {
            child,
            stdin,
            reader,
            next_id: 1,
            language_id,
            diagnostics: HashMap::new(),
        })
    }

    pub fn initialize(&mut self, root_path: &Path) -> Result<(), LspError> {
        let root_uri = Url::from_directory_path(root_path)
            .map_err(|_| LspError::Protocol("invalid root path".into()))?;

        let params = InitializeParams {
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: root_uri,
                name: "root".to_string(),
            }]),
            ..Default::default()
        };

        let res = self.call("initialize", params)?;

        // Basic capability check - ensure the server can actually do what we need.
        if let Some(caps) = res.get("capabilities")
            && caps.get("definitionProvider").is_none() {
            log::warn!("LSP server does not support gotoDefinition");
        }

        self.notify("initialized", json!({}))?;

        // Give the server some time to index.
        std::thread::sleep(std::time::Duration::from_secs(2));

        Ok(())
    }

    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn call<P: serde::Serialize>(&mut self, method: &str, params: P) -> Result<Value, LspError> {
        let id = self.next_id;
        self.next_id += 1;

        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        self.write_request(&req)?;
        self.read_response(id)
    }

    pub fn notify<P: serde::Serialize>(&mut self, method: &str, params: P) -> Result<(), LspError> {
        let req = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        self.write_request(&req)
    }

    pub fn did_open(&mut self, path: &Path) -> Result<(), LspError> {
        let uri = Url::from_file_path(path)
            .map_err(|_| LspError::Protocol("invalid file path".into()))?;
        let text = std::fs::read_to_string(path)?;

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: self.language_id.clone(),
                version: 1,
                text,
            },
        };
        self.notify("textDocument/didOpen", params)
    }

    pub fn get_diagnostics(&self, path: &Path) -> Vec<Diagnostic> {
        let Ok(uri) = Url::from_file_path(path) else { return vec![]; };
        self.diagnostics.get(&uri).cloned().unwrap_or_default()
    }

    fn write_request(&mut self, req: &Value) -> Result<(), LspError> {
        let body = serde_json::to_string(req)?;
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_response(&mut self, expected_id: i64) -> Result<Value, LspError> {
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line)?;
            if line.is_empty() { return Err(LspError::Exited); }

            if let Some(stripped) = line.strip_prefix("Content-Length: ") {
                let len: usize = stripped.trim().parse()
                    .map_err(|_| LspError::Protocol("invalid content-length".into()))?;

                // Skip the \r\n\r\n
                let mut junk = String::new();
                self.reader.read_line(&mut junk)?;

                let mut body = vec![0u8; len];
                self.reader.read_exact(&mut body)?;
                let resp: Value = serde_json::from_slice(&body)?;

                if let Some(id) = resp.get("id")
                    && id.as_i64() == Some(expected_id) {
                    if let Some(error) = resp.get("error") {
                        return Err(LspError::Protocol(error.to_string()));
                    }
                    return Ok(resp.get("result").cloned().unwrap_or(Value::Null));
                }

                // Handle notifications (no ID)
                if resp.get("id").is_none()
                    && let Some(method) = resp.get("method").and_then(|m| m.as_str())
                    && method == "textDocument/publishDiagnostics"
                    && let Ok(params) = serde_json::from_value::<PublishDiagnosticsParams>(resp["params"].clone())
                {
                    self.diagnostics.insert(params.uri, params.diagnostics);
                }
            }
        }
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
