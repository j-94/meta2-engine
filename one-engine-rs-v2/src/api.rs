use axum::{extract::{State, Path}, Json, response::IntoResponse, http::HeaderMap};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use utoipa::{ToSchema, OpenApi};
use crate::engine::{self, types::{Bits, Policy, Manifest}, validate};
use crate::integrations::{self, UIState, AgentGoal};
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppState {
    pub users: HashMap<String, UserContext>,
}

#[derive(Clone, Debug)]
pub struct UserContext {
    pub user_id: String,
    pub api_key: String,
    pub quota_remaining: u32,
    pub policy_overrides: Option<Policy>,
}

impl Default for AppState {
    fn default() -> Self {
        let mut users = HashMap::new();
        // Demo users
        users.insert("demo".to_string(), UserContext {
            user_id: "demo".to_string(),
            api_key: "demo-key-123".to_string(),
            quota_remaining: 1000,
            policy_overrides: None,
        });
        users.insert("premium".to_string(), UserContext {
            user_id: "premium".to_string(),
            api_key: "premium-key-456".to_string(),
            quota_remaining: 10000,
            policy_overrides: Some(Policy {
                gamma_gate: 0.3, // Lower threshold for premium
                time_ms: 60000,  // Longer timeout
                max_risk: 0.5,   // Higher risk tolerance
                tiny_diff_loc: 500,
            }),
        });
        Self { users }
    }
}

fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    headers.get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn authenticate_user(state: &AppState, api_key: &str) -> Option<UserContext> {
    state.users.values()
        .find(|user| user.api_key == api_key)
        .cloned()
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct UserRunReq {
    pub goal_id: String,
    #[serde(default)]
    pub inputs: serde_json::Value,
    pub policy: Option<Policy>, // User can override default policy
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct UserRunResp {
    pub user_id: String,
    pub quota_remaining: u32,
    pub manifest: Manifest,
    pub bits: Bits,
    pub pr_created: Option<String>,
    pub meta2_proposal: Option<String>,
}

#[utoipa::path(
    post,
    path = "/users/{user_id}/run",
    request_body = UserRunReq,
    responses(
        (status = 200, description = "Run completed", body = UserRunResp),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Quota exceeded")
    )
)]
pub async fn user_run_handler(
    State(mut state): State<AppState>,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UserRunReq>
) -> impl IntoResponse {
    // Authenticate
    let api_key = match extract_api_key(&headers) {
        Some(key) => key,
        None => return (axum::http::StatusCode::UNAUTHORIZED, "Missing x-api-key header".to_string()).into_response()
    };
    
    let mut user = match authenticate_user(&state, &api_key) {
        Some(user) if user.user_id == user_id => user,
        _ => return (axum::http::StatusCode::UNAUTHORIZED, "Invalid API key or user ID".to_string()).into_response()
    };
    
    // Check quota
    if user.quota_remaining == 0 {
        return (axum::http::StatusCode::TOO_MANY_REQUESTS, "Quota exceeded".to_string()).into_response();
    }
    
    // Use user's policy or provided override
    let policy = req.policy
        .or(user.policy_overrides.clone())
        .unwrap_or_else(|| Policy {
            gamma_gate: 0.5,
            time_ms: 30000,
            max_risk: 0.3,
            tiny_diff_loc: 120,
        });
    
    // Namespace goal with user ID to prevent conflicts
    let namespaced_goal = format!("user:{}.{}", user_id, req.goal_id);
    
    match run_with_integrations(&namespaced_goal, req.inputs, &policy).await {
        Ok((manifest, bits, pr_id, meta2_proposal)) => {
            // Decrement quota
            user.quota_remaining -= 1;
            state.users.insert(user_id.clone(), user.clone());
            
            Json(UserRunResp {
                user_id: user.user_id,
                quota_remaining: user.quota_remaining,
                manifest,
                bits,
                pr_created: pr_id,
                meta2_proposal,
            }).into_response()
        },
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response()
    }
}

