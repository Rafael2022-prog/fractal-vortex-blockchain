// FVChain Network Configuration Example
// Copy this file to networks.ts and update with your actual values

import { NetworkConfig } from '../types';

export const NETWORKS: Record<string, NetworkConfig> = {
  fvchain_mainnet: {
    name: 'FVChain Mainnet',
    rpcUrl: 'https://your-mainnet-rpc-url.com',
    chainId: 369,
    symbol: 'FVC',
    decimals: 18,
    explorerUrl: 'https://your-mainnet-explorer.com',
    blockExplorer: 'https://your-mainnet-explorer.com',
  },
  fvchain_testnet: {
    name: 'FVChain Testnet',
    rpcUrl: 'https://your-testnet-rpc-url.com',
    chainId: 370,
    symbol: 'FVC',
    decimals: 18,
    explorerUrl: 'https://your-testnet-explorer.com',
    blockExplorer: 'https://your-testnet-explorer.com',
  },
  fvchain_local: {
    name: 'FVChain Local',
    rpcUrl: 'http://localhost:8080',
    chainId: 31337,
    symbol: 'FVC',
    decimals: 18,
    explorerUrl: 'http://localhost:8080/explorer',
    blockExplorer: 'http://localhost:8080/explorer',
  },
};

export const DEFAULT_NETWORK = 'fvchain_local';

export const TOKEN_LIST = [
  {
    chainId: 369,
    address: 'native',
    name: 'Fractal Vortex Coin',
    symbol: 'FVC',
    decimals: 18,
    logoURI: '/logo.svg',
  },
  {
    chainId: 370,
    address: 'native',
    name: 'Fractal Vortex Coin Testnet',
    symbol: 'FVC',
    decimals: 18,
    logoURI: '/logo.svg',
  },
];

export default NETWORKS;