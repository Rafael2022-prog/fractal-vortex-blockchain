@echo off
REM FVChain Mainnet Startup Script

echo 🌟 Starting FVChain Mainnet Node...
echo ===================================

REM Start mainnet node
.\target\release\fvc-node.exe start --node-id 0
pause
