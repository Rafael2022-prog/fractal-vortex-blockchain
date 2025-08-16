#!/usr/bin/env pwsh
# Fix Mining Page Routing Issues
# Mengatasi masalah ERR_ABORTED pada halaman mining

Write-Host "[INFO] Memperbaiki masalah routing halaman mining..." -ForegroundColor Cyan

# 1. Backup current mining page
Write-Host "[BACKUP] Membuat backup halaman mining..." -ForegroundColor Yellow
if (Test-Path "dashboard/src/app/mining/page.tsx") {
    Copy-Item "dashboard/src/app/mining/page.tsx" "dashboard/src/app/mining/page.tsx.backup" -Force
    Write-Host "[OK] Backup dibuat: page.tsx.backup" -ForegroundColor Green
}

# 2. Update next.config.mjs untuk mengatasi masalah routing
Write-Host "[CONFIG] Memperbarui konfigurasi Next.js..." -ForegroundColor Yellow

$nextConfig = @'
/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    serverComponentsExternalPackages: ['axios'],
  },
  output: 'standalone',
  staticPageGenerationTimeout: 120,
  poweredByHeader: false,
  compress: true,
  async headers() {
    return [
      {
        source: '/(.*)',
        headers: [
          {
            key: 'Cache-Control',
            value: 'no-cache, no-store, must-revalidate',
          },
          {
            key: 'Pragma',
            value: 'no-cache',
          },
          {
            key: 'Expires',
            value: '0',
          },
        ],
      },
      {
        source: '/mining',
        headers: [
          {
            key: 'X-Frame-Options',
            value: 'SAMEORIGIN',
          },
          {
            key: 'X-Content-Type-Options',
            value: 'nosniff',
          },
        ],
      },
    ];
  },
  async redirects() {
    return [
      {
        source: '/mining/:path*',
        has: [
          {
            type: 'query',
            key: '_rsc',
          },
        ],
        destination: '/mining',
        permanent: false,
      },
    ];
  },
};

export default nextConfig;
'@

$nextConfig | Out-File -FilePath "dashboard/next.config.mjs" -Encoding UTF8 -Force
Write-Host "[OK] Konfigurasi Next.js diperbarui" -ForegroundColor Green

# 3. Clear Next.js cache
Write-Host "[CACHE] Membersihkan cache Next.js..." -ForegroundColor Yellow
Set-Location dashboard
if (Test-Path ".next") {
    Remove-Item ".next" -Recurse -Force
    Write-Host "[OK] Cache .next dibersihkan" -ForegroundColor Green
}

# 4. Install dependencies if needed
Write-Host "[DEPS] Memeriksa dependensi..." -ForegroundColor Yellow
if (-not (Test-Path "node_modules")) {
    Write-Host "[INSTALL] Menginstal dependensi..." -ForegroundColor Yellow
    npm install
}

# 5. Build dashboard
Write-Host "[BUILD] Membangun dashboard..." -ForegroundColor Yellow
try {
    npm run build
    Write-Host "[OK] Build berhasil" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Build gagal: $_" -ForegroundColor Red
    Set-Location ..
    exit 1
}

# 6. Test locally
Write-Host "[TEST] Menguji secara lokal..." -ForegroundColor Yellow
try {
    Start-Process "npm" -ArgumentList "run", "start" -NoNewWindow -PassThru
    Start-Sleep 3
    Write-Host "[OK] Server lokal dimulai" -ForegroundColor Green
} catch {
    Write-Host "[WARN] Gagal memulai server lokal: $_" -ForegroundColor Yellow
}

Set-Location ..

Write-Host "[SUCCESS] Perbaikan routing halaman mining selesai!" -ForegroundColor Green
Write-Host "[INFO] Perubahan yang dilakukan:" -ForegroundColor Cyan
Write-Host "  - Backup halaman mining dibuat" -ForegroundColor White
Write-Host "  - Konfigurasi Next.js diperbarui dengan redirect _rsc" -ForegroundColor White
Write-Host "  - Cache Next.js dibersihkan" -ForegroundColor White
Write-Host "  - Dashboard dibangun ulang" -ForegroundColor White
Write-Host "[NEXT] Jalankan smart-dashboard-update.ps1 untuk deploy ke server" -ForegroundColor Yellow