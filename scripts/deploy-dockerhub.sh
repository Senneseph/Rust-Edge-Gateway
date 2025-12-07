#!/bin/bash
set -e

# Rust Edge Gateway Deployment Script via Docker Hub
# Deploys to DigitalOcean Droplet using Docker Hub for image transfer
# MUCH faster than SCP because only changed layers are pushed/pulled

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Configuration
SERVER_IP="${DEPLOY_SERVER_IP}"
SSH_KEY="${SSH_KEY:-$HOME/.ssh/a-icon-deploy}"
TARGET_DOMAIN="${TARGET_DOMAIN:-rust-edge-gateway.iffuso.com}"
REMOTE_DIR="/opt/rust-edge-gateway"

# Docker Hub configuration
# Set DOCKER_HUB_USERNAME in your .env file or export it
DOCKER_HUB_USERNAME="${DOCKER_HUB_USERNAME:-senneseph}"
IMAGE_NAME="${DOCKER_HUB_USERNAME}/rust-edge-gateway"
TAG="${GIT_SHA:-$(git rev-parse --short HEAD 2>/dev/null || echo 'latest')}"

echo "=== Rust Edge Gateway Deployment via Docker Hub ==="
echo "Target: ${SERVER_IP}"
echo "Domain: ${TARGET_DOMAIN}"
echo "Image:  ${IMAGE_NAME}:${TAG}"

# Check if logged in to Docker Hub
echo "=== Checking Docker Hub authentication ==="
if ! docker info 2>/dev/null | grep -q "Username"; then
    echo "Please log in to Docker Hub first: docker login"
    exit 1
fi

# Build production image
echo "=== Building production image ==="
docker build -f Dockerfile.prod -t ${IMAGE_NAME}:${TAG} -t ${IMAGE_NAME}:latest .

# Push to Docker Hub
echo "=== Pushing to Docker Hub ==="
docker push ${IMAGE_NAME}:${TAG}
docker push ${IMAGE_NAME}:latest

# Copy supporting files to server
echo "=== Uploading configuration files ==="
scp -i "${SSH_KEY}" docker-compose.prod.yml root@${SERVER_IP}:${REMOTE_DIR}/docker-compose.yml
scp -i "${SSH_KEY}" .env root@${SERVER_IP}:${REMOTE_DIR}/.env

# Deploy on server
echo "=== Deploying on server ==="
ssh -i "${SSH_KEY}" root@${SERVER_IP} << ENDSSH
    cd ${REMOTE_DIR}
    
    # Pull the new image (only downloads changed layers!)
    docker pull ${IMAGE_NAME}:latest
    
    # Update docker-compose to use the hub image
    sed -i 's|image: rust-edge-gateway:latest|image: ${IMAGE_NAME}:latest|g' docker-compose.yml
    
    # Stop existing container
    docker-compose down || true
    
    # Start new container
    docker-compose up -d
    
    # Wait and show logs
    sleep 5
    docker-compose logs --tail 20
    
    # Clean up old images
    docker image prune -f
ENDSSH

echo "=== Deployment complete ==="
echo "Admin UI: https://${TARGET_DOMAIN}/admin/"
echo "Gateway:  https://${TARGET_DOMAIN}/"
echo ""
echo "Image pushed to: ${IMAGE_NAME}:${TAG}"

