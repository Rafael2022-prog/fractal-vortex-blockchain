@echo off
echo 🌀 Starting Fractal-Vortex Genesis Network...

for /L %%i in (0,1,3) do (
    echo 🚀 Starting Node %%i...
    cargo run -- start --node-id %%i
    timeout /t 2
)

echo ✅ All nodes started!
echo 📊 Check logs in node-*.log files
pause
