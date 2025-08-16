#!/bin/bash
echo "ðŸ”§ [SERVER] Clearing all server caches..."

# Stop services
sudo systemctl stop nginx
sudo pm2 stop all

# Clear all caches
sudo rm -rf /var/www/fvchain.xyz/.next
sudo rm -rf /var/www/fvchain.xyz/_next
sudo rm -rf /tmp/next-*
sudo rm -rf /var/cache/nginx/*
sudo rm -rf /tmp/nginx-*

# Clear PM2 logs and cache
sudo pm2 flush
sudo pm2 delete all

# Upload new build
echo "ðŸ“¤ [UPLOAD] Uploading new build..."
rsync -avz --delete dashboard/out/ root@fvchain.xyz:/var/www/fvchain.xyz/
rsync -avz target/release/fractal-vortex-chain root@fvchain.xyz:/opt/fvchain/

# Restart services
echo "ðŸ”„ [RESTART] Restarting services..."
sudo systemctl start nginx
sudo pm2 start /opt/fvchain/ecosystem.config.js

echo "âœ… [SUCCESS] Device isolation deployment completed"
