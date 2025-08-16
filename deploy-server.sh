#!/bin/bash
echo "=== FVChain Cloud Deployment ==="
echo "Stopping services..."
systemctl stop fvc-rpc 2>/dev/null || true

echo "Updating RPC binary..."
cp /tmp/fvc-rpc /opt/fvchain/fvc-rpc
chmod +x /opt/fvchain/fvc-rpc

echo "Updating nginx config..."
cp /tmp/fvchain-nginx.conf /etc/nginx/conf.d/fvchain.conf

echo "Testing nginx config..."
nginx -t
if [ $? -eq 0 ]; then
    echo "Nginx config valid"
    systemctl reload nginx
else
    echo "Nginx config invalid"
    exit 1
fi

echo "Starting RPC server..."
systemctl start fvc-rpc
systemctl enable fvc-rpc

echo "Checking service status..."
systemctl status fvc-rpc --no-pager -l

echo "=== Deployment Complete ==="
echo "Test: curl -X GET https://fvchain.xyz/api/mining/miner/status?device_id=test"
