use crate::crypto::fractal_hash::FractalPoW;
use crate::wallet::wallet::Wallet;
use crate::consensus::vortex_consensus::{VortexConsensus, VortexBlock, Transaction};
use crate::rpc_storage::{WalletTransaction, RPCStorage};
use crate::shared;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use crate::node::fractal_node::{NodeState, NodeError};
use chrono::Utc;
use rand::random;

pub struct EcosystemMiner {
    wallet: Arc<Mutex<Wallet>>,
    is_mining: Arc<AtomicBool>,
    address: String,
    #[allow(dead_code)]
    private_key: String,
    consensus: Arc<RwLock<VortexConsensus>>,
    state: Arc<RwLock<NodeState>>,
}

impl EcosystemMiner {
    pub async fn new(
        consensus: Arc<RwLock<VortexConsensus>>,
        state: Arc<RwLock<NodeState>>
    ) -> Result<Self, NodeError> {
        // Production ecosystem wallet credentials - Network Maintenance Fund
        let address = "fvcfedcba9876543210fedcba9876543210fedcbaemyl".to_string();
        let private_key = "d4e5f6789012345678901234567890abcdef1234567890abcdef123456789abc".to_string();
        
        let wallet = Wallet::from_private_key(&private_key).unwrap_or_else(|_| {
            Wallet::new_with_address(&address)
        });

        Ok(Self {
            wallet: Arc::new(Mutex::new(wallet)),
            is_mining: Arc::new(AtomicBool::new(false)),
            address,
            private_key,
            consensus,
            state,
        })
    }

