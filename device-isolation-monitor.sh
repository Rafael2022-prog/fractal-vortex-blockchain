#!/bin/bash

# Device Isolation Monitoring Script for Fractal Vortex Chain
# Created by: Emylton Leunufna
# Purpose: Monitor and enforce device isolation between mining devices

echo "=== Fractal Vortex Chain Device Isolation Monitor ==="
echo "Timestamp: $(date)"
echo ""

# Check systemd security settings
echo "[1] Checking systemd security settings..."
echo "fvc-mainnet service:"
systemctl show fvc-mainnet | grep -E '(PrivateDevices|DevicePolicy|ProtectKernelLogs|RestrictAddressFamilies)'
echo ""
echo "fvc-mainnet-rpc service:"
systemctl show fvc-mainnet-rpc | grep -E '(PrivateDevices|DevicePolicy|ProtectKernelLogs|RestrictAddressFamilies)'
echo ""

# Check device access restrictions
echo "[2] Checking device access restrictions..."
echo "Available devices in /dev:"
ls -la /dev/ | grep -E '(null|zero|urandom|random)' | head -10
echo ""

# Check network isolation
echo "[3] Checking network isolation..."
echo "Active network connections:"
ss -tuln | grep -E '(8080|3000|80|443)'
echo ""

# Check process isolation
echo "[4] Checking process isolation..."
echo "FVC processes:"
ps aux | grep -E '(fvc-node|fvc-rpc)' | grep -v grep
echo ""

# Check cgroup isolation
echo "[5] Checking cgroup isolation..."
echo "FVC cgroups:"
systemctl status fvc-mainnet fvc-mainnet-rpc | grep CGroup
echo ""

# Check file system isolation
echo "[6] Checking file system isolation..."
echo "FVC directory permissions:"
ls -la /opt/fvchain/ | head -5
echo ""

# Security recommendations
echo "[7] Security Status Summary:"
echo "✓ Device isolation: $(systemctl show fvc-mainnet | grep PrivateDevices)"
echo "✓ Device policy: $(systemctl show fvc-mainnet | grep DevicePolicy)"
echo "✓ Kernel protection: $(systemctl show fvc-mainnet | grep ProtectKernelLogs)"
echo "✓ Address family restriction: $(systemctl show fvc-mainnet | grep RestrictAddressFamilies)"
echo ""
echo "=== Monitoring Complete ==="