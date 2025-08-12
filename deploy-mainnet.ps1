# Fractal Vortex Chain Mainnet Deployment Script (PowerShell)
# Deploy FVC to production server via SSH

param(
    [string]$ServerIP = "103.245.38.44",
    [string]$Username = "root",
    [string]$Password = "a6?#PMWdik52"
)

$ErrorActionPreference = "Stop"

Write-Host "🚀 Starting FVC Mainnet Deployment..." -ForegroundColor Green

# Configuration
$RemoteDir = "/opt/fvc"
$BinaryName = "fvc-mainnet"
$ServiceName = "fvc-mainnet"

function Write-Info($message) {
    Write-Host "[INFO] $message" -ForegroundColor Blue
}

function Write-Success($message) {
    Write-Host "[SUCCESS] $message" -ForegroundColor Green
}

function Write-Warning($message) {
    Write-Host "[WARNING] $message" -ForegroundColor Yellow
}

function Write-Error($message) {
    Write-Host "[ERROR] $message" -ForegroundColor Red
}

# Check if binary exists
if (-not (Test-Path "target\release\fvc-rpc.exe")) {
    Write-Error "Binary not found. Please build first with: cargo build --release"
    exit 1
}

Write-Info "Preparing deployment files..."

# Create deployment directory
if (Test-Path "deploy") {
    Remove-Item "deploy" -Recurse -Force
}
New-Item -ItemType Directory -Path "deploy" | Out-Null

# Copy necessary files
Copy-Item "target\release\fvc-rpc.exe" "deploy\$BinaryName"
Copy-Item "mainnet-genesis.json" "deploy\"
Copy-Item "mainnet.env" "deploy\"
Copy-Item "security-config.toml" "deploy\"

# Create systemd service file
$serviceContent = @"
[Unit]
Description=Fractal Vortex Chain Mainnet Node
After=network.target
Wants=network.target

[Service]
Type=simple
User=fvc
Group=fvc
WorkingDirectory=$RemoteDir
EnvironmentFile=$RemoteDir/mainnet.env
ExecStart=$RemoteDir/$BinaryName
Restart=always
RestartSec=10
KillMode=process
TimeoutSec=300

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$RemoteDir

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=fvc-mainnet

[Install]
WantedBy=multi-user.target
"@

$serviceContent | Out-File -FilePath "deploy\fvc-mainnet.service" -Encoding UTF8

# Create installation script
$installScript = @'
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
'@

$installScript | Out-File -FilePath "deploy\install.sh" -Encoding UTF8

Write-Info "Using SCP to upload files to server..."

# Use SCP via PowerShell (requires OpenSSH or similar)
try {
    # Create remote directory first
    $sshCommand = "ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null ${Username}@${ServerIP} 'mkdir -p /tmp/fvc-deploy'"
    cmd /c $sshCommand
    
    # Upload files
    $scpCommand = "scp -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -r deploy/* ${Username}@${ServerIP}:/tmp/fvc-deploy/"
    cmd /c $scpCommand
    
    Write-Success "Files uploaded successfully!"
} catch {
    Write-Error "Failed to upload files: $($_.Exception.Message)"
    Write-Warning "Please ensure OpenSSH is installed and available in PATH"
    Write-Warning "Alternative: Use WinSCP or similar tool to upload 'deploy' folder contents to /tmp/fvc-deploy/"
    exit 1
}

Write-Info "Installing FVC on server..."

# Execute installation on server
$remoteCommands = @"
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
"@

try {
    $sshInstallCommand = "ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null ${Username}@${ServerIP} '$remoteCommands'"
    cmd /c $sshInstallCommand
    
    Write-Success "🎉 FVC Mainnet deployment completed successfully!"
} catch {
    Write-Error "Failed to install on server: $($_.Exception.Message)"
    exit 1
}

Write-Info "Server Details:"
Write-Host "  - IP: $ServerIP" -ForegroundColor Cyan
Write-Host "  - RPC Port: 8332" -ForegroundColor Cyan
Write-Host "  - P2P Port: 8333" -ForegroundColor Cyan
Write-Host "  - Service: $ServiceName" -ForegroundColor Cyan
Write-Host "  - Data Directory: $RemoteDir" -ForegroundColor Cyan
Write-Host ""
Write-Info "Useful Commands:"
Write-Host "  - Check status: ssh ${Username}@${ServerIP} 'systemctl status ${ServiceName}'" -ForegroundColor Yellow
Write-Host "  - View logs: ssh ${Username}@${ServerIP} 'journalctl -u ${ServiceName} -f'" -ForegroundColor Yellow
Write-Host "  - Restart service: ssh ${Username}@${ServerIP} 'systemctl restart ${ServiceName}'" -ForegroundColor Yellow
Write-Host ""
Write-Warning "Please save these credentials securely and change default passwords!"

# Cleanup
Remove-Item "deploy" -Recurse -Force

Write-Success "Deployment script completed! 🚀"
Write-Info "Mainnet is now running at: http://$ServerIP:8332"