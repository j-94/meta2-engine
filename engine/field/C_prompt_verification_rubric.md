---
lane: C
title: Verify-First Prompt Rubric (U≥τ)
source: docs/VERIFY.md
tags: [prompt, rubric, verify, trust]
bits: {A: 1, U: 1, P: 1, E: 0, Δ: 0, I: 0, R: 0, T: 0}
---

1-liner
- When uncertainty is high, switch to verify-first: dry-run + refs before acting.

context
- Avoids confident wrong actions; scales explanation depth with uncertainty.

prompt (parametric)
```
You are in verify mode (U={{U}} τ={{tau}}).
Task: {{task}}

Output JSON with:
- dry_run_ok: 0/1 and a short spec of the dry-run steps taken
- evidence: [links/logs/specs]
- refs: 1–3 verifiable anchors for claims
- limits: assumptions + what could fail
```

rubric (pass criteria)
- dry_run_ok=1, at least one log/spec reference, limits stated.

CTA
- Gate act on this rubric when U≥τ; attach `dry_run.log` in PRs.

