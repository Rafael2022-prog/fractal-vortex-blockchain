#!/bin/bash

# Fractal Vortex Chain Mainnet Deployment Script
# Deploy FVC to production server

set -e

echo "🚀 Starting FVC Mainnet Deployment..."

# Configuration
SERVER_IP="103.245.38.44"
SERVER_USER="root"
REMOTE_DIR="/opt/fvc"
BINARY_NAME="fvc-mainnet"
SERVICE_NAME="fvc-mainnet"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

echo_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

echo_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if binary exists
if [ ! -f "target/release/fvc-rpc" ]; then
    echo_error "Binary not found. Please build first with: cargo build --release"
    exit 1
fi

echo_info "Preparing deployment files..."

# Create deployment directory
mkdir -p deploy

# Copy necessary files
cp target/release/fvc-rpc deploy/$BINARY_NAME
cp mainnet-genesis.json deploy/
cp mainnet.env deploy/
cp security-config.toml deploy/

# Create systemd service file
cat > deploy/fvc-mainnet.service << EOF
[Unit]
Description=Fractal Vortex Chain Mainnet Node
After=network.target
Wants=network.target

[Service]
Type=simple
User=fvc
Group=fvc
WorkingDirectory=$REMOTE_DIR
EnvironmentFile=$REMOTE_DIR/mainnet.env
ExecStart=$REMOTE_DIR/$BINARY_NAME
Restart=always
RestartSec=10
KillMode=process
TimeoutSec=300

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$REMOTE_DIR

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=fvc-mainnet

[Install]
WantedBy=multi-user.target
EOF

# Create installation script
cat > deploy/install.sh << 'EOF'
#!/bin/bash
set -e

echo "Installing FVC Mainnet..."

# Create user and directories
useradd -r -s /bin/false fvc || true
mkdir -p /opt/fvc/{data,logs,backup}
chown -R fvc:fvc /opt/fvc

# Install binary and configs
cp fvc-mainnet /opt/fvc/
cp mainnet-genesis.json /opt/fvc/
cp mainnet.env /opt/fvc/
cp security-config.toml /opt/fvc/
chmod +x /opt/fvc/fvc-mainnet

# Install systemd service
cp fvc-mainnet.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable fvc-mainnet

# Install dependencies
apt-get update
apt-get install -y curl wget htop iotop ufw fail2ban

# Configure firewall
ufw --force enable
ufw allow 22/tcp
ufw allow 8332/tcp
ufw allow 8333/tcp

# Configure fail2ban
cat > /etc/fail2ban/jail.local << 'FAIL2BAN_EOF'
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 5

[sshd]
enabled = true
port = ssh
logpath = /var/log/auth.log
maxretry = 3
FAIL2BAN_EOF

systemctl restart fail2ban

echo "✅ FVC Mainnet installation completed!"
echo "To start the service: systemctl start fvc-mainnet"
echo "To check status: systemctl status fvc-mainnet"
echo "To view logs: journalctl -u fvc-mainnet -f"
EOF

chmod +x deploy/install.sh

echo_info "Uploading files to server..."

# Upload files using scp
scp -r deploy/* $SERVER_USER@$SERVER_IP:/tmp/fvc-deploy/

echo_info "Installing FVC on server..."

# Execute installation on server
ssh $SERVER_USER@$SERVER_IP << 'REMOTE_SCRIPT'
cd /tmp/fvc-deploy
chmod +x install.sh
./install.sh

# Start the service
systemctl start fvc-mainnet
sleep 5

# Check status
echo "🔍 Checking service status..."
systemctl status fvc-mainnet --no-pager

echo "📊 Checking logs..."
journalctl -u fvc-mainnet --no-pager -n 20

echo "🌐 Checking network connectivity..."
ss -tlnp | grep :833

echo "💾 Checking disk usage..."
df -h /opt/fvc

echo "🔧 Checking process..."
ps aux | grep fvc-mainnet | grep -v grep

REMOTE_SCRIPT

echo_success "🎉 FVC Mainnet deployment completed successfully!"
echo_info "Server Details:"
echo "  - IP: $SERVER_IP"
echo "  - RPC Port: 8332"
echo "  - P2P Port: 8333"
echo "  - Service: $SERVICE_NAME"
echo "  - Data Directory: $REMOTE_DIR"
echo ""
echo_info "Useful Commands:"
echo "  - Check status: ssh $SERVER_USER@$SERVER_IP 'systemctl status $SERVICE_NAME'"
echo "  - View logs: ssh $SERVER_USER@$SERVER_IP 'journalctl -u $SERVICE_NAME -f'"
echo "  - Restart service: ssh $SERVER_USER@$SERVER_IP 'systemctl restart $SERVICE_NAME'"
echo ""
echo_warning "Please save these credentials securely and change default passwords!"

# Cleanup
rm -rf deploy

echo_success "Deployment script completed! 🚀"