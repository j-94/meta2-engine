use axum::{Json, response::IntoResponse, response::Html};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use utoipa::ToSchema;
use tokio::{fs, process::Command as TokioCommand};

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct NStarRunReq { pub task: String }

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct NStarRunResp { pub ok: bool, pub result: String, pub policy: serde_json::Value, pub adapt: serde_json::Value }

#[utoipa::path(
    post,
    path = "/nstar/run",
    request_body = NStarRunReq,
    responses((status=200, description="Run nstar loop", body=NStarRunResp))
)]
pub async fn nstar_run_handler(Json(req): Json<NStarRunReq>) -> impl IntoResponse {
    let script = std::env::var("NSTAR_SCRIPT").unwrap_or_else(|_| "scripts/nstar.py".to_string());
    let out = TokioCommand::new("python3")
        .arg(&script)
        .arg(&req.task)
        .output()
        .await;
    match out {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(v) => {
                    let resp = NStarRunResp{
                        ok: v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false),
                        result: v.get("result").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                        policy: v.get("policy").cloned().unwrap_or(serde_json::json!({})),
                        adapt: v.get("adapt").cloned().unwrap_or(serde_json::json!({})),
                    };
                    Json(resp).into_response()
                },
                Err(e) => (axum::http::StatusCode::BAD_REQUEST, format!("nstar invalid json: {}", e)).into_response()
            }
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            (axum::http::StatusCode::BAD_REQUEST, format!("nstar failed: {}", stderr)).into_response()
        },
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("spawn error: {}", e)).into_response()
    }
}

#[utoipa::path(get, path = "/nstar/hud", responses((status=200, description="HTML HUD")))]
pub async fn nstar_hud_handler() -> impl IntoResponse {
    let path = std::env::var("NSTAR_RECEIPTS").unwrap_or_else(|_| "trace/receipts.jsonl".to_string());
    let mut rows: Vec<String> = Vec::new();
    if let Ok(s) = fs::read_to_string(&path).await {
        for line in s.lines().rev().take(100) { rows.push(html_escape_min(line)); }
    }
    let body = format!(r#"<!doctype html><html><head><meta charset='utf-8'><title>N* HUD</title>
    <style>body{{font-family:system-ui, sans-serif}} pre{{background:#111;color:#eee;padding:8px;border-radius:6px;overflow:auto}}</style>
    </head><body><h1>N* Receipts (tail)</h1><p>File: {}</p><pre>{}</pre></body></html>"#, path, rows.join("\n"));
    Html(body)
}

fn html_escape_min(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
