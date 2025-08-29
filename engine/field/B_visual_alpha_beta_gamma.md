---
lane: B
title: α/β/γ Loop with Bits Overlay
source: ARCHITECTURE_CONTEXT.md
tags: [diagram, kernel, bits, control]
bits: {A: 1, U: 0, P: 1, E: 0, Δ: 0, I: 0, R: 0, T: 1, M: 0}
---

1-liner
- α (distinctions) → β (plans) → γ (control) with bits gating each hop.

context
- Communicates the control stack to others in one glance; useful for READMEs and PRs.

diagram (mermaid)
```mermaid
flowchart TD
  A[α Distinctions\n(schemas, types, policies)] --> B[β Metacognition\n(plan selection)]
  B --> C[γ Metacybernetics\n(control selection)]
  C --> D[L1 Execution\n(act/apply)]
  D -->|telemetry s| B
  D -->|telemetry s| C
  subgraph Bits Gate
    GA[A]:::bit -->|∧| GP[P]:::bit -->|∧| GD[Δ=0]:::bit -->|= allow| D
  end
  classDef bit fill:#eef,stroke:#66f,stroke-width:1px
```

legend
- s: {pass, cost, time, mdl, errors, drift}; gates: Ask–Act, Evidence, CAPS.

CTA
- Drop this diagram into your repo’s ARCHITECTURE.md to align collaborators.

