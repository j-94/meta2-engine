pub mod types;
pub mod policy;
pub mod observe;
mod lm;
mod planner;
mod executor;
mod verify;
mod trace;

use types::{Bits, Manifest, Policy};
use lm::Lm;

pub async fn plan(goal_id: &str, params: serde_json::Value) -> anyhow::Result<(Vec<String>, Bits)> {
    let policy = Policy {
        gamma_gate: 0.5,
        time_ms: 30000,
        max_risk: 0.3,
        tiny_diff_loc: 120,
    };
    
    let cand = planner::select_plan(goal_id, &params, &policy).await?;
    let bits = Bits::init();
    Ok((cand.steps, bits))
}

pub async fn run(goal_id: &str, params: serde_json::Value, policy: &Policy) -> anyhow::Result<(Manifest, Bits, Vec<String>)> {
    let mut bits = Bits::init();
    let before = observe::snapshot();

    // 1) PLAN with LM (γ-gate handled inside planner)
    let cand = planner::select_plan(goal_id, &params, policy).await?;

    // 2) CRITIC (pre-act risk check)
    let lm = lm::StubLm;
    let risk = lm.critic_pre(&cand.steps[0], &params).await?;
    if risk > policy.max_risk {
        bits.u = risk; bits.i = 1.0;
        anyhow::bail!("risk-gate: risk {} > {}", risk, policy.max_risk);
    }

    // 3) EXECUTE - handle override_cmds or goal-based execution
    let (results, deliverables) = if let Some(override_cmds) = params.get("override_cmds") {
        // Use override commands if provided
        let cmds: Vec<String> = override_cmds.as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.as_str().unwrap_or("echo 'invalid cmd'").to_string())
            .collect();
        let results = executor::execute_parallel(cmds).await;
        (results, vec![])
    } else if goal_id.starts_with("swe.") && cand.steps.len() > 1 {
        // Parallel CLI execution for SWE goals
        let results = executor::execute_parallel(cand.steps.clone()).await;
        (results, vec![])
    } else {
        // Single file write for demo goals
        let msg = params.get("text").or_else(|| params.get("message"))
            .and_then(|v| v.as_str()).unwrap_or("hello from one-engine");
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("out/hello.txt");
        let action = executor::Action::WriteFile { 
            path: path.to_string(), 
            content: format!("{}\n", msg) 
        };
        let res = executor::execute(action).await?;
        let deliverables = res.artifact.iter().cloned().collect::<Vec<_>>();
        (vec![res], deliverables)
    };

    // Check for errors/drift
    for res in &results {
        if res.drift { bits.d = 1.0; }
        if !res.ok { bits.e = 1.0; bits.r = 1.0; }
    }

    // 4) VERIFY
    let after = observe::snapshot();
    let passed = results.iter().all(|r| r.ok);
    bits.t = policy::trust_from(passed, &bits);

    // 5) MANIFEST + TRACE
    let stdout = results.iter().map(|r| &r.stdout).cloned().collect::<Vec<_>>().join("\n");
    let manifest = types::Manifest{
        run_id: format!("r-{}", uuid::Uuid::new_v4()),
        goal_id: goal_id.to_string(),
        deliverables,
        evidence: serde_json::json!({"before": before, "after": after, "stdout": stdout}),
        bits: bits.clone()
    };
    trace::append_jsonl(&serde_json::json!({
        "ts": chrono::Utc::now().to_rfc3339(),
        "goal": goal_id,
        "run_id": manifest.run_id,
        "steps": cand.steps,
        "bits": manifest.bits,
        "deliverables": manifest.deliverables
    })).await?;

    Ok((manifest, bits, cand.steps))
}
