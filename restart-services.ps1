# Manual restart services on cloud server
Write-Host "=== RESTARTING SERVICES ON CLOUD SERVER ===" -ForegroundColor Green

# Try different SSH approaches
$commands = @(
    "systemctl restart fvc-rpc",
    "systemctl restart nginx",
    "systemctl status fvc-rpc"
)

foreach ($cmd in $commands) {
    Write-Host "\nExecuting: $cmd" -ForegroundColor Yellow
    
    # Try with ssh command directly
    try {
        $result = ssh root@103.245.38.44 $cmd 2>&1
        Write-Host "✅ Command executed successfully" -ForegroundColor Green
        Write-Host "Output: $result" -ForegroundColor Cyan
    } catch {
        Write-Host "❌ SSH command failed: $_" -ForegroundColor Red
        
        # Alternative: try with Invoke-Expression
        try {
            $sshCmd = "ssh root@103.245.38.44 '$cmd'"
            $result = Invoke-Expression $sshCmd
            Write-Host "✅ Alternative method worked" -ForegroundColor Green
            Write-Host "Output: $result" -ForegroundColor Cyan
        } catch {
            Write-Host "❌ Alternative method also failed: $_" -ForegroundColor Red
        }
    }
}

Write-Host "\n=== RESTART COMPLETE ===" -ForegroundColor Green
Write-Host "Please test the device isolation again." -ForegroundColor White