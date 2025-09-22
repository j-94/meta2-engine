use crate::engine::openai::call_openai;
use anyhow::Result;
use serde_json::{json, Value};
use std::fs;

pub async fn handle(message: &str, bits: &crate::engine::bits::Bits) -> Result<Value> {
    // Load composed persona/system prompt
    let sys = fs::read_to_string("prompts/meta_omni.md").unwrap_or_else(|_| {
        "You are One Engine v0.2, a metacognitive AI system. Respond conversationally.".to_string()
    });

    let response = call_openai(&sys, message, "gpt-3.5-turbo").await?;

    Ok(json!({
        "intent": {
            "goal": response,
            "constraints": ["Follow bits-native protocol", "Propose minimal diffs only"],
            "evidence": ["OpenAI chat response", "System prompt loaded"]
        },
        "bits": bits,
        "explanation": {
            "self": format!("effectiveness={:.2}, uncertainty={:.2}", bits.a, bits.u)
        }
    }))
}

// Legacy function for compatibility
pub fn process_meta_prompt(
    _system: &str,
    message: &str,
    _history: &[Value],
    _self_obs: Option<&str>,
) -> String {
    // Simple fallback response
    match message.to_lowercase().as_str() {
        msg if msg.contains("who am i") => "I am One Engine v0.2, a metacognitive AI system with self-awareness capabilities.".to_string(),
        msg if msg.contains("hello") => "Hello! I'm One Engine, ready to assist with metacognitive validation and adaptive control.".to_string(),
        msg if msg.contains("help") => "I can process tasks with uncertainty tracking, trust calibration, and failure awareness. Try asking about my capabilities!".to_string(),
        _ => "I'm processing your request with metacognitive awareness. How can I help you today?".to_string()
    }
}
