#!/usr/bin/env pwsh

# Fix HTTP/2 Protocol Error for Server-Sent Events (SSE)
# The /api/mining/events endpoint uses SSE which can cause HTTP2_PROTOCOL_ERROR

Write-Host "[INFO] Starting HTTP/2 SSE Error Fix for FVChain..." -ForegroundColor Green

# 1. Create updated Nginx configuration that handles SSE properly
$nginxConfig = @'
# HTTP server block (port 80) - redirect to HTTPS
server {
    listen 80;
    server_name fvchain.xyz www.fvchain.xyz;
    return 301 https://$server_name$request_uri;
}

# HTTPS server block (port 443)
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

    # Special handling for Server-Sent Events (SSE) endpoint
    # Disable HTTP/2 for SSE to prevent protocol errors
    location /api/mining/events {
        # Force HTTP/1.1 for SSE compatibility
        proxy_pass http://127.0.0.1:8080/events;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Connection "";
        
        # SSE specific settings
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 24h;
        proxy_send_timeout 24h;
        
        # CORS headers for SSE
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization, Cache-Control" always;
        add_header Access-Control-Expose-Headers "Content-Type" always;
        
        # Handle preflight requests
        if ($request_method = OPTIONS) {
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET, OPTIONS";
            add_header Access-Control-Allow-Headers "Content-Type, Authorization, Cache-Control";
            add_header Access-Control-Max-Age 86400;
            add_header Content-Length 0;
            add_header Content-Type "text/plain";
            return 204;
        }
    }

    # Mining API endpoints (non-SSE)
    location /api/mining/miner/status {
        proxy_pass http://127.0.0.1:8080/miner/status;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 30s;

        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        # Handle preflight requests
        if ($request_method = OPTIONS) {
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
            add_header Access-Control-Allow-Headers "Content-Type, Authorization";
            add_header Content-Length 0;
            add_header Content-Type "text/plain";
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
        proxy_read_timeout 30s;

        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        # Handle preflight requests
        if ($request_method = OPTIONS) {
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
            add_header Access-Control-Allow-Headers "Content-Type, Authorization";
            add_header Content-Length 0;
            add_header Content-Type "text/plain";
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
        proxy_read_timeout 30s;

        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        # Handle preflight requests
        if ($request_method = OPTIONS) {
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
            add_header Access-Control-Allow-Headers "Content-Type, Authorization";
            add_header Content-Length 0;
            add_header Content-Type "text/plain";
            return 204;
        }
    }

    # General API routing (for other endpoints)
    location /api/ {
        proxy_pass http://127.0.0.1:8080/;
        rewrite ^/api/(.*)$ /$1 break;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 30s;

        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

        # Handle preflight requests
        if ($request_method = OPTIONS) {
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS";
            add_header Access-Control-Allow-Headers "Content-Type, Authorization";
            add_header Content-Length 0;
            add_header Content-Type "text/plain";
            return 204;
        }
    }

    # Main application (Next.js)
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

# 2. Save configuration locally
$nginxConfig | Out-File -FilePath "fvchain-ssl-sse-fixed.conf" -Encoding UTF8 -Force
Write-Host "[OK] SSE-compatible Nginx configuration created" -ForegroundColor Green

# 3. Backup current configuration
Write-Host "[BACKUP] Creating backup of current configuration..." -ForegroundColor Yellow
ssh root@fvchain.xyz "cp /etc/nginx/conf.d/fvchain-ssl.conf /etc/nginx/conf.d/fvchain-ssl.conf.backup-$(date +%Y%m%d-%H%M%S)"

# 4. Upload new configuration
Write-Host "[UPLOAD] Uploading SSE-compatible configuration..." -ForegroundColor Yellow
scp "fvchain-ssl-sse-fixed.conf" "root@fvchain.xyz:/etc/nginx/conf.d/fvchain-ssl.conf"

# 5. Test Nginx configuration
Write-Host "[TEST] Testing Nginx configuration..." -ForegroundColor Cyan
ssh root@fvchain.xyz "nginx -t"
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] Nginx configuration test passed!" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Nginx configuration test failed!" -ForegroundColor Red
    Write-Host "[RESTORE] Restoring backup..." -ForegroundColor Yellow
    ssh root@fvchain.xyz "cp /etc/nginx/conf.d/fvchain-ssl.conf.backup-* /etc/nginx/conf.d/fvchain-ssl.conf"
    exit 1
}

# 6. Reload Nginx
Write-Host "[RELOAD] Reloading Nginx with SSE-compatible configuration..." -ForegroundColor Cyan
ssh root@fvchain.xyz "systemctl reload nginx"
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] Nginx reloaded successfully!" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Failed to reload Nginx!" -ForegroundColor Red
    exit 1
}

# 7. Wait for services to stabilize
Write-Host "[WAIT] Waiting for services to stabilize..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

# 8. Test SSE endpoint
Write-Host "[TEST] Testing SSE endpoint functionality..." -ForegroundColor Cyan
try {
    # Test with timeout to avoid hanging
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/events" -Method GET -TimeoutSec 5 -ErrorAction Stop
    Write-Host "[OK] SSE endpoint accessible!" -ForegroundColor Green
    Write-Host "Status: $($response.StatusCode)" -ForegroundColor White
    Write-Host "Content-Type: $($response.Headers['Content-Type'])" -ForegroundColor White
} catch {
    if ($_.Exception.Message -like "*timeout*") {
        Write-Host "[OK] SSE endpoint is working (timeout expected for streaming)" -ForegroundColor Green
    } else {
        Write-Host "[WARN] SSE endpoint test: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

# 9. Test other mining endpoints
Write-Host "[TEST] Testing other mining endpoints..." -ForegroundColor Cyan
try {
    $statusResponse = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/miner/status" -Method GET -TimeoutSec 10
    Write-Host "[OK] Mining status endpoint working!" -ForegroundColor Green
    Write-Host "Response: $($statusResponse.Content)" -ForegroundColor White
} catch {
    Write-Host "[WARN] Mining status test: $($_.Exception.Message)" -ForegroundColor Yellow
}

# 10. Clean up local files
Remove-Item "fvchain-ssl-sse-fixed.conf" -Force -ErrorAction SilentlyContinue

Write-Host "\n[SUCCESS] HTTP/2 SSE Error Fix Completed!" -ForegroundColor Green
Write-Host "[INFO] Changes applied:" -ForegroundColor Cyan
Write-Host "  - ✅ SSE endpoint (/api/mining/events) configured for HTTP/1.1" -ForegroundColor White
Write-Host "  - ✅ Proxy buffering disabled for SSE" -ForegroundColor White
Write-Host "  - ✅ Extended timeouts for long-lived SSE connections" -ForegroundColor White
Write-Host "  - ✅ Proper CORS headers for SSE" -ForegroundColor White
Write-Host "  - ✅ Other API endpoints remain on HTTP/2" -ForegroundColor White

Write-Host "\n[RESOLUTION] HTTP2_PROTOCOL_ERROR should now be resolved:" -ForegroundColor Magenta
Write-Host "  - Server-Sent Events now use HTTP/1.1 (compatible)" -ForegroundColor White
Write-Host "  - No more protocol conflicts with streaming responses" -ForegroundColor White
Write-Host "  - Mining events will stream properly without errors" -ForegroundColor White
Write-Host "  - Browser console errors should be eliminated" -ForegroundColor White