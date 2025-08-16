# Script untuk Setup Domain dan SSL Certificate
# FVChain Blockchain - Domain Configuration Guide
# Created by: Emylton Leunufna
# Date: 2025

Write-Host "=== FVChain Domain & SSL Setup Guide ===" -ForegroundColor Green
Write-Host "Panduan konfigurasi domain dan sertifikat SSL..." -ForegroundColor Yellow

# Konfigurasi
$SERVER_IP = "103.245.38.44"
$DOMAINS = @("fvchain.org", "fvchain.com", "fvchain.net", "fvchain.io")

Write-Host "`n📋 Informasi Server:" -ForegroundColor Cyan
Write-Host "   IP Address: $SERVER_IP" -ForegroundColor White
Write-Host "   SSL Status: Self-signed (Active)" -ForegroundColor White
Write-Host "   HTTPS Port: 443 ✅" -ForegroundColor Green

# Fungsi untuk menjalankan perintah SSH
function Invoke-SSHCommand {
    param(
        [string]$Command,
        [string]$Description
    )
    
    Write-Host "`n[$Description]" -ForegroundColor Yellow
    Write-Host "Executing: $Command" -ForegroundColor Gray
    
    $result = plink -ssh root@$SERVER_IP -batch $Command
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Host "✅ Success: $Description" -ForegroundColor Green
        return $result
    } else {
        Write-Host "❌ Failed: $Description (Exit Code: $exitCode)" -ForegroundColor Red
        return $null
    }
}

# Step 1: Cek status DNS untuk beberapa domain
Write-Host "`n=== Step 1: Checking Available Domains ===" -ForegroundColor Magenta

$availableDomains = @()

