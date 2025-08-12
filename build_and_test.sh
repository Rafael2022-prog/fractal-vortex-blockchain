#!/bin/bash

# Fractal-Vortex Chain Build & Test Script
# This script builds and tests the entire FVC ecosystem

set -e

echo "🌀 Fractal-Vortex Chain Build & Test Script"
echo "============================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
print_status "Checking prerequisites..."

if ! command -v rustc &> /dev/null; then
    print_error "Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

if ! command -v node &> /dev/null; then
    print_error "Node.js is not installed. Please install Node.js 18+"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    print_error "npm is not installed. Please install npm"
    exit 1
fi

if ! command -v docker &> /dev/null; then
    print_warning "Docker is not installed. Docker deployment will not be available"
fi

print_status "Prerequisites check completed"

# Build the blockchain node
print_status "Building Fractal-Vortex Chain node..."
cargo build --release

if [ $? -eq 0 ]; then
    print_status "✅ Blockchain node built successfully"
else
    print_error "❌ Failed to build blockchain node"
    exit 1
fi

# Run blockchain tests
print_status "Running blockchain tests..."
cargo test --release

if [ $? -eq 0 ]; then
    print_status "✅ All blockchain tests passed"
else
    print_error "❌ Some blockchain tests failed"
    exit 1
fi

# Build the dashboard
print_status "Building Fractal-Vortex Dashboard..."
cd dashboard

if [ ! -d "node_modules" ]; then
    print_status "Installing dashboard dependencies..."
    npm install
fi

npm run build

if [ $? -eq 0 ]; then
    print_status "✅ Dashboard built successfully"
else
    print_error "❌ Failed to build dashboard"
    exit 1
fi

# Run dashboard tests
print_status "Running dashboard linting..."
npm run lint

if [ $? -eq 0 ]; then
    print_status "✅ Dashboard linting passed"
else
    print_warning "⚠️ Dashboard has linting warnings"
fi

cd ..

# Create deployment directories
print_status "Setting up deployment structure..."
mkdir -p dist/{node,dashboard}
cp target/release/fractal-vortex-chain dist/node/
cp -r dashboard/dist/* dist/dashboard/

# Create system configuration
print_status "Creating system configuration..."
cat > dist/config.json << EOF
{
  "version": "1.0.0",
  "node": {
    "binary": "./node/fractal-vortex-chain",
    "port": 30333,
    "rpc_port": 9944
  },
  "dashboard": {
    "path": "./dashboard",
    "port": 3000,
    "proxy": "http://localhost:30333"
  },
  "features": {
    "fractal_consensus": true,
    "vortex_math": true,
    "torus_network": true,
    "quantum_security": true
  }
}
EOF

# Create startup scripts
print_status "Creating startup scripts..."
cat > dist/start_node.sh << 'EOF'
#!/bin/bash
echo "Starting Fractal-Vortex Chain node..."
./node/fractal-vortex-chain --dev --ws-external --rpc-external
EOF

cat > dist/start_dashboard.sh << 'EOF'
#!/bin/bash
echo "Starting Fractal-Vortex Dashboard..."
cd dashboard
npx serve -s . -l 3000
EOF

cat > dist/start_all.sh << 'EOF'
#!/bin/bash
echo "Starting Fractal-Vortex Chain ecosystem..."

# Start node in background
./start_node.sh &
NODE_PID=$!

# Wait for node to start
sleep 5

# Start dashboard
./start_dashboard.sh &
DASHBOARD_PID=$!

echo "Node PID: $NODE_PID"
echo "Dashboard PID: $DASHBOARD_PID"
echo ""
echo "Fractal-Vortex Chain is running!"
echo "Dashboard: http://localhost:3000"
echo "Node RPC: http://localhost:9944"
echo "Node WS: ws://localhost:30333"
echo ""
echo "Press Ctrl+C to stop all services"

# Trap to kill processes on exit
trap "kill $NODE_PID $DASHBOARD_PID 2>/dev/null" EXIT

# Wait for both processes
wait
EOF

chmod +x dist/*.sh

# Create Docker setup if Docker is available
if command -v docker &> /dev/null; then
    print_status "Creating Docker deployment..."
    cp Dockerfile dist/
    cp docker-compose.yml dist/
    cp deploy.sh dist/
    
    print_status "✅ Docker deployment files ready"
fi

# Run integration tests
print_status "Running integration tests..."
cargo test --test integration_test --release

if [ $? -eq 0 ]; then
    print_status "✅ Integration tests passed"
else
    print_warning "⚠️ Some integration tests failed"
fi

# Performance benchmark
print_status "Running performance benchmarks..."
cargo bench

print_status "✅ Build and test process completed successfully!"
echo ""
echo "🎉 Fractal-Vortex Chain is ready for deployment!"
echo ""
echo "Quick start commands:"
echo "  ./dist/start_all.sh          # Start everything"
echo "  ./dist/start_node.sh         # Start only blockchain node"
echo "  ./dist/start_dashboard.sh    # Start only dashboard"
echo ""
echo "Access points:"
echo "  Dashboard: http://localhost:3000"
echo "  Node RPC:  http://localhost:9944"
echo "  Node WS:   ws://localhost:30333"
echo ""
echo "For Docker deployment:"
echo "  cd dist && ./deploy.sh"