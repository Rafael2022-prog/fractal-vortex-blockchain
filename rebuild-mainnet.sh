#!/bin/bash
# FVChain Mainnet Rebuild Script for Linux/macOS
# Updated with new difficulty configuration: Initial difficulty 10, 2023-block adjustment

set -e

# Configuration
INSTALL_PATH="${1:-/opt/fvchain}"
SKIP_BUILD="${SKIP_BUILD:-false}"
SKIP_BACKUP="${SKIP_BACKUP:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}FVChain Mainnet Rebuild Script${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "${YELLOW}New Configuration:${NC}"
echo -e "${BLUE}  - Max Supply: 3.6B FVC${NC}"
echo -e "${BLUE}  - Mining Allocation: 65% (2.34B FVC)${NC}"
echo -e "${BLUE}  - Owner Allocation: 15% (540M FVC)${NC}"
echo -e "${BLUE}  - Developer Allocation: 10% (360M FVC)${NC}"
echo -e "${BLUE}  - Ecosystem Maintenance: 10% (360M FVC)${NC}"
echo -e "${BLUE}  - Block Time: 5 seconds${NC}"
echo -e "${BLUE}  - Block Reward: 6.25 FVC${NC}"
echo -e "${BLUE}  - Halving: Every 2 years${NC}"
echo -e "${BLUE}  - Difficulty Adjustment: Bitcoin-style every 2023 blocks${NC}"
echo -e "${BLUE}  - Initial Difficulty: 10${NC}"
echo -e "${GREEN}========================================${NC}"

# Stop existing services
if command -v systemctl &> /dev/null; then
    echo -e "${YELLOW}Stopping existing FVChain services...${NC}"
    sudo systemctl stop fvc-mainnet 2>/dev/null || true
    sudo systemctl stop fvc-rpc 2>/dev/null || true
    sleep 2
fi

# Create installation directory
if [[ "$EUID" -eq 0 ]]; then
    mkdir -p "$INSTALL_PATH"
else
    sudo mkdir -p "$INSTALL_PATH"
    sudo chown "$(whoami):$(whoami)" "$INSTALL_PATH"
fi

