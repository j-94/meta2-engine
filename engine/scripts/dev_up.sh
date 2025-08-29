#!/usr/bin/env bash
set -euo pipefail

# Load .env if present
if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
fi

: "${OPENAI_API_URL:=https://api.openai.com/v1}"
export OPENAI_API_URL

echo "Env:"; echo "  OPENAI_API_URL=${OPENAI_API_URL}"
echo "  OPENAI_API_KEY=${OPENAI_API_KEY:+(set)}"  # don't echo secret
echo
echo "Starting One Engine at http://127.0.0.1:8080 ..."
RUST_LOG=info cargo run

