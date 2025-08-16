# Panduan Setup Wallet di Device Baru (Device Lama Masih Work)

## Situasi Anda
✅ **Device Lama**: PIN sudah dibuat dan wallet aktif  
❌ **Device Baru**: Tidak bisa membuat PIN baru  

## Penyebab
Ini **normal** dan **by design** untuk keamanan. Setiap device memerlukan:
1. PIN baru yang unik untuk device tersebut
2. Import wallet menggunakan private key dari device lama

## Langkah-by-Langkah Lengkap

### 🔍 LANGKAH 1: Ambil Private Key dari Device Lama

1. **Login di device lama** (yang sudah berhasil)
2. **Buka wallet panel** di dashboard
3. **Klik tombol "Export Private Key"**
4. **Simpan private key** di tempat sangat amat aman
   - Catat di kertas
- Jangan screenshot
- Jangan simpan di cloud

```
Contoh private key (jangan bagikan ini):
0x4f3c9b2a1d8e7f6c5b4a9d2e1f8c7b6a5d4e3f2c1b0a9f8e7d6c5b4a3f2e1
```

### 🆕 LANGKAH 2: Setup Device Baru

1. **Buka dashboard FVChain di device baru**
2. **Klik "Buat PIN Wallet"**
3. **Buat PIN baru** (bisa berbeda dengan device lama)
   - Minimal 6 digit
   - Hanya angka
   - Simpan di tempat aman

### 📥 LANGKAH 3: Import Wallet

1. **Setelah PIN dibuat**, klik **"Import Wallet"**
2. **Masukkan private key** dari langkah 1
3. **Klik "Import"**
4. **Verifikasi** address dan saldo sama dengan device lama

### ✅ LANGKAH 4: Verifikasi

**Checklist verifikasi:**
- [ ] Address wallet sama dengan device lama
- [ ] Saldo sama dengan device lama
- [ ] Transaksi history sama (jika ada)
- [ ] Private key sudah dihapus dari clipboard

## Troubleshooting Device Baru

### Error: "Gagal membuat PIN"

#### Solusi 1: Browser Issues
```
1. Keluar dari mode private/incognito
2. Enable JavaScript: Settings → Privacy & Security → JavaScript → Allow
3. Clear cache: Ctrl+Shift+Delete → Clear browsing data
4. Restart browser
```

#### Solusi 2: Storage Issues
```
1. Buka F12 (Developer Tools)
2. Tab Application → Local Storage
3. Check apakah ada entry: fvchain_device_id
4. Jika tidak ada: refresh halaman
```

#### Solusi 3: Network Issues
```
1. Pastikan internet stabil
2. Coba refresh halaman (Ctrl+F5)
3. Coba browser lain (Chrome/Firefox/Safari)
```

### Error: "Import wallet gagal"

#### Solusi 1: Private Key Format
```
1. Pastikan private key lengkap (64 karakter hex)
2. Tidak ada spasi di awal/akhir
3. Tidak ada karakter khusus
```

#### Solusi 2: Browser Console Check
```
1. Buka F12 → Console tab
2. Lihat error messages
3. Screenshot error untuk support
```

## Visual Guide (Text-based)

### Device Lama (Working)
```
┌─────────────────────────────────┐
│ FVChain Dashboard               │
│                                 │
│ ┌─────────────────────────────┐ │
│ │ Wallet Panel                │ │
│ │ Address: 0x1234...abcd      │ │
│ │ Balance: 1000 FVC           │ │
│ │ [Export Private Key]        │ │
│ └─────────────────────────────┘ │
└─────────────────────────────────┘
```

### Device Baru (Setup)
```
┌─────────────────────────────────┐
│ Buat PIN Wallet                 │
│                                 │
│ ┌─────────────────────────────┐ │
│ │ PIN: [______]               │ │
│ │ Confirm: [______]           │ │
│ │ [Buat PIN]                  │ │
│ └─────────────────────────────┘ │
│                                 │
│ 💡 PIN unik untuk device ini    │
└─────────────────────────────────┘
```

### Import Process
```
┌─────────────────────────────────┐
│ Import Wallet                   │
│                                 │
│ Private Key:                    │
│ [0x4f3c9b2a1d8e7f6c...____]    │
│                                 │
│ [Import Wallet] [Cancel]       │
└─────────────────────────────────┘
```

## FAQ Device Baru

### Q: Kenapa tidak bisa pakai PIN yang sama?
**A:** Keamanan device isolation - setiap device memiliki PIN terpisah untuk mencegah akses tidak sah.

### Q: Apakah saldo akan hilang?
**A:** Tidak, saldo akan sama karena menggunakan private key yang sama.

### Q: Berapa lama proses ini?
**A:** 2-3 menit jika semua langkah diikuti dengan benar.

### Q: Apakah bisa pakai mnemonic phrase?
**A:** Untuk saat ini belum, hanya private key. Mnemonic akan ditambahkan di update berikutnya.

## Emergency Contacts

Jika masalah berlanjut:
1. **Screenshot error** dari browser console (F12)
2. **Catat device ID** yang terlihat di wallet
3. **Simpan private key** di tempat aman
4. **Hubungi support** dengan informasi lengkap

## Quick Commands (Windows)

```powershell
# Jalankan diagnostic script
.\diagnose-wallet-pin.ps1

# Clear browser cache Chrome
Start-Process "chrome://settings/clearBrowserData"

# Clear browser cache Firefox
Start-Process "about:preferences#privacy"
```

## Success Checklist

**Setelah setup berhasil:**
- [ ] PIN baru dibuat di device baru
- [ ] Wallet imported dengan private key
- [ ] Address sama dengan device lama
- [ ] Saldo sama dengan device lama
- [ ] Private key sudah dihapus dari clipboard
- [ ] Device baru siap digunakan

---

**Dokumen ini untuk situasi spesifik:** Device lama work, device baru tidak bisa membuat PIN. Gunakan langkah-langkah di atas untuk setup yang benar.