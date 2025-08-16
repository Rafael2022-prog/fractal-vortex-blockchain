# Script untuk memperbaiki routing mining API di server cloud fvchain.xyz
# Mengatasi error 500 dan 405 pada endpoint mining

Write-Host "Memperbaiki routing mining API di server cloud..." -ForegroundColor Cyan

# Membuat file konfigurasi Nginx
$configContent = @'
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
    ssl_prefer_server_ciphers on;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Mining API endpoints - route to RPC server on port 8080
    location /api/mining/miner/status {
        proxy_pass http://127.0.0.1:8080/miner/status;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
        
        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;
        
        if ($request_method = OPTIONS) {
            return 204;
        }
    }

    location /api/mining/miner/start {
        proxy_pass http://127.0.0.1:8080/miner/start;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
        
        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;
        
        if ($request_method = OPTIONS) {
            return 204;
        }
    }

    location /api/mining/miner/stop {
        proxy_pass http://127.0.0.1:8080/miner/stop;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
        
        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;
        
        if ($request_method = OPTIONS) {
            return 204;
        }
    }

    # Mining events endpoint - SSE stream
    location /api/mining/events {
        proxy_pass http://127.0.0.1:8080/events;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # SSE specific headers
        proxy_set_header Connection "";
        proxy_http_version 1.1;
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 24h;
        proxy_send_timeout 24h;
        
        # CORS headers for SSE
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Credentials "true" always;
        add_header Cache-Control "no-cache" always;
    }

    # Other API endpoints - route to Next.js on port 3000
    location ~ ^/api/(wallet|transactions|blocks|network)/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # CORS headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;
    }

    # Next.js application
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }

    # Static files
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
'@

# Simpan ke file lokal dengan encoding ASCII
$configFile = "fvchain-mining-fix.conf"
[System.IO.File]::WriteAllText($configFile, $configContent, [System.Text.Encoding]::ASCII)

try {
    Write-Host "Mengunggah konfigurasi Nginx baru..." -ForegroundColor Yellow
    
    # Buat backup konfigurasi saat ini
    $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
    ssh root@fvchain.xyz "cp /etc/nginx/conf.d/fvchain.conf /etc/nginx/conf.d/fvchain.conf.backup-$timestamp"
    
    # Upload konfigurasi baru
    scp $configFile root@fvchain.xyz:/etc/nginx/conf.d/fvchain.conf
    
    Write-Host "Menguji konfigurasi Nginx..." -ForegroundColor Yellow
    $testResult = ssh root@fvchain.xyz "nginx -t 2>&1"
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Konfigurasi Nginx valid" -ForegroundColor Green
        
        Write-Host "Memuat ulang Nginx..." -ForegroundColor Yellow
        ssh root@fvchain.xyz "systemctl reload nginx"
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Nginx berhasil dimuat ulang" -ForegroundColor Green
            
            # Periksa status layanan RPC
            Write-Host "Memeriksa status layanan RPC..." -ForegroundColor Yellow
            $rpcStatus = ssh root@fvchain.xyz "systemctl is-active fvc-mainnet-rpc.service"
            
            if ($rpcStatus -eq "active") {
                Write-Host "Layanan RPC aktif" -ForegroundColor Green
            } else {
                Write-Host "Layanan RPC tidak aktif, memulai ulang..." -ForegroundColor Yellow
                ssh root@fvchain.xyz "systemctl restart fvc-mainnet-rpc.service"
                Start-Sleep -Seconds 3
                $newStatus = ssh root@fvchain.xyz "systemctl is-active fvc-mainnet-rpc.service"
                if ($newStatus -eq "active") {
                    Write-Host "Layanan RPC berhasil dimulai" -ForegroundColor Green
                } else {
                    Write-Host "Gagal memulai layanan RPC" -ForegroundColor Red
                }
            }
            
            # Test endpoint mining
            Write-Host "Menguji endpoint mining..." -ForegroundColor Yellow
            Start-Sleep -Seconds 2
            
            # Test direct RPC endpoint
            $testRpcStatus = ssh root@fvchain.xyz "curl -s -o /dev/null -w '%{http_code}' 'http://localhost:8080/miner/status?device_id=test'"
            Write-Host "Status RPC endpoint /miner/status: $testRpcStatus" -ForegroundColor Cyan
            
            # Test through Nginx proxy
            $testProxyStatus = ssh root@fvchain.xyz "curl -s -o /dev/null -w '%{http_code}' 'https://localhost/api/mining/miner/status?device_id=test' -k"
            Write-Host "Status Nginx proxy /api/mining/miner/status: $testProxyStatus" -ForegroundColor Cyan
            
            $testEvents = ssh root@fvchain.xyz "timeout 3 curl -s 'http://localhost:8080/events' | head -1"
            if ($testEvents) {
                Write-Host "Endpoint /events: Aktif (SSE stream)" -ForegroundColor Green
            } else {
                Write-Host "Endpoint /events: Tidak merespons" -ForegroundColor Yellow
            }
            
            Write-Host "" 
            Write-Host "PERBAIKAN SELESAI!" -ForegroundColor Green
            Write-Host "Silakan cek halaman: https://fvchain.xyz/mining" -ForegroundColor Cyan
            Write-Host "" 
            
        } else {
            Write-Host "Gagal memuat ulang Nginx" -ForegroundColor Red
        }
    } else {
        Write-Host "Konfigurasi Nginx tidak valid:" -ForegroundColor Red
        Write-Host $testResult -ForegroundColor Red
        
        # Kembalikan backup
        Write-Host "Mengembalikan konfigurasi backup..." -ForegroundColor Yellow
        ssh root@fvchain.xyz "cp /etc/nginx/conf.d/fvchain.conf.backup-$timestamp /etc/nginx/conf.d/fvchain.conf"
        ssh root@fvchain.xyz "systemctl reload nginx"
    }
    
} catch {
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Script selesai dijalankan." -ForegroundColor White

# Hapus file konfigurasi lokal
Remove-Item $configFile -ErrorAction SilentlyContinue