# Rust Edge Gateway - Deploy Hello World Endpoint
# Usage: .\scripts\deploy-hello-world.ps1

$ErrorActionPreference = "Stop"

# Load .env file
if (Test-Path .env) {
    Get-Content .env | ForEach-Object {
        if ($_ -match '^([^#][^=]+)=(.*)$') {
            $name = $matches[1].Trim()
            $value = $matches[2].Trim()
            Set-Item -Path "env:$name" -Value $value
        }
    }
}

# Validate required variables
if (-not $env:RUST_EDGE_GATEWAY_API_KEY) {
    Write-Error "RUST_EDGE_GATEWAY_API_KEY not set in .env"
    exit 1
}

if (-not $env:TARGET_DOMAIN) {
    Write-Error "TARGET_DOMAIN not set in .env"
    exit 1
}

$API_KEY = $env:RUST_EDGE_GATEWAY_API_KEY
$BASE_URL = "https://$($env:TARGET_DOMAIN)"

Write-Host "=== Deploying Hello World Endpoint ===" -ForegroundColor Green
Write-Host "Target Domain: $env:TARGET_DOMAIN"
Write-Host "API Key: $($API_KEY.Substring(0, 8))..."
Write-Host ""

# Read the handler code
$handlerCode = Get-Content -Path "examples\hello-world\handler.rs" -Raw

# Step 1: Create Domain (if it doesn't exist)
Write-Host "Step 1: Creating domain 'api'..." -ForegroundColor Cyan

$domainData = @{
    name = "api"
    host = $env:TARGET_DOMAIN
    description = "API Domain"
} | ConvertTo-Json -Depth 10

try {
    $domainResponse = Invoke-RestMethod -Uri "$BASE_URL/api/domains" -Method Post -Headers @{"Authorization" = "Bearer $API_KEY"; "Content-Type" = "application/json"} -Body $domainData -ErrorAction Stop
    
    $domainId = $domainResponse.data.id
    Write-Host "Domain created successfully. ID: $domainId" -ForegroundColor Green
} catch {
    # Check if domain already exists
    $existingDomains = Invoke-RestMethod -Uri "$BASE_URL/api/domains" -Method Get -Headers @{"Authorization" = "Bearer $API_KEY"} -ErrorAction Stop
    
    $domain = $existingDomains.data | Where-Object { $_.name -eq "api" }
    if ($domain) {
        $domainId = $domain.id
        Write-Host "Domain 'api' already exists. Using ID: $domainId" -ForegroundColor Yellow
    } else {
        Write-Error "Failed to create domain: $_"
        exit 1
    }
}

# Step 2: Create Collection (if it doesn't exist)
Write-Host "" 
Write-Host "Step 2: Creating collection 'hello-world'..." -ForegroundColor Cyan

$collectionData = @{
    domain_id = $domainId
    name = "hello-world"
    description = "Hello World Collection"
    base_path = "/api"
} | ConvertTo-Json -Depth 10

try {
    $collectionResponse = Invoke-RestMethod -Uri "$BASE_URL/api/collections" -Method Post -Headers @{"Authorization" = "Bearer $API_KEY"; "Content-Type" = "application/json"} -Body $collectionData -ErrorAction Stop
    
    $collectionId = $collectionResponse.data.id
    Write-Host "Collection created successfully. ID: $collectionId" -ForegroundColor Green
} catch {
    # Check if collection already exists
    $existingCollections = Invoke-RestMethod -Uri "$BASE_URL/api/collections" -Method Get -Headers @{"Authorization" = "Bearer $API_KEY"} -ErrorAction Stop
    
    $collection = $existingCollections.data | Where-Object { $_.name -eq "hello-world" }
    if ($collection) {
        $collectionId = $collection.id
        Write-Host "Collection 'hello-world' already exists. Using ID: $collectionId" -ForegroundColor Yellow
    } else {
        Write-Error "Failed to create collection: $_"
        exit 1
    }
}

# Step 3: Create Endpoint
Write-Host "" 
Write-Host "Step 3: Creating endpoint 'hello-world'..." -ForegroundColor Cyan

