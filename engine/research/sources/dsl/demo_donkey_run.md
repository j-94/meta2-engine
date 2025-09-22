# Donkey-DSL Demo Replication

This note captures an offline reproduction of the `demo_donkey.py` experiment from [`j-94/DSL`](https://github.com/j-94/DSL). The goal was to document how the recommender behaves with the bundled trajectory corpus and to capture the baseline metacognitive metrics that can be indexed alongside other Meta² research artifacts.

## Environment
- Python 3.12
- Dependencies installed with `pip install -r requirements.txt` from the DSL repository (installs `flask` and `numpy`).
- Dataset: `data/trajectories.json` provided by the DSL repository. The demo creates the file automatically on first run when it is missing.

## Reproduction Steps
1. Clone the DSL repository and install dependencies:
   ```bash
   git clone https://github.com/j-94/DSL.git
   pip install -r DSL/requirements.txt
   ```
2. Execute the demo script:
   ```bash
   python DSL/demo_donkey.py
   ```

## Key Observations
- The system bootstraps by creating `data/trajectories.json` and loads 10 historical trajectories.
- Initial recommendations for diverse engineering tasks were:
  - Bug fix (authentication): `u0.5 | f? | mem(0.5) | test(0.8)`
  - Refactor (database): `u0.5 | f? | mem(0.5) | test(0.5)`
  - Exploration (ML architecture): `u2.0 | f? | mem(0.5) | test(0.3)`
  - Production incident: `u0.5 | f? | mem(0.5) | test(0.8)`
  - New caching layer: `u1.0 | f? | mem(0.5) | test(0.5)`
- Recording a new successful trajectory (`u0.3 f! | mem(0.9) | test(0.9)`) immediately influenced the follow-up recommendation for a related “memory leak” task, which now reuses the high-test, high-memory profile with higher confidence.
- Pattern analysis highlights that successful bug-fix trajectories skew towards lower exploration (`u≈0.3`) and aggressive testing (`test≈0.9`).
- The integrated behavior analysis module measured:
  - Creative response: actual exploration intensity `u≈4.1`, compliance 39%, tests detected.
  - Conservative response: actual exploration intensity `u≈0.9`, compliance 94%, tests not detected.
- Simulated DSL calibration mapped requested → realized exploration values (`u0.1→0.7`, `u0.5→1.5`, `u1.0→1.5`, `u2.0→1.5`, `u5.0→5.0`).
- A short three-turn DSL-guided conversation achieved ~48% behavior compliance with a success estimate of 42%.

## Artifacts for Indexing
- Command transcript and the generated `data/trajectories.json` (10 entries) are available from the DSL repository and can be mirrored into `research/sources/` if deeper analysis is required.
- The observations above can be cross-referenced when evaluating future Meta² policy changes that interact with DSL-style behavioral injections.
