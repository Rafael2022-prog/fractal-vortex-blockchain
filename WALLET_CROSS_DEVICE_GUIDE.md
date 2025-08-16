# Panduan Cross-Device Wallet Access FVChain

## Masalah: PIN Gagal Disimpan di Device Berbeda

### Penyebab Utama
Sistem keamanan wallet FVChain menggunakan **Device Isolation** sebagai fitur keamanan utama. Ini berarti:

1. **Setiap device memiliki ID unik** yang di-generate secara otomatis
2. **PIN disimpan secara lokal** di perangkat tersebut
3. **Tidak ada sharing data** antar perangkat untuk keamanan
4. **Private key dienkripsi** dengan PIN yang berbeda per device

### Kenapa Ini Dirancang Seperti Ini?

#### Alasan Keamanan:
- **Mencegah akses tidak sah** jika satu device dicuri
- **Isolasi kebocoran data** - jika satu device terkompromi, device lain tetap aman
- **Compliance dengan standar keamanan** perbankan modern
- **Zero-trust architecture** - tidak mengandalkan server untuk otentikasi

#### Limitasi:
- User harus setup PIN baru di setiap device
- Tidak bisa "login" dengan PIN yang sama di device berbeda
- Setiap device diperlakukan sebagai wallet terpisah

## Solusi yang Tersedia

### 1. Setup PIN di Setiap Device (Recommended)

**Langkah-langkah:**
1. Buka dashboard wallet di device baru
2. Klik "Buat PIN Wallet"
3. Buat PIN baru (bisa sama atau berbeda dengan device lain)
4. Import wallet menggunakan private key atau mnemonic
5. Wallet siap digunakan di device baru

### 2. Backup & Restore Wallet

#### Metode A: Private Key Backup
```bash
# Di device lama:
1. Login ke wallet
2. Export private key
3. Simpan di tempat aman

# Di device baru:
1. Buka wallet
2. Pilih "Import Wallet"
3. Masukkan private key
4. Buat PIN baru untuk device ini
```

#### Metode B: Mnemonic Phrase (Future)
- Fitur mnemonic akan ditambahkan di update berikutnya
- Akan memungkinkan restore wallet dengan 12-24 kata

### 3. Multi-Device Sync (Experimental)

#### Konfigurasi Manual (Advanced Users)
Jika Anda menginginkan PIN yang sama di beberapa device:

```javascript
// WARNING: Hanya untuk advanced users yang mengerti risiko
// Di device pertama:
const deviceId1 = localStorage.getItem('fvchain_device_id');
const pinHash1 = localStorage.getItem(`wallet_pin_${deviceId1}`);

// Di device kedua:
const deviceId2 = localStorage.getItem('fvchain_device_id');
localStorage.setItem(`wallet_pin_${deviceId2}`, pinHash1);
```

**⚠️ Peringatan:** Metode ini mengurangi keamanan dan tidak direkomendasikan untuk production use.

## Troubleshooting PIN Issues

### Error: "Gagal menyimpan PIN"

#### Kemungkinan Penyebab:
1. **Browser tidak mendukung localStorage**
   - Solusi: Gunakan browser modern (Chrome, Firefox, Safari terbaru)
   - Check: Browser dalam mode private/incognito kadang block localStorage

2. **Storage penuh**
   - Solusi: Clear browser cache dan coba lagi
   - Check: Browser settings > Privacy > Clear browsing data

3. **JavaScript di-disable**
   - Solusi: Enable JavaScript di browser settings
   - Check: Browser tidak dalam mode "Enhanced Privacy"

4. **Third-party cookies blocked**
   - Solusi: Allow cookies untuk domain FVChain
   - Check: Browser privacy settings

### Error: "PIN tidak cocok"

#### Di Device Baru:
1. **Ini normal** - setiap device memiliki PIN terpisah
2. **Setup PIN baru** di device tersebut
3. **Import wallet** dengan private key

### Error: "Wallet terkunci"

#### Setelah 5 percobaan gagal:
1. **Tunggu 5 menit** - lockout akan otomatis reset
2. **Clear browser cache** jika perlu
3. **Pastikan PIN yang benar** sebelum mencoba lagi

## Best Practices untuk Multi-Device

### Untuk User:
1. **Catat private key** di tempat amat sangat aman
2. **Gunakan PIN yang berbeda** di setiap device untuk keamanan ekstra
3. **Jangan share PIN** antar device
4. **Backup wallet** sebelum setup di device baru

### Untuk Setup Device Baru:
```bash
# Flow yang direkomendasikan:
1. Login di device lama
2. Export private key
3. Setup PIN baru di device baru
4. Import wallet dengan private key
5. Verifikasi saldo dan transaksi
6. Hapus private key dari clipboard
```

## FAQ (Frequently Asked Questions)

### Q: Kenapa tidak bisa pakai PIN yang sama di semua device?
**A:** Untuk keamanan. Jika satu device dicuri, device lain tetap aman.

### Q: Bagaimana cara pindah wallet ke device baru?
**A:** Export private key di device lama, import di device baru dengan PIN baru.

### Q: Apakah PIN bisa di-reset?
**A:** Tidak bisa. PIN terikat dengan device. Setup PIN baru di device baru.

### Q: Apakah data wallet bisa di-sync otomatis?
**A:** Tidak untuk saat ini. Fitur sync cloud akan ditambahkan di masa depan.

### Q: Kenapa tidak ada "lupa PIN" option?
**A:** Karena semua data lokal. Gunakan private key untuk restore wallet.

## Roadmap Fitur

### Q1 2025:
- ✅ PIN Authentication per device
- ✅ Private key encryption
- ✅ Backup/restore dengan private key

### Q2 2025:
- 🔄 Mnemonic phrase backup
- 🔄 Cloud sync (optional)
- 🔄 Multi-signature support

### Q3 2025:
- 🔄 Hardware wallet integration
- 🔄 Biometric authentication
- 🔄 Social recovery

## Support

Jika masih mengalami masalah:
1. Check browser console untuk error messages
2. Clear browser cache dan cookies
3. Gunakan browser yang didukung (Chrome, Firefox, Safari)
4. Hubungi support dengan screenshot error

---

**Document Version:** 1.0  
**Last Updated:** 13 Januari 2025  
**Status:** Production Ready