use serde::{Serialize, Deserialize};
use std::fs;

use bip39::{Mnemonic, Language};
use rand::rngs::OsRng;
use rand::RngCore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManager {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
    address: String,
}

impl KeyManager {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let mut private_key = [0u8; 32];
        rng.fill_bytes(&mut private_key);
        
        // Generate public key (simplified)
        let public_key = private_key.iter().map(|&b| b ^ 0x42).collect::<Vec<_>>();
        let address_hash = private_key.iter().fold(0u64, |acc, &b| acc.wrapping_add(b as u64));
        let address = format!("0x{:016x}", address_hash);
        
        Self {
            private_key: private_key.to_vec(),
            public_key,
            address,
        }
    }

    pub fn from_mnemonic(mnemonic: &str) -> Self {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic).expect("Invalid mnemonic");
        let seed = mnemonic.to_seed("");
        
        let private_key = seed[..32].to_vec();
        let public_key = private_key.iter().map(|&b| b ^ 0x42).collect::<Vec<_>>();
        let address_hash = private_key.iter().fold(0u64, |acc, &b| acc.wrapping_add(b as u64));
        let address = format!("0x{:016x}", address_hash);
        
        Self {
            private_key,
            public_key,
            address,
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let key_manager: KeyManager = serde_json::from_str(&data)?;
        Ok(key_manager)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn get_address(&self) -> String {
        self.address.clone()
    }

    pub fn get_peer_id(&self) -> String {
        self.address.clone()
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }

    pub fn get_private_key(&self) -> Vec<u8> {
        self.private_key.clone()
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        // Simplified signing - XOR with private key
        let mut signature = Vec::new();
        for (i, &byte) in data.iter().enumerate() {
            let key_byte = self.private_key[i % self.private_key.len()];
            signature.push(byte ^ key_byte);
        }
        signature
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        // Simplified verification
        let expected = self.sign(data);
        signature == expected
    }

    pub fn generate_mnemonic() -> String {
        let mut rng = OsRng;
        let mut entropy = [0u8; 16];
        rng.fill_bytes(&mut entropy);
        
        let mnemonic = Mnemonic::from_entropy(&entropy).expect("Failed to generate mnemonic");
        mnemonic.to_string()
    }

    pub fn from_private_key_hex(private_key_hex: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Remove optional 0x prefix
        let cleaned = private_key_hex.trim_start_matches("0x");
        // Ensure even length
        if cleaned.len() % 2 != 0 {
            return Err("Private key hex should have even length".into());
        }
        // Parse hex into bytes
        let bytes_res: Result<Vec<u8>, _> = (0..cleaned.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16))
            .collect();
        let private_key = bytes_res?;
        if private_key.len() != 32 {
            return Err("Private key must be 32 bytes".into());
        }
        // Derive public key and address using same toy algorithm
        let public_key = private_key.iter().map(|&b| b ^ 0x42).collect::<Vec<_>>();
        let address_hash = private_key.iter().fold(0u64, |acc, &b| acc.wrapping_add(b as u64));
        let address = format!("0x{:016x}", address_hash);
        Ok(Self {
            private_key,
            public_key,
            address,
        })
    }

    /// Construct a read-only key manager with a pre-defined address.
    /// The generated private/public keys are zeroed and should **not** be used for signing.
    pub fn new_with_address(address: &str) -> Self {
        let private_key = vec![0u8; 32];
        let public_key = private_key.iter().map(|&b| b ^ 0x42).collect::<Vec<_>>();
        Self {
            private_key,
            public_key,
            address: address.to_string(),
        }
    }
}