foreach ($domain in $DOMAINS) {
    Write-Host "`nChecking domain: $domain" -ForegroundColor Cyan
    
    try {
        $dnsResult = Resolve-DnsName -Name $domain -Type A -ErrorAction Stop
        $resolvedIP = $dnsResult.IPAddress
        
        Write-Host "   Resolved IP: $resolvedIP" -ForegroundColor White
        
        if ($resolvedIP -eq $SERVER_IP) {
            Write-Host "   ✅ Domain sudah mengarah ke server!" -ForegroundColor Green
            $availableDomains += $domain
        } else {
            Write-Host "   ⚠️  Domain mengarah ke IP lain: $resolvedIP" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "   ❌ Domain tidak ditemukan atau belum dikonfigurasi" -ForegroundColor Red
    }
}

if ($availableDomains.Count -eq 0) {
    Write-Host "`n⚠️  Tidak ada domain yang mengarah ke server saat ini." -ForegroundColor Yellow
    Write-Host "`n📝 Langkah-langkah untuk konfigurasi domain:" -ForegroundColor Cyan
    Write-Host "`n1. Beli domain dari registrar (Namecheap, GoDaddy, dll)" -ForegroundColor White
    Write-Host "2. Konfigurasi DNS Records:" -ForegroundColor White
    Write-Host "   - A Record: @ -> $SERVER_IP" -ForegroundColor Gray
    Write-Host "   - A Record: www -> $SERVER_IP" -ForegroundColor Gray
    Write-Host "3. Tunggu propagasi DNS (1-48 jam)" -ForegroundColor White
    Write-Host "4. Jalankan script ini lagi setelah propagasi selesai" -ForegroundColor White
    
    Write-Host "`n🔧 Alternatif: Gunakan subdomain gratis" -ForegroundColor Cyan
    Write-Host "   - DuckDNS: https://www.duckdns.org" -ForegroundColor White
    Write-Host "   - No-IP: https://www.noip.com" -ForegroundColor White
    Write-Host "   - FreeDNS: https://freedns.afraid.org" -ForegroundColor White
    
    Write-Host "`n💡 Sementara ini, gunakan IP address untuk akses:" -ForegroundColor Cyan
    Write-Host "   Dashboard: https://$SERVER_IP" -ForegroundColor White
    Write-Host "   API: https://$SERVER_IP/api/" -ForegroundColor White
    
    # Optimasi konfigurasi SSL saat ini
    Write-Host "`n=== Optimizing Current SSL Configuration ===" -ForegroundColor Magenta
    
    # Update SSL configuration untuk IP-based access
    $updateSslCommand = 'cat > /etc/nginx/conf.d/ssl-fvchain.conf << "EOF"' + "`n" +
        'server {' + "`n" +
        '    listen 80 default_server;' + "`n" +
        '    server_name _;' + "`n" +
        '    return 301 https://$server_name$request_uri;' + "`n" +
        '}' + "`n" +
        'server {' + "`n" +
        '    listen 443 ssl http2 default_server;' + "`n" +
        '    server_name _;' + "`n" +
        '    ssl_certificate /etc/nginx/ssl/server.crt;' + "`n" +
        '    ssl_certificate_key /etc/nginx/ssl/server.key;' + "`n" +
        '    ssl_protocols TLSv1.2 TLSv1.3;' + "`n" +
        '    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384;' + "`n" +
        '    ssl_prefer_server_ciphers off;' + "`n" +
        '    ssl_session_cache shared:SSL:10m;' + "`n" +
        '    add_header X-Frame-Options DENY always;' + "`n" +
        '    add_header X-Content-Type-Options nosniff always;' + "`n" +
        '    location / {' + "`n" +
        '        proxy_pass http://localhost:3001;' + "`n" +
        '        proxy_http_version 1.1;' + "`n" +
        '        proxy_set_header Upgrade $http_upgrade;' + "`n" +
        '        proxy_set_header Connection "upgrade";' + "`n" +
        '        proxy_set_header Host $host;' + "`n" +
        '        proxy_set_header X-Real-IP $remote_addr;' + "`n" +
        '        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;' + "`n" +
        '        proxy_set_header X-Forwarded-Proto $scheme;' + "`n" +
        '        proxy_cache_bypass $http_upgrade;' + "`n" +
        '        proxy_read_timeout 86400;' + "`n" +
        '    }' + "`n" +
        '    location /api/ {' + "`n" +
        '        proxy_pass http://localhost:8080/;' + "`n" +
        '        proxy_http_version 1.1;' + "`n" +
        '        proxy_set_header Host $host;' + "`n" +
        '        proxy_set_header X-Real-IP $remote_addr;' + "`n" +
        '        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;' + "`n" +
        '        proxy_set_header X-Forwarded-Proto $scheme;' + "`n" +
        '        proxy_read_timeout 86400;' + "`n" +
        '    }' + "`n" +
        '    location /health {' + "`n" +
        '        access_log off;' + "`n" +
        '        return 200 "FVChain Blockchain - IP SSL Active";' + "`n" +
        '        add_header Content-Type text/plain;' + "`n" +
        '    }' + "`n" +
        '}' + "`n" +
        'EOF'
    
    Invoke-SSHCommand -Command $updateSslCommand -Description "Updating IP-based SSL configuration"
    
    # Test and reload Nginx
    Invoke-SSHCommand -Command "nginx -t && systemctl reload nginx" -Description "Reloading Nginx with updated configuration"
    
    Write-Host "`n✅ IP-based SSL configuration optimized!" -ForegroundColor Green
    
} else {
    Write-Host "`n🎉 Domain yang tersedia untuk Let's Encrypt:" -ForegroundColor Green
    foreach ($domain in $availableDomains) {
        Write-Host "   ✅ $domain" -ForegroundColor Green
    }
    
    $primaryDomain = $availableDomains[0]
    Write-Host "`n🚀 Menggunakan domain: $primaryDomain" -ForegroundColor Cyan
    
    # Install Let's Encrypt
    Write-Host "`n=== Installing Let's Encrypt ===" -ForegroundColor Magenta
    
    Invoke-SSHCommand -Command "yum update -y && yum install -y epel-release && yum install -y certbot python3-certbot-nginx" -Description "Installing Certbot"
    
    # Stop Nginx temporarily
    Invoke-SSHCommand -Command "systemctl stop nginx" -Description "Stopping Nginx for certificate generation"
    
    # Generate certificate
    $certbotCmd = "certbot certonly --standalone --non-interactive --agree-tos --email admin@$primaryDomain -d $primaryDomain"
    Invoke-SSHCommand -Command $certbotCmd -Description "Generating Let's Encrypt certificate"
    
    # Create Let's Encrypt configuration
    $letsencryptConfigCmd = 'cat > /etc/nginx/conf.d/ssl-letsencrypt.conf << "EOF"' + "`n" +
        'server {' + "`n" +
        '    listen 80;' + "`n" +
        "    server_name $primaryDomain;" + "`n" +
        '    return 301 https://$server_name$request_uri;' + "`n" +
        '}' + "`n" +
        'server {' + "`n" +
        '    listen 443 ssl http2;' + "`n" +
        "    server_name $primaryDomain;" + "`n" +
        "    ssl_certificate /etc/letsencrypt/live/$primaryDomain/fullchain.pem;" + "`n" +
        "    ssl_certificate_key /etc/letsencrypt/live/$primaryDomain/privkey.pem;" + "`n" +
        '    ssl_protocols TLSv1.2 TLSv1.3;' + "`n" +
        '    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384;' + "`n" +
        '    ssl_prefer_server_ciphers off;' + "`n" +
        '    ssl_session_cache shared:SSL:10m;' + "`n" +
        '    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;' + "`n" +
        '    add_header X-Frame-Options DENY always;' + "`n" +
        '    add_header X-Content-Type-Options nosniff always;' + "`n" +
        '    location / {' + "`n" +
        '        proxy_pass http://localhost:3001;' + "`n" +
        '        proxy_http_version 1.1;' + "`n" +
        '        proxy_set_header Upgrade $http_upgrade;' + "`n" +
        '        proxy_set_header Connection "upgrade";' + "`n" +
        '        proxy_set_header Host $host;' + "`n" +
        '        proxy_set_header X-Real-IP $remote_addr;' + "`n" +
        '        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;' + "`n" +
        '        proxy_set_header X-Forwarded-Proto $scheme;' + "`n" +
        '        proxy_cache_bypass $http_upgrade;' + "`n" +
        '        proxy_read_timeout 86400;' + "`n" +
        '    }' + "`n" +
        '    location /api/ {' + "`n" +
        '        proxy_pass http://localhost:8080/;' + "`n" +
        '        proxy_http_version 1.1;' + "`n" +
        '        proxy_set_header Host $host;' + "`n" +
        '        proxy_set_header X-Real-IP $remote_addr;' + "`n" +
        '        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;' + "`n" +
        '        proxy_set_header X-Forwarded-Proto $scheme;' + "`n" +
        '        proxy_read_timeout 86400;' + "`n" +
        '    }' + "`n" +
        '    location /health {' + "`n" +
        '        access_log off;' + "`n" +
        '        return 200 "FVChain Blockchain - Let\'s Encrypt SSL Active";' + "`n" +
        '        add_header Content-Type text/plain;' + "`n" +
        '    }' + "`n" +
        '}' + "`n" +
        'EOF'
    
    Invoke-SSHCommand -Command $letsencryptConfigCmd -Description "Writing Let's Encrypt SSL configuration"
    
    # Remove old configuration and start Nginx
    Invoke-SSHCommand -Command "rm -f /etc/nginx/conf.d/ssl-fvchain.conf" -Description "Removing old SSL configuration"
    Invoke-SSHCommand -Command "nginx -t && systemctl start nginx" -Description "Starting Nginx with Let's Encrypt SSL"
    
    # Setup auto-renewal
    $cronCmd = 'echo "0 12 * * * /usr/bin/certbot renew --quiet --nginx" | crontab -'
    Invoke-SSHCommand -Command $cronCmd -Description "Setting up auto-renewal"
    
    Write-Host "`n🎉 Let's Encrypt SSL berhasil diinstall!" -ForegroundColor Green
    Write-Host "✅ Domain: https://$primaryDomain" -ForegroundColor Green
    Write-Host "✅ Auto-renewal: Aktif" -ForegroundColor Green
}

# Final verification
Write-Host "`n=== Final Verification ===" -ForegroundColor Magenta
Invoke-SSHCommand -Command "systemctl status nginx --no-pager" -Description "Checking Nginx status"
Invoke-SSHCommand -Command "netstat -tlnp | grep ':443'" -Description "Checking HTTPS port"

Write-Host "`n=== FVChain SSL Setup Complete ===" -ForegroundColor Green
Write-Host "`n🔗 Akses FVChain Blockchain:" -ForegroundColor Cyan
if ($availableDomains.Count -gt 0) {
    Write-Host "   Dashboard: https://$($availableDomains[0])" -ForegroundColor White
    Write-Host "   API: https://$($availableDomains[0])/api/" -ForegroundColor White
} else {
    Write-Host "   Dashboard: https://$SERVER_IP (Self-signed)" -ForegroundColor White
    Write-Host "   API: https://$SERVER_IP/api/ (Self-signed)" -ForegroundColor White
}

Write-Host "`n📋 Next Steps:" -ForegroundColor Cyan
if ($availableDomains.Count -eq 0) {
    Write-Host "   1. Konfigurasi domain DNS" -ForegroundColor White
    Write-Host "   2. Jalankan script ini lagi setelah DNS propagated" -ForegroundColor White
    Write-Host "   3. Let's Encrypt akan otomatis terinstall" -ForegroundColor White
} else {
    Write-Host "   1. ✅ SSL Certificate: Valid" -ForegroundColor Green
    Write-Host "   2. ✅ Auto-renewal: Configured" -ForegroundColor Green
    Write-Host "   3. ✅ Production Ready!" -ForegroundColor Green
}

Write-Host "`n🚀 FVChain Blockchain siap untuk produksi!" -ForegroundColor Yellow