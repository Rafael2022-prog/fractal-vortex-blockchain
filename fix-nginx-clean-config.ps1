#!/usr/bin/env pwsh
# Fix Nginx Configuration - Clean and Repair
# Removes duplicate API blocks and fixes corrupted headers

Write-Host "=== CLEANING NGINX CONFIGURATION ===" -ForegroundColor Green
Write-Host "Server: fvchain.xyz"
Write-Host "Target: Clean and fix Nginx configuration"
Write-Host ""

# Backup current configuration
Write-Host "1. Creating backup of current configuration..."
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
ssh root@fvchain.xyz "cp /etc/nginx/conf.d/fvchain-ssl.conf /etc/nginx/conf.d/fvchain-ssl.conf.backup.$timestamp"

# Create clean configuration
Write-Host "2. Creating clean Nginx configuration..."
$cleanConfig = @'
server {
    listen 80;
    server_name fvchain.xyz www.fvchain.xyz;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name fvchain.xyz www.fvchain.xyz;

    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/fvchain.xyz/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fvchain.xyz/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-AES128-SHA256:ECDHE-RSA-AES256-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security Headers
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # API routing to RPC server (port 8080)
    location /api/ {
        proxy_pass http://127.0.0.1:8080/;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;
        
        # CORS headers for API
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;
        
        # Handle preflight requests
        if ($request_method = OPTIONS) {
            return 204;
        }
    }

    # Proxy to Next.js app
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 86400;
    }
}
'@

# Upload clean configuration
Write-Host "3. Uploading clean configuration..."
$cleanConfig | ssh root@fvchain.xyz "cat > /etc/nginx/conf.d/fvchain-ssl.conf"

# Test configuration
Write-Host "4. Testing Nginx configuration..."
$testResult = ssh root@fvchain.xyz "nginx -t 2>&1"
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Nginx configuration test passed!" -ForegroundColor Green
    
    # Reload Nginx
    Write-Host "5. Reloading Nginx..."
    ssh root@fvchain.xyz "systemctl reload nginx"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Nginx reloaded successfully!" -ForegroundColor Green
    } else {
        Write-Host "❌ Failed to reload Nginx" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "❌ Nginx configuration test failed:" -ForegroundColor Red
    Write-Host $testResult
    
    # Restore backup
    Write-Host "6. Restoring backup configuration..."
    ssh root@fvchain.xyz "cp /etc/nginx/conf.d/fvchain-ssl.conf.backup.$timestamp /etc/nginx/conf.d/fvchain-ssl.conf"
    ssh root@fvchain.xyz "systemctl reload nginx"
    exit 1
}

# Test endpoints
Write-Host "6. Testing endpoints..."
Write-Host "Testing main site..."
try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz" -Method HEAD -TimeoutSec 10
    Write-Host "✅ Main site: $($response.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "❌ Main site test failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Testing API endpoint..."
try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/api/network/info" -Method GET -TimeoutSec 10
    Write-Host "✅ API endpoint: $($response.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "❌ API endpoint test failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Testing mining page..."
try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/mining" -Method HEAD -TimeoutSec 10
    Write-Host "✅ Mining page: $($response.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "❌ Mining page test failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== NGINX CONFIGURATION CLEANUP COMPLETED ===" -ForegroundColor Green
Write-Host "✅ Duplicate API blocks removed"
Write-Host "✅ Corrupted headers fixed"
Write-Host "✅ Clean configuration applied"
Write-Host "✅ CORS headers properly configured"
Write-Host ""
Write-Host "Device isolation should now work properly!"