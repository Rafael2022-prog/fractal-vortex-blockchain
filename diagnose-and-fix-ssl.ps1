#!/bin/bash

# Diagnose and Fix SSL Implementation
echo "=== SSL Diagnosis and Fix Script ==="

# Step 1: Check current nginx status
echo "[STEP 1] Checking nginx status..."
systemctl status nginx --no-pager -l

# Step 2: Check what ports nginx is listening on
echo "[STEP 2] Checking nginx listening ports..."
ss -tlnp | grep nginx

# Step 3: Check nginx configuration syntax
echo "[STEP 3] Testing nginx configuration..."
nginx -t

# Step 4: Check SSL certificate files
echo "[STEP 4] Checking SSL certificate files..."
ls -la /etc/ssl/certs/fvchain.crt /etc/ssl/private/fvchain.key

# Step 5: Check nginx configuration files
echo "[STEP 5] Checking nginx configuration files..."
ls -la /etc/nginx/conf.d/

# Step 6: Show actual nginx configuration being loaded
echo "[STEP 6] Showing loaded nginx configuration..."
nginx -T | grep -A 20 -B 5 'listen 443'

# Step 7: Check for conflicting configurations
echo "[STEP 7] Checking for conflicting configurations..."
grep -r "listen 443" /etc/nginx/

# Step 8: Remove any conflicting configurations and create clean SSL config
echo "[STEP 8] Creating clean SSL configuration..."

# Backup existing configs
cp /etc/nginx/conf.d/fvchain-ssl.conf /etc/nginx/conf.d/fvchain-ssl.conf.backup 2>/dev/null || true

# Remove all existing configs
rm -f /etc/nginx/conf.d/fvc-*.conf
rm -f /etc/nginx/conf.d/fvchain*.conf
rm -f /etc/nginx/sites-enabled/default 2>/dev/null || true

# Create new clean SSL configuration
cat > /etc/nginx/conf.d/ssl-fvchain.conf << 'EOF'
# HTTP to HTTPS redirect
server {
    listen 80 default_server;
    listen [::]:80 default_server;
    server_name _;
    return 301 https://$host$request_uri;
}

# HTTPS server
server {
    listen 443 ssl http2 default_server;
    listen [::]:443 ssl http2 default_server;
    server_name _;

    # SSL Configuration
    ssl_certificate /etc/ssl/certs/fvchain.crt;
    ssl_certificate_key /etc/ssl/private/fvchain.key;
    
    # SSL Settings
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-AES128-SHA256:ECDHE-RSA-AES256-SHA384:ECDHE-RSA-AES128-SHA:ECDHE-RSA-AES256-SHA:DHE-RSA-AES128-SHA256:DHE-RSA-AES256-SHA256:AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA256:AES256-SHA256:AES128-SHA:AES256-SHA:DES-CBC3-SHA;
    ssl_prefer_server_ciphers on;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Dashboard proxy
    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # API proxy
    location /api/ {
        proxy_pass http://127.0.0.1:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Wallet proxy
    location /wallet/ {
        proxy_pass http://127.0.0.1:8080/wallet/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
EOF

# Step 9: Test new configuration
echo "[STEP 9] Testing new nginx configuration..."
nginx -t

if [ $? -eq 0 ]; then
    echo "Configuration test passed!"
    
    # Step 10: Restart nginx
    echo "[STEP 10] Restarting nginx..."
    systemctl stop nginx
    sleep 2
    systemctl start nginx
    sleep 3
    
    # Step 11: Verify nginx is running and listening on 443
    echo "[STEP 11] Verifying nginx status..."
    systemctl status nginx --no-pager
    echo "\nListening ports:"
    ss -tlnp | grep nginx
    
    # Step 12: Test HTTPS connection
    echo "[STEP 12] Testing HTTPS connection..."
    curl -k -I https://localhost || echo "HTTPS connection failed"
    curl -k -I https://103.245.38.44 || echo "External HTTPS connection failed"
    
else
    echo "Configuration test failed! Check the error above."
    exit 1
fi

echo "\n=== SSL Fix Complete ==="
echo "If HTTPS is working, you can access:"
echo "- https://103.245.38.44 (direct IP)"
echo "- https://www.fvchain.xyz (when DNS propagates)"
echo "\nNote: Browser will show security warning for self-signed certificate."
echo "Click 'Advanced' -> 'Proceed to site' to continue."