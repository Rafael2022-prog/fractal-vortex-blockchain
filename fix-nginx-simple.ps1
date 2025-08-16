#!/usr/bin/env pwsh
# Fix Nginx Configuration Only - Simple Version
# Memperbaiki konfigurasi nginx untuk error 405 dan port 8443

Write-Host "=== FIX NGINX CONFIGURATION ONLY ===" -ForegroundColor Cyan
Write-Host "Target: 103.245.38.44" -ForegroundColor White
Write-Host "Issue: 405 Method Not Allowed + Port 8443 Error" -ForegroundColor Yellow
Write-Host ""

$ServerIP = "103.245.38.44"
$ServerUser = "root"
$ServerPassword = "a6?#PMWdik52"

# Create simple nginx configuration
Write-Host "Creating nginx configuration..." -ForegroundColor Yellow

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
    ssl_prefer_server_ciphers off;

    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;

    location /api/mining/miner/status {
        proxy_pass http://127.0.0.1:8080/miner/status;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;

        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        if ($request_method = OPTIONS) {
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
        proxy_read_timeout 86400;
        proxy_buffering off;
        proxy_cache off;

        add_header Cache-Control "no-cache";
        add_header Connection "keep-alive";
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        if ($request_method = OPTIONS) {
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

        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        if ($request_method = OPTIONS) {
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

        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        if ($request_method = OPTIONS) {
            return 204;
        }
    }

    location /api/ {
        proxy_pass http://127.0.0.1:8080/;
        rewrite ^/api/(.*)$ /$1 break;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        if ($request_method = OPTIONS) {
            return 204;
        }
    }

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
'@

$nginxConfig | Out-File -FilePath "./nginx-fix.conf" -Encoding UTF8
Write-Host "Nginx config created" -ForegroundColor Green

# Upload and apply
Write-Host "Uploading to server..." -ForegroundColor Yellow
try {
    echo $ServerPassword | pscp -pw $ServerPassword "./nginx-fix.conf" "${ServerUser}@${ServerIP}:/tmp/nginx-fix.conf"
    Write-Host "Config uploaded" -ForegroundColor Green
} catch {
    Write-Host "Upload failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host "Applying configuration..." -ForegroundColor Yellow
$commands = "cp /tmp/nginx-fix.conf /etc/nginx/conf.d/fvchain.conf && nginx -t && systemctl reload nginx"

try {
    echo $ServerPassword | plink -pw $ServerPassword "${ServerUser}@${ServerIP}" $commands
    Write-Host "Configuration applied" -ForegroundColor Green
} catch {
    Write-Host "Apply failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test
Write-Host "Testing endpoints..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/miner/status?device_id=test" -Method GET -TimeoutSec 10
    Write-Host "GET test: $($response.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "GET test failed: $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== FIX COMPLETED ===" -ForegroundColor Green
Write-Host "Test: https://fvchain.xyz/mining" -ForegroundColor Cyan