# Backup existing configuration
if [[ "$SKIP_BACKUP" != "true" ]] && [[ -f "$INSTALL_PATH/mainnet-genesis.json" ]]; then
    echo -e "${YELLOW}Backing up existing configuration...${NC}"
    BACKUP_DIR="$INSTALL_PATH/backup/$(date +%Y%m%d_%H%M%S)"
    if [[ "$EUID" -eq 0 ]]; then
        mkdir -p "$BACKUP_DIR"
    else
        sudo mkdir -p "$BACKUP_DIR"
        sudo chown "$(whoami):$(whoami)" "$BACKUP_DIR"
    fi
    
    if [[ "$EUID" -eq 0 ]]; then
        cp "$INSTALL_PATH"/*.json "$BACKUP_DIR/" 2>/dev/null || true
        cp "$INSTALL_PATH"/*.toml "$BACKUP_DIR/" 2>/dev/null || true
        cp "$INSTALL_PATH"/*.env "$BACKUP_DIR/" 2>/dev/null || true
    else
        sudo cp "$INSTALL_PATH"/*.json "$BACKUP_DIR/" 2>/dev/null || true
        sudo cp "$INSTALL_PATH"/*.toml "$BACKUP_DIR/" 2>/dev/null || true
        sudo cp "$INSTALL_PATH"/*.env "$BACKUP_DIR/" 2>/dev/null || true
    fi
fi

# Build binaries
if [[ "$SKIP_BUILD" != "true" ]]; then
    echo -e "${YELLOW}Building FVChain binaries...${NC}"
    cargo build --release --bin fvc-mainnet
    if [ $? -ne 0 ]; then
        echo -e "${RED}Failed to build fvc-mainnet${NC}"
        exit 1
    fi
    
    cargo build --release --bin fvc-rpc
    if [ $? -ne 0 ]; then
        echo -e "${RED}Failed to build fvc-rpc${NC}"
        exit 1
    fi
fi

# Copy files to installation directory
echo -e "${YELLOW}Copying files to installation directory...${NC}"
if [[ "$EUID" -eq 0 ]]; then
    cp target/release/fvc-mainnet "$INSTALL_PATH/"
    cp target/release/fvc-rpc "$INSTALL_PATH/"
    cp mainnet-genesis.json "$INSTALL_PATH/"
    cp mainnet.env "$INSTALL_PATH/"
else
    sudo cp target/release/fvc-mainnet "$INSTALL_PATH/"
    sudo cp target/release/fvc-rpc "$INSTALL_PATH/"
    sudo cp mainnet-genesis.json "$INSTALL_PATH/"
    sudo cp mainnet.env "$INSTALL_PATH/"
fi

# Create mining configuration
cat > "$INSTALL_PATH/mining-config.json" << EOF
{
  "network": {
    "name": "FVChain Mainnet",
    "chainId": 369,
    "blockTime": 5
  },
  "mining": {
    "enabled": true,
    "algorithm": "FractalVortex",
    "blockReward": "6.25",
    "halvingInterval": "2 years",
    "difficulty": {
      "initial": 10,
      "adjustmentInterval": 2023,
      "targetBlockTime": 5,
      "algorithm": "bitcoin_adjustment"
    }
  },
  "allocation": {
    "totalSupply": "3.6B FVC",
    "mining": "65% (2.34B FVC)",
    "owner": "15% (540M FVC)",
    "developer": "10% (360M FVC)",
    "ecosystem": "10% (360M FVC)"
  }
}
EOF

# Create systemd service files
if command -v systemctl &> /dev/null; then
    echo -e "${YELLOW}Creating systemd service files...${NC}"
    
    # Mainnet service
    cat > /tmp/fvc-mainnet.service << EOF
[Unit]
Description=FVChain Mainnet Node with Bitcoin-style mining
Documentation=https://fvchain.xyz/docs
After=network.target

[Service]
Type=simple
User=fvchain
Group=fvchain
WorkingDirectory=$INSTALL_PATH
ExecStart=$INSTALL_PATH/fvc-mainnet --config $INSTALL_PATH/mainnet-genesis.json --env $INSTALL_PATH/mainnet.env
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=fvc-mainnet

[Install]
WantedBy=multi-user.target
EOF

    # RPC service
    cat > /tmp/fvc-rpc.service << EOF
[Unit]
Description=FVChain RPC Server for mainnet
Documentation=https://fvchain.xyz/docs
After=network.target fvc-mainnet.service

[Service]
Type=simple
User=fvchain
Group=fvchain
WorkingDirectory=$INSTALL_PATH
ExecStart=$INSTALL_PATH/fvc-rpc --config $INSTALL_PATH/mainnet-genesis.json --env $INSTALL_PATH/mainnet.env
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=fvc-rpc

[Install]
WantedBy=multi-user.target
EOF

    if [[ "$EUID" -eq 0 ]]; then
        # Create user if it doesn't exist
        if ! id -u fvchain &>/dev/null; then
            useradd -r -s /bin/false fvchain
        fi
        
        # Set ownership
        chown -R fvchain:fvchain "$INSTALL_PATH"
        
        # Install services
        mv /tmp/fvc-mainnet.service /etc/systemd/system/
        mv /tmp/fvc-rpc.service /etc/systemd/system/
        
        systemctl daemon-reload
        systemctl enable fvc-mainnet
        systemctl enable fvc-rpc
        
        echo -e "${GREEN}Services installed successfully!${NC}"
        echo -e "${YELLOW}Start with: sudo systemctl start fvc-mainnet fvc-rpc${NC}"
    else
        echo -e "${YELLOW}To install systemd services, run as root or use sudo${NC}"
    fi

# Create startup scripts
echo -e "${YELLOW}Creating startup scripts...${NC}"

# Mainnet startup script
cat > "$INSTALL_PATH/start-mainnet.sh" << 'EOF'
#!/bin/bash
echo "Starting FVChain Mainnet..."
echo "Configuration:"
echo "  - Initial Difficulty: 10"
echo "  - Adjustment Interval: 2023 blocks"
echo "  - Block Reward: 6.25 FVC"
echo "  - Block Time: 5 seconds"

./fvc-mainnet --config mainnet-genesis.json --env mainnet.env
EOF

# RPC startup script
cat > "$INSTALL_PATH/start-rpc.sh" << 'EOF'
#!/bin/bash
echo "Starting FVChain RPC Server..."
echo "RPC will be available at http://localhost:8545"

./fvc-rpc --config mainnet-genesis.json --env mainnet.env
EOF

# Combined startup script
cat > "$INSTALL_PATH/start-all.sh" << 'EOF'
#!/bin/bash
echo "Starting FVChain Mainnet and RPC Server..."
echo "========================================"
echo "FVChain Mainnet Configuration:"
echo "  - Network: FVChain Mainnet (Chain ID: 369)"
echo "  - Initial Difficulty: 10"
echo "  - Adjustment: Every 2023 blocks"
echo "  - Block Reward: 6.25 FVC"
echo "  - Halving: Every 2 years"
echo "  - Mining Allocation: 65% of 3.6B FVC"
echo "========================================"

nohup ./fvc-mainnet --config mainnet-genesis.json --env mainnet.env > mainnet.log 2>&1 &
MAINNET_PID=$!
echo "Mainnet started with PID: $MAINNET_PID"

sleep 2

nohup ./fvc-rpc --config mainnet-genesis.json --env mainnet.env > rpc.log 2>&1 &
RPC_PID=$!
echo "RPC Server started with PID: $RPC_PID"

echo "Services started. Check logs: mainnet.log, rpc.log"
echo "To stop: kill $MAINNET_PID $RPC_PID"
EOF

# Make scripts executable
if [[ "$EUID" -eq 0 ]]; then
    chmod +x "$INSTALL_PATH"/*.sh
else
    sudo chmod +x "$INSTALL_PATH"/*.sh
fi

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}FVChain Mainnet Rebuild Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "${BLUE}Files deployed to: $INSTALL_PATH${NC}"
echo -e "${YELLOW}Configuration updated:${NC}"
echo -e "${GREEN}  - Initial Difficulty: 10${NC}"
echo -e "${GREEN}  - Adjustment Interval: 2023 blocks${NC}"
echo -e "${GREEN}  - Bitcoin-style difficulty adjustment${NC}"
echo -e "${GREEN}  - 6.25 FVC block reward with 2-year halving${NC}"
echo -e "${GREEN}  - 65% mining allocation (2.34B FVC)${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "${YELLOW}To start the network:${NC}"
echo -e "${BLUE}  1. Run: $INSTALL_PATH/start-all.sh${NC}"
echo -e "${BLUE}  2. Or run individual: start-mainnet.sh and start-rpc.sh${NC}"
if command -v systemctl &> /dev/null; then
    echo -e "${BLUE}  3. For systemd: sudo systemctl start fvc-mainnet fvc-rpc${NC}"
fi
echo -e "${GREEN}========================================${NC}"