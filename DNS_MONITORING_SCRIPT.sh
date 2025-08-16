#!/bin/bash

# DNS Propagation Monitoring Script
# Created: August 12, 2025
# Purpose: Monitor DNS propagation status for www.fvchain.xyz

DOMAIN="www.fvchain.xyz"
EXPECTED_IP="103.245.38.44"
LOG_FILE="/var/log/dns_monitoring.log"

echo "=== DNS Propagation Monitor ==="
echo "Domain: $DOMAIN"
echo "Expected IP: $EXPECTED_IP"
echo "Started: $(date)"
echo

# Function to log with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a $LOG_FILE
}

# Function to check DNS from multiple servers
check_dns_propagation() {
    log_message "=== DNS Propagation Check ==="
    
    # Install DNS tools if needed
    if ! command -v dig &> /dev/null; then
        log_message "Installing bind-utils..."
        yum install -y bind-utils >> $LOG_FILE 2>&1
    fi
    
    # DNS servers to check
    declare -A dns_servers=(
        ["Google Primary"]="8.8.8.8"
        ["Google Secondary"]="8.8.4.4"
        ["Cloudflare Primary"]="1.1.1.1"
        ["Cloudflare Secondary"]="1.0.0.1"
        ["OpenDNS Primary"]="208.67.222.222"
        ["OpenDNS Secondary"]="208.67.220.220"
        ["Quad9"]="9.9.9.9"
    )
    
    propagated_count=0
    total_servers=${#dns_servers[@]}
    
    for server_name in "${!dns_servers[@]}"; do
        dns_ip="${dns_servers[$server_name]}"
        
        # Query DNS server
        resolved_ip=$(dig +short $DOMAIN @$dns_ip | tail -n1 2>/dev/null)
        
        if [ "$resolved_ip" = "$EXPECTED_IP" ]; then
            log_message "✅ $server_name ($dns_ip): $DOMAIN -> $resolved_ip"
            ((propagated_count++))
        elif [ -z "$resolved_ip" ]; then
            log_message "❌ $server_name ($dns_ip): No response"
        else
            log_message "⚠️  $server_name ($dns_ip): $DOMAIN -> $resolved_ip (expected: $EXPECTED_IP)"
        fi
    done
    
    # Calculate propagation percentage
    propagation_percentage=$((propagated_count * 100 / total_servers))
    
    log_message "📊 Propagation Status: $propagated_count/$total_servers servers ($propagation_percentage%)"
    
    if [ $propagated_count -eq $total_servers ]; then
        log_message "🎉 DNS FULLY PROPAGATED! Ready for SSL installation."
        return 0
    elif [ $propagated_count -gt 0 ]; then
        log_message "🟡 DNS PARTIALLY PROPAGATED. Wait for full propagation."
        return 1
    else
        log_message "🔴 DNS NOT PROPAGATED. Check domain configuration."
        return 2
    fi
}

# Function to test domain accessibility
test_domain_access() {
    log_message "=== Domain Accessibility Test ==="
    
    # Test HTTP
    if curl -I --connect-timeout 10 http://$DOMAIN &>/dev/null; then
        log_message "✅ HTTP accessible: http://$DOMAIN"
    else
        log_message "❌ HTTP not accessible: http://$DOMAIN"
    fi
    
    # Test HTTPS (will fail until proper SSL is installed)
    if curl -I --connect-timeout 10 -k https://$DOMAIN &>/dev/null; then
        log_message "✅ HTTPS accessible: https://$DOMAIN"
    else
        log_message "❌ HTTPS not accessible: https://$DOMAIN"
    fi
}

# Function to check server status
check_server_status() {
    log_message "=== Server Status Check ==="
    
    # Check nginx
    if systemctl is-active nginx &>/dev/null; then
        log_message "✅ Nginx: Running"
    else
        log_message "❌ Nginx: Not running"
    fi
    
    # Check dashboard
    if pgrep -f "next-server" &>/dev/null; then
        log_message "✅ Dashboard: Running on port 3001"
    else
        log_message "❌ Dashboard: Not running"
    fi
    
    # Check port 443
    if netstat -tlnp | grep ":443" &>/dev/null; then
        log_message "✅ Port 443: Listening"
    else
        log_message "❌ Port 443: Not listening"
    fi
    
    # Check SSL certificate
    if [ -f "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" ]; then
        cert_expiry=$(openssl x509 -enddate -noout -in "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" 2>/dev/null | cut -d= -f2)
        log_message "✅ Let's Encrypt SSL: Valid until $cert_expiry"
    elif [ -f "/etc/ssl/certs/fvchain.crt" ]; then
        log_message "⚠️  Self-signed SSL: Active (needs upgrade to Let's Encrypt)"
    else
        log_message "❌ SSL Certificate: Not found"
    fi
}

# Function to provide recommendations
provide_recommendations() {
    log_message "=== Recommendations ==="
    
    if [ $propagated_count -eq $total_servers ]; then
        log_message "🚀 READY: Run 'bash /root/setup_ssl_domain.sh' to install Let's Encrypt SSL"
    elif [ $propagated_count -gt 0 ]; then
        log_message "⏳ WAIT: DNS partially propagated. Check again in 2-4 hours."
    else
        log_message "🔧 ACTION NEEDED: Check domain DNS configuration at your registrar."
        log_message "   Required DNS record: A record 'www' pointing to $EXPECTED_IP"
    fi
    
    log_message "📱 Monitor propagation: https://dnschecker.org/?domain=$DOMAIN&type=A"
    log_message "🔄 Run this script again: bash /root/dns_monitor.sh"
}

# Main execution
log_message "Starting DNS monitoring for $DOMAIN"

# Check DNS propagation
check_dns_propagation
propagation_result=$?

echo

# Test domain accessibility
test_domain_access

echo

# Check server status
check_server_status

echo

# Provide recommendations
provide_recommendations

echo
log_message "DNS monitoring completed. Check $LOG_FILE for detailed logs."

# Exit with propagation status
exit $propagation_result