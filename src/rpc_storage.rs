use crate::storage::{LedgerDB, StorageError};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use once_cell::sync::Lazy;
use serde_json;
use hex;

/// Global LevelDB instance for RPC server
static RPC_DB: Lazy<Arc<LedgerDB>> = Lazy::new(|| {
    let rpc_data_dir = std::env::var("RPC_DATA_DIR").unwrap_or_else(|_| "./data/rpc_storage".to_string());
    Arc::new(LedgerDB::open(&rpc_data_dir).expect("Failed to open RPC storage database"))
});

/// Transaction structure for RPC storage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletTransaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
    pub transaction_type: String,
    pub block_height: u64,
}

/// Block structure for real blockchain storage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub hash: String,
    pub height: u64,
    pub timestamp: u64,
    pub transactions: Vec<WalletTransaction>,
    pub transaction_count: u64,
    pub miner: String,
    pub parent_hash: String,
    pub nonce: u64,
    pub difficulty: u64,
    pub size: u64,
}

impl Block {
    pub fn new(height: u64, miner: String, parent_hash: String) -> Self {
        let timestamp = chrono::Utc::now().timestamp() as u64;
        let hash = format!("0x{:064x}", height + 12345); // Simple hash for now - will be replaced with real hash
        
        Self {
            hash,
            height,
            timestamp,
            transactions: Vec::new(),
            transaction_count: 0,
            miner,
            parent_hash,
            nonce: height * 1000 + 42,
            difficulty: 2,
            size: 1000 + (height * 100),
        }
    }
    
    /// Create block with custom timestamp (for syncing with transaction timestamps)
    pub fn new_with_timestamp(height: u64, miner: String, parent_hash: String, timestamp: u64) -> Self {
        let hash = format!("0x{:064x}", height + 12345); // Simple hash for now - will be replaced with real hash
        
        Self {
            hash,
            height,
            timestamp,
            transactions: Vec::new(),
            transaction_count: 0,
            miner,
            parent_hash,
            nonce: height * 1000 + 42,
            difficulty: 2,
            size: 1000 + (height * 100),
        }
    }
    
    /// Create block with real hash from FractalPoW mining
    pub fn new_with_real_hash(
        height: u64, 
        miner: String, 
        parent_hash: String, 
        real_hash: [u8; 32],
        nonce: u64,
        difficulty: u32
    ) -> Self {
        let timestamp = chrono::Utc::now().timestamp() as u64;
        let hash = format!("0x{}", hex::encode(real_hash));
        
        Self {
            hash,
            height,
            timestamp,
            transactions: Vec::new(),
            transaction_count: 0,
            miner,
            parent_hash,
            nonce,
            difficulty: difficulty as u64,
            size: 1000 + (height * 100),
        }
    }
    
    /// Create block with real hash and custom timestamp (for syncing with transaction timestamps)
    pub fn new_with_real_hash_and_timestamp(
        height: u64, 
        miner: String, 
        parent_hash: String, 
        real_hash: [u8; 32],
        nonce: u64,
        difficulty: u32,
        timestamp: u64
    ) -> Self {
        let hash = format!("0x{}", hex::encode(real_hash));
        
        Self {
            hash,
            height,
            timestamp,
            transactions: Vec::new(),
            transaction_count: 0,
            miner,
            parent_hash,
            nonce,
            difficulty: difficulty as u64,
            size: 1000 + (height * 100),
        }
    }
    
    pub fn add_transaction(&mut self, tx: WalletTransaction) {
        self.transactions.push(tx);
        self.transaction_count = self.transactions.len() as u64;
        // Recalculate size based on transactions
        self.size = 1000 + (self.height * 100) + (self.transaction_count * 200);
    }
    
    /// Verify if the block hash is valid (not a fake hash)
    pub fn is_valid_hash(&self) -> bool {
        // Check if hash is not the fake pattern (0x{:064x} from height + 12345)
        let fake_hash = format!("0x{:064x}", self.height + 12345);
        self.hash != fake_hash && self.hash.len() == 66 && self.hash.starts_with("0x")
    }
    
    /// Get hash as bytes for verification
    pub fn get_hash_bytes(&self) -> Result<[u8; 32], String> {
        if self.hash.len() != 66 || !self.hash.starts_with("0x") {
            return Err("Invalid hash format".to_string());
        }
        
        let hex_str = &self.hash[2..]; // Remove 0x prefix
        match hex::decode(hex_str) {
            Ok(bytes) => {
                if bytes.len() == 32 {
                    let mut hash_bytes = [0u8; 32];
                    hash_bytes.copy_from_slice(&bytes);
                    Ok(hash_bytes)
                } else {
                    Err("Hash must be 32 bytes".to_string())
                }
            }
            Err(_) => Err("Invalid hex encoding".to_string())
        }
    }
}

