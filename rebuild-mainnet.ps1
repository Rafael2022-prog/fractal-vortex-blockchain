# FVChain Mainnet Rebuild Script for Windows
# Updated with new difficulty configuration: Initial difficulty 10, 2023-block adjustment

param(
    [string]$InstallPath = "$env:USERPROFILE\FVChain",
    [switch]$SkipBuild = $false,
    [switch]$SkipBackup = $false
)

# Set execution policy for this script
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process -Force

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Cyan"

Write-Host "========================================" -ForegroundColor $Green
Write-Host "FVChain Mainnet Rebuild Script" -ForegroundColor $Green
Write-Host "========================================" -ForegroundColor $Green
Write-Host "New Configuration:" -ForegroundColor $Yellow
Write-Host "  - Max Supply: 3.6B FVC" -ForegroundColor $Blue
Write-Host "  - Mining Allocation: 65% (2.34B FVC)" -ForegroundColor $Blue
Write-Host "  - Owner Allocation: 15% (540M FVC)" -ForegroundColor $Blue
Write-Host "  - Developer Allocation: 10% (360M FVC)" -ForegroundColor $Blue
Write-Host "  - Ecosystem Maintenance: 10% (360M FVC)" -ForegroundColor $Blue
Write-Host "  - Block Time: 5 seconds" -ForegroundColor $Blue
Write-Host "  - Block Reward: 6.25 FVC" -ForegroundColor $Blue
Write-Host "  - Halving: Every 2 years" -ForegroundColor $Blue
Write-Host "  - Difficulty Adjustment: Bitcoin-style every 2023 blocks" -ForegroundColor $Blue
Write-Host "  - Initial Difficulty: 10" -ForegroundColor $Blue
Write-Host "========================================" -ForegroundColor $Green

# Stop existing processes
Write-Host "Stopping existing FVChain processes..." -ForegroundColor $Yellow
Get-Process -Name "fvc-mainnet" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "fvc-rpc" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

# Create installation directory
Write-Host "Creating installation directory..." -ForegroundColor $Yellow
if (-not (Test-Path $InstallPath)) {
    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
}

# Backup existing configuration
if (-not $SkipBackup -and (Test-Path "$InstallPath\mainnet-genesis.json")) {
    Write-Host "Backing up existing configuration..." -ForegroundColor $Yellow
    $backupDir = "$InstallPath\backup\$(Get-Date -Format 'yyyyMMdd_HHmmss')"
    New-Item -ItemType Directory -Path $backupDir -Force | Out-Null
    
    Copy-Item "$InstallPath\*.json" $backupDir -ErrorAction SilentlyContinue
    Copy-Item "$InstallPath\*.env" $backupDir -ErrorAction SilentlyContinue
    Copy-Item "$InstallPath\*.toml" $backupDir -ErrorAction SilentlyContinue
}

# Build binaries
if (-not $SkipBuild) {
    Write-Host "Building FVChain binaries..." -ForegroundColor $Yellow
    
    cargo build --release --bin fvc-node
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to build fvc-mainnet" -ForegroundColor $Red
        exit 1
    }
    
    cargo build --release --bin fvc-rpc
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to build fvc-rpc" -ForegroundColor $Red
        exit 1
    }
}

# Copy files to installation directory
Write-Host "Copying files to installation directory..." -ForegroundColor $Yellow
Copy-Item "target\release\fvc-node.exe" "$InstallPath\fvc-mainnet.exe" -Force
Copy-Item "target\release\fvc-rpc.exe" "$InstallPath\" -Force
Copy-Item "mainnet-genesis.json" "$InstallPath\" -Force
Copy-Item "mainnet.env" "$InstallPath\" -Force

# Create mining configuration
$miningConfig = @"
{
  "network": {
    "name": "FVChain Mainnet",
    "chainId": 369,
    "blockTime": 5
  },
  "mining": {
    "enabled": true,
    "algorithm": "FractalVortex",
    "blockReward": "6.25",
    "halvingInterval": "2 years",
    "difficulty": {
      "initial": 10,
      "adjustmentInterval": 2023,
      "targetBlockTime": 5,
      "algorithm": "bitcoin_adjustment"
    }
  },
  "allocation": {
    "totalSupply": "3.6B FVC",
    "mining": "65% (2.34B FVC)",
    "owner": "15% (540M FVC)",
    "developer": "10% (360M FVC)",
    "ecosystem": "10% (360M FVC)"
  }
}
"@

$miningConfig | Out-File "$InstallPath\mining-config.json" -Encoding UTF8

# Create startup scripts
$startMainnet = @"
@echo off
echo Starting FVChain Mainnet...
echo Configuration:
echo   - Initial Difficulty: 10
echo   - Adjustment Interval: 2023 blocks
echo   - Block Reward: 6.25 FVC
echo   - Block Time: 5 seconds

fvc-node.exe --config mainnet-genesis.json --env mainnet.env
pause
"@

$startRpc = @"
@echo off
echo Starting FVChain RPC Server...
echo RPC will be available at http://localhost:8080

fvc-rpc.exe --config mainnet-genesis.json --env mainnet.env
pause
"@

$startAll = @"
@echo off
echo Starting FVChain Mainnet and RPC Server...
echo ========================================
echo FVChain Mainnet Configuration:
echo   - Network: FVChain Mainnet (Chain ID: 369)
echo   - Initial Difficulty: 10
echo   - Adjustment: Every 2023 blocks
echo   - Block Reward: 6.25 FVC
echo   - Halving: Every 2 years
echo   - Mining Allocation: 65% of 3.6B FVC
echo ========================================

start "FVChain Mainnet" cmd /k "fvc-node.exe --config mainnet-genesis.json --env mainnet.env"
timeout /t 2 /nobreak >nul
start "FVChain RPC" cmd /k "fvc-rpc.exe --config mainnet-genesis.json --env mainnet.env"

echo Services started in new windows!
echo Press any key to exit...
pause >nul
"@

$startMainnet | Out-File "$InstallPath\start-mainnet.bat" -Encoding ASCII
$startRpc | Out-File "$InstallPath\start-rpc.bat" -Encoding ASCII
$startAll | Out-File "$InstallPath\start-all.bat" -Encoding ASCII

Write-Host "========================================" -ForegroundColor $Green
Write-Host "FVChain Mainnet Rebuild Complete!" -ForegroundColor $Green
Write-Host "========================================" -ForegroundColor $Green
Write-Host "Files deployed to: $InstallPath" -ForegroundColor $Blue
Write-Host "Configuration updated:" -ForegroundColor $Yellow
Write-Host "  - Initial Difficulty: 10" -ForegroundColor $Green
Write-Host "  - Adjustment Interval: 2023 blocks" -ForegroundColor $Green
Write-Host "  - Bitcoin-style difficulty adjustment" -ForegroundColor $Green
Write-Host "  - 6.25 FVC block reward with 2-year halving" -ForegroundColor $Green
Write-Host "  - 65% mining allocation (2.34B FVC)" -ForegroundColor $Green
Write-Host "========================================" -ForegroundColor $Green
Write-Host "To start the network:" -ForegroundColor $Yellow
Write-Host "  1. Run: $InstallPath\start-all.bat" -ForegroundColor $Blue
Write-Host "  2. Or run individual: start-mainnet.bat and start-rpc.bat" -ForegroundColor $Blue
Write-Host "========================================" -ForegroundColor $Green