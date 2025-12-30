# Script to update API key permissions in the database
# This is a temporary workaround to give the API key the right permissions

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

if (-not $env:RUST_EDGE_GATEWAY_API_KEY) {
    Write-Error "RUST_EDGE_GATEWAY_API_KEY not set in .env"
    exit 1
}

if (-not $env:DEFAULT_ADMIN_PASSWORD) {
    Write-Error "DEFAULT_ADMIN_PASSWORD not set in .env"
    exit 1
}

# Find the admin database
$adminDbPath = Join-Path -Path $PWD -ChildPath "data/admin.db"

if (-not (Test-Path $adminDbPath)) {
    Write-Error "Admin database not found at $adminDbPath"
    exit 1
}

Write-Host "Updating API key permissions..." -ForegroundColor Green

# The permissions we need for our deployment script
$requiredPermissions = @("domains:*", "collections:*", "endpoints:*")
$permissionsJson = $requiredPermissions | ConvertTo-Json -Compress

try {
    # Use SQLite to update the API key permissions
    $sqliteProcess = Start-Process -FilePath "sqlite3" -ArgumentList @(
        $adminDbPath,
        "UPDATE api_keys SET permissions = '$permissionsJson' WHERE key = '$($env:RUST_EDGE_GATEWAY_API_KEY)'"
    ) -NoNewWindow -Wait -PassThru
    
    if ($sqliteProcess.ExitCode -eq 0) {
        Write-Host "API key permissions updated successfully!" -ForegroundColor Green
        Write-Host "Permissions: $($requiredPermissions -join ', ')" -ForegroundColor Cyan
    } else {
        Write-Error "Failed to update API key permissions"
        exit 1
    }
} catch {
    Write-Error "Failed to update API key permissions: $_"
    exit 1
}

Write-Host "You can now run the deployment script." -ForegroundColor Green