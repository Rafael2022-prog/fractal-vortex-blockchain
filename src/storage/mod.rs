use rocksdb::{DB, Options};
use std::path::Path;
use thiserror::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error type for storage operations
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),
}

/// Simple ledger/UTXO storage backed by RocksDB
pub struct LedgerDB {
    db: Arc<RwLock<DB>>, // protected DB for async context
}

impl LedgerDB {
    /// Open or create a RocksDB instance at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Self {
            db: Arc::new(RwLock::new(db)),
        })
    }

    /// Put arbitrary key/value pair
    pub async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let db = self.db.write().await;
        db.put(key, value)?;
        Ok(())
    }

    /// Get arbitrary value
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let db = self.db.read().await;
        let res = db.get(key)?;
        Ok(res.map(|v| v.to_vec()))
    }

    /// Delete key
    pub async fn delete(&self, key: &[u8]) -> Result<(), StorageError> {
        let db = self.db.write().await;
        db.delete(key)?;
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
}