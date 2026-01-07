# Test script to verify Docker setup
# This script tests the basic Docker commands to ensure the setup is working correctly

Write-Host "=== Testing Docker Setup ===" -ForegroundColor Green
Write-Host ""

# Test 1: Check if Docker is installed
Write-Host "Test 1: Checking if Docker is installed..." -ForegroundColor Cyan
try {
    docker --version
    Write-Host "✓ Docker is installed" -ForegroundColor Green
} catch {
    Write-Host "✗ Docker is not installed" -ForegroundColor Red
    Write-Host "Please install Docker first: https://docs.docker.com/get-docker/" -ForegroundColor Yellow
    exit 1
}

# Test 2: Check if docker-compose is installed
Write-Host ""
Write-Host "Test 2: Checking if docker-compose is installed..." -ForegroundColor Cyan
try {
    docker-compose --version
    Write-Host "✓ docker-compose is installed" -ForegroundColor Green
} catch {
    Write-Host "✗ docker-compose is not installed" -ForegroundColor Red
    Write-Host "Please install docker-compose: https://docs.docker.com/compose/install/" -ForegroundColor Yellow
    exit 1
}

# Test 3: Test building an image
Write-Host ""
Write-Host "Test 3: Testing Docker build..." -ForegroundColor Cyan
try {
    docker build -f Dockerfile -t rust-edge-gateway-test .
    Write-Host "✓ Docker build successful" -ForegroundColor Green
} catch {
    Write-Host "✗ Docker build failed" -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Yellow
    exit 1
}

# Test 4: Test docker-compose configuration
Write-Host ""
Write-Host "Test 4: Testing docker-compose configuration..." -ForegroundColor Cyan
try {
    docker-compose -f docker-compose.yml config
    Write-Host "✓ docker-compose configuration is valid" -ForegroundColor Green
} catch {
    Write-Host "✗ docker-compose configuration failed" -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Yellow
    exit 1
}

# Clean up test image
Write-Host ""
Write-Host "Cleaning up test image..." -ForegroundColor Cyan
docker rmi rust-edge-gateway-test -f

Write-Host ""
Write-Host "=== All Docker tests passed! ===" -ForegroundColor Green
Write-Host ""
Write-Host "Your setup is ready to use Docker." -ForegroundColor Green
Write-Host "You can now use:" -ForegroundColor Green
Write-Host "  - docker build (instead of podman build)" -ForegroundColor Green
Write-Host "  - docker-compose up (instead of podman-compose up)" -ForegroundColor Green
Write-Host "  - docker push (instead of podman push)" -ForegroundColor Green
Write-Host "  - docker pull (instead of podman pull)" -ForegroundColor Green