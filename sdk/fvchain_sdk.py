#!/usr/bin/env python3
"""
FVChain SDK - Python Client Library

Official SDK for integrating with Fractal Vortex Chain (FVChain)
Provides easy-to-use methods for blockchain interactions

Version: 1.0.0
Author: FVChain Development Team
"""

import requests
import json
import time
import random
import string
from typing import Dict, List, Optional, Union
from dataclasses import dataclass


class FVChainError(Exception):
    """Custom exception class for FVChain SDK errors"""
    
    def __init__(self, message: str, status_code: int = 0, error_code: str = "UNKNOWN_ERROR"):
        super().__init__(message)
        self.message = message
        self.status_code = status_code
        self.error_code = error_code


@dataclass
class NetworkInfo:
    """Network information data structure"""
    latest_block_height: int
    active_nodes: int
    network_id: str
    chain_id: str


@dataclass
class WalletInfo:
    """Wallet information data structure"""
    address: str
    private_key: Optional[str] = None
    balance: Optional[float] = None


@dataclass
class BlockInfo:
    """Block information data structure"""
    height: int
    hash: str
    timestamp: int
    transactions_count: int
    miner: str


@dataclass
class TransactionInfo:
    """Transaction information data structure"""
    hash: str
    from_address: str
    to_address: str
    amount: float
    fee: float
    timestamp: int
    block_height: int


