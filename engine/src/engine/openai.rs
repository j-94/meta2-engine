use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

pub async fn chat_json(system: &str, user: &str) -> Result<Value> {
    let body = json!({
      "model":"gpt-3.5-turbo",
      "response_format":{"type":"json_object"},
      "messages":[{"role":"system","content":system},{"role":"user","content":user}]
    });
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()?;
    let res = client
        .post(
            std::env::var("OPENAI_API_URL")
                .unwrap_or("https://api.openai.com/v1/chat/completions".into()),
        )
        .bearer_auth(std::env::var("OPENAI_API_KEY")?)
        .json(&body)
        .send()
        .await?;
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    if status != StatusCode::OK {
        eprintln!("[openai] http_error status={} body={}", status, text);
        return Ok(
            json!({"reply":"⟂ openai http_error","error":text,"bits":{"A":0,"U":1,"P":0,"E":1,"Δ":0,"I":0,"R":0,"T":0,"M":0}}),
        );
    }
    // Expect JSON in content; fall back gracefully if plain text
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|_| json!({}));
    let content = v
        .pointer("/choices/0/message/content")
        .and_then(|c| c.as_str())
        .unwrap_or("");
    match serde_json::from_str::<Value>(content) {
        Ok(j) => Ok(j),
        Err(_) => {
            eprintln!("[openai] non-json content='{}'", content);
            Ok(
                json!({"reply":content,"bits":{"A":1,"U":0,"P":0,"E":0,"Δ":0,"I":0,"R":0,"T":1,"M":0}}),
            )
        }
    }
}
