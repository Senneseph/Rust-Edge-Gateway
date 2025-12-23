# Rust Edge Gateway Deployment Instructions

## Prerequisites

- Rust toolchain installed (`rustup`)
- Docker installed and running
- SSH access to DigitalOcean Droplet
- `.env` file with correct credentials

## Deployment Steps

### 1. Build the Production Binary

```bash
# Build the Rust application in release mode
 cargo build --release --manifest-path crates/rust-edge-gateway/Cargo.toml
```

### 2. Prepare Environment Variables

Create a `.env` file from the template and fill in your values:

```bash
cp .env.example .env
# Edit .env with your actual values
```

Required environment variables:
- `DEFAULT_ADMIN_PASSWORD` - Initial admin password (will require change on first login)
- `DIGITALOCEAN_ACCESS_TOKEN` - Your DigitalOcean API token
- `DEPLOY_SERVER_IP` - Your server's IP address
- `TARGET_DOMAIN` - Your domain name
- `DOCS_DOMAIN` - Your documentation domain
- `SSH_KEY` - Path to your SSH key

The `DEFAULT_ADMIN_PASSWORD` will be used to create the initial admin user on first startup and will require a password change on first login.

### 2. Build the Production Docker Image

```bash
# Build the Docker image using the production Dockerfile
 docker build -t rust-edge-gateway:prod -f Dockerfile.prod .
```

### 3. Copy Files to DigitalOcean Droplet

```bash
# Copy the built binary and docker-compose file to the server
scp -i "$SSH_KEY" target/release/rust-edge-gateway root@$DEPLOY_SERVER_IP:/opt/rust-edge-gateway/
scp -i "$SSH_KEY" docker-compose.prod.yml root@$DEPLOY_SERVER_IP:/opt/rust-edge-gateway/
scp -i "$SSH_KEY" .env root@$DEPLOY_SERVER_IP:/opt/rust-edge-gateway/
```

### 4. Connect to the Droplet and Deploy

```bash
# Connect to the DigitalOcean Droplet
ssh -i "$SSH_KEY" root@$DEPLOY_SERVER_IP
```

Once connected to the server, run the following commands:

```bash
# Navigate to the deployment directory
cd /opt/rust-edge-gateway

# Stop the current service (if running)
docker-compose -f docker-compose.prod.yml down

# Pull the latest image (if needed)
docker pull rust-edge-gateway:prod

# Start the service with the production configuration
docker-compose -f docker-compose.prod.yml up -d

# Check the service status
docker-compose -f docker-compose.prod.yml logs -f
```

### 5. Verify Deployment

Visit the application at: https://${TARGET_DOMAIN}/

Check that:
- The application is responding
- All services are running
- The correct version is deployed
- Login with username `admin` and the password from `DEFAULT_ADMIN_PASSWORD`
- You are prompted to change your password on first login (security requirement)

## Troubleshooting

If the deployment fails:

1. Check Docker logs: `docker-compose -f docker-compose.prod.yml logs`
2. Verify the binary has execute permissions: `chmod +x /opt/rust-edge-gateway/rust-edge-gateway`
3. Ensure the .env file has the correct environment variables
4. Verify network connectivity to required services (database, storage, etc.)

## Rollback Procedure

To rollback to a previous version:

1. Stop the current service: `docker-compose -f docker-compose.prod.yml down`
2. Restore the previous docker-compose.prod.yml and .env files from backup
3. Restart the service: `docker-compose -f docker-compose.prod.yml up -d`

> **Note**: The SSH key path in the instructions assumes the key is stored at `$HOME/.ssh/a-icon-deploy`. If your key is stored elsewhere, update the paths accordingly.

## Docker Hub Publishing

### Prerequisites

1. Docker Hub account (sign up at https://hub.docker.com/)
2. Docker Hub repository created for this project
3. Docker Hub access token (create in Account Settings > Security)
4. Docker installed and running locally

### Setup

1. Add your Docker Hub username to the `.env` file:
   ```
   DOCKER_HUB_USERNAME=your_dockerhub_username
   ```

2. Log in to Docker Hub:
   ```bash
   docker login
   ```
   Enter your Docker Hub username and access token when prompted.

### Building the Docker Image

Use the provided build script:

```bash
# Build with default version (latest)
./scripts/build-docker.sh

# Build with specific version
./scripts/build-docker.sh v1.0.0
```

### Publishing to Docker Hub

Use the provided publish script:

```bash
# Publish with default version (latest)
./scripts/publish-docker.sh your_dockerhub_username

# Publish with specific version
./scripts/publish-docker.sh v1.0.0 your_dockerhub_username
```

### Using the Published Image

1. Pull the image from Docker Hub:
   ```bash
   docker pull your_dockerhub_username/rust-edge-gateway:latest
   ```

2. Update your `docker-compose.prod.yml` to use the published image:
   ```yaml
   services:
     rust-edge-gateway:
       image: your_dockerhub_username/rust-edge-gateway:latest
       # ... rest of configuration
   ```

3. Start the service:
   ```bash
   docker-compose -f docker-compose.prod.yml up -d
   ```

### Automated CI/CD (Optional)

For automated builds, you can set up a GitHub Actions workflow or use Docker Hub's automated builds feature.

Example GitHub Actions workflow (`.github/workflows/docker-publish.yml`):

```yaml
name: Docker Publish

on:
  push:
    tags:
      - 'v*'

jobs:
  build-and-push:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Login to Docker Hub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKER_HUB_USERNAME }}
          password: ${{ secrets.DOCKER_HUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          file: Dockerfile.prod
          push: true
          tags: ${{ secrets.DOCKER_HUB_USERNAME }}/rust-edge-gateway:${{ github.ref_name }}
```

### Best Practices

1. **Version Tagging**: Always use semantic versioning (e.g., v1.0.0) for production releases
2. **Multi-arch Builds**: Consider building for multiple architectures (amd64, arm64) for broader compatibility
3. **Image Optimization**: The production Dockerfile already uses multi-stage builds for minimal image size
4. **Security**: Keep your Docker Hub access token secure and use it only in trusted environments