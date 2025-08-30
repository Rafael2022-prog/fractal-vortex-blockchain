/**
 * FVChain SDK - JavaScript/Node.js Client Library
 * 
 * Official SDK for integrating with Fractal Vortex Chain (FVChain)
 * Provides easy-to-use methods for blockchain interactions
 * 
 * @version 1.0.0
 * @author FVChain Development Team
 */

const axios = require('axios');

class FVChainSDK {
  /**
   * Initialize FVChain SDK
   * @param {Object} config - Configuration options
   * @param {string} config.baseURL - Base URL for FVChain RPC server
   * @param {number} config.timeout - Request timeout in milliseconds
   * @param {Object} config.headers - Additional headers
   */
  constructor(config = {}) {
    this.config = {
      baseURL: config.baseURL || 'http://localhost:8080',
      timeout: config.timeout || 10000,
      headers: {
        'Content-Type': 'application/json',
        ...config.headers
      }
    };

    this.client = axios.create({
      baseURL: this.config.baseURL,
      timeout: this.config.timeout,
      headers: this.config.headers
    });

    // Add response interceptor for error handling
    this.client.interceptors.response.use(
      response => response,
      error => {
        if (error.response) {
          // Server responded with error status
          const errorData = error.response.data;
          throw new FVChainError(
            errorData.message || error.message,
            error.response.status,
            errorData.code || 'UNKNOWN_ERROR'
          );
        } else if (error.request) {
          // Request was made but no response received
          throw new FVChainError(
            'Network error: Unable to connect to FVChain server',
            0,
            'NETWORK_ERROR'
          );
        } else {
          // Something else happened
          throw new FVChainError(error.message, 0, 'REQUEST_ERROR');
        }
      }
    );
  }

  // ==================== NETWORK METHODS ====================

  /**
   * Get network information
   * @returns {Promise<Object>} Network information
   */
  async getNetworkInfo() {
    const response = await this.client.get('/network/info');
    return response.data;
  }

  /**
   * Get network nodes
   * @returns {Promise<Object>} Active network nodes
   */
  async getNetworkNodes() {
    const response = await this.client.get('/network/nodes');
    return response.data;
  }

  // ==================== BLOCK METHODS ====================

  /**
   * Get latest blocks
   * @param {number} limit - Number of blocks to retrieve (default: 10, max: 100)
   * @returns {Promise<Object>} Latest blocks data
   */
  async getLatestBlocks(limit = 10) {
    if (limit > 100) limit = 100;
    const response = await this.client.get(`/blocks/latest?limit=${limit}`);
    return response.data;
  }

  /**
   * Get block by height
   * @param {number} height - Block height
   * @returns {Promise<Object>} Block data
   */
  async getBlockByHeight(height) {
    const response = await this.client.get(`/blocks/${height}`);
    return response.data;
  }

  /**
   * Get current block height
   * @returns {Promise<number>} Current block height
   */
  async getCurrentBlockHeight() {
    const networkInfo = await this.getNetworkInfo();
    return networkInfo.latest_block_height;
  }

  // ==================== TRANSACTION METHODS ====================

  /**
   * Get latest transactions
   * @param {number} limit - Number of transactions to retrieve (default: 10, max: 200)
   * @returns {Promise<Object>} Latest transactions data
   */
  async getLatestTransactions(limit = 10) {
    if (limit > 200) limit = 200;
    const response = await this.client.get(`/transactions?limit=${limit}`);
    return response.data;
  }

  /**
   * Get transaction by hash
   * @param {string} hash - Transaction hash
   * @returns {Promise<Object>} Transaction data
   */
  async getTransactionByHash(hash) {
    const response = await this.client.get(`/transactions/${hash}`);
    return response.data;
  }

  // ==================== WALLET METHODS ====================

  /**
   * Create a new wallet
   * @returns {Promise<Object>} New wallet data with address and private key
   */
  async createWallet() {
    const response = await this.client.get('/wallet/create');
    return response.data;
  }

  /**
   * Get wallet balance
   * @param {string} address - Wallet address
   * @returns {Promise<Object>} Wallet balance data
   */
  async getWalletBalance(address) {
    const response = await this.client.post('/wallet/balance', { address });
    return response.data;
  }

  /**
   * Validate wallet address format
   * @param {string} address - Wallet address to validate
   * @returns {boolean} True if address format is valid
   */
  validateAddress(address) {
    // FVChain addresses start with 'fvc', followed by 36 hex chars, and end with 'emyl' (43 chars total)
    const addressRegex = /^fvc[0-9a-fA-F]{36}emyl$/;
    return addressRegex.test(address);
  }

  // ==================== MINING METHODS ====================

  /**
   * Get miner status
   * @param {string} deviceId - Device ID
   * @returns {Promise<Object>} Miner status data
   */
  async getMinerStatus(deviceId) {
    const response = await this.client.get(`/miner/status?device_id=${deviceId}`);
    return response.data;
  }

