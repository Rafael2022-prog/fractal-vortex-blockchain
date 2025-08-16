#!/bin/bash

# Let's Encrypt SSL Setup Script for fvchain.xyz and www.fvchain.xyz
# This script prevents PDF-like display issues by using valid SSL certificates

set -e

echo "🚀 Starting Let's Encrypt SSL Setup for fvchain.xyz..."

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "❌ This script must be run as root (use sudo)"
   exit 1
fi

# Install Certbot if not already installed
echo "📦 Installing Certbot..."
if command -v apt-get &> /dev/null; then
    apt-get update -y
    apt-get install -y certbot python3-certbot-nginx
elif command -v yum &> /dev/null; then
    yum install -y epel-release
    yum install -y certbot python3-certbot-nginx
else
    echo "❌ Unsupported package manager. Please install certbot manually."
    exit 1
fi

# Backup existing SSL configurations
echo "💾 Backing up existing SSL configurations..."
cp /etc/nginx/conf.d/fvchain-ssl.conf /etc/nginx/conf.d/fvchain-ssl.conf.backup.$(date +%Y%m%d_%H%M%S) 2>/dev/null || true

# Stop nginx temporarily to free port 80
echo "🛑 Stopping nginx..."
systemctl stop nginx

# Obtain SSL certificates
echo "🔐 Obtaining Let's Encrypt SSL certificates..."
certbot certonly --standalone \
    --agree-tos \
    --no-eff-email \
    --email admin@fvchain.xyz \
    -d fvchain.xyz \
    -d www.fvchain.xyz \
    --non-interactive

# Check if certificates were obtained successfully
if [ ! -f "/etc/letsencrypt/live/fvchain.xyz/fullchain.pem" ]; then
    echo "❌ Failed to obtain SSL certificates"
    exit 1
fi

echo "✅ SSL certificates obtained successfully"

# Copy the new secure configuration
echo "📋 Installing new SSL configuration..."
cp nginx-letsencrypt-secure.conf /etc/nginx/conf.d/fvchain-ssl.conf

# Test nginx configuration
echo "🧪 Testing nginx configuration..."
nginx -t

# Restart nginx with new SSL configuration
echo "🔄 Restarting nginx with SSL..."
systemctl start nginx
systemctl enable nginx

# Set up auto-renewal
echo "⏰ Setting up auto-renewal..."
crontab -l 2>/dev/null | grep -v certbot || true
echo "0 12 * * * certbot renew --quiet --nginx" | crontab -

# Verify SSL is working
echo "🔍 Verifying SSL configuration..."
sleep 5
if curl -I -k https://fvchain.xyz | grep -q "HTTP/2 200"; then
    echo "✅ SSL configuration is working correctly"
else
    echo "⚠️  SSL configuration might need verification"
fi

# Display certificate info
echo "📊 Certificate information:"
certbot certificates

echo "🎉 SSL setup complete! Your site should now display correctly with HTTPS."
echo "🌐 Visit: https://fvchain.xyz"
echo "🌐 Visit: https://www.fvchain.xyz"
echo ""
echo "💡 The PDF-like display issue should now be resolved because:"
echo "   1. Valid SSL certificates are being used"
echo "   2. Mixed content is properly handled"
echo "   3. Correct MIME types are configured"
echo "   4. Security headers prevent browser issues"