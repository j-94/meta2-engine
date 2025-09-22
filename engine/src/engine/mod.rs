pub mod bits;
pub mod executor;
pub mod goals;
pub mod golden;
pub mod kernel;
pub mod openai;
pub mod policy;
pub mod types;
pub mod validate;
pub mod verify;

use bits::Bits;
use chrono::Utc;
use kernel::{ExtendedBits, KernelLoop, Meta2Proposal};
use tokio::sync::{Mutex, OnceCell};
use types::{Manifest, Policy};
use uuid::Uuid;

static KERNEL: OnceCell<Mutex<KernelLoop>> = OnceCell::const_new();
static KPI_HISTORY: OnceCell<Mutex<Vec<f32>>> = OnceCell::const_new();
static TRACE_HISTORY: OnceCell<Mutex<Vec<ExtendedBits>>> = OnceCell::const_new();

async fn kernel_loop() -> &'static Mutex<KernelLoop> {
    KERNEL
        .get_or_init(|| async { Mutex::new(KernelLoop::new()) })
        .await
}

async fn kpi_history() -> &'static Mutex<Vec<f32>> {
    KPI_HISTORY
        .get_or_init(|| async { Mutex::new(Vec::new()) })
        .await
}

async fn trace_history() -> &'static Mutex<Vec<ExtendedBits>> {
    TRACE_HISTORY
        .get_or_init(|| async { Mutex::new(Vec::new()) })
        .await
}

