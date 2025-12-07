#!/bin/bash
set -e

# Rust Edge Gateway Deployment Script for DigitalOcean Droplet
# Deploys Rust Edge Gateway via Docker and configures nginx catch-all

REMOTE_DIR="/opt/rust-edge-gateway"
NGINX_CONF="/etc/nginx/sites-available/rust-edge-gateway-catchall"

echo "=== Rust Edge Gateway Deployment ==="
echo "Target: ${SERVER_IP}"

# Create remote directory structure
echo "=== Setting up directories ==="
ssh -i "${SSH_KEY}" root@${SERVER_IP} << 'ENDSSH'
    mkdir -p /opt/rust-edge-gateway/{data,handlers,static,crates}
    mkdir -p /var/www/certbot
ENDSSH

# Build production Docker image
echo "=== Building Docker image ==="
docker build -f Dockerfile -t rust-edge-gateway:latest .

# Save and transfer Docker image
echo "=== Transferring Docker image ==="
docker save rust-edge-gateway:latest | gzip > /tmp/rust-edge-gateway.tar.gz
scp -i "${SSH_KEY}" /tmp/rust-edge-gateway.tar.gz root@${SERVER_IP}:/tmp/
rm /tmp/rust-edge-gateway.tar.gz

# Copy supporting files
echo "=== Uploading files ==="
scp -i "${SSH_KEY}" -r static root@${SERVER_IP}:${REMOTE_DIR}/
scp -i "${SSH_KEY}" -r crates/rust-edge-gateway-sdk root@${SERVER_IP}:${REMOTE_DIR}/crates/
scp -i "${SSH_KEY}" deploy/docker-compose.droplet.yml root@${SERVER_IP}:${REMOTE_DIR}/docker-compose.yml
scp -i "${SSH_KEY}" deploy/nginx-rust-edge-gateway-catchall.conf root@${SERVER_IP}:${NGINX_CONF}

# Load image and start container
echo "=== Starting Rust Edge Gateway container ==="
ssh -i "${SSH_KEY}" root@${SERVER_IP} << 'ENDSSH'
    cd /opt/rust-edge-gateway

    # Load Docker image
    docker load < /tmp/rust-edge-gateway.tar.gz
    rm /tmp/rust-edge-gateway.tar.gz

    # Stop existing container if running
    docker-compose down || true

    # Start new container
    docker-compose up -d

    # Wait and show logs
    sleep 5
    docker-compose logs --tail 20
ENDSSH

echo "=== Configuring nginx ==="
ssh -i "${SSH_KEY}" root@${SERVER_IP} << 'ENDSSH'
    # Remove old default if it exists in sites-enabled
    rm -f /etc/nginx/sites-enabled/default

    # Enable Rust Edge Gateway catch-all
    ln -sf /etc/nginx/sites-available/rust-edge-gateway-catchall /etc/nginx/sites-enabled/

    # Test nginx config
    nginx -t

    # Reload nginx
    systemctl reload nginx
ENDSSH

echo "=== Deployment complete ==="
echo "Rust Edge Gateway is now running on ${SERVER_IP}"
echo ""
echo "Next steps:"
echo "1. Get wildcard SSL cert: certbot certonly --manual --preferred-challenges dns -d '*.iffuso.com'"
echo "2. Access admin: https://rust-edge-gateway.iffuso.com/admin/"

