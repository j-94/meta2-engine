# Agents Guide

This repository requires a PR-only workflow. Human and automated agents must follow these rules to propose, review, and merge changes safely.

## Goals
- Keep `main` stable: no direct pushes.
- Ensure every change is reviewed, reproducible, and tested.
- Make automated agent activity transparent and auditable.

## How Agents Contribute
- Open a Pull Request from a feature branch; do not push to `main`.
- Use conventional PR titles (e.g., `feat: add validation endpoint`).
- Fill out the PR template with summary, risks, and rollout notes.
- Request review from code owners (see `.github/CODEOWNERS`).

## Required Checks
- Format: `cargo fmt --all -- --check`.
- Lint: `cargo clippy --all-targets -- -D warnings`.
- Build: `cargo build --release`.
- Test: `cargo test --all --locked`.
- CI must be green before merge (see `.github/workflows/`).

## Change Boundaries
- Allowed by default: code under `src/`, schemas in `schemas/`, prompts in `prompts/`, and policies in `policies/` when relevant to the change.
- Needs explicit human approval and rationale: changes to CI workflows, security-related configs, CODEOWNERS, or repository settings.
- Never commit secrets or personal data. Use environment variables and `.gitignore`d paths.

## Design & Quality
- Prefer small, focused PRs with clear intent.
- Write tests when changing logic; keep tests near the code where practical.
- Maintain API docs and schemas in sync when endpoints or types change.
- Explain metacognitive or policy changes and expected impact on behavior.

## Safety & Policy
- Follow repository policies in `policies/` and refer to `README.md` for runtime expectations.
- Do not check in runtime artifacts (`trace/`, `users/`, `.chat_state/`).
- Use clear failure modes and avoid silent fallbacks.

## Release Flow
- Merge via squash after approvals and passing CI.
- Tag releases from `main` using semantic versions (e.g., `v0.2.0`).

For detailed contributor steps, see `CONTRIBUTING.md`.

