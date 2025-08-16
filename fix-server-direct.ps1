# Script untuk memperbaiki RPC server di cloud server
# Lokasi ditemukan: /opt/fvchain dengan systemd service fvc-rpc.service

$SERVER_IP = "103.245.38.44"
$SERVER_USER = "root"
$PROJECT_PATH = "/opt/fvchain"
$SOURCE_FILE = "/opt/fvchain/src/bin/rpc-server.rs"
$SERVICE_NAME = "fvc-rpc.service"

Write-Host "=== MEMPERBAIKI RPC SERVER DI CLOUD ==="
Write-Host "Server: $SERVER_IP"
Write-Host "Project Path: $PROJECT_PATH"
Write-Host "Service: $SERVICE_NAME"
Write-Host "Target: Mengubah binding dari 127.0.0.1:8080 ke 0.0.0.0:8080"
Write-Host ""

# 1. Backup file asli
Write-Host "1. Backup file rpc-server.rs..."
ssh $SERVER_USER@$SERVER_IP "cp $SOURCE_FILE $SOURCE_FILE.backup-$(date +%Y%m%d-%H%M%S)"

# 2. Cek isi file sebelum edit
Write-Host "2. Cek binding saat ini..."
ssh $SERVER_USER@$SERVER_IP "grep -n '127.0.0.1:8080' $SOURCE_FILE"

# 3. Edit file - ganti 127.0.0.1:8080 dengan 0.0.0.0:8080
Write-Host "3. Mengedit file rpc-server.rs..."
ssh $SERVER_USER@$SERVER_IP "sed -i 's/127\.0\.0\.1:8080/0.0.0.0:8080/g' $SOURCE_FILE"

# 4. Verifikasi perubahan
Write-Host "4. Verifikasi perubahan..."
ssh $SERVER_USER@$SERVER_IP "grep -n '0.0.0.0:8080' $SOURCE_FILE"

# 5. Compile ulang project
Write-Host "5. Compile ulang project..."
ssh $SERVER_USER@$SERVER_IP "cd $PROJECT_PATH && cargo build --release --bin rpc-server"

# 6. Stop service
Write-Host "6. Stop service $SERVICE_NAME..."
ssh $SERVER_USER@$SERVER_IP "systemctl stop $SERVICE_NAME"

# 7. Start service
Write-Host "7. Start service $SERVICE_NAME..."
ssh $SERVER_USER@$SERVER_IP "systemctl start $SERVICE_NAME"

# 8. Cek status service
Write-Host "8. Cek status service..."
ssh $SERVER_USER@$SERVER_IP "systemctl status $SERVICE_NAME --no-pager"

# 9. Tunggu sebentar untuk startup
Write-Host "9. Menunggu server startup..."
Start-Sleep -Seconds 5

# 10. Test koneksi
Write-Host "10. Test koneksi ke RPC server..."
try {
    $response = Invoke-RestMethod -Uri "http://$SERVER_IP:8080/network/info" -TimeoutSec 10
    Write-Host "✅ SUCCESS: RPC server berhasil diakses!" -ForegroundColor Green
    Write-Host "Response: $($response | ConvertTo-Json -Compress)"
} catch {
    Write-Host "❌ ERROR: Tidak bisa mengakses RPC server" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)"
    
    # Check service logs jika ada error
    Write-Host "Checking service logs..."
    ssh $SERVER_USER@$SERVER_IP "journalctl -u $SERVICE_NAME --no-pager -n 20"
}

# 11. Cek port binding
Write-Host "11. Cek port binding..."
ssh $SERVER_USER@$SERVER_IP "ss -tlnp | grep 8080"

Write-Host ""
Write-Host "=== SELESAI ==="
Write-Host "RPC server sekarang seharusnya bisa diakses dari:"
Write-Host "- http://$SERVER_IP:8080/network/info"
Write-Host "- Dashboard akan otomatis terhubung ke server"
Write-Host "- Service: systemctl status $SERVICE_NAME"