# Build FVC on Server Script
$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "🚀 Building FVC on Server..." -ForegroundColor Green

# Install Rust and dependencies
Write-Host "[INFO] Installing Rust and dependencies..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
echo y | plink -ssh -pw $Password $Username@$ServerIP "source ~/.cargo/env"
echo y | plink -ssh -pw $Password $Username@$ServerIP "yum update -y"
echo y | plink -ssh -pw $Password $Username@$ServerIP "yum install -y git gcc openssl-devel pkg-config"

# Create source directory
Write-Host "[INFO] Creating source directory..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "mkdir -p /opt/fvc-source"

# Upload source files
Write-Host "[INFO] Uploading source files..." -ForegroundColor Cyan
echo y | pscp -pw $Password -r src $Username@${ServerIP}:/opt/fvc-source/
echo y | pscp -pw $Password Cargo.toml $Username@${ServerIP}:/opt/fvc-source/
echo y | pscp -pw $Password Cargo.lock $Username@${ServerIP}:/opt/fvc-source/
echo y | pscp -pw $Password mainnet-genesis.json $Username@${ServerIP}:/opt/fvc/genesis.json
echo y | pscp -pw $Password mainnet.env $Username@${ServerIP}:/opt/fvc/.env

# Build on server
Write-Host "[INFO] Building FVC binary on server..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "cd /opt/fvc-source && source ~/.cargo/env && cargo build --release --bin fvc-rpc"

# Copy binary to deployment location
Write-Host "[INFO] Installing binary..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "cp /opt/fvc-source/target/release/fvc-rpc /opt/fvc/"
echo y | plink -ssh -pw $Password $Username@$ServerIP "chmod +x /opt/fvc/fvc-rpc"

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

# Start service
Write-Host "[INFO] Starting FVC service..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl daemon-reload"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl enable fvc-mainnet"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl restart fvc-mainnet"

# Check status
Write-Host "[INFO] Checking service status..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl status fvc-mainnet --no-pager"

Write-Host "✅ FVC Mainnet build and deployment completed!" -ForegroundColor Green
Write-Host "Server: $ServerIP" -ForegroundColor Yellow
Write-Host "RPC Port: 8332" -ForegroundColor Yellow
Write-Host "P2P Port: 8333" -ForegroundColor Yellow