  /**
   * Send mining heartbeat
   * @param {string} deviceId - Device ID
   * @param {string} sessionToken - Session token
   * @param {number} timestamp - Current timestamp
   * @returns {Promise<Object>} Heartbeat response
   */
  async sendMiningHeartbeat(deviceId, sessionToken, timestamp = Date.now()) {
    const response = await this.client.post('/api/v1/mining/heartbeat', {
      device_id: deviceId,
      session_token: sessionToken,
      timestamp: Math.floor(timestamp / 1000)
    });
    return response.data;
  }

  /**
   * Get mining detection statistics
   * @returns {Promise<Object>} Mining detection stats
   */
  async getMiningStats() {
    const response = await this.client.get('/mining/detection/stats');
    return response.data;
  }

  // ==================== ADMIN METHODS ====================

  /**
   * Get rate limit statistics (admin only)
   * @returns {Promise<Object>} Rate limit statistics
   */
  async getRateLimitStats() {
    const response = await this.client.get('/admin/rate-limit/stats');
    return response.data;
  }

  // ==================== UTILITY METHODS ====================

  /**
   * Generate a unique device ID
   * @returns {string} Unique device ID
   */
  generateDeviceId() {
    const timestamp = Date.now().toString(36);
    const random = Math.random().toString(36).substr(2, 9);
    return `device_${timestamp}_${random}`;
  }

  /**
   * Generate a session token
   * @returns {string} Session token
   */
  generateSessionToken() {
    const timestamp = Date.now().toString(36);
    const random = Math.random().toString(36).substr(2, 15);
    return `session_${timestamp}_${random}`;
  }

  /**
   * Convert FVC amount to smallest unit (similar to wei in Ethereum)
   * @param {number} fvc - Amount in FVC
   * @returns {string} Amount in smallest unit
   */
  toSmallestUnit(fvc) {
    return (fvc * 1e18).toString();
  }

  /**
   * Convert from smallest unit to FVC
   * @param {string|number} smallestUnit - Amount in smallest unit
   * @returns {number} Amount in FVC
   */
  fromSmallestUnit(smallestUnit) {
    return Number(smallestUnit) / 1e18;
  }

  /**
   * Check if the FVChain server is reachable
   * @returns {Promise<boolean>} True if server is reachable
   */
  async isServerReachable() {
    try {
      await this.getNetworkInfo();
      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * Get server health status
   * @returns {Promise<Object>} Server health information
   */
  async getServerHealth() {
    try {
      const networkInfo = await this.getNetworkInfo();
      const isReachable = await this.isServerReachable();
      
      return {
        status: 'healthy',
        reachable: isReachable,
        block_height: networkInfo.latest_block_height,
        active_nodes: networkInfo.active_nodes,
        timestamp: new Date().toISOString()
      };
    } catch (error) {
      return {
        status: 'unhealthy',
        reachable: false,
        error: error.message,
        timestamp: new Date().toISOString()
      };
    }
  }
}

/**
 * Custom error class for FVChain SDK
 */
class FVChainError extends Error {
  constructor(message, statusCode = 0, code = 'UNKNOWN_ERROR') {
    super(message);
    this.name = 'FVChainError';
    this.statusCode = statusCode;
    this.code = code;
  }
}

// Export classes
module.exports = {
  FVChainSDK,
  FVChainError
};

// Example usage (commented out)
/*
const { FVChainSDK } = require('./fvchain-sdk');

// Initialize SDK
const fvchain = new FVChainSDK({
  baseURL: 'http://localhost:8080',
  timeout: 15000
});

// Example usage
(async () => {
  try {
    // Check server health
    const health = await fvchain.getServerHealth();
    console.log('Server Health:', health);

    // Get network information
    const networkInfo = await fvchain.getNetworkInfo();
    console.log('Network Info:', networkInfo);

    // Create a new wallet
    const wallet = await fvchain.createWallet();
    console.log('New Wallet:', wallet);

    // Get wallet balance
    const balance = await fvchain.getWalletBalance(wallet.address);
    console.log('Wallet Balance:', balance);

    // Get latest blocks
    const blocks = await fvchain.getLatestBlocks(5);
    console.log('Latest Blocks:', blocks);

    // Generate device ID for mining
    const deviceId = fvchain.generateDeviceId();
    console.log('Device ID:', deviceId);

    // Get miner status
    const minerStatus = await fvchain.getMinerStatus(deviceId);
    console.log('Miner Status:', minerStatus);

  } catch (error) {
    console.error('SDK Error:', error.message);
    if (error.statusCode) {
      console.error('Status Code:', error.statusCode);
    }
    if (error.code) {
      console.error('Error Code:', error.code);
    }
  }
})();
*/