impl WalletTransaction {
    pub fn new_mining_reward(address: String, amount: u64, hash: String, block_height: u64) -> Self {
        Self {
            hash,
            from: "Mining-Reward".to_string(),
            to: address,
            amount,
            timestamp: chrono::Utc::now().timestamp() as u64,
            transaction_type: "mining_reward".to_string(),
            block_height,
        }
    }
    
    pub fn new_transfer(from: String, to: String, amount: u64, hash: String, block_height: u64) -> Self {
        Self {
            hash,
            from,
            to,
            amount,
            timestamp: chrono::Utc::now().timestamp() as u64,
            transaction_type: "transfer".to_string(),
            block_height,
        }
    }
}

/// RPC Storage Manager - handles all persistent data for RPC server
pub struct RPCStorage;

impl RPCStorage {
    /// Balance operations
    pub async fn get_balance(address: &str) -> Result<u64, StorageError> {
        RPC_DB.get_balance(address).await
    }

    pub async fn set_balance(address: &str, balance: u64) -> Result<(), StorageError> {
        RPC_DB.set_balance(address, balance).await
    }

    pub async fn update_balance(address: &str, delta: i64) -> Result<u64, StorageError> {
        let current = Self::get_balance(address).await?;
        let new_balance = if delta < 0 {
            current.saturating_sub((-delta) as u64)
        } else {
            current + (delta as u64)
        };
        Self::set_balance(address, new_balance).await?;
        Ok(new_balance)
    }

    pub async fn get_device_balance(device_id: &str, address: &str) -> Result<u64, StorageError> {
        let key = format!("device_balance:{}:{}", device_id, address);
        match RPC_DB.get_u64(&key).await {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Ok(0),
            Err(e) => Err(e),
        }
    }

    pub async fn set_device_balance(device_id: &str, address: &str, balance: u64) -> Result<(), StorageError> {
        let key = format!("device_balance:{}:{}", device_id, address);
        RPC_DB.set(&key, balance).await
    }

    // Overload functions for device_id only (uses stored device address)
    pub async fn get_device_balance_by_id(device_id: &str) -> Result<u64, StorageError> {
        if let Some(address) = Self::get_device_address(device_id).await? {
            Self::get_device_balance(device_id, &address).await
        } else {
            Ok(0)
        }
    }

    pub async fn set_device_balance_by_id(device_id: &str, balance: u64) -> Result<(), StorageError> {
        if let Some(address) = Self::get_device_address(device_id).await? {
            Self::set_device_balance(device_id, &address, balance).await
        } else {
            Err(StorageError::NotFound(format!("Device address not found for device_id: {}", device_id)))
        }
    }

    /// Device-specific operations
    pub async fn get_device_mining_status(device_id: &str) -> Result<bool, StorageError> {
        let key = format!("device_mining:{}", device_id);
        match RPC_DB.get(key.as_bytes()).await? {
            Some(bytes) => Ok(bytes[0] == 1),
            None => Ok(false),
        }
    }

    pub async fn set_device_mining_status(device_id: &str, status: bool) -> Result<(), StorageError> {
        let key = format!("device_mining:{}", device_id);
        let value = if status { [1u8] } else { [0u8] };
        RPC_DB.put(key.as_bytes(), &value).await
    }

    pub async fn get_device_session(device_id: &str) -> Result<Option<(String, u64)>, StorageError> {
        let key = format!("device_session:{}", device_id);
        match RPC_DB.get(key.as_bytes()).await? {
            Some(bytes) => {
                let session_data: (String, u64) = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                Ok(Some(session_data))
            },
            None => Ok(None),
        }
    }

    pub async fn set_device_session(device_id: &str, session_id: &str, timestamp: u64) -> Result<(), StorageError> {
        let key = format!("device_session:{}", device_id);
        let session_data = (session_id.to_string(), timestamp);
        let value = serde_json::to_vec(&session_data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        RPC_DB.put(key.as_bytes(), &value).await?;
        
        // Update session keys registry
        let session_keys_key = b"session_keys_registry";
        let mut session_keys: Vec<String> = match RPC_DB.get(session_keys_key).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => Vec::new(),
        };
        
        if !session_keys.contains(&device_id.to_string()) {
            session_keys.push(device_id.to_string());
            let registry_data = serde_json::to_vec(&session_keys)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
            RPC_DB.put(session_keys_key, &registry_data).await?;
        }
        
        Ok(())
    }

