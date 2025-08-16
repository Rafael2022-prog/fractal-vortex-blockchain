# Check Binaries and Duplicates on Cloud Server
$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "Checking Binaries and Duplicates on Cloud Server..." -ForegroundColor Green

# Create a server-side check script
$serverCheckScript = @'
#!/bin/bash

# Check Binaries and Duplicates
echo "=== CHECKING BINARIES AND DUPLICATES ==="

# 1. Find all FVC-related binaries
echo "\n=== FVC Binaries ==="
find /root -type f -name "*fvc*" -executable 2>/dev/null | sort

# 2. Find all RPC-related binaries
echo "\n=== RPC Binaries ==="
find /root -type f -name "*rpc*" -executable 2>/dev/null | sort

# 3. Check target/release directory
echo "\n=== Target Release Directory ==="
if [ -d "/root/fvchain/target/release" ]; then
    ls -la /root/fvchain/target/release/ | grep -E "(fvc|rpc)"
else
    echo "Target release directory not found"
fi

# 4. Check for duplicate binaries by size and name
echo "\n=== Duplicate Binaries Check ==="
find /root -type f -executable 2>/dev/null | grep -E "(fvc|rpc)" | while read file; do
    if [ -f "$file" ]; then
        size=$(stat -c%s "$file" 2>/dev/null)
        echo "$file - Size: $size bytes"
    fi
done | sort -k3 -n

# 5. Check systemd services
echo "\n=== Systemd Services ==="
systemctl list-units --type=service | grep -E "(fvc|rpc)"

# 6. Check running processes
echo "\n=== Running Processes ==="
ps aux | grep -E "(fvc|rpc)" | grep -v grep

# 7. Check disk usage of binaries
echo "\n=== Disk Usage of Binaries ==="
find /root -type f -executable 2>/dev/null | grep -E "(fvc|rpc)" | xargs du -h 2>/dev/null | sort -hr

# 8. Check for old backup files
echo "\n=== Backup Files ==="
find /root -type f -name "*fvc*.bak" -o -name "*rpc*.bak" -o -name "*fvc*.old" -o -name "*rpc*.old" 2>/dev/null

echo "\n=== Check completed! ==="
'@

# Save the server check script to a temporary file
$serverScriptFile = "check-binaries.sh"
Set-Content -Path $serverScriptFile -Value $serverCheckScript

# Upload the server check script to the server
Write-Host "[INFO] Uploading binary check script..." -ForegroundColor Cyan
echo y | pscp -pw $Password $serverScriptFile $Username@${ServerIP}:/tmp/check-binaries.sh

# Make the server check script executable and run it
Write-Host "[INFO] Running binary check script..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "chmod +x /tmp/check-binaries.sh && /tmp/check-binaries.sh"

# Remove temporary files
Remove-Item -Path $serverScriptFile

Write-Host "\nBinary check completed!" -ForegroundColor Green