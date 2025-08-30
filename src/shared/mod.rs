// Shared storage and data structures for FVChain
use once_cell::sync::Lazy;
use tokio::sync::RwLock as TokioRwLock;
use serde::{Deserialize, Serialize};

// Shared transaction structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletTransaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
    pub fee: u64,
    pub timestamp: u64,
    pub signature: Option<Vec<u8>>,
    pub hash: Option<String>,
}

// Global shared transaction storage
pub static TRANSACTIONS: Lazy<TokioRwLock<Vec<WalletTransaction>>> = Lazy::new(|| {
    TokioRwLock::new(Vec::new())
});

// Global shared block height
pub static BLOCK_HEIGHT: Lazy<TokioRwLock<u64>> = Lazy::new(|| {
    TokioRwLock::new(1) // Start from block 1 (genesis is 0)
});

// Helper functions for transaction management
impl WalletTransaction {
    pub fn new_mining_reward(to: String, amount: u64, hash: String) -> Self {
        Self {
            from: "fvc0000000000000000000000000000000000000000".to_string(), // coinbase
            to,
            amount,
            nonce: 0,
            fee: 0,
            timestamp: chrono::Utc::now().timestamp() as u64,
            signature: None,
            hash: Some(hash),
        }
    }
    
    pub fn new_transfer(from: String, to: String, amount: u64, fee: u64, signature: Option<Vec<u8>>, hash: String) -> Self {
        Self {
            from,
            to,
            amount,
            nonce: 0,
            fee,
            timestamp: chrono::Utc::now().timestamp() as u64,
            signature,
            hash: Some(hash),
        }
    }
}

// Transaction storage helper functions
pub async fn add_transaction(tx: WalletTransaction) {
    let mut txs = TRANSACTIONS.write().await;
    txs.push(tx);
}

pub async fn get_transaction_count() -> u64 {
    let txs = TRANSACTIONS.read().await;
    txs.len() as u64
}

pub async fn get_latest_transactions(limit: usize) -> Vec<WalletTransaction> {
    let txs = TRANSACTIONS.read().await;
    let start = if txs.len() > limit { txs.len() - limit } else { 0 };
    txs[start..].to_vec()
}

// Block height management functions
pub async fn increment_block_height() -> u64 {
    let mut height = BLOCK_HEIGHT.write().await;
    *height += 1;
    *height
}

pub async fn get_block_height() -> u64 {
    let height = BLOCK_HEIGHT.read().await;
    *height
}

pub async fn set_block_height(new_height: u64) {
    let mut height = BLOCK_HEIGHT.write().await;
    *height = new_height;
}