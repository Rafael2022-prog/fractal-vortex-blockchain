#!/bin/bash

# SSL Verification Script for fvchain.xyz
# This script ensures the PDF-like display issue won't occur

set -e

echo "🔍 Verifying SSL Configuration and Display Issues..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
    fi
}

# 1. Check SSL Certificate Validity
echo "1. Checking SSL Certificate Validity..."
if openssl x509 -in /etc/letsencrypt/live/fvchain.xyz/fullchain.pem -text -noout | grep -q "Let's Encrypt"; then
    print_status 0 "SSL Certificate is valid and issued by Let's Encrypt"
else
    print_status 1 "SSL Certificate may not be valid"
fi

# 2. Check Certificate Expiration
echo "2. Checking Certificate Expiration..."
exp_date=$(openssl x509 -in /etc/letsencrypt/live/fvchain.xyz/fullchain.pem -enddate -noout | cut -d= -f2)
exp_timestamp=$(date -d "$exp_date" +%s)
current_timestamp=$(date +%s)
days_until_exp=$(( (exp_timestamp - current_timestamp) / 86400 ))

if [ $days_until_exp -gt 30 ]; then
    print_status 0 "Certificate expires in $days_until_exp days"
else
    print_status 1 "Certificate expires in $days_until_exp days - consider renewal"
fi

# 3. Test HTTPS Connectivity
echo "3. Testing HTTPS Connectivity..."
if curl -I -s https://fvchain.xyz | grep -q "HTTP/2 200"; then
    print_status 0 "HTTPS connectivity working"
else
    print_status 1 "HTTPS connectivity issue"
fi

# 4. Check for Mixed Content
echo "4. Checking for Mixed Content Issues..."
if curl -s https://fvchain.xyz | grep -q "http://"; then
    print_status 1 "Mixed content detected (HTTP resources on HTTPS page)"
else
    print_status 0 "No mixed content detected"
fi

# 5. Verify Content-Type Headers
echo "5. Verifying Content-Type Headers..."
css_content_type=$(curl -I -s https://fvchain.xyz/_next/static/css/ | grep -i "content-type" | cut -d: -f2 | tr -d ' ')
if [[ "$css_content_type" == *"text/css"* ]]; then
    print_status 0 "CSS files have correct Content-Type"
else
    print_status 1 "CSS files may have incorrect Content-Type: $css_content_type"
fi

js_content_type=$(curl -I -s https://fvchain.xyz/_next/static/chunks/ | grep -i "content-type" | cut -d: -f2 | tr -d ' ')
if [[ "$js_content_type" == *"application/javascript"* ]]; then
    print_status 0 "JavaScript files have correct Content-Type"
else
    print_status 1 "JavaScript files may have incorrect Content-Type: $js_content_type"
fi

# 6. Check Security Headers
echo "6. Checking Security Headers..."
if curl -I -s https://fvchain.xyz | grep -q "X-Content-Type-Options: nosniff"; then
    print_status 0 "Security headers properly configured"
else
    print_status 1 "Security headers may be missing"
fi

# 7. Test SSL Grade
echo "7. Testing SSL Configuration Grade..."
if command -v nmap &> /dev/null; then
    ssl_grade=$(nmap --script ssl-enum-ciphers -p 443 fvchain.xyz | grep -o "TLS.*" | head -1)
    if [[ "$ssl_grade" == *"TLSv1.3"* ]] || [[ "$ssl_grade" == *"TLSv1.2"* ]]; then
        print_status 0 "SSL configuration uses modern protocols"
    else
        print_status 1 "SSL configuration may need improvement"
    fi
else
    echo -e "${YELLOW}⚠️  nmap not available, skipping SSL grade test${NC}"
fi

# 8. Verify Domain Redirects
echo "8. Verifying Domain Redirects..."
http_response=$(curl -I -s http://fvchain.xyz | grep -i "location" | grep -o "https://.*")
if [[ "$http_response" == *"https://fvchain.xyz"* ]]; then
    print_status 0 "HTTP to HTTPS redirect working"
else
    print_status 1 "HTTP to HTTPS redirect may not be configured"
fi

# 9. Check Next.js Build Configuration
echo "9. Checking Next.js Build Configuration..."
if [ -f "/opt/fvc-dashboard/package.json" ]; then
    if grep -q "https://fvchain.xyz" /opt/fvc-dashboard/package.json || grep -q "https://fvchain.xyz" /opt/fvc-dashboard/.env* 2>/dev/null; then
        print_status 0 "Next.js configured for HTTPS domain"
    else
        print_status 1 "Next.js may not be configured for HTTPS domain"
    fi
else
    echo -e "${YELLOW}⚠️  Next.js directory not found, skipping build config check${NC}"
fi

# 10. Test Browser Compatibility
echo "10. Testing Browser Compatibility..."
if curl -s -A "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36" https://fvchain.xyz | grep -q "<!DOCTYPE html>"; then
    print_status 0 "Website serves valid HTML to browsers"
else
    print_status 1 "Website may not serve proper HTML"
fi

echo ""
echo "📋 Summary:"
echo "   If all checks show ✅, your SSL configuration is correct and"
echo "   the PDF-like display issue should not occur."
echo ""
echo "🛠️  If any ❌ appear, run the following commands:"
echo "   1. sudo bash setup-ssl-letsencrypt.sh"
echo "   2. npm run build (in Next.js directory)"
echo "   3. sudo systemctl restart nginx"
echo ""
echo "🌐 Test your site: https://fvchain.xyz"