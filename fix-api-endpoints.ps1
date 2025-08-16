#!/usr/bin/env pwsh
# Fix API Endpoints Configuration for FVChain
# This script fixes the Nginx proxy configuration to match the correct API endpoints

Write-Host "=== FVChain API Endpoints Fix ===" -ForegroundColor Green
Write-Host "Fixing Nginx proxy configuration for correct API endpoints..." -ForegroundColor Yellow

# Test correct API endpoints first
Write-Host "\n1. Testing correct API endpoints..." -ForegroundColor Cyan
ssh root@103.245.38.44 "curl -s http://127.0.0.1:8080/miner/status"
Write-Host "\n"
ssh root@103.245.38.44 "curl -s http://127.0.0.1:8080/events"

# Backup current configuration
Write-Host "\n2. Backing up current Nginx configuration..." -ForegroundColor Cyan
ssh root@103.245.38.44 "cp /etc/nginx/conf.d/fvchain-letsencrypt.conf /etc/nginx/conf.d/fvchain-letsencrypt.conf.backup-$(date +%Y%m%d-%H%M%S)"

# Create corrected Nginx configuration
Write-Host "\n3. Creating corrected Nginx configuration..." -ForegroundColor Cyan

$nginxConfig = @'
server {
    listen 80;
    server_name fvchain.xyz;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name fvchain.xyz;

    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/fvchain.xyz/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fvchain.xyz/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-AES128-SHA256:ECDHE-RSA-AES256-SHA384;
    ssl_prefer_server_ciphers on;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Dashboard (port 3001)
    location / {
        proxy_pass http://127.0.0.1:3001/;
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

    # API (port 8080) - Fixed endpoints
    location /api/ {
        # Remove /api prefix and proxy to correct endpoints
        rewrite ^/api/(.*)$ /$1 break;
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;
        
        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;
        
        # Handle preflight requests
        if ($request_method = 'OPTIONS') {
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
            add_header Access-Control-Allow-Headers "Content-Type, Authorization";
            add_header Content-Length 0;
            add_header Content-Type text/plain;
            return 204;
        }
    }

    # Direct API access (without /api prefix)
    location ~ ^/(miner|wallet|transactions?|block|address|events) {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;
        
        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;
    }

    # WebSocket support
    location /ws {
        proxy_pass http://127.0.0.1:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Favicon
    location /favicon.ico {
        proxy_pass http://127.0.0.1:3001/favicon.ico;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
'@

# Upload corrected configuration
Write-Host "\n4. Uploading corrected configuration..." -ForegroundColor Cyan
$nginxConfig | ssh root@103.245.38.44 "cat > /etc/nginx/conf.d/fvchain-letsencrypt-fixed.conf"

# Test Nginx configuration
Write-Host "\n5. Testing Nginx configuration..." -ForegroundColor Cyan
ssh root@103.245.38.44 "nginx -t"

if ($LASTEXITCODE -eq 0) {
    Write-Host "\n6. Nginx configuration test passed. Applying changes..." -ForegroundColor Green
    
    # Remove old configuration and activate new one
    ssh root@103.245.38.44 "rm -f /etc/nginx/conf.d/fvchain-letsencrypt.conf"
    ssh root@103.245.38.44 "mv /etc/nginx/conf.d/fvchain-letsencrypt-fixed.conf /etc/nginx/conf.d/fvchain-letsencrypt.conf"
    
    # Reload Nginx
    Write-Host "\n7. Reloading Nginx..." -ForegroundColor Cyan
    ssh root@103.245.38.44 "systemctl reload nginx"
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "\n8. Testing fixed API endpoints..." -ForegroundColor Cyan
        
        Write-Host "\nTesting: https://fvchain.xyz/api/miner/status"
        ssh root@103.245.38.44 "curl -s https://fvchain.xyz/api/miner/status"
        
        Write-Host "\n\nTesting: https://fvchain.xyz/api/events"
        ssh root@103.245.38.44 "curl -s https://fvchain.xyz/api/events"
        
        Write-Host "\n\n=== API Endpoints Fix Completed Successfully! ===" -ForegroundColor Green
        Write-Host "The following endpoints should now work:" -ForegroundColor Yellow
        Write-Host "- https://fvchain.xyz/api/miner/status" -ForegroundColor White
        Write-Host "- https://fvchain.xyz/api/miner/start" -ForegroundColor White
        Write-Host "- https://fvchain.xyz/api/miner/stop" -ForegroundColor White
        Write-Host "- https://fvchain.xyz/api/events" -ForegroundColor White
        Write-Host "- https://fvchain.xyz/api/transactions" -ForegroundColor White
        Write-Host "- https://fvchain.xyz/api/wallet/balance" -ForegroundColor White
        Write-Host "\nMixed Content errors should now be resolved!" -ForegroundColor Green
    } else {
        Write-Host "\nError: Failed to reload Nginx" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "\nError: Nginx configuration test failed" -ForegroundColor Red
    exit 1
}