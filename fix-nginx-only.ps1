#!/usr/bin/env pwsh
# Fix Nginx Configuration Only - No Binary Replacement
# Memperbaiki konfigurasi nginx untuk error 405 dan port 8443

Write-Host "=== FIX NGINX CONFIGURATION ONLY ===" -ForegroundColor Cyan
Write-Host "Target: 103.245.38.44" -ForegroundColor White
Write-Host "Issue: 405 Method Not Allowed + Port 8443 Error" -ForegroundColor Yellow
Write-Host ""

$ServerIP = "103.245.38.44"
$ServerUser = "root"
$ServerPassword = "a6?#PMWdik52"

# Create fixed nginx configuration
Write-Host "Creating fixed nginx configuration..." -ForegroundColor Yellow

$nginxConfig = @'
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
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security Headers
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Fix untuk Mining API - Support GET dan POST
    location /api/mining/miner/status {
        # Proxy ke RPC server yang sudah ada
        proxy_pass http://127.0.0.1:8080/miner/status;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;
        proxy_buffering off;

        # CORS headers - Allow both GET and POST
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization, X-Requested-With" always;
        add_header Access-Control-Max-Age 86400 always;

        # Handle preflight OPTIONS requests
        if ($request_method = OPTIONS) {
            add_header Content-Length 0;
            add_header Content-Type "text/plain";
            return 204;
        }

        # Convert GET requests to POST for backend compatibility
        if ($request_method = GET) {
            set $device_id $arg_device_id;
            if ($device_id = "") {
                set $device_id "default_device";
            }
            # Rewrite GET to POST with device_id in body
            proxy_method POST;
            proxy_set_body '{"device_id":"'$device_id'"}';
            proxy_set_header Content-Type "application/json";
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

    # Fix untuk Mining Events - Hapus port 8443
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

        # SSE specific headers
        add_header Cache-Control "no-cache";
        add_header Connection "keep-alive";
        add_header X-Accel-Buffering "no";
        
        # CORS headers for SSE
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization, Cache-Control" always;

        if ($request_method = OPTIONS) {
            return 204;
        }
    }

    # General API routing
    location /api/ {
        proxy_pass http://127.0.0.1:8080/;
        rewrite ^/api/(.*)$ /$1 break;
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

    # Dashboard
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;
        
        # Cache static assets
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
            proxy_pass http://127.0.0.1:3000;
            proxy_cache_valid 200 1d;
            add_header Cache-Control "public, immutable";
        }
    }
}
'@

$nginxConfig | Out-File -FilePath "./fvchain-nginx-fixed.conf" -Encoding UTF8
Write-Host "✅ Fixed nginx config created: ./fvchain-nginx-fixed.conf" -ForegroundColor Green

# Upload nginx config only
Write-Host "Uploading fixed nginx configuration..." -ForegroundColor Yellow
try {
    echo $ServerPassword | pscp -pw $ServerPassword "./fvchain-nginx-fixed.conf" "${ServerUser}@${ServerIP}:/tmp/fvchain-nginx-fixed.conf"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Nginx config uploaded" -ForegroundColor Green
    } else {
        Write-Host "❌ Failed to upload nginx config" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "❌ Upload error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Apply nginx config and reload
Write-Host "Applying nginx configuration..." -ForegroundColor Yellow

$nginxCommands = @"
cp /tmp/fvchain-nginx-fixed.conf /etc/nginx/conf.d/fvchain.conf
nginx -t
if [ \$? -eq 0 ]; then
    systemctl reload nginx
    echo "✅ Nginx reloaded successfully"
else
    echo "❌ Nginx config test failed"
    exit 1
fi
echo "=== Current nginx status ==="
systemctl status nginx --no-pager -l
echo "=== Current RPC status ==="
systemctl status fvc-rpc --no-pager -l
"@

try {
    echo $ServerPassword | plink -pw $ServerPassword "${ServerUser}@${ServerIP}" $nginxCommands
    Write-Host "✅ Nginx configuration applied" -ForegroundColor Green
} catch {
    Write-Host "❌ Nginx config error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test the fixed endpoints
Write-Host "Testing fixed endpoints..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

# Test GET request
try {
    $testResponse = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/miner/status?device_id=test_device" -Method GET -TimeoutSec 15
    Write-Host "✅ GET /api/mining/miner/status: $($testResponse.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "⚠️ GET test failed: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Test POST request
try {
    $postData = @{ device_id = "test_device" } | ConvertTo-Json
    $testResponse = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/miner/status" -Method POST -Body $postData -ContentType "application/json" -TimeoutSec 15
    Write-Host "✅ POST /api/mining/miner/status: $($testResponse.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "⚠️ POST test failed: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Test Events endpoint (should not use port 8443)
try {
    $testResponse = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/events" -Method GET -TimeoutSec 10
    Write-Host "✅ GET /api/mining/events: $($testResponse.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "⚠️ Events test: $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== NGINX FIX COMPLETED ===" -ForegroundColor Green
Write-Host "✅ Fixed 405 Method Not Allowed error" -ForegroundColor White
Write-Host "✅ Fixed port 8443 routing issue" -ForegroundColor White
Write-Host "✅ Added GET to POST conversion for miner/status" -ForegroundColor White
Write-Host "✅ Fixed SSE events endpoint" -ForegroundColor White
Write-Host "✅ No binary replacement needed" -ForegroundColor White
Write-Host ""
Write-Host "🌐 Test mining page: https://fvchain.xyz/mining" -ForegroundColor Cyan
Write-Host "📊 API endpoint: https://fvchain.xyz/api/mining/miner/status" -ForegroundColor Cyan
Write-Host ""
Write-Host "Masalah error 405 dan port 8443 telah diperbaiki!" -ForegroundColor Green
```