#!/usr/bin/env pwsh
# Upload FVChain Native Address Implementation to Server Cloud
# This script uploads the compiled binary and restarts the RPC server

param(
    [Parameter(Mandatory=$true)]
    [string]$ServerIP,
    
    [Parameter(Mandatory=$true)]
    [string]$Username,
    
    [Parameter(Mandatory=$false)]
    [string]$ServerPath = "/home/fvchain"
)

Write-Host "FVChain Server Cloud Deployment" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan
Write-Host ""

# Validate binary exists
if (!(Test-Path "./target/release/fvc-rpc.exe")) {
    Write-Host "Error: Binary not found. Run 'cargo build --release' first." -ForegroundColor Red
    exit 1
}

Write-Host "Server: $ServerIP" -ForegroundColor Yellow
Write-Host "User: $Username" -ForegroundColor Yellow
Write-Host "Path: $ServerPath" -ForegroundColor Yellow
Write-Host ""

# Step 1: Upload binary
Write-Host "1. Uploading binary to server..." -ForegroundColor Green
$uploadCommand = "scp ./target/release/fvc-rpc.exe ${Username}@${ServerIP}:${ServerPath}/"
Write-Host "Command: $uploadCommand" -ForegroundColor Gray
Write-Host "Please run this command manually:" -ForegroundColor Yellow
Write-Host $uploadCommand -ForegroundColor White
Write-Host ""

# Step 2: Stop existing server
Write-Host "2. Stop existing RPC server:" -ForegroundColor Green
$stopCommand = "ssh ${Username}@${ServerIP} 'pkill -f fvc-rpc'"
Write-Host "Command: $stopCommand" -ForegroundColor Gray
Write-Host "Please run this command manually:" -ForegroundColor Yellow
Write-Host $stopCommand -ForegroundColor White
Write-Host ""

# Step 3: Start new server
Write-Host "3. Start updated RPC server:" -ForegroundColor Green
$startCommand = "ssh ${Username}@${ServerIP} 'cd ${ServerPath} && nohup ./fvc-rpc.exe > rpc.log 2>&1 &'"
Write-Host "Command: $startCommand" -ForegroundColor Gray
Write-Host "Please run this command manually:" -ForegroundColor Yellow
Write-Host $startCommand -ForegroundColor White
Write-Host ""

# Step 4: Test deployment
Write-Host "4. Test deployment:" -ForegroundColor Green
$testCommand = "curl http://${ServerIP}:8080/wallet/create"
Write-Host "Command: $testCommand" -ForegroundColor Gray
Write-Host "Please run this command manually:" -ForegroundColor Yellow
Write-Host $testCommand -ForegroundColor White
Write-Host ""

# Additional verification commands
Write-Host "5. Additional verification:" -ForegroundColor Green
$statusCommand = "ssh ${Username}@${ServerIP} 'ps aux | grep fvc-rpc'"
Write-Host "Check server status: $statusCommand" -ForegroundColor Gray
$logCommand = "ssh ${Username}@${ServerIP} 'tail -f ${ServerPath}/rpc.log'"
Write-Host "Monitor logs: $logCommand" -ForegroundColor Gray
Write-Host ""

Write-Host "Expected Response Format:" -ForegroundColor Cyan
Write-Host '{' -ForegroundColor White
Write-Host '  "address": "fvc...",  // 43 characters total' -ForegroundColor White
Write-Host '  "address_format": "FVChain Native (160-bit)",' -ForegroundColor White
Write-Host '  "address_valid": true,' -ForegroundColor White
Write-Host '  "cryptography": "secp256k1"' -ForegroundColor White
Write-Host '}' -ForegroundColor White
Write-Host ""

Write-Host "Deployment commands generated successfully!" -ForegroundColor Green
Write-Host "Please execute the commands manually on your system." -ForegroundColor Yellow