// weaveback-serve/src/server/ai/backends/anthropic.rs
// I'd Really Rather You Didn't edit this generated file.

use std::io::BufRead;

/// Call the Anthropic Messages API directly.
///
/// Requires `ANTHROPIC_API_KEY`.  Parses the native Anthropic SSE stream
/// (`content_block_delta` events) and forwards text deltas to the channel.
pub(in crate::server::ai) fn call_anthropic_api(
    api_key: String,
    api_body: serde_json::Value,
    tx: std::sync::mpsc::Sender<String>,
) {
    let resp = match ureq::AgentBuilder::new()
        .build()
        .post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", &api_key)
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json")
        .send_json(&api_body)
    {
        Ok(r) => r,
        Err(e) => {
            let msg = format!(
                "event: error\ndata: {}\n\nevent: done\ndata:\n\n",
                serde_json::json!({"error": format!("{e}")})
            );
            let _ = tx.send(msg);
            return;
        }
    };

    let mut reader = std::io::BufReader::new(resp.into_reader());
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
        if !trimmed.starts_with("data: ") { continue; }
        let json_str = &trimmed["data: ".len()..];
        if json_str == "[DONE]" { break; }
        let v: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if v["type"] == "content_block_delta"
            && let Some(text) = v["delta"]["text"].as_str()
            && !text.is_empty() {
            let data = serde_json::json!({"t": text}).to_string();
            if tx.send(format!("event: token\ndata: {data}\n\n")).is_err() {
                return;
            }
        } else if v["type"] == "message_stop" {
            break;
        }
    }
    let _ = tx.send("event: done\ndata:\n\n".to_string());
}
