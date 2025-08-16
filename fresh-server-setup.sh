#!/bin/bash

# FVChain Fresh Server Setup Script
# Untuk setup lengkap setelah reinstall server

set -e

echo "=== FVChain Fresh Server Setup ==="
echo "Starting fresh installation..."

# Update system
echo "[1/10] Updating system packages..."
apt update && apt upgrade -y

# Install essential packages
echo "[2/10] Installing essential packages..."
apt install -y curl wget git build-essential pkg-config libssl-dev nginx certbot python3-certbot-nginx

# Install Node.js 20
echo "[3/10] Installing Node.js 20..."
curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
apt install -y nodejs

# Install Rust
echo "[4/10] Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Create directories
echo "[5/10] Creating project directories..."
mkdir -p /opt/fvc
mkdir -p /opt/fvc-dashboard
mkdir -p /var/log/fvc

# Clone and setup FVChain
echo "[6/10] Setting up FVChain core..."
cd /opt/fvc
# Note: Replace with actual git clone when repository is ready
# git clone https://github.com/your-repo/fvchain.git .
echo "FVChain source code will be uploaded here"

# Setup dashboard
echo "[7/10] Setting up dashboard..."
cd /opt/fvc-dashboard
echo "Dashboard files will be uploaded here"

# Configure Nginx
echo "[8/10] Configuring Nginx..."
cat > /etc/nginx/sites-available/fvchain << 'EOF'
server {
    listen 80;
    server_name fvchain.xyz www.fvchain.xyz;
    
    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name fvchain.xyz www.fvchain.xyz;
    
    # SSL configuration will be added by certbot
    
    # Dashboard (Next.js)
    location / {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
    
    # RPC API
    location /api/ {
        proxy_pass http://localhost:8080/;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # CORS headers
        add_header Access-Control-Allow-Origin *;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
        add_header Access-Control-Allow-Headers "Content-Type, Authorization";
    }
}
EOF

# Enable site
ln -sf /etc/nginx/sites-available/fvchain /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default
nginx -t
systemctl restart nginx

# Setup SSL with Let's Encrypt
echo "[9/10] Setting up SSL certificate..."
certbot --nginx -d fvchain.xyz -d www.fvchain.xyz --non-interactive --agree-tos --email admin@fvchain.xyz

# Create systemd services
echo "[10/10] Creating systemd services..."

# FVChain RPC Service
cat > /etc/systemd/system/fvc-rpc.service << 'EOF'
[Unit]
Description=FVChain RPC Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/fvc
ExecStart=/root/.cargo/bin/cargo run --bin fvc-rpc
Restart=always
RestartSec=10
Environment=RUST_LOG=info
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# FVChain Dashboard Service
cat > /etc/systemd/system/fvc-dashboard.service << 'EOF'
[Unit]
Description=FVChain Dashboard
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/fvc-dashboard
ExecStart=/usr/bin/npm start
Restart=always
RestartSec=10
Environment=NODE_ENV=production
Environment=PORT=3001
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and enable services
systemctl daemon-reload
systemctl enable nginx
systemctl enable fvc-rpc
systemctl enable fvc-dashboard

echo "✅ Fresh server setup completed!"
echo ""
echo "Next steps:"
echo "1. Upload FVChain source code to /opt/fvc"
echo "2. Upload dashboard files to /opt/fvc-dashboard"
echo "3. Install dashboard dependencies: cd /opt/fvc-dashboard && npm install"
echo "4. Build dashboard: npm run build"
echo "5. Start services: systemctl start fvc-rpc fvc-dashboard"
echo "6. Check status: systemctl status fvc-rpc fvc-dashboard nginx"
echo ""
echo "🌐 Website will be available at: https://fvchain.xyz"
echo "📊 Mining dashboard: https://fvchain.xyz/mining"
echo "🔗 API endpoint: https://fvchain.xyz/api/"