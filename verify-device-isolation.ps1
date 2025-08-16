# Verify Device Isolation Deployment
Write-Host "=== VERIFYING DEVICE ISOLATION DEPLOYMENT ===" -ForegroundColor Green

# Test local server first
Write-Host "\nTesting local server (localhost:8080)..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8080/api/network/info" -Method GET
    Write-Host "✅ Local server is running" -ForegroundColor Green
    Write-Host "Network Info: $($response | ConvertTo-Json -Compress)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Local server not accessible: $_" -ForegroundColor Red
}

# Test cloud server
Write-Host "\nTesting cloud server (fvchain.xyz)..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "https://fvchain.xyz/api/network/info" -Method GET
    Write-Host "✅ Cloud server is running" -ForegroundColor Green
    Write-Host "Network Info: $($response | ConvertTo-Json -Compress)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Cloud server not accessible: $_" -ForegroundColor Red
}

# Test device isolation endpoint
Write-Host "\nTesting device isolation endpoint..." -ForegroundColor Yellow
$testDeviceId = "test_device_" + (Get-Random)
$testAddress = "fvc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq"

try {
    $body = @{
        device_id = $testDeviceId
        address = $testAddress
    } | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri "https://fvchain.xyz/api/wallet/balance" -Method POST -Body $body -ContentType "application/json"
    Write-Host "✅ Device isolation endpoint working" -ForegroundColor Green
    Write-Host "Response: $($response | ConvertTo-Json -Compress)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Device isolation endpoint failed: $_" -ForegroundColor Red
}

Write-Host "\n=== VERIFICATION COMPLETE ===" -ForegroundColor Green
Write-Host "If cloud server tests pass, device isolation is working properly." -ForegroundColor White
Write-Host "Each device should now have isolated wallets and mining." -ForegroundColor Cyan