use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::engine::{self, types::*};

#[derive(Deserialize, ToSchema)]
pub struct RunRequest {
    pub goal_id: String,
    pub params: serde_json::Value,
    #[serde(default)]
    pub policy: Option<Policy>,
}

#[derive(Serialize, ToSchema)]
pub struct RunResponse {
    pub manifest: Manifest,
    pub bits: Bits,
    pub steps: Vec<String>,
    pub trace_id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct PlanRequest {
    pub goal_id: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, ToSchema)]
pub struct PlanResponse {
    pub steps: Vec<String>,
    pub est_cost: f32,
    pub bits: Bits,
}

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub ok: bool,
    pub version: String,
    pub pid: u32,
    pub openai_connected: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct ChatResponse {
    pub response: String,
    pub executed: bool,
    pub trace_id: Option<String>,
}

#[utoipa::path(
    post,
    path = "/run",
    request_body = RunRequest,
    responses(
        (status = 200, description = "Engine run completed", body = RunResponse)
    )
)]
pub async fn run_handler(Json(req): Json<RunRequest>) -> Result<Json<RunResponse>, StatusCode> {
    let policy = req.policy.unwrap_or(Policy {
        gamma_gate: 0.5,
        time_ms: 30000,
        max_risk: 0.3,
        tiny_diff_loc: 120,
    });
    
    match engine::run(&req.goal_id, req.params, &policy).await {
        Ok((manifest, bits, steps)) => Ok(Json(RunResponse { 
            trace_id: manifest.run_id.clone(),
            steps,
            manifest, 
            bits 
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/plan",
    request_body = PlanRequest,
    responses(
        (status = 200, description = "Plan generated", body = PlanResponse)
    )
)]
pub async fn plan_handler(Json(req): Json<PlanRequest>) -> Result<Json<PlanResponse>, StatusCode> {
    match engine::plan(&req.goal_id, req.params).await {
        Ok((steps, bits)) => Ok(Json(PlanResponse { 
            steps, 
            est_cost: 0.1, 
            bits 
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Health check", body = HealthResponse)
    )
)]
pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        pid: std::process::id(),
        openai_connected: std::env::var("OPENAI_API_KEY").is_ok(),
    })
}

#[utoipa::path(
    post,
    path = "/chat",
    request_body = ChatRequest,
    responses(
        (status = 200, description = "Chat response", body = ChatResponse)
    )
)]
pub async fn chat_handler(Json(req): Json<ChatRequest>) -> Result<Json<ChatResponse>, StatusCode> {
    // Simple chat interface - convert message to goal and execute
    let goal_id = if req.message.contains("analyze") || req.message.contains("swe") {
        "swe.analyze"
    } else {
        "chat.respond"
    };
    
    let params = serde_json::json!({
        "message": req.message,
        "chat": true
    });
    
    let policy = Policy {
        gamma_gate: 0.5,
        time_ms: 30000,
        max_risk: 0.3,
        tiny_diff_loc: 120,
    };
    
    match engine::run(goal_id, params, &policy).await {
        Ok((manifest, _bits, _steps)) => {
            let response = manifest.evidence
                .get("stdout")
                .and_then(|v| v.as_str())
                .unwrap_or("Task completed")
                .to_string();
            
            Ok(Json(ChatResponse {
                response,
                executed: true,
                trace_id: Some(manifest.run_id),
            }))
        },
        Err(e) => Ok(Json(ChatResponse {
            response: format!("Error: {}", e),
            executed: false,
            trace_id: None,
        }))
    }
}
