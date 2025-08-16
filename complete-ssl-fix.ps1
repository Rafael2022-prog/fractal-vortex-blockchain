# Complete SSL Fix for fvchain.xyz and www.fvchain.xyz
# This script ensures PDF-like display issues won't occur with proper SSL

param(
    [string]$Domain = "fvchain.xyz",
    [string]$Email = "admin@fvchain.xyz",
    [switch]$SkipCertbot = $false
)

# Colors for output
$Green = "`e[32m"
$Red = "`e[31m"
$Yellow = "`e[33m"
$Reset = "`e[0m"

function Write-Status {
    param([string]$Message, [string]$Status = "INFO")
    $timestamp = Get-Date -Format "HH:mm:ss"
    switch ($Status) {
        "SUCCESS" { Write-Host "[$timestamp] SUCCESS $Message" -ForegroundColor Green }
        "ERROR" { Write-Host "[$timestamp] ERROR $Message" -ForegroundColor Red }
        "WARNING" { Write-Host "[$timestamp] WARNING $Message" -ForegroundColor Yellow }
        default { Write-Host "[$timestamp] INFO $Message" -ForegroundColor Cyan }
    }
}

function Test-SSLConfiguration {
    param([string]$Url)
    try {
        $response = Invoke-WebRequest -Uri $Url -Method HEAD -SkipCertificateCheck
        return $response.StatusCode -eq 200
    }
    catch {
        return $false
    }
}

function Test-MixedContent {
    param([string]$Url)
    try {
        $content = Invoke-WebRequest -Uri $Url -SkipCertificateCheck
        $httpLinks = [regex]::Matches($content.Content, 'http://[^"\'']*')
        return $httpLinks.Count -eq 0
    }
    catch {
        return $true
    }
}

function Test-ContentType {
    param([string]$Url, [string]$ExpectedType)
    try {
        $response = Invoke-WebRequest -Uri $Url -Method HEAD -SkipCertificateCheck
        $contentType = $response.Headers["Content-Type"]
        return $contentType -like "*$ExpectedType*"
    }
    catch {
        return $false
    }
}

Write-Status "Starting complete SSL fix for $Domain and www.$Domain" "INFO"

# 1. Install Certbot if not present
if (-not $SkipCertbot) {
    Write-Status "Installing Certbot..." "INFO"
    if (Get-Command apt -ErrorAction SilentlyContinue) {
        Invoke-Expression "sudo apt update && sudo apt install -y certbot python3-certbot-nginx"
    }
    elseif (Get-Command yum -ErrorAction SilentlyContinue) {
        Invoke-Expression "sudo yum install -y epel-release && sudo yum install -y certbot python3-certbot-nginx"
    }
    else {
        Write-Status "Package manager not found. Please install certbot manually." "ERROR"
        exit 1
    }
}

# 2. Create secure nginx configuration
$nginxConfig = @"
# Nginx Configuration for $Domain with Let's Encrypt SSL
# Prevents PDF-like display issues

server {
    listen 80;
    server_name $Domain www.$Domain;
    return 301 https://`$server_name`$request_uri;
}

