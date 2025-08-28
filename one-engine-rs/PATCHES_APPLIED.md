# ✅ One Engine + One LM Patches Applied

## Minimal patches successfully applied to upgrade the MVP to match the "One Engine + One LM" spec:

### 1. LM Integration (`src/engine/lm.rs`)
- **StubLm** with planner/critic roles
- **γ-gate**: Abstains when uncertainty > threshold
- **Trait-based**: Ready for real LM clients (OpenAI/Anthropic)

### 2. Policy Gates (`src/engine/planner.rs`)
- **γ-gate**: `uncertainty > policy.gamma_gate` → bail
- **Risk gate**: `risk > policy.max_risk` → bail with intervention bit

### 3. Execution & Deliverables (`src/engine/executor.rs`)
- **File creation**: Writes `out/hello.txt` from `inputs.message`
- **Artifact tracking**: Returns deliverable paths
- **Drift detection**: Sets Δ bit when needed

### 4. Verification (`src/engine/verify.rs`)
- **Deliverable validation**: Checks file exists
- **Trust computation**: 0.9 for success, 0.1 for failure

### 5. Observability (`src/engine/trace.rs`)
- **JSONL ledger**: `trace/ledger.jsonl` with full audit trail
- **Structured logging**: run_id, goal, steps, bits, deliverables

### 6. Complete Engine Loop (`src/engine/mod.rs`)
1. **PLAN** → LM selects steps (γ-gated)
2. **CRITIC** → Pre-execution risk assessment  
3. **EXECUTE** → File write with artifact tracking
4. **VERIFY** → Deliverable existence check
5. **TRACE** → JSONL append with full context

## 🎯 Test Results

```bash
curl -X POST http://127.0.0.1:8080/run \
  -d '{"goal_id":"swe.echo","inputs":{"message":"test"},"policy":{"gamma_gate":0.5,"time_ms":5000,"max_risk":0.3,"tiny_diff_loc":120}}'
```

**✅ Expected Results Achieved:**
- `out/hello.txt` created with message content
- `manifest.deliverables: ["out/hello.txt"]`
- `bits.t: 0.9` (high trust)
- Full trace in `trace/ledger.jsonl`

## 📦 Single Binary
- **Size**: ~14MB static binary
- **Dependencies**: Minimal (no Python/Node.js runtime)
- **Deployment**: Copy binary + run
- **Observability**: Built-in OpenAPI docs at `/swagger-ui`

## 🚀 Ready for Extension
- **Real LM**: Replace `StubLm` with OpenAI/Anthropic client
- **More Actions**: Add CLI/HTTP/WASM executors
- **Domain Atoms**: SWE, Marketing, Research libraries
- **Advanced Gates**: Budget limits, sandbox policies

The engine now fully implements the "One Engine + One LM" architecture with γ-gates, policy enforcement, deliverable tracking, and complete audit trails.
