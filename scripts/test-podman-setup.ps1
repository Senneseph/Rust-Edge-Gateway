# Test script to verify Podman setup
# This script tests the basic Podman commands to ensure the migration was successful

Write-Host "=== Testing Podman Setup ===" -ForegroundColor Green
Write-Host ""

# Test 1: Check if Podman is installed
Write-Host "Test 1: Checking if Podman is installed..." -ForegroundColor Cyan
try {
    podman --version
    Write-Host "✓ Podman is installed" -ForegroundColor Green
} catch {
    Write-Host "✗ Podman is not installed" -ForegroundColor Red
    Write-Host "Please install Podman first: https://podman.io/getting-started/installation" -ForegroundColor Yellow
    exit 1
}

# Test 2: Check if podman-compose is installed
Write-Host ""  
Write-Host "Test 2: Checking if podman-compose is installed..." -ForegroundColor Cyan
try {
    podman-compose --version
    Write-Host "✓ podman-compose is installed" -ForegroundColor Green
} catch {
    Write-Host "✗ podman-compose is not installed" -ForegroundColor Red
    Write-Host "Please install podman-compose: pip install podman-compose" -ForegroundColor Yellow
    exit 1
}

# Test 3: Test building an image
Write-Host ""  
Write-Host "Test 3: Testing Podman build..." -ForegroundColor Cyan
try {
    podman build -f Dockerfile -t rust-edge-gateway-test .
    Write-Host "✓ Podman build successful" -ForegroundColor Green
} catch {
    Write-Host "✗ Podman build failed" -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Yellow
    exit 1
}

# Test 4: Test podman-compose up (dry run)
Write-Host ""  
Write-Host "Test 4: Testing podman-compose configuration..." -ForegroundColor Cyan
try {
    podman-compose -f docker-compose.yml config
    Write-Host "✓ podman-compose configuration is valid" -ForegroundColor Green
} catch {
    Write-Host "✗ podman-compose configuration failed" -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Yellow
    exit 1
}

# Clean up test image
Write-Host ""  
Write-Host "Cleaning up test image..." -ForegroundColor Cyan
podman rmi rust-edge-gateway-test -f

Write-Host ""  
Write-Host "=== All Podman tests passed! ===" -ForegroundColor Green
Write-Host ""  
Write-Host "Your setup is ready to use Podman instead of Docker." -ForegroundColor Green
Write-Host "You can now use:" -ForegroundColor Green
Write-Host "  - podman build (instead of docker build)" -ForegroundColor Green
Write-Host "  - podman-compose up (instead of docker-compose up)" -ForegroundColor Green
Write-Host "  - podman push (instead of docker push)" -ForegroundColor Green
Write-Host "  - podman pull (instead of docker pull)" -ForegroundColor Green