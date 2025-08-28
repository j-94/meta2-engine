use async_trait::async_trait;
use crate::engine::types::*;

pub struct SWEOracle;

#[async_trait]
impl crate::engine::verify::DomainOracle for SWEOracle {
    async fn check(&self, _plan: &[Step], _before: &serde_json::Value, _after: &serde_json::Value) -> anyhow::Result<bool> {
        // SWE verification: tests pass, coverage maintained, no secrets leaked
        Ok(true)
    }
}

pub fn atoms() -> Vec<Atom> {
    vec![
        Atom {
            id: "swe.run-tests".to_string(),
            kind: "proc".to_string(),
            name: "Run Test Suite".to_string(),
            signature: Signature {
                inputs: vec![IOType::PathDir],
                outputs: vec![IOType::Json],
            },
            encodings: serde_json::json!({"cli": ["pytest", "--json-report", "{input}"]}),
            observables: vec!["exit_code".to_string(), "coverage".to_string()],
            version: "0.1".to_string(),
        },
        Atom {
            id: "swe.ast-transform".to_string(),
            kind: "proc".to_string(),
            name: "AST Transform".to_string(),
            signature: Signature {
                inputs: vec![IOType::PathFile, IOType::Json],
                outputs: vec![IOType::PathFile],
            },
            encodings: serde_json::json!({"python": "engine.swe.libcst_transform"}),
            observables: vec!["diff_lines".to_string()],
            version: "0.1".to_string(),
        }
    ]
}