$endpointData = @{
    collection_id = $collectionId
    name = "hello-world"
    domain = $env:TARGET_DOMAIN
    path = "/api/hello-world"
    method = "GET"
    description = "Hello World API Endpoint"
    code = $handlerCode
} | ConvertTo-Json -Depth 10

try {
    $endpointResponse = Invoke-RestMethod -Uri "$BASE_URL/api/endpoints" -Method Post -Headers @{"Authorization" = "Bearer $API_KEY"; "Content-Type" = "application/json"} -Body $endpointData -ErrorAction Stop
    
    $endpointId = $endpointResponse.data.id
    Write-Host "Endpoint created successfully. ID: $endpointId" -ForegroundColor Green
} catch {
    # Check if endpoint already exists
    $existingEndpoints = Invoke-RestMethod -Uri "$BASE_URL/api/endpoints" -Method Get -Headers @{"Authorization" = "Bearer $API_KEY"} -ErrorAction Stop
    
    $endpoint = $existingEndpoints.data | Where-Object { $_.name -eq "hello-world" }
    if ($endpoint) {
        $endpointId = $endpoint.id
        Write-Host "Endpoint 'hello-world' already exists. Using ID: $endpointId" -ForegroundColor Yellow
        
        # Update the endpoint code if it exists
        Write-Host "Updating endpoint code..." -ForegroundColor Cyan
        $updateCodeData = @{
            code = $handlerCode
        } | ConvertTo-Json -Depth 10
        
        Invoke-RestMethod -Uri "$BASE_URL/api/endpoints/$endpointId/code" -Method Put -Headers @{"Authorization" = "Bearer $API_KEY"; "Content-Type" = "application/json"} -Body $updateCodeData -ErrorAction Stop | Out-Null
        
        Write-Host "Endpoint code updated successfully." -ForegroundColor Green
    } else {
        Write-Error "Failed to create endpoint: $_"
        exit 1
    }
}

# Step 4: Compile the Endpoint
Write-Host "" 
Write-Host "Step 4: Compiling endpoint..." -ForegroundColor Cyan

try {
    $compileResponse = Invoke-RestMethod -Uri "$BASE_URL/api/endpoints/$endpointId/compile" -Method Post -Headers @{"Authorization" = "Bearer $API_KEY"} -ErrorAction Stop
    
    Write-Host "Endpoint compiled successfully: $($compileResponse.data)" -ForegroundColor Green
} catch {
    Write-Error "Failed to compile endpoint: $_"
    exit 1
}

# Step 5: Start the Endpoint
Write-Host "" 
Write-Host "Step 5: Starting endpoint..." -ForegroundColor Cyan

try {
    $startResponse = Invoke-RestMethod -Uri "$BASE_URL/api/endpoints/$endpointId/start" -Method Post -Headers @{"Authorization" = "Bearer $API_KEY"} -ErrorAction Stop
    
    Write-Host "Endpoint started successfully." -ForegroundColor Green
} catch {
    Write-Error "Failed to start endpoint: $_"
    exit 1
}

# Step 6: Test the Endpoint
Write-Host "" 
Write-Host "Step 6: Testing endpoint..." -ForegroundColor Cyan

try {
    $testResponse = Invoke-RestMethod -Uri "https://$($env:TARGET_DOMAIN)/api/hello-world" -Method Get -Headers @{"Authorization" = "Bearer $API_KEY"} -ErrorAction Stop
    
    Write-Host "Endpoint test successful!" -ForegroundColor Green
    Write-Host "Response: $($testResponse | ConvertTo-Json -Depth 10)" -ForegroundColor Cyan
} catch {
    Write-Error "Failed to test endpoint: $_"
    exit 1
}

Write-Host "" 
Write-Host "=== Hello World Endpoint Deployment Complete ===" -ForegroundColor Green
Write-Host ""
Write-Host "Your endpoint is now available at:" -ForegroundColor Green
Write-Host "  $BASE_URL/api/hello-world" -ForegroundColor Cyan
Write-Host ""
Write-Host "To test it manually:" -ForegroundColor Green
Write-Host "  curl -H \"Authorization: Bearer $API_KEY\" $BASE_URL/api/hello-world" -ForegroundColor Cyan