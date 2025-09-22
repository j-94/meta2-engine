use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;

use crate::engine::openai::chat_json;

pub async fn handle(user_msg: &str) -> Result<Value> {
    let system = fs::read_to_string("prompts/META_OMNI.md").unwrap_or_else(|_| {
        "You are One Engine v0.2. Respond with JSON containing a 'reply' field.".to_string()
    });

    // Try OpenAI, fallback to simple response on error
    let out = match chat_json(&system, user_msg).await {
        Ok(response) => response,
        Err(e) => {
            eprintln!("OpenAI error: {}", e);
            // Fallback response
            let reply = match user_msg.to_lowercase().as_str() {
                msg if msg.contains("who am i") => "I am One Engine v0.2, a metacognitive AI system with self-awareness capabilities.",
                msg if msg.contains("hello") => "Hello! I'm One Engine, ready to assist with metacognitive validation and adaptive control.",
                msg if msg.contains("help") => "I can process tasks with uncertainty tracking, trust calibration, and failure awareness.",
                _ => "I'm processing your request with metacognitive awareness. How can I help you today?"
            };
            json!({
                "reply": reply,
                "bits": {"A":1,"U":0,"P":1,"E":0,"Δ":0,"I":0,"R":0,"T":1,"M":0},
                "intent": {"goal":"chat","constraints":[],"evidence":["fallback response"]},
                "patch": null,
                "explanation": {"assumptions":["OpenAI unavailable"],"evidence":["fallback mode"],"limits":["no LM processing"]}
            })
        }
    };

    // Hard guards: ensure required fields present
    let bits = out
        .get("bits")
        .cloned()
        .unwrap_or(json!({"A":0,"U":1,"P":0,"E":0,"Δ":0,"I":0,"R":0,"T":0,"M":0}));
    let reply = out
        .get("reply")
        .and_then(|v| v.as_str())
        .unwrap_or("⟂ no reply");
    let intent = out
        .get("intent")
        .cloned()
        .unwrap_or(json!({"goal":"chat","constraints":[],"evidence":[]}));
    let patch = out.get("patch").cloned().unwrap_or(Value::Null);
    let explanation = out
        .get("explanation")
        .cloned()
        .unwrap_or(json!({"assumptions":[],"evidence":[],"limits":[]}));

    Ok(json!({
      "intent": intent,
      "bits": bits,
      "patch": patch,
      "explanation": explanation,
      "manifest": { "evidence": { "reply": reply } }
    }))
}
