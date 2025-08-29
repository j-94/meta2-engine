---
lane: C
title: Intent Triad Compressor (goal/constraints/evidence)
source: PROMPT_TEMPLATES/triad.md
tags: [prompt, triad, alignment]
bits: {A: 0, U: 1, P: 0, E: 0, Δ: 0, I: 1, R: 0, T: 0}
---

1-liner
- Compress ambiguity into a 3-field intent; ask one concrete question.

prompt
```
Compress the request into:
- goal: <one sentence>
- constraints: [≤3]
- evidence: [≤3]
If anything is missing, ask exactly one concrete question to proceed.
Return JSON {goal, constraints, evidence, question?}.
```

CTA
- Trigger this when A=0 or I=1; attach result to the next turn.

