# One Engine + One LM Architecture

## ✅ Implemented

### Core Engine
- **Single Binary**: Static Rust executable with embedded assets
- **HTTP API**: `/run` endpoint with OpenAPI 3.1 docs at `/swagger-ui`
- **Policy Gates**: γ-gates, risk assessment, time/LOC limits
- **Observability**: JSONL trace, before/after snapshots, bits tracking

### LM Integration
- **Pluggable LM**: Trait-based with StubLM for testing
- **Four Roles**: Planner, Critic (risk assessment), Reducer, Synthesizer
- **γ-Gates**: Skip LM when uncertainty > threshold

### Multi-Domain Support
- **SWE Domain**: Test running, AST transforms, code verification
- **Marketing Domain**: Keyword research, blog drafts, SEO validation
- **Extensible**: Add domains via atoms + oracles

### Contract (Stable)
```rust
Inputs: {goal_id, inputs, policy}
Outputs: {manifest: {run_id, deliverables, evidence, bits}, bits}
Bits: {A,U,P,E,Δ,I,R,T} ∈ [0..1]
Policy: {gamma_gate, time_ms, max_risk, tiny_diff_loc}
```

## 🚀 Usage

```bash
# Build single binary
cargo build --release

# Run server
./target/release/one-engine

# Test endpoints
curl http://127.0.0.1:8080/health
curl -X POST http://127.0.0.1:8080/run -d '{"goal_id":"swe.echo","inputs":{"message":"test"},"policy":{"gamma_gate":0.5,"time_ms":5000,"max_risk":0.3,"tiny_diff_loc":120}}'

# View docs
open http://127.0.0.1:8080/swagger-ui
```

## 📁 Structure
```
src/
  main.rs           # HTTP server + CLI
  api.rs            # /run endpoint
  engine/           # Core loop
    types.rs        # Bits, Policy, Manifest, Atom
    lm.rs           # LM trait + StubLM
    planner.rs      # Plan selection
    executor.rs     # Action execution
    verify.rs       # Domain oracles
    observe.rs      # Snapshots
    trace.rs        # JSONL logging
  domains/
    swe.rs          # SWE atoms + oracle
    mkt.rs          # Marketing atoms + oracle
assets/
  atoms/            # Domain atom definitions
  templates/        # LM prompt templates
```

## 🔄 Engine Loop
1. **Plan**: LM selects atoms based on goal + available atoms
2. **Gate**: Risk assessment via LM critic
3. **Execute**: Run atoms (CLI/HTTP/Python/WASM)
4. **Verify**: Domain oracles check outputs
5. **Trace**: Log to JSONL with bits

## 🎯 Next Steps
- Add real LM client (OpenAI/Anthropic)
- Implement WASM sandbox for untrusted code
- Add file deliverables and artifact management
- Extend SWE/Marketing atom libraries
- Add CI integration for automated testing
