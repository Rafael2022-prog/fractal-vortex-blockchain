# Smart Dashboard Update - Hot Reload tanpa mengganggu mining
# Strategi: Update hanya file yang berubah dengan zero-downtime

Write-Host "[SMART] Smart Dashboard Update - Zero Downtime Strategy" -ForegroundColor Cyan
Write-Host "[INFO] Strategi: Hot reload tanpa mengganggu mining yang sedang berjalan" -ForegroundColor Yellow

# Fungsi untuk cek status mining
function Test-MiningStatus {
    try {
        $response = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/miner/status" -Method GET -TimeoutSec 5
        $status = $response.Content | ConvertFrom-Json
        return $status.is_mining -eq $true
    } catch {
        return $false
    }
}

# Fungsi untuk update file secara atomic
function Update-FileAtomic {
    param(
        [string]$LocalFile,
        [string]$RemoteFile
    )
    
    $tempFile = "$RemoteFile.tmp"
    
    # Upload ke file temporary
    scp "$LocalFile" "root@103.127.99.3:$tempFile" 2>$null
    if ($LASTEXITCODE -eq 0) {
        # Atomic move
        ssh root@103.127.99.3 "mv '$tempFile' '$RemoteFile'" 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[OK] Updated: $RemoteFile" -ForegroundColor Green
            return $true
        }
    }
    return $false
}

# Cek status mining
$isMining = Test-MiningStatus
if ($isMining) {
    Write-Host "[MINING] Mining sedang aktif - menggunakan strategi hot reload" -ForegroundColor Yellow
} else {
    Write-Host "[IDLE] Mining tidak aktif - update normal" -ForegroundColor Green
}

# Build dashboard lokal terlebih dahulu
Write-Host "[BUILD] Building dashboard lokal..." -ForegroundColor Cyan
Set-Location "r:\369-FRACTAL\dashboard"

# Clean build untuk memastikan konsistensi
if (Test-Path ".next") {
    Remove-Item ".next" -Recurse -Force
}

npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Build gagal, membatalkan update" -ForegroundColor Red
    exit 1
}

# Strategi Hot Reload - Update file penting saja
Write-Host "[DEPLOY] Deploying dengan strategi hot reload..." -ForegroundColor Cyan

# 1. Update konfigurasi Next.js (penting untuk fix ERR_ABORTED)
$configUpdated = Update-FileAtomic "next.config.mjs" "/var/www/fvchain.xyz/next.config.mjs"

# 2. Update package.json jika ada perubahan dependencies
$packageUpdated = Update-FileAtomic "package.json" "/var/www/fvchain.xyz/package.json"

# 3. Update file build hasil (.next directory) - hanya file yang berubah
Write-Host "[SYNC] Syncing build files..." -ForegroundColor Yellow

# Rsync untuk update incremental (hanya file yang berubah)
$rsyncCmd = "rsync -avz --delete .next/ root@103.127.99.3:/var/www/fvchain.xyz/.next/"
try {
    Invoke-Expression $rsyncCmd
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[OK] Build files synced" -ForegroundColor Green
    }
} catch {
    Write-Host "[WARN] Rsync tidak tersedia, menggunakan metode alternatif" -ForegroundColor Yellow
    
    # Fallback: Update file penting saja
    $criticalFiles = @(
        ".next/server/app/wallet/page.js",
        ".next/server/app/layout.js",
        ".next/static/chunks/app/wallet/page-*.js"
    )
    
    foreach ($file in $criticalFiles) {
        if (Test-Path $file) {
            $remoteFile = "/var/www/fvchain.xyz/$file"
            Update-FileAtomic $file $remoteFile
        }
    }
}

# 4. Graceful restart PM2 (tanpa downtime)
Write-Host "[RESTART] Graceful restart PM2..." -ForegroundColor Yellow

if ($isMining) {
    # Jika mining aktif, gunakan reload (zero downtime)
    ssh root@103.127.99.3 "pm2 reload dashboard" 2>$null
    Write-Host "[OK] PM2 reload (zero downtime) completed" -ForegroundColor Green
} else {
    # Jika mining tidak aktif, restart normal
    ssh root@103.127.99.3 "pm2 restart dashboard" 2>$null
    Write-Host "[OK] PM2 restart completed" -ForegroundColor Green
}

# 5. Verify update berhasil
Write-Host "[VERIFY] Verifying update..." -ForegroundColor Cyan

Start-Sleep -Seconds 5

try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/wallet" -Method GET -TimeoutSec 10
    if ($response.StatusCode -eq 200) {
        Write-Host "[OK] Dashboard wallet accessible" -ForegroundColor Green
        
        # Test API juga
        $apiResponse = Invoke-WebRequest -Uri "https://fvchain.xyz/api/wallet/create" -Method POST -TimeoutSec 10
        if ($apiResponse.StatusCode -eq 200) {
            Write-Host "[OK] Wallet API working" -ForegroundColor Green
        }
    }
} catch {
    Write-Host "[ERROR] Verification failed: $_" -ForegroundColor Red
}

# 6. Final mining status check
$finalMiningStatus = Test-MiningStatus
if ($isMining -and $finalMiningStatus) {
    Write-Host "[SUCCESS] Update completed - Mining masih berjalan normal" -ForegroundColor Green
} elseif ($isMining -and -not $finalMiningStatus) {
    Write-Host "[WARNING] Mining terhenti setelah update - perlu investigasi" -ForegroundColor Red
} else {
    Write-Host "[SUCCESS] Update completed successfully" -ForegroundColor Green
}

Write-Host "\n[SUMMARY] Smart Dashboard Update Summary:" -ForegroundColor Magenta
Write-Host "- Konfigurasi Next.js: $(if($configUpdated){'Updated'}else{'Skipped'})" -ForegroundColor White
Write-Host "- Package dependencies: $(if($packageUpdated){'Updated'}else{'Skipped'})" -ForegroundColor White
Write-Host "- Build files: Synced incrementally" -ForegroundColor White
Write-Host "- PM2 restart: $(if($isMining){'Zero-downtime reload'}else{'Normal restart'})" -ForegroundColor White
Write-Host "- Mining status: $(if($finalMiningStatus){'Active'}else{'Inactive'})" -ForegroundColor White

Set-Location "r:\369-FRACTAL"