    pub async fn remove_device_session(device_id: &str) -> Result<(), StorageError> {
        let key = format!("device_session:{}", device_id);
        RPC_DB.delete(key.as_bytes()).await?;
        
        // Remove from session keys registry
        let session_keys_key = b"session_keys_registry";
        let mut session_keys: Vec<String> = match RPC_DB.get(session_keys_key).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => Vec::new(),
        };
        
        session_keys.retain(|id| id != device_id);
        let registry_data = serde_json::to_vec(&session_keys)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        RPC_DB.put(session_keys_key, &registry_data).await?;
        
        Ok(())
    }

    pub async fn get_device_address(device_id: &str) -> Result<Option<String>, StorageError> {
        let key = format!("device_addr:{}", device_id);
        match RPC_DB.get(key.as_bytes()).await? {
            Some(bytes) => {
                let address = String::from_utf8(bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                Ok(Some(address))
            },
            None => Ok(None),
        }
    }

    pub async fn set_device_address(device_id: &str, address: &str) -> Result<(), StorageError> {
        let key = format!("device_addr:{}", device_id);
        RPC_DB.put(key.as_bytes(), address.as_bytes()).await?;
        
        // Update device IDs registry
        let device_registry_key = b"device_ids_registry";
        let mut device_ids: Vec<String> = match RPC_DB.get(device_registry_key).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => Vec::new(),
        };
        
        if !device_ids.contains(&device_id.to_string()) {
            device_ids.push(device_id.to_string());
            let registry_data = serde_json::to_vec(&device_ids)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
            RPC_DB.put(device_registry_key, &registry_data).await?;
        }
        
        Ok(())
    }

    pub async fn remove_device_address(device_id: &str) -> Result<(), StorageError> {
        let key = format!("device_addr:{}", device_id);
        RPC_DB.delete(key.as_bytes()).await?;
        
        // Remove from device IDs registry
        let device_registry_key = b"device_ids_registry";
        let mut device_ids: Vec<String> = match RPC_DB.get(device_registry_key).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => Vec::new(),
        };
        
        device_ids.retain(|id| id != device_id);
        let registry_data = serde_json::to_vec(&device_ids)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        RPC_DB.put(device_registry_key, &registry_data).await?;
        
        Ok(())
    }

    pub async fn get_device_first_mining(device_id: &str) -> Result<bool, StorageError> {
        let key = format!("device_first_mining:{}", device_id);
        match RPC_DB.get(key.as_bytes()).await? {
            Some(bytes) => Ok(bytes[0] == 1),
            None => Ok(false),
        }
    }

    pub async fn set_device_first_mining(device_id: &str, has_mined: bool) -> Result<(), StorageError> {
        let key = format!("device_first_mining:{}", device_id);
        let value = if has_mined { [1u8] } else { [0u8] };
        RPC_DB.put(key.as_bytes(), &value).await
    }

    /// Wallet data operations
    pub async fn get_wallet_data(device_id: &str) -> Result<Option<serde_json::Value>, StorageError> {
        let key = format!("wallet_data:{}", device_id);
        match RPC_DB.get(key.as_bytes()).await? {
            Some(bytes) => {
                let wallet_data: serde_json::Value = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                Ok(Some(wallet_data))
            },
            None => Ok(None),
        }
    }

    pub async fn set_wallet_data(device_id: &str, wallet_data: &serde_json::Value) -> Result<(), StorageError> {
        let key = format!("wallet_data:{}", device_id);
        let value = serde_json::to_vec(wallet_data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        RPC_DB.put(key.as_bytes(), &value).await
    }

    pub async fn save_device_wallet(device_id: &str, wallet_data: &str) -> Result<(), StorageError> {
        let wallet_json: serde_json::Value = serde_json::from_str(wallet_data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        Self::set_wallet_data(device_id, &wallet_json).await
    }

    pub async fn get_device_wallet(device_id: &str) -> Result<Option<serde_json::Value>, StorageError> {
        Self::get_wallet_data(device_id).await
    }

    pub async fn remove_device_wallet(device_id: &str) -> Result<(), StorageError> {
        let key = format!("wallet_data:{}", device_id);
        RPC_DB.delete(key.as_bytes()).await
    }

    /// Private key operations
    pub async fn get_device_private_key(device_id: &str) -> Result<Option<String>, StorageError> {
        let key = format!("device_private_key:{}", device_id);
        match RPC_DB.get(key.as_bytes()).await? {
            Some(bytes) => {
                let private_key = String::from_utf8(bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                Ok(Some(private_key))
            },
            None => Ok(None),
        }
    }

    pub async fn set_device_private_key(device_id: &str, encrypted_private_key: &str) -> Result<(), StorageError> {
        let key = format!("device_private_key:{}", device_id);
        RPC_DB.put(key.as_bytes(), encrypted_private_key.as_bytes()).await
    }

    pub async fn remove_device_private_key(device_id: &str) -> Result<(), StorageError> {
        let key = format!("device_private_key:{}", device_id);
        RPC_DB.delete(key.as_bytes()).await
    }

    /// Transaction operations
    pub async fn add_transaction(tx: &WalletTransaction) -> Result<(), StorageError> {
        let key = format!("tx:{}", tx.hash);
        
        // Check if transaction already exists
        let _tx_exists = RPC_DB.get(key.as_bytes()).await?.is_some();
        
        let value = serde_json::to_vec(tx)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        RPC_DB.put(key.as_bytes(), &value).await?;
        
        // Update transaction hashes registry
        let tx_registry_key = b"transaction_hashes_registry";
        let mut tx_hashes: Vec<String> = match RPC_DB.get(tx_registry_key).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => Vec::new(),
        };
        
        let is_new_tx = !tx_hashes.contains(&tx.hash);
        if is_new_tx {
            tx_hashes.push(tx.hash.clone());
            let registry_data = serde_json::to_vec(&tx_hashes)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
            RPC_DB.put(tx_registry_key, &registry_data).await?;
            
            // Only increment counter for truly new transactions
            Self::increment_transaction_count().await?;
        }
        
        Ok(())
    }

    pub async fn get_transaction(hash: &str) -> Result<Option<WalletTransaction>, StorageError> {
        let key = format!("tx:{}", hash);
        match RPC_DB.get(key.as_bytes()).await? {
            Some(bytes) => {
                let tx: WalletTransaction = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                Ok(Some(tx))
            },
            None => Ok(None),
        }
    }

    pub async fn get_latest_transactions(limit: usize) -> Result<Vec<WalletTransaction>, StorageError> {
        // Get list of all transaction hashes from registry
        let tx_registry_key = b"transaction_hashes_registry";
        let tx_hashes: Vec<String> = match RPC_DB.get(tx_registry_key).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => Vec::new(),
        };
        
        let mut transactions = Vec::new();
        
        // Get ALL transactions first, then sort and limit
        for hash in tx_hashes {
            if let Ok(Some(tx)) = Self::get_transaction(&hash).await {
                transactions.push(tx);
            }
        }
        
        // Sort by timestamp (newest first)
        transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Take only the latest transactions up to limit
        transactions.truncate(limit);
        Ok(transactions)
    }

    /// Block height operations
    pub async fn get_block_height() -> Result<u64, StorageError> {
        match RPC_DB.get(b"block_height").await? {
            Some(bytes) => {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes[..8.min(bytes.len())]);
                Ok(u64::from_le_bytes(arr))
            },
            None => Ok(1), // Genesis block
        }
    }

    pub async fn set_block_height(height: u64) -> Result<(), StorageError> {
        RPC_DB.put(b"block_height", &height.to_le_bytes()).await
    }

    pub async fn increment_block_height() -> Result<u64, StorageError> {
        let current = Self::get_block_height().await?;
        let new_height = current + 1;
        Self::set_block_height(new_height).await?;
        Ok(new_height)
    }

    /// Transaction count operations
    pub async fn get_transaction_count() -> Result<u64, StorageError> {
        // Get count from registry for accuracy
        let tx_registry_key = b"transaction_hashes_registry";
        match RPC_DB.get(tx_registry_key).await? {
            Some(data) => {
                let tx_hashes: Vec<String> = serde_json::from_slice(&data).unwrap_or_default();
                Ok(tx_hashes.len() as u64)
            },
            None => Ok(0),
        }
    }

    pub async fn set_transaction_count(count: u64) -> Result<(), StorageError> {
        RPC_DB.put(b"transaction_count", &count.to_le_bytes()).await
    }

    pub async fn increment_transaction_count() -> Result<u64, StorageError> {
        let current = Self::get_transaction_count().await?;
        let new_count = current + 1;
        Self::set_transaction_count(new_count).await?;
        Ok(new_count)
    }

    /// Device registration operations
    pub async fn get_device_registration(device_id: &str) -> Result<Option<serde_json::Value>, StorageError> {
        let key = format!("device_reg:{}", device_id);
        match RPC_DB.get(key.as_bytes()).await? {
            Some(bytes) => {
                let reg_data: serde_json::Value = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                Ok(Some(reg_data))
            },
            None => Ok(None),
        }
    }

    pub async fn set_device_registration(device_id: &str, reg_data: &serde_json::Value) -> Result<(), StorageError> {
        let key = format!("device_reg:{}", device_id);
        let value = serde_json::to_vec(reg_data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        RPC_DB.put(key.as_bytes(), &value).await
    }

    pub async fn remove_device_registration(device_id: &str) -> Result<(), StorageError> {
        let key = format!("device_reg:{}", device_id);
        RPC_DB.delete(key.as_bytes()).await
    }

    /// Get all device addresses that are currently mining
    pub async fn get_all_mining_devices() -> Result<Vec<String>, StorageError> {
        // Get list of all device IDs from registry
        let device_registry_key = b"device_ids_registry";
        let device_ids: Vec<String> = match RPC_DB.get(device_registry_key).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => Vec::new(),
        };
        
        let mut mining_devices = Vec::new();
        
        for device_id in device_ids {
            // Check if device is currently mining
            if let Ok(is_mining) = Self::get_device_mining_status(&device_id).await {
                if is_mining {
                    // Get the address for this device
                    if let Ok(Some(address)) = Self::get_device_address(&device_id).await {
                        mining_devices.push(address);
                    }
                }
            }
        }
        
        Ok(mining_devices)
    }

    /// Get all device addresses mapped by device_id
    pub async fn get_all_device_addresses() -> Result<std::collections::HashMap<String, String>, StorageError> {
        // Get list of all device IDs from registry
        let device_registry_key = b"device_ids_registry";
        let device_ids: Vec<String> = match RPC_DB.get(device_registry_key).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => Vec::new(),
        };
        
        let mut device_addresses = std::collections::HashMap::new();
        
        for device_id in device_ids {
            if let Ok(Some(address)) = Self::get_device_address(&device_id).await {
                device_addresses.insert(device_id, address);
            }
        }
        
        Ok(device_addresses)
    }

    /// Get device_id by wallet address (reverse lookup)
    pub async fn get_device_id_by_address(address: &str) -> Result<Option<String>, StorageError> {
        // Get list of all device IDs from registry
        let device_registry_key = b"device_ids_registry";
        let device_ids: Vec<String> = match RPC_DB.get(device_registry_key).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => Vec::new(),
        };
        
        // Search through all device IDs to find the one with matching address
        for device_id in device_ids {
            if let Ok(Some(device_address)) = Self::get_device_address(&device_id).await {
                if device_address == address {
                    return Ok(Some(device_id));
                }
            }
        }
        
        Ok(None)
    }

    /// Get session registry
    pub async fn get_session_registry() -> Result<Vec<String>, StorageError> {
        let session_keys_key = b"session_keys_registry";
        let session_keys: Vec<String> = match RPC_DB.get(session_keys_key).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => Vec::new(),
        };
        Ok(session_keys)
    }

    /// Get all active devices that have sessions
    pub async fn get_all_active_devices() -> Result<Vec<String>, StorageError> {
        // Get list of all device session keys from registry
        let session_keys = Self::get_session_registry().await?;
        
        let mut active_devices = Vec::new();
        
        for device_id in session_keys {
            // Check if device has an active session
            if let Ok(Some(_)) = Self::get_device_session(&device_id).await {
                active_devices.push(device_id);
            }
        }
        
        Ok(active_devices)
    }

    /// Device PIN management operations
    pub async fn get_device_pin(device_id: &str) -> Result<String, StorageError> {
        let key = format!("device_pin:{}", device_id);
        match RPC_DB.get(key.as_bytes()).await? {
            Some(bytes) => {
                let pin_hash = String::from_utf8(bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                Ok(pin_hash)
            },
            None => Err(StorageError::NotFound(format!("PIN not found for device: {}", device_id))),
        }
    }

    pub async fn set_device_pin(device_id: &str, pin_hash: &str) -> Result<(), StorageError> {
        let key = format!("device_pin:{}", device_id);
        RPC_DB.put(key.as_bytes(), pin_hash.as_bytes()).await
    }

    pub async fn remove_device_pin(device_id: &str) -> Result<(), StorageError> {
        let key = format!("device_pin:{}", device_id);
        RPC_DB.delete(key.as_bytes()).await
    }

    pub async fn get_device_failed_attempts(device_id: &str) -> Result<u32, StorageError> {
        let key = format!("device_failed_attempts:{}", device_id);
        match RPC_DB.get_u64(&key).await {
            Ok(Some(value)) => Ok(value as u32),
            Ok(None) => Ok(0),
            Err(e) => Err(e),
        }
    }

    pub async fn set_device_failed_attempts(device_id: &str, attempts: u32) -> Result<(), StorageError> {
        let key = format!("device_failed_attempts:{}", device_id);
        RPC_DB.set(&key, attempts as u64).await
    }

    pub async fn get_device_lockout(device_id: &str) -> Result<u64, StorageError> {
        let key = format!("device_lockout:{}", device_id);
        match RPC_DB.get_u64(&key).await {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Ok(0),
            Err(e) => Err(e),
        }
    }

    pub async fn set_device_lockout(device_id: &str, lockout_time: u64) -> Result<(), StorageError> {
        let key = format!("device_lockout:{}", device_id);
        RPC_DB.set(&key, lockout_time).await
    }

    /// Cleanup operations
    pub async fn cleanup_old_sessions(max_age_seconds: u64) -> Result<u64, StorageError> {
        let current_time = chrono::Utc::now().timestamp() as u64;
        let cutoff_time = current_time.saturating_sub(max_age_seconds);
        
        // Get list of all device session keys from a registry
        let session_keys_key = b"session_keys_registry";
        let session_keys: Vec<String> = match RPC_DB.get(session_keys_key).await? {
            Some(data) => serde_json::from_slice(&data).unwrap_or_default(),
            None => Vec::new(),
        };
        
        let mut keys_to_delete = Vec::new();
        let mut remaining_keys = Vec::new();
        
        for device_id in session_keys {
            let session_key = format!("device_session:{}", device_id);
            if let Ok(Some((_, timestamp))) = Self::get_device_session(&device_id).await {
                if timestamp < cutoff_time {
                    keys_to_delete.push(session_key);
                } else {
                    remaining_keys.push(device_id);
                }
            }
        }
        
        // Delete expired sessions
        let cleaned_count = keys_to_delete.len() as u64;
        for key in &keys_to_delete {
            RPC_DB.delete(key.as_bytes()).await?;
        }
        
        // Update the registry with remaining keys
        let remaining_data = serde_json::to_vec(&remaining_keys)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        RPC_DB.put(session_keys_key, &remaining_data).await?;
        
        Ok(cleaned_count)
    }

    /// Block storage operations
    pub async fn store_block(block: &Block) -> Result<(), StorageError> {
        let key = format!("block:{}", block.height);
        let serialized = serde_json::to_string(block)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        RPC_DB.put(key.as_bytes(), serialized.as_bytes()).await?;
        
        // Also store each transaction individually
        for tx in &block.transactions {
            Self::add_transaction(tx).await?;
        }
        
        Ok(())
    }

    pub async fn get_block_by_height(height: u64) -> Result<Option<Block>, StorageError> {
        let key = format!("block:{}", height);
        match RPC_DB.get(key.as_bytes()).await? {
            Some(data) => {
                let block: Block = serde_json::from_slice(&data)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                Ok(Some(block))
            },
            None => Ok(None),
        }
    }

    pub async fn get_blocks_range(start_height: u64, end_height: u64) -> Result<Vec<Block>, StorageError> {
        let mut blocks = Vec::new();
        
        for height in start_height..=end_height {
            if let Some(block) = Self::get_block_by_height(height).await? {
                blocks.push(block);
            }
        }
        
        // Sort by height descending (latest first)
        blocks.sort_by(|a, b| b.height.cmp(&a.height));
        Ok(blocks)
    }

    pub async fn get_latest_blocks(limit: usize) -> Result<Vec<Block>, StorageError> {
        let latest_height = Self::get_block_height().await?;
        let mut blocks = Vec::new();
        
        // Search backwards from latest height to find existing blocks
        let mut current_height = latest_height;
        let mut found_count = 0;
        
        while found_count < limit && current_height > 0 {
            if let Ok(Some(block)) = Self::get_block_by_height(current_height).await {
                blocks.push(block);
                found_count += 1;
            }
            current_height = current_height.saturating_sub(1);
        }
        
        // Always include genesis block if we haven't reached the limit
        if found_count < limit {
            if let Ok(Some(genesis_block)) = Self::get_block_by_height(0).await {
                blocks.push(genesis_block);
            }
        }
        
        // Sort by height descending (latest first)
        blocks.sort_by(|a, b| b.height.cmp(&a.height));
        Ok(blocks)
    }



    pub async fn create_genesis_block() -> Result<(), StorageError> {
        // Check if genesis block already exists
        if Self::get_block_by_height(0).await?.is_some() {
            return Ok(()); // Genesis block already exists
        }

        // Load mainnet genesis configuration
        let genesis_config_path = "mainnet-genesis.json";
        if std::path::Path::new(genesis_config_path).exists() {
            match std::fs::read_to_string(genesis_config_path) {
                Ok(config_data) => {
                    if let Ok(genesis_config) = serde_json::from_str::<serde_json::Value>(&config_data) {
                        // Initialize ecosystem wallets with genesis allocations
                        if let Some(alloc) = genesis_config["alloc"].as_object() {
                            for (address, allocation) in alloc {
                                if let Some(balance_str) = allocation["balance"].as_str() {
                                    // Convert from wei (18 decimals) to microFVC (6 decimals)
                                    if let Ok(balance_wei) = balance_str.parse::<u128>() {
                                        let balance_fvc = (balance_wei / 1_000_000_000_000u128) as u64; // Convert wei to microFVC
                                        if let Err(e) = Self::set_balance(address, balance_fvc).await {
                                            println!("Warning: Failed to set genesis balance for {}: {}", address, e);
                                        } else {
                                            println!("✅ Genesis allocation: {} = {} FVC", address, balance_fvc as f64 / 1_000_000.0);
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Err(e) => println!("Warning: Failed to read genesis config: {}", e)
            }
        }

        // Create consistent timestamp for genesis block and transaction
        let genesis_timestamp = chrono::Utc::now().timestamp() as u64;
        
        let mut genesis_block = Block::new_with_timestamp(
            0,
            "Genesis".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            genesis_timestamp
        );

        // Add genesis transaction with consistent timestamp
        let genesis_tx = WalletTransaction {
            hash: "genesis".to_string(),
            from: "Genesis".to_string(),
            to: "Genesis".to_string(),
            amount: 0,
            timestamp: genesis_timestamp, // Use consistent timestamp
            transaction_type: "genesis".to_string(),
            block_height: 0,
        };
        
        genesis_block.add_transaction(genesis_tx);
        Self::store_block(&genesis_block).await
    }

    /// Get network information
    pub async fn get_network_info() -> Result<serde_json::Value, StorageError> {
        let block_height = Self::get_block_height().await.unwrap_or(1);
        let transaction_count = Self::get_transaction_count().await.unwrap_or(0);
        
        // Get real active nodes count from cached cluster health
        let active_nodes = crate::node_health::get_active_nodes_count();
        
        // Calculate Smart Rate and vPoW indicators
        let smart_rate = Self::calculate_smart_rate(block_height, transaction_count).await;
        let vortex_energy_rate = Self::calculate_vortex_energy_rate(block_height, transaction_count).await;
        let fractal_contribution_score = Self::calculate_fractal_contribution_score(block_height, transaction_count).await;
        let mathematical_efficiency_index = Self::calculate_mathematical_efficiency_index(block_height).await;
        let network_harmony_factor = Self::calculate_network_harmony_factor(block_height, transaction_count).await;
        
        Ok(serde_json::json!({
            "latest_block_height": block_height,
            "active_nodes": active_nodes,
            "total_supply": 3600900000u64,
            "circulating_supply": 3583900000u64,
            "transaction_count": transaction_count,
            "avg_block_time": 5,
            "network_smart_rate": smart_rate,
            "avg_vortex_energy_rate": vortex_energy_rate,
            "avg_fractal_contribution_score": fractal_contribution_score,
            "mathematical_efficiency_index": mathematical_efficiency_index,
            "network_harmony_factor": network_harmony_factor
        }))
    }

    /// Get network statistics
    pub async fn get_stats() -> Result<serde_json::Value, StorageError> {
        let block_height = Self::get_block_height().await.unwrap_or(1);
        let transaction_count = Self::get_transaction_count().await.unwrap_or(0);
        
        // Get real active miners count from active devices
        let active_miners = match Self::get_all_active_devices().await {
            Ok(devices) => {
                let count = devices.len() as u32;
                if count > 0 { count } else { 1 }
            },
            Err(_) => 1
        };
        
        let smart_rate = Self::calculate_smart_rate(block_height, transaction_count).await;
        
        Ok(serde_json::json!({
            "blocks_mined": block_height,
            "total_transactions": transaction_count,
            "network_smart_rate": smart_rate,
            "smart_rate_unit": "ss/s",
            "difficulty": 1000000,
            "active_miners": active_miners,
            "last_block_time": chrono::Utc::now().timestamp()
        }))
    }

    /// Calculate Vortex Energy Rate (VER)
    async fn calculate_vortex_energy_rate(block_height: u64, transaction_count: u64) -> f64 {
        if block_height == 0 {
            return 0.0;
        }
        
        // Base transaction throughput
        let throughput = transaction_count as f64 / block_height as f64;
        
        // Energy efficiency factor (simulated)
        let energy_efficiency = 0.85; // 85% efficiency
        
        // Network load factor
        let network_load = (throughput / 10.0).min(1.0); // Normalize to max 10 tx/block
        
        // Vortex Energy Rate calculation
        let ver = throughput * energy_efficiency * (1.0 + network_load) * 100.0;
        
        // Ensure minimum value and round
        ver.max(1.0).min(1000.0)
    }

    /// Calculate Fractal Contribution Score (FCS)
    async fn calculate_fractal_contribution_score(block_height: u64, transaction_count: u64) -> f64 {
        if block_height <= 1 {
            return 0.0;
        }
        
        // Fractal depth based on block height
        let fractal_depth = (block_height as f64).log2().floor();
        
        // Transaction density
        let tx_density = transaction_count as f64 / block_height as f64;
        
        // Fractal pattern strength
        let pattern_strength = (fractal_depth / 20.0).min(1.0); // Normalize to max depth 20
        
        // Contribution score calculation
        let fcs = tx_density * (1.0 + pattern_strength) * fractal_depth * 10.0;
        
        // Ensure minimum value and cap at 1000
        fcs.max(1.0).min(1000.0)
    }

    /// Calculate Mathematical Efficiency Index (MEI)
    async fn calculate_mathematical_efficiency_index(block_height: u64) -> f64 {
        let transaction_count = Self::get_transaction_count().await.unwrap_or(0);
        
        if block_height == 0 {
            return 0.0;
        }
        
        // Processing efficiency
        let throughput = transaction_count as f64 / block_height as f64;
        
        // Mathematical constants
        let golden_ratio = 1.618033988749; // φ (phi)
        let euler_number = std::f64::consts::E; // e
        
        // Computational complexity factor
        let complexity_factor = (block_height as f64).log10() / 10.0; // Normalize log scale
        
        // Mathematical efficiency calculation
        let mei = throughput * golden_ratio * (1.0 + complexity_factor) * euler_number * 10.0;
        
        // Ensure minimum value and cap at 1000
        mei.max(1.0).min(1000.0)
    }

    /// Calculate Network Harmony Factor (NHF)
    async fn calculate_network_harmony_factor(block_height: u64, transaction_count: u64) -> f64 {
        if block_height == 0 {
            return 0.0;
        }
        
        // Block production consistency
        let block_consistency = if block_height > 10 {
            let recent_blocks = std::cmp::min(block_height, 100);
            recent_blocks as f64 / 100.0
        } else {
            block_height as f64 / 10.0
        };
        
        // Transaction flow smoothness
        let tx_flow = (transaction_count as f64 / block_height as f64) / 10.0; // Normalize to expected 10 tx/block
        let transaction_smoothness = tx_flow.min(1.0);
        
        // Network synchronization (simulated)
        let sync_factor = 0.95; // 95% network sync
        
        // Harmony calculation with weighted average
        let harmony = (block_consistency * 0.4 + transaction_smoothness * 0.4 + sync_factor * 0.2) * 1000.0;
        
        // Ensure minimum value and cap at 1000
        harmony.max(1.0).min(1000.0)
    }

    /// Calculate Smart Rate
    async fn calculate_smart_rate(block_height: u64, transaction_count: u64) -> f64 {
        // Calculate individual components
        let vortex_energy = Self::calculate_vortex_energy_rate(block_height, transaction_count).await;
        let fractal_score = Self::calculate_fractal_contribution_score(block_height, transaction_count).await;
        let efficiency_index = Self::calculate_mathematical_efficiency_index(block_height).await;
        let harmony_factor = Self::calculate_network_harmony_factor(block_height, transaction_count).await;
        
        // Normalize components to 0-1 range
        let normalized_ver = (vortex_energy / 1000.0).min(1.0);
        let normalized_fcs = (fractal_score / 1000.0).min(1.0);
        let normalized_mei = (efficiency_index / 1000.0).min(1.0);
        let normalized_nhf = (harmony_factor / 1000.0).min(1.0);
        
        // Weights for each component (sum = 1.0)
        let w_ver = 0.30; // Vortex Energy Rate weight
        let w_fcs = 0.25; // Fractal Contribution Score weight
        let w_mei = 0.25; // Mathematical Efficiency Index weight
        let w_nhf = 0.20; // Network Harmony Factor weight
        
        // Calculate weighted geometric mean
        let weighted_geometric_mean = (normalized_ver.powf(w_ver) * 
                                     normalized_fcs.powf(w_fcs) * 
                                     normalized_mei.powf(w_mei) * 
                                     normalized_nhf.powf(w_nhf)).max(0.001);
        
        // Base Smart Rate (in Smart Steps per Second)
        let base_smart_rate = 1000.0;
        
        // Vortex Pattern based on block height
        let vortex_pattern = 1.0 + (0.1 * ((block_height as f64 * 0.618).sin() + 1.0));
        
        // Final Smart Rate calculation
        let smart_rate = base_smart_rate * weighted_geometric_mean * vortex_pattern;
        
        // Round to 2 decimal places
        (smart_rate * 100.0).round() / 100.0
    }
}