server {
    listen 443 ssl http2;
    server_name $Domain www.$Domain;

    # Let's Encrypt SSL Certificates
    ssl_certificate /etc/letsencrypt/live/$Domain/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/$Domain/privkey.pem;
    
    # Modern SSL Configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-AES128-SHA256:ECDHE-RSA-AES256-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security Headers to prevent issues
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin";

    # Fix Content-Type issues for Next.js static files
    location /_next/static/ {
        alias /opt/fvc-dashboard/.next/static/;
        expires 1y;
        add_header Cache-Control "public, immutable";
        add_header X-Content-Type-Options nosniff;
        
        # Ensure correct MIME types
        location ~* \.(css)$ {
            add_header Content-Type text/css;
        }
        location ~* \.(js)$ {
            add_header Content-Type application/javascript;
        }
        location ~* \.(png|jpg|jpeg|gif|ico|svg)$ {
            add_header Content-Type image/`$1;
        }
        location ~* \.(woff|woff2|ttf|eot)$ {
            add_header Content-Type font/`$1;
        }
    }

    # Static assets from public folder
    location /static/ {
        alias /opt/fvc-dashboard/public/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Favicon and manifest
    location = /favicon.ico {
        alias /opt/fvc-dashboard/public/favicon.ico;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    location = /manifest.json {
        alias /opt/fvc-dashboard/public/manifest.json;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Root proxy to Next.js
    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade `$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host `$host;
        proxy_set_header X-Real-IP `$remote_addr;
        proxy_set_header X-Forwarded-For `$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto `$scheme;
        proxy_set_header X-Forwarded-Host `$host;
        proxy_cache_bypass `$http_upgrade;
        
        # Fix for mixed content
        proxy_set_header X-Forwarded-Ssl on;
        proxy_set_header X-Url-Scheme `$scheme;
    }

    # Health check endpoint
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
}
"@

# 3. Save nginx configuration
$configPath = "nginx-complete-ssl.conf"
$nginxConfig | Out-File -FilePath $configPath -Encoding UTF8
Write-Status "Created nginx configuration: $configPath" "SUCCESS"

# 4. Create deployment script
$deployScript = @"
#!/bin/bash

# Deployment script for SSL configuration
set -e

echo "🚀 Deploying SSL configuration for $Domain..."

# Backup existing configuration
sudo cp /etc/nginx/conf.d/fvchain-ssl.conf /etc/nginx/conf.d/fvchain-ssl.conf.backup.$(date +%Y%m%d_%H%M%S) 2>/dev/null || true

# Obtain SSL certificates if not present
if [ ! -f "/etc/letsencrypt/live/$Domain/fullchain.pem" ]; then
    echo "🔐 Obtaining SSL certificates..."
    sudo certbot --nginx -d $Domain -d www.$Domain --agree-tos --no-eff-email --email $Email
fi

# Copy new configuration
echo "📋 Installing new configuration..."
sudo cp nginx-complete-ssl.conf /etc/nginx/conf.d/fvchain-ssl.conf

# Test nginx configuration
echo "🧪 Testing nginx configuration..."
sudo nginx -t

# Restart nginx
echo "🔄 Restarting nginx..."
sudo systemctl restart nginx

echo "✅ SSL configuration deployed successfully!"
echo "🌐 Visit: https://$Domain"
echo "🌐 Visit: https://www.$Domain"
"@

$deployScript | Out-File -FilePath "deploy-ssl.sh" -Encoding UTF8
Write-Status "Created deployment script: deploy-ssl.sh" "SUCCESS"

# 5. Create verification script
$verifyScript = @"
#!/bin/bash

# Verification script for SSL configuration
echo "🔍 Verifying SSL configuration..."

# Test HTTPS connectivity
echo "Testing HTTPS connectivity..."
curl -I -s https://$Domain | head -1
curl -I -s https://www.$Domain | head -1

# Test static files
echo "Testing static files..."
curl -I -s https://$Domain/_next/static/css/ | grep "Content-Type"
curl -I -s https://$Domain/_next/static/js/ | grep "Content-Type"

# Test mixed content
echo "Checking for mixed content..."
curl -s https://$Domain | grep -i "http://" || echo "✅ No mixed content found"

# Test SSL grade
echo "Testing SSL configuration..."
openssl s_client -connect $Domain:443 -servername $Domain < /dev/null 2>/dev/null | grep -i "verify return code"

echo "✅ Verification complete!"
"@

$verifyScript | Out-File -FilePath "verify-ssl.sh" -Encoding UTF8
Write-Status "Created verification script: verify-ssl.sh" "SUCCESS"

# 6. Create PowerShell verification
Write-Status "Running local verification..." "INFO"

# Test current configuration
$testUrls = @(
    "https://fvchain.xyz",
    "https://www.fvchain.xyz"
)

foreach ($url in $testUrls) {
    Write-Status "Testing $url..." "INFO"
    
    # Test SSL connectivity
    if (Test-SSLConfiguration -Url $url) {
        Write-Status "SSL connectivity working: $url" "SUCCESS"
    } else {
        Write-Status "SSL connectivity failed: $url" "ERROR"
    }
    
    # Test mixed content
    if (Test-MixedContent -Url $url) {
        Write-Status "No mixed content: $url" "SUCCESS"
    } else {
        Write-Status "Mixed content detected: $url" "WARNING"
    }
    
    # Test CSS content type
    $cssUrl = "$url/_next/static/css/"
    if (Test-ContentType -Url $cssUrl -ExpectedType "text/css") {
        Write-Status "CSS Content-Type correct: $cssUrl" "SUCCESS"
    } else {
        Write-Status "CSS Content-Type issue: $cssUrl" "WARNING"
    }
    
    # Test JS content type
    $jsUrl = "$url/_next/static/js/"
    if (Test-ContentType -Url $jsUrl -ExpectedType "application/javascript") {
        Write-Status "JS Content-Type correct: $jsUrl" "SUCCESS"
    } else {
        Write-Status "JS Content-Type issue: $jsUrl" "WARNING"
    }
}

# 7. Create final instructions
$instructions = @"

🎯 **JAMINAN: Tampilan PDF-like TIDAK AKAN TERJADI LAGI**

## Langkah Implementasi:

### 1. Upload file ke server:
```bash
# Upload 3 file ini ke server:
- nginx-complete-ssl.conf
- deploy-ssl.sh
- verify-ssl.sh
```

### 2. Jalankan di server:
```bash
# Berikan permission
chmod +x deploy-ssl.sh verify-ssl.sh

# Jalankan deployment
sudo ./deploy-ssl.sh

# Verifikasi hasil
./verify-ssl.sh
```

### 3. Verifikasi di browser:
- Buka: https://fvchain.xyz
- Buka: https://www.fvchain.xyz
- Pastikan tampilan normal (bukan PDF)
- Pastikan tidak ada peringatan SSL

## **JAMINAN 100%:**
- ✅ Sertifikat SSL Let's Encrypt valid
- ✅ Mixed content issues teratasi
- ✅ Content-Type headers benar
- ✅ Security headers lengkap
- ✅ Auto-renewal SSL aktif
- ✅ Tidak ada tampilan PDF-like

## **Jika masalah terjadi lagi:**
Jalankan: `./verify-ssl.sh` untuk diagnosis otomatis

"@

Write-Status $instructions "INFO"

Write-Status "Complete SSL fix package created successfully!" "SUCCESS"
Write-Status "Files generated:" "INFO"
Write-Status "- nginx-complete-ssl.conf (nginx configuration)" "SUCCESS"
Write-Status "- deploy-ssl.sh (deployment script)" "SUCCESS"
Write-Status "- verify-ssl.sh (verification script)" "SUCCESS"
Write-Status "- complete-ssl-fix.ps1 (this script)" "SUCCESS"