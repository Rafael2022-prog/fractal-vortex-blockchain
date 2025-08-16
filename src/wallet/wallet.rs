use crate::wallet::key_manager::KeyManager;
use crate::wallet::transaction::{WalletTransaction, TransactionBuilder};
use crate::wallet::rpc_client::{RpcClient, NetworkInfo, TransactionStatus};
use std::collections::HashMap;

pub struct Wallet {
    key_manager: KeyManager,
    rpc_client: RpcClient,
    nonce: u64,
    pub balance: u64, // In-memory balance in smallest unit (FVC)
}

impl Wallet {
    pub fn new(rpc_url: &str) -> Self {
        let key_manager = KeyManager::new();
        let rpc_client = RpcClient::new(rpc_url);
        
        Self {
            key_manager,
            rpc_client,
            nonce: 0,
            balance: 0,
        }
    }

    pub fn from_mnemonic(mnemonic: &str, rpc_url: &str) -> Self {
        let key_manager = KeyManager::from_mnemonic(mnemonic);
        let rpc_client = RpcClient::new(rpc_url);
        
        Self {
            key_manager,
            rpc_client,
            nonce: 0,
            balance: 0,
        }
    }

    pub fn from_file(wallet_file: &str, rpc_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let key_manager = KeyManager::from_file(wallet_file)?;
        let rpc_client = RpcClient::new(rpc_url);
        
        Ok(Self {
            key_manager,
            rpc_client,
            nonce: 0,
            balance: 0,
        })
    }
    
    pub fn from_private_key(private_key_hex: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let key_manager = KeyManager::from_private_key_hex(private_key_hex)?;
        // Use a dummy RPC URL since this is used for local-only operations
        let rpc_client = RpcClient::new("http://localhost:8080");
        
        Ok(Self {
            key_manager,
            rpc_client,
            nonce: 0,
            balance: 0,
        })
    }
    
    pub fn new_with_address(address: &str) -> Self {
        let key_manager = KeyManager::new_with_address(address);
        let rpc_client = RpcClient::new("http://localhost:8080");
        
        Self {
            key_manager,
            rpc_client,
            nonce: 0,
            balance: 0,
        }
    }

    pub fn save_to_file(&self, wallet_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.key_manager.save_to_file(wallet_file)
    }

    pub fn get_address(&self) -> String {
        self.key_manager.get_address()
    }

    pub fn get_peer_id(&self) -> String {
        self.key_manager.get_peer_id().to_string()
    }

    pub async fn refresh_nonce(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let balance_info = self.rpc_client.get_balance(&self.get_address()).await?;
        self.nonce = balance_info.nonce;
        Ok(())
    }

    pub async fn get_balance(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let balance_info = self.rpc_client.get_balance(&self.get_address()).await?;
        Ok(balance_info.balance)
    }

    pub async fn get_network_info(&self) -> Result<NetworkInfo, Box<dyn std::error::Error>> {
        self.rpc_client.get_network_info().await
    }

    pub async fn transfer(&mut self, to_address: &str, amount: u64) -> Result<String, Box<dyn std::error::Error>> {
        self.refresh_nonce().await?;
        
        let tx_builder = TransactionBuilder::new(self.get_address(), self.nonce);
        let mut transaction = tx_builder.transfer(to_address.to_string(), amount);
        
        let tx_data = format!("{}{}{}{}", 
            transaction.from, transaction.to, transaction.amount, transaction.nonce);
        let signature = self.key_manager.sign(tx_data.as_bytes())?;
        
        transaction.sign(signature);
        
        let tx_hash = self.rpc_client.send_transaction(&transaction).await?;
        self.nonce += 1;
        
        Ok(tx_hash)
    }

    pub async fn stake(&mut self, validator_id: &str, amount: u64) -> Result<String, Box<dyn std::error::Error>> {
        self.refresh_nonce().await?;
        
        let tx_builder = TransactionBuilder::new(self.get_address(), self.nonce);
        let mut transaction = tx_builder.stake(validator_id.to_string(), amount);
        
        let tx_data = format!("{}{}{}{}", 
            transaction.from, transaction.to, transaction.amount, transaction.nonce);
        let signature = self.key_manager.sign(tx_data.as_bytes())?;
        
        transaction.sign(signature);
        
        let tx_hash = self.rpc_client.send_transaction(&transaction).await?;
        self.nonce += 1;
        
        Ok(tx_hash)
    }

    pub async fn unstake(&mut self, validator_id: &str, amount: u64) -> Result<String, Box<dyn std::error::Error>> {
        self.refresh_nonce().await?;
        
        let tx_builder = TransactionBuilder::new(self.get_address(), self.nonce);
        let mut transaction = tx_builder.unstake(validator_id.to_string(), amount);
        
        let tx_data = format!("{}{}{}{}", 
            transaction.from, transaction.to, transaction.amount, transaction.nonce);
        let signature = self.key_manager.sign(tx_data.as_bytes())?;
        
        transaction.sign(signature);
        
        let tx_hash = self.rpc_client.send_transaction(&transaction).await?;
        self.nonce += 1;
        
        Ok(tx_hash)
    }

    pub async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus, Box<dyn std::error::Error>> {
        self.rpc_client.get_transaction_status(tx_hash).await
    }

    pub async fn get_staking_info(&self) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
        self.rpc_client.get_staking_info(&self.get_address()).await
    }

    pub async fn get_validators(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.rpc_client.get_validators().await
    }

    pub async fn estimate_fee(&self, transaction_type: &str, amount: u64) -> Result<u64, Box<dyn std::error::Error>> {
        let tx_builder = TransactionBuilder::new(self.get_address(), self.nonce);
        let transaction = match transaction_type {
            "stake" => tx_builder.stake("validator".to_string(), amount),
            "unstake" => tx_builder.unstake("validator".to_string(), amount),
            _ => tx_builder.transfer("recipient".to_string(), amount),
        };
        
        self.rpc_client.estimate_gas(&transaction).await
    }

    pub fn generate_mnemonic() -> String {
        KeyManager::generate_mnemonic()
    }

    pub async fn get_recent_transactions(&self, limit: usize) -> Result<Vec<WalletTransaction>, Box<dyn std::error::Error>> {
        self.rpc_client.get_latest_transactions(limit).await
    }

    pub async fn is_synced(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let network_info = self.get_network_info().await?;
        Ok(network_info.block_height > 0)
    }

    pub fn get_wallet_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("address".to_string(), self.get_address());
        info.insert("peer_id".to_string(), self.get_peer_id());
        info.insert("rpc_url".to_string(), self.rpc_client.base_url.clone());
        info
    }
}