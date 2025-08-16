#!/bin/bash

# FVChain SSL & Domain Setup Script
# Created: August 12, 2025
# Purpose: Automate SSL certificate installation after DNS propagation

set -e

DOMAIN="www.fvchain.xyz"
SERVER_IP="103.245.38.44"
EMAIL="admin@fvchain.xyz"

echo "=== FVChain SSL & Domain Setup ==="
echo "Domain: $DOMAIN"
echo "Server IP: $SERVER_IP"
echo "Date: $(date)"
echo

# Function to check DNS propagation
check_dns() {
    echo "🔍 Checking DNS propagation..."
    
    # Install DNS tools if not available
    if ! command -v dig &> /dev/null; then
        echo "📦 Installing bind-utils..."
        yum install -y bind-utils
    fi
    
    # Check DNS resolution
    RESOLVED_IP=$(dig +short $DOMAIN @8.8.8.8 | tail -n1)
    
    if [ "$RESOLVED_IP" = "$SERVER_IP" ]; then
        echo "✅ DNS propagation complete: $DOMAIN -> $RESOLVED_IP"
        return 0
    else
        echo "❌ DNS not yet propagated: $DOMAIN -> $RESOLVED_IP (expected: $SERVER_IP)"
        return 1
    fi
}

# Function to install Let's Encrypt certificate
install_letsencrypt() {
    echo "🔒 Installing Let's Encrypt certificate..."
    
    # Stop nginx temporarily
    systemctl stop nginx
    
    # Install certificate
    certbot certonly --standalone \
        --non-interactive \
        --agree-tos \
        --email $EMAIL \
        -d $DOMAIN
    
    if [ $? -eq 0 ]; then
        echo "✅ Let's Encrypt certificate installed successfully"
    else
        echo "❌ Failed to install Let's Encrypt certificate"
        systemctl start nginx
        exit 1
    fi
}

# Function to update nginx configuration
update_nginx_config() {
    echo "⚙️ Updating nginx configuration..."
    
    # Backup current config
    cp /etc/nginx/conf.d/fvc-dashboard-ssl.conf /etc/nginx/conf.d/fvc-dashboard-ssl.conf.backup
    
    # Create new config with Let's Encrypt certificates
    cat > /etc/nginx/conf.d/fvc-dashboard-ssl.conf << EOF
server {
    listen 80;
    server_name $DOMAIN;
    return 301 https://\$server_name\$request_uri;
}

server {
    listen 443 ssl http2;
    server_name $DOMAIN;

    # Let's Encrypt SSL certificates
    ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;
    
    # SSL Security Configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    
    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    
    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/letsencrypt/live/$DOMAIN/chain.pem;
    resolver 8.8.8.8 8.8.4.4 valid=300s;
    resolver_timeout 5s;

    location / {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_cache_bypass \$http_upgrade;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    # API endpoint optimization
    location /api/ {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        
        # Cache for static API responses
        proxy_cache_valid 200 5m;
        add_header X-Cache-Status \$upstream_cache_status;
    }
}
EOF

    # Test nginx configuration
    nginx -t
    
    if [ $? -eq 0 ]; then
        echo "✅ Nginx configuration updated successfully"
    else
        echo "❌ Nginx configuration error, restoring backup"
        cp /etc/nginx/conf.d/fvc-dashboard-ssl.conf.backup /etc/nginx/conf.d/fvc-dashboard-ssl.conf
        exit 1
    fi
}

# Function to setup auto-renewal
setup_auto_renewal() {
    echo "🔄 Setting up auto-renewal..."
    
    # Create renewal script
    cat > /etc/cron.daily/certbot-renewal << 'EOF'
#!/bin/bash
/usr/bin/certbot renew --quiet --post-hook "systemctl reload nginx"
EOF
    
    chmod +x /etc/cron.daily/certbot-renewal
    
    # Test renewal
    certbot renew --dry-run
    
    if [ $? -eq 0 ]; then
        echo "✅ Auto-renewal setup successfully"
    else
        echo "❌ Auto-renewal setup failed"
        exit 1
    fi
}

# Function to verify SSL installation
verify_ssl() {
    echo "🔍 Verifying SSL installation..."
    
    # Start nginx
    systemctl start nginx
    systemctl enable nginx
    
    # Wait for nginx to start
    sleep 5
    
    # Test HTTPS connection
    curl -I --connect-timeout 10 https://$DOMAIN
    
    if [ $? -eq 0 ]; then
        echo "✅ SSL verification successful"
        echo "🎉 Domain is now accessible at: https://$DOMAIN"
    else
        echo "❌ SSL verification failed"
        exit 1
    fi
}

# Main execution
echo "🚀 Starting SSL & Domain setup process..."
echo

# Check if DNS is propagated
if check_dns; then
    echo "✅ DNS propagation confirmed, proceeding with SSL installation..."
    echo
    
    # Install Let's Encrypt certificate
    install_letsencrypt
    echo
    
    # Update nginx configuration
    update_nginx_config
    echo
    
    # Setup auto-renewal
    setup_auto_renewal
    echo
    
    # Verify SSL installation
    verify_ssl
    echo
    
    echo "🎉 SSL & Domain setup completed successfully!"
    echo "📊 Summary:"
    echo "   - Domain: https://$DOMAIN"
    echo "   - SSL Certificate: Let's Encrypt (Valid)"
    echo "   - Auto-renewal: Enabled"
    echo "   - Security Headers: Enabled"
    echo "   - HSTS: Enabled"
    echo
    echo "🔗 Test your site:"
    echo "   - Website: https://$DOMAIN"
    echo "   - API: https://$DOMAIN/api/download/WHITEPAPER_INDONESIA.md"
    echo "   - SSL Test: https://www.ssllabs.com/ssltest/analyze.html?d=$DOMAIN"
    
else
    echo "❌ DNS propagation not yet complete"
    echo "⏰ Please wait for DNS propagation (6-48 hours) and run this script again"
    echo
    echo "📋 To monitor DNS propagation:"
    echo "   - dig +short $DOMAIN @8.8.8.8"
    echo "   - https://dnschecker.org"
    echo "   - https://www.whatsmydns.net"
    echo
    echo "🔄 Run this script again with: bash setup_ssl_domain.sh"
    exit 1
fi