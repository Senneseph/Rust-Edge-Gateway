# Setup SSL certificates and deploy nginx config
# Usage: .\deploy\setup-ssl-and-deploy.ps1

param(
    [string]$ServerIP = "167.71.191.234",
    [string]$SshKey = "$env:USERPROFILE\.ssh\a-icon-deploy",
    [string]$TargetDomain = "rust-edge-gateway.iffuso.com",
    [string]$DocsDomain = "docs.rust-edge-gateway.iffuso.com"
)

Write-Host "=== SSL Setup and Deployment ===" -ForegroundColor Cyan
Write-Host "Target Server: $ServerIP"
Write-Host "Target Domain: $TargetDomain"
Write-Host "Docs Domain: $DocsDomain"
Write-Host ""

# Step 1: Get SSL certificates for new domains
Write-Host "=== Step 1: Obtaining SSL Certificates ===" -ForegroundColor Yellow
$sslCommands = @"
# Check if certs already exist
if [ ! -d /etc/letsencrypt/live/$TargetDomain ]; then
    echo 'Getting cert for $TargetDomain...'
    certbot certonly --nginx -d $TargetDomain --non-interactive --agree-tos -m admin@iffuso.com
else
    echo 'Cert for $TargetDomain already exists'
fi

if [ ! -d /etc/letsencrypt/live/$DocsDomain ]; then
    echo 'Getting cert for $DocsDomain...'
    certbot certonly --nginx -d $DocsDomain --non-interactive --agree-tos -m admin@iffuso.com
else
    echo 'Cert for $DocsDomain already exists'
fi
echo '---CERTS-DONE---'
"@

$result = ssh -i $SshKey root@$ServerIP $sslCommands 2>&1
$result | ForEach-Object { Write-Host $_ }

# Step 2: Upload updated nginx config
Write-Host ""
Write-Host "=== Step 2: Uploading Nginx Config ===" -ForegroundColor Yellow

# Process nginx config - replace placeholders
$nginxConfig = Get-Content "deploy/nginx-rust-edge-gateway-catchall.conf" -Raw
$nginxConfig = $nginxConfig -replace '\{\{TARGET_DOMAIN\}\}', $TargetDomain
$nginxConfig = $nginxConfig -replace '\{\{DOCS_DOMAIN\}\}', $DocsDomain

# Write without BOM
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText("$env:TEMP\nginx-rust-edge-gateway.conf", $nginxConfig, $utf8NoBom)

# Upload to 00-rust-edge-gateway so it loads first (before block-chan alphabetically)
scp -i $SshKey "$env:TEMP\nginx-rust-edge-gateway.conf" root@${ServerIP}:/etc/nginx/sites-available/00-rust-edge-gateway
Remove-Item "$env:TEMP\nginx-rust-edge-gateway.conf"

# Step 3: Enable and test config
Write-Host ""
Write-Host "=== Step 3: Enabling and Testing Config ===" -ForegroundColor Yellow
$enableCommands = @"
# Remove old symlink if exists
rm -f /etc/nginx/sites-enabled/rust-edge-gateway

# Create new symlink with 00- prefix so it loads first
ln -sf /etc/nginx/sites-available/00-rust-edge-gateway /etc/nginx/sites-enabled/00-rust-edge-gateway

# Test nginx config
nginx -t && systemctl reload nginx && echo 'Nginx reloaded successfully' || echo 'Nginx config test FAILED'
"@

$result = ssh -i $SshKey root@$ServerIP $enableCommands 2>&1
$result | ForEach-Object { Write-Host $_ }

Write-Host ""
Write-Host "=== Deployment Complete ===" -ForegroundColor Green
Write-Host "Test the sites:"
Write-Host "  - https://$DocsDomain/"
Write-Host "  - https://$TargetDomain/admin/"

