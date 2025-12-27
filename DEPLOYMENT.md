# Rust Edge Gateway - Deployment Guide

This guide covers deploying Rust Edge Gateway to DigitalOcean using Docker Hub.

## Prerequisites

1. **Docker Hub Account**: Create an account at https://hub.docker.com
2. **DigitalOcean Droplet**: Ubuntu 22.04 or later with Docker installed
3. **SSH Access**: SSH key configured for droplet access
4. **Environment Variables**: Configure `.env` file (see below)

## Environment Configuration

Copy `.env.example` to `.env` and configure:

```bash
# Docker Hub
DOCKER_HUB_USERNAME=your_username
DOCKER_HUB_TOKEN=your_token

# DigitalOcean
DEPLOY_SERVER_IP=your_droplet_ip
SSH_KEY=$env:USERPROFILE\.ssh\your_key  # Windows
# SSH_KEY=$HOME/.ssh/your_key            # Linux/Mac

# Application
TARGET_DOMAIN=rust-edge-gateway.yourdomain.com
DEFAULT_ADMIN_PASSWORD=your_secure_password
```

## Deployment Process

### Step 1: Build and Publish to Docker Hub

**Windows (PowerShell):**
```powershell
.\scripts\build-and-publish.ps1
# Or with version tag:
.\scripts\build-and-publish.ps1 v1.0.0
```

**Linux/Mac (Bash):**
```bash
./scripts/build-and-publish.sh
# Or with version tag:
./scripts/build-and-publish.sh v1.0.0
```

This will:
- Build the production Docker image
- Tag it for Docker Hub
- Login to Docker Hub
- Push the image

### Step 2: Deploy to DigitalOcean Droplet

**Windows (PowerShell):**
```powershell
.\scripts\deploy-to-droplet.ps1
# Or with specific version:
.\scripts\deploy-to-droplet.ps1 v1.0.0
```

**Linux/Mac (Bash):**
```bash
./scripts/deploy-to-droplet.sh
# Or with specific version:
./scripts/deploy-to-droplet.sh v1.0.0
```

This will:
- Copy `.env` file to the droplet
- Copy `docker-compose.prod.yml` to the droplet
- Pull the image from Docker Hub
- Stop old containers
- Start new containers
- Run health checks

### Step 3: Verify Deployment

After deployment completes, access:

- **Admin UI**: https://your-domain.com/admin/
- **Gateway Health**: https://your-domain.com/health

Login with:
- Username: `admin`
- Password: (from `.env` DEFAULT_ADMIN_PASSWORD)

**⚠️ IMPORTANT**: Change your password immediately after first login!

## Manual Deployment (Alternative)

If you prefer to deploy manually on the droplet:

```bash
# SSH into droplet
ssh -i ~/.ssh/your_key root@your_droplet_ip

# Navigate to app directory
cd /opt/rust-edge-gateway

# Pull latest image
docker pull your_username/rust-edge-gateway:latest

# Stop and restart
docker-compose down
docker-compose up -d

# Check logs
docker-compose logs -f
```

## Troubleshooting

### Check Container Status
```bash
ssh -i ~/.ssh/your_key root@your_droplet_ip
docker-compose ps
docker-compose logs --tail 50
```

### Check Health Endpoints
```bash
curl http://localhost:8081/auth/login  # Should return 200
curl http://localhost:8080/health      # Should return 200
```

### Restart Containers
```bash
docker-compose restart
```

### View Real-time Logs
```bash
docker-compose logs -f
```

## API Key Management

After deployment:

1. Login to Admin UI
2. Navigate to "API Keys" section
3. Create a new API key
4. Use the API key in your requests:
   ```bash
   curl -H "Authorization: Bearer your_api_key" \
        https://your-domain.com/api/endpoints
   ```

## Nginx Configuration

Ensure your Nginx reverse proxy is configured to forward requests:

```nginx
server {
    listen 80;
    server_name rust-edge-gateway.yourdomain.com;

    location / {
        proxy_pass http://localhost:8081;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# For handler endpoints (different domains)
server {
    listen 80;
    server_name a-icon.com;

    location /api/ {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Security Notes

- Always use HTTPS in production (configure SSL certificates)
- Change default admin password immediately
- Rotate API keys regularly
- Keep Docker images updated
- Monitor logs for suspicious activity

