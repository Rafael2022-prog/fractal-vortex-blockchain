# FVChain SDK

Official Software Development Kit (SDK) for **Fractal Vortex Chain (FVChain)** - a secure, scalable, and production-ready Layer 1 blockchain.

## 🚀 Features

- **Multi-language Support**: JavaScript/Node.js and Python SDKs
- **Complete API Coverage**: All FVChain RPC endpoints supported
- **Type Safety**: Full TypeScript definitions and Python type hints
- **Error Handling**: Comprehensive error handling with custom exceptions
- **Easy Integration**: Simple and intuitive API design
- **Production Ready**: Built for mainnet deployment

## 📦 Installation

### JavaScript/Node.js

```bash
npm install fvchain-sdk
# or
yarn add fvchain-sdk
```

### Python

```bash
pip install fvchain-sdk
# or
pip3 install fvchain-sdk
```

## 🔧 Quick Start

### JavaScript/Node.js

```javascript
const { FVChainSDK } = require('fvchain-sdk');

// Initialize SDK
const fvchain = new FVChainSDK({
  baseURL: 'http://localhost:8080', // or https://api.fvchain.xyz for mainnet
  timeout: 15000
});

// Example usage
(async () => {
  try {
    // Get network information
    const networkInfo = await fvchain.getNetworkInfo();
    console.log('Network Info:', networkInfo);

    // Create a new wallet
    const wallet = await fvchain.createWallet();
    console.log('New Wallet:', wallet);

    // Get wallet balance
    const balance = await fvchain.getWalletBalance(wallet.address);
    console.log('Balance:', balance);

    // Get latest blocks
    const blocks = await fvchain.getLatestBlocks(5);
    console.log('Latest Blocks:', blocks);

  } catch (error) {
    console.error('Error:', error.message);
  }
})();
```

### Python

```python
from fvchain_sdk import FVChainSDK, FVChainError

# Initialize SDK
fvchain = FVChainSDK(
    base_url="http://localhost:8080",  # or https://api.fvchain.xyz for mainnet
    timeout=15
)

try:
    # Get network information
    network_info = fvchain.get_network_info()
    print(f"Network Info: {network_info}")

    # Create a new wallet
    wallet = fvchain.create_wallet()
    print(f"New Wallet: {wallet}")

    # Get wallet balance
    balance = fvchain.get_wallet_balance(wallet.address)
    print(f"Balance: {balance}")

    # Get latest blocks
    blocks = fvchain.get_latest_blocks(5)
    print(f"Latest Blocks: {len(blocks)} blocks")

except FVChainError as e:
    print(f"FVChain Error: {e.message}")
except Exception as e:
    print(f"Error: {str(e)}")
```

## 📚 API Reference

### Network Methods

- `getNetworkInfo()` / `get_network_info()` - Get network information
- `getNetworkNodes()` / `get_network_nodes()` - Get active network nodes

### Block Methods

- `getLatestBlocks(limit)` / `get_latest_blocks(limit)` - Get latest blocks
- `getBlockByHeight(height)` / `get_block_by_height(height)` - Get block by height
- `getCurrentBlockHeight()` / `get_current_block_height()` - Get current block height

### Transaction Methods

- `getLatestTransactions(limit)` / `get_latest_transactions(limit)` - Get latest transactions
- `getTransactionByHash(hash)` / `get_transaction_by_hash(hash)` - Get transaction by hash

### Wallet Methods

- `createWallet()` / `create_wallet()` - Create a new wallet
- `getWalletBalance(address)` / `get_wallet_balance(address)` - Get wallet balance
- `validateAddress(address)` / `validate_address(address)` - Validate address format

### Mining Methods

- `getMinerStatus(deviceId)` / `get_miner_status(device_id)` - Get miner status
- `sendMiningHeartbeat(deviceId, sessionToken, timestamp)` / `send_mining_heartbeat(device_id, session_token, timestamp)` - Send mining heartbeat
- `getMiningStats()` / `get_mining_stats()` - Get mining statistics

### Admin Methods

- `getRateLimitStats()` / `get_rate_limit_stats()` - Get rate limit statistics (admin only)

### Utility Methods

- `generateDeviceId()` / `generate_device_id()` - Generate unique device ID
- `generateSessionToken()` / `generate_session_token()` - Generate session token
- `toSmallestUnit(amount)` / `to_smallest_unit(amount)` - Convert to smallest unit
- `fromSmallestUnit(amount)` / `from_smallest_unit(amount)` - Convert from smallest unit
- `isServerReachable()` / `is_server_reachable()` - Check server connectivity
- `getServerHealth()` / `get_server_health()` - Get server health status