#[utoipa::path(
    get,
    path = "/users/{user_id}/status",
    responses(
        (status = 200, description = "User status", body = UserStatus)
    )
)]
pub async fn user_status_handler(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let api_key = match extract_api_key(&headers) {
        Some(key) => key,
        None => return (axum::http::StatusCode::UNAUTHORIZED, "Missing x-api-key header".to_string()).into_response()
    };
    
    let user = match authenticate_user(&state, &api_key) {
        Some(user) if user.user_id == user_id => user,
        _ => return (axum::http::StatusCode::UNAUTHORIZED, "Invalid API key or user ID".to_string()).into_response()
    };
    
    Json(UserStatus {
        user_id: user.user_id,
        quota_remaining: user.quota_remaining,
        has_premium_policy: user.policy_overrides.is_some(),
    }).into_response()
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct UserStatus {
    pub user_id: String,
    pub quota_remaining: u32,
    pub has_premium_policy: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct RunReq {
    pub goal_id: String,
    #[serde(default)]
    pub inputs: serde_json::Value,
    pub policy: Policy
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct RunResp {
    pub manifest: Manifest,
    pub bits: Bits,
    pub pr_created: Option<String>,
    pub meta2_proposal: Option<String>, // JSON serialized Meta2Proposal
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ValidateReq {
    pub suite: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ValidateResp {
    pub metacognitive_score: f32,
    pub results: Vec<ValidationResult>,
    pub summary: String
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ValidationResult {
    pub task: String,
    pub expected_difficulty: f32,
    pub actual_bits: Bits,
    pub score: f32
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct VersionInfo {
    pub engine: &'static str,
    pub build_token: Option<&'static str>,
    pub git_ref: Option<&'static str>,
    pub ts: String
}

impl VersionInfo {
    pub fn current() -> Self {
        let ts = chrono::Utc::now().to_rfc3339();
        Self{
            engine: env!("CARGO_PKG_VERSION"),
            build_token: option_env!("BUILD_TOKEN"),
            git_ref: option_env!("GIT_REF"),
            ts
        }
    }
}

#[utoipa::path(
    get,
    path = "/version",
    responses(
        (status = 200, description = "Engine version", body = VersionInfo)
    )
)]
pub async fn version_handler() -> impl IntoResponse {
    Json(VersionInfo::current())
}

#[utoipa::path(
    post,
    path = "/run",
    request_body = RunReq,
    responses(
        (status = 200, description = "Run completed", body = RunResp)
    )
)]
pub async fn run_handler(State(_state): State<AppState>, Json(req): Json<RunReq>) -> impl IntoResponse {
    match run_with_integrations(&req.goal_id, req.inputs, &req.policy).await {
        Ok((manifest, bits, pr_id, meta2_proposal)) => Json(RunResp{ manifest, bits, pr_created: pr_id, meta2_proposal }).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response()
    }
}

#[utoipa::path(
    post,
    path = "/validate",
    request_body = ValidateReq,
    responses(
        (status = 200, description = "Validation completed", body = ValidateResp)
    )
)]
pub async fn validate_handler(State(_state): State<AppState>, Json(req): Json<ValidateReq>) -> impl IntoResponse {
    match validate::run_suite(&req.suite).await {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response()
    }
}

#[utoipa::path(
    get,
    path = "/dashboard",
    responses(
        (status = 200, description = "Unified dashboard state", body = UIState)
    )
)]
pub async fn dashboard_handler() -> impl IntoResponse {
    match integrations::ui::render_unified_state().await {
        Ok(state) => Json(state).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}

#[utoipa::path(
    get,
    path = "/planning",
    responses(
        (status = 200, description = "Weekly planning goals", body = Vec<AgentGoal>)
    )
)]
pub async fn planning_handler() -> impl IntoResponse {
    match integrations::kpi::weekly_planning().await {
        Ok(goals) => Json(goals).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}

async fn run_with_integrations(goal_id: &str, inputs: serde_json::Value, policy: &Policy) -> anyhow::Result<(Manifest, Bits, Option<String>, Option<String>)> {
    // 1. Search flywheel for context
    let _context = integrations::flywheel::search(goal_id).await?;
    
    // 2. Run engine with meta² layer
    let (manifest, ext_bits, meta2_proposal) = engine::run(goal_id, inputs, policy).await?;
    let bits: Bits = ext_bits.into(); // Convert to legacy format
    
    // 3. Update flywheel metadata
    integrations::flywheel::update_metadata(goal_id, &manifest, bits.t).await?;
    
    // 4. Create PR if confident
    let pr = integrations::monorepo::create_pr_if_confident(&manifest, &bits).await?;
    let pr_id = pr.map(|p| p.id);
    
    // 5. Serialize meta² proposal if present
    let meta2_json = meta2_proposal.map(|p| serde_json::to_string(&p).unwrap_or_default());
    
    Ok((manifest, bits, pr_id, meta2_json))
}

#[derive(OpenApi)]
#[openapi(
    paths(version_handler, run_handler, validate_handler, dashboard_handler, planning_handler, user_run_handler, user_status_handler),
    components(schemas(Bits, Policy, Manifest, RunReq, RunResp, VersionInfo, ValidateReq, ValidateResp, ValidationResult, UIState, AgentGoal, UserRunReq, UserRunResp, UserStatus)),
    tags((name="one-engine", description="Multi-tenant metacognitive system"))
)]
pub struct ApiDoc;
