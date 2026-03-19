#!/bin/bash
set -euo pipefail

# Install Docker Engine
apt-get update
apt-get install -y ca-certificates curl
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc

echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  tee /etc/apt/sources.list.d/docker.list > /dev/null

apt-get update
apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

systemctl enable docker
systemctl start docker

# Create app directory
mkdir -p /opt/spoons
cd /opt/spoons

# Authenticate to GHCR
echo '${ghcr_token}' | docker login ghcr.io -u '${ghcr_user}' --password-stdin

# Write .env file
cat > .env <<'ENVEOF'
DATABASE_URL=${database_url}
PODCAST_INDEX_API_KEY=${podcast_index_api_key}
PODCAST_INDEX_API_SECRET=${podcast_index_api_secret}
SUPABASE_URL=${supabase_url}
JWT_SECRET=${jwt_secret}
REDIS_URL=redis://redis:6379
ENVEOF

# Write config files (rendered by Terraform from templates/)
cat > docker-compose.prod.yml <<'COMPOSEEOF'
${docker_compose}
COMPOSEEOF

cat > Caddyfile <<'CADDYEOF'
${caddyfile}
CADDYEOF

cat > config.yaml <<'CONFIGEOF'
${config_yaml}
CONFIGEOF

# Pull and start services
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d
