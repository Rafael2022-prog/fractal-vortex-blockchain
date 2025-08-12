# Simple FVC Mainnet Deployment Script
$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "🚀 Starting Simple FVC Mainnet Deployment..." -ForegroundColor Green

# Create deployment directory
Write-Host "[INFO] Creating deployment directory..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "mkdir -p /opt/fvc"

# Upload binary
Write-Host "[INFO] Uploading FVC binary..." -ForegroundColor Cyan
echo y | pscp -pw $Password target\release\fvc-rpc.exe $Username@${ServerIP}:/opt/fvc/fvc-rpc

# Upload genesis file
Write-Host "[INFO] Uploading genesis configuration..." -ForegroundColor Cyan
echo y | pscp -pw $Password mainnet-genesis.json $Username@${ServerIP}:/opt/fvc/genesis.json

# Upload environment file
Write-Host "[INFO] Uploading environment configuration..." -ForegroundColor Cyan
echo y | pscp -pw $Password mainnet.env $Username@${ServerIP}:/opt/fvc/.env

# Create systemd service
Write-Host "[INFO] Creating systemd service..." -ForegroundColor Cyan
$serviceContent = @'
[Unit]
Description=Fractal Vortex Chain Mainnet
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/fvc
EnvironmentFile=/opt/fvc/.env
ExecStart=/opt/fvc/fvc-rpc --config /opt/fvc/genesis.json
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
'@

echo y | plink -ssh -pw $Password $Username@$ServerIP "cat > /etc/systemd/system/fvc-mainnet.service << 'EOF'
$serviceContent
EOF"

# Set permissions and start service
Write-Host "[INFO] Setting permissions and starting service..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "chmod +x /opt/fvc/fvc-rpc"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl daemon-reload"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl enable fvc-mainnet"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl start fvc-mainnet"

# Check status
Write-Host "[INFO] Checking service status..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl status fvc-mainnet --no-pager"

Write-Host "✅ FVC Mainnet deployment completed!" -ForegroundColor Green
Write-Host "Server: $ServerIP" -ForegroundColor Yellow
Write-Host "RPC Port: 8332" -ForegroundColor Yellow
Write-Host "P2P Port: 8333" -ForegroundColor Yellow