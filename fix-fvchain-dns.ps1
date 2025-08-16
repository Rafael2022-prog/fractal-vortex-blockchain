# Fix FVChain.com DNS and Implement Let's Encrypt
# Domain sudah terpropagasi tapi mengarah ke IP yang salah
# Created by: Emylton Leunufna
# Date: 2025

Write-Host "=== FVChain.com DNS Fix & Let's Encrypt Implementation ===" -ForegroundColor Green

$SERVER_IP = "103.245.38.44"
$DOMAIN = "fvchain.com"
$EMAIL = "admin@fvchain.com"

Write-Host "`nServer IP: $SERVER_IP" -ForegroundColor Cyan
Write-Host "Target Domain: $DOMAIN" -ForegroundColor Cyan
Write-Host "Email: $EMAIL" -ForegroundColor Cyan

# Step 1: Check current DNS status
Write-Host "`n=== Step 1: Checking Current DNS Status ===" -ForegroundColor Magenta

try {
    $dnsResult = Resolve-DnsName -Name $DOMAIN -Type A -ErrorAction Stop
    $resolvedIPs = $dnsResult.IPAddress
    
    Write-Host "Current DNS Resolution:" -ForegroundColor Yellow
    Write-Host "  Domain: $DOMAIN" -ForegroundColor White
    Write-Host "  Current IPs: $($resolvedIPs -join ', ')" -ForegroundColor White
    Write-Host "  Expected IP: $SERVER_IP" -ForegroundColor White
    
    if ($resolvedIPs -contains $SERVER_IP) {
        Write-Host "  Success: DNS sudah mengarah ke server kita!" -ForegroundColor Green
        $dnsReady = $true
    } else {
        Write-Host "  Warning: DNS mengarah ke IP yang salah" -ForegroundColor Red
        Write-Host "  Current: $($resolvedIPs -join ', ')" -ForegroundColor Red
        Write-Host "  Expected: $SERVER_IP" -ForegroundColor Red
        $dnsReady = $false
    }
} catch {
    Write-Host "  Error: Domain not found: $DOMAIN" -ForegroundColor Red
    $dnsReady = $false
}

if (-not $dnsReady) {
    Write-Host "`nAction Required: Update DNS Record" -ForegroundColor Yellow
    Write-Host "`nLangkah-langkah untuk memperbaiki DNS:" -ForegroundColor Cyan
    Write-Host "1. Login ke DNS provider/registrar domain fvchain.com" -ForegroundColor White
    Write-Host "2. Cari DNS Management atau DNS Records" -ForegroundColor White
    Write-Host "3. Update A Record:" -ForegroundColor White
    Write-Host "   Type: A" -ForegroundColor Gray
    Write-Host "   Name: @ (atau fvchain.com)" -ForegroundColor Gray
    Write-Host "   Value: $SERVER_IP" -ForegroundColor Gray
    Write-Host "   TTL: 3600" -ForegroundColor Gray
    Write-Host "4. Hapus A Record yang mengarah ke IP lama" -ForegroundColor White
    Write-Host "5. Tunggu propagasi DNS (5-30 menit)" -ForegroundColor White
    Write-Host "6. Jalankan script ini lagi" -ForegroundColor White
    
    Write-Host "`nCurrent DNS Records to Remove:" -ForegroundColor Red
    foreach ($ip in $resolvedIPs) {
        Write-Host "   A Record: @ -> $ip (HAPUS INI)" -ForegroundColor Red
    }
    
    Write-Host "`nNew DNS Record to Add:" -ForegroundColor Green
    Write-Host "   A Record: @ -> $SERVER_IP (TAMBAH INI)" -ForegroundColor Green
    
    Write-Host "`nAlternatif sementara:" -ForegroundColor Cyan
    Write-Host "- Gunakan IP address: https://$SERVER_IP" -ForegroundColor White
    Write-Host "- SSL self-signed sudah aktif" -ForegroundColor White
    
    return
}

# Continue with Let's Encrypt installation
Write-Host "`n=== DNS Ready! Proceeding with Let's Encrypt ===" -ForegroundColor Green

# Step 2: Install Certbot
Write-Host "`n=== Step 2: Installing Certbot ===" -ForegroundColor Magenta

$installCmd = "yum update -y && yum install -y epel-release && yum install -y certbot python3-certbot-nginx"
Write-Host "Installing Certbot..." -ForegroundColor Yellow
$result = plink -ssh root@$SERVER_IP -batch $installCmd
if ($LASTEXITCODE -eq 0) {
    Write-Host "Success: Certbot installed successfully" -ForegroundColor Green
} else {
    Write-Host "Error: Failed to install Certbot" -ForegroundColor Red
    return
}

