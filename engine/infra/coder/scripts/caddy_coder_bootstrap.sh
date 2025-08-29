#!/usr/bin/env bash
set -euxo pipefail

DOMAIN="${domain}"
EMAIL="${email}"

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

# Run coder on 3000
docker pull ghcr.io/coder/coder:latest
docker rm -f coder || true
docker run -d --name coder --restart unless-stopped \
  -p 3000:3000 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v coder-data:/var/lib/coder \
  ghcr.io/coder/coder:latest

# Install Caddy (reverse proxy + TLS)
if ! command -v caddy >/dev/null 2>&1; then
  apt-get install -y debian-keyring debian-archive-keyring apt-transport-https
  curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
  curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list
  apt-get update -y
  apt-get install -y caddy
fi

mkdir -p /etc/caddy

if [ -n "$DOMAIN" ]; then
  cat >/etc/caddy/Caddyfile <<CFG
{
  email $EMAIL
}
$DOMAIN {
  reverse_proxy 127.0.0.1:3000
}
CFG
else
  # HTTP only fallback on port 80
  cat >/etc/caddy/Caddyfile <<CFG
:80 {
  reverse_proxy 127.0.0.1:3000
}
CFG
fi

systemctl enable --now caddy
echo "Coder running on 3000, Caddy proxy on 80/443 (domain: ${DOMAIN:-none})"

