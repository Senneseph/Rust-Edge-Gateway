#!/bin/bash
set -e

# Rust Edge Gateway Deployment Script for DigitalOcean Droplet
# Deploys Rust Edge Gateway via Docker and configures nginx
#
# Required environment variables (from .env or exported):
#   - SERVER_IP or DEPLOY_SERVER_IP
#   - SSH_KEY
#   - TARGET_DOMAIN
#   - DOCS_DOMAIN

REMOTE_DIR="/opt/rust-edge-gateway"
NGINX_CONF="/etc/nginx/sites-available/rust-edge-gateway"

# Load .env file if it exists
if [ -f .env ]; then
    echo "Loading .env file..."
    export $(grep -v '^#' .env | xargs)
fi

# Use DEPLOY_SERVER_IP if SERVER_IP not set
SERVER_IP="${SERVER_IP:-$DEPLOY_SERVER_IP}"

# Validate required variables
if [ -z "$SERVER_IP" ]; then
    echo "Error: SERVER_IP or DEPLOY_SERVER_IP must be set"
    exit 1
fi
if [ -z "$TARGET_DOMAIN" ]; then
    echo "Error: TARGET_DOMAIN must be set"
    exit 1
fi
if [ -z "$DOCS_DOMAIN" ]; then
    echo "Error: DOCS_DOMAIN must be set"
    exit 1
fi

echo "=== Rust Edge Gateway Deployment ==="
echo "Target Server: ${SERVER_IP}"
echo "Admin Domain:  ${TARGET_DOMAIN}"
echo "Docs Domain:   ${DOCS_DOMAIN}"
echo ""

# Create remote directory structure
echo "=== Setting up directories ==="
ssh -i "${SSH_KEY}" root@${SERVER_IP} << 'ENDSSH'
    mkdir -p /opt/rust-edge-gateway/{data,handlers,static,crates}
    mkdir -p /var/www/certbot
    mkdir -p /var/www/rust-edge-gateway-docs
ENDSSH

# Build production Docker image
echo "=== Building Docker image ==="
docker build -f Dockerfile -t rust-edge-gateway:latest .

# Save and transfer Docker image
echo "=== Transferring Docker image ==="
docker save rust-edge-gateway:latest | gzip > /tmp/rust-edge-gateway.tar.gz
scp -i "${SSH_KEY}" /tmp/rust-edge-gateway.tar.gz root@${SERVER_IP}:/tmp/
rm /tmp/rust-edge-gateway.tar.gz

# Process nginx config template - replace placeholders with actual domains
echo "=== Processing nginx config ==="
NGINX_TMP="/tmp/nginx-rust-edge-gateway.conf"
sed -e "s/{{TARGET_DOMAIN}}/${TARGET_DOMAIN}/g" \
    -e "s/{{DOCS_DOMAIN}}/${DOCS_DOMAIN}/g" \
    deploy/nginx-rust-edge-gateway-catchall.conf > "${NGINX_TMP}"

# Build documentation with mdBook
echo "=== Building documentation ==="
if command -v mdbook &> /dev/null; then
    cd docs && mdbook build && cd ..
    DOCS_BUILT=true
elif [ -d "docs/book" ]; then
    echo "mdBook not installed, using existing docs/book directory"
    DOCS_BUILT=true
else
    echo "Warning: mdBook not installed and no pre-built docs found"
    echo "Install mdBook with: cargo install mdbook"
    echo "Skipping docs deployment..."
    DOCS_BUILT=false
fi

# Copy supporting files
echo "=== Uploading files ==="
scp -i "${SSH_KEY}" -r static root@${SERVER_IP}:${REMOTE_DIR}/
scp -i "${SSH_KEY}" -r crates/rust-edge-gateway-sdk root@${SERVER_IP}:${REMOTE_DIR}/crates/
scp -i "${SSH_KEY}" deploy/docker-compose.droplet.yml root@${SERVER_IP}:${REMOTE_DIR}/docker-compose.yml
scp -i "${SSH_KEY}" "${NGINX_TMP}" root@${SERVER_IP}:${NGINX_CONF}
rm "${NGINX_TMP}"

# Upload documentation if built
if [ "$DOCS_BUILT" = true ] && [ -d "docs/book" ]; then
    echo "=== Uploading documentation ==="
    scp -i "${SSH_KEY}" -r docs/book/* root@${SERVER_IP}:/var/www/rust-edge-gateway-docs/
fi

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
    # Remove old configs if they exist
    rm -f /etc/nginx/sites-enabled/default
    rm -f /etc/nginx/sites-enabled/rust-edge-gateway-catchall

    # Enable Rust Edge Gateway config
    ln -sf /etc/nginx/sites-available/rust-edge-gateway /etc/nginx/sites-enabled/

    # Test nginx config
    nginx -t

    # Reload nginx
    systemctl reload nginx
ENDSSH

echo ""
echo "=== Deployment complete ==="
echo "Rust Edge Gateway is now running on ${SERVER_IP}"
echo ""
echo "Configured domains:"
echo "  - Admin UI: http://${TARGET_DOMAIN}/admin/"
echo "  - Docs:     http://${DOCS_DOMAIN}/"
echo ""
echo "Next steps for HTTPS:"
echo "1. SSH to server: ssh -i \"\${SSH_KEY}\" root@${SERVER_IP}"
echo "2. Get SSL certs: certbot certonly --nginx -d ${TARGET_DOMAIN} -d ${DOCS_DOMAIN}"
echo "3. Or wildcard:   certbot certonly --manual --preferred-challenges dns -d '*.iffuso.com' -d 'iffuso.com'"
echo "4. Edit ${NGINX_CONF} to enable HTTPS blocks"
echo "5. Reload nginx:  nginx -t && systemctl reload nginx"

