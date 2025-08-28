# Meta² Metacognitive Engine

[![CI](https://github.com/j-94/meta2-engine/workflows/CI/badge.svg)](https://github.com/j-94/meta2-engine/actions)

A self-tuning, bits-native metacognitive system with inherent meta-meta-cognition (L3) that learns how its learning should change.

## 🧠 Architecture

**L1 (Task Level)**: Plan→Act→Verify with metacognitive bits  
**L2 (Control Level)**: Ask-act gates, confidence thresholds, retry policies  
**L3 (Meta Level)**: Learns how L2 should adapt via guarded policy changes  

## 🎯 Core Features

- **Bits-Native**: All operations emit {A,U,P,E,Δ,I,R,T,M} metacognitive bits
- **PR-First**: Agent never commits directly, only creates PRs when confident
- **Self-Tuning**: L3 layer proposes policy adaptations via META2_PRs
- **Multi-Tenant**: Per-user isolation with quotas and custom policies
- **Conversational**: Chat interface with META_OMNI prompt system

## 🚀 Quick Start

```bash
# Clone and build
git clone https://github.com/j-94/meta2-engine.git
cd meta2-engine
cargo build --release

# Start the engine
cd engine
tmux new-session -d -s engine 'RUST_LOG=info ../target/release/one-engine'

# Test metacognitive validation
curl -s -X POST http://127.0.0.1:8080/validate \
  -H 'content-type: application/json' \
  -d '{"suite":"easy"}' | jq '.metacognitive_score'

# Chat with the omni-agent
make chat-demo
```

## 📊 API Endpoints

- `POST /run` - Execute goals with metacognitive control
- `POST /validate` - Test metacognitive calibration
- `GET /dashboard` - Unified system state
- `POST /users/{id}/run` - Multi-tenant execution
- `GET /users/{id}/status` - User quota and status

## 🎛️ CLI Tools

```bash
# Direct engine interface
cd engine
./meta2-engine "add unit tests" --lm gpt-5 --trace TRACE.json

# Interactive chat
./meta2-chat demo demo-key-123 omni-1

# User management
./user-admin create alice 1000
```

## 🔬 Validation

Run the validation suite to test metacognitive calibration:

```bash
cd engine
cargo run --release &
sleep 3
curl -s -X POST http://127.0.0.1:8080/validate \
  -H 'content-type: application/json' \
  -d '{"suite":"easy"}' | jq
```

## 🏗️ Project Structure

```
├── engine/              # Main metacognitive engine
│   ├── src/
│   │   ├── engine/      # Core metacognitive engine
│   │   ├── integrations/ # System integrations
│   │   └── api.rs       # Multi-tenant API
│   ├── policies/        # L2/L3 policy definitions
│   ├── prompts/         # META_OMNI system prompts
│   └── schemas/         # Validation schemas
└── .github/workflows/   # CI/CD pipelines
```

## 🔒 Security

See [SECURITY.md](SECURITY.md) for security policy and reporting vulnerabilities.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
