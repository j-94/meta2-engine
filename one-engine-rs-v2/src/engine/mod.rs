pub mod bits;
pub mod types;
pub mod executor;
pub mod verify;
pub mod policy;
pub mod validate;
pub mod kernel;
pub mod meta_prompt;

use types::{Manifest, Policy};
use kernel::{ExtendedBits, KernelLoop, Meta2Proposal};
use uuid::Uuid;

static mut KERNEL: Option<KernelLoop> = None;
static mut KPI_HISTORY: Vec<f32> = Vec::new();

pub async fn run(goal_id: &str, inputs: serde_json::Value, policy: &Policy) -> anyhow::Result<(Manifest, ExtendedBits, Option<Meta2Proposal>)> {
    let kernel = unsafe { KERNEL.get_or_insert_with(KernelLoop::new) };
    let mut bits = ExtendedBits::init();
    
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
    
    let message = if goal_id.contains("meta.omni") {
        // Handle meta-prompt processing
        let system = inputs.get("system").and_then(|v| v.as_str()).unwrap_or("");
        let user_message = inputs.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let empty_history = vec![];
        let history = inputs.get("history").and_then(|v| v.as_array()).unwrap_or(&empty_history);
        
        meta_prompt::process_meta_prompt(system, user_message, history)
    } else {
        inputs.get("message").and_then(|v| v.as_str()).unwrap_or("hello from one-engine").to_string()
    };
    
    // Simulate different outcomes based on goal type
    let (action, expected_success) = match goal_id {
        id if id.contains("impossible") => (executor::Action::Cli("false".to_string()), false),
        id if id.contains("hard") => (executor::Action::Cli(format!("sleep 0.1 && echo {}", shell_escape::escape(message.clone().into()))), true),
        id if id.contains("meta.omni") => (executor::Action::Cli(format!("echo {}", shell_escape::escape(message.clone().into()))), true),
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
        Self { a: ext.a, u: ext.u, p: ext.p, e: ext.e, d: ext.d, i: ext.i, r: ext.r, t: ext.t }
    }
}
