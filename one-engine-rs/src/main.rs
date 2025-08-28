use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod api;
mod engine;
mod routes;

use routes::episodes::{list_episodes, get_trace};

#[derive(OpenApi)]
#[openapi(
    paths(api::run_handler, api::plan_handler, api::health_handler, api::chat_handler),
    components(schemas(
        api::RunRequest,
        api::RunResponse,
        api::PlanRequest,
        api::PlanResponse,
        api::HealthResponse,
        api::ChatRequest,
        api::ChatResponse,
        engine::types::Policy,
        engine::types::Bits,
        engine::types::Manifest
    ))
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    
    let app = Router::new()
        .route("/run", post(api::run_handler))
        .route("/plan", post(api::plan_handler))
        .route("/chat", post(api::chat_handler))
        .route("/healthz", get(api::health_handler))
        .route("/health", get(api::health_handler))
        .route("/episodes", get(list_episodes))
        .route("/trace/:id", get(get_trace))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors);

    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    println!("listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
