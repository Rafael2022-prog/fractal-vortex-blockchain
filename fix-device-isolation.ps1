# Fix Device Isolation on Cloud Server
param(
    [string]$ServerIP = "103.245.38.44",
    [string]$Username = "root"
)

Write-Host "=== FIXING DEVICE ISOLATION ON CLOUD SERVER ===" -ForegroundColor Green

# Step 1: Set executable permissions
Write-Host "Setting executable permissions..." -ForegroundColor Yellow
try {
    & ssh ${Username}@${ServerIP} "chmod +x /usr/local/bin/fvc-rpc"
    Write-Host "✅ Permissions set" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to set permissions: $_" -ForegroundColor Red
}

# Step 2: Restart RPC service
Write-Host "Restarting RPC service..." -ForegroundColor Yellow
try {
    & ssh ${Username}@${ServerIP} "systemctl restart fvc-rpc"
    Write-Host "✅ RPC service restarted" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to restart RPC: $_" -ForegroundColor Red
}

# Step 3: Check service status
Write-Host "Checking service status..." -ForegroundColor Yellow
try {
    & ssh ${Username}@${ServerIP} "systemctl is-active fvc-rpc"
    Write-Host "✅ Service status checked" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to check status: $_" -ForegroundColor Red
}

# Step 4: Restart Nginx
Write-Host "Restarting Nginx..." -ForegroundColor Yellow
try {
    & ssh ${Username}@${ServerIP} "systemctl restart nginx"
    Write-Host "✅ Nginx restarted" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to restart Nginx: $_" -ForegroundColor Red
}

Write-Host "\n✅ Device isolation fix deployment completed!" -ForegroundColor Green
Write-Host "Device isolation should now work properly on the cloud server." -ForegroundColor Cyan
Write-Host "Each device will have isolated wallets and mining based on device_id." -ForegroundColor White