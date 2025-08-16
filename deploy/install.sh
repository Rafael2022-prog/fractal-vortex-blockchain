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

echo "âœ… FVC Mainnet installation completed!"
echo "To start the service: systemctl start fvc-mainnet"
echo "To check status: systemctl status fvc-mainnet"
echo "To view logs: journalctl -u fvc-mainnet -f"
