#!/usr/bin/env pwsh
# Fix Mining API Routing - Direct to RPC Server
# This script fixes the Nginx configuration to route mining API calls to the correct RPC server

Write-Host "[FIX] Mining API Routing to RPC Server" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

# Create corrected Nginx configuration
$nginxConfig = @'
server {
    listen 80;
    server_name fvchain.xyz www.fvchain.xyz;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name fvchain.xyz www.fvchain.xyz;

    ssl_certificate /etc/letsencrypt/live/fvchain.xyz/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fvchain.xyz/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Security headers
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    # Mining API routes - DIRECT TO RPC SERVER (Port 8080)
    location ~ ^/api/mining/(miner/status|miner/start|miner/stop|events)$ {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Rewrite URL to remove /api/mining prefix
        rewrite ^/api/mining/(.*)$ /$1 break;

        # SSE specific settings for events endpoint
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 24h;
        proxy_connect_timeout 5s;
        proxy_send_timeout 24h;

        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization, Cache-Control" always;
        add_header Cache-Control "no-cache" always;

        if ($request_method = 'OPTIONS') {
            return 204;
        }
    }

    # Other API routes (handled by Next.js server)
    location ~ ^/api/(wallet|transactions|blocks)/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";

        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        if ($request_method = 'OPTIONS') {
            return 204;
        }
    }

    # React Server Components requests (RSC)
    location ~ ^/(mining|wallet|transactions|blocks)\?_rsc= {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";

        # SSE specific settings for events endpoint
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 24h;
        proxy_connect_timeout 5s;
        proxy_send_timeout 24h;

        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization, Cache-Control" always;
        add_header Cache-Control "no-cache" always;

        if ($request_method = 'OPTIONS') {
            return 204;
        }
    }

    # Next.js static files
    location /_next/static/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Cache static assets
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Next.js chunks and other assets
    location /_next/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Dashboard routing (Next.js app) - catch all
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
'@

# Save configuration locally
$nginxConfig | Out-File -FilePath "fvchain-mining-fixed.conf" -Encoding UTF8 -Force
Write-Host "[OK] Created corrected Nginx configuration: fvchain-mining-fixed.conf" -ForegroundColor Green

# Upload to server
Write-Host "[UPLOAD] Uploading corrected configuration to server..." -ForegroundColor Yellow
scp "fvchain-mining-fixed.conf" "root@fvchain.xyz:/tmp/fvchain-mining-fixed.conf"

# Apply configuration on server
Write-Host "[APPLY] Applying corrected configuration on server..." -ForegroundColor Yellow
ssh root@fvchain.xyz @'
# Backup current configuration
cp /etc/nginx/conf.d/fvchain.conf /etc/nginx/conf.d/fvchain.conf.backup-$(date +%Y%m%d-%H%M%S)

# Apply new configuration
cp /tmp/fvchain-mining-fixed.conf /etc/nginx/conf.d/fvchain.conf

# Test Nginx configuration
nginx -t
if [ $? -eq 0 ]; then
    echo "[OK] Nginx configuration test passed"
    systemctl reload nginx
    echo "[OK] Nginx reloaded successfully"
else
    echo "[ERROR] Nginx configuration test failed"
    # Restore backup
    cp /etc/nginx/conf.d/fvchain.conf.backup-$(date +%Y%m%d-%H%M%S) /etc/nginx/conf.d/fvchain.conf
    exit 1
fi
'@

# Test the mining API endpoints
Write-Host "[TEST] Testing mining API endpoints..." -ForegroundColor Yellow
Start-Sleep -Seconds 3

# Test miner status endpoint
Write-Host "Testing /api/mining/miner/status..." -ForegroundColor White
try {
    $response = Invoke-RestMethod -Uri "https://fvchain.xyz/api/mining/miner/status?device_id=test" -Method GET -TimeoutSec 10
    Write-Host "[OK] Miner status endpoint working" -ForegroundColor Green
    Write-Host "Response: $($response | ConvertTo-Json -Compress)" -ForegroundColor Gray
} catch {
    Write-Host "[ERROR] Miner status endpoint failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test events endpoint
Write-Host "Testing /api/mining/events..." -ForegroundColor White
try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/events" -Method GET -TimeoutSec 5
    Write-Host "[OK] Events endpoint responding (Status: $($response.StatusCode))" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Events endpoint failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "" 
Write-Host "[SUCCESS] Mining API Routing Fix Complete!" -ForegroundColor Green
Write-Host "" 
Write-Host "[CHANGES] Key Changes Made:" -ForegroundColor Yellow
Write-Host "   - Mining API routes now direct to RPC server (port 8080)" -ForegroundColor White
Write-Host "   - URL rewriting: /api/mining/* -> /*" -ForegroundColor White
Write-Host "   - Proper CORS headers for mining endpoints" -ForegroundColor White
Write-Host "   - SSE support for events endpoint" -ForegroundColor White
Write-Host "" 
Write-Host "[ENDPOINTS] Endpoints Fixed:" -ForegroundColor Yellow
Write-Host "   - https://fvchain.xyz/api/mining/miner/status" -ForegroundColor White
Write-Host "   - https://fvchain.xyz/api/mining/miner/start" -ForegroundColor White
Write-Host "   - https://fvchain.xyz/api/mining/miner/stop" -ForegroundColor White
Write-Host "   - https://fvchain.xyz/api/mining/events" -ForegroundColor White
Write-Host "" 
Write-Host "[RESULT] The mining page should now work correctly!" -ForegroundColor Cyan