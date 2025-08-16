# Check SSL and Domain Status for fvchain.xyz
# Created: August 14, 2025

$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"
$Domain = "www.fvchain.xyz"

Write-Host "=== FVChain SSL & Domain Status Check ===" -ForegroundColor Green
Write-Host "Domain: $Domain" -ForegroundColor Cyan
Write-Host "Server IP: $ServerIP" -ForegroundColor Cyan
Write-Host "Date: $(Get-Date)" -ForegroundColor Cyan
Write-Host ""

# Check if port 443 is listening
Write-Host "[INFO] Checking if HTTPS port 443 is listening..." -ForegroundColor Yellow
echo y | plink -ssh -pw $Password $Username@$ServerIP "ss -tlnp | grep :443"

# Check SSL certificate status
Write-Host "[INFO] Checking SSL certificate..." -ForegroundColor Yellow
echo y | plink -ssh -pw $Password $Username@$ServerIP "ls -la /etc/letsencrypt/live/"

# Check nginx configuration
Write-Host "[INFO] Testing nginx configuration..." -ForegroundColor Yellow
echo y | plink -ssh -pw $Password $Username@$ServerIP "nginx -t"

# Check nginx SSL configuration file
Write-Host "[INFO] Checking SSL configuration file..." -ForegroundColor Yellow
echo y | plink -ssh -pw $Password $Username@$ServerIP "cat /etc/nginx/conf.d/fvc-dashboard-ssl.conf | head -20"

# Test local HTTPS connection
Write-Host "[INFO] Testing local HTTPS connection..." -ForegroundColor Yellow
echo y | plink -ssh -pw $Password $Username@$ServerIP "curl -k -I https://localhost"

Write-Host "[INFO] Status check completed!" -ForegroundColor Green