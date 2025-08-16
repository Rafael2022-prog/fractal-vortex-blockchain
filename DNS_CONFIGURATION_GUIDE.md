# Panduan Konfigurasi DNS untuk FVChain Let's Encrypt

## Status DNS Saat Ini ❌

**Hasil Pemeriksaan:**
```
nslookup fvchain.org
*** gpon.net can't find fvchain.org: Non-existent domain

nslookup fvchain.org 8.8.8.8
*** dns.google can't find fvchain.org: Non-existent domain
```

**Kesimpulan:** Domain `fvchain.org` belum terdaftar atau DNS belum dikonfigurasi.

## Langkah-Langkah Konfigurasi DNS

### Opsi 1: Registrasi Domain Berbayar (Recommended)

#### 1. Beli Domain fvchain.org
**Provider Domain Terpercaya:**
- Namecheap: https://www.namecheap.com
- GoDaddy: https://www.godaddy.com
- Cloudflare: https://www.cloudflare.com/products/registrar/
- Google Domains: https://domains.google

#### 2. Konfigurasi DNS Records
**Setelah membeli domain, tambahkan DNS records berikut:**

```
Type: A
Name: @
Value: 103.245.38.44
TTL: 3600

Type: A
Name: www
Value: 103.245.38.44
TTL: 3600

Type: CNAME
Name: api
Value: fvchain.org
TTL: 3600

Type: CNAME
Name: dashboard
Value: fvchain.org
TTL: 3600
```

#### 3. Verifikasi Propagasi
**Tunggu 1-48 jam, lalu cek:**
```powershell
nslookup fvchain.org
nslookup www.fvchain.org
```

### Opsi 2: Subdomain Gratis untuk Testing

#### 1. Gunakan Layanan Subdomain Gratis
**Provider Gratis:**
- DuckDNS: https://www.duckdns.org
- No-IP: https://www.noip.com
- FreeDNS: https://freedns.afraid.org

#### 2. Contoh dengan DuckDNS
```
1. Daftar di https://www.duckdns.org
2. Buat subdomain: fvchain.duckdns.org
3. Set IP: 103.245.38.44
4. Update script untuk menggunakan fvchain.duckdns.org
```

### Opsi 3: Cloudflare (Recommended untuk Production)

#### 1. Setup Cloudflare
```
1. Daftar di https://www.cloudflare.com
2. Tambahkan domain fvchain.org
3. Update nameservers di registrar
4. Konfigurasi DNS records
```

#### 2. Keuntungan Cloudflare
- ✅ CDN gratis
- ✅ DDoS protection
- ✅ SSL/TLS management
- ✅ Analytics
- ✅ Caching

## Script Update untuk Domain Alternatif

### Jika Menggunakan Subdomain Gratis
```powershell
# Update simple-letsencrypt.ps1
$DOMAIN = "fvchain.duckdns.org"  # Ganti dengan subdomain Anda
$EMAIL = "admin@gmail.com"       # Ganti dengan email valid
```

### Jika Menggunakan Domain Lain
```powershell
# Update simple-letsencrypt.ps1
$DOMAIN = "yourdomain.com"       # Ganti dengan domain Anda
$EMAIL = "admin@yourdomain.com"  # Ganti dengan email valid
```

## Testing DNS Propagation

### Command Line Tools
```bash
# Windows
nslookup fvchain.org
nslookup fvchain.org 8.8.8.8

# PowerShell
Resolve-DnsName -Name fvchain.org -Type A

# Linux/Mac
dig fvchain.org A
host fvchain.org
```

### Online Tools
```
1. https://dnschecker.org
2. https://whatsmydns.net
3. https://www.nslookup.io
4. https://mxtoolbox.com/DNSLookup.aspx
```

## Implementasi Let's Encrypt Setelah DNS Siap

### 1. Update Script Configuration
```powershell
# Edit R:\369-FRACTAL\simple-letsencrypt.ps1
$DOMAIN = "fvchain.org"          # Domain yang sudah dikonfigurasi
$EMAIL = "admin@fvchain.org"     # Email valid untuk notifikasi
```

### 2. Jalankan Script
```powershell
powershell -ExecutionPolicy Bypass -File "R:\369-FRACTAL\simple-letsencrypt.ps1"
```

### 3. Verifikasi SSL
```bash
# Test HTTPS connection
curl -I https://fvchain.org

# Check SSL certificate
openssl s_client -connect fvchain.org:443 -servername fvchain.org
```

## Troubleshooting DNS

### Common Issues

#### 1. Domain Belum Terdaftar
```
Error: Non-existent domain
Solusi: Beli domain dari registrar terpercaya
```

#### 2. DNS Records Salah
```
Error: Domain tidak resolve ke IP yang benar
Solusi: Periksa A Record di DNS management
```

#### 3. Propagasi Lambat
```
Error: DNS belum terpropagasi
Solusi: Tunggu 1-48 jam, gunakan DNS checker online
```

#### 4. Nameserver Belum Update
```
Error: Masih menggunakan nameserver lama
Solusi: Update nameserver di registrar
```

### DNS Checker Commands
```powershell
# Check multiple DNS servers
nslookup fvchain.org 8.8.8.8      # Google DNS
nslookup fvchain.org 1.1.1.1      # Cloudflare DNS
nslookup fvchain.org 208.67.222.222 # OpenDNS

# Check from different locations
dig @8.8.8.8 fvchain.org A
dig @1.1.1.1 fvchain.org A
```

## Rekomendasi untuk FVChain Production

### 1. Domain Strategy
```
Primary: fvchain.org
WWW: www.fvchain.org
API: api.fvchain.org
Dashboard: dashboard.fvchain.org
Explorer: explorer.fvchain.org
```

### 2. DNS Configuration
```
# Main domain
A     @           103.245.38.44
A     www         103.245.38.44

# Subdomains
CNAME api         fvchain.org
CNAME dashboard   fvchain.org
CNAME explorer    fvchain.org

# Email (optional)
MX    @           10 mail.fvchain.org

# Security
TXT   @           "v=spf1 include:_spf.google.com ~all"
```

### 3. SSL Strategy
```
# Wildcard certificate (recommended)
certbot certonly --dns-cloudflare --email admin@fvchain.org -d fvchain.org -d *.fvchain.org

# Multiple domains
certbot certonly --nginx --email admin@fvchain.org -d fvchain.org -d www.fvchain.org -d api.fvchain.org
```

## Next Steps

### Immediate Actions
1. ✅ **Beli domain fvchain.org** (atau gunakan subdomain gratis untuk testing)
2. ✅ **Konfigurasi DNS A Record** → 103.245.38.44
3. ✅ **Tunggu propagasi DNS** (1-48 jam)
4. ✅ **Jalankan script Let's Encrypt**
5. ✅ **Verifikasi SSL certificate**

### Alternative Testing
```powershell
# Untuk testing cepat, gunakan subdomain gratis:
# 1. Daftar di duckdns.org
# 2. Buat: fvchain.duckdns.org → 103.245.38.44
# 3. Update script dengan domain baru
# 4. Jalankan Let's Encrypt
```

---

**Status Saat Ini:** SSL self-signed aktif di https://103.245.38.44  
**Target:** SSL valid Let's Encrypt di https://fvchain.org  
**Blocker:** Domain belum terdaftar/dikonfigurasi  
**Action Required:** Registrasi domain + DNS configuration  

**Created by:** Emylton Leunufna  
**Date:** 2025  
**Project:** FVChain Blockchain Layer 1