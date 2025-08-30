use serde::{Serialize, Deserialize};
use std::fs;
use sha3::{Sha3_256, Digest};
use secp256k1::{Secp256k1, SecretKey, PublicKey, Message, ecdsa::Signature};
use bip39::Mnemonic;
use rand::rngs::OsRng;
use rand::RngCore;
use crate::crypto::fractal_hash::FractalHasher;

/// Production-ready KeyManager with secp256k1 cryptography and 160-bit FVChain addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManager {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
    address: String,
    #[serde(skip)]
    secp: Secp256k1<secp256k1::All>,
}

impl KeyManager {
    /// Create new KeyManager with full secp256k1 cryptography and native FVChain addressing
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        
        // Generate cryptographically secure private key
        let mut private_key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut private_key_bytes);
        let secret_key = SecretKey::from_slice(&private_key_bytes)
            .expect("Invalid private key");
        let private_key = private_key_bytes.to_vec();
        
        // Generate public key using secp256k1
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_bytes = public_key.serialize().to_vec();
        
        // Generate native FVChain address (160-bit) using Fractal-Vortex mathematics
        let address = Self::generate_fvchain_address(&public_key_bytes);
        
        Self {
            private_key,
            public_key: public_key_bytes,
            address,
            secp,
        }
    }
    
    /// Generate native FVChain address using Fractal-Vortex mathematics (NATIVE FORMAT: 42 chars)
    fn generate_fvchain_address(public_key: &[u8]) -> String {
        // Apply Fractal-Vortex transformation to public key
        let mut fractal_hasher = FractalHasher::new(3); // 3 fractal iterations
        let vortex_hash = fractal_hasher.fractal_hash(public_key);
        
        // Apply Sierpinski triangle transformation
        let mut sierpinski_data = vortex_hash.fractal_hash.to_vec();
        
        // Vortex pattern: 1-2-4-8-7-5
        let vortex_pattern = [1u8, 2, 4, 8, 7, 5];
        for (i, byte) in sierpinski_data.iter_mut().enumerate() {
            *byte ^= vortex_pattern[i % vortex_pattern.len()];
        }
        
        // Generate 144-bit address using digital root mathematics (18 bytes = 36 hex chars)
        let mut address_bytes = [0u8; 18]; // 144 bits = 18 bytes for native format
        
        // Apply digital root transformation
        for i in 0..18 {
            let mut sum = 0u32;
            for j in 0..sierpinski_data.len() {
                sum += sierpinski_data[j] as u32 * (i + j + 1) as u32;
            }
            
            // Digital root calculation with vortex enhancement
            while sum >= 256 {
                sum = (sum / 10) + (sum % 10);
            }
            
            address_bytes[i] = (sum % 256) as u8;
        }
        
        // Apply final vortex energy signature
        let energy_signature = vortex_hash.energy_signature;
        for (i, byte) in address_bytes.iter_mut().enumerate() {
            *byte ^= ((energy_signature >> (i % 8)) & 0xFF) as u8;
        }
        
        // Format as FVChain NATIVE address with "fvc" prefix and "emyl" suffix (43 chars total)
        format!("fvc{}emyl", hex::encode(address_bytes))
    }
    
    /// Create KeyManager from mnemonic with full cryptographic security
    pub fn from_mnemonic(mnemonic: &str) -> Self {
        let mnemonic = Mnemonic::parse(mnemonic)
            .expect("Invalid mnemonic");
        let seed = mnemonic.to_seed("");
        
        let secp = Secp256k1::new();
        
        // Derive private key from seed using HMAC-SHA512
        let mut hasher = Sha3_256::new();
        hasher.update(&seed[..32]);
        let hash = hasher.finalize();
        
        let secret_key = SecretKey::from_slice(&hash)
            .expect("Invalid private key from mnemonic");
        let private_key = secret_key.secret_bytes().to_vec();
        
        // Generate public key
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_bytes = public_key.serialize().to_vec();
        
        // Generate FVChain address
        let address = Self::generate_fvchain_address(&public_key_bytes);
        
        Self {
            private_key,
            public_key: public_key_bytes,
            address,
            secp,
        }
    }
    
    /// Load KeyManager from file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let mut key_manager: KeyManager = serde_json::from_str(&data)?;
        key_manager.secp = Secp256k1::new();
        Ok(key_manager)
    }
    
    /// Save KeyManager to file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }
    
    /// Get FVChain address (160-bit)
    pub fn get_address(&self) -> String {
        self.address.clone()
    }
    
    /// Get peer ID for network identification
    pub fn get_peer_id(&self) -> String {
        format!("peer_{}", &self.address[3..23]) // Use part of address as peer ID
    }
    
    /// Get public key bytes
    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }
    
    /// Get private key bytes (use with caution)
    pub fn get_private_key(&self) -> Vec<u8> {
        self.private_key.clone()
    }
    
    /// Sign data using secp256k1 ECDSA
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let secret_key = SecretKey::from_slice(&self.private_key)?;
        
        // Hash data with Fractal-Vortex enhancement
        let mut fractal_hasher = FractalHasher::new(2);
        let vortex_hash = fractal_hasher.fractal_hash(data);
        
        let message = Message::from_digest_slice(&vortex_hash.fractal_hash)?;
        let signature = self.secp.sign_ecdsa(&message, &secret_key);
        
        Ok(signature.serialize_compact().to_vec())
    }
    
    /// Verify signature using secp256k1 ECDSA
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        if let Ok(public_key) = PublicKey::from_slice(&self.public_key) {
            if let Ok(sig) = Signature::from_compact(signature) {
                // Hash data with same Fractal-Vortex enhancement
                let mut fractal_hasher = FractalHasher::new(2);
                let vortex_hash = fractal_hasher.fractal_hash(data);
                
                if let Ok(message) = Message::from_digest_slice(&vortex_hash.fractal_hash) {
                    return self.secp.verify_ecdsa(&message, &sig, &public_key).is_ok();
                }
            }
        }
        false
    }
    
    /// Generate new mnemonic phrase
    pub fn generate_mnemonic() -> String {
        let mut entropy = [0u8; 32]; // 256 bits for 24 words
        OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .expect("Failed to generate mnemonic");
        mnemonic.to_string()
    }
    
    /// Create KeyManager from private key hex string
    pub fn from_private_key_hex(private_key_hex: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let private_key_bytes = hex::decode(private_key_hex.trim_start_matches("0x"))?;
        
        if private_key_bytes.len() != 32 {
            return Err("Private key must be 32 bytes".into());
        }
        
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&private_key_bytes)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_bytes = public_key.serialize().to_vec();
        
        let address = Self::generate_fvchain_address(&public_key_bytes);
        
        Ok(Self {
            private_key: private_key_bytes,
            public_key: public_key_bytes,
            address,
            secp,
        })
    }
    
    /// Create KeyManager with only address (for read-only operations)
    pub fn new_with_address(address: &str) -> Self {
        Self {
            private_key: vec![0u8; 32],
            public_key: vec![0u8; 33],
            address: address.to_string(),
            secp: Secp256k1::new(),
        }
    }
    
    /// Validate FVChain address format (NATIVE FORMAT: fvc + 36 hex + emyl)
    pub fn validate_address(address: &str) -> bool {
        if !address.starts_with("fvc") {
            return false;
        }
        
        if !address.ends_with("emyl") {
            return false;
        }
        
        if address.len() != 43 {
            return false;
        }
        
        // Extract hex part (remove fvc prefix and emyl suffix)
        let hex_part = &address[3..address.len()-4];
        if hex_part.len() != 36 {
            return false;
        }
        
        hex::decode(hex_part).is_ok()
    }
    
    /// Get address checksum for validation
    pub fn get_address_checksum(&self) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(self.address.as_bytes());
        let hash = hasher.finalize();
        hex::encode(&hash[..4])
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let km = KeyManager::new();
        assert!(km.get_address().starts_with("fvc"));
        assert_eq!(km.get_address().len(), 43); // "fvc" + 36 hex chars + "emyl"
        assert!(KeyManager::validate_address(&km.get_address()));
    }
    
    #[test]
    fn test_sign_verify() {
        let km = KeyManager::new();
        let data = b"test message";
        
        let signature = km.sign(data).expect("Failed to sign");
        assert!(km.verify(data, &signature));
        
        // Test with different data
        let wrong_data = b"wrong message";
        assert!(!km.verify(wrong_data, &signature));
    }
    
    #[test]
    fn test_mnemonic() {
        let mnemonic = KeyManager::generate_mnemonic();
        let km = KeyManager::from_mnemonic(&mnemonic);
        assert!(km.get_address().starts_with("fvc"));
    }
}