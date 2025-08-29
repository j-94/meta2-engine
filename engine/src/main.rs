mod api;
mod engine;
mod integrations;
mod nstar;
mod meta;

use axum::{Router, routing::{get, post}};
use tracing_subscriber::{fmt, EnvFilter};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(env_filter).init();

    let state = api::AppState::default();
    let openapi = api::ApiDoc::openapi();

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/version", get(api::version_handler))
        .route("/run", post(api::run_handler))
        .route("/validate", post(api::validate_handler))
        .route("/golden/:name", get(api::golden_handler))
        .route("/dashboard", get(api::dashboard_handler))
        .route("/planning", get(api::planning_handler))
        .route("/research/index", get(api::research_index_handler))
        // Multi-tenant user endpoints
        .route("/users/:user_id/run", post(api::user_run_handler))
        .route("/users/:user_id/chat", post(api::user_chat_handler))
        .route("/progress.sse", get(api::progress_sse_handler))
        .route("/users/:user_id/status", get(api::user_status_handler))
        .route("/nstar/run", post(nstar::nstar_run_handler))
        .route("/nstar/hud", get(nstar::nstar_hud_handler))
        .route("/meta/run", post(meta::meta_run_handler))
        .route("/meta/state", get(meta::meta_state_handler))
        .route("/meta/reset", post(meta::meta_reset_handler))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127,0,0,1], 8080));
    tracing::info!("🚀 Integrated One Engine listening on http://{addr}");
    tracing::info!("📊 Dashboard: http://{addr}/dashboard");
    tracing::info!("📋 Planning: http://{addr}/planning");
    tracing::info!("📖 Docs: http://{addr}/swagger-ui");
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
