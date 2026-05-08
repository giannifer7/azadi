// weaveback-serve/src/server/ai/backends/claude.rs
// I'd Really Rather You Didn't edit this generated file.

use std::io::BufRead;
use std::process::Stdio;

/// Call `claude -p --output-format stream-json --verbose` as a subprocess.
///
/// Uses the existing Claude Code session credentials — no API key needed.
/// The system context is appended to the default Claude Code system prompt via
/// `--append-system-prompt`.  `user_content` is passed as the `-p` argument.
///
/// The `stream-json --verbose` format emits one JSON object per line.
/// We handle two event types:
/// * `type == "assistant"` — message with `message.content[].text` fields;
///   send each text chunk as a token event.
/// * `type == "result"` — final summary; stop reading.
pub(in crate::server::ai) fn call_claude_cli(
    system_prompt: String,
    user_content: String,
    tx: std::sync::mpsc::Sender<String>,
) {
    let mut child = match std::process::Command::new("claude")
        .args([
            "-p", &user_content,
            "--output-format", "stream-json",
            "--verbose",
            "--append-system-prompt", &system_prompt,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let msg = format!(
                "event: error\ndata: {}\n\nevent: done\ndata:\n\n",
                serde_json::json!({"error": format!("cannot spawn claude: {e}")})
            );
            let _ = tx.send(msg);
            return;
        }
    };

    let stdout = child.stdout.take().expect("piped stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        let v: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if v["type"] == "assistant" {
            // Content is an array of blocks; we want text blocks.
            if let Some(content) = v["message"]["content"].as_array() {
                for block in content {
                    if block["type"] == "text"
                        && let Some(text) = block["text"].as_str()
                        && !text.is_empty() {
                        let data = serde_json::json!({"t": text}).to_string();
                        if tx.send(format!("event: token\ndata: {data}\n\n")).is_err() {
                            let _ = child.kill();
                            return;
                        }
                    }
                }
            }
        } else if v["type"] == "result" {
            break;
        }
    }
    let _ = child.wait();
    let _ = tx.send("event: done\ndata:\n\n".to_string());
}
