#!/usr/bin/env pwsh
# Upload Nginx Configuration to Server

Write-Host "Uploading nginx configuration..." -ForegroundColor Cyan

# Create nginx configuration content
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
    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-AES128-SHA256:ECDHE-RSA-AES256-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

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
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
            add_header Access-Control-Allow-Headers "Content-Type, Authorization";
            add_header Access-Control-Max-Age 86400;
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

    location /api/mining/events {
        proxy_pass http://127.0.0.1:8080/events;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Connection "";
        
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 24h;
        proxy_send_timeout 24h;
        
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization, Cache-Control" always;
        add_header Access-Control-Expose-Headers "Content-Type" always;
        
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

# Upload using multiple commands to avoid issues
Write-Host "Creating temporary config file on server..." -ForegroundColor Yellow
ssh root@fvchain.xyz "rm -f /tmp/fvchain.conf"

# Split config into smaller parts to avoid command line length issues
$lines = $nginxConfig -split "`n"
$chunkSize = 10

for ($i = 0; $i -lt $lines.Length; $i += $chunkSize) {
    $chunk = $lines[$i..([Math]::Min($i + $chunkSize - 1, $lines.Length - 1))] -join "`n"
    if ($i -eq 0) {
        ssh root@fvchain.xyz "echo '$chunk' > /tmp/fvchain.conf"
    } else {
        ssh root@fvchain.xyz "echo '$chunk' >> /tmp/fvchain.conf"
    }
}

Write-Host "Moving config to nginx directory..." -ForegroundColor Yellow
ssh root@fvchain.xyz "cp /tmp/fvchain.conf /etc/nginx/conf.d/fvchain.conf"

Write-Host "Testing nginx configuration..." -ForegroundColor Cyan
ssh root@fvchain.xyz "nginx -t"

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Nginx configuration is valid!" -ForegroundColor Green
    
    Write-Host "Reloading nginx..." -ForegroundColor Cyan
    ssh root@fvchain.xyz "systemctl reload nginx"
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Nginx reloaded successfully!" -ForegroundColor Green
    } else {
        Write-Host "❌ Failed to reload nginx!" -ForegroundColor Red
    }
} else {
    Write-Host "❌ Nginx configuration has errors!" -ForegroundColor Red
}

Write-Host "Cleaning up temporary files..." -ForegroundColor Yellow
ssh root@fvchain.xyz "rm -f /tmp/fvchain.conf"

Write-Host "✅ Nginx configuration upload completed!" -ForegroundColor Green