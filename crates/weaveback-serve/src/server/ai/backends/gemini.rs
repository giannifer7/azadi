// weaveback-serve/src/server/ai/backends/gemini.rs
// I'd Really Rather You Didn't edit this generated file.

use std::io::BufRead;

/// Call the Google Gemini API directly via HTTP.
///
/// Requires `GOOGLE_API_KEY`. Uses the `streamGenerateContent` endpoint.
pub(in crate::server::ai) fn call_gemini_api(
    api_key: String,
    model: String,
    system_prompt: String,
    user_content: String,
    tx: std::sync::mpsc::Sender<String>,
) {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}",
        model, api_key
    );

    let body = serde_json::json!({
        "contents": [
            {
                "role": "user",
                "parts": [{ "text": format!("System: {}\n\n{}", system_prompt, user_content) }]
            }
        ],
        "generationConfig": {
            "maxOutputTokens": 1024,
        }
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

        // Gemini stream format is a JSON array of objects, but delivered as individual
        // chunks. Sometimes it starts with '[' and ends with ']'.
        let clean = trimmed.trim_start_matches(',').trim_start_matches('[').trim_end_matches(']');
        if clean.is_empty() { continue; }

        let v: serde_json::Value = match serde_json::from_str(clean) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(text) = v["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            let data = serde_json::json!({"t": text}).to_string();
            if tx.send(format!("event: token\ndata: {data}\n\n")).is_err() {
                return;
            }
        }
    }
    let _ = tx.send("event: done\ndata:\n\n".to_string());
}
