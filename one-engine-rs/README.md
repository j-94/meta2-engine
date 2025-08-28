# One Engine RS

A minimal Rust implementation of the One Engine MVP.

## ✅ Implemented (MVP+)
- Single binary (Rust) exposing `/run` + OpenAPI UI at `/swagger-ui`
- LM roles (planner/critic) via `StubLm` behind γ-gate
- Policy gates: γ (uncertainty), risk
- Observability: JSONL `trace/ledger.jsonl`, before/after snapshots, Bits (A/U/P/E/Δ/I/R/T)
- Deliverables: writes `out/hello.txt` from inputs.message

## Quick Start

```bash
cargo build --release
RUST_LOG=info ./target/release/one-engine
```

Server runs on http://127.0.0.1:8080

- Health check: `GET /health`
- OpenAPI docs: `GET /swagger-ui`
- Engine endpoint: `POST /run`

## Example Request

```bash
curl -s -X POST 'http://127.0.0.1:8080/run' \
  -H 'content-type: application/json' \
  -d '{
    "goal_id":"demo.echo",
    "inputs":{"message":"hello from one-binary"},
    "policy":{"gamma_gate":0.5,"time_ms":5000,"max_risk":0.3,"tiny_diff_loc":120}
  }'
```

Expected:
- `out/hello.txt` exists and contains your message
- Response includes `manifest.deliverables: ["out/hello.txt"]` and `bits.t ≈ 0.9`
- Trace logged to `trace/ledger.jsonl`
