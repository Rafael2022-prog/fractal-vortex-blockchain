use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::{Options, WriteOptions, ReadOptions};
// Iterator imports removed - not currently used
use std::path::Path;
use thiserror::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error type for storage operations
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("LevelDB error: {0}")]
    LevelDB(#[from] leveldb::error::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

/// Simple ledger/UTXO storage backed by LevelDB
pub struct LedgerDB {
    db: Arc<RwLock<Database<i32>>>, // protected DB for async context
}

impl LedgerDB {
    /// Open or create a LevelDB instance at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut opts = Options::new();
        opts.create_if_missing = true;
        let db = Database::open(path.as_ref(), opts)?;
        Ok(Self {
            db: Arc::new(RwLock::new(db)),
        })
    }

    /// Convert byte key to i32 key for LevelDB
    fn bytes_to_key(key: &[u8]) -> i32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as i32
    }

    /// Put arbitrary key/value pair
    pub async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let db = self.db.write().await;
        let write_opts = WriteOptions::new();
        let key_i32 = Self::bytes_to_key(key);
        db.put(write_opts, key_i32, value)?;
        Ok(())
    }

    /// Get arbitrary value
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let db = self.db.read().await;
        let read_opts = ReadOptions::new();
        let key_i32 = Self::bytes_to_key(key);
        match db.get(read_opts, key_i32)? {
            Some(value) => Ok(Some(value)),
            None => Ok(None),
        }
    }

    /// Set a string key with u64 value
    pub async fn set(&self, key: &str, value: u64) -> Result<(), StorageError> {
        let key_bytes = key.as_bytes();
        let value_bytes = value.to_le_bytes();
        self.put(key_bytes, &value_bytes).await
    }

    /// Get u64 value by string key
    pub async fn get_u64(&self, key: &str) -> Result<Option<u64>, StorageError> {
        let key_bytes = key.as_bytes();
        match self.get(key_bytes).await? {
            Some(bytes) => {
                if bytes.len() == 8 {
                    let mut array = [0u8; 8];
                    array.copy_from_slice(&bytes);
                    Ok(Some(u64::from_le_bytes(array)))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Delete key
    pub async fn delete(&self, key: &[u8]) -> Result<(), StorageError> {
        let db = self.db.write().await;
        let write_opts = WriteOptions::new();
        let key_i32 = Self::bytes_to_key(key);
        db.delete(write_opts, key_i32)?;
        Ok(())
    }

    /// Convenience helpers -------------------------------------------------
    /// Wallet balance helpers
    pub async fn set_balance(&self, address: &str, balance: u64) -> Result<(), StorageError> {
        self.put(address.as_bytes(), &balance.to_le_bytes()).await
    }

    pub async fn get_balance(&self, address: &str) -> Result<u64, StorageError> {
        match self.get(address.as_bytes()).await? {
            Some(bytes) => {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes[..8.min(bytes.len())]);
                Ok(u64::from_le_bytes(arr))
            },
            None => Ok(0),
        }
    }

    // --------------------------------------------------------------------
    // Block storage helpers

    /// Store a raw serialized block by its hash
    pub async fn put_block(&self, hash: &[u8; 32], block_bytes: &[u8]) -> Result<(), StorageError> {
        let key = Self::block_key(hash);
        self.put(&key, block_bytes).await
    }

    /// Retrieve a raw serialized block by its hash
    pub async fn get_block(&self, hash: &[u8; 32]) -> Result<Option<Vec<u8>>, StorageError> {
        let key = Self::block_key(hash);
        self.get(&key).await
    }

    /// Map block height -> block hash (little-endian height key)
    pub async fn set_height_index(&self, height: u64, hash: &[u8; 32]) -> Result<(), StorageError> {
        self.put(&Self::height_key(height), hash).await
    }

    /// Fetch block hash from height index
    pub async fn get_hash_by_height(&self, height: u64) -> Result<Option<[u8; 32]>, StorageError> {
        match self.get(&Self::height_key(height)).await? {
            Some(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(Some(arr))
            },
            _ => Ok(None),
        }
    }

    /// Helper: prefix keys
    fn block_key(hash: &[u8; 32]) -> Vec<u8> {
        // "b:" prefix distinguishes block records
        // Safety: &[u8;34] lives long enough because function returns during call usage
        let mut v = Vec::with_capacity(34);
        v.extend_from_slice(b"b:");
        v.extend_from_slice(hash);
        v
    }

    fn height_key(height: u64) -> Vec<u8> {
        let mut v = Vec::with_capacity(10);
        v.extend_from_slice(b"h:");
        v.extend_from_slice(&height.to_le_bytes());
        v
    }

    /// Get the latest block height from LevelDB
    /// Update latest block height
    pub async fn set_latest_block_height(&self, height: u64) -> Result<(), StorageError> {
        let height_key = b"latest_height";
        let height_bytes = height.to_le_bytes();
        self.put(height_key, &height_bytes).await
    }

    pub async fn get_latest_block_height(&self) -> Result<u64, StorageError> {
        // Use a dedicated key for latest block height
        let height_key = b"latest_height";
        match self.get(height_key).await? {
            Some(height_bytes) => {
                if height_bytes.len() == 8 {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&height_bytes);
                    Ok(u64::from_le_bytes(arr))
                } else {
                    Ok(0)
                }
            }
            None => Ok(0),
        }
    }

    /// Increment transaction counter
    pub async fn increment_transaction_count(&self) -> Result<(), StorageError> {
        let counter_key = b"tx_count";
        let current_count = self.get_total_transactions().await?;
        let new_count = current_count + 1;
        let count_bytes = new_count.to_le_bytes();
        self.put(counter_key, &count_bytes).await
    }

    /// Get total transaction count from LevelDB
    pub async fn get_total_transactions(&self) -> Result<u64, StorageError> {
        // Use a dedicated counter key for transaction count
        let counter_key = b"tx_count";
        match self.get(counter_key).await? {
            Some(count_bytes) => {
                if count_bytes.len() == 8 {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&count_bytes);
                    Ok(u64::from_le_bytes(arr))
                } else {
                    Ok(0)
                }
            }
            None => Ok(0),
        }
    }
}