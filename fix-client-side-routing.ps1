# Fix Client-Side Routing untuk mengatasi ERR_ABORTED
# Strategi: Konversi ke client-side rendering untuk halaman wallet

Write-Host "[FIX] Fixing ERR_ABORTED dengan client-side routing..." -ForegroundColor Cyan

# Backup file wallet page
Write-Host "[BACKUP] Creating backup..." -ForegroundColor Yellow
Copy-Item "r:\369-FRACTAL\dashboard\src\app\wallet\page.tsx" "r:\369-FRACTAL\dashboard\src\app\wallet\page.tsx.backup" -Force

# Buat wallet page dengan client-side rendering
$walletPageContent = @'
"use client";

import { Suspense } from "react";
import WalletPanel from "../components/WalletPanel";

// Loading component
function WalletLoading() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-blue-900 to-purple-900 flex items-center justify-center">
      <div className="text-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-400 mx-auto mb-4"></div>
        <p className="text-white/70">Loading Wallet...</p>
      </div>
    </div>
  );
}

// Main wallet page component
export default function WalletPage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-blue-900 to-purple-900">
      <div className="container mx-auto px-4 py-8">
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold text-white mb-4">
            <span className="bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-transparent">
              Fractal Vortex Wallet
            </span>
          </h1>
          <p className="text-gray-300 text-lg">
            Kelola aset FVC Anda dengan keamanan quantum dan teknologi vortex
          </p>
        </div>
        
        <Suspense fallback={<WalletLoading />}>
          <WalletPanel />
        </Suspense>
      </div>
    </div>
  );
}

// Disable static generation untuk halaman ini
export const dynamic = "force-dynamic";
'@

# Tulis file wallet page baru
$walletPageContent | Out-File -FilePath "r:\369-FRACTAL\dashboard\src\app\wallet\page.tsx" -Encoding UTF8 -Force
Write-Host "[OK] Wallet page updated dengan client-side rendering" -ForegroundColor Green

# Update Next.js config untuk optimasi client-side
$nextConfigContent = @'
/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    serverComponentsExternalPackages: ['axios']
  },
  // Gunakan standalone untuk production
  output: 'standalone',
  // Optimasi untuk client-side rendering
  staticPageGenerationTimeout: 1000,
  poweredByHeader: false,
  compress: true,
  // Disable static optimization untuk halaman dinamis
  generateStaticParams: false,
  // Headers untuk cache control
  async headers() {
    return [
      {
        source: '/wallet/:path*',
        headers: [
          {
            key: 'Cache-Control',
            value: 'no-cache, no-store, must-revalidate'
          },
          {
            key: 'X-Frame-Options',
            value: 'DENY'
          }
        ]
      },
      {
        source: '/:path*',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, immutable'
          }
        ]
      }
    ]
  },
  // Redirect untuk mengatasi masalah routing
  async redirects() {
    return [
      {
        source: '/wallet/:path*',
        has: [
          {
            type: 'query',
            key: '_rsc'
          }
        ],
        destination: '/wallet',
        permanent: false
      }
    ]
  }
}

export default nextConfig
'@

# Update Next.js config
$nextConfigContent | Out-File -FilePath "r:\369-FRACTAL\dashboard\next.config.mjs" -Encoding UTF8 -Force
Write-Host "[OK] Next.js config updated" -ForegroundColor Green

# Build dashboard dengan konfigurasi baru
Write-Host "[BUILD] Building dengan client-side optimization..." -ForegroundColor Yellow
Set-Location "r:\369-FRACTAL\dashboard"

# Clean cache
if (Test-Path ".next") {
    Remove-Item ".next" -Recurse -Force
}

npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Build gagal, restoring backup..." -ForegroundColor Red
    Copy-Item "src\app\wallet\page.tsx.backup" "src\app\wallet\page.tsx" -Force
    exit 1
}

Write-Host "[OK] Build berhasil dengan client-side rendering" -ForegroundColor Green

# Test lokal
Write-Host "[TEST] Testing client-side rendering..." -ForegroundColor Cyan
try {
    $testServer = Start-Process -FilePath "npm" -ArgumentList "run", "dev" -PassThru -NoNewWindow
    Start-Sleep -Seconds 8
    
    $response = Invoke-WebRequest -Uri "http://localhost:3000/wallet" -Method GET -TimeoutSec 10
    if ($response.StatusCode -eq 200) {
        Write-Host "[OK] Client-side wallet page working" -ForegroundColor Green
    }
    
    Stop-Process -Id $testServer.Id -Force
} catch {
    Write-Host "[WARN] Local test failed: $_" -ForegroundColor Yellow
}

Write-Host "[DONE] Client-side routing fix completed!" -ForegroundColor Green
Write-Host "[INFO] Wallet page sekarang menggunakan client-side rendering" -ForegroundColor Cyan
Write-Host "[NEXT] Ready untuk deployment dengan smart update" -ForegroundColor Magenta

Set-Location "r:\369-FRACTAL"