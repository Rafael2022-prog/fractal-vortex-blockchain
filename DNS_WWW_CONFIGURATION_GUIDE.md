# Panduan Konfigurasi DNS untuk www.fvchain.xyz

## Status Saat Ini ✅❌
- ✅ Domain utama `fvchain.xyz` sudah mengarah ke IP `103.245.38.44`
- ❌ Subdomain `www.fvchain.xyz` belum dikonfigurasi
- ✅ Konfigurasi Nginx sudah siap untuk kedua domain
- ❌ Sertifikat SSL untuk `www.fvchain.xyz` belum tersedia

## Langkah-Langkah Konfigurasi DNS

### 1. Akses Panel DNS Provider
Masuk ke panel kontrol DNS provider tempat domain `fvchain.xyz` terdaftar.

### 2. Tambahkan Record DNS A untuk www
```
Type: A
Name: www
Value: 103.245.38.44
TTL: 300 (atau gunakan default)
```

### 3. Verifikasi Konfigurasi DNS
Setelah menambahkan record, tunggu 5-30 menit untuk propagasi DNS, kemudian verifikasi:

**Dari komputer lokal:**
```bash
nslookup www.fvchain.xyz
# Harus mengembalikan: 103.245.38.44
```

**Dari server:**
```bash
ssh root@103.245.38.44 'nslookup www.fvchain.xyz 8.8.8.8'
```

### 4. Test Akses HTTP
Setelah DNS aktif, test akses:
```bash
curl -I http://www.fvchain.xyz
# Harus mengembalikan redirect 301 ke https://fvchain.xyz
```

## Konfigurasi SSL Let's Encrypt

### Setelah DNS Aktif, Jalankan Certbot
```bash
ssh root@103.245.38.44 'certbot --nginx -d fvchain.xyz -d www.fvchain.xyz --expand'
```

### Verifikasi Sertifikat SSL
```bash
ssh root@103.245.38.44 'certbot certificates'
```

## Konfigurasi Nginx Saat Ini

Konfigurasi Nginx sudah disiapkan untuk:
- ✅ Menerima request untuk `fvchain.xyz` dan `www.fvchain.xyz`
- ✅ Redirect HTTP ke HTTPS
- ✅ Proxy ke Next.js server pada port 3000
- ✅ Header keamanan SSL

## Troubleshooting

### Jika DNS Tidak Propagasi
1. Periksa TTL record DNS (gunakan nilai rendah seperti 300)
2. Flush DNS cache lokal: `ipconfig /flushdns` (Windows)
3. Test dengan DNS server berbeda: `nslookup www.fvchain.xyz 1.1.1.1`

### Jika Certbot Gagal
1. Pastikan DNS sudah propagasi penuh
2. Periksa firewall port 80 dan 443
3. Restart Nginx: `systemctl restart nginx`

## Hasil Akhir yang Diharapkan

Setelah konfigurasi selesai:
- ✅ `http://www.fvchain.xyz` → redirect ke `https://fvchain.xyz`
- ✅ `https://www.fvchain.xyz` → redirect ke `https://fvchain.xyz`
- ✅ Sertifikat SSL valid untuk kedua domain
- ✅ Semua traffic terenkripsi dengan HTTPS

## Catatan Penting

⚠️ **PENTING**: Tanpa record DNS A untuk `www.fvchain.xyz`, Let's Encrypt tidak dapat memverifikasi kepemilikan domain dan sertifikat SSL tidak dapat diterbitkan.

📝 **Rekomendasi**: Setelah menambahkan record DNS, tunggu minimal 15 menit sebelum menjalankan certbot untuk memastikan propagasi DNS yang stabil.

---

**Status Terakhir**: Record DNS untuk www.fvchain.xyz masih perlu dikonfigurasi di DNS provider.