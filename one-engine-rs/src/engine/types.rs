use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Clone, ToSchema, Default)]
pub struct Bits {
    pub a: f32,  // Action
    pub u: f32,  // Uncertainty
    pub p: f32,  // Policy
    pub e: f32,  // Error
    pub d: f32,  // Diff (Δ)
    pub i: f32,  // Intervention
    pub r: f32,  // Rollback
    pub t: f32,  // Trust
}

impl Bits {
    pub fn init() -> Self {
        Self { a: 1.0, p: 1.0, ..Default::default() }
    }
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Policy {
    pub gamma_gate: f32,
    pub time_ms: u64,
    pub max_risk: f32,
    pub tiny_diff_loc: u32,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Manifest {
    pub run_id: String,
    pub goal_id: String,
    pub deliverables: Vec<String>,
    pub evidence: serde_json::Value,
    pub bits: Bits,
}
