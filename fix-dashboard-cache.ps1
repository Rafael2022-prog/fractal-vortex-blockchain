# Script untuk memperbaiki masalah cache dashboard tanpa deployment penuh

Write-Host "[FIX] Memperbaiki masalah cache dashboard FVChain..." -ForegroundColor Cyan

# Fungsi untuk menjalankan perintah SSH
function Invoke-SSHCommand {
    param(
        [string]$Command,
        [string]$Description
    )
    
    Write-Host "[SSH] $Description..." -ForegroundColor Yellow
    try {
        $result = ssh root@103.127.99.3 $Command
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[OK] $Description berhasil" -ForegroundColor Green
            return $result
        } else {
            Write-Host "[ERROR] $Description gagal" -ForegroundColor Red
            return $null
        }
    } catch {
        Write-Host "[ERROR] Error: $_" -ForegroundColor Red
        return $null
    }
}

# 1. Restart PM2 dashboard untuk clear cache
Invoke-SSHCommand "pm2 restart dashboard" "Restart dashboard PM2"

# 2. Clear Next.js cache
Invoke-SSHCommand "cd /var/www/fvchain.xyz && rm -rf .next/cache" "Clear Next.js cache"

# 3. Rebuild Next.js (hanya jika diperlukan)
Write-Host "[BUILD] Rebuilding Next.js..." -ForegroundColor Yellow
Invoke-SSHCommand "cd /var/www/fvchain.xyz && npm run build" "Rebuild Next.js"

# 4. Restart Nginx untuk clear server cache
Invoke-SSHCommand "systemctl reload nginx" "Reload Nginx"

# 5. Clear browser cache headers
Invoke-SSHCommand "cd /var/www/fvchain.xyz && find . -name '*.js' -o -name '*.css' -o -name '*.html' | xargs touch" "Update file timestamps"

# 6. Restart PM2 dashboard lagi
Invoke-SSHCommand "pm2 restart dashboard" "Final restart dashboard"

# 7. Check status
Write-Host "[STATUS] Checking dashboard status..." -ForegroundColor Cyan
$status = Invoke-SSHCommand "pm2 status dashboard" "Check PM2 status"
if ($status) {
    Write-Host $status -ForegroundColor White
}

# 8. Test endpoint
Write-Host "[TEST] Testing dashboard endpoint..." -ForegroundColor Cyan
try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/wallet" -Method GET -TimeoutSec 10
    if ($response.StatusCode -eq 200) {
        Write-Host "[OK] Dashboard wallet page accessible" -ForegroundColor Green
    } else {
        Write-Host "[WARN] Dashboard response code: $($response.StatusCode)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "[ERROR] Dashboard test failed: $_" -ForegroundColor Red
}

# 9. Test transactions endpoint
Write-Host "[TEST] Testing transactions API..." -ForegroundColor Cyan
try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/api/transactions" -Method GET -TimeoutSec 10
    if ($response.StatusCode -eq 200) {
        Write-Host "[OK] Transactions API accessible" -ForegroundColor Green
    } else {
        Write-Host "[WARN] Transactions API response code: $($response.StatusCode)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "[ERROR] Transactions API test failed: $_" -ForegroundColor Red
}

Write-Host "`n[DONE] Cache fix completed!" -ForegroundColor Green
Write-Host "[TIPS] Tips untuk user:" -ForegroundColor Cyan
Write-Host "   - Lakukan hard refresh (Ctrl+Shift+R) di browser" -ForegroundColor White
Write-Host "   - Clear browser cache jika masih ada masalah" -ForegroundColor White
Write-Host "   - Coba buka dalam incognito/private mode" -ForegroundColor White
Write-Host "`n[URL] Test URL: https://fvchain.xyz/wallet" -ForegroundColor Magenta