use serde::{Serialize, Deserialize};

use reqwest;
use std::collections::HashMap;
use crate::wallet::transaction::WalletTransaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u64,
    pub pending_balance: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub chain_id: String,
    pub block_height: u64,
    pub network_hash_rate: f64,
    pub active_validators: usize,
    pub total_supply: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub hash: String,
    pub status: String,
    pub block_height: Option<u64>,
    pub confirmations: u64,
}

pub struct RpcClient {
    pub base_url: String,
    client: reqwest::Client,
}

impl RpcClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_balance(&self, address: &str) -> Result<BalanceResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/balance/{}" , self.base_url, address);
        let response = self.client.get(&url).send().await?;
        let balance: BalanceResponse = response.json().await?;
        Ok(balance)
    }

    pub async fn get_network_info(&self) -> Result<NetworkInfo, Box<dyn std::error::Error>> {
        let url = format!("{}/network/info" , self.base_url);
        let response = self.client.get(&url).send().await?;
        let info: NetworkInfo = response.json().await?;
        Ok(info)
    }

    pub async fn send_transaction(&self, transaction: &WalletTransaction) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/transaction" , self.base_url);
        let response = self.client.post(&url)
            .json(transaction)
            .send().await?;
        
        let result: HashMap<String, String> = response.json().await?;
        Ok(result.get("hash").unwrap_or(&"unknown".to_string()).clone())
    }

    pub async fn get_transaction_status(&self, hash: &str) -> Result<TransactionStatus, Box<dyn std::error::Error>> {
        let url = format!("{}/transaction/{}" , self.base_url, hash);
        let response = self.client.get(&url).send().await?;
        let status: TransactionStatus = response.json().await?;
        Ok(status)
    }

    pub async fn get_gas_price(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let url = format!("{}/gas-price" , self.base_url);
        let response = self.client.get(&url).send().await?;
        let result: HashMap<String, u64> = response.json().await?;
        Ok(result.get("gas_price").copied().unwrap_or(1000))
    }

    pub async fn get_validators(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!("{}/validators" , self.base_url);
        let response = self.client.get(&url).send().await?;
        let validators: Vec<String> = response.json().await?;
        Ok(validators)
    }

    pub async fn get_staking_info(&self, address: &str) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
        let url = format!("{}/staking/{}" , self.base_url, address);
        let response = self.client.get(&url).send().await?;
        let staking_info: HashMap<String, u64> = response.json().await?;
        Ok(staking_info)
    }

    pub async fn estimate_gas(&self, transaction: &WalletTransaction) -> Result<u64, Box<dyn std::error::Error>> {
        let url = format!("{}/estimate-gas" , self.base_url);
        let response = self.client.post(&url)
            .json(transaction)
            .send().await?;
        
        let result: HashMap<String, u64> = response.json().await?;
        Ok(result.get("gas_estimate").copied().unwrap_or(21000))
    }

    pub async fn get_block_height(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let url = format!("{}/block/height" , self.base_url);
        let response = self.client.get(&url).send().await?;
        let result: HashMap<String, u64> = response.json().await?;
        Ok(result.get("height").copied().unwrap_or(0))
    }

    pub async fn get_latest_transactions(&self, limit: usize) -> Result<Vec<WalletTransaction>, Box<dyn std::error::Error>> {
        let url = format!("{}/transactions/latest?limit={}" , self.base_url, limit);
        let response = self.client.get(&url).send().await?;
        let transactions: Vec<WalletTransaction> = response.json().await?;
        Ok(transactions)
    }
}