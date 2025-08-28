use serde_json::Value;
use super::types::Policy;
use super::lm::{Lm, StubLm, OpenAiLm, PlanCand};

pub async fn select_plan(goal: &str, inputs: &Value, policy: &Policy) -> anyhow::Result<PlanCand> {
    let lm: Box<dyn Lm> = if std::env::var("OPENAI_API_KEY").is_ok() {
        match OpenAiLm::new() {
            Ok(openai_lm) => Box::new(openai_lm),
            Err(_) => Box::new(StubLm),
        }
    } else {
        Box::new(StubLm)
    };
    
    let cand = lm.plan(goal, inputs).await?;
    // γ-gate: abstain if uncertainty too high
    if cand.uncertainty > policy.gamma_gate {
        anyhow::bail!("gamma-gate: uncertainty {} > {}", cand.uncertainty, policy.gamma_gate);
    }
    Ok(cand)
}
