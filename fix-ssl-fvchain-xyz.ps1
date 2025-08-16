# Fix SSL Configuration untuk fvchain.xyz dengan Let's Encrypt
# Menggunakan struktur direktori conf.d yang benar

Write-Host "=== Memperbaiki SSL Configuration untuk fvchain.xyz ===" -ForegroundColor Green

$DOMAIN = "fvchain.xyz"
$WWW_DOMAIN = "www.fvchain.xyz"
$SERVER_IP = "103.245.38.44"

# Step 1: Verifikasi sertifikat Let's Encrypt sudah ada
Write-Host "\n1. Memeriksa sertifikat Let's Encrypt..." -ForegroundColor Yellow
$certCheck = ssh root@$SERVER_IP "ls -la /etc/letsencrypt/live/$DOMAIN/"
Write-Host "Sertifikat ditemukan:" -ForegroundColor Cyan
Write-Host $certCheck -ForegroundColor White

# Step 2: Backup konfigurasi lama
Write-Host "\n2. Backup konfigurasi lama..." -ForegroundColor Yellow
ssh root@$SERVER_IP "cp /etc/nginx/conf.d/ssl-fvchain.conf /etc/nginx/conf.d/ssl-fvchain.conf.backup-$(date +%Y%m%d-%H%M%S)"

# Step 3: Buat konfigurasi SSL yang benar
Write-Host "\n3. Membuat konfigurasi SSL yang benar..." -ForegroundColor Yellow

$nginxSSLConfig = @"
# HTTP to HTTPS redirect
server {
    listen 80;
    server_name $DOMAIN $WWW_DOMAIN;
    return 301 https://`$server_name`$request_uri;
}

# HTTPS server with Let's Encrypt SSL
server {
    listen 443 ssl http2;
    server_name $DOMAIN $WWW_DOMAIN;

    # Let's Encrypt SSL Configuration
    ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;
    
    # Modern SSL Configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:DHE-RSA-AES128-GCM-SHA256:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    ssl_session_tickets off;
    
    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/letsencrypt/live/$DOMAIN/chain.pem;
    resolver 8.8.8.8 8.8.4.4 valid=300s;
    resolver_timeout 5s;

    # Security Headers
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' wss: ws:;" always;

    # Dashboard (port 3001)
    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade `$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
        proxy_cache_bypass `$http_upgrade;
        proxy_read_timeout 86400;
        proxy_connect_timeout 60;
        proxy_send_timeout 60;
    }

    # API (port 8080)
    location /api/ {
        proxy_pass http://127.0.0.1:8080/;
        proxy_http_version 1.1;
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
        proxy_read_timeout 86400;
        proxy_connect_timeout 60;
        proxy_send_timeout 60;
    }

    # WebSocket support
    location /ws {
        proxy_pass http://127.0.0.1:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade `$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
    }
    
    # Favicon
    location /favicon.ico {
        proxy_pass http://127.0.0.1:3001/favicon.ico;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
"@

# Step 4: Upload konfigurasi baru
Write-Host "\n4. Mengunggah konfigurasi SSL baru..." -ForegroundColor Yellow
$nginxSSLConfig | ssh root@$SERVER_IP "cat > /etc/nginx/conf.d/fvchain-ssl.conf"

# Step 5: Hapus konfigurasi lama
Write-Host "\n5. Menghapus konfigurasi lama..." -ForegroundColor Yellow
ssh root@$SERVER_IP "rm -f /etc/nginx/conf.d/ssl-fvchain.conf"
ssh root@$SERVER_IP "rm -f /etc/nginx/conf.d/fvc-dashboard.conf.backup"

# Step 6: Test dan restart Nginx
Write-Host "\n6. Testing dan restart Nginx..." -ForegroundColor Yellow
$testResult = ssh root@$SERVER_IP "nginx -t 2>&1"
Write-Host "Nginx test result: $testResult" -ForegroundColor Cyan

if ($testResult -like "*successful*") {
    ssh root@$SERVER_IP "systemctl restart nginx"
    ssh root@$SERVER_IP "systemctl status nginx --no-pager -l"
    Write-Host "Nginx berhasil direstart dengan SSL Let's Encrypt!" -ForegroundColor Green
} else {
    Write-Host "Error: Konfigurasi Nginx tidak valid!" -ForegroundColor Red
    Write-Host $testResult -ForegroundColor Red
    exit 1
}

# Step 7: Verifikasi SSL
Write-Host "\n7. Verifikasi SSL Certificate..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

# Test SSL dengan curl
$sslTest = ssh root@$SERVER_IP "curl -I https://$DOMAIN --connect-timeout 10 2>&1"
Write-Host "SSL Test Result:" -ForegroundColor Cyan
Write-Host $sslTest -ForegroundColor White

# Test certificate details
$certDetails = ssh root@$SERVER_IP "echo | openssl s_client -servername $DOMAIN -connect $DOMAIN:443 2>/dev/null | openssl x509 -noout -issuer -subject -dates"
Write-Host "\nCertificate Details:" -ForegroundColor Cyan
Write-Host $certDetails -ForegroundColor White

Write-Host "\n=== SSL CONFIGURATION SELESAI ===" -ForegroundColor Green
Write-Host "Domain: https://$DOMAIN" -ForegroundColor Cyan
Write-Host "WWW: https://$WWW_DOMAIN" -ForegroundColor Cyan
Write-Host "\nSilakan akses https://$DOMAIN untuk memverifikasi!" -ForegroundColor Green
Write-Host "Peringatan keamanan Firefox seharusnya sudah hilang." -ForegroundColor Yellow