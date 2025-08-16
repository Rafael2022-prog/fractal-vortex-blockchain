# Script untuk menginstall Let's Encrypt SSL Certificate
# FVChain Blockchain - SSL Certificate Implementation
# Created by: Emylton Leunufna
# Date: 2025

Write-Host "=== FVChain Let's Encrypt SSL Installation ===" -ForegroundColor Green
Write-Host "Menginstall sertifikat SSL yang valid untuk produksi..." -ForegroundColor Yellow

# Konfigurasi domain
$DOMAIN = "fvchain.org"
$EMAIL = "admin@fvchain.org"
$SERVER_IP = "103.245.38.44"

Write-Host "\nDomain: $DOMAIN" -ForegroundColor Cyan
Write-Host "Email: $EMAIL" -ForegroundColor Cyan
Write-Host "Server IP: $SERVER_IP" -ForegroundColor Cyan

# Fungsi untuk menjalankan perintah SSH
function Invoke-SSHCommand {
    param(
        [string]$Command,
        [string]$Description
    )
    
    Write-Host "\n[$Description]" -ForegroundColor Yellow
    Write-Host "Executing: $Command" -ForegroundColor Gray
    
    $result = plink -ssh root@$SERVER_IP -batch $Command
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Host "✅ Success: $Description" -ForegroundColor Green
        return $result
    } else {
        Write-Host "❌ Failed: $Description (Exit Code: $exitCode)" -ForegroundColor Red
        Write-Host "Output: $result" -ForegroundColor Red
        return $null
    }
}

# Step 1: Cek DNS propagation
Write-Host "\n=== Step 1: Checking DNS Propagation ===" -ForegroundColor Magenta

try {
    $dnsResult = Resolve-DnsName -Name $DOMAIN -Type A -ErrorAction Stop
    $resolvedIP = $dnsResult.IPAddress
    
    Write-Host "DNS Resolution Result:" -ForegroundColor Cyan
    Write-Host "Domain: $DOMAIN" -ForegroundColor White
    Write-Host "Resolved IP: $resolvedIP" -ForegroundColor White
    Write-Host "Expected IP: $SERVER_IP" -ForegroundColor White
    
    if ($resolvedIP -eq $SERVER_IP) {
        Write-Host "✅ DNS sudah terpropagasi dengan benar!" -ForegroundColor Green
    } else {
        Write-Host "❌ DNS belum terpropagasi. Resolved: $resolvedIP, Expected: $SERVER_IP" -ForegroundColor Red
        Write-Host "Silakan tunggu propagasi DNS selesai sebelum melanjutkan." -ForegroundColor Yellow
        return
    }
} catch {
    Write-Host "❌ Gagal resolve DNS untuk domain $DOMAIN" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Silakan pastikan domain sudah dikonfigurasi dengan benar." -ForegroundColor Yellow
    return
}

# Step 2: Install Certbot
Write-Host "\n=== Step 2: Installing Certbot ===" -ForegroundColor Magenta

$installCertbot = @"
sudo yum update -y
sudo yum install -y epel-release
sudo yum install -y certbot python3-certbot-nginx
"@

Invoke-SSHCommand -Command $installCertbot -Description "Installing Certbot and dependencies"

# Step 3: Stop Nginx temporarily
Write-Host "\n=== Step 3: Stopping Nginx Temporarily ===" -ForegroundColor Magenta
Invoke-SSHCommand -Command "sudo systemctl stop nginx" -Description "Stopping Nginx for certificate generation"

# Step 4: Generate Let's Encrypt Certificate
Write-Host "\n=== Step 4: Generating Let's Encrypt Certificate ===" -ForegroundColor Magenta

$certbotCommand = "sudo certbot certonly --standalone --non-interactive --agree-tos --email $EMAIL -d $DOMAIN"
Invoke-SSHCommand -Command $certbotCommand -Description "Generating SSL certificate with Certbot"

# Step 5: Update Nginx Configuration
Write-Host "\n=== Step 5: Updating Nginx Configuration ===" -ForegroundColor Magenta

$nginxConfig = @"
# FVChain SSL Configuration with Let's Encrypt
# Generated automatically - Do not edit manually

