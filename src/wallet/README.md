# Fractal-Vortex Chain Official Wallet (FVC Wallet)

The official wallet implementation for the Fractal-Vortex Chain (FVC) blockchain, providing secure key management, transaction handling, and blockchain interaction capabilities.

## Features

- **Secure Key Management**: Generate and manage cryptographic keys using libp2p identity system
- **Mnemonic Support**: BIP39 compatible mnemonic phrase generation and recovery
- **Transaction Management**: Create, sign, and broadcast FVC transactions
- **Staking Support**: Stake and unstake FVC tokens to validators
- **Balance Tracking**: Real-time balance and transaction history
- **CLI Interface**: Command-line interface for all wallet operations
- **RPC Integration**: Direct communication with FVC nodes

## Installation

### Prerequisites

- Rust 1.70+ (latest stable recommended)
- FVC node running locally or accessible RPC endpoint

### Build from Source

```bash
cd fractal-vortex-chain
cargo build --release --bin fvc-wallet
```

The binary will be available at `target/release/fvc-wallet`

## Quick Start

### 1. Create a New Wallet

```bash
# Create new wallet and save to file
./fvc-wallet create --save-to wallet.json

# Create wallet with custom mnemonic
./fvc-wallet create --mnemonic "your twelve word mnemonic phrase here" --save-to wallet.json
```

### 2. Check Wallet Information

```bash
# Show basic wallet info
./fvc-wallet --wallet-file wallet.json info

# Show detailed balance
./fvc-wallet --wallet-file wallet.json info --balance
```

### 3. Send FVC

```bash
# Send 100 FVC to another address
./fvc-wallet --wallet-file wallet.json send 0x123...abc 100

# Send with custom fee
./fvc-wallet --wallet-file wallet.json send 0x123...abc 100 --fee 1000
```

### 4. Staking Operations

```bash
# Stake FVC to validator
./fvc-wallet --wallet-file wallet.json stake validator_peer_id 1000

# Unstake FVC from validator
./fvc-wallet --wallet-file wallet.json unstake validator_peer_id 500

# Check staking information
./fvc-wallet --wallet-file wallet.json staking
```

## CLI Commands

### Wallet Management
- `create` - Create new wallet
- `info` - Show wallet information
- `export` - Export wallet to file
- `import` - Import wallet from file

### Transaction Operations
- `send` - Send FVC to address
- `stake` - Stake FVC to validator
- `unstake` - Unstake FVC from validator
- `tx` - Check transaction status

### Information Queries
- `balance` - Check wallet balance
- `staking` - Show staking information
- `network` - Show network information
- `history` - Show transaction history

## Configuration

### Environment Variables
- `FVC_RPC_URL` - Default RPC endpoint (default: http://localhost:8080)
- `FVC_WALLET_FILE` - Default wallet file path

### Node Configuration
The wallet connects to FVC nodes via RPC. Ensure your node is running and accessible:

```bash
# Start FVC node
cargo run --release -- start --node-id 0

# Check node status
curl http://localhost:8080/network/info
```

## Wallet Structure

### Key Components

1. **KeyManager** (`key_manager.rs`)
   - Key generation and management
   - Mnemonic phrase support
   - Digital signatures

2. **Transaction** (`transaction.rs`)
   - Transaction building
   - Transaction signing
   - Transaction verification

3. **RPC Client** (`rpc_client.rs`)
   - Node communication
   - Balance queries
   - Transaction broadcasting

4. **Wallet** (`wallet.rs`)
   - Unified wallet interface
   - High-level operations
   - Balance management

5. **CLI** (`cli.rs`)
   - Command-line interface
   - User interaction
   - Error handling

## Security Features

- **Private Key Encryption**: Keys are encrypted when saved to disk
- **Mnemonic Backup**: BIP39 compatible recovery phrases
- **Transaction Signing**: All transactions are cryptographically signed
- **Secure Storage**: Private keys never leave the wallet
- **Network Security**: TLS/SSL support for RPC connections

## API Usage

### Programmatic Usage

```rust
use fractal_vortex_chain::wallet::Wallet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create wallet
    let mut wallet = Wallet::new("http://localhost:8080");
    
    // Check balance
    let balance = wallet.get_balance().await?;
    println!("Balance: {} FVC", balance);
    
    // Send transaction
    let tx_hash = wallet.transfer("recipient_address", 100).await?;
    println!("Transaction: {}", tx_hash);
    
    Ok(())
}
```

### Wallet Operations

```rust
// Create from mnemonic
let wallet = Wallet::from_mnemonic("twelve word mnemonic...", "http://localhost:8080");

// Load from file
let wallet = Wallet::from_file("wallet.json", "http://localhost:8080")?;

// Save to file
wallet.save_to_file("wallet.json")?;

// Get address
let address = wallet.get_address();
let peer_id = wallet.get_peer_id();
```

## Development

### Running Tests

```bash
cargo test --package fractal-vortex-chain --lib wallet::tests
```

### Adding New Features

1. Add new commands to `cli.rs`
2. Implement functionality in `wallet.rs`
3. Add RPC methods to `rpc_client.rs`
4. Update documentation

## Troubleshooting

### Common Issues

**Connection Error**
```bash
# Check if node is running
curl http://localhost:8080/health

# Check network connectivity
telnet localhost 8080
```

**Insufficient Balance**
```bash
# Check balance
./fvc-wallet --wallet-file wallet.json info --balance

# Check network status
./fvc-wallet --wallet-file wallet.json network
```

**Invalid Transaction**
```bash
# Check transaction status
./fvc-wallet --wallet-file wallet.json tx <tx_hash>

# Check nonce
./fvc-wallet --wallet-file wallet.json info --balance
```

## Support

For issues and questions:
- GitHub Issues: [FVC Protocol](https://github.com/fractalvortex/fvc-protocol/issues)
- Documentation: [FVC Docs](https://docs.fractalvortex.com)
- Community: [Discord](https://discord.gg/fractalvortex)

## License

MIT License - see [LICENSE](LICENSE) file for details.