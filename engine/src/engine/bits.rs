use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct Bits {
    pub a: f32,
    pub u: f32,
    pub p: f32,
    pub e: f32,
    #[serde(rename = "d")]
    pub d: f32,
    pub i: f32,
    pub r: f32,
    pub t: f32,
    pub m: f32,
}
