# 🔄 Panduan Recovery Wallet di Device Baru

## Situasi Anda
Anda bisa membuat PIN di device pertama, tetapi tidak bisa membuat PIN di device kedua. Ini adalah **fitur keamanan** yang disebut **Device Isolation**.

## Kenapa Ini Terjadi?
- Setiap device memiliki ID unik
- PIN disimpan secara lokal di setiap device
- Wallet Anda tidak bisa langsung diakses di device baru
- Private key Anda aman terenkripsi di device lama

## 🔧 Solusi Langsung (2 Menit)

### Langkah 1: Ambil Private Key dari Device Lama
1. Buka dashboard di device lama: `http://localhost:3001`
2. Masuk ke **Wallet Panel**
3. Klik **Export Private Key**
4. Simpan private key dengan aman (catat atau copy)

### Langkah 2: Setup di Device Baru
1. Buka dashboard di device baru: `http://localhost:3001`
2. Klik **Create New Wallet**
3. Buat PIN baru (6-12 digit)
4. Klik **Import Wallet**
5. Paste private key dari device lama
6. Klik **Import**

### Langkah 3: Verifikasi
- Pastikan address sama dengan device lama
- Periksa saldo sudah muncul
- Test kirim 0.01 FVC untuk verifikasi

## 🎯 Quick Diagnostic Tool

Buka file ini di browser untuk cek status device Anda:
```
r:\369-FRACTAL\fvchain-diagnostic.html
```

Atau klik: [Diagnostic Tool](fvchain-diagnostic.html)

## 📱 LAN Access untuk Multi-Device

Dashboard sekarang bisa diakses dari LAN:
- **Device lama**: `http://localhost:3001`
- **Device baru**: Gunakan IP komputer Anda

Untuk cek IP komputer Anda:
1. Buka Command Prompt
2. Ketik: `ipconfig`
3. Cari: **IPv4 Address**
4. Device baru akses: `http://[IP-ANDA]:3001`

## 🛡️ Security Checklist

- ✅ Private key selalu dienkripsi
- ✅ PIN tidak bisa dipindahkan antar device
- ✅ Setiap device butuh setup baru
- ✅ Private key adalah backup universal

## ⚡ Emergency Commands

Jika dashboard tidak bisa diakses:

```bash
# Restart dashboard
cd r:\369-FRACTAL\dashboard
npx next dev -H 0.0.0.0 -p 3001
```

## 🆘 Masalah Umum

### "PIN creation failed"
- **Penyebab**: Device ID conflict atau storage penuh
- **Solusi**: Clear browser cache atau gunakan browser incognito

### "Private key not valid"
- **Penyebab**: Format salah atau typo
- **Solusi**: Cek kembali setiap karakter private key

### "Address tidak sama"
- **Penyebab**: Private key salah diimport
- **Solusi**: Ambil ulang private key dari device lama

## 📞 Support

Jika masih ada masalah:
1. Jalankan diagnostic tool
2. Screenshot hasil diagnostic
3. Hubungi support dengan informasi lengkap

## ✅ Status Saat Ini

- ✅ Dashboard berjalan di port 3001
- ✅ LAN access enabled
- ✅ Device isolation aktif
- ✅ Recovery guide tersedia
- ✅ Diagnostic tool siap digunakan

**Selamat mencoba! Wallet Anda aman dan bisa diakses di semua device dengan private key.**