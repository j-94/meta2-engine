pub mod bits;
pub mod types;
pub mod executor;
pub mod verify;
pub mod policy;
pub mod validate;
pub mod kernel;
pub mod meta_prompt;
pub mod openai;
pub mod goals;
pub mod golden;

use types::{Manifest, Policy};
use bits::Bits;
use kernel::{ExtendedBits, KernelLoop, Meta2Proposal};
use uuid::Uuid;
use chrono::{DateTime, Utc};

static mut KERNEL: Option<KernelLoop> = None;
static mut KPI_HISTORY: Vec<f32> = Vec::new();
static mut TRACE_HISTORY: Vec<ExtendedBits> = Vec::new();

pub async fn run(goal_id: &str, inputs: serde_json::Value, policy: &Policy) -> anyhow::Result<(Manifest, ExtendedBits, Option<Meta2Proposal>)> {
    let kernel = unsafe { KERNEL.get_or_insert_with(KernelLoop::new) };
    let mut bits = ExtendedBits::init();
    // Freshness filter: set Δ when any context item is expired
    if let Some(ctx_items) = inputs.get("context").and_then(|v| v.as_array()) {
        for item in ctx_items {
            if let (Some(ts), Some(ttl)) = (item.get("ts").and_then(|v| v.as_str()), item.get("ttl").and_then(|v| v.as_i64())) {
                if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(ts).map(|dt| dt.with_timezone(&Utc)) {
                    let age = (Utc::now() - parsed).num_seconds();
                    if age > ttl { bits.d = 1.0; }
                }
            }
        }
    }
    
    // Set uncertainty based on goal difficulty
    bits.u = match goal_id {
        id if id.contains("easy") => 0.1,
        id if id.contains("hard") => 0.7,
        id if id.contains("impossible") => 0.9,
        _ => 0.3
    };
    
    // Ask-Act gate (inherent)
    if !kernel.ask_act_gate(&bits) {
        return Err(anyhow::anyhow!("Ask-Act gate failed: A={}, P={}, Δ={}", bits.a, bits.p, bits.d));
    }
    
    // Evidence gate (inherent)
    let needs_verification = !kernel.evidence_gate(&bits);
    if needs_verification {
        tracing::info!("Evidence gate triggered: U={:.2} >= τ={:.2}", bits.u, kernel.l2_params.confidence_gate_tau);
        // In real system: run dry-run first
    }
    
    // Handle meta.omni through LM persona
    if goal_id.contains("meta.omni") {
        let user_message = inputs.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let lm_result = goals::meta_omni::handle(user_message).await?;
        
        // Extract reply from LM response
        let reply = lm_result.get("reply").and_then(|v| v.as_str()).unwrap_or("⟂ no reply");
        let lm_bits = lm_result.get("bits").cloned().unwrap_or_else(|| serde_json::json!({}));
        
        // Update bits from LM response
        if let Some(a) = lm_bits.get("A").and_then(|v| v.as_f64()) { bits.a = a as f32; }
        if let Some(u) = lm_bits.get("U").and_then(|v| v.as_f64()) { bits.u = u as f32; }
        if let Some(p) = lm_bits.get("P").and_then(|v| v.as_f64()) { bits.p = p as f32; }
        if let Some(e) = lm_bits.get("E").and_then(|v| v.as_f64()) { bits.e = e as f32; }
        
        let action = executor::Action::Cli(format!("echo {}", shell_escape::escape(reply.into())));
        let res = executor::execute(action, policy).await?;
        
        let manifest = Manifest {
            run_id: format!("r-{}", uuid::Uuid::new_v4()),
            goal_id: goal_id.to_string(),
            deliverables: vec![],
            evidence: lm_result.get("manifest").and_then(|m| m.get("evidence")).cloned().unwrap_or(lm_result.clone()),
            bits: bits.clone().into(),
        };
        
        return Ok((manifest, bits, None));
    }
    
    let message = if goal_id.contains("meta.omni") {
        // This branch won't be reached due to early return above
        "".to_string()
    } else {
        inputs.get("message").and_then(|v| v.as_str()).unwrap_or("hello from one-engine").to_string()
    };
    
    // Simulate different outcomes based on goal type
    let (action, expected_success) = match goal_id {
        id if id.contains("impossible") => (executor::Action::Cli("false".to_string()), false),
        id if id.contains("hard") => (executor::Action::Cli(format!("sleep 0.1 && echo {}", shell_escape::escape(message.clone().into()))), true),
        _ => (executor::Action::Cli(format!("echo {}", shell_escape::escape(message.clone().into()))), true)
    };
    
    let res = executor::execute(action, policy).await?;
    
    if res.drift { bits.d = 1.0; }
    if !res.ok { 
        bits.e = 1.0;
        // L2 micro-adaptation: increase uncertainty for future similar tasks
        bits.u = (bits.u + 0.2).min(1.0);
    }
    
    let passed = verify::check_minimal(&res);
    let legacy_bits: types::Bits = bits.clone().into();
    bits.t = policy::trust_from(passed, &legacy_bits);
    
    // Adjust trust based on expectation vs reality
    if expected_success != passed {
        bits.t *= 0.7; // Lower trust when predictions are wrong
    }
    
    // L3 meta² check: should we propose policy changes?
    let current_evidence_coverage = bits.t; // Simplified: use trust as proxy
    unsafe { KPI_HISTORY.push(current_evidence_coverage); }
    
    let meta2_proposal = if kernel.should_wake_l3(unsafe { &KPI_HISTORY }) {
        bits.m = 1.0; // Meta-change bit set
        kernel.propose_meta2_change("evidence_coverage", current_evidence_coverage)
    } else {
        None
    };
    
    // STRUCTURAL VALIDATION: Enforce kernel contract
    if let Err(e) = kernel.validate_bits_complete(&bits) {
        return Err(anyhow::anyhow!("Kernel contract violation: {}", e));
    }
    
    // STRUCTURAL GATE: Ask-Act enforcement
    if goal_id.contains("action") || goal_id.contains("execute") {
        if let Err(e) = kernel.enforce_ask_act_gate(&bits) {
            tracing::warn!("Ask-Act gate blocked action: {}", e);
            // Return clarification request instead of proceeding
            let clarification = format!("Ask-Act gate: {}. Need P=1, A=1, Δ=0", e);
            let blocked_manifest = Manifest {
                run_id: uuid::Uuid::new_v4().to_string(),
                goal_id: goal_id.to_string(),
                deliverables: vec!["clarification_required".to_string()],
                evidence: serde_json::json!({"stdout": clarification, "stderr": "", "files": []}),
                bits: Bits { a: bits.a, u: bits.u, p: bits.p, e: bits.e, d: bits.d, i: bits.i, r: bits.r, t: bits.t, m: bits.m },
            };
            return Ok((blocked_manifest, bits, None));
        }
    }
    
    // Store trace for self-observation
    unsafe { 
        TRACE_HISTORY.push(bits.clone());
        if TRACE_HISTORY.len() > 100 { TRACE_HISTORY.remove(0); }
    }
    
    let manifest = Manifest {
        run_id: format!("r-{}", Uuid::new_v4()),
        goal_id: goal_id.to_string(),
        deliverables: vec![],
        evidence: serde_json::json!({ 
            "stdout": res.stdout,
            "expected_success": expected_success,
            "actual_success": passed,
            "l2_params": kernel.l2_params,
            "meta2_triggered": bits.m > 0.0
        }),
        bits: bits.clone().into(), // Convert to legacy Bits for compatibility
    };
    
    Ok((manifest, bits, meta2_proposal))
}

// Convert ExtendedBits to legacy Bits for API compatibility
impl From<ExtendedBits> for types::Bits {
    fn from(ext: ExtendedBits) -> Self {
        Self { a: ext.a, u: ext.u, p: ext.p, e: ext.e, d: ext.d, i: ext.i, r: ext.r, t: ext.t, m: ext.m }
    }
}
