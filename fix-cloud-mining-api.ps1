#!/usr/bin/env pwsh
# Fix Cloud Mining API 405 Error
# Memperbaiki error 405 Method Not Allowed pada endpoint /api/mining/miner/status di server cloud

Write-Host "=== FIXING CLOUD MINING API 405 ERROR ===" -ForegroundColor Cyan
Write-Host "Target: https://fvchain.xyz/api/mining/miner/status" -ForegroundColor White
Write-Host ""

$ServerIP = "103.245.38.44"
$ServerUser = "root"
$ServerPassword = "FVChain2025!"

# Step 1: Upload updated RPC binary
Write-Host "1. Uploading updated RPC binary..." -ForegroundColor Yellow
try {
    echo $ServerPassword | pscp -pw $ServerPassword "./target/release/fvc-rpc.exe" "${ServerUser}@${ServerIP}:/opt/fvchain/fvc-rpc"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "RPC binary uploaded successfully" -ForegroundColor Green
    } else {
        Write-Host "Failed to upload RPC binary" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "Upload error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Step 2: Create and upload nginx config
Write-Host "2. Updating Nginx configuration..." -ForegroundColor Yellow

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

$nginxConfig | Out-File -FilePath "./temp-nginx.conf" -Encoding UTF8

try {
    echo $ServerPassword | pscp -pw $ServerPassword "./temp-nginx.conf" "${ServerUser}@${ServerIP}:/etc/nginx/conf.d/fvchain.conf"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Nginx config uploaded successfully" -ForegroundColor Green
    } else {
        Write-Host "Failed to upload nginx config" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "Nginx config upload error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Step 3: Restart services
Write-Host "3. Restarting services..." -ForegroundColor Yellow

$restartCommands = "chmod +x /opt/fvchain/fvc-rpc && systemctl stop fvc-rpc && systemctl start fvc-rpc && nginx -t && systemctl reload nginx"

try {
    echo $ServerPassword | plink -pw $ServerPassword "${ServerUser}@${ServerIP}" $restartCommands
    Write-Host "Services restarted successfully" -ForegroundColor Green
} catch {
    Write-Host "Service restart error: $($_.Exception.Message)" -ForegroundColor Red
}

# Step 4: Test endpoint
Write-Host "4. Testing endpoint..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

try {
    $testResponse = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/miner/status?device_id=test" -Method GET -TimeoutSec 10
    Write-Host "GET test successful: $($testResponse.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "GET test failed: $($_.Exception.Message)" -ForegroundColor Yellow
}

Remove-Item "./temp-nginx.conf" -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "=== CLOUD FIX COMPLETED ===" -ForegroundColor Green
Write-Host "Test: https://fvchain.xyz/mining" -ForegroundColor Cyan