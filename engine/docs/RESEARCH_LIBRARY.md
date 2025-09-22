# Research Library

A compact, auditable collection of prompts, policies, schemas, traces, and docs from many repos/branches, indexed into a single JSONL for retrieval and analysis.

## Goals
- Compression: minimal, bits-aware metadata per artifact.
- Provenance: stable IDs with checksums and timestamps.
- Freshness: TTLs to trigger Δ=1 when stale.
- Auditability: artifacts are files in git; index is reproducible.

## Layout
- `research/index.jsonl` — normalized artifact index (append-only, reproducible)
- Sources scanned by default:
  - `prompts/**`, `policies/**`, `schemas/**`, `trace/golden/**`, `docs/**`, `README.md`

## Artifact Schema
- See `schemas/RESEARCH_ARTIFACT.schema.json`.
- Minimal fields: `{id, kind, path, ts, ttl, tags, checksum}`

## Building the Index & Reports
- CLI: `one-research` (binary in this crate)
- Examples:
  - `cargo run --bin one-research -- --root . --out research/index.jsonl`
  - `cargo run --bin one-research -- --root . --report-out research/report.json`
  - Or via Make: `make research-index` (writes both the index and `research/report.json`)
- The report surfaces counts by kind/tag/branch plus TTL-based freshness alerts.

## Bringing External Repos
- Option A: clone them under `external/` and run the indexer with `--root external`.
- Option B: export selected folders (prompts/policies/schemas/docs/traces) into `research/sources/NAME/` and re-index.
- Option C: use `history_miner` (local) and point `--root` to its export directory.

## Notes
- No network access required; runs on local files.
- Checksums use a simple Adler32 for determinism without external crates.

