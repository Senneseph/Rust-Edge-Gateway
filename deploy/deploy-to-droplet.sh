#!/bin/bash
set -e

# Edge Hive Deployment Script for DigitalOcean Droplet
# Deploys Edge Hive via Docker and configures nginx catch-all

REMOTE_DIR="/opt/edge-hive"
NGINX_CONF="/etc/nginx/sites-available/edge-hive-catchall"

echo "=== Edge Hive Deployment ==="
echo "Target: ${SERVER_IP}"

# Create remote directory structure
echo "=== Setting up directories ==="
ssh -i "${SSH_KEY}" root@${SERVER_IP} << 'ENDSSH'
    mkdir -p /opt/edge-hive/{data,handlers,static,crates}
    mkdir -p /var/www/certbot
ENDSSH

# Build production Docker image
echo "=== Building Docker image ==="
docker build -f Dockerfile -t edge-hive:latest .

# Save and transfer Docker image
echo "=== Transferring Docker image ==="
docker save edge-hive:latest | gzip > /tmp/edge-hive.tar.gz
scp -i "${SSH_KEY}" /tmp/edge-hive.tar.gz root@${SERVER_IP}:/tmp/
rm /tmp/edge-hive.tar.gz

# Copy supporting files
echo "=== Uploading files ==="
scp -i "${SSH_KEY}" -r static root@${SERVER_IP}:${REMOTE_DIR}/
scp -i "${SSH_KEY}" -r crates/edge-hive-sdk root@${SERVER_IP}:${REMOTE_DIR}/crates/
scp -i "${SSH_KEY}" deploy/docker-compose.droplet.yml root@${SERVER_IP}:${REMOTE_DIR}/docker-compose.yml
scp -i "${SSH_KEY}" deploy/nginx-edge-hive-catchall.conf root@${SERVER_IP}:${NGINX_CONF}

# Load image and start container
echo "=== Starting Edge Hive container ==="
ssh -i "${SSH_KEY}" root@${SERVER_IP} << 'ENDSSH'
    cd /opt/edge-hive

    # Load Docker image
    docker load < /tmp/edge-hive.tar.gz
    rm /tmp/edge-hive.tar.gz

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

    # Enable Edge Hive catch-all
    ln -sf /etc/nginx/sites-available/edge-hive-catchall /etc/nginx/sites-enabled/

    # Test nginx config
    nginx -t

    # Reload nginx
    systemctl reload nginx
ENDSSH

echo "=== Deployment complete ==="
echo "Edge Hive is now running on ${SERVER_IP}"
echo ""
echo "Next steps:"
echo "1. Get wildcard SSL cert: certbot certonly --manual --preferred-challenges dns -d '*.iffuso.com'"
echo "2. Access admin: https://edge-hive.iffuso.com/admin/"

