# Fractal Vortex Chain (FVChain) - Blockchain Layer 1

<mcreference link="https://github.com/Rafael2022-prog/fractal-vortex-blockchain" index="0">0</mcreference>

## Overview

Fractal Vortex Chain (FVC) adalah blockchain Layer-1 generasi berikutnya yang mengintegrasikan matematika fractal dengan pola vortex untuk menciptakan arsitektur blockchain yang revolusioner. FVC menggabungkan self-similarity fractal dengan matematika vortex (pola 1-2-4-8-7-5) untuk mencapai skalabilitas tak terbatas, efisiensi energi tinggi, dan keamanan kriptografi yang kuat.

## Key Features

- **Fractal Mathematics**: Implementasi self-similarity fractal untuk skalabilitas
- **Vortex Pattern**: Pola matematika vortex (1-2-4-8-7-5) untuk optimasi konsensus
- **Energy Efficient**: Algoritma konsensus yang hemat energi
- **High Security**: Kriptografi tingkat enterprise dengan audit keamanan
- **Native Address**: Format address FVChain native dengan prefix `fvc1`
- **Smart Contracts**: Dukungan smart contract dengan Rust dan Solidity

## Architecture

### Core Components

- **Blockchain Core** (`src/`): Implementasi blockchain utama dalam Rust
- **Consensus Engine** (`src/consensus/`): Algoritma konsensus fractal-vortex
- **Cryptography** (`src/crypto/`): Implementasi kriptografi fractal hash
- **Network Layer** (`src/network/`): P2P networking dan komunikasi node
- **Storage Engine** (`src/storage/`): Penyimpanan blockchain dan state
- **Mining System** (`src/mining/`): Sistem mining dengan smart rate
- **Wallet System** (`src/wallet/`): Wallet CLI dan key management

### Network Specifications

- **Mainnet Chain ID**: 369
- **Testnet Chain ID**: 370
- **Block Time**: 10 minutes (600 seconds)
- **Token Symbol**: FVC
- **Decimals**: 18
- **Total Supply**: 375,390,000 FVC (fixed supply)

## Quick Start

### Prerequisites

- Rust 1.70+ 
- Node.js 18+
- Git

### Installation

1. Clone the repository:
```bash
git clone https://github.com/Rafael2022-prog/fractal-vortex-blockchain.git
cd fractal-vortex-blockchain
```

2. Setup configuration:
```bash
cp config.example.toml config.toml
cp .env.example .env
```

3. Edit configuration files with your settings:
- Update `config.toml` with your network configuration
- Update `.env` with your environment variables
- **Never commit real credentials or production IPs**

4. Build the project:
```bash
cargo build --release
```

5. Initialize blockchain:
```bash
cargo run --bin fvchain -- init
```

6. Start the node:
```bash
cargo run --bin fvchain -- start
```

### Development Setup

For development environment:

```bash
# Start local node
cargo run -- start --network local

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

## Configuration

### Network Configuration

Edit `config.toml`:

```toml
[network]
rpc_host = "0.0.0.0"
rpc_port = 8080
chain_id = 369  # Use 370 for testnet
network_name = "fvchain_mainnet"
```

### Security Configuration

```toml
[security]
max_connections = 100
rate_limit_requests_per_minute = 60
```

### Mining Configuration

```toml
[mining]
difficulty_adjustment_interval = 2016
target_block_time = 600
max_block_size = 1048576
```

## API Documentation

### RPC Endpoints

- `GET /network/info` - Get network information
- `GET /network/health` - Health check
- `GET /blocks` - Get latest blocks
- `GET /blocks/{height}` - Get block by height
- `POST /transactions` - Submit transaction
- `GET /wallet/balance/{address}` - Get wallet balance

### Example API Calls

```bash
# Get network info
curl http://localhost:8080/network/info

# Get latest blocks
curl http://localhost:8080/blocks?limit=10

# Check wallet balance
curl http://localhost:8080/wallet/balance/fvc1...
```

## Wallet Usage

### CLI Wallet

```bash
# Create new wallet
cargo run --bin wallet-cli -- create

# Import existing wallet
cargo run --bin wallet-cli -- import --private-key <key>

# Check balance
cargo run --bin wallet-cli -- balance --address <address>

# Send transaction
cargo run --bin wallet-cli -- send --to <address> --amount <amount>
```

### Address Format

FVChain uses native address format:
- Prefix: `fvc1`
- Length: 42 characters
- Example: `fvc1qw3r7y8u9i0p2s4d6f8g0h2j4k6l8n0q2w4e6r8t`

## Mining

### Start Mining

```bash
# Start mining with default settings
cargo run -- mine --address <your_address>

# Start mining with custom difficulty
cargo run -- mine --address <your_address> --difficulty 1000
```

### Mining Pool

For mining pool setup, refer to the mining documentation in the repository.

## Security

### Best Practices

1. **Never commit private keys or credentials**
2. **Use environment variables for sensitive data**
3. **Enable SSL/TLS in production**
4. **Implement proper rate limiting**
5. **Regular security audits**
6. **Use hardware security modules (HSM) for production keys**

### Security Features

- Fractal hash algorithm for enhanced security
- Multi-signature support
- Hardware wallet integration
- Encrypted key storage
- Rate limiting and DDoS protection

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
cargo test --test integration_test
```

### Security Tests

```bash
cargo test --test security_tests
```

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

### Development Guidelines

- Follow Rust coding standards
- Add tests for new features
- Update documentation
- Ensure security best practices
- No sensitive data in commits

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support and questions:
- Create an issue on GitHub
- Join our community discussions
- Check the documentation

## Roadmap

- [ ] Smart contract virtual machine
- [ ] Cross-chain bridges
- [ ] Mobile wallet applications
- [ ] Decentralized governance
- [ ] Layer 2 scaling solutions
- [ ] Enterprise integrations

## Disclaimer

This is experimental blockchain technology. Use at your own risk. Never use production credentials in development or testing environments.

---

**Created by Emylton Leunufna - 2025**

*Fractal Vortex Chain: Where Mathematics Meets Blockchain Innovation*