use axum::{routing::get, Json, extract::Path};
use serde_json::Value;
use std::{fs, path::Path as P};

pub async fn list_episodes() -> Json<Value> {
    let mut eps = vec![];
    if let Ok(dir) = fs::read_dir("_episodes") {
        for e in dir.flatten() {
            if e.path().extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(txt) = fs::read_to_string(e.path()) {
                    if let Ok(v) = serde_json::from_str::<Value>(&txt) { eps.push(v); }
                }
            }
        }
    }
    Json(serde_json::json!({ "episodes": eps }))
}

pub async fn get_trace(Path(id): Path<String>) -> Json<Value> {
    // Accept either raw trace id or file name base
    let p = format!("_trace/{}.json", id);
    let fallback = format!("_trace/{}", id);
    let path = if P::new(&p).exists() { p } else { fallback };
    let v: Value = serde_json::from_str(&fs::read_to_string(path).unwrap_or("{}".into())).unwrap_or(serde_json::json!({}));
    Json(v)
}
