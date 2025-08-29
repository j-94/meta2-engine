---
lane: A
title: Evidence Gate (U≥τ → verify-first)
source: docs/VERIFY.md
tags: [bits, gates, verify, uncertainty]
bits: {A: 1, U: 1, P: 1, E: 0, Δ: 0, I: 0, R: 0, T: 0}
---

1-liner
- If U≥τ, switch to verify-first: simulate, cite, then act.

context
- Calibrates overconfidence; prevents silent wrong actions.

pattern (3 steps)
1) Detect U≥τ; set mode=verify.
2) Produce dry_run.log + 1–3 refs; attach to manifest.
3) If dry_run_ok=0 after 2 attempts → escalate.

evidence
- docs/VERIFY.md; engine kernel evidence_gate; demo in /validate.

CTA
- Add a verify mode branch to your loop before any irreversible action.

