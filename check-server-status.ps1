# Check Server Status and Logs
$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "Checking Server Status and Logs..." -ForegroundColor Green

# Create a server-side check script
$serverCheckScript = @'
#!/bin/bash

# Check Server Status and Logs
echo "Checking Server Status and Logs..."

# 1. Check Nginx status
echo "\n=== Nginx Status ==="
systemctl status nginx

# 2. Check Dashboard service status
echo "\n=== Dashboard Service Status ==="
systemctl status fvc-dashboard

# 3. Check Nginx error logs
echo "\n=== Nginx Error Logs (Last 20 lines) ==="
tail -n 20 /var/log/nginx/error.log

# 4. Check Dashboard service logs
echo "\n=== Dashboard Service Logs (Last 20 lines) ==="
journalctl -u fvc-dashboard -n 20

# 5. Check Nginx configuration
echo "\n=== Nginx Configuration ==="
nginx -T | grep -A 10 -B 10 "whitepaper"

# 6. Check if files exist and are accessible
echo "\n=== File Accessibility Check ==="
for file in WHITEPAPER_INDONESIA.md WHITEPAPER_ENGLISH.md README.md SECURITY.md; do
    echo "Checking $file..."
    if [ -f "/opt/fvc-dashboard/public/$file" ]; then
        echo "✅ $file exists in public directory"
        ls -la "/opt/fvc-dashboard/public/$file"
    else
        echo "❌ $file not found in public directory"
    fi
done

# 7. Check network connectivity
echo "\n=== Network Connectivity Check ==="
echo "Checking if dashboard service is listening on port 3001..."
netstat -tulpn | grep 3001

# 8. Check if API is accessible locally
echo "\n=== Local API Accessibility Check ==="
echo "Checking if API is accessible locally..."
curl -v http://localhost:3001/api/download/WHITEPAPER_INDONESIA.md 2>&1 | head -n 20

# 9. Check direct file access locally
echo "\n=== Local Direct File Access Check ==="
echo "Checking if file is accessible directly..."
curl -v http://localhost/WHITEPAPER_INDONESIA.md 2>&1 | head -n 20

echo "Check completed!"
'@

# Save the server check script to a temporary file
$serverScriptFile = "check-server-status.sh"
Set-Content -Path $serverScriptFile -Value $serverCheckScript

# Upload the server check script to the server
Write-Host "[INFO] Uploading server check script..." -ForegroundColor Cyan
echo y | pscp -pw $Password $serverScriptFile $Username@${ServerIP}:/tmp/check-server-status.sh

# Make the server check script executable and run it
Write-Host "[INFO] Running server check script..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "chmod +x /tmp/check-server-status.sh && /tmp/check-server-status.sh"

# Remove temporary files
Remove-Item -Path $serverScriptFile

Write-Host "\nServer status check completed!" -ForegroundColor Green