use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

pub async fn call_openai(
    system_prompt: &str,
    user_message: &str,
    model: &str,
) -> anyhow::Result<String> {
    let api_key =
        std::env::var("OPENAI_API_KEY").map_err(|_| anyhow::anyhow!("OPENAI_API_KEY not set"))?;

    let client = reqwest::Client::new();

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_message.to_string(),
            },
        ],
        max_tokens: Some(1000),
        temperature: Some(0.7),
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
    }

    let chat_response: ChatResponse = response.json().await?;

    chat_response
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))
}
