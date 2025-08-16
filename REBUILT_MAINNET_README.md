# FVChain Mainnet Rebuilt Configuration

## Overview
This document describes the new FVChain mainnet configuration with updated mining parameters and Bitcoin-style difficulty adjustment.

## Token Economics
- **Total Supply**: 3,600,000,000 FVC (3.6 billion)
- **Mining Allocation**: 65% (2,340,000,000 FVC)
- **Owner Allocation**: 15% (540,000,000 FVC) - 100% PAID at genesis
- **Developer Allocation**: 10% (360,000,000 FVC) - 100% PAID at genesis  
- **Ecosystem Maintenance**: 10% (360,000,000 FVC) - 100% PAID at genesis

## Mining Configuration
- **Block Time**: 5 seconds
- **Block Reward**: 6.25 FVC
- **Halving**: Every 2 years (12,614,400 blocks)
- **Difficulty Adjustment**: Bitcoin-style every 2023 blocks
- **Initial Difficulty**: 10
- **Adjustment Algorithm**: Bitcoin-style adjustment based on actual vs target block times

## New Components

### 1. Difficulty Adjuster (`src/consensus/difficulty_adjuster.rs`)
- Implements Bitcoin-style difficulty adjustment
- Adjusts every 2023 blocks (instead of Bitcoin's 2016)
- Maximum adjustment factor of 4x (400%) or 0.25x (25%)
- Minimum difficulty of 1

### 2. Mining Engine (`src/consensus/mining_engine.rs`)
- Integrates difficulty adjustment with mining rewards
- Tracks network hashrate and mining statistics
- Manages current difficulty and block height
- Provides mining performance metrics

### 3. Mining Rewards (`src/consensus/mining_rewards.rs`)
- Updated to reflect 2.34B FVC mining allocation
- 6.25 FVC base reward with 2-year halving
- 32 maximum halvings (64 years total)

### 4. Genesis Configuration (`mainnet-genesis.json`)
- Updated to enable mining with proper allocations
- Initial difficulty set to 10
- Bitcoin-style adjustment algorithm enabled
- 2023-block adjustment interval

## Deployment Instructions

### Windows
1. Run `rebuild-mainnet.ps1` as Administrator
2. Configuration will be deployed to `C:\Program Files\FVChain\`
3. Use `start-all.bat` to start services

### Linux/macOS
1. Run `rebuild-mainnet.sh` with sudo
2. Configuration will be deployed to `/opt/fvchain/`
3. Use systemd services: `sudo systemctl start fvc-mainnet fvc-rpc`

## Initial Mining Statistics
- **Expected Network Hashrate**: ~2 MH/s (with difficulty 10)
- **Blocks per Day**: 17,280 blocks
- **Daily Mining Reward**: 108,000 FVC
- **Mining Duration**: 64 years (until all 2.34B FVC mined)
- **Final Block**: Block 12,614,400

## API Endpoints

### Mining Statistics
- `GET /api/v1/mining/stats` - Current mining statistics
- `GET /api/v1/mining/reward` - Current block reward
- `GET /api/v1/mining/difficulty` - Current difficulty and next adjustment

### Block Information
- `GET /api/v1/blocks/latest` - Latest block details
- `GET /api/v1/blocks/{height}` - Block at specific height
- `GET /api/v1/blocks/difficulty/{height}` - Difficulty at block height

## Monitoring

### Key Metrics to Monitor
1. **Block Time**: Target 5 seconds
2. **Difficulty**: Should adjust based on network hashrate
3. **Hashrate**: Network mining power
4. **Block Rewards**: Decreasing every 2 years
5. **Supply Mined**: Progress toward 2.34B FVC

### Log Files
- Mainnet: `mainnet.log`
- RPC Server: `rpc.log`
- System logs: `journalctl -u fvc-mainnet -f`

## Troubleshooting

### Common Issues
1. **Difficulty too high/low**: Check network hashrate and adjustment algorithm
2. **Slow block times**: Monitor network connectivity and mining participation
3. **Reward calculation errors**: Verify block height and halving schedule

### Reset to Genesis
To reset the chain to genesis state:
```bash
# Stop services
sudo systemctl stop fvc-mainnet fvc-rpc

# Remove blockchain data
sudo rm -rf /opt/fvchain/data/

# Restart services
sudo systemctl start fvc-mainnet fvc-rpc
```

## Security Considerations
- Private keys for genesis wallets are securely managed
- Mining rewards are distributed according to predetermined schedule
- Difficulty adjustment prevents manipulation
- Halving schedule ensures predictable supply emission

## Next Steps
1. Monitor initial mining performance
2. Adjust parameters based on real network conditions
3. Prepare for first halving in 2 years
4. Implement additional mining pool support
5. Add advanced difficulty adjustment algorithms if needed

## Configuration Files
- `mainnet-genesis.json` - Updated genesis configuration
- `mining-config.json` - Mining parameters and allocation details
- `mainnet.env` - Environment variables for the mainnet

## Support
For technical support or questions about the mining configuration, please refer to:
- GitHub Issues: [FVChain Repository](https://github.com/fvchain/fvchain)
- Documentation: [FVChain Docs](https://fvchain.xyz/docs)
- Community: [FVChain Discord](https://discord.gg/fvchain)