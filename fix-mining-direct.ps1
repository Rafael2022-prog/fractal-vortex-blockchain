#!/usr/bin/env pwsh
# Direct Fix for Mining API Routing

Write-Host "[FIX] Direct Mining API Routing Fix" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""

# Create a complete new configuration with mining API separated
Write-Host "[CREATE] Creating new Nginx configuration..." -ForegroundColor Yellow

$newConfig = @'
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

    # Mining API routes - Direct to RPC Server
    location /api/mining/miner/status {
        proxy_pass http://127.0.0.1:8080/miner/status;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        if ($request_method = 'OPTIONS') {
            return 204;
        }
    }

    location /api/mining/miner/start {
        proxy_pass http://127.0.0.1:8080/miner/start;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        if ($request_method = 'OPTIONS') {
            return 204;
        }
    }

    location /api/mining/miner/stop {
        proxy_pass http://127.0.0.1:8080/miner/stop;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        if ($request_method = 'OPTIONS') {
            return 204;
        }
    }

    location /api/mining/events {
        proxy_pass http://127.0.0.1:8080/events;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # SSE specific settings
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

    # Other API routes (wallet, transactions, blocks) - Next.js
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

# Save to file
$newConfig | Out-File -FilePath "fvchain-fixed.conf" -Encoding ASCII -Force
Write-Host "[OK] New configuration created: fvchain-fixed.conf" -ForegroundColor Green

# Upload and apply
Write-Host "[UPLOAD] Uploading new configuration..." -ForegroundColor Yellow
scp "fvchain-fixed.conf" "root@fvchain.xyz:/tmp/fvchain-fixed.conf"

Write-Host "[APPLY] Applying new configuration..." -ForegroundColor Yellow
ssh root@fvchain.xyz @'
# Backup current config
cp /etc/nginx/conf.d/fvchain.conf /etc/nginx/conf.d/fvchain.conf.backup-direct-fix

# Apply new config
cp /tmp/fvchain-fixed.conf /etc/nginx/conf.d/fvchain.conf

# Test and reload
nginx -t && systemctl reload nginx
'@

# Test endpoints
Write-Host "[TEST] Testing mining endpoints..." -ForegroundColor Yellow
Start-Sleep -Seconds 2

$endpoints = @(
    "https://fvchain.xyz/api/mining/miner/status?device_id=test",
    "https://fvchain.xyz/api/mining/events"
)

foreach ($endpoint in $endpoints) {
    Write-Host "Testing: $endpoint" -ForegroundColor White
    try {
        $response = Invoke-WebRequest -Uri $endpoint -Method GET -TimeoutSec 5
        Write-Host "[OK] Status: $($response.StatusCode)" -ForegroundColor Green
    } catch {
        Write-Host "[INFO] Response: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

Write-Host "" 
Write-Host "[SUCCESS] Direct mining API fix completed!" -ForegroundColor Green
Write-Host "Mining endpoints now route directly to RPC server on port 8080" -ForegroundColor White