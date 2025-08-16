# FVChain Domain & SSL Setup Guide

## Status Saat Ini ✅

**SSL sudah aktif dan berfungsi:**
- ✅ HTTPS Port 443 aktif
- ✅ SSL Certificate terpasang (self-signed)
- ✅ HTTP to HTTPS redirect berfungsi
- ✅ Nginx konfigurasi valid
- ✅ Dashboard dan API dapat diakses

**Akses sementara:**
- Dashboard: https://103.245.38.44
- API: https://103.245.38.44/api/
- Health Check: https://103.245.38.44/health

## Implementasi Let's Encrypt (Solusi Permanen)

### Langkah 1: Persiapan Domain

**Domain yang dibutuhkan:**
- `fvchain.org` (primary domain)
- `www.fvchain.org` (subdomain)

**Konfigurasi DNS yang diperlukan:**
```
Type: A Record
Name: @
Value: 103.245.38.44
TTL: 3600

Type: A Record  
Name: www
Value: 103.245.38.44
TTL: 3600
```

### Langkah 2: Verifikasi DNS Propagation

**Cara cek DNS propagation:**
1. Online tools:
   - https://dnschecker.org
   - https://whatsmydns.net
   - https://www.nslookup.io

2. Command line:
   ```bash
   nslookup fvchain.org
   dig fvchain.org A
   ```

3. PowerShell:
   ```powershell
   Resolve-DnsName -Name fvchain.org -Type A
   ```

### Langkah 3: Instalasi Let's Encrypt

**Setelah DNS terpropagasi, jalankan:**
```powershell
powershell -ExecutionPolicy Bypass -File "R:\369-FRACTAL\simple-letsencrypt.ps1"
```

**Script akan otomatis:**
1. ✅ Cek DNS propagation
2. ✅ Install Certbot
3. ✅ Generate SSL certificate
4. ✅ Update Nginx configuration
5. ✅ Setup auto-renewal
6. ✅ Restart services

## Timeline Implementasi

### Immediate (Sudah Selesai) ✅
- [x] SSL self-signed certificate
- [x] HTTPS port 443 aktif
- [x] Nginx konfigurasi SSL
- [x] HTTP to HTTPS redirect
- [x] Security headers
- [x] WebSocket support

### Pending (Menunggu DNS)
- [ ] Domain registration/purchase
- [ ] DNS A Record configuration
- [ ] DNS propagation (1-48 hours)
- [ ] Let's Encrypt certificate
- [ ] Auto-renewal setup

## Keamanan & Best Practices

### SSL Configuration ✅
```nginx
# Modern SSL configuration
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
ssl_prefer_server_ciphers off;
ssl_session_cache shared:SSL:10m;
ssl_session_timeout 10m;
```

### Security Headers ✅
```nginx
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
add_header X-Frame-Options DENY always;
add_header X-Content-Type-Options nosniff always;
add_header X-XSS-Protection "1; mode=block" always;
```

### Firewall Rules ✅
```bash
# Ports yang dibuka:
- 22 (SSH)
- 80 (HTTP - redirect to HTTPS)
- 443 (HTTPS)
- 3001 (Dashboard internal)
- 8080 (API internal)
```

## Monitoring & Maintenance

### Certificate Monitoring
```bash
# Check certificate expiry
certbot certificates

# Test auto-renewal
certbot renew --dry-run

# Manual renewal (if needed)
certbot renew --nginx
```

### Nginx Monitoring
```bash
# Check Nginx status
systemctl status nginx

# Test configuration
nginx -t

# Reload configuration
systemctl reload nginx
```

### SSL Testing
```bash
# Test SSL connection
curl -I https://fvchain.org

# Check SSL certificate
openssl s_client -connect fvchain.org:443 -servername fvchain.org
```

## Troubleshooting

### Common Issues

1. **DNS not propagated**
   - Wait 1-48 hours
   - Check with multiple DNS checkers
   - Verify DNS configuration

2. **Certificate generation failed**
   - Ensure port 80 is accessible
   - Check domain ownership
   - Verify DNS resolution

3. **Nginx configuration error**
   - Run `nginx -t` to test
   - Check log files: `/var/log/nginx/error.log`
   - Verify file permissions

### Log Files
```bash
# Nginx logs
tail -f /var/log/nginx/access.log
tail -f /var/log/nginx/error.log

# Certbot logs
tail -f /var/log/letsencrypt/letsencrypt.log

# System logs
journalctl -u nginx -f
```

## Production Checklist

### Pre-Production ✅
- [x] SSL certificate installed
- [x] HTTPS redirect working
- [x] Security headers configured
- [x] Firewall rules applied
- [x] Nginx optimized
- [x] WebSocket support enabled

### Post-DNS Propagation
- [ ] Let's Encrypt certificate
- [ ] Auto-renewal configured
- [ ] Certificate monitoring setup
- [ ] Backup procedures
- [ ] Documentation updated

## Contact & Support

**Created by:** Emylton Leunufna  
**Date:** 2025  
**Project:** FVChain Blockchain Layer 1  
**Environment:** Production Ready  

---

**Note:** FVChain blockchain sudah siap produksi dengan SSL aktif. Let's Encrypt akan menghilangkan peringatan browser setelah DNS terpropagasi.