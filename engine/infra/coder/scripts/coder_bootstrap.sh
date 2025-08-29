#!/usr/bin/env bash
set -euxo pipefail

# Install Docker
if ! command -v docker >/dev/null 2>&1; then
  apt-get update -y
  apt-get install -y ca-certificates curl gnupg lsb-release
  install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  chmod a+r /etc/apt/keyrings/docker.gpg
  echo \
"deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  tee /etc/apt/sources.list.d/docker.list > /dev/null
  apt-get update -y
  apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
fi

systemctl enable --now docker

# Run coder in Docker (HTTP for quick start; put TLS/ALB later)
docker pull ghcr.io/coder/coder:latest
docker rm -f coder || true
docker run -d --name coder --restart unless-stopped \
  -p 80:3000 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v coder-data:/var/lib/coder \
  ghcr.io/coder/coder:latest

echo "Coder started on port 80"

