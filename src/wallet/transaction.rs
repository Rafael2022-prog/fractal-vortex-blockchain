use serde::{Serialize, Deserialize};
use sha3::{Sha3_256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub struct TransactionBuilder {
    from: String,
    nonce: u64,
}

impl TransactionBuilder {
    pub fn new(from: String, nonce: u64) -> Self {
        Self { from, nonce }
    }

    pub fn transfer(&self, to: String, amount: u64) -> WalletTransaction {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        WalletTransaction {
            from: self.from.clone(),
            to,
            amount,
            nonce: self.nonce,
            fee: 1000, // Base fee in FVC
            timestamp,
            signature: None,
            hash: None,
        }
    }

    pub fn stake(&self, validator_id: String, amount: u64) -> WalletTransaction {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        WalletTransaction {
            from: self.from.clone(),
            to: format!("STAKE_{}", validator_id),
            amount,
            nonce: self.nonce,
            fee: 500, // Lower fee for staking
            timestamp,
            signature: None,
            hash: None,
        }
    }

    pub fn unstake(&self, validator_id: String, amount: u64) -> WalletTransaction {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        WalletTransaction {
            from: self.from.clone(),
            to: format!("UNSTAKE_{}", validator_id),
            amount,
            nonce: self.nonce,
            fee: 500,
            timestamp,
            signature: None,
            hash: None,
        }
    }
}

impl WalletTransaction {
    pub fn calculate_hash(&mut self) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(self.from.as_bytes());
        hasher.update(self.to.as_bytes());
        hasher.update(self.amount.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.update(self.fee.to_le_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        
        let result = hasher.finalize();
        format!("0x{:x}", result)
    }

    pub fn sign(&mut self, signature: Vec<u8>) {
        self.signature = Some(signature);
        self.hash = Some(self.calculate_hash());
    }



    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }

    pub fn verify_signature(&self, _public_key: &[u8]) -> bool {
        if let Some(signature) = &self.signature {
            // Simplified signature verification - in production use proper crypto
            !signature.is_empty()
        } else {
            false
        }
    }

    pub fn estimate_gas(&self) -> u64 {
        match self.to.as_str() {
            t if t.starts_with("STAKE_") => 21000,
            t if t.starts_with("UNSTAKE_") => 25000,
            _ => 21000 + (self.amount / 1000),
        }
    }

    pub fn get_transaction_type(&self) -> &'static str {
        if self.to.starts_with("STAKE_") {
            "STAKE"
        } else if self.to.starts_with("UNSTAKE_") {
            "UNSTAKE"
        } else {
            "TRANSFER"
        }
    }
}