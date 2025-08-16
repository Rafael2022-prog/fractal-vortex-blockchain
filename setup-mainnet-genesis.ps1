#!/usr/bin/env pwsh
# FVChain Mainnet Genesis Setup Script
# This script generates ecosystem wallets and prepares the mainnet genesis block

Write-Host "[FVCHAIN] Mainnet Genesis Setup" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan
Write-Host ""

# Function to validate supply calculations
function Test-SupplyCalculation {
    Write-Host "[VALIDATE] Supply Calculations..." -ForegroundColor Yellow
    
    $totalSupply = [decimal]"3600900000"
    $ownerAllocation = [decimal]"9000000"
    $developerAllocation = [decimal]"8000000"
    $gasPoolAllocation = [decimal]"3583900000"
    
    $calculatedTotal = $ownerAllocation + $developerAllocation + $gasPoolAllocation
    
    Write-Host "   Total Supply:        $($totalSupply.ToString('N0')) FVC" -ForegroundColor White
    $ownerPercent = ($ownerAllocation/$totalSupply*100).ToString('F2')
    $developerPercent = ($developerAllocation/$totalSupply*100).ToString('F2')
    $gasPoolPercent = ($gasPoolAllocation/$totalSupply*100).ToString('F2')
    
    Write-Host "   Owner Allocation:    $($ownerAllocation.ToString('N0')) FVC ($ownerPercent%)" -ForegroundColor Green
    Write-Host "   Developer Allocation: $($developerAllocation.ToString('N0')) FVC ($developerPercent%)" -ForegroundColor Green
    Write-Host "   Gas Pool Allocation: $($gasPoolAllocation.ToString('N0')) FVC ($gasPoolPercent%)" -ForegroundColor Green
    Write-Host "   Calculated Total:    $($calculatedTotal.ToString('N0')) FVC" -ForegroundColor White
    
    if ($calculatedTotal -eq $totalSupply) {
        Write-Host "   [OK] Supply calculation VERIFIED" -ForegroundColor Green
        return $true
    } else {
        Write-Host "   [ERROR] Supply calculation FAILED" -ForegroundColor Red
        Write-Host "   Expected: $($totalSupply.ToString('N0')), Got: $($calculatedTotal.ToString('N0'))" -ForegroundColor Red
        return $false
    }
}

# Function to check prerequisites
function Test-Prerequisites {
    Write-Host "[CHECK] Prerequisites..." -ForegroundColor Yellow
    
    $rustInstalled = Get-Command "cargo" -ErrorAction SilentlyContinue
    if (-not $rustInstalled) {
        Write-Host "   [ERROR] Rust/Cargo not found. Please install Rust first." -ForegroundColor Red
        return $false
    }
    Write-Host "   [OK] Rust/Cargo found" -ForegroundColor Green
    
    if (-not (Test-Path "Cargo.toml")) {
        Write-Host "   [ERROR] Cargo.toml not found. Run from project root." -ForegroundColor Red
        return $false
    }
    Write-Host "   [OK] Cargo.toml found" -ForegroundColor Green
    
    if (-not (Test-Path "mainnet-genesis-final.json")) {
        Write-Host "   [ERROR] mainnet-genesis-final.json not found" -ForegroundColor Red
        return $false
    }
    Write-Host "   [OK] Genesis template found" -ForegroundColor Green
    
    return $true
}

# Function to add wallet generator dependencies
function Add-WalletDependencies {
    Write-Host "[DEPS] Adding Required Dependencies..." -ForegroundColor Yellow
    
    $dependencies = @(
        'secp256k1 = "0.28"',
        'sha3 = "0.10"',
        'hex = "0.4"',
        'rand = "0.8"',
        'chrono = { version = "0.4", features = ["serde"] }'
    )
    
    $cargoContent = Get-Content "Cargo.toml" -Raw
    
    foreach ($dep in $dependencies) {
        $depName = $dep.Split(' ')[0]
        if ($cargoContent -notmatch [regex]::Escape($depName)) {
            Write-Host "   Adding dependency: $depName" -ForegroundColor Cyan
            $cargoContent += "`n$dep"
        } else {
            Write-Host "   Dependency already exists: $depName" -ForegroundColor Gray
        }
    }
    
    Set-Content "Cargo.toml" $cargoContent
    Write-Host "   [OK] Dependencies updated" -ForegroundColor Green
}