class FVChainSDK:
    """FVChain SDK main class for blockchain interactions"""
    
    def __init__(self, base_url: str = "http://localhost:8080", timeout: int = 10, headers: Optional[Dict] = None):
        """
        Initialize FVChain SDK
        
        Args:
            base_url (str): Base URL for FVChain RPC server
            timeout (int): Request timeout in seconds
            headers (Dict, optional): Additional headers for requests
        """
        self.base_url = base_url.rstrip('/')
        self.timeout = timeout
        self.session = requests.Session()
        
        # Set default headers
        default_headers = {
            'Content-Type': 'application/json',
            'User-Agent': 'FVChain-Python-SDK/1.0.0'
        }
        
        if headers:
            default_headers.update(headers)
            
        self.session.headers.update(default_headers)
    
    def _make_request(self, method: str, endpoint: str, data: Optional[Dict] = None, params: Optional[Dict] = None) -> Dict:
        """
        Make HTTP request to FVChain server
        
        Args:
            method (str): HTTP method (GET, POST, etc.)
            endpoint (str): API endpoint
            data (Dict, optional): Request body data
            params (Dict, optional): Query parameters
            
        Returns:
            Dict: Response data
            
        Raises:
            FVChainError: If request fails
        """
        url = f"{self.base_url}{endpoint}"
        
        try:
            response = self.session.request(
                method=method,
                url=url,
                json=data,
                params=params,
                timeout=self.timeout
            )
            
            # Check if request was successful
            if response.status_code >= 400:
                try:
                    error_data = response.json()
                    error_message = error_data.get('message', f'HTTP {response.status_code} Error')
                    error_code = error_data.get('code', 'HTTP_ERROR')
                except (json.JSONDecodeError, ValueError):
                    error_message = f'HTTP {response.status_code} Error'
                    error_code = 'HTTP_ERROR'
                
                raise FVChainError(error_message, response.status_code, error_code)
            
            return response.json()
            
        except requests.exceptions.Timeout:
            raise FVChainError("Request timeout", 0, "TIMEOUT_ERROR")
        except requests.exceptions.ConnectionError:
            raise FVChainError("Network error: Unable to connect to FVChain server", 0, "NETWORK_ERROR")
        except requests.exceptions.RequestException as e:
            raise FVChainError(f"Request error: {str(e)}", 0, "REQUEST_ERROR")
        except json.JSONDecodeError:
            raise FVChainError("Invalid JSON response from server", 0, "JSON_DECODE_ERROR")
    
    # ==================== NETWORK METHODS ====================
    
    def get_network_info(self) -> NetworkInfo:
        """
        Get network information
        
        Returns:
            NetworkInfo: Network information object
        """
        data = self._make_request('GET', '/network/info')
        return NetworkInfo(
            latest_block_height=data.get('latest_block_height', 0),
            active_nodes=data.get('active_nodes', 0),
            network_id=data.get('network_id', ''),
            chain_id=data.get('chain_id', '')
        )
    
    def get_network_nodes(self) -> Dict:
        """
        Get network nodes
        
        Returns:
            Dict: Active network nodes data
        """
        return self._make_request('GET', '/network/nodes')
    
    # ==================== BLOCK METHODS ====================
    
    def get_latest_blocks(self, limit: int = 10) -> List[Dict]:
        """
        Get latest blocks
        
        Args:
            limit (int): Number of blocks to retrieve (default: 10, max: 100)
            
        Returns:
            List[Dict]: Latest blocks data
        """
        if limit > 100:
            limit = 100
        
        data = self._make_request('GET', '/blocks/latest', params={'limit': limit})
        return data.get('blocks', [])
    
    def get_block_by_height(self, height: int) -> Dict:
        """
        Get block by height
        
        Args:
            height (int): Block height
            
        Returns:
            Dict: Block data
        """
        return self._make_request('GET', f'/blocks/{height}')
    
    def get_current_block_height(self) -> int:
        """
        Get current block height
        
        Returns:
            int: Current block height
        """
        network_info = self.get_network_info()
        return network_info.latest_block_height
    
    # ==================== TRANSACTION METHODS ====================
    
    def get_latest_transactions(self, limit: int = 10) -> List[Dict]:
        """
        Get latest transactions
        
        Args:
            limit (int): Number of transactions to retrieve (default: 10, max: 200)
            
        Returns:
            List[Dict]: Latest transactions data
        """
        if limit > 200:
            limit = 200
        
        data = self._make_request('GET', '/transactions', params={'limit': limit})
        return data.get('transactions', [])
    
    def get_transaction_by_hash(self, tx_hash: str) -> Dict:
        """
        Get transaction by hash
        
        Args:
            tx_hash (str): Transaction hash
            
        Returns:
            Dict: Transaction data
        """
        return self._make_request('GET', f'/transactions/{tx_hash}')
    
    # ==================== WALLET METHODS ====================
    
    def create_wallet(self) -> WalletInfo:
        """
        Create a new wallet
        
        Returns:
            WalletInfo: New wallet information
        """
        data = self._make_request('GET', '/wallet/create')
        return WalletInfo(
            address=data.get('address', ''),
            private_key=data.get('private_key', '')
        )
    
    def get_wallet_balance(self, address: str) -> WalletInfo:
        """
        Get wallet balance
        
        Args:
            address (str): Wallet address
            
        Returns:
            WalletInfo: Wallet information with balance
        """
        data = self._make_request('POST', '/wallet/balance', data={'address': address})
        return WalletInfo(
            address=data.get('address', address),
            balance=data.get('balance', 0.0)
        )
    
    def validate_address(self, address: str) -> bool:
        """
        Validate wallet address format
        
        Args:
            address (str): Wallet address to validate
            
        Returns:
            bool: True if address format is valid
        """
        import re
        # FVChain addresses start with 'fvc', followed by 36 hex chars, and end with 'emyl' (43 chars total)
        address_pattern = r'^fvc[0-9a-fA-F]{36}emyl$'
        return bool(re.match(address_pattern, address))
    
    # ==================== MINING METHODS ====================
    
    def get_miner_status(self, device_id: str) -> Dict:
        """
        Get miner status
        
        Args:
            device_id (str): Device ID
            
        Returns:
            Dict: Miner status data
        """
        return self._make_request('GET', '/miner/status', params={'device_id': device_id})
    
    def send_mining_heartbeat(self, device_id: str, session_token: str, timestamp: Optional[int] = None) -> Dict:
        """
        Send mining heartbeat
        
        Args:
            device_id (str): Device ID
            session_token (str): Session token
            timestamp (int, optional): Current timestamp (defaults to current time)
            
        Returns:
            Dict: Heartbeat response
        """
        if timestamp is None:
            timestamp = int(time.time())
        
        data = {
            'device_id': device_id,
            'session_token': session_token,
            'timestamp': timestamp
        }
        
        return self._make_request('POST', '/api/v1/mining/heartbeat', data=data)
    
    def get_mining_stats(self) -> Dict:
        """
        Get mining detection statistics
        
        Returns:
            Dict: Mining detection stats
        """
        return self._make_request('GET', '/mining/detection/stats')
    
    # ==================== ADMIN METHODS ====================
    
    def get_rate_limit_stats(self) -> Dict:
        """
        Get rate limit statistics (admin only)
        
        Returns:
            Dict: Rate limit statistics
        """
        return self._make_request('GET', '/admin/rate-limit/stats')
    
    # ==================== UTILITY METHODS ====================
    
    def generate_device_id(self) -> str:
        """
        Generate a unique device ID
        
        Returns:
            str: Unique device ID
        """
        timestamp = str(int(time.time()))
        random_str = ''.join(random.choices(string.ascii_lowercase + string.digits, k=9))
        return f"device_{timestamp}_{random_str}"
    
    def generate_session_token(self) -> str:
        """
        Generate a session token
        
        Returns:
            str: Session token
        """
        timestamp = str(int(time.time()))
        random_str = ''.join(random.choices(string.ascii_lowercase + string.digits, k=15))
        return f"session_{timestamp}_{random_str}"
    
    def to_smallest_unit(self, fvc_amount: float) -> str:
        """
        Convert FVC amount to smallest unit (similar to wei in Ethereum)
        
        Args:
            fvc_amount (float): Amount in FVC
            
        Returns:
            str: Amount in smallest unit
        """
        return str(int(fvc_amount * 1e18))
    
    def from_smallest_unit(self, smallest_unit: Union[str, int]) -> float:
        """
        Convert from smallest unit to FVC
        
        Args:
            smallest_unit (Union[str, int]): Amount in smallest unit
            
        Returns:
            float: Amount in FVC
        """
        return float(smallest_unit) / 1e18
    
    def is_server_reachable(self) -> bool:
        """
        Check if the FVChain server is reachable
        
        Returns:
            bool: True if server is reachable
        """
        try:
            self.get_network_info()
            return True
        except FVChainError:
            return False
    
    def get_server_health(self) -> Dict:
        """
        Get server health status
        
        Returns:
            Dict: Server health information
        """
        try:
            network_info = self.get_network_info()
            is_reachable = self.is_server_reachable()
            
            return {
                'status': 'healthy',
                'reachable': is_reachable,
                'block_height': network_info.latest_block_height,
                'active_nodes': network_info.active_nodes,
                'timestamp': time.strftime('%Y-%m-%dT%H:%M:%S.%fZ')
            }
        except FVChainError as e:
            return {
                'status': 'unhealthy',
                'reachable': False,
                'error': e.message,
                'timestamp': time.strftime('%Y-%m-%dT%H:%M:%S.%fZ')
            }


