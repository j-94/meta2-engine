# Contributing

We use a PR-only workflow with required reviews and CI checks. All changes must land via Pull Request to `main`.

## Workflow
- Create a feature branch from `main`.
- Keep PRs focused and small when possible.
- Ensure `cargo fmt`, `cargo clippy`, `cargo build`, and `cargo test` all pass locally.
- Open a PR and fill out the template. Link related issues.
- Request at least one review. Address feedback via additional commits.
- Squash-merge after CI is green and approvals are in.

## Conventions
- Rust edition 2021, edition lints enabled.
- Deny warnings in CI via Clippy.
- Prefer small, pure functions and unit tests near logic.
- Commit messages should be clear and imperative.

## Running locally
```bash
rustup toolchain install stable
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo test --all --locked
```

## Security
- Do not commit secrets. Use env vars locally and in CI.
- Report vulnerabilities privately to the maintainers.

## Releases
- Changes are merged to `main` via PR.
- Tag releases with semantic versions (e.g., `v0.2.0`).

