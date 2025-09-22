# Meta² Heuristic Refinement Benchmark — Research Note

_Date:_ 2025-09-21  _Author:_ Meta² AgentOps

## 1. Goal

Demonstrate a reproducible testbed where a participant refines a crowd-derived
heuristic. Being able to start from noisy, biased crowd rules and systematically
improve them is a key marker of adaptive intelligence; the subject must
identify systematic errors, propose fixes, and generalise beyond provided
examples.

## 2. Experimental Setup

### 2.1 Synthetic environment

- Features: `color ∈ {red, blue, green}`, `size ∈ {S, M, L}`, `weight ∈ [1, 10]`.
- Oracle policy: accept iff `(color == red AND weight > 5)` OR `(size == L AND weight > 7)`.
- Crowd heuristic: accept iff `color == red`, with 10% random label noise to
  mimic inconsistent annotators.
- Dataset generation: `meta2-engine heuristics generate --out benchmarks/refinement.jsonl --count 200 --seed 42 --noise 0.1`.
- Splits: 120 train / 40 val / 40 test. Crowd error rate ≈ 25.8% (train), 27.5%
  (test).

### 2.2 Evaluation protocol

- Subject receives the dataset and the crowd heuristic behaviour.
- Allowed to inspect a small labelled subset (train) and produce a refined rule.
- Scoring uses held-out test split only.
- Metrics: accuracy, F1, confusion counts, performance delta vs crowd baseline,
  accuracy on the crowd-error subpopulation.

### 2.3 Baselines

1. **Crowd heuristic** (status quo)
2. **Naïve refinements** that copy the crowd rule (`color == "red"`).
3. **Oracle-aligned refinement** either via baseline refiner or simple boolean
   expression replicating the ground-truth decision surface. Serves as an upper
   bound.

## 3. Results (test split, n=40)

| Refiner | Accuracy | Δ vs crowd | F1 | F1 Δ | Crowd FP/FN | Refined FP/FN | Accuracy on crowd mistakes |
|---------|----------|------------|-----|------|-------------|----------------|-----------------------------|
| Crowd heuristic | 0.725 | — | 0.5217 | — | 3 / 8 | 3 / 8 | 0.0 |
| Naïve copy (`color == "red"`) | 0.675 | -0.050 | 0.4800 | -0.0417 | 3 / 8 | 5 / 8 | 0.0 |
| Oracle-style expression | 1.000 | +0.275 | 1.0000 | +0.4783 | 3 / 8 | 0 / 0 | 1.0 |
| Built-in baseline refiner | 1.000 | +0.275 | 1.0000 | +0.4783 | 3 / 8 | 0 / 0 | 1.0 |

Key observations:

- Crowd majority rule misses 11 of 40 test cases (27.5%) and mislabels every
  example where size/weight override the colour cue.
- A naïve refinement that simply repeats the colour check actually degrades
  performance (-5% absolute accuracy).
- Correcting the heuristic with an interpretable boolean rule eliminates all
  false positives/negatives and perfectly resolves the biased subpopulation.
- Accuracy on the "crowd mistake" subset (n=11) jumps from 0.0 to 1.0, showing
  complete recovery of the previously ignored cohort.

## 4. Interpreting the benchmark

- Intelligence marker: participants must detect the failure mode (dependency on
  weight/size) and generalise a corrective rule with minimal supervision.
- Human level reference: run the same protocol with human participants; expect
  partial fixes (accuracy 0.80–0.90) given limited time/budget.
- Agent aspiration: match or exceed human Δ while keeping rules simple and
  explainable.

## 5. Reproduction recipe

```bash
# 1. Generate dataset (adjust count/noise as needed)
meta2-engine heuristics generate --out benchmarks/refinement.jsonl --count 200 --seed 42 --noise 0.1

# 2. Score the crowd baseline
meta2-engine heuristics score --dataset benchmarks/refinement.jsonl --refiner baseline

# 3. Evaluate custom refinement script
meta2-engine heuristics score \
    --dataset benchmarks/refinement.jsonl \
    --refiner-script my_refiner.py \
    --function refined

# 4. Quick experimentation with boolean expressions
meta2-engine heuristics score \
    --dataset benchmarks/refinement.jsonl \
    --expression '(color == "red" and weight > 5) or (size == "L" and weight > 7)'
```

## 6. Next steps

1. Collect human-written refinements to establish an empirical baseline.
2. Integrate the task into the Meta² planner loop (agent proposes rule, engine
   validates via this harness).
3. Extend dataset variants: non-linear boundaries, multi-class labels, and
   sequential decision heuristics (e.g., triage queues).
4. Track longitudinal metrics (Δ accuracy, sample efficiency) in SSOT for model
   governance.

---
_Artifacts:_ `benchmarks/refinement.jsonl`, CLI in `meta2-engine heuristics`, report
stored at `northstar/1-init/red/engine/reports/heuristic_refinement_report.md`.
