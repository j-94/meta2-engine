pub mod flywheel;
pub mod kpi;
pub mod monorepo;
pub mod telemetry;
pub mod ui;

use crate::engine::types::{Bits, Manifest};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataReality {
    Real,
    Simulated,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct TelemetryEvent {
    pub ts: String, // ISO 8601 timestamp
    pub component: String,
    pub event_type: String,
    pub run_id: Option<String>,
    pub bits: Option<Bits>,
    pub cost: Option<f32>,
    pub kpi_impact: Option<f32>,
    pub metadata: serde_json::Value,
    pub reality: DataReality,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct AgentGoal {
    pub id: String,
    pub kpi_target: String,
    pub priority: f32,
    pub estimated_impact: f32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct UIState {
    pub search_hits: Vec<SearchResult>,
    pub agent_runs: Vec<Manifest>,
    pub eval_scores: Vec<EvalResult>,
    pub cost_tracking: CostSummary,
    pub kpi_dashboard: KPIDashboard,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub relevance: f32,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct EvalResult {
    pub eval_id: String,
    pub score: f32,
    pub component: String,
    pub timestamp: String, // ISO 8601 timestamp
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct CostSummary {
    pub total_tokens: u64,
    pub total_cost: f32,
    pub cost_per_success: f32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct KPIDashboard {
    pub signal_density: f32,
    pub flow_minutes: f32,
    pub knowledge_yield: f32,
    pub noise_ratio: f32,
    pub weekly_trend: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct IntegrationReality {
    pub component: String,
    pub reality: DataReality,
    pub description: String,
}

pub fn integration_reality_matrix() -> Vec<IntegrationReality> {
    vec![
        IntegrationReality {
            component: "flywheel".to_string(),
            reality: DataReality::Simulated,
            description:
                "Search results are mocked locally; no external embeddings or APIs are queried.".to_string(),
        },
        IntegrationReality {
            component: "monorepo".to_string(),
            reality: DataReality::Simulated,
            description:
                "Pull request IDs and CI checks are generated in-memory for demonstration purposes.".to_string(),
        },
        IntegrationReality {
            component: "kpi".to_string(),
            reality: DataReality::Simulated,
            description:
                "KPI dashboards and planning goals are heuristics derived from hard-coded sample data.".to_string(),
        },
        IntegrationReality {
            component: "ui".to_string(),
            reality: DataReality::Simulated,
            description:
                "Dashboard aggregates reuse recent run state and mock analytics instead of live services.".to_string(),
        },
        IntegrationReality {
            component: "telemetry".to_string(),
            reality: DataReality::Simulated,
            description:
                "Telemetry events are stored in-process and not forwarded to any production pipeline.".to_string(),
        },
    ]
}