# Function to generate ecosystem wallets
function New-EcosystemWallets {
    Write-Host "[GENERATE] Ecosystem Wallets..." -ForegroundColor Yellow
    
    try {
        Write-Host "   Compiling wallet generator..." -ForegroundColor Cyan
        $result = cargo run --bin generate-ecosystem-wallets 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "   [OK] Ecosystem wallets generated successfully" -ForegroundColor Green
            return $true
        } else {
            Write-Host "   [ERROR] Failed to generate wallets" -ForegroundColor Red
            Write-Host "   Error: $result" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "   [ERROR] Exception during wallet generation: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to validate generated files
function Test-GeneratedFiles {
    Write-Host "[VALIDATE] Generated Files..." -ForegroundColor Yellow
    
    $requiredFiles = @(
        "mainnet-genesis-with-keys.json",
        "ecosystem-wallets-with-keys.json",
        "wallet-security-checklist.md"
    )
    
    $allValid = $true
    foreach ($file in $requiredFiles) {
        if (Test-Path $file) {
            $size = (Get-Item $file).Length
            Write-Host "   [OK] $file ($size bytes)" -ForegroundColor Green
        } else {
            Write-Host "   [ERROR] $file (missing)" -ForegroundColor Red
            $allValid = $false
        }
    }
    
    return $allValid
}

# Function to create backup
function New-Backup {
    Write-Host "[BACKUP] Creating Backup..." -ForegroundColor Yellow
    
    $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $backupDir = "mainnet-backup-$timestamp"
    
    New-Item -ItemType Directory -Path $backupDir -Force | Out-Null
    
    $filesToBackup = @(
        "mainnet-genesis-final.json",
        "mainnet-genesis-with-keys.json",
        "ecosystem-wallets.json",
        "ecosystem-wallets-with-keys.json",
        "wallet-security-checklist.md"
    )
    
    foreach ($file in $filesToBackup) {
        if (Test-Path $file) {
            Copy-Item $file $backupDir
            Write-Host "   Backed up: $file" -ForegroundColor Cyan
        }
    }
    
    Write-Host "   [OK] Backup created in: $backupDir" -ForegroundColor Green
}

# Function to display security warnings
function Show-SecurityWarnings {
    Write-Host ""
    Write-Host "[WARNING] CRITICAL SECURITY WARNINGS" -ForegroundColor Red -BackgroundColor Yellow
    Write-Host "================================" -ForegroundColor Red
    Write-Host ""
    Write-Host "[CRITICAL] PRIVATE KEYS GENERATED - IMMEDIATE ACTION REQUIRED:" -ForegroundColor Red
    Write-Host "   1. Transfer private keys to hardware security modules (HSM)" -ForegroundColor Yellow
    Write-Host "   2. Create encrypted backups in multiple secure locations" -ForegroundColor Yellow
    Write-Host "   3. Implement multi-signature schemes for critical wallets" -ForegroundColor Yellow
    Write-Host "   4. Never store private keys on networked systems" -ForegroundColor Yellow
    Write-Host "   5. Follow the security checklist in wallet-security-checklist.md" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "[NEXT STEPS]:" -ForegroundColor Cyan
    Write-Host "   1. Review generated files carefully" -ForegroundColor White
    Write-Host "   2. Implement security measures from checklist" -ForegroundColor White
    Write-Host "   3. Test wallet operations on testnet first" -ForegroundColor White
    Write-Host "   4. Conduct security audit before mainnet launch" -ForegroundColor White
    Write-Host ""
}

# Main execution
try {
    # Step 1: Validate supply calculations
    if (-not (Test-SupplyCalculation)) {
        throw "Supply calculation validation failed"
    }
    Write-Host ""
    
    # Step 2: Check prerequisites
    if (-not (Test-Prerequisites)) {
        throw "Prerequisites check failed"
    }
    Write-Host ""
    
    # Step 3: Add dependencies
    Add-WalletDependencies
    Write-Host ""
    
    # Step 4: Generate wallets
    if (-not (New-EcosystemWallets)) {
        throw "Wallet generation failed"
    }
    Write-Host ""
    
    # Step 5: Validate generated files
    if (-not (Test-GeneratedFiles)) {
        throw "Generated files validation failed"
    }
    Write-Host ""
    
    # Step 6: Create backup
    New-Backup
    Write-Host ""
    
    # Step 7: Show security warnings
    Show-SecurityWarnings
    
    Write-Host "[SUCCESS] MAINNET GENESIS SETUP COMPLETED SUCCESSFULLY!" -ForegroundColor Green -BackgroundColor Black
    
} catch {
    Write-Host ""
    Write-Host "[FAILED] SETUP FAILED: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Please review the errors above and try again." -ForegroundColor Yellow
    exit 1
}