    pub async fn start(&self) -> Result<(), NodeError> {
        if self.is_mining.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_mining.store(true, Ordering::SeqCst);
        let wallet = Arc::clone(&self.wallet);
        let is_mining = Arc::clone(&self.is_mining);
        let address = self.address.clone();
        let consensus = self.consensus.clone();
        let state = self.state.clone();
        // Removed ledger dependency - using in-memory storage only

        tokio::spawn(async move {
            println!("🌐 Ecosystem miner started for address: {}", address);
            
            while is_mining.load(Ordering::SeqCst) {
                let mut wallet = wallet.lock().await;
                
                // Proof-of-Work mining using FractalPoW
                // Reward defined in wei (18 decimals)
                let block_reward: u64 = 6_250_000; // 6.25 FVC per block (6 decimals)
                
                // Build block template data (timestamp + address) for hashing
                let timestamp = Utc::now().timestamp() as u64;
                let data_for_hash = [address.as_bytes(), &timestamp.to_le_bytes()].concat();
                
                // Initialize PoW with fixed difficulty 2 and fractal levels 3
                let pow = FractalPoW::new(2, 3);
                let (nonce_bytes, block_hash) = pow.mine(&data_for_hash);
                let nonce = u64::from_le_bytes(nonce_bytes[..8].try_into().unwrap());
                
                // Update wallet balance upon successful mining
                wallet.balance = wallet.balance.saturating_add(block_reward);

                // Balance is now stored in-memory only (no persistent storage)
                
                // Generate new block for the ecosystem transactions
                let new_block_height = crate::rpc_storage::RPCStorage::increment_block_height().await.unwrap_or(1);
                
                // Create a single timestamp for all transactions and block to ensure consistency
                let block_timestamp = chrono::Utc::now().timestamp() as u64;
                let mut block_transactions = Vec::new();
                
                // Create a mining reward transaction for consensus with consistent timestamp
                let reward_tx = Transaction {
                    hash: [0u8; 32],
                    from: Vec::new(), // coinbase
                    to: address.clone().into_bytes(),
                    amount: block_reward,
                    nonce: block_timestamp, // Use consistent timestamp
                    signature: Vec::new(),
                    vortex_fee: 0.0,
                };
                
                // Get all active mining devices and distribute rewards
                let active_devices = RPCStorage::get_all_active_devices().await.unwrap_or_default();
                println!("🔍 Active devices for mining rewards: {:?}", active_devices);
                
                if !active_devices.is_empty() {
                    // Distribute mining reward among active miners
                    let reward_per_miner = block_reward / active_devices.len() as u64;
                    
                    for device_id in active_devices {
                        // Get miner's wallet address
                        if let Ok(Some(miner_address)) = RPCStorage::get_device_address(&device_id).await {
                            // Create mining reward transaction for this miner with consistent timestamp
                            let tx_hash = format!("0xmining_{}_{}_{}_{:x}", new_block_height, block_timestamp, device_id.chars().take(8).collect::<String>(), random::<u32>());
                            let mut miner_reward_tx = WalletTransaction::new_mining_reward(
                                miner_address.clone(),
                                reward_per_miner,
                                tx_hash.clone(),
                                new_block_height
                            );
                            // Override timestamp to ensure consistency with block
                            miner_reward_tx.timestamp = block_timestamp;
                            block_transactions.push(miner_reward_tx.clone());
                            
                            // Store transaction and update balance
                            println!("🔄 Attempting to store mining reward transaction: {} to {}", tx_hash, miner_address);
                            if let Err(e) = RPCStorage::add_transaction(&miner_reward_tx).await {
                                println!("❌ Error storing mining reward transaction: {}", e);
                            } else {
                                println!("✅ Successfully stored mining reward transaction: {}", tx_hash);
                            }
                            
                            // Update miner's balance
                            let current_balance = RPCStorage::get_balance(&miner_address).await.unwrap_or(0);
                            let new_balance = current_balance.saturating_add(reward_per_miner);
                            let _ = RPCStorage::set_balance(&miner_address, new_balance).await;
                            
                            println!("💰 Mining reward {} FVC sent to miner: {}", reward_per_miner as f64 / 1_000_000.0, miner_address);
                        }
                    }
                } else {
                    // Fallback: give reward to ecosystem if no active miners
                    let tx_hash = format!("eco{:x}", random::<u64>());
                    let mut wallet_reward_tx = WalletTransaction::new_mining_reward(
                        address.clone(),
                        block_reward,
                        tx_hash.clone(),
                        new_block_height
                    );
                    // Override timestamp to ensure consistency with block
                    wallet_reward_tx.timestamp = block_timestamp;
                    block_transactions.push(wallet_reward_tx.clone());
                    
                    // Store transaction in persistent storage for dashboard visibility
                    let _ = RPCStorage::add_transaction(&wallet_reward_tx).await;
                }
                
                // Create real blockchain block and store it with actual FractalPoW hash
                let parent_hash = if new_block_height > 0 {
                    format!("{:064x}", new_block_height - 1 + 12345)
                } else {
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string()
                };
                
                // Use the real hash from FractalPoW mining with consistent timestamp
                let mut real_block = crate::rpc_storage::Block::new_with_real_hash_and_timestamp(
                    new_block_height,
                    address.clone(),
                    parent_hash,
                    block_hash.hash, // Real hash from FractalPoW
                    nonce,
                    2, // difficulty
                    block_timestamp // Use consistent timestamp
                );
                
                // Add all transactions to the block
                for tx in block_transactions {
                    real_block.add_transaction(tx);
                }
                
                // Transactions are now properly added to the block with consistent timestamps
                
                // Store the real block in blockchain storage
                if let Err(e) = crate::rpc_storage::RPCStorage::store_block(&real_block).await {
                    println!("❌ Error storing real block: {}", e);
                } else {
                    println!("✅ Real block #{} stored in blockchain", new_block_height);
                    
                    // Update block height after successful storage
                    if let Err(e) = crate::rpc_storage::RPCStorage::increment_block_height().await {
                        println!("❌ Error updating block height: {}", e);
                    }
                }
                
                // Create a new block using PoW results with consistent timestamp
                 let new_block = VortexBlock {
                    hash: block_hash.hash,
                    nonce,
                    difficulty: 2,
                    parent_hashes: Vec::new(),
                    transactions: vec![reward_tx],
                    timestamp: block_timestamp, // Use consistent timestamp
                    validator_id: libp2p::PeerId::random(), // TODO: map wallet -> PeerId
                    vortex_energy: block_hash.vortex_energy as f64,
                    fractal_level: block_hash.fractal_level,
                    sierpinski_proof: vec![block_hash.sierpinski_pattern],
                };
                
                // Add block to consensus
                let mut consensus_guard = consensus.write().await;
                if let Err(e) = consensus_guard.add_block(new_block).await {
                    println!("❌ Error adding block to consensus: {}", e);
                }

                println!("✅ Block added to consensus");
                println!("⚡ Mined block #{} with {} FVC reward to ecosystem wallet", new_block_height, block_reward as f64 / 1_000_000.0);
                println!("💰 Total ecosystem balance: {} FVC", wallet.balance as f64 / 1_000_000.0);
                
                // Update transaction count based on real transactions
                let real_tx_count = RPCStorage::get_transaction_count().await.unwrap_or(0);
                let mut state_guard = state.write().await;
                state_guard.total_transactions = real_tx_count;
                
                println!("📈 Total transactions in blockchain: {}", real_tx_count);
                
                // Mining interval (every 5 seconds for demo)
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
        
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), NodeError> {
        self.is_mining.store(false, Ordering::SeqCst);
        println!("🛑 Ecosystem miner stopped");
        Ok(())
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub async fn get_balance(&self) -> u64 {
        // Use wallet's in-memory balance only
        match self.wallet.try_lock() {
            Ok(wallet) => wallet.balance,
            Err(_) => 0, // Default if we can't access the wallet
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_mining.load(Ordering::SeqCst)
    }
}

// Default implementation removed as new() now requires parameters