# Fix Nginx Configuration for Whitepaper Download
$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "Fixing Nginx Configuration for Whitepaper Download..." -ForegroundColor Green

# Create a server-side fix script
$serverFixScript = @'
#!/bin/bash

# Fix Nginx Configuration for Whitepaper Download
echo "Fixing Nginx Configuration for Whitepaper Download..."

# 1. Remove invalid configuration files
echo "Removing invalid configuration files..."
rm -f /etc/nginx/conf.d/api-fix.conf
rm -f /etc/nginx/conf.d/whitepaper-direct.conf

# 2. Create a proper Nginx configuration file
echo "Creating proper Nginx configuration file..."

cat > /etc/nginx/conf.d/whitepaper-fix.conf << EOL
server {
    listen 80;
    server_name fvchain.xyz www.fvchain.xyz;

    # Redirect to HTTPS
    location / {
        return 301 https://\$host\$request_uri;
    }
}

server {
    listen 443 ssl;
    server_name fvchain.xyz www.fvchain.xyz;

    # SSL configuration
    ssl_certificate /etc/letsencrypt/live/fvchain.xyz/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fvchain.xyz/privkey.pem;

    # Root directory
    root /var/www/dashboard;

    # Direct whitepaper file access
    location ~ ^/(WHITEPAPER_.*\.md|README\.md|SECURITY\.md)$ {
        root /opt/fvc-dashboard/public;
        try_files \$uri =404;
        add_header Content-Type "text/markdown; charset=utf-8";
        add_header Content-Disposition "inline; filename=\$uri";
        add_header Cache-Control "public, max-age=3600";
        add_header Access-Control-Allow-Origin "*";
    }

    # API routes
    location /api/ {
        proxy_pass http://localhost:3001/api/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
        proxy_read_timeout 300s;
        proxy_connect_timeout 300s;
        proxy_send_timeout 300s;
    }

    # Next.js app
    location / {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
    }
}
EOL

# 3. Test Nginx configuration
echo "Testing Nginx configuration..."
nginx -t

# 4. Reload Nginx if configuration is valid
if [ $? -eq 0 ]; then
    echo "Nginx configuration is valid, reloading..."
    systemctl reload nginx
else
    echo "Nginx configuration is invalid, not reloading"
    exit 1
fi

# 5. Verify whitepaper files exist in public directory
echo "Verifying whitepaper files exist in public directory..."
for file in WHITEPAPER_INDONESIA.md WHITEPAPER_ENGLISH.md README.md SECURITY.md; do
    if [ ! -f "/opt/fvc-dashboard/public/$file" ]; then
        echo "Creating $file in public directory..."
        cp "/opt/fvc-dashboard/$file" "/opt/fvc-dashboard/public/$file" 2>/dev/null || echo "# $file - Placeholder" > "/opt/fvc-dashboard/public/$file"
    fi
    chmod 644 "/opt/fvc-dashboard/public/$file"
    echo "✅ $file exists in public directory with proper permissions"
done

# 6. Restart dashboard service
echo "Restarting dashboard service..."
systemctl restart fvc-dashboard

# 7. Verify services are running
echo "Verifying services are running..."
sleep 5

if systemctl is-active --quiet nginx; then
    echo "✅ Nginx is running"
else
    echo "❌ Nginx is not running"
    systemctl status nginx
fi

if systemctl is-active --quiet fvc-dashboard; then
    echo "✅ Dashboard service is running"
else
    echo "❌ Dashboard service is not running"
    systemctl status fvc-dashboard
fi

echo "Fix completed!"
'@

# Save the server fix script to a temporary file
$serverScriptFile = "fix-nginx-config.sh"
Set-Content -Path $serverScriptFile -Value $serverFixScript

# Upload the server fix script to the server
Write-Host "[INFO] Uploading server fix script..." -ForegroundColor Cyan
echo y | pscp -pw $Password $serverScriptFile $Username@${ServerIP}:/tmp/fix-nginx-config.sh

# Make the server fix script executable and run it
Write-Host "[INFO] Running server fix script..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "chmod +x /tmp/fix-nginx-config.sh && /tmp/fix-nginx-config.sh"

# Remove temporary files
Remove-Item -Path $serverScriptFile

Write-Host "\nNginx configuration fix completed!" -ForegroundColor Green
Write-Host "You can verify the changes at: https://fvchain.xyz/WHITEPAPER_INDONESIA.md" -ForegroundColor Yellow