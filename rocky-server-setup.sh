#!/bin/bash

# FVChain Fresh Server Setup for Rocky Linux
echo "=== FVChain Fresh Server Setup (Rocky Linux) ==="
echo "Starting fresh installation..."

# [1/10] Update system packages
echo "[1/10] Updating system packages..."
dnf update -y

# [2/10] Install essential packages
echo "[2/10] Installing essential packages..."
dnf install -y curl wget git unzip nginx certbot python3-certbot-nginx firewalld

# [3/10] Install Node.js 20
echo "[3/10] Installing Node.js 20..."
curl -fsSL https://rpm.nodesource.com/setup_20.x | bash -
dnf install -y nodejs

# [4/10] Install Rust
echo "[4/10] Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# [5/10] Create directories
echo "[5/10] Creating directories..."
mkdir -p /opt/fvchain
mkdir -p /opt/fvc-dashboard
mkdir -p /var/log/fvchain
chown -R root:root /opt/fvchain /opt/fvc-dashboard /var/log/fvchain

# [6/10] Configure Nginx
echo "[6/10] Configuring Nginx..."
cat > /etc/nginx/conf.d/fvchain.conf << 'EOF'
server {
    listen 80;
    server_name fvchain.xyz www.fvchain.xyz;
    
    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name fvchain.xyz www.fvchain.xyz;
    
    # SSL certificates (will be configured by certbot)
    ssl_certificate /etc/letsencrypt/live/fvchain.xyz/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fvchain.xyz/privkey.pem;
    
    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    
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

# [7/10] Configure firewall
echo "[7/10] Configuring firewall..."
systemctl enable firewalld
systemctl start firewalld
firewall-cmd --permanent --add-service=http
firewall-cmd --permanent --add-service=https
firewall-cmd --permanent --add-service=ssh
firewall-cmd --permanent --add-port=3001/tcp
firewall-cmd --permanent --add-port=8080/tcp
firewall-cmd --reload

# [8/10] Create systemd services
echo "[8/10] Creating systemd services..."

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

[Install]
WantedBy=multi-user.target
EOF

# FVChain RPC Service
cat > /etc/systemd/system/fvc-rpc.service << 'EOF'
[Unit]
Description=FVChain RPC Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/fvchain
ExecStart=/opt/fvchain/target/release/rpc-server
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# [9/10] Enable services
echo "[9/10] Enabling services..."
systemctl daemon-reload
systemctl enable nginx
systemctl enable fvc-dashboard
systemctl enable fvc-rpc

# [10/10] Setup SSL (Let's Encrypt)
echo "[10/10] Setting up SSL..."
systemctl start nginx

# Note: SSL setup will be done after domain is properly configured
echo "SSL setup will be done after domain configuration"

echo "=== Setup completed! ==="
echo "Next steps:"
echo "1. Upload FVChain source code to /opt/fvchain"
echo "2. Upload dashboard files to /opt/fvc-dashboard"
echo "3. Build and start services"
echo "4. Configure SSL with: certbot --nginx -d fvchain.xyz -d www.fvchain.xyz"