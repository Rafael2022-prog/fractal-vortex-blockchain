# Diagnostic Script untuk PIN Wallet Cross-Device Issues
# FVChain Wallet Security Diagnostic Tool

Write-Host "=== FVCHAIN WALLET PIN DIAGNOSTIC TOOL ===" -ForegroundColor Green
Write-Host "Membantu troubleshoot masalah PIN di device baru" -ForegroundColor Yellow

# 1. Check Browser Compatibility
Write-Host "`n[1] Memeriksa kompatibilitas browser..." -ForegroundColor Cyan

# Browser check function
function Test-BrowserCompatibility {
    $userAgent = $env:HTTP_USER_AGENT
    Write-Host "User Agent: $userAgent"
    
    # Check for modern browser features
    $features = @(
        @{Name="localStorage"; Test="typeof(localStorage) !== 'undefined'"},
        @{Name="crypto"; Test="typeof(crypto) !== 'undefined'"},
        @{Name="crypto.subtle"; Test="typeof(crypto.subtle) !== 'undefined'"}
    )
    
    Write-Host "Browser features yang dibutuhkan:"
    foreach ($feature in $features) {
        Write-Host "  $($feature.Name): $($feature.Test)"
    }
}

# 2. Check LocalStorage Status
Write-Host "`n[2] Memeriksa LocalStorage..." -ForegroundColor Cyan
Write-Host "Pastikan browser tidak dalam mode private/incognito"
Write-Host "Pastikan JavaScript enabled"
Write-Host "Pastikan third-party cookies tidak diblokir"

# 3. Device ID Check
Write-Host "`n[3] Device ID Information:" -ForegroundColor Cyan
Write-Host "Setiap device memiliki ID unik yang di-generate otomatis"
Write-Host "PIN disimpan berdasarkan device ID ini"
Write-Host "Device baru = Device ID baru = PIN baru"

# 4. Wallet Recovery Steps
Write-Host "`n[4] LANGKAH RECOVERY WALLET DI DEVICE BARU:" -ForegroundColor Green
Write-Host ""
Write-Host "A. Dapatkan Private Key dari Device Lama:" -ForegroundColor Yellow
Write-Host "   1. Login di device lama (yang sudah berhasil)"
Write-Host "   2. Buka wallet panel"
Write-Host "   3. Klik 'Export Private Key'"
Write-Host "   4. Simpan private key di tempat amat sangat aman"
Write-Host ""
Write-Host "B. Setup Device Baru:" -ForegroundColor Yellow
Write-Host "   1. Buka dashboard di device baru"
Write-Host "   2. Klik 'Buat PIN Wallet'"
Write-Host "   3. Buat PIN baru (bisa berbeda dengan device lama)"
Write-Host "   4. Klik 'Import Wallet'"
Write-Host "   5. Masukkan private key dari langkah A"
Write-Host "   6. Wallet akan restore dengan saldo yang sama"

# 5. Common Issues Checklist
Write-Host "`n[5] CHECKLIST MASALUM UMUM:" -ForegroundColor Red
Write-Host "   ☐ Browser dalam mode private/incognito"
Write-Host "   ☐ JavaScript disabled"
Write-Host "   ☐ Third-party cookies blocked"
Write-Host "   ☐ Browser cache penuh"
Write-Host "   ☐ LocalStorage quota exceeded"
Write-Host "   ☐ Network connectivity issues"

# 6. Browser Debug Instructions
Write-Host "`n[6] CARA DEBUG DI BROWSER:" -ForegroundColor Cyan
Write-Host "   1. Buka browser console (F12)"
Write-Host "   2. Ketik: localStorage.getItem('fvchain_device_id')"
Write-Host "   3. Ketik: localStorage.getItem('wallet_pin_' + localStorage.getItem('fvchain_device_id'))"
Write-Host "   4. Check untuk error messages di console"

# 7. Quick Fix Commands
Write-Host "`n[7] QUICK FIX COMMANDS:" -ForegroundColor Green
Write-Host "   Clear browser cache: Ctrl+Shift+Delete"
Write-Host "   Hard refresh: Ctrl+F5"
Write-Host "   Check console: F12 → Console tab"
Write-Host "   Test localStorage: F12 → Application → Local Storage"

# 8. Emergency Recovery
Write-Host "`n[8] EMERGENCY RECOVERY:" -ForegroundColor Magenta
Write-Host "   Jika semua gagal, gunakan wallet CLI:"
Write-Host "   ```bash"
Write-Host "   cargo run --bin wallet-cli -- --recover"
Write-Host "   ```"

Write-Host "`n=== DIAGNOSTIC SELESAI ===" -ForegroundColor Green
Write-Host "Gunakan langkah recovery di atas untuk device baru" -ForegroundColor Yellow

# Browser-based diagnostic tool
Write-Host "`n[9] BROWSER DIAGNOSTIC TOOL:" -ForegroundColor Cyan
Write-Host "Simpan kode di bawah sebagai fvchain-diagnostic.html dan buka di browser"
Write-Host "```html"
Write-Host "<!DOCTYPE html>"
Write-Host "<html><head><title>FVChain Diagnostic</title></head><body>"
Write-Host "<h2>FVChain Wallet Diagnostic Tool</h2>"
Write-Host "<script>"
Write-Host "  document.write('<h3>Browser Check:</h3>');"
Write-Host "  document.write('localStorage: ' + (typeof localStorage !== 'undefined') + '<br>');"
Write-Host "  document.write('crypto: ' + (typeof crypto !== 'undefined') + '<br>');"
Write-Host "  document.write('crypto.subtle: ' + (typeof crypto.subtle !== 'undefined') + '<br>');"
Write-Host "  document.write('<h3>Device ID:</h3>');"
Write-Host "  document.write(localStorage.getItem('fvchain_device_id') || 'Not found');"
Write-Host "  document.write('<h3>PIN Status:</h3>');"
Write-Host "  const deviceId = localStorage.getItem('fvchain_device_id');"
Write-Host "  document.write(localStorage.getItem('wallet_pin_' + deviceId) ? 'PIN exists' : 'PIN not found');"
Write-Host "</script>"
Write-Host "</body></html>"
Write-Host "```"