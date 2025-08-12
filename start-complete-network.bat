@echo off
echo 🌀 Starting Complete Fractal-Vortex Network...
echo.

REM Start RPC Server
echo 🌐 Starting RPC Server...
start cmd /k "title FVC RPC Server && cargo run --bin fvc-rpc"


REM Start Node 0 (Genesis)
echo 🚀 Starting Genesis Node 0...
start cmd /k "title FVC Node 0 && cargo run --bin fvc-node -- start --node-id 0"


REM Start Node 1
echo 🚀 Starting Node 1...
start cmd /k "title FVC Node 1 && cargo run --bin fvc-node -- start --node-id 1"


REM Start Node 2
echo 🚀 Starting Node 2...
start cmd /k "title FVC Node 2 && cargo run --bin fvc-node -- start --node-id 2"


REM Start Node 3
echo 🚀 Starting Node 3...
start cmd /k "title FVC Node 3 && cargo run --bin fvc-node -- start --node-id 3"
timeout /t 3

echo ✅ All network components started!
echo 📊 RPC Server: http://localhost:8080
echo 🔗 WebSocket Node 0: ws://localhost:30333
echo 🔗 WebSocket Node 1: ws://localhost:30334
echo 🔗 WebSocket Node 2: ws://localhost:30335
echo 🔗 WebSocket Node 3: ws://localhost:30336
echo 🌐 Dashboard: http://localhost:3000
echo.