# Unified Architecture Context Pack

This document compiles the essential context to derive ONE production architecture from the current codebases and branches.

## 1) North Stars
- One interface, PR-first governance, CI-gated changes.
- Bits-native control (A,U,P,E,Δ,I,R,T,M) and simple gates (Ask–Act, Evidence, CAPS).
- Self-observation → control → meta-control loops; golden traces for regression.

## 2) Layered Model (α / β / γ)
- α Distinctions: Schemas, types, policies.
- β Metacognition (pattern selection): plan shapes (direct, single_tool, two_step) and triad compression.
- γ Metacybernetics (control selection): model/params/tools; ask–act and verify-first.
- L1 execution, L2 gates, L3 meta-proposals (shadow rollout, rollback covenants).

## 3) Bits and Gates
- Bits: A,U,P,E,Δ,I,R,T,M (code: `src/engine/bits.rs`, kernel: `src/engine/kernel.rs`).
- Gates: Ask–Act (A=1 ∧ P=1 ∧ Δ=0); Evidence (U≥τ → verify-first); CAPS (P=1 for network/file_write/identity).
- Policies: `policies/POLICY.ask_act.yaml`, `policies/CAPS.yaml`, `policies/RETRY.yaml`, `policies/BOUNDARIES.yaml`, `policies/LANES.yaml`.

## 4) APIs and Loops
- Rust service (Axum):
  - Health/info: `GET /health`, `GET /version`
  - Engine: `POST /run`, `POST /validate`, `GET /swagger-ui`
  - Multitenant chat: `POST /users/{id}/chat` (meta.omni), SSE beacons at `GET /progress.sse`
  - Golden: `GET /golden/{name}`, `POST /validate_golden`
  - Research index: `GET /research/index`
  - Python loops: `POST /nstar/run`, `GET /nstar/hud`
  - Meta bandit step: `POST /meta/run`

- Python single-file loops:
  - `scripts/nstar.py`: 4-layer cognition→metacog→cybernetics→metacybernetics with receipts to `trace/receipts.jsonl`.
  - `scripts/meta_loop.py`: β/γ bandit pick (UCB) optimizing R(s) with receipts to `trace/meta_receipts.jsonl`.

## 5) Schemas & Artifacts
- Trace schema: `schemas/TRACE.schema.json` (bits + telemetry envelope).
- Kernel contract: `schemas/KERNEL_CONTRACT.json`.
- Intent schema: `schemas/INTENT.schema.json`.
- Context freshness: `schemas/CONTEXT.schema.json`.
- Research artifact schema: `schemas/RESEARCH_ARTIFACT.schema.json`.
- Golden examples: `trace/golden/wolfram_unity.json`.

## 6) Golden Validation
- API: `POST /validate_golden` `{name}` → pass/fail summary and bits.
- Use for regression + CI gates.

## 7) Research Library (Compression)
- Indexer: `one-research` binary (or `make research-index`) builds `research/index.jsonl`.
- Sources: prompts, policies, schemas, docs, golden.
- Provenance: checksum + git branch/commit.

## 8) External Repo Aggregation (j-94/v0-j4cob)
- Clone path: `_external/v0-j4cob` (default: main).
- Aggregation branch: `aggregate` starting from default.
- Merge attempt results: `_output/merge_branches_report.json` (see conflicts vs merged).
- Production snapshot repo: `_output/v0-j4cob-production` (single commit).

## 9) Quick Architecture Decision Records (ADRs)
- ADR-01 Bits & Gates: Use A,U,P,E,Δ,I,R,T,M + Ask–Act/Evidence/CAPS as invariants.
- ADR-02 PR-First: All changes via PR; CI: fmt, clippy -D warnings, build, test, golden.
- ADR-03 Verify-First: U≥τ requires dry-run evidence per `docs/VERIFY.md` before act.
- ADR-04 Meta Selection: β/γ via bandit (UCB), hysteresis ε; persist across runs.

## 10) Join Points (Integration Plan)
- Unify prompts: consolidate system prompts into a versioned `prompts/` with triad template.
- Route control: drive β/γ selections for tasks matching patterns (length/type) via `/meta/run`.
- Evidence hooks: wire dry-run logs into manifests; expose in UI.
- Approvals: add simple `POST /approve` to set P=1 for specific actions (optional).
- Shadow rollouts: implement policy registry + replay on golden; propose Meta‑PR.

## 11) Governance & CI
- PR template, CODEOWNERS, CONTRIBUTING, CI workflows in `.github/`.
- Gate on golden validation and tests.

## 12) Packaging
- Production repo ready at `_output/v0-j4cob-production`.
- Merge reports:
  - `_output/merge_report.json` (PR heads)
  - `_output/merge_branches_report.json` (remote branches)

## 13) Commands
- Run server: `RUST_LOG=info cargo run`
- Chat demo: `curl -s -X POST -H 'x-api-key: demo-key-123' -H 'content-type: application/json' http://127.0.0.1:8080/users/demo/chat -d '{"message":"hello"}'`
- Meta pick: `curl -s -X POST -H 'content-type: application/json' http://127.0.0.1:8080/meta/run -d '{"task":"compress_chatlog"}'`
- Golden validate: `curl -s -X POST -H 'content-type: application/json' http://127.0.0.1:8080/validate_golden -d '{"name":"wolfram_unity"}'`
- Research index: `make research-index`

---

This pack should be sufficient to bootstrap a unified architecture review and build a consolidated production repo. See the merge reports to decide which conflicting branches to cherry-pick or rebase.
