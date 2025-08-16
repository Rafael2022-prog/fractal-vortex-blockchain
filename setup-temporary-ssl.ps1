# Setup Temporary SSL with Self-Signed Certificate
# This will work while waiting for DNS propagation

$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"
$Domain = "www.fvchain.xyz"

Write-Host "=== Setting up Temporary SSL for $Domain ===" -ForegroundColor Green

# Step 1: Create self-signed certificate
Write-Host "[STEP 1] Creating self-signed SSL certificate..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP @"
mkdir -p /etc/ssl/certs /etc/ssl/private
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/fvchain.key \
  -out /etc/ssl/certs/fvchain.crt \
  -subj "/C=ID/ST=Jakarta/L=Jakarta/O=FVChain/OU=IT/CN=www.fvchain.xyz"
"@

# Step 2: Create nginx SSL configuration with self-signed cert
Write-Host "[STEP 2] Creating nginx SSL configuration..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP @"
cat > /etc/nginx/conf.d/fvc-dashboard-ssl.conf << 'NGINXEOF'
# HTTP to HTTPS redirect
server {
    listen 80;
    server_name www.fvchain.xyz fvchain.xyz 103.245.38.44;
    return 301 https://`$server_name`$request_uri;
}

# HTTPS server block with self-signed certificate
server {
    listen 443 ssl http2;
    server_name www.fvchain.xyz fvchain.xyz 103.245.38.44;
    
    # SSL Configuration (Self-Signed)
    ssl_certificate /etc/ssl/certs/fvchain.crt;
    ssl_certificate_key /etc/ssl/private/fvchain.key;
    
    # SSL Security Settings
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
    
    # Proxy to dashboard
    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
        proxy_set_header X-Forwarded-Host `$host;
        proxy_set_header X-Forwarded-Port `$server_port;
        
        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade `$http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    # API endpoints for RPC
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
    }
    
    # Direct RPC access
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

# Step 3: Test nginx configuration
Write-Host "[STEP 3] Testing nginx configuration..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "nginx -t"

# Step 4: Restart nginx
Write-Host "[STEP 4] Restarting nginx..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl restart nginx"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl enable nginx"

# Step 5: Verify HTTPS is working
Write-Host "[STEP 5] Verifying HTTPS is working..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "ss -tlnp | grep :443"
echo y | plink -ssh -pw $Password $Username@$ServerIP "curl -k -I https://localhost"

# Step 6: Test with IP address
Write-Host "[STEP 6] Testing HTTPS with IP address..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "curl -k -I https://103.245.38.44"

Write-Host "" 
Write-Host "=== Temporary SSL Setup Complete ===" -ForegroundColor Green
Write-Host "HTTPS URLs:" -ForegroundColor Yellow
Write-Host "  - https://103.245.38.44 (IP access)" -ForegroundColor White
Write-Host "  - https://www.fvchain.xyz (when DNS resolves)" -ForegroundColor White
Write-Host "" 
Write-Host "Certificate Type: Self-Signed (temporary)" -ForegroundColor Yellow
Write-Host "Browser Warning: Expected (click 'Advanced' -> 'Proceed')" -ForegroundColor Red
Write-Host "" 
Write-Host "Next Steps:" -ForegroundColor Cyan
Write-Host "1. Configure DNS A record: www.fvchain.xyz -> 103.245.38.44" -ForegroundColor White
Write-Host "2. Wait for DNS propagation (24-48 hours)" -ForegroundColor White
Write-Host "3. Run Let's Encrypt certificate installation" -ForegroundColor White