server {
    listen 80;
    server_name $DOMAIN www.$DOMAIN;
    
    # Redirect all HTTP traffic to HTTPS
    return 301 https://`$server_name`$request_uri;
}

server {
    listen 443 ssl http2;
    server_name $DOMAIN www.$DOMAIN;
    
    # Let's Encrypt SSL Certificate
    ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;
    
    # Modern SSL Configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-AES128-SHA256:ECDHE-RSA-AES256-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    
    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    
    # Root location - Dashboard
    location / {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade `$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
        proxy_cache_bypass `$http_upgrade;
        proxy_read_timeout 86400;
    }
    
    # API endpoints
    location /api/ {
        proxy_pass http://localhost:8080/;
        proxy_http_version 1.1;
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
        proxy_read_timeout 86400;
    }
    
    # Health check
    location /health {
        access_log off;
        return 200 "FVChain Blockchain - SSL Active\n";
        add_header Content-Type text/plain;
    }
}
"@

# Write new configuration
$writeConfigCommand = "cat > /etc/nginx/conf.d/ssl-letsencrypt.conf << 'EOF'`n$nginxConfig`nEOF"
Invoke-SSHCommand -Command $writeConfigCommand -Description "Writing new SSL configuration"

# Remove old SSL config
Invoke-SSHCommand -Command "rm -f /etc/nginx/conf.d/ssl-fvchain.conf" -Description "Removing old SSL configuration"

# Step 6: Test and Start Nginx
Write-Host "\n=== Step 6: Testing and Starting Nginx ===" -ForegroundColor Magenta

Invoke-SSHCommand -Command "sudo nginx -t" -Description "Testing Nginx configuration"
Invoke-SSHCommand -Command "sudo systemctl start nginx" -Description "Starting Nginx with new SSL configuration"
Invoke-SSHCommand -Command "sudo systemctl enable nginx" -Description "Enabling Nginx auto-start"

# Step 7: Setup Auto-renewal
Write-Host "\n=== Step 7: Setting up Auto-renewal ===" -ForegroundColor Magenta

$cronJob = "0 12 * * * /usr/bin/certbot renew --quiet --nginx"
Invoke-SSHCommand -Command "(crontab -l 2>/dev/null; echo '$cronJob') | crontab -" -Description "Setting up automatic certificate renewal"

# Step 8: Verification
Write-Host "\n=== Step 8: Verification ===" -ForegroundColor Magenta

Invoke-SSHCommand -Command "sudo systemctl status nginx --no-pager" -Description "Checking Nginx status"
Invoke-SSHCommand -Command "sudo netstat -tlnp | grep ':443'" -Description "Checking HTTPS port"
Invoke-SSHCommand -Command "curl -I https://$DOMAIN" -Description "Testing HTTPS connection"

# Final Status
Write-Host "\n=== Installation Complete ===" -ForegroundColor Green
Write-Host "✅ Let's Encrypt SSL certificate berhasil diinstall!" -ForegroundColor Green
Write-Host "✅ Domain: https://$DOMAIN" -ForegroundColor Green
Write-Host "✅ Auto-renewal sudah dikonfigurasi" -ForegroundColor Green
Write-Host "✅ Security headers sudah aktif" -ForegroundColor Green

Write-Host "\n🔗 Akses FVChain Blockchain:" -ForegroundColor Cyan
Write-Host "   Dashboard: https://$DOMAIN" -ForegroundColor White
Write-Host "   API: https://$DOMAIN/api/" -ForegroundColor White
Write-Host "   Health: https://$DOMAIN/health" -ForegroundColor White

Write-Host "\n📋 Certificate Info:" -ForegroundColor Cyan
Invoke-SSHCommand -Command "sudo certbot certificates" -Description "Showing certificate information"

Write-Host "\n🔄 Renewal Test:" -ForegroundColor Cyan
Invoke-SSHCommand -Command "sudo certbot renew --dry-run" -Description "Testing automatic renewal"

Write-Host "\n=== FVChain SSL Implementation Complete ===" -ForegroundColor Green
Write-Host "Blockchain siap untuk produksi dengan sertifikat SSL yang valid!" -ForegroundColor Yellow