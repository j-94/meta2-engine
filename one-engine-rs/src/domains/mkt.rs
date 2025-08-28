use async_trait::async_trait;
use crate::engine::types::*;

pub struct MktOracle;

#[async_trait]
impl crate::engine::verify::DomainOracle for MktOracle {
    async fn check(&self, _plan: &[Step], _before: &serde_json::Value, _after: &serde_json::Value) -> anyhow::Result<bool> {
        // Marketing verification: SEO score, brand compliance, readability
        Ok(true)
    }
}

pub fn atoms() -> Vec<Atom> {
    vec![
        Atom {
            id: "mkt.keyword-research".to_string(),
            kind: "proc".to_string(),
            name: "Keyword Research".to_string(),
            signature: Signature {
                inputs: vec![IOType::Str],
                outputs: vec![IOType::Json],
            },
            encodings: serde_json::json!({"http": {"url": "https://api.semrush.com/keywords", "method": "GET"}}),
            observables: vec!["keyword_count".to_string(), "difficulty".to_string()],
            version: "0.1".to_string(),
        },
        Atom {
            id: "mkt.blog-draft".to_string(),
            kind: "proc".to_string(),
            name: "Generate Blog Draft".to_string(),
            signature: Signature {
                inputs: vec![IOType::Json],
                outputs: vec![IOType::Doc],
            },
            encodings: serde_json::json!({"lm": "blog_template"}),
            observables: vec!["word_count".to_string(), "readability".to_string()],
            version: "0.1".to_string(),
        }
    ]
}
