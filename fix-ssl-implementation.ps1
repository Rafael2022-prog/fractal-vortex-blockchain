# Fix SSL Implementation for fvchain.xyz
# This script properly installs SSL certificate and configures nginx

$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"
$Domain = "www.fvchain.xyz"

Write-Host "=== Fixing SSL Implementation for $Domain ===" -ForegroundColor Green

# Step 1: Install certbot if not already installed
Write-Host "[STEP 1] Installing certbot and dependencies..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "yum install -y epel-release"
echo y | plink -ssh -pw $Password $Username@$ServerIP "yum install -y certbot python3-certbot-nginx"

# Step 2: Stop nginx temporarily
Write-Host "[STEP 2] Stopping nginx temporarily..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl stop nginx"

# Step 3: Check if domain resolves to our server
Write-Host "[STEP 3] Checking DNS resolution..." -ForegroundColor Cyan
$dnsResult = nslookup $Domain 8.8.8.8
Write-Host "DNS Result: $dnsResult" -ForegroundColor Yellow

# Step 4: Generate SSL certificate using standalone mode
Write-Host "[STEP 4] Generating SSL certificate..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "certbot certonly --standalone --non-interactive --agree-tos --email admin@fvchain.xyz -d $Domain --force-renewal"

# Step 5: Verify certificate was created
Write-Host "[STEP 5] Verifying certificate creation..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "ls -la /etc/letsencrypt/live/$Domain/"

# Step 6: Create proper SSL nginx configuration
Write-Host "[STEP 6] Creating SSL nginx configuration..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP @"
cat > /etc/nginx/conf.d/fvc-dashboard-ssl.conf << 'NGINXEOF'
# HTTP to HTTPS redirect
server {
    listen 80;
    server_name www.fvchain.xyz;
    return 301 https://`$server_name`$request_uri;
}

# HTTPS server block
server {
    listen 443 ssl http2;
    server_name www.fvchain.xyz;
    
    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/www.fvchain.xyz/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/www.fvchain.xyz/privkey.pem;
    
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
    
    # API endpoints
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
    }
}
NGINXEOF
"@

# Step 7: Test nginx configuration
Write-Host "[STEP 7] Testing nginx configuration..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "nginx -t"

# Step 8: Start nginx
Write-Host "[STEP 8] Starting nginx..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl start nginx"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl enable nginx"

# Step 9: Setup auto-renewal
Write-Host "[STEP 9] Setting up SSL auto-renewal..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "echo '0 12 * * * /usr/bin/certbot renew --quiet' | crontab -"

# Step 10: Final verification
Write-Host "[STEP 10] Final verification..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "ss -tlnp | grep :443"
echo y | plink -ssh -pw $Password $Username@$ServerIP "curl -k -I https://localhost"

Write-Host "" 
Write-Host "=== SSL Implementation Complete ===" -ForegroundColor Green
Write-Host "Domain: https://$Domain" -ForegroundColor Yellow
Write-Host "SSL Certificate: Let's Encrypt" -ForegroundColor Yellow
Write-Host "Auto-renewal: Configured" -ForegroundColor Yellow
Write-Host "" 
Write-Host "Note: If domain doesn't resolve yet, please check DNS settings." -ForegroundColor Red