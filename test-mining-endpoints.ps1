# Script untuk test endpoint mining setelah perbaikan
Write-Host "Testing mining endpoints..." -ForegroundColor Cyan

try {
    # Test miner status endpoint
    Write-Host "Testing /api/mining/miner/status..." -ForegroundColor Yellow
    $response = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/miner/status" -Method POST -ContentType "application/json" -Body '{"device_id":"test_device"}' -UseBasicParsing
    Write-Host "Status Code: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "Response: $($response.Content.Substring(0, [Math]::Min(200, $response.Content.Length)))" -ForegroundColor White
    
    # Test events endpoint
    Write-Host "\nTesting /api/mining/events..." -ForegroundColor Yellow
    $eventsResponse = Invoke-WebRequest -Uri "https://fvchain.xyz/api/mining/events" -Method GET -UseBasicParsing -TimeoutSec 5
    Write-Host "Events Status Code: $($eventsResponse.StatusCode)" -ForegroundColor Green
    
    Write-Host "\n✅ SEMUA ENDPOINT MINING BERFUNGSI!" -ForegroundColor Green
    Write-Host "Halaman mining di https://fvchain.xyz/mining sudah siap digunakan." -ForegroundColor Cyan
    
} catch {
    Write-Host "Error testing endpoints: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Status Code: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Yellow
}

Write-Host "\nTest selesai." -ForegroundColor White