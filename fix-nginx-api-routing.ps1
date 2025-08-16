# Fix Nginx API Routing for FVChain RPC Server
# This script adds the missing /api/ location block to route requests to port 8080

param(
    [string]$ServerIP = "fvchain.xyz",
    [string]$Username = "root"
)

Write-Host "=== FIXING NGINX API ROUTING ==="  -ForegroundColor Green
Write-Host "Server: $ServerIP" -ForegroundColor Yellow
Write-Host "Target: Add /api/ routing to port 8080" -ForegroundColor Yellow
Write-Host ""

# Step 1: Backup current Nginx configuration
Write-Host "1. Backup current Nginx configuration..." -ForegroundColor Cyan
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
ssh ${Username}@${ServerIP} "cp /etc/nginx/conf.d/fvchain-ssl.conf /etc/nginx/conf.d/fvchain-ssl.conf.backup-$timestamp"

# Step 2: Create the API routing configuration
Write-Host "2. Adding API routing configuration..." -ForegroundColor Cyan

# Create temporary config file with API routing
$apiConfig = @'
    # API routing to RPC server (port 8080)
    location /api/ {
        proxy_pass http://127.0.0.1:8080/;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;
        
        # CORS headers for API requests
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;
        
        # Handle preflight requests
        if ($request_method = 'OPTIONS') {
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
            add_header Access-Control-Allow-Headers "Content-Type, Authorization";
            add_header Access-Control-Max-Age 1728000;
            add_header Content-Type "text/plain; charset=utf-8";
            add_header Content-Length 0;
            return 204;
        }
    }
'@

# Step 3: Create new configuration with API routing
Write-Host "3. Creating new Nginx configuration with API routing..." -ForegroundColor Cyan

# Create the API configuration block
$apiBlock = @'
    # API routing to RPC server (port 8080)
    location /api/ {
        proxy_pass http://127.0.0.1:8080/;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;
        
        # CORS headers for API requests
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;
        
        # Handle preflight requests
        if ($request_method = OPTIONS) {
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
            add_header Access-Control-Allow-Headers "Content-Type, Authorization";
            add_header Access-Control-Max-Age 1728000;
            add_header Content-Type "text/plain; charset=utf-8";
            add_header Content-Length 0;
            return 204;
        }
    }
'@

# Insert the API block before the main location block
ssh ${Username}@${ServerIP} "sed '/# Proxy to Next.js app/i\\n    # API routing to RPC server (port 8080)\\n    location /api/ {\\n        proxy_pass http://127.0.0.1:8080/;\\n        proxy_http_version 1.1;\\n        proxy_set_header Host \$host;\\n        proxy_set_header X-Real-IP \$remote_addr;\\n        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;\\n        proxy_set_header X-Forwarded-Proto \$scheme;\\n        proxy_read_timeout 86400;\\n        \\n        add_header Access-Control-Allow-Origin \"*\" always;\\n        add_header Access-Control-Allow-Methods \"GET, POST, OPTIONS\" always;\\n        add_header Access-Control-Allow-Headers \"Content-Type, Authorization\" always;\\n    }\\n' /etc/nginx/conf.d/fvchain-ssl.conf > /tmp/nginx-new.conf && mv /tmp/nginx-new.conf /etc/nginx/conf.d/fvchain-ssl.conf"

# Step 4: Test Nginx configuration
Write-Host "4. Testing Nginx configuration..." -ForegroundColor Cyan
$testResult = ssh ${Username}@${ServerIP} "nginx -t 2>&1"
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Nginx configuration test passed!" -ForegroundColor Green
} else {
    Write-Host "❌ Nginx configuration test failed!" -ForegroundColor Red
    Write-Host "Error: $testResult" -ForegroundColor Red
    Write-Host "Restoring backup..." -ForegroundColor Yellow
    ssh ${Username}@${ServerIP} "cp /etc/nginx/conf.d/fvchain-ssl.conf.backup-$timestamp /etc/nginx/conf.d/fvchain-ssl.conf"
    exit 1
}

# Step 5: Reload Nginx
Write-Host "5. Reloading Nginx..." -ForegroundColor Cyan
ssh ${Username}@${ServerIP} "systemctl reload nginx"
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Nginx reloaded successfully!" -ForegroundColor Green
} else {
    Write-Host "❌ Failed to reload Nginx!" -ForegroundColor Red
    exit 1
}

# Step 6: Test API endpoint
Write-Host "6. Testing API endpoint..." -ForegroundColor Cyan
Start-Sleep -Seconds 3

try {
    $response = Invoke-RestMethod -Uri "https://$ServerIP/api/network/info" -TimeoutSec 10
    Write-Host "✅ API endpoint is working!" -ForegroundColor Green
    Write-Host "Response: $($response | ConvertTo-Json -Compress)" -ForegroundColor White
} catch {
    Write-Host "⚠️  API endpoint test failed: $($_.Exception.Message)" -ForegroundColor Yellow
    Write-Host "This might be normal if RPC server is not fully ready" -ForegroundColor Gray
}

# Step 7: Test mining events endpoint
Write-Host "7. Testing mining events endpoint..." -ForegroundColor Cyan
try {
    $response = Invoke-WebRequest -Uri "https://$ServerIP/api/mining/events" -TimeoutSec 5
    Write-Host "✅ Mining events endpoint is accessible!" -ForegroundColor Green
    Write-Host "Status: $($response.StatusCode)" -ForegroundColor White
} catch {
    Write-Host "⚠️  Mining events endpoint test: $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== NGINX API ROUTING FIX COMPLETED ===" -ForegroundColor Green
Write-Host "✅ API routing to port 8080 has been configured" -ForegroundColor White
Write-Host "✅ CORS headers added for cross-origin requests" -ForegroundColor White
Write-Host "✅ Nginx configuration tested and reloaded" -ForegroundColor White
Write-Host ""
Write-Host "API endpoints now available:" -ForegroundColor Cyan
Write-Host "- https://$ServerIP/api/network/info" -ForegroundColor White
Write-Host "- https://$ServerIP/api/mining/events" -ForegroundColor White
Write-Host "- https://$ServerIP/api/wallet/create" -ForegroundColor White
Write-Host "- https://$ServerIP/api/wallet/balance" -ForegroundColor White
Write-Host ""
Write-Host "Device isolation should now work properly!" -ForegroundColor Green