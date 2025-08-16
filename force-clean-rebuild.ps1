#!/usr/bin/env pwsh
# Force Clean Rebuild - Mengatasi masalah chunk JavaScript lama
# Membersihkan semua cache dan membangun ulang dari awal

Write-Host "[INFO] Memulai pembersihan menyeluruh dan rebuild..." -ForegroundColor Cyan

# 1. Stop any running processes
Write-Host "[STOP] Menghentikan proses yang berjalan..." -ForegroundColor Yellow
try {
    Stop-Process -Name "node" -Force -ErrorAction SilentlyContinue
    Stop-Process -Name "npm" -Force -ErrorAction SilentlyContinue
    Write-Host "[OK] Proses Node.js dihentikan" -ForegroundColor Green
} catch {
    Write-Host "[INFO] Tidak ada proses Node.js yang berjalan" -ForegroundColor Gray
}

Set-Location dashboard

# 2. Remove all cache and build directories
Write-Host "[CLEAN] Membersihkan semua cache dan build..." -ForegroundColor Yellow

$dirsToClean = @(".next", "node_modules/.cache", ".vercel", "out", "dist")
foreach ($dir in $dirsToClean) {
    if (Test-Path $dir) {
        Remove-Item $dir -Recurse -Force
        Write-Host "[OK] Dihapus: $dir" -ForegroundColor Green
    }
}

# 3. Clear npm cache
Write-Host "[NPM] Membersihkan cache npm..." -ForegroundColor Yellow
npm cache clean --force
Write-Host "[OK] Cache npm dibersihkan" -ForegroundColor Green

# 4. Reinstall dependencies
Write-Host "[DEPS] Menginstal ulang dependensi..." -ForegroundColor Yellow
Remove-Item "package-lock.json" -Force -ErrorAction SilentlyContinue
Remove-Item "node_modules" -Recurse -Force -ErrorAction SilentlyContinue
npm install
Write-Host "[OK] Dependensi diinstal ulang" -ForegroundColor Green

# 5. Update next.config.mjs with force refresh
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
  generateBuildId: async () => {
    // Force new build ID to invalidate all caches
    return `build-${Date.now()}-${Math.random().toString(36).substring(2, 15)}`;
  },
  async headers() {
    return [
      {
        source: '/(.*)',
        headers: [
          {
            key: 'Cache-Control',
            value: 'no-cache, no-store, must-revalidate, max-age=0',
          },
          {
            key: 'Pragma',
            value: 'no-cache',
          },
          {
            key: 'Expires',
            value: '0',
          },
          {
            key: 'X-Frame-Options',
            value: 'SAMEORIGIN',
          },
        ],
      },
      {
        source: '/_next/static/(.*)',
        headers: [
          {
            key: 'Cache-Control',
            value: 'no-cache, no-store, must-revalidate',
          },
        ],
      },
    ];
  },
  async redirects() {
    return [
      {
        source: '/:path*',
        has: [
          {
            type: 'query',
            key: '_rsc',
          },
        ],
        destination: '/:path*',
        permanent: false,
      },
    ];
  },
};

export default nextConfig;
'@

$nextConfig | Out-File -FilePath "next.config.mjs" -Encoding UTF8 -Force
Write-Host "[OK] Konfigurasi Next.js diperbarui dengan force refresh" -ForegroundColor Green

# 6. Build with clean slate
Write-Host "[BUILD] Membangun dashboard dari awal..." -ForegroundColor Yellow
try {
    npm run build
    Write-Host "[OK] Build berhasil dengan chunk baru" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Build gagal: $_" -ForegroundColor Red
    Set-Location ..
    exit 1
}

Set-Location ..

Write-Host "[SUCCESS] Pembersihan menyeluruh dan rebuild selesai!" -ForegroundColor Green
Write-Host "[INFO] Perubahan yang dilakukan:" -ForegroundColor Cyan
Write-Host "  - Semua cache dan build directory dihapus" -ForegroundColor White
Write-Host "  - Dependencies diinstal ulang" -ForegroundColor White
Write-Host "  - Build ID baru dipaksa untuk invalidasi cache" -ForegroundColor White
Write-Host "  - Header no-cache ditambahkan untuk semua file" -ForegroundColor White
Write-Host "  - Redirect _rsc parameter ditambahkan" -ForegroundColor White
Write-Host "[NEXT] Jalankan smart-dashboard-update.ps1 untuk deploy" -ForegroundColor Yellow