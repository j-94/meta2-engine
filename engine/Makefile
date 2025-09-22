build: ; cargo build --release
run: ; RUST_LOG=info cargo run
musl: ; cross build --release --target x86_64-unknown-linux-musl

# Setup local dev environment and git hooks
.PHONY: init
init:
	@echo "Setting up git hooks and Rust components..."
	@rustup component add rustfmt clippy >/dev/null 2>&1 || true
	@git config core.hooksPath .githooks
	@echo "Hooks path set to .githooks"
	@echo "Done. Commits will run fmt/clippy checks locally."

.PHONY: field-publish
field-publish:
	@python3 scripts/field_publish.py field/*.md

.PHONY: research-index
research-index: ; cargo run --bin one-research -- --root . --out research/index.jsonl --report-out research/report.json

# Meta² Chat Interface
chat: ; ./meta2-chat $(USER) $(KEY) $(THREAD)
chat-demo: ; ./meta2-chat demo demo-key-123 omni-demo
chat-premium: ; ./meta2-chat premium premium-key-456 omni-premium

# One-shot meta-prompt test
test-meta: 
	@echo "Testing meta-prompt with: $(MSG)"
	@curl -s -X POST \
		-H "x-api-key: demo-key-123" \
		-H "content-type: application/json" \
		-d "$$(jq -n --arg msg "$(MSG)" --arg sys "$$(cat prompts/META_OMNI.md)" \
			'{goal_id:"meta.omni",inputs:{thread:"test",system:$$sys,message:$$msg,history:[]},policy:null}')" \
		"http://127.0.0.1:8080/users/demo/run" | jq '.bits, .manifest.evidence.stdout'

# User management
create-user: ; ./user-admin create $(USER) $(QUOTA)
list-users: ; ./user-admin list
