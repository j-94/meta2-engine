use crate::engine::{
    self,
    types::{Bits, Manifest, Policy},
    validate,
};
use crate::integrations::{self, AgentGoal, IntegrationReality, UIState};
use crate::{meta, nstar};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    Json,
};
use one_engine::research::{self, ResearchArtifact};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::Infallible;
use tokio::fs;
use tokio::sync::{broadcast, OnceCell};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use utoipa::{OpenApi, ToSchema};

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
        users.insert(
            "demo".to_string(),
            UserContext {
                user_id: "demo".to_string(),
                api_key: "demo-key-123".to_string(),
                quota_remaining: 1000,
                policy_overrides: None,
            },
        );
        users.insert(
            "premium".to_string(),
            UserContext {
                user_id: "premium".to_string(),
                api_key: "premium-key-456".to_string(),
                quota_remaining: 10000,
                policy_overrides: Some(Policy {
                    gamma_gate: 0.3, // Lower threshold for premium
                    time_ms: 60000,  // Longer timeout
                    max_risk: 0.5,   // Higher risk tolerance
                    tiny_diff_loc: 500,
                }),
            },
        );
        Self { users }
    }
}

// Simple progress bus
static PROGRESS_TX: OnceCell<broadcast::Sender<String>> = OnceCell::const_new();

async fn progress_tx() -> broadcast::Sender<String> {
    PROGRESS_TX
        .get_or_init(|| async {
            let (tx, _rx) = broadcast::channel(100);
            tx
        })
        .await
        .clone()
}

fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn authenticate_user(state: &AppState, api_key: &str) -> Option<UserContext> {
    state
        .users
        .values()
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

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ChatReq {
    pub message: String,
    #[serde(default)]
    pub thread: Option<String>,
    #[serde(default)]
    pub policy: Option<Policy>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ChatResp {
    pub run_id: String,
    pub user_id: String,
    pub reply: String,
    pub manifest: Manifest,
    pub bits: Bits,
}

#[utoipa::path(
    get,
    path = "/integrations/reality",
    responses(
        (
            status = 200,
            description = "Reality map describing which integrations are simulated",
            body = [IntegrationReality]
        )
    )
)]
pub async fn integrations_reality_handler() -> impl IntoResponse {
    Json(integrations::integration_reality_matrix())
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
    Json(req): Json<UserRunReq>,
) -> impl IntoResponse {
    // Authenticate
    let api_key = match extract_api_key(&headers) {
        Some(key) => key,
        None => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "Missing x-api-key header".to_string(),
            )
                .into_response()
        }
    };

    let mut user = match authenticate_user(&state, &api_key) {
        Some(user) if user.user_id == user_id => user,
        _ => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid API key or user ID".to_string(),
            )
                .into_response()
        }
    };

    // Check quota
    if user.quota_remaining == 0 {
        return (
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "Quota exceeded".to_string(),
        )
            .into_response();
    }

    // Use user's policy or provided override
    let policy = req
        .policy
        .or(user.policy_overrides.clone())
        .unwrap_or(Policy {
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
            })
            .into_response()
        }
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
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
        None => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "Missing x-api-key header".to_string(),
            )
                .into_response()
        }
    };

    let user = match authenticate_user(&state, &api_key) {
        Some(user) if user.user_id == user_id => user,
        _ => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid API key or user ID".to_string(),
            )
                .into_response()
        }
    };

    Json(UserStatus {
        user_id: user.user_id,
        quota_remaining: user.quota_remaining,
        has_premium_policy: user.policy_overrides.is_some(),
    })
    .into_response()
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
    pub policy: Policy,
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
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct GoldenReq {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct GoldenResp {
    pub name: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub details: Vec<engine::golden::GoldenCase>,
    pub bits: Bits,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ValidationResult {
    pub task: String,
    pub expected_difficulty: f32,
    pub actual_bits: Bits,
    pub score: f32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct VersionInfo {
    pub engine: &'static str,
    pub build_token: Option<&'static str>,
    pub git_ref: Option<&'static str>,
    pub ts: String,
}

impl VersionInfo {
    pub fn current() -> Self {
        let ts = chrono::Utc::now().to_rfc3339();
        Self {
            engine: env!("CARGO_PKG_VERSION"),
            build_token: option_env!("BUILD_TOKEN"),
            git_ref: option_env!("GIT_REF"),
            ts,
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
pub async fn run_handler(
    State(_state): State<AppState>,
    Json(req): Json<RunReq>,
) -> impl IntoResponse {
    match run_with_integrations(&req.goal_id, req.inputs, &req.policy).await {
        Ok((manifest, bits, pr_id, meta2_proposal)) => Json(RunResp {
            manifest,
            bits,
            pr_created: pr_id,
            meta2_proposal,
        })
        .into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
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
pub async fn validate_handler(
    State(_state): State<AppState>,
    Json(req): Json<ValidateReq>,
) -> impl IntoResponse {
    match validate::run_suite(&req.suite).await {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/validate_golden",
    request_body = GoldenReq,
    responses((status = 200, description = "Golden validation", body = GoldenResp))
)]
pub async fn validate_golden_handler(Json(req): Json<GoldenReq>) -> impl IntoResponse {
    match engine::golden::validate_golden(&req.name).await {
        Ok(sum) => Json(GoldenResp {
            name: sum.name,
            total: sum.total,
            passed: sum.passed,
            failed: sum.failed,
            details: sum.details,
            bits: sum.bits,
        })
        .into_response(),
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
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
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
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
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/users/{user_id}/chat",
    request_body = ChatReq,
    responses((status = 200, description = "Chat reply", body = ChatResp))
)]
pub async fn user_chat_handler(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<ChatReq>,
) -> impl IntoResponse {
    // Auth
    let api_key = match extract_api_key(&headers) {
        Some(k) => k,
        None => return (axum::http::StatusCode::UNAUTHORIZED, "Missing x-api-key").into_response(),
    };
    let user = match authenticate_user(&state, &api_key) {
        Some(u) if u.user_id == user_id => u,
        _ => return (axum::http::StatusCode::UNAUTHORIZED, "Invalid user").into_response(),
    };
    let policy = req
        .policy
        .or(user.policy_overrides.clone())
        .unwrap_or(Policy {
            gamma_gate: 0.5,
            time_ms: 30000,
            max_risk: 0.3,
            tiny_diff_loc: 120,
        });

    let run_id = format!("r-{}", uuid::Uuid::new_v4());
    let tx = progress_tx().await;
    let _ = tx.send(format!("{{\"run_id\":\"{}\",\"phase\":\"start\"}}", run_id));

    // Use goal meta.omni
    let inputs = serde_json::json!({"message": req.message});
    match run_with_integrations("meta.omni", inputs, &policy).await {
        Ok((manifest, bits, _pr, _m2)) => {
            let reply = manifest
                .evidence
                .get("reply")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let _ = tx.send(format!("{{\"run_id\":\"{}\",\"phase\":\"done\"}}", run_id));
            Json(ChatResp {
                run_id,
                user_id: user.user_id,
                reply,
                manifest,
                bits,
            })
            .into_response()
        }
        Err(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ProgressQuery {
    pub run_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/progress.sse",
    responses((status = 200, description = "SSE progress stream"))
)]
pub async fn progress_sse_handler(
    Query(q): Query<ProgressQuery>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let rx = progress_tx().await.subscribe();
    let filter_id = q.run_id;
    let stream = BroadcastStream::new(rx).filter_map(move |evt| {
        let filter_id = filter_id.clone();
        match evt {
            Ok(payload) => {
                if let Some(ref expected_id) = filter_id {
                    let matches = serde_json::from_str::<Value>(&payload).ok().and_then(|v| {
                        v.get("run_id")
                            .and_then(|r| r.as_str())
                            .map(|s| s.to_string())
                    });
                    if matches.as_deref() != Some(expected_id.as_str()) {
                        return None;
                    }
                }
                Some(Ok(Event::default().data(payload)))
            }
            Err(_) => None,
        }
    });
    Sse::new(stream)
}

#[utoipa::path(
    get,
    path = "/golden/{name}",
    responses((status = 200, description = "Golden trace JSON"))
)]
pub async fn golden_handler(Path(name): Path<String>) -> impl IntoResponse {
    // basic sanitization: allow [a-zA-Z0-9_\-]
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return (axum::http::StatusCode::BAD_REQUEST, "invalid name").into_response();
    }
    let path = format!("trace/golden/{}.json", name);
    match fs::read_to_string(&path).await {
        Ok(s) => match serde_json::from_str::<serde_json::Value>(&s) {
            Ok(v) => Json(v).into_response(),
            Err(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("invalid JSON: {}", e),
            )
                .into_response(),
        },
        Err(e) => (
            axum::http::StatusCode::NOT_FOUND,
            format!("not found: {}", e),
        )
            .into_response(),
    }
}

async fn run_with_integrations(
    goal_id: &str,
    inputs: serde_json::Value,
    policy: &Policy,
) -> anyhow::Result<(Manifest, Bits, Option<String>, Option<String>)> {
    // 1. Search flywheel for context
    let _context = integrations::flywheel::search(goal_id).await?;

    // 2. Run engine with meta² layer
    let (manifest, ext_bits, meta2_proposal) = engine::run(goal_id, inputs, policy).await?;
    let bits: Bits = ext_bits.into(); // Convert to legacy format

    // 3. Update flywheel metadata
    integrations::flywheel::update_metadata(goal_id, &manifest, bits.t).await?;

    // 4. Create PR if confident and run CI gate
    let pr_id = match integrations::monorepo::create_pr_if_confident(&manifest, &bits).await? {
        Some(pr) => {
            let ci_passed = integrations::monorepo::ci_gate_check(&pr)
                .await
                .unwrap_or(false);
            if !ci_passed {
                tracing::warn!("CI gate failed for {}", pr.id);
                None
            } else {
                Some(pr.id)
            }
        }
        None => None,
    };

    // 5. Track KPI impact heuristically using trust as proxy
    let goal_snapshot = AgentGoal {
        id: goal_id.to_string(),
        kpi_target: "trust".to_string(),
        priority: bits.t,
        estimated_impact: bits.t,
    };
    let _ = integrations::kpi::track_kpi_impact(&goal_snapshot, bits.t).await;

    // 6. Serialize meta² proposal if present
    let meta2_json = meta2_proposal.map(|p| serde_json::to_string(&p).unwrap_or_default());

    Ok((manifest, bits, pr_id, meta2_json))
}

#[derive(OpenApi)]
#[openapi(
    paths(version_handler, run_handler, validate_handler, validate_golden_handler, dashboard_handler, planning_handler, integrations_reality_handler, user_run_handler, user_status_handler, user_chat_handler, progress_sse_handler, golden_handler, research_index_handler, meta::meta_run_handler, meta::meta_state_handler, meta::meta_reset_handler, nstar::nstar_run_handler, nstar::nstar_hud_handler),
    components(schemas(Bits, Policy, Manifest, RunReq, RunResp, VersionInfo, ValidateReq, ValidateResp, GoldenReq, GoldenResp, ValidationResult, UIState, AgentGoal, IntegrationReality, crate::integrations::DataReality, UserRunReq, UserRunResp, UserStatus, ChatReq, ChatResp, nstar::NStarRunReq, nstar::NStarRunResp, meta::MetaRunReq, meta::MetaRunResp, meta::MetaState)),
    tags((name="one-engine", description="Multi-tenant metacognitive system"))
)]
pub struct ApiDoc;

#[utoipa::path(
    get,
    path = "/research/index",
    responses((status = 200, description = "Research artifact index", body = [ResearchArtifact]))
)]
pub async fn research_index_handler() -> impl IntoResponse {
    // Prefer on-disk index if present; else build from current workspace.
    let disk = tokio::fs::read_to_string("research/index.jsonl").await;
    let mut items: Vec<ResearchArtifact> = Vec::new();
    if let Ok(s) = disk {
        for line in s.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(a) = serde_json::from_str::<ResearchArtifact>(line) {
                items.push(a);
            }
        }
    } else {
        // Fallback: build ephemeral index from '.' (no network)
        if let Ok(v) = research::build_index(std::path::Path::new(".")) {
            items = v;
        }
    }
    Json(items)
}
