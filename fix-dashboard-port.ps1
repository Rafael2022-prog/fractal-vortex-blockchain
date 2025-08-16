# Script untuk memperbaiki konfigurasi port dashboard dari 3001 ke 3000
$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "Memperbaiki konfigurasi port dashboard dari 3001 ke 3000..." -ForegroundColor Green

# Membuat script server-side untuk memperbaiki konfigurasi
$serverScript = @'
#!/bin/bash
set -e

echo "Memeriksa konfigurasi dashboard service..."

# Memeriksa file konfigurasi systemd
if grep -q "PORT=3001" /etc/systemd/system/fvc-dashboard.service; then
    echo "Mengubah PORT=3001 menjadi PORT=3000 di fvc-dashboard.service"
    sed -i 's/PORT=3001/PORT=3000/g' /etc/systemd/system/fvc-dashboard.service
    systemctl daemon-reload
    echo "Konfigurasi systemd diperbarui."
else
    echo "PORT sudah dikonfigurasi ke 3000 atau tidak ditemukan di fvc-dashboard.service"
fi

# Memeriksa konfigurasi Nginx
if grep -q "proxy_pass http://127.0.0.1:3001" /etc/nginx/conf.d/fvc-dashboard.conf; then
    echo "Mengubah proxy_pass dari port 3001 ke 3000 di fvc-dashboard.conf"
    sed -i 's/proxy_pass http:\/\/127.0.0.1:3001/proxy_pass http:\/\/127.0.0.1:3000/g' /etc/nginx/conf.d/fvc-dashboard.conf
    sed -i 's/proxy_pass http:\/\/localhost:3001/proxy_pass http:\/\/localhost:3000/g' /etc/nginx/conf.d/fvc-dashboard.conf
    nginx -t && systemctl reload nginx
    echo "Konfigurasi Nginx diperbarui."
else
    echo "Proxy pass sudah dikonfigurasi ke port 3000 atau tidak ditemukan di fvc-dashboard.conf"
fi

# Restart dashboard service
echo "Merestart fvc-dashboard service..."
systemctl restart fvc-dashboard

# Verifikasi port yang digunakan
echo "Verifikasi port yang digunakan:"
ss -tlnp | grep -E ':(3000|3001)'

echo "Proses perbaikan selesai."
'@

# Upload script ke server
Write-Host "Mengupload script ke server..." -ForegroundColor Cyan
$scriptPath = "/tmp/fix-dashboard-port.sh"
echo y | plink -ssh -pw $Password $Username@$ServerIP "cat > $scriptPath << 'EOF'
$serverScript
EOF"

# Membuat script executable dan menjalankannya
Write-Host "Menjalankan script perbaikan..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "chmod +x $scriptPath && $scriptPath"

# Verifikasi status service
Write-Host "Memeriksa status service..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl status fvc-dashboard --no-pager"
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl status nginx --no-pager"

Write-Host "Perbaikan port dashboard selesai!" -ForegroundColor Green