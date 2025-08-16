# Fix manifest.json issue on Cloud Server
$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "Fixing manifest.json issue on Cloud Server..." -ForegroundColor Green

# Create a script to fix the issue
$fixScript = @"
#!/bin/bash

# Check if manifest.json exists in the public directory
echo "[INFO] Checking manifest.json location..."
if [ -f "/opt/fvc-dashboard/public/manifest.json" ]; then
    echo "[INFO] manifest.json exists in public directory"
    # Set proper permissions
    chmod 644 /opt/fvc-dashboard/public/manifest.json
    echo "[INFO] Set permissions to 644"
else
    echo "[ERROR] manifest.json not found in public directory"
    exit 1
fi

# Create a direct symlink in the root directory
echo "[INFO] Creating symlink in root directory..."
ln -sf /opt/fvc-dashboard/public/manifest.json /opt/fvc-dashboard/manifest.json
echo "[INFO] Symlink created"

# Update nginx configuration to serve manifest.json from both locations
echo "[INFO] Updating nginx configuration..."
cat > /etc/nginx/conf.d/manifest-fix.conf << 'EOF'
server {
    listen 80;
    server_name fvchain.xyz www.fvchain.xyz;
    
    # Direct manifest.json access
    location = /manifest.json {
        add_header Content-Type application/json;
        add_header Access-Control-Allow-Origin *;
        root /opt/fvc-dashboard/public;
        try_files /manifest.json =404;
    }
}
EOF

# Test and reload nginx
echo "[INFO] Testing and reloading nginx..."
nginx -t && systemctl reload nginx

# Verify the file is accessible
echo "[INFO] Verifying manifest.json is accessible..."
curl -I http://localhost/manifest.json

echo "[INFO] Fix completed"
"@

# Upload and execute the fix script
Write-Host "[INFO] Uploading fix script..." -ForegroundColor Cyan
$fixScriptPath = "fix-manifest.sh"
$fixScript | Out-File -FilePath $fixScriptPath -Encoding UTF8
echo y | pscp -pw $Password $fixScriptPath $Username@${ServerIP}:/tmp/

# Execute the fix script
Write-Host "[INFO] Executing fix script..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "chmod +x /tmp/fix-manifest.sh && /tmp/fix-manifest.sh"

# Restart the dashboard service
Write-Host "[INFO] Restarting dashboard service..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl restart fvc-dashboard"

# Verify deployment
Write-Host "[INFO] Verifying fix..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "curl -I http://localhost/manifest.json"

Write-Host "\nFix for manifest.json completed!" -ForegroundColor Green
Write-Host "You can verify the changes at: https://fvchain.xyz/manifest.json" -ForegroundColor Yellow