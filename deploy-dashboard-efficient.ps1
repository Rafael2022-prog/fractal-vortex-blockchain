# Deploy FVC Dashboard to Cloud Server (Efficient Version)
$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "Deploying FVC Dashboard to Cloud Server (Efficient)..." -ForegroundColor Green

# Install Node.js and dependencies
Write-Host "[INFO] Installing Node.js and dependencies..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "curl -fsSL https://rpm.nodesource.com/setup_20.x | bash -"
echo y | plink -ssh -pw $Password $Username@$ServerIP "yum install -y nodejs nginx"

# Create dashboard directory
Write-Host "[INFO] Creating dashboard directory..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "mkdir -p /opt/fvc-dashboard"
echo y | plink -ssh -pw $Password $Username@$ServerIP "rm -rf /opt/fvc-dashboard/*"

# Upload only source files (exclude node_modules and .next)
Write-Host "[INFO] Uploading dashboard source files..." -ForegroundColor Cyan
echo y | pscp -pw $Password dashboard/package.json $Username@${ServerIP}:/opt/fvc-dashboard/
echo y | pscp -pw $Password dashboard/package-lock.json $Username@${ServerIP}:/opt/fvc-dashboard/
echo y | pscp -pw $Password dashboard/next.config.mjs $Username@${ServerIP}:/opt/fvc-dashboard/
echo y | pscp -pw $Password dashboard/tailwind.config.ts $Username@${ServerIP}:/opt/fvc-dashboard/
echo y | pscp -pw $Password dashboard/tsconfig.json $Username@${ServerIP}:/opt/fvc-dashboard/
echo y | pscp -pw $Password dashboard/postcss.config.mjs $Username@${ServerIP}:/opt/fvc-dashboard/
echo y | pscp -pw $Password dashboard/next-env.d.ts $Username@${ServerIP}:/opt/fvc-dashboard/
echo y | pscp -pw $Password -r dashboard/src $Username@${ServerIP}:/opt/fvc-dashboard/
echo y | pscp -pw $Password -r dashboard/public $Username@${ServerIP}:/opt/fvc-dashboard/

# Install dependencies on server
Write-Host "[INFO] Installing dependencies on server..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "cd /opt/fvc-dashboard && npm install"

# Build dashboard on server
Write-Host "[INFO] Building dashboard on server..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "cd /opt/fvc-dashboard && npm run build"

# Create nginx configuration
Write-Host "[INFO] Configuring nginx..." -ForegroundColor Cyan
$nginxConfig = @'
server {
    listen 80;
    server_name _;
    
    # Dashboard (Next.js)
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
    
    # FVC RPC API
    location /api/rpc {
        proxy_pass http://localhost:8332;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # Mining API
    location /api/mining {
        proxy_pass http://localhost:8333;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
'@

echo y | plink -ssh -pw $Password $Username@$ServerIP "cat > /etc/nginx/conf.d/fvc-dashboard.conf << 'EOF'
$nginxConfig
EOF"

# Create systemd service for dashboard
Write-Host "[INFO] Creating dashboard service..." -ForegroundColor Cyan
$dashboardService = @'
[Unit]
Description=FVC Dashboard
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/fvc-dashboard
Environment=NODE_ENV=production
Environment=PORT=3000
Environment=NEXT_PUBLIC_RPC_URL=http://localhost:8332
Environment=NEXT_PUBLIC_MINING_URL=http://localhost:8333
ExecStart=/usr/bin/npm start
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
'@

echo y | plink -ssh -pw $Password $Username@$ServerIP "cat > /etc/systemd/system/fvc-dashboard.service << 'EOF'
$dashboardService
EOF"

# Start services
Write-Host "[INFO] Starting services..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl daemon-reload"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl enable nginx fvc-dashboard"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl start nginx fvc-dashboard"

# Check status
Write-Host "[INFO] Checking services status..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl status nginx --no-pager"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl status fvc-dashboard --no-pager"

Write-Host "FVC Dashboard deployment completed!" -ForegroundColor Green
Write-Host "Dashboard URL: http://$ServerIP" -ForegroundColor Yellow
Write-Host "Mining Interface: http://$ServerIP/mining" -ForegroundColor Yellow
Write-Host "RPC API: http://$ServerIP/api/rpc" -ForegroundColor Yellow