## 🔐 Configuration

### Environment Variables

You can configure the SDK using environment variables:

```bash
# Set default base URL
export FVCHAIN_BASE_URL=https://api.fvchain.xyz

# Set default timeout
export FVCHAIN_TIMEOUT=30000

# Set API key (if required)
export FVCHAIN_API_KEY=your_api_key_here
```

### Custom Headers

```javascript
// JavaScript
const fvchain = new FVChainSDK({
  baseURL: 'https://api.fvchain.xyz',
  headers: {
    'Authorization': 'Bearer your_token_here',
    'X-Custom-Header': 'custom_value'
  }
});
```

```python
# Python
fvchain = FVChainSDK(
    base_url="https://api.fvchain.xyz",
    headers={
        'Authorization': 'Bearer your_token_here',
        'X-Custom-Header': 'custom_value'
    }
)
```

## 🌐 Network Endpoints

### Mainnet
- **RPC URL**: `https://rpc.fvchain.xyz`
- **API URL**: `https://api.fvchain.xyz`
- **Explorer**: `https://explorer.fvchain.xyz`

### Testnet
- **RPC URL**: `https://testnet-rpc.fvchain.xyz`
- **API URL**: `https://testnet-api.fvchain.xyz`
- **Explorer**: `https://testnet-explorer.fvchain.xyz`

### Local Development
- **RPC URL**: `http://localhost:8080`
- **Dashboard**: `http://localhost:3000`

## 🛡️ Error Handling

The SDK provides comprehensive error handling:

### JavaScript

```javascript
try {
  const result = await fvchain.getNetworkInfo();
} catch (error) {
  if (error instanceof FVChainError) {
    console.error('FVChain Error:', error.message);
    console.error('Status Code:', error.statusCode);
    console.error('Error Code:', error.code);
  } else {
    console.error('Unexpected Error:', error.message);
  }
}
```

### Python

```python
try:
    result = fvchain.get_network_info()
except FVChainError as e:
    print(f"FVChain Error: {e.message}")
    print(f"Status Code: {e.status_code}")
    print(f"Error Code: {e.error_code}")
except Exception as e:
    print(f"Unexpected Error: {str(e)}")
```

## 🔄 Rate Limiting

The FVChain API implements rate limiting to ensure fair usage:

- **Default Limit**: 100 requests per minute per IP
- **Burst Limit**: 20 requests per 10 seconds
- **Admin Endpoints**: 10 requests per minute

The SDK automatically handles rate limit responses and provides appropriate error messages.

## 🧪 Testing

### JavaScript

```bash
# Install dependencies
npm install

# Run tests
npm test

# Run example
npm run example
```

### Python

```bash
# Install development dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Run with coverage
pytest --cov=fvchain_sdk

# Run example
python fvchain_sdk.py
```

## 📖 Examples

Check the `examples/` directory for more comprehensive usage examples:

- `basic-usage.js` / `basic_usage.py` - Basic SDK operations
- `wallet-operations.js` / `wallet_operations.py` - Wallet management
- `mining-integration.js` / `mining_integration.py` - Mining operations
- `block-explorer.js` / `block_explorer.py` - Block and transaction queries

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

1. Clone the repository
2. Install dependencies
3. Run tests
4. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: [https://docs.fvchain.xyz](https://docs.fvchain.xyz)
- **Discord**: [https://discord.gg/fvchain](https://discord.gg/fvchain)
- **Telegram**: [https://t.me/fvchain](https://t.me/fvchain)
- **Email**: [support@fvchain.xyz](mailto:support@fvchain.xyz)
- **GitHub Issues**: [https://github.com/fvchain/fvchain-sdk/issues](https://github.com/fvchain/fvchain-sdk/issues)

## 🔗 Links

- **Website**: [https://fvchain.xyz](https://fvchain.xyz)
- **Whitepaper**: [https://fvchain.xyz/whitepaper](https://fvchain.xyz/whitepaper)
- **GitHub**: [https://github.com/fvchain](https://github.com/fvchain)
- **Twitter**: [https://twitter.com/fvchain](https://twitter.com/fvchain)

---

**Built with ❤️ by the FVChain Development Team**

*Fractal Vortex Chain - Secure, Scalable, Sustainable*