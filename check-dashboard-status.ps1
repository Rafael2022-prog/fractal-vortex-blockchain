# Script untuk memeriksa status layanan dashboard dan nginx
$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "Memeriksa status layanan dashboard dan nginx..." -ForegroundColor Green

# Membuat script server-side untuk memeriksa status
$serverScript = @'
#!/bin/bash
set -e

echo "=== Status Layanan ==="
echo "Status fvc-dashboard:"
systemctl status fvc-dashboard --no-pager

echo "\nStatus nginx:"
systemctl status nginx --no-pager

echo "\n=== Konfigurasi Port ==="
echo "Port yang digunakan:"
ss -tlnp | grep -E ':(3000|3001|80|443)'

echo "\n=== Konfigurasi Dashboard Service ==="
cat /etc/systemd/system/fvc-dashboard.service | grep PORT

echo "\n=== Konfigurasi Nginx ==="
grep -r "proxy_pass" /etc/nginx/conf.d/

echo "\n=== Log Dashboard ==="
journalctl -u fvc-dashboard --no-pager -n 20

echo "\n=== Log Nginx ==="
journalctl -u nginx --no-pager -n 20

echo "\n=== Coba Akses Lokal ==="
curl -I http://localhost:3000
'@

# Upload script ke server
Write-Host "Mengupload script ke server..." -ForegroundColor Cyan
$scriptPath = "/tmp/check-dashboard-status.sh"
echo y | plink -ssh -pw $Password $Username@$ServerIP "cat > $scriptPath << 'EOF'
$serverScript
EOF"

# Membuat script executable dan menjalankannya
Write-Host "Menjalankan script pemeriksaan..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "chmod +x $scriptPath && $scriptPath"

Write-Host "Pemeriksaan status layanan selesai!" -ForegroundColor Green