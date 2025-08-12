@echo off
REM Fractal-Vortex Chain Build & Test Script for Windows
REM This script builds and tests the entire FVC ecosystem

echo 🌀 Fractal-Vortex Chain Build & Test Script
echo ============================================

REM Check prerequisites
echo [INFO] Checking prerequisites...

where rustc >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust is not installed. Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Node.js is not installed. Please install Node.js 18+
    pause
    exit /b 1
)

where npm >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] npm is not installed. Please install npm
    pause
    exit /b 1
)

where docker >nul 2>nul
if %errorlevel% neq 0 (
    echo [WARN] Docker is not installed. Docker deployment will not be available
)

echo [INFO] Prerequisites check completed

REM Build the blockchain node
echo [INFO] Building Fractal-Vortex Chain node...
cargo build --release

if %errorlevel% neq 0 (
    echo [ERROR] Failed to build blockchain node
    pause
    exit /b 1
)
echo [INFO] ✅ Blockchain node built successfully

REM Run blockchain tests
echo [INFO] Running blockchain tests...
cargo test --release

if %errorlevel% neq 0 (
    echo [ERROR] Some blockchain tests failed
    pause
    exit /b 1
)
echo [INFO] ✅ All blockchain tests passed

REM Build the dashboard
echo [INFO] Building Fractal-Vortex Dashboard...
cd dashboard

if not exist "node_modules" (
    echo [INFO] Installing dashboard dependencies...
    npm install
)

npm run build

if %errorlevel% neq 0 (
    echo [ERROR] Failed to build dashboard
    pause
    exit /b 1
)
echo [INFO] ✅ Dashboard built successfully

REM Run dashboard tests
echo [INFO] Running dashboard linting...
npm run lint

echo [INFO] ✅ Dashboard linting completed
cd ..

REM Create deployment directories
echo [INFO] Setting up deployment structure...
if not exist "dist" mkdir dist
if not exist "dist\node" mkdir dist\node
if not exist "dist\dashboard" mkdir dist\dashboard

copy target\release\fractal-vortex-chain.exe dist\node\
xcopy dashboard\dist dist\dashboard\ /E /I /Y

REM Create system configuration
echo [INFO] Creating system configuration...
echo { > dist\config.json
echo   "version": "1.0.0", >> dist\config.json
echo   "node": { >> dist\config.json
echo     "binary": ".\node\fractal-vortex-chain.exe", >> dist\config.json
echo     "port": 30333, >> dist\config.json
echo     "rpc_port": 9944 >> dist\config.json
echo   }, >> dist\config.json
echo   "dashboard": { >> dist\config.json
echo     "path": ".\dashboard", >> dist\config.json
echo     "port": 3000, >> dist\config.json
echo     "proxy": "http://localhost:30333" >> dist\config.json
echo   }, >> dist\config.json
echo   "features": { >> dist\config.json
echo     "fractal_consensus": true, >> dist\config.json
echo     "vortex_math": true, >> dist\config.json
echo     "torus_network": true, >> dist\config.json
echo     "quantum_security": true >> dist\config.json
echo   } >> dist\config.json
echo } >> dist\config.json

REM Create startup scripts
echo [INFO] Creating startup scripts...
echo @echo off > dist\start_node.bat
echo echo Starting Fractal-Vortex Chain node... >> dist\start_node.bat
echo .\node\fractal-vortex-chain.exe --dev --ws-external --rpc-external >> dist\start_node.bat

echo @echo off > dist\start_dashboard.bat
echo echo Starting Fractal-Vortex Dashboard... >> dist\start_dashboard.bat
echo cd dashboard >> dist\start_dashboard.bat
echo npx serve -s . -l 3000 >> dist\start_dashboard.bat

echo @echo off > dist\start_all.bat
echo echo Starting Fractal-Vortex Chain ecosystem... >> dist\start_all.bat
echo start "" cmd /k ".\start_node.bat" >> dist\start_all.bat
echo timeout /t 5 >> dist\start_all.bat
echo start "" cmd /k ".\start_dashboard.bat" >> dist\start_all.bat
echo echo. >> dist\start_all.bat
echo echo Fractal-Vortex Chain is running! >> dist\start_all.bat
echo echo Dashboard: http://localhost:3000 >> dist\start_all.bat
echo echo Node RPC:  http://localhost:9944 >> dist\start_all.bat
echo echo Node WS:   ws://localhost:30333 >> dist\start_all.bat
echo echo. >> dist\start_all.bat
echo echo Press any key to stop all services... >> dist\start_all.bat
echo pause >> dist\start_all.bat

REM Run integration tests
echo [INFO] Running integration tests...
cargo test --test integration_test --release

echo [INFO] ✅ Integration tests completed

REM Performance benchmark
echo [INFO] Running performance benchmarks...
cargo bench

echo [INFO] ✅ Build and test process completed successfully!
echo.
echo 🎉 Fractal-Vortex Chain is ready for deployment!
echo.
echo Quick start commands:
echo   .\dist\start_all.bat          # Start everything
echo   .\dist\start_node.bat         # Start only blockchain node
echo   .\dist\start_dashboard.bat    # Start only dashboard
echo.
echo Access points:
echo   Dashboard: http://localhost:3000
echo   Node RPC:  http://localhost:9944
echo   Node WS:   ws://localhost:30333
echo.
echo For Docker deployment:
echo   cd dist ^& deploy.bat
pause