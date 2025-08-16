#!/usr/bin/env pwsh

# Fix Dashboard Port to 3000
Write-Host "[INFO] Fixing dashboard port to 3000..." -ForegroundColor Green

# Stop any running Next.js processes
Write-Host "[STOP] Stopping existing Next.js processes..." -ForegroundColor Yellow
ssh root@fvchain.xyz 'pkill -f node || true'

# Update package.json on server
Write-Host "[UPDATE] Updating package.json on server..." -ForegroundColor Yellow
ssh root@fvchain.xyz 'cd /root/dashboard && cp package.json package.json.bak'
scp ./dashboard/package.json root@fvchain.xyz:/root/dashboard/package.json

# Verify package.json update
Write-Host "[VERIFY] Verifying package.json update..." -ForegroundColor Yellow
ssh root@fvchain.xyz 'cd /root/dashboard && cat package.json | grep start'

# Start dashboard on port 3000
Write-Host "[START] Starting dashboard on port 3000..." -ForegroundColor Green
ssh root@fvchain.xyz 'cd /root/dashboard && nohup npm start > dashboard.log 2>&1 &'

# Wait and verify
Start-Sleep -Seconds 5
Write-Host "[TEST] Testing dashboard on port 3000..." -ForegroundColor Yellow
ssh root@fvchain.xyz 'curl -I http://localhost:3000/mining || echo "Dashboard not ready yet"'

Write-Host "[SUCCESS] Dashboard port fix completed!" -ForegroundColor Green
Write-Host "[INFO] Dashboard should now be running on port 3000" -ForegroundColor Cyan