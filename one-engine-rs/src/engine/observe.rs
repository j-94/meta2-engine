use chrono::Utc;
use std::process;

pub fn snapshot() -> serde_json::Value {
    serde_json::json!({
        "ts": Utc::now().to_rfc3339(),
        "pid": process::id(),
        "cwd": std::env::current_dir().unwrap_or_default().to_string_lossy()
    })
}
