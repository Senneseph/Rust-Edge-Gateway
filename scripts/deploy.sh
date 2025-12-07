#!/bin/bash
set -e

# Edge Hive Deployment Script
# Deploys to DigitalOcean Droplet

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Configuration
SERVER_IP="${DEPLOY_SERVER_IP}"
SSH_KEY="${SSH_KEY:-$HOME/.ssh/a-icon-deploy}"
TARGET_DOMAIN="${TARGET_DOMAIN:-edge-hive.iffuso.com}"
REMOTE_DIR="/opt/edge-hive"

echo "=== Edge Hive Deployment ==="
echo "Target: ${SERVER_IP}"
echo "Domain: ${TARGET_DOMAIN}"

# Build production image locally
echo "=== Building production image ==="
docker build -f Dockerfile.prod -t edge-hive:latest .

# Save image to tarball
echo "=== Exporting image ==="
docker save edge-hive:latest | gzip > edge-hive.tar.gz

# Copy to server
echo "=== Uploading to server ==="
scp -i "${SSH_KEY}" edge-hive.tar.gz root@${SERVER_IP}:/tmp/
scp -i "${SSH_KEY}" docker-compose.prod.yml root@${SERVER_IP}:${REMOTE_DIR}/docker-compose.yml
scp -i "${SSH_KEY}" .env root@${SERVER_IP}:${REMOTE_DIR}/.env

# Deploy on server
echo "=== Deploying on server ==="
ssh -i "${SSH_KEY}" root@${SERVER_IP} << 'ENDSSH'
    cd /opt/edge-hive
    
    # Load the image
    docker load < /tmp/edge-hive.tar.gz
    rm /tmp/edge-hive.tar.gz
    
    # Stop existing container
    docker-compose down || true
    
    # Start new container
    docker-compose up -d
    
    # Show logs
    sleep 5
    docker-compose logs --tail 20
ENDSSH

# Cleanup local tarball
rm edge-hive.tar.gz

echo "=== Deployment complete ==="
echo "Admin UI: https://${TARGET_DOMAIN}/admin/"
echo "Gateway:  https://${TARGET_DOMAIN}/"

