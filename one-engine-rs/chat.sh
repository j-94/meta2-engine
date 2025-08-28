#!/bin/bash
# Simple chat interface for tmux

PORT=${PORT:-8080}
URL="http://localhost:$PORT"

echo "🤖 One Engine Chat (Ctrl+C to exit)"
echo "OpenAI: $(curl -s $URL/healthz | jq -r '.openai_connected // false')"
echo ""

while true; do
    echo -n "You: "
    read -r message
    
    if [[ -z "$message" ]]; then
        continue
    fi
    
    echo -n "🤖 "
    response=$(curl -s "$URL/chat" \
        -H 'content-type: application/json' \
        -d "{\"message\":\"$message\"}" | jq -r '.response // "Error"')
    
    echo "$response"
    echo ""
done
