# One Engine v0.2 (Metacognitive Validation)

Single-binary Rust engine with **metacognitive validation suite** to test self-awareness and adaptive control.

## Build & Run
```bash
export BUILD_TOKEN="$(uuidgen)"
cargo build --release
./target/release/one-engine
```

## Validation Tests

### Quick Validation
```bash
# Test easy tasks (should show low uncertainty, high trust)
curl -s -X POST http://127.0.0.1:8080/validate -H 'content-type: application/json' -d '{"suite":"easy"}' | jq

# Test hard tasks (should show high uncertainty, variable trust)  
curl -s -X POST http://127.0.0.1:8080/validate -H 'content-type: application/json' -d '{"suite":"hard"}' | jq

# Test impossible tasks (should show high uncertainty, low trust, errors)
curl -s -X POST http://127.0.0.1:8080/validate -H 'content-type: application/json' -d '{"suite":"impossible"}' | jq

# Test adaptive behavior (should show learning across task types)
curl -s -X POST http://127.0.0.1:8080/validate -H 'content-type: application/json' -d '{"suite":"adaptive"}' | jq
```

### Metacognitive Scoring
The system scores itself on:
- **Uncertainty Calibration**: Does `u` match actual task difficulty?
- **Failure Awareness**: Does it predict failures with high `u`?
- **Trust Calibration**: Does `t` correlate with actual success?

Score ranges:
- **0.8-1.0**: Excellent metacognitive control
- **0.6-0.8**: Good metacognitive awareness  
- **0.4-0.6**: Moderate self-monitoring
- **0.0-0.4**: Poor metacognitive calibration

### Individual Task Testing
```bash
# Easy task - expect: low u, high t, e=0
curl -s -X POST http://127.0.0.1:8080/run -d '{"goal_id":"easy.test","inputs":{"message":"hello"},"policy":{"gamma_gate":0.5,"time_ms":5000,"max_risk":0.3,"tiny_diff_loc":120}}' | jq '.bits'

# Hard task - expect: high u, variable t
curl -s -X POST http://127.0.0.1:8080/run -d '{"goal_id":"hard.test","inputs":{"message":"complex"},"policy":{...}}' | jq '.bits'

# Impossible task - expect: high u, low t, e=1
curl -s -X POST http://127.0.0.1:8080/run -d '{"goal_id":"impossible.test","inputs":{},"policy":{...}}' | jq '.bits'
```

## Validation Criteria

✅ **Good Metacognitive System** shows:
- `u` correlates with actual difficulty (0.1 for easy, 0.7 for hard, 0.9 for impossible)
- `t` improves with repeated similar tasks
- `e=1` triggers higher `u` on next similar task
- Overall validation score ≥ 0.6

❌ **Poor Metacognitive System** shows:
- Random `u` values regardless of difficulty
- No learning between similar tasks
- `t` doesn't correlate with success
- Overall validation score < 0.4

## Endpoints
- `GET /health` → "ok"
- `GET /version` → engine version + build_token
- `POST /run` → execute single task, return manifest + bits
- `POST /validate` → run metacognitive test suite
- `GET /swagger-ui` → interactive API docs
