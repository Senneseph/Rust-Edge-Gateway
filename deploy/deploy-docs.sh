#!/bin/bash
# Deploy just the documentation site
# Run this from Windows with: wsl bash deploy/deploy-docs.sh

set -e

# Load .env file if it exists
if [ -f .env ]; then
    echo "Loading .env file..."
    export $(grep -v '^#' .env | xargs)
fi

# Use DEPLOY_SERVER_IP if SERVER_IP not set
SERVER_IP="${SERVER_IP:-$DEPLOY_SERVER_IP}"

# Convert Windows path to WSL path if needed
if [[ "$SSH_KEY" == *"env:USERPROFILE"* ]]; then
    SSH_KEY=$(echo "$SSH_KEY" | sed 's/\$env:USERPROFILE/\/mnt\/c\/Users\/blank-banshee/g')
fi

echo "=== Docs Deployment ==="
echo "Target Server: ${SERVER_IP}"
echo "Docs Domain:   ${DOCS_DOMAIN}"
echo "SSH Key:       ${SSH_KEY}"
echo ""

# Ensure docs are built
if [ ! -d "docs/book" ]; then
    echo "Error: docs/book directory not found. Run 'mdbook build' in docs/ first."
    exit 1
fi

# Create directory on server
echo "=== Creating docs directory ==="
ssh -i "${SSH_KEY}" root@${SERVER_IP} "mkdir -p /var/www/rust-edge-gateway-docs"

# Upload documentation
echo "=== Uploading documentation ==="
scp -i "${SSH_KEY}" -r docs/book/* root@${SERVER_IP}:/var/www/rust-edge-gateway-docs/

# Process and upload nginx config
echo "=== Updating nginx config ==="
NGINX_TMP="/tmp/nginx-rust-edge-gateway.conf"
sed -e "s/{{TARGET_DOMAIN}}/${TARGET_DOMAIN}/g" \
    -e "s/{{DOCS_DOMAIN}}/${DOCS_DOMAIN}/g" \
    deploy/nginx-rust-edge-gateway-catchall.conf > "${NGINX_TMP}"

scp -i "${SSH_KEY}" "${NGINX_TMP}" root@${SERVER_IP}:/etc/nginx/sites-available/rust-edge-gateway
rm "${NGINX_TMP}"

# Enable and reload nginx
echo "=== Configuring nginx ==="
ssh -i "${SSH_KEY}" root@${SERVER_IP} << 'ENDSSH'
    # Enable Rust Edge Gateway config
    ln -sf /etc/nginx/sites-available/rust-edge-gateway /etc/nginx/sites-enabled/
    
    # Test nginx config
    nginx -t
    
    # Reload nginx
    systemctl reload nginx
    
    echo "Nginx reloaded successfully"
ENDSSH

echo ""
echo "=== Docs deployment complete ==="
echo "Docs should be available at: http://${DOCS_DOMAIN}/"