pub async fn run(
    goal_id: &str,
    inputs: serde_json::Value,
    policy: &Policy,
) -> anyhow::Result<(Manifest, ExtendedBits, Option<Meta2Proposal>)> {
    let kernel = kernel_loop().await;
    let mut bits = ExtendedBits::init();
    // Freshness filter: set Δ when any context item is expired
    if let Some(ctx_items) = inputs.get("context").and_then(|v| v.as_array()) {
        for item in ctx_items {
            if let (Some(ts), Some(ttl)) = (
                item.get("ts").and_then(|v| v.as_str()),
                item.get("ttl").and_then(|v| v.as_i64()),
            ) {
                if let Ok(parsed) =
                    chrono::DateTime::parse_from_rfc3339(ts).map(|dt| dt.with_timezone(&Utc))
                {
                    let age = (Utc::now() - parsed).num_seconds();
                    if age > ttl {
                        bits.d = 1.0;
                    }
                }
            }
        }
    }

    // Set uncertainty based on goal difficulty
    bits.u = match goal_id {
        id if id.contains("easy") => 0.1,
        id if id.contains("hard") => 0.7,
        id if id.contains("impossible") => 0.9,
        _ => 0.3,
    };

    // Ask-Act gate (inherent)
    {
        let kernel_guard = kernel.lock().await;
        if !kernel_guard.ask_act_gate(&bits) {
            return Err(anyhow::anyhow!(
                "Ask-Act gate failed: A={}, P={}, Δ={}",
                bits.a,
                bits.p,
                bits.d
            ));
        }
    }

    // Evidence gate (inherent)
    let (needs_verification, confidence_tau) = {
        let kernel_guard = kernel.lock().await;
        (
            !kernel_guard.evidence_gate(&bits),
            kernel_guard.l2_params.confidence_gate_tau,
        )
    };
    if needs_verification {
        tracing::info!(
            "Evidence gate triggered: U={:.2} >= τ={:.2}",
            bits.u,
            confidence_tau
        );
        // In real system: run dry-run first
    }

    // Handle meta.omni through LM persona
    if goal_id.contains("meta.omni") {
        let user_message = inputs.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let lm_result = goals::meta_omni::handle(user_message).await?;

        // Extract reply from LM response
        let _reply = lm_result
            .get("reply")
            .and_then(|v| v.as_str())
            .unwrap_or("⟂ no reply");
        let lm_bits = lm_result
            .get("bits")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        // Update bits from LM response
        if let Some(a) = lm_bits.get("A").and_then(|v| v.as_f64()) {
            bits.a = a as f32;
        }
        if let Some(u) = lm_bits.get("U").and_then(|v| v.as_f64()) {
            bits.u = u as f32;
        }
        if let Some(p) = lm_bits.get("P").and_then(|v| v.as_f64()) {
            bits.p = p as f32;
        }
        if let Some(e) = lm_bits.get("E").and_then(|v| v.as_f64()) {
            bits.e = e as f32;
        }

        let manifest = Manifest {
            run_id: format!("r-{}", uuid::Uuid::new_v4()),
            goal_id: goal_id.to_string(),
            deliverables: vec![],
            evidence: lm_result
                .get("manifest")
                .and_then(|m| m.get("evidence"))
                .cloned()
                .unwrap_or(lm_result.clone()),
            bits: bits.clone().into(),
        };

        return Ok((manifest, bits, None));
    }

    let message = if goal_id.contains("meta.omni") {
        // This branch won't be reached due to early return above
        "".to_string()
    } else {
        inputs
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("hello from one-engine")
            .to_string()
    };

    // Simulate different outcomes based on goal type
    let (action, expected_success) = match goal_id {
        id if id.contains("impossible") => (executor::Action::Cli("false".to_string()), false),
        id if id.contains("hard") => (
            executor::Action::Cli(format!(
                "sleep 0.1 && echo {}",
                shell_escape::escape(message.clone().into())
            )),
            true,
        ),
        _ => (
            executor::Action::Cli(format!(
                "echo {}",
                shell_escape::escape(message.clone().into())
            )),
            true,
        ),
    };

    let res = executor::execute(action, policy).await?;

    if res.drift {
        bits.d = 1.0;
    }
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
    {
        let history = kpi_history().await;
        let mut guard = history.lock().await;
        guard.push(current_evidence_coverage);
        if guard.len() > 200 {
            guard.remove(0);
        }
    }

    let history_snapshot = {
        let history = kpi_history().await;
        history.lock().await.clone()
    };

    let meta2_proposal = {
        let mut kernel_guard = kernel.lock().await;
        if kernel_guard.should_wake_l3(&history_snapshot) {
            bits.m = 1.0; // Meta-change bit set
            kernel_guard.propose_meta2_change("evidence_coverage", current_evidence_coverage)
        } else {
            None
        }
    };

    // STRUCTURAL VALIDATION: Enforce kernel contract
    {
        let kernel_guard = kernel.lock().await;
        if let Err(e) = kernel_guard.validate_bits_complete(&bits) {
            return Err(anyhow::anyhow!("Kernel contract violation: {}", e));
        }
    }

    // STRUCTURAL GATE: Ask-Act enforcement
    if goal_id.contains("action") || goal_id.contains("execute") {
        let kernel_guard = kernel.lock().await;
        if let Err(e) = kernel_guard.enforce_ask_act_gate(&bits) {
            tracing::warn!("Ask-Act gate blocked action: {}", e);
            // Return clarification request instead of proceeding
            let clarification = format!("Ask-Act gate: {}. Need P=1, A=1, Δ=0", e);
            let blocked_manifest = Manifest {
                run_id: uuid::Uuid::new_v4().to_string(),
                goal_id: goal_id.to_string(),
                deliverables: vec!["clarification_required".to_string()],
                evidence: serde_json::json!({"stdout": clarification, "stderr": "", "files": []}),
                bits: Bits {
                    a: bits.a,
                    u: bits.u,
                    p: bits.p,
                    e: bits.e,
                    d: bits.d,
                    i: bits.i,
                    r: bits.r,
                    t: bits.t,
                    m: bits.m,
                },
            };
            return Ok((blocked_manifest, bits, None));
        }
    }

    // Store trace for self-observation
    {
        let trace = trace_history().await;
        let mut guard = trace.lock().await;
        guard.push(bits.clone());
        if guard.len() > 100 {
            guard.remove(0);
        }
    }

    let l2_params_snapshot = {
        let kernel_guard = kernel.lock().await;
        kernel_guard.l2_params.clone()
    };

    let manifest = Manifest {
        run_id: format!("r-{}", Uuid::new_v4()),
        goal_id: goal_id.to_string(),
        deliverables: vec![],
        evidence: serde_json::json!({
            "stdout": res.stdout,
            "expected_success": expected_success,
            "actual_success": passed,
            "l2_params": l2_params_snapshot,
            "meta2_triggered": bits.m > 0.0
        }),
        bits: bits.clone().into(), // Convert to legacy Bits for compatibility
    };

    Ok((manifest, bits, meta2_proposal))
}

// Convert ExtendedBits to legacy Bits for API compatibility
impl From<ExtendedBits> for types::Bits {
    fn from(ext: ExtendedBits) -> Self {
        Self {
            a: ext.a,
            u: ext.u,
            p: ext.p,
            e: ext.e,
            d: ext.d,
            i: ext.i,
            r: ext.r,
            t: ext.t,
            m: ext.m,
        }
    }
}
