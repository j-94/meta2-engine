use serde_json::{json, Value};

pub fn process_meta_prompt(system: &str, message: &str, history: &[Value], self_obs: Option<&str>) -> String {
    let context_summary = if history.is_empty() {
        "No previous context".to_string()
    } else {
        format!("Previous {} interactions", history.len())
    };
    
    let mut evidence = vec![context_summary];
    if let Some(obs) = self_obs {
        evidence.push(format!("Self-observation: {}", obs));
    }
    
    let response = json!({
        "intent": {
            "goal": message,
            "constraints": ["Follow bits-native protocol", "Propose minimal diffs only"],
            "evidence": evidence
        },
        "bits": {
            "A": 1, "U": 0, "P": 1, "E": 0, "Δ": 0, "I": 0, "R": 0, 
            "T": if self_obs.is_some() { 1 } else { 0 }, "M": 0
        },
        "patch": {
            "files": [],
            "post_checks": ["Validate against schema", "Run tests"]
        },
        "explanation": {
            "assumptions": ["User request is well-formed", "System has necessary permissions"],
            "evidence": ["META_OMNI.md system prompt", "User message context"],
            "limits": ["Cannot perform side effects", "Must respect L2/L3 gates"],
            "self": self_obs.unwrap_or("No self-observation available")
        },
        "questions": []
    });
    
    serde_json::to_string_pretty(&response).unwrap_or_else(|_| message.to_string())
}
