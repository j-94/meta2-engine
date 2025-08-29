# VERIFY Grid

When `U=1` or above `τ`, switch to verify mode before acting.

- Dry run: simulate command or run static checks.
- Attach `dry_run.log` to evidence for PR/trace.
- Require at least one verifiable reference or sandbox output for claims.
- Abort when `dry_run_ok=0` and `attempts ≥ 2`.

Checklist
- [ ] Dry-run completed (`dry_run_ok=1`)
- [ ] Evidence attached (`dry_run.log`)
- [ ] References included (spec, logs, links)

