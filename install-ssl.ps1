# Install SSL Certificate for www.fvchain.xyz
# This script installs Let's Encrypt SSL certificate using certbot

$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"
$Domain = "www.fvchain.xyz"

Write-Host "Installing SSL Certificate for $Domain..." -ForegroundColor Green

# Install certbot and dependencies
Write-Host "[INFO] Installing certbot and dependencies..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "yum install -y epel-release"
echo y | plink -ssh -pw $Password $Username@$ServerIP "yum install -y certbot python3-certbot-nginx"

# Stop nginx temporarily for certificate generation
Write-Host "[INFO] Stopping nginx temporarily..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl stop nginx"

# Obtain SSL certificate using standalone mode
Write-Host "[INFO] Obtaining SSL certificate for $Domain..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "certbot certonly --standalone --non-interactive --agree-tos --email admin@fvchain.xyz -d $Domain"

# Create SSL-enabled nginx configuration
Write-Host "[INFO] Creating SSL-enabled nginx configuration..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "cat > /etc/nginx/conf.d/fvc-dashboard-ssl.conf << 'NGINXEOF'
server {
    listen 80;
    server_name www.fvchain.xyz;
    return 301 https://\$server_name\$request_uri;
}

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
    add_header Strict-Transport-Security \"max-age=31536000; includeSubDomains\" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection \"1; mode=block\" always;
    add_header Referrer-Policy \"strict-origin-when-cross-origin\" always;
    
    # Dashboard (Next.js)
    location / {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_cache_bypass \$http_upgrade;
    }
    
    # FVC RPC API
    location /api/rpc {
        proxy_pass http://localhost:8332;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
    
    # Mining API
    location /api/mining {
        proxy_pass http://localhost:8333;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
    
    # WebSocket support for real-time updates
    location /ws {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection \"upgrade\";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
NGINXEOF"

# Remove old non-SSL configuration
Write-Host "[INFO] Removing old non-SSL configuration..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "rm -f /etc/nginx/conf.d/fvc-dashboard.conf"

# Test nginx configuration
Write-Host "[INFO] Testing nginx configuration..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "nginx -t"

# Start nginx
Write-Host "[INFO] Starting nginx with SSL configuration..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl start nginx"

# Setup automatic certificate renewal
Write-Host "[INFO] Setting up automatic certificate renewal..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "(crontab -l 2>/dev/null; echo '0 12 * * * /usr/bin/certbot renew --quiet --post-hook \"systemctl reload nginx\"') | crontab -"

# Test SSL certificate
Write-Host "[INFO] Testing SSL certificate..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "certbot certificates"

# Check services status
Write-Host "[INFO] Checking services status..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl status nginx --no-pager"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl status fvc-dashboard --no-pager"

Write-Host "SSL Certificate installation completed!" -ForegroundColor Green
Write-Host "HTTPS URL: https://$Domain" -ForegroundColor Yellow
Write-Host "SSL Certificate: Let's Encrypt" -ForegroundColor Yellow
Write-Host "Auto-renewal: Configured (daily check at 12:00)" -ForegroundColor Yellow
Write-Host "" 
Write-Host "Important: Make sure DNS A record for $Domain points to $ServerIP" -ForegroundColor Red
Write-Host "You can verify SSL at: https://www.ssllabs.com/ssltest/" -ForegroundColor Red

# Test HTTPS connection
Write-Host "[INFO] Testing HTTPS connection..." -ForegroundColor Cyan
try {
    $response = Invoke-WebRequest -Uri "https://$Domain" -Method Head -SkipCertificateCheck
    Write-Host "HTTPS connection successful! Status: $($response.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "HTTPS connection failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "This might be due to DNS propagation delay or domain configuration." -ForegroundColor Yellow
}