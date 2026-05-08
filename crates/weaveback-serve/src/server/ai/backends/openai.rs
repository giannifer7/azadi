// weaveback-serve/src/server/ai/backends/openai.rs
// I'd Really Rather You Didn't edit this generated file.

use std::io::BufRead;

/// Call an OpenAI-compatible API directly via HTTP.
///
/// Handles standard Chat Completions streaming format.
pub(in crate::server::ai) fn call_openai_api(
    api_key: Option<String>,
    base_url: String,
    model: String,
    system_prompt: String,
    user_content: String,
    tx: std::sync::mpsc::Sender<String>,
) {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_content }
        ],
        "stream": true,
    });

    let mut req = ureq::AgentBuilder::new()
        .build()
        .post(&url)
        .set("Content-Type", "application/json");

    if let Some(key) = api_key {
        req = req.set("Authorization", &format!("Bearer {}", key));
    }

    let resp = match req.send_json(&body) {
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
        if !trimmed.starts_with("data: ") { continue; }
        let data_str = &trimmed[6..];
        if data_str == "[DONE]" { break; }

        let v: serde_json::Value = match serde_json::from_str(data_str) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(text) = v["choices"][0]["delta"]["content"].as_str() {
            let data = serde_json::json!({"t": text}).to_string();
            if tx.send(format!("event: token\ndata: {data}\n\n")).is_err() {
                return;
            }
        }
    }
    let _ = tx.send("event: done\ndata:\n\n".to_string());
}
