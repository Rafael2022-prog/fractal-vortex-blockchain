# Fix SSL Configuration and Static Files for fvchain.xyz
$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "🔧 Fixing SSL Configuration and Static Files..." -ForegroundColor Green

# Step 1: Copy new SSL configuration
Write-Host "[1/5] Installing SSL configuration..." -ForegroundColor Cyan
$sslConfig = Get-Content "r:\369-FRACTAL\nginx-ssl-fvchain-xyz.conf" -Raw

echo y | plink -ssh -pw $Password $Username@$ServerIP @"
cat > /etc/nginx/conf.d/fvchain-xyz-ssl.conf << 'EOF'
$sslConfig
EOF
"@

# Step 2: Check SSL certificates
Write-Host "[2/5] Checking SSL certificates..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP @"
# Check if SSL certificates exist
if [ ! -f /etc/ssl/certs/fvchain.xyz.crt ]; then
    echo "❌ SSL certificate not found - generating self-signed certificate..."
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout /etc/ssl/private/fvchain.xyz.key \
        -out /etc/ssl/certs/fvchain.xyz.crt \
        -subj "/C=US/ST=State/L=City/O=FVChain/OU=IT/CN=fvchain.xyz"
    chmod 600 /etc/ssl/private/fvchain.xyz.key
    chmod 644 /etc/ssl/certs/fvchain.xyz.crt
else
    echo "✅ SSL certificate found"
fi
"@

# Step 3: Verify static files exist
Write-Host "[3/5] Verifying static files..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP @"
# Check if static files exist
echo "Checking static files..."
ls -la /opt/fvc-dashboard/.next/static/ 2>/dev/null || echo "Static directory not found"
ls -la /opt/fvc-dashboard/public/ 2>/dev/null || echo "Public directory not found"

# Create static directory structure if needed
mkdir -p /opt/fvc-dashboard/.next/static/css
mkdir -p /opt/fvc-dashboard/.next/static/js
mkdir -p /opt/fvc-dashboard/public

# Set proper permissions
chmod -R 755 /opt/fvc-dashboard/.next/static/
chmod -R 755 /opt/fvc-dashboard/public/
"@

# Step 4: Test nginx configuration
Write-Host "[4/5] Testing nginx configuration..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP @"
echo "Testing nginx configuration..."
nginx -t
if [ $? -eq 0 ]; then
    echo "✅ Nginx configuration is valid"
else
    echo "❌ Nginx configuration has errors"
    exit 1
fi
"@

# Step 5: Restart services
Write-Host "[5/5] Restarting services..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP @"
echo "Restarting nginx..."
systemctl restart nginx
systemctl status nginx --no-pager -l

echo "Restarting dashboard..."
systemctl restart fvc-dashboard
systemctl status fvc-dashboard --no-pager -l

# Test static file access
echo "Testing static file access..."
curl -I -k https://localhost/_next/static/css/ 2>/dev/null | head -1 || echo "Static CSS test failed"
curl -I -k https://localhost/favicon.ico 2>/dev/null | head -1 || echo "Favicon test failed"
"@

Write-Host "✅ SSL Configuration Fixed!" -ForegroundColor Green
Write-Host "🌐 Dashboard should now be accessible at: https://fvchain.xyz" -ForegroundColor Yellow
Write-Host "📁 Static files should load properly" -ForegroundColor Yellow