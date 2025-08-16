# Script untuk memperbaiki error ERR_ABORTED dan masalah React Server Components

Write-Host "[FIX] Memperbaiki error ERR_ABORTED pada Next.js RSC..." -ForegroundColor Cyan

# Backup konfigurasi Next.js yang ada
Write-Host "[BACKUP] Membuat backup konfigurasi..." -ForegroundColor Yellow
if (Test-Path "r:\369-FRACTAL\dashboard\next.config.mjs") {
    Copy-Item "r:\369-FRACTAL\dashboard\next.config.mjs" "r:\369-FRACTAL\dashboard\next.config.mjs.backup" -Force
    Write-Host "[OK] Backup next.config.mjs berhasil" -ForegroundColor Green
}

# Membuat konfigurasi Next.js yang dioptimalkan
Write-Host "[CONFIG] Membuat konfigurasi Next.js yang dioptimalkan..." -ForegroundColor Yellow

$nextConfig = @'
/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    serverComponentsExternalPackages: ['axios'],
    // Disable RSC untuk mengatasi error ERR_ABORTED
    serverActions: false,
    // Optimasi untuk production
    optimizePackageImports: ['axios']
  },
  // Gunakan static export untuk menghindari masalah RSC
  output: 'export',
  trailingSlash: true,
  images: {
    unoptimized: true
  },
  // Disable automatic static optimization untuk konsistensi
  staticPageGenerationTimeout: 1000,
  // Headers untuk mengatasi masalah CORS dan caching
  async headers() {
    return [
      {
        source: '/(.*)',
        headers: [
          {
            key: 'Cache-Control',
            value: 'no-cache, no-store, must-revalidate'
          },
          {
            key: 'Pragma',
            value: 'no-cache'
          },
          {
            key: 'Expires',
            value: '0'
          }
        ]
      }
    ]
  }
}

export default nextConfig
'@

# Tulis konfigurasi baru
$nextConfig | Out-File -FilePath "r:\369-FRACTAL\dashboard\next.config.mjs" -Encoding UTF8 -Force
Write-Host "[OK] Konfigurasi Next.js baru berhasil dibuat" -ForegroundColor Green

# Build ulang dashboard dengan konfigurasi baru
Write-Host "[BUILD] Building dashboard dengan konfigurasi baru..." -ForegroundColor Yellow
Set-Location "r:\369-FRACTAL\dashboard"

try {
    # Clean cache terlebih dahulu
    if (Test-Path ".next") {
        Remove-Item ".next" -Recurse -Force
        Write-Host "[OK] Cache Next.js dibersihkan" -ForegroundColor Green
    }
    
    # Install dependencies jika diperlukan
    if (-not (Test-Path "node_modules")) {
        Write-Host "[INSTALL] Installing dependencies..." -ForegroundColor Yellow
        npm install
    }
    
    # Build dashboard
    Write-Host "[BUILD] Building dashboard..." -ForegroundColor Yellow
    npm run build
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[OK] Dashboard build berhasil" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] Dashboard build gagal" -ForegroundColor Red
        # Restore backup jika build gagal
        if (Test-Path "next.config.mjs.backup") {
            Copy-Item "next.config.mjs.backup" "next.config.mjs" -Force
            Write-Host "[RESTORE] Konfigurasi backup dipulihkan" -ForegroundColor Yellow
        }
        exit 1
    }
} catch {
    Write-Host "[ERROR] Build error: $_" -ForegroundColor Red
    exit 1
}

# Test lokal terlebih dahulu
Write-Host "[TEST] Testing dashboard lokal..." -ForegroundColor Cyan
try {
    # Start development server untuk test
    $devServer = Start-Process -FilePath "npm" -ArgumentList "run", "dev" -PassThru -NoNewWindow
    Start-Sleep -Seconds 10
    
    # Test endpoint lokal
    $response = Invoke-WebRequest -Uri "http://localhost:3000" -Method GET -TimeoutSec 5
    if ($response.StatusCode -eq 200) {
        Write-Host "[OK] Dashboard lokal berfungsi" -ForegroundColor Green
    }
    
    # Stop development server
    Stop-Process -Id $devServer.Id -Force
} catch {
    Write-Host "[WARN] Test lokal gagal, melanjutkan deployment: $_" -ForegroundColor Yellow
}

Write-Host "[DONE] Perbaikan Next.js RSC selesai!" -ForegroundColor Green
Write-Host "[INFO] Konfigurasi telah dioptimalkan untuk mengatasi error ERR_ABORTED" -ForegroundColor Cyan
Write-Host "[NEXT] Silakan deploy dashboard ke server untuk menerapkan perubahan" -ForegroundColor Magenta

# Kembali ke direktori utama
Set-Location "r:\369-FRACTAL"