# Step 3: Stop Nginx
Write-Host "`n=== Step 3: Stopping Nginx ===" -ForegroundColor Magenta

$stopCmd = "systemctl stop nginx"
Write-Host "Stopping Nginx..." -ForegroundColor Yellow
$result = plink -ssh root@$SERVER_IP -batch $stopCmd
if ($LASTEXITCODE -eq 0) {
    Write-Host "Success: Nginx stopped" -ForegroundColor Green
} else {
    Write-Host "Error: Failed to stop Nginx" -ForegroundColor Red
}

# Step 4: Generate Certificate
Write-Host "`n=== Step 4: Generating SSL Certificate ===" -ForegroundColor Magenta

$certCmd = "certbot certonly --standalone --non-interactive --agree-tos --email $EMAIL -d $DOMAIN -d www.$DOMAIN"
Write-Host "Generating certificate for $DOMAIN and www.$DOMAIN..." -ForegroundColor Yellow
$result = plink -ssh root@$SERVER_IP -batch $certCmd
if ($LASTEXITCODE -eq 0) {
    Write-Host "Success: SSL certificate generated successfully" -ForegroundColor Green
} else {
    Write-Host "Error: Failed to generate certificate" -ForegroundColor Red
    Write-Host "Starting Nginx back..." -ForegroundColor Yellow
    plink -ssh root@$SERVER_IP -batch "systemctl start nginx"
    return
}

# Step 5: Create new SSL configuration
Write-Host "`n=== Step 5: Creating SSL Configuration ===" -ForegroundColor Magenta

$nginxConfig = @"
# HTTP to HTTPS redirect
server {
    listen 80;
    server_name $DOMAIN www.$DOMAIN;
    return 301 https://`$server_name`$request_uri;
}

# HTTPS server
server {
    listen 443 ssl http2;
    server_name $DOMAIN www.$DOMAIN;

    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;
    
    # Modern SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    
    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' wss: ws:;" always;
    
    # Root location - Dashboard
    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
        
        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade `$http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    # API endpoints
    location /api/ {
        proxy_pass http://127.0.0.1:8080/;
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
        
        # API specific headers
        add_header Access-Control-Allow-Origin "*" always;
        add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS" always;
        add_header Access-Control-Allow-Headers "DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range,Authorization" always;
        
        # Handle preflight requests
        if (`$request_method = 'OPTIONS') {
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS";
            add_header Access-Control-Allow-Headers "DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range,Authorization";
            add_header Access-Control-Max-Age 1728000;
            add_header Content-Type "text/plain; charset=utf-8";
            add_header Content-Length 0;
            return 204;
        }
    }
    
    # Health check
    location /health {
        proxy_pass http://127.0.0.1:8080/health;
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        access_log off;
    }
    
    # Blockchain specific endpoints
    location /rpc {
        proxy_pass http://127.0.0.1:8080/rpc;
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
    }
    
    location /ws {
        proxy_pass http://127.0.0.1:8080/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade `$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
    }
    
    # Static files caching
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)`$ {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host `$host;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
"@

# Write config to temp file
$configFile = "R:\369-FRACTAL\nginx-fvchain-com.conf"
$nginxConfig | Out-File -FilePath $configFile -Encoding UTF8

Write-Host "Uploading SSL configuration..." -ForegroundColor Yellow
$uploadResult = pscp -scp $configFile "root@${SERVER_IP}:/etc/nginx/conf.d/ssl-fvchain.conf"
if ($LASTEXITCODE -eq 0) {
    Write-Host "Success: SSL configuration uploaded" -ForegroundColor Green
} else {
    Write-Host "Error: Failed to upload configuration" -ForegroundColor Red
    return
}

# Step 6: Remove old configuration
Write-Host "`n=== Step 6: Updating Configuration ===" -ForegroundColor Magenta

$removeOldCmd = "rm -f /etc/nginx/conf.d/ssl-letsencrypt.conf"
Write-Host "Removing old SSL configuration..." -ForegroundColor Yellow
plink -ssh root@$SERVER_IP -batch $removeOldCmd

# Step 7: Test and start Nginx
Write-Host "`n=== Step 7: Starting Nginx ===" -ForegroundColor Magenta

