# Final SSL Fix for FVChain
# This script creates a working SSL configuration

$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "=== Final SSL Fix for FVChain ===" -ForegroundColor Green

# Step 1: Remove all existing nginx configs
Write-Host "[STEP 1] Cleaning up existing configurations..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "rm -f /etc/nginx/conf.d/*.conf"

# Step 2: Create self-signed certificate with proper permissions
Write-Host "[STEP 2] Creating SSL certificate..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP @"
mkdir -p /etc/ssl/certs /etc/ssl/private
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/fvchain.key \
  -out /etc/ssl/certs/fvchain.crt \
  -subj "/C=ID/ST=Jakarta/L=Jakarta/O=FVChain/OU=IT/CN=fvchain.xyz/subjectAltName=DNS:www.fvchain.xyz,DNS:fvchain.xyz,IP:103.245.38.44"
chmod 600 /etc/ssl/private/fvchain.key
chmod 644 /etc/ssl/certs/fvchain.crt
"@

# Step 3: Create simple working SSL configuration
Write-Host "[STEP 3] Creating SSL nginx configuration..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP @"
cat > /etc/nginx/conf.d/fvchain-ssl.conf << 'NGINXEOF'
# HTTP server - redirect to HTTPS
server {
    listen 80 default_server;
    server_name _;
    return 301 https://`$host`$request_uri;
}

# HTTPS server
server {
    listen 443 ssl http2 default_server;
    server_name _;
    
    # SSL Configuration
    ssl_certificate /etc/ssl/certs/fvchain.crt;
    ssl_certificate_key /etc/ssl/private/fvchain.key;
    
    # SSL Settings
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;
    
    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    
    # Root location - proxy to dashboard
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
    }
    
    # API endpoints
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
    }
    
    # Wallet endpoints
    location /wallet/ {
        proxy_pass http://127.0.0.1:8080/wallet/;
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
    }
}
NGINXEOF
"@

# Step 4: Test configuration
Write-Host "[STEP 4] Testing nginx configuration..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "nginx -t"

# Step 5: Restart nginx
Write-Host "[STEP 5] Restarting nginx..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl restart nginx"

# Step 6: Wait and verify
Write-Host "[STEP 6] Verifying SSL is working..." -ForegroundColor Cyan
Start-Sleep -Seconds 5
echo y | plink -ssh -pw $Password $Username@$ServerIP "ss -tlnp | grep :443"
echo y | plink -ssh -pw $Password $Username@$ServerIP "curl -k -I https://localhost"

# Step 7: Test external access
Write-Host "[STEP 7] Testing external HTTPS access..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "curl -k -I https://103.245.38.44"

Write-Host "" 
Write-Host "=== SSL Implementation Complete ===" -ForegroundColor Green
Write-Host "HTTPS URLs:" -ForegroundColor Yellow
Write-Host "  - https://103.245.38.44" -ForegroundColor White
Write-Host "  - https://www.fvchain.xyz (when DNS resolves)" -ForegroundColor White
Write-Host "" 
Write-Host "Certificate: Self-Signed (temporary)" -ForegroundColor Yellow
Write-Host "Browser Warning: Click 'Advanced' -> 'Proceed to site'" -ForegroundColor Red
Write-Host "" 
Write-Host "Next Steps for Production:" -ForegroundColor Cyan
Write-Host "1. Configure DNS: www.fvchain.xyz -> 103.245.38.44" -ForegroundColor White
Write-Host "2. Install Let's Encrypt certificate after DNS propagation" -ForegroundColor White