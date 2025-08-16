use crate::crypto::fractal_hash::FractalPoW;
use crate::wallet::wallet::Wallet;
use crate::consensus::vortex_consensus::{VortexConsensus, VortexBlock, Transaction};
use crate::storage::LedgerDB;
use crate::shared::{WalletTransaction, add_transaction, get_transaction_count, increment_block_height};
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
    ledger: Arc<LedgerDB>,
}

impl EcosystemMiner {
    pub async fn new(
        consensus: Arc<RwLock<VortexConsensus>>,
        state: Arc<RwLock<NodeState>>,
        ledger: Arc<LedgerDB>
    ) -> Result<Self, NodeError> {
        // Fixed ecosystem wallet credentials (note tetap)
        let address = "0xEC0SYS73M0000000000000000000000000000000".to_string();
        let private_key = "0xec0sys73m0000000000000000000000000000000000000000000000000000000000000000".to_string();
        
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
            ledger,
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
        let ledger = self.ledger.clone();

        tokio::spawn(async move {
            println!("🌐 Ecosystem miner started for address: {}", address);
            
            while is_mining.load(Ordering::SeqCst) {
                let mut wallet = wallet.lock().await;
                
                // Proof-of-Work mining using FractalPoW
                // Reward defined in wei (18 decimals)
                let block_reward: u64 = 6_250_000_000; // 6.25 FVC per block in wei (9 decimals)
                
                // Build block template data (timestamp + address) for hashing
                let timestamp = Utc::now().timestamp() as u64;
                let data_for_hash = [address.as_bytes(), &timestamp.to_le_bytes()].concat();
                
                // Initialize PoW with fixed difficulty 2 and fractal levels 3
                let pow = FractalPoW::new(2, 3);
                let (nonce_bytes, block_hash) = pow.mine(&data_for_hash);
                let nonce = u64::from_le_bytes(nonce_bytes[..8].try_into().unwrap());
                
                // Update wallet balance upon successful mining
                wallet.balance = wallet.balance.saturating_add(block_reward);

                // Persist balance in ledger
                if let Err(e) = ledger.set_balance(&address, wallet.balance).await {
                    println!("❌ Error updating ledger: {}", e);
                }
                
                // Generate new block for the ecosystem transactions
                let new_block_height = increment_block_height().await;
                
                // Create a mining reward transaction for consensus
                let reward_tx = Transaction {
                    hash: [0u8; 32],
                    from: Vec::new(), // coinbase
                    to: address.clone().into_bytes(),
                    amount: block_reward,
                    nonce: timestamp,
                    signature: Vec::new(),
                    vortex_fee: 0.0,
                };
                
                // Create and store mining reward transaction for dashboard visibility
                let tx_hash = format!("0xeco{:x}", random::<u64>());
                let wallet_reward_tx = WalletTransaction::new_mining_reward(
                    address.clone(),
                    block_reward,
                    tx_hash.clone()
                );
                
                // Store transaction in shared storage for dashboard visibility
                add_transaction(wallet_reward_tx).await;
                
                // Create a new block using PoW results
                 let new_block = VortexBlock {
                    hash: block_hash.hash,
                    nonce,
                    difficulty: 2,
                    parent_hashes: Vec::new(),
                    transactions: vec![reward_tx],
                    timestamp,
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
                println!("⚡ Mined block #{} with {} FVC reward to ecosystem wallet", new_block_height, block_reward as f64 / 1_000_000_000.0);
                println!("💰 Total ecosystem balance: {} FVC", wallet.balance as f64 / 1_000_000_000.0);
                println!("📊 Mining reward transaction recorded: {}", tx_hash);
                
                // Update transaction count based on real transactions
                let real_tx_count = get_transaction_count().await;
                let mut state_guard = state.write().await;
                state_guard.total_transactions = real_tx_count;
                
                println!("📈 Total transactions in blockchain: {}", real_tx_count);
                
                // Mining interval (every 10 seconds for demo)
                tokio::time::sleep(Duration::from_secs(10)).await;
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
        // Query the ledger for actual balance using new async API
        match self.ledger.get_balance(&self.address).await {
            Ok(balance) => balance,
            Err(_) => {
                // Fallback to wallet's in-memory balance if ledger fails
                match self.wallet.try_lock() {
                    Ok(wallet) => wallet.balance,
                    Err(_) => 0, // Default if we can't access the wallet
                }
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_mining.load(Ordering::SeqCst)
    }
}

// Default implementation removed as new() now requires parameters