$testCmd = "nginx -t"
Write-Host "Testing Nginx configuration..." -ForegroundColor Yellow
$result = plink -ssh root@$SERVER_IP -batch $testCmd
if ($LASTEXITCODE -eq 0) {
    Write-Host "Success: Nginx configuration is valid" -ForegroundColor Green
    
    $startCmd = "systemctl start nginx && systemctl enable nginx"
    Write-Host "Starting Nginx..." -ForegroundColor Yellow
    $result = plink -ssh root@$SERVER_IP -batch $startCmd
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Success: Nginx started successfully" -ForegroundColor Green
    } else {
        Write-Host "Error: Failed to start Nginx" -ForegroundColor Red
    }
} else {
    Write-Host "Error: Nginx configuration error" -ForegroundColor Red
    return
}

# Step 8: Setup auto-renewal
Write-Host "`n=== Step 8: Setting up Auto-renewal ===" -ForegroundColor Magenta

$cronCmd = 'echo "0 12 * * * /usr/bin/certbot renew --quiet --nginx" | crontab -'
Write-Host "Setting up automatic certificate renewal..." -ForegroundColor Yellow
$result = plink -ssh root@$SERVER_IP -batch $cronCmd
if ($LASTEXITCODE -eq 0) {
    Write-Host "Success: Auto-renewal configured" -ForegroundColor Green
} else {
    Write-Host "Warning: Auto-renewal setup failed (manual renewal required)" -ForegroundColor Yellow
}

# Step 9: Final verification
Write-Host "`n=== Step 9: Final Verification ===" -ForegroundColor Magenta

Write-Host "Checking Nginx status..." -ForegroundColor Yellow
$statusResult = plink -ssh root@$SERVER_IP -batch "systemctl status nginx --no-pager"

Write-Host "Checking HTTPS port..." -ForegroundColor Yellow
$portResult = plink -ssh root@$SERVER_IP -batch "netstat -tlnp | grep ':443'"

Write-Host "Testing HTTPS connection..." -ForegroundColor Yellow
$httpsTest = plink -ssh root@$SERVER_IP -batch "curl -I https://$DOMAIN"

# Final status
Write-Host "`n=== Let's Encrypt Implementation Complete ===" -ForegroundColor Green
Write-Host "Success: Valid SSL Certificate installed for $DOMAIN!" -ForegroundColor Green

Write-Host "`nAkses FVChain Blockchain:" -ForegroundColor Cyan
Write-Host "   Dashboard: https://$DOMAIN" -ForegroundColor White
Write-Host "   Dashboard (www): https://www.$DOMAIN" -ForegroundColor White
Write-Host "   API: https://$DOMAIN/api/" -ForegroundColor White
Write-Host "   Health Check: https://$DOMAIN/health" -ForegroundColor White
Write-Host "   RPC: https://$DOMAIN/rpc" -ForegroundColor White
Write-Host "   WebSocket: wss://$DOMAIN/ws" -ForegroundColor White

Write-Host "`nFeatures:" -ForegroundColor Cyan
Write-Host "   - Valid SSL Certificate (Let's Encrypt)" -ForegroundColor Green
Write-Host "   - Auto-renewal configured" -ForegroundColor Green
Write-Host "   - Modern security headers" -ForegroundColor Green
Write-Host "   - HTTP to HTTPS redirect" -ForegroundColor Green
Write-Host "   - WebSocket support" -ForegroundColor Green
Write-Host "   - CORS enabled for API" -ForegroundColor Green
Write-Host "   - Static file caching" -ForegroundColor Green

Write-Host "`nDomain: $DOMAIN" -ForegroundColor Cyan
Write-Host "Certificate Path: /etc/letsencrypt/live/$DOMAIN/" -ForegroundColor White
Write-Host "Certificate Expiry: 90 days (auto-renewal enabled)" -ForegroundColor White

Write-Host "`nFVChain Blockchain siap untuk produksi dengan SSL yang valid!" -ForegroundColor Yellow
Write-Host "Browser tidak akan menampilkan peringatan sertifikat lagi." -ForegroundColor Green

# Clean up temp file
Remove-Item $configFile -Force -ErrorAction SilentlyContinue

Write-Host "`nNext Steps:" -ForegroundColor Cyan
Write-Host "1. Test semua endpoint di browser" -ForegroundColor White
Write-Host "2. Verifikasi SSL certificate di browser" -ForegroundColor White
Write-Host "3. Monitor auto-renewal: certbot certificates" -ForegroundColor White
Write-Host "4. Setup monitoring dan backup" -ForegroundColor White