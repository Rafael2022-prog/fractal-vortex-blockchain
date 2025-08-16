#!/usr/bin/env powershell
# Verify Nginx Configuration Fix for FVChain
# This script tests all endpoints to ensure proper routing

Write-Host "=== FVChain Nginx Configuration Verification ===" -ForegroundColor Green
Write-Host ""

# Test main dashboard
Write-Host "1. Testing main dashboard..." -ForegroundColor Yellow
$response = Invoke-WebRequest -Uri "https://fvchain.xyz/" -Method HEAD -UseBasicParsing
Write-Host "   Status: $($response.StatusCode) - $($response.StatusDescription)" -ForegroundColor $(if($response.StatusCode -eq 200) {'Green'} else {'Red'})
Write-Host "   Content-Type: $($response.Headers['Content-Type'])" -ForegroundColor Cyan
Write-Host ""

# Test mining page
Write-Host "2. Testing mining page..." -ForegroundColor Yellow
$response = Invoke-WebRequest -Uri "https://fvchain.xyz/mining" -Method HEAD -UseBasicParsing
Write-Host "   Status: $($response.StatusCode) - $($response.StatusDescription)" -ForegroundColor $(if($response.StatusCode -eq 200) {'Green'} else {'Red'})
Write-Host "   Content-Type: $($response.Headers['Content-Type'])" -ForegroundColor Cyan
Write-Host ""

# Test Next.js static files
Write-Host "3. Testing Next.js static CSS..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/_next/static/css/4bd69e708fcdad65.css" -Method HEAD -UseBasicParsing
    Write-Host "   Status: $($response.StatusCode) - $($response.StatusDescription)" -ForegroundColor $(if($response.StatusCode -eq 200) {'Green'} else {'Red'})
    Write-Host "   Content-Type: $($response.Headers['Content-Type'])" -ForegroundColor Cyan
    Write-Host "   Cache-Control: $($response.Headers['Cache-Control'])" -ForegroundColor Cyan
} catch {
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Red
}
Write-Host ""

# Test SSE endpoint redirect
Write-Host "4. Testing SSE endpoint redirect..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/events" -Method HEAD -UseBasicParsing -MaximumRedirection 0
    Write-Host "   Status: $($response.StatusCode) - $($response.StatusDescription)" -ForegroundColor $(if($response.StatusCode -eq 307) {'Green'} else {'Red'})
    Write-Host "   Location: $($response.Headers['Location'])" -ForegroundColor Cyan
} catch {
    if ($_.Exception.Response.StatusCode -eq 307) {
        Write-Host "   Status: 307 - Temporary Redirect" -ForegroundColor Green
        Write-Host "   Location: $($_.Exception.Response.Headers['Location'])" -ForegroundColor Cyan
    } else {
        Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Red
    }
}
Write-Host ""

# Test SSE endpoint final destination
Write-Host "5. Testing SSE endpoint final destination..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz:8443/api/mining/events" -Method HEAD -UseBasicParsing
    Write-Host "   Status: $($response.StatusCode) - $($response.StatusDescription)" -ForegroundColor $(if($response.StatusCode -eq 200) {'Green'} else {'Red'})
    Write-Host "   Content-Type: $($response.Headers['Content-Type'])" -ForegroundColor Cyan
    Write-Host "   CORS Origin: $($response.Headers['Access-Control-Allow-Origin'])" -ForegroundColor Cyan
} catch {
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Red
}
Write-Host ""

# Test API endpoint
Write-Host "6. Testing API endpoint..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/miner/status" -Method HEAD -UseBasicParsing
    Write-Host "   Status: $($response.StatusCode) - $($response.StatusDescription)" -ForegroundColor $(if($response.StatusCode -eq 200) {'Green'} elseif($response.StatusCode -eq 404) {'Yellow'} else {'Red'})
    if ($response.StatusCode -eq 404) {
        Write-Host "   Note: 404 is expected if endpoint not implemented in backend" -ForegroundColor Gray
    }
} catch {
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Red
}
Write-Host ""

Write-Host "=== Verification Complete ===" -ForegroundColor Green
Write-Host "Summary of fixes applied:" -ForegroundColor Cyan
Write-Host "- Fixed main location / to proxy to Next.js dashboard (port 3000)" -ForegroundColor White
Write-Host "- Added specific location /_next/static/ for Next.js static files" -ForegroundColor White
Write-Host "- Maintained SSE redirect from HTTP/2 to HTTP/1.1 (port 8443)" -ForegroundColor White
Write-Host "- Preserved CORS headers for all endpoints" -ForegroundColor White
Write-Host "- Added proper caching for static assets" -ForegroundColor White
Write-Host ""