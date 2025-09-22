---
lane: A
title: Ask–Act Gate (A=1 ∧ P=1 ∧ Δ=0)
source: src/engine/kernel.rs
tags: [bits, gates, safety, ask-act]
bits: {A: 1, U: 0, P: 1, E: 0, Δ: 0, I: 0, R: 0, T: 1, M: 0}
---

1-liner
- Only act when aligned and approved with no drift: A=1 ∧ P=1 ∧ Δ=0.

context
- Prevents unsafe or premature actions; turns ambiguity into one clarifying question.

pattern (3 steps)
1) Check bits; if A=0 or Δ=1 → compress to intent triad and ask one question.
2) If P=0 and capability risky → request approval (show exact diff/command).
3) Act only after conditions hold; attach evidence to manifest.

evidence
- Kernel gate in code; demo traces from /validate (easy/hard/impossible) and /users/*/chat.

CTA
- Add this gate to any agent loop before executing tools or patches.

