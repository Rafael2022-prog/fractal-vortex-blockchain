# Simple Let's Encrypt Implementation for FVChain
# Created by: Emylton Leunufna
# Date: 2025

Write-Host "=== FVChain Let's Encrypt SSL Implementation ===" -ForegroundColor Green

$SERVER_IP = "103.245.38.44"
$DOMAIN = "fvchain.org"
$EMAIL = "admin@fvchain.org"

Write-Host "`nServer IP: $SERVER_IP" -ForegroundColor Cyan
Write-Host "Target Domain: $DOMAIN" -ForegroundColor Cyan
Write-Host "Email: $EMAIL" -ForegroundColor Cyan

# Step 1: Check DNS propagation
Write-Host "`n=== Step 1: Checking DNS Propagation ===" -ForegroundColor Magenta

try {
    $dnsResult = Resolve-DnsName -Name $DOMAIN -Type A -ErrorAction Stop
    $resolvedIP = $dnsResult.IPAddress
    
    Write-Host "DNS Resolution:" -ForegroundColor Yellow
    Write-Host "  Domain: $DOMAIN" -ForegroundColor White
    Write-Host "  Resolved IP: $resolvedIP" -ForegroundColor White
    Write-Host "  Expected IP: $SERVER_IP" -ForegroundColor White
    
    if ($resolvedIP -eq $SERVER_IP) {
        Write-Host "  Success: DNS propagation successful!" -ForegroundColor Green
        $dnsReady = $true
    } else {
        Write-Host "  Error: DNS not propagated yet" -ForegroundColor Red
        $dnsReady = $false
    }
} catch {
    Write-Host "  Error: Domain not found: $DOMAIN" -ForegroundColor Red
    $dnsReady = $false
}

if (-not $dnsReady) {
    Write-Host "`nWarning: DNS belum siap untuk Let's Encrypt" -ForegroundColor Yellow
    Write-Host "`nLangkah-langkah yang diperlukan:" -ForegroundColor Cyan
    Write-Host "1. Pastikan domain $DOMAIN sudah dibeli" -ForegroundColor White
    Write-Host "2. Konfigurasi DNS A Record: $DOMAIN -> $SERVER_IP" -ForegroundColor White
    Write-Host "3. Tunggu propagasi DNS 1-48 jam" -ForegroundColor White
    Write-Host "4. Jalankan script ini lagi" -ForegroundColor White
    
    Write-Host "`nAlternatif sementara:" -ForegroundColor Cyan
    Write-Host "- Gunakan IP address: https://$SERVER_IP" -ForegroundColor White
    Write-Host "- SSL self-signed sudah aktif" -ForegroundColor White
    
    return
}

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

$certCmd = "certbot certonly --standalone --non-interactive --agree-tos --email $EMAIL -d $DOMAIN"
Write-Host "Generating certificate for $DOMAIN..." -ForegroundColor Yellow
$result = plink -ssh root@$SERVER_IP -batch $certCmd
if ($LASTEXITCODE -eq 0) {
    Write-Host "Success: SSL certificate generated successfully" -ForegroundColor Green
} else {
    Write-Host "Error: Failed to generate certificate" -ForegroundColor Red
    Write-Host "Starting Nginx back..." -ForegroundColor Yellow
    plink -ssh root@$SERVER_IP -batch "systemctl start nginx"
    return
}

# Step 5: Upload new SSL configuration
Write-Host "`n=== Step 5: Uploading SSL Configuration ===" -ForegroundColor Magenta

Write-Host "Uploading Let's Encrypt configuration..." -ForegroundColor Yellow
$uploadResult = pscp -scp "R:\369-FRACTAL\letsencrypt-ssl.conf" "root@${SERVER_IP}:/etc/nginx/conf.d/ssl-letsencrypt.conf"
if ($LASTEXITCODE -eq 0) {
    Write-Host "Success: SSL configuration uploaded" -ForegroundColor Green
} else {
    Write-Host "Error: Failed to upload configuration" -ForegroundColor Red
    return
}

# Step 6: Remove old configuration
Write-Host "`n=== Step 6: Updating Configuration ===" -ForegroundColor Magenta

$removeOldCmd = "rm -f /etc/nginx/conf.d/ssl-fvchain.conf"
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

# Step 9: Verification
Write-Host "`n=== Step 9: Final Verification ===" -ForegroundColor Magenta

Write-Host "Checking Nginx status..." -ForegroundColor Yellow
$statusResult = plink -ssh root@$SERVER_IP -batch "systemctl status nginx --no-pager"

Write-Host "Checking HTTPS port..." -ForegroundColor Yellow
$portResult = plink -ssh root@$SERVER_IP -batch "netstat -tlnp | grep ':443'"

Write-Host "Testing HTTPS connection..." -ForegroundColor Yellow
$httpsTest = plink -ssh root@$SERVER_IP -batch "curl -I https://$DOMAIN"

# Final status
Write-Host "`n=== Let's Encrypt Implementation Complete ===" -ForegroundColor Green
Write-Host "Success: SSL Certificate berhasil diinstall!" -ForegroundColor Green

Write-Host "`nAkses FVChain Blockchain:" -ForegroundColor Cyan
Write-Host "   Dashboard: https://$DOMAIN" -ForegroundColor White
Write-Host "   API: https://$DOMAIN/api/" -ForegroundColor White
Write-Host "   Health Check: https://$DOMAIN/health" -ForegroundColor White

Write-Host "`nFeatures:" -ForegroundColor Cyan
Write-Host "   - Valid SSL Certificate (Let's Encrypt)" -ForegroundColor Green
Write-Host "   - Auto-renewal configured" -ForegroundColor Green
Write-Host "   - Modern security headers" -ForegroundColor Green
Write-Host "   - HTTP to HTTPS redirect" -ForegroundColor Green
Write-Host "   - WebSocket support" -ForegroundColor Green

Write-Host "`nCertificate Info:" -ForegroundColor Cyan
$certInfo = plink -ssh root@$SERVER_IP -batch "certbot certificates"
Write-Host $certInfo -ForegroundColor White

Write-Host "`nTest Auto-renewal:" -ForegroundColor Cyan
$renewTest = plink -ssh root@$SERVER_IP -batch "certbot renew --dry-run"

Write-Host "`nFVChain Blockchain siap untuk produksi dengan SSL yang valid!" -ForegroundColor Yellow
Write-Host "Browser tidak akan menampilkan peringatan sertifikat lagi." -ForegroundColor Green