# Deploy documentation to DigitalOcean droplet
# Usage: .\deploy\deploy-docs.ps1

param(
    [string]$ServerIP = $env:DEPLOY_SERVER_IP,
    [string]$SshKey = "$env:USERPROFILE\.ssh\a-icon-deploy",
    [string]$TargetDomain = "rust-edge-gateway.iffuso.com",
    [string]$DocsDomain = "docs.rust-edge-gateway.iffuso.com"
)

# Load .env file if exists
if (Test-Path .env) {
    Get-Content .env | ForEach-Object {
        if ($_ -match "^([^#][^=]+)=(.*)$") {
            $name = $matches[1].Trim()
            $value = $matches[2].Trim()
            # Expand $env:USERPROFILE in the value
            $value = $value -replace '\$env:USERPROFILE', $env:USERPROFILE
            Set-Variable -Name $name -Value $value -Scope Script
            [Environment]::SetEnvironmentVariable($name, $value)
        }
    }
    $ServerIP = $env:DEPLOY_SERVER_IP
    if ($env:SSH_KEY) { $SshKey = $env:SSH_KEY -replace '\$env:USERPROFILE', $env:USERPROFILE }
    if ($env:TARGET_DOMAIN) { $TargetDomain = $env:TARGET_DOMAIN }
    if ($env:DOCS_DOMAIN) { $DocsDomain = $env:DOCS_DOMAIN }
}

Write-Host "=== Docs Deployment ===" -ForegroundColor Cyan
Write-Host "Target Server: $ServerIP"
Write-Host "SSH Key: $SshKey"
Write-Host "Target Domain: $TargetDomain"
Write-Host "Docs Domain: $DocsDomain"
Write-Host ""

# Check if docs are built
if (-not (Test-Path "docs/book")) {
    Write-Host "Building docs with mdbook..." -ForegroundColor Yellow
    Push-Location docs
    mdbook build
    Pop-Location
}

# Create directory on server
Write-Host "=== Creating docs directory ===" -ForegroundColor Cyan
ssh -i $SshKey root@$ServerIP "mkdir -p /var/www/rust-edge-gateway-docs"

# Upload documentation
Write-Host "=== Uploading documentation ===" -ForegroundColor Cyan
scp -i $SshKey -r docs/book/* root@${ServerIP}:/var/www/rust-edge-gateway-docs/

# Process nginx config - replace placeholders
Write-Host "=== Processing nginx config ===" -ForegroundColor Cyan
$nginxConfig = Get-Content "deploy/nginx-rust-edge-gateway-catchall.conf" -Raw
$nginxConfig = $nginxConfig -replace '\{\{TARGET_DOMAIN\}\}', $TargetDomain
$nginxConfig = $nginxConfig -replace '\{\{DOCS_DOMAIN\}\}', $DocsDomain
# Write without BOM
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText("$env:TEMP\nginx-rust-edge-gateway.conf", $nginxConfig, $utf8NoBom)

# Upload nginx config
Write-Host "=== Uploading nginx config ===" -ForegroundColor Cyan
scp -i $SshKey "$env:TEMP\nginx-rust-edge-gateway.conf" root@${ServerIP}:/etc/nginx/sites-available/rust-edge-gateway
Remove-Item "$env:TEMP\nginx-rust-edge-gateway.conf"

# Enable and reload nginx
Write-Host "=== Configuring nginx ===" -ForegroundColor Cyan
ssh -i $SshKey root@$ServerIP @"
ln -sf /etc/nginx/sites-available/rust-edge-gateway /etc/nginx/sites-enabled/
nginx -t && systemctl reload nginx
echo 'Nginx reloaded successfully'
"@

Write-Host ""
Write-Host "=== Docs deployment complete ===" -ForegroundColor Green
Write-Host "Docs should be available at: http://$DocsDomain/"

