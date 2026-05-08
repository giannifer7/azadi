// weaveback-serve/src/server/ai/backends/ollama.rs
// I'd Really Rather You Didn't edit this generated file.

use std::io::BufRead;

/// Call a local Ollama API via HTTP.
///
/// Uses the `/api/chat` endpoint with `stream: true`.
pub(in crate::server::ai) fn call_ollama_api(
    base_url: String,
    model: String,
    system_prompt: String,
    user_content: String,
    tx: std::sync::mpsc::Sender<String>,
) {
    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_content }
        ],
        "stream": true,
    });

    let resp = match ureq::AgentBuilder::new()
        .build()
        .post(&url)
        .set("Content-Type", "application/json")
        .send_json(&body)
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
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        let v: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(text) = v["message"]["content"].as_str() {
            let data = serde_json::json!({"t": text}).to_string();
            if tx.send(format!("event: token\ndata: {data}\n\n")).is_err() {
                return;
            }
        }
        if v["done"].as_bool().unwrap_or(false) {
            break;
        }
    }
    let _ = tx.send("event: done\ndata:\n\n".to_string());
}
