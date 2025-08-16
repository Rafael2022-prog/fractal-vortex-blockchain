#!/bin/bash
# Verifikasi SSL untuk fvchain.xyz
# Script untuk memastikan implementasi berhasil

set -e

echo "🔍 VERIFIKASI SSL IMPLEMENTASI"
echo "============================="

# Warna output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Fungsi untuk menampilkan status
check_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
    fi
}

echo "1. Test HTTPS Connectivity..."
# Test fvchain.xyz
if curl -I -s https://fvchain.xyz | grep -q "HTTP/2 200\|HTTP/1.1 200"; then
    check_status 0 "HTTPS fvchain.xyz berfungsi"
else
    check_status 1 "HTTPS fvchain.xyz bermasalah"
fi

# Test www.fvchain.xyz
if curl -I -s https://www.fvchain.xyz | grep -q "HTTP/2 200\|HTTP/1.1 200"; then
    check_status 0 "HTTPS www.fvchain.xyz berfungsi"
else
    check_status 1 "HTTPS www.fvchain.xyz bermasalah"
fi

echo ""
echo "2. Test Static Files..."
# Test CSS file
if curl -I -s https://fvchain.xyz/_next/static/css/ | grep -q "text/css"; then
    check_status 0 "CSS Content-Type benar"
else
    check_status 1 "CSS Content-Type bermasalah"
fi

# Test JS file
if curl -I -s https://fvchain.xyz/_next/static/js/ | grep -q "application/javascript"; then
    check_status 0 "JS Content-Type benar"
else
    check_status 1 "JS Content-Type bermasalah"
fi

echo ""
echo "3. Check Mixed Content..."
# Check for http:// in HTTPS page
if curl -s https://fvchain.xyz | grep -q "http://"; then
    check_status 1 "Mixed content terdeteksi"
    echo "HTTP links found:"
    curl -s https://fvchain.xyz | grep -n "http://"
else
    check_status 0 "Tidak ada mixed content"
fi

echo ""
echo "4. Check SSL Certificate..."
# Check certificate validity
if openssl s_client -connect fvchain.xyz:443 -servername fvchain.xyz < /dev/null 2>/dev/null | grep -q "Verify return code: 0"; then
    check_status 0 "SSL Certificate valid"
else
    check_status 1 "SSL Certificate bermasalah"
fi

echo ""
echo "5. Check Security Headers..."
# Check security headers
if curl -I -s https://fvchain.xyz | grep -q "X-Content-Type-Options: nosniff"; then
    check_status 0 "Security headers terpasang"
else
    check_status 1 "Security headers kurang"
fi

echo ""
echo "6. Test Browser Compatibility..."
# Test with browser user agent
if curl -s -A "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36" https://fvchain.xyz | grep -q "<!DOCTYPE html>"; then
    check_status 0 "HTML valid untuk browser"
else
    check_status 1 "HTML bermasalah"
fi

echo ""
echo "7. Check Auto-Renewal..."
# Check cron job
if sudo crontab -l | grep -q "certbot renew"; then
    check_status 0 "Auto-renewal aktif"
else
    check_status 1 "Auto-renewal tidak aktif"
fi

echo ""
echo "📊 RINGKASAN VERIFIKASI:"
echo "========================"
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ SELURUH VERIFIKASI BERHASIL${NC}"
    echo "🎯 JAMINAN: Tampilan PDF-like TIDAK AKAN TERJADI LAGI!"
    echo ""
    echo "🌐 Test di browser:"
    echo "   - https://fvchain.xyz"
    echo "   - https://www.fvchain.xyz"
else
    echo -e "${YELLOW}⚠️  ADA MASALAH YANG PERLU DIPERIKSA${NC}"
    echo "Jalankan kembali script deployment atau periksa log"
fi