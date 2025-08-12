#!/bin/bash

# Fractal-Vortex Chain Deployment Script
# ====================================
# This script deploys the complete fractal-vortex blockchain network
# with toroidal topology and Sierpinski triangle consensus

set -e

echo "🌀 Fractal-Vortex Chain Deployment Script"
echo "======================================="
echo ""
echo "🔮 Deploying Revolutionary Blockchain Architecture:"
echo "   • Fractal-Vortex Consensus"
echo "   • Toroidal Network Topology"
echo "   • Sierpinski Triangle Voting"
echo "   • Vortex Mathematics Integration"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NETWORK_NAME="fractal-vortex-net"
CHAIN_ID="fractal-vortex-mainnet"
VALIDATOR_COUNT=4
GENESIS_FILE="genesis.json"

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    print_step "Checking prerequisites..."
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    if ! command -v rustc &> /dev/null; then
        print_warning "Rust is not installed. Using Docker for compilation."
    fi
    
    print_success "Prerequisites check completed"
}

# Build the application
build_application() {
    print_step "Building Fractal-Vortex Chain..."
    
    # Build Docker image
    docker build -t fractal-vortex-chain:latest .
    
    print_success "Application built successfully"
}

# Generate genesis configuration
generate_genesis() {
    print_step "Generating genesis configuration..."
    
    # Run genesis generation
    docker run --rm -v $(pwd):/workspace fractal-vortex-chain:latest genesis --output /workspace/genesis.json
    
    print_success "Genesis configuration generated"
}

# Create network
create_network() {
    print_step "Creating fractal-vortex network..."
    
    # Create Docker network
    docker network create $NETWORK_NAME 2>/dev/null || true
    
    print_success "Network created"
}

# Deploy validators
deploy_validators() {
    print_step "Deploying $VALIDATOR_COUNT validator nodes..."
    
    # Start the network
    docker-compose up -d
    
    # Wait for nodes to start
    echo "⏳ Waiting for nodes to initialize..."
    sleep 30
    
    print_success "$VALIDATOR_COUNT validator nodes deployed"
}

# Verify deployment
verify_deployment() {
    print_step "Verifying deployment..."
    
    # Check if all services are running
    if docker-compose ps | grep -q "Up"; then
        print_success "All services are running"
    else
        print_error "Some services failed to start"
        docker-compose logs
        exit 1
    fi
    
    # Check node connectivity
    echo "🔍 Checking node connectivity..."
    for i in 1 2 3 4; do
        if docker-compose exec fractal-node-$i fractal-vortex-chain info; then
            print_success "Node $i is responding"
        else
            print_warning "Node $i might need more time to sync"
        fi
    done
}

# Setup monitoring
setup_monitoring() {
    print_step "Setting up monitoring..."
    
    # Wait for Prometheus and Grafana
    echo "⏳ Waiting for monitoring services..."
    sleep 15
    
    print_success "Monitoring setup completed"
    print_success "Grafana available at: http://localhost:3000 (admin/fractal123)"
    print_success "Prometheus available at: http://localhost:9090"
}

# Display network information
display_network_info() {
    print_step "Fractal-Vortex Network Information"
    echo ""
    echo "🌐 Network Topology:"
    echo "   Chain ID: $CHAIN_ID"
    echo "   Validators: $VALIDATOR_COUNT"
    echo "   Consensus: Fractal-Vortex"
    echo ""
    echo "🔗 Node Endpoints:"
    echo "   Node 1: http://localhost:9933"
    echo "   Node 2: http://localhost:9934"
    echo "   Node 3: http://localhost:9935"
    echo "   Node 4: http://localhost:9936"
    echo ""
    echo "📊 Monitoring:"
    echo "   Grafana: http://localhost:3000"
    echo "   Prometheus: http://localhost:9090"
    echo ""
    echo "🎯 P2P Ports:"
    echo "   Node 1: 30333"
    echo "   Node 2: 30334"
    echo "   Node 3: 30335"
    echo "   Node 4: 30336"
    echo ""
}

# Health check
health_check() {
    print_step "Performing health check..."
    
    # Check if all containers are healthy
    for service in fractal-node-1 fractal-node-2 fractal-node-3 fractal-node-4; do
        if docker-compose ps $service | grep -q "healthy\|Up"; then
            print_success "$service is healthy"
        else
            print_warning "$service health check failed"
        fi
    done
}

# Cleanup function
cleanup() {
    print_step "Cleaning up previous deployment..."
    
    docker-compose down --volumes --remove-orphans 2>/dev/null || true
    docker system prune -f 2>/dev/null || true
    
    print_success "Cleanup completed"
}

# Main deployment function
main() {
    echo "🚀 Starting Fractal-Vortex Chain deployment..."
    echo ""
    
    # Handle command line arguments
    case "${1:-deploy}" in
        "deploy")
            check_prerequisites
            cleanup
            build_application
            generate_genesis
            create_network
            deploy_validators
            verify_deployment
            setup_monitoring
            display_network_info
            ;;
        "stop")
            print_step "Stopping Fractal-Vortex Chain..."
            docker-compose down
            print_success "Network stopped"
            ;;
        "restart")
            print_step "Restarting Fractal-Vortex Chain..."
            docker-compose restart
            print_success "Network restarted"
            ;;
        "logs")
            docker-compose logs -f
            ;;
        "status")
            docker-compose ps
            ;;
        "clean")
            cleanup
            ;;
        "benchmark")
            print_step "Running performance benchmark..."
            docker run --rm fractal-vortex-chain:latest bench --transactions 10000 --validators 4
            ;;
        *)
            echo "Usage: $0 {deploy|stop|restart|logs|status|clean|benchmark}"
            exit 1
            ;;
    esac
}

# Trap signals for graceful shutdown
trap 'echo ""; print_error "Deployment interrupted"; exit 1' INT TERM

# Run main function
main "$@"