# Example usage
if __name__ == "__main__":
    # Initialize SDK
    fvchain = FVChainSDK(
        base_url="http://localhost:8080",
        timeout=15
    )
    
    try:
        # Check server health
        health = fvchain.get_server_health()
        print(f"Server Health: {health}")
        
        # Get network information
        network_info = fvchain.get_network_info()
        print(f"Network Info: {network_info}")
        
        # Create a new wallet
        wallet = fvchain.create_wallet()
        print(f"New Wallet: {wallet}")
        
        # Get wallet balance
        balance = fvchain.get_wallet_balance(wallet.address)
        print(f"Wallet Balance: {balance}")
        
        # Get latest blocks
        blocks = fvchain.get_latest_blocks(5)
        print(f"Latest Blocks: {len(blocks)} blocks retrieved")
        
        # Generate device ID for mining
        device_id = fvchain.generate_device_id()
        print(f"Device ID: {device_id}")
        
        # Get mining stats
        mining_stats = fvchain.get_mining_stats()
        print(f"Mining Stats: {mining_stats}")
        
    except FVChainError as e:
        print(f"SDK Error: {e.message}")
        if e.status_code:
            print(f"Status Code: {e.status_code}")
        if e.error_code:
            print(f"Error Code: {e.error_code}")
    except Exception as e:
        print(f"Unexpected Error: {str(e)}")