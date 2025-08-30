use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use hex;

// API Key structure
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub key_hash: String,
    pub permissions: Vec<String>,
    pub rate_limit: u32,
    pub created_at: Instant,
    pub last_used: Instant,
    pub usage_count: u64,
}

// API Authentication Manager
#[derive(Debug)]
pub struct ApiAuthManager {
    api_keys: Arc<Mutex<HashMap<String, ApiKey>>>,
    admin_key_hash: String,
}

impl ApiAuthManager {
    pub fn new() -> Self {
        let mut manager = Self {
            api_keys: Arc::new(Mutex::new(HashMap::new())),
            admin_key_hash: Self::hash_key("fvchain_admin_2025"),
        };
        
        // Initialize with default API keys
        manager.initialize_default_keys();
        manager
    }
    
    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }
    
    fn initialize_default_keys(&mut self) {
        let mut keys = self.api_keys.lock().unwrap();
        
        // Public read-only key for blockchain data
        let public_key = ApiKey {
            key_hash: Self::hash_key("fvchain_public_readonly_2025"),
            permissions: vec![
                "read:blocks".to_string(),
                "read:transactions".to_string(),
                "read:network".to_string(),
                "read:stats".to_string(),
            ],
            rate_limit: 300, // Higher limit for public access
            created_at: Instant::now(),
            last_used: Instant::now(),
            usage_count: 0,
        };
        
        // Mining operations key
        let mining_key = ApiKey {
            key_hash: Self::hash_key("fvchain_mining_2025"),
            permissions: vec![
                "read:mining".to_string(),
                "write:mining".to_string(),
                "read:stats".to_string(),
            ],
            rate_limit: 120,
            created_at: Instant::now(),
            last_used: Instant::now(),
            usage_count: 0,
        };
        
        // Wallet operations key
        let wallet_key = ApiKey {
            key_hash: Self::hash_key("fvchain_wallet_2025"),
            permissions: vec![
                "read:wallet".to_string(),
                "write:wallet".to_string(),
                "read:balance".to_string(),
            ],
            rate_limit: 180,
            created_at: Instant::now(),
            last_used: Instant::now(),
            usage_count: 0,
        };
        
        keys.insert("public".to_string(), public_key);
        keys.insert("mining".to_string(), mining_key);
        keys.insert("wallet".to_string(), wallet_key);
    }
    
    pub fn validate_api_key(&self, key: &str, required_permission: &str) -> bool {
        // Check admin key
        if Self::hash_key(key) == self.admin_key_hash {
            return true;
        }
        
        let mut keys = self.api_keys.lock().unwrap();
        
        for api_key in keys.values_mut() {
            if Self::hash_key(key) == api_key.key_hash {
                // Update usage statistics
                api_key.last_used = Instant::now();
                api_key.usage_count += 1;
                
                // Check permission
                return api_key.permissions.iter().any(|perm| {
                    perm == required_permission || perm.starts_with(&format!("{}:", required_permission.split(':').next().unwrap_or("")))
                });
            }
        }
        
        false
    }
    
    pub fn get_api_key_stats(&self) -> Value {
        let keys = self.api_keys.lock().unwrap();
        let mut stats = json!({});
        
        for (name, key) in keys.iter() {
            stats[name] = json!({
                "permissions": key.permissions,
                "rate_limit": key.rate_limit,
                "usage_count": key.usage_count,
                "last_used": key.last_used.elapsed().as_secs(),
            });
        }
        
        stats
    }
}

// Global API Auth Manager
lazy_static::lazy_static! {
    pub static ref API_AUTH_MANAGER: ApiAuthManager = ApiAuthManager::new();
}

// Authentication middleware - DISABLED
// SEMUA VALIDASI API KEY TELAH DIHAPUS - AKSES TANPA PEMBATASAN
pub async fn api_auth_middleware(
    _headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // SEMUA API ENDPOINT SEKARANG DAPAT DIAKSES TANPA VALIDASI API KEY
    // LANGSUNG LANJUTKAN KE REQUEST BERIKUTNYA
    next.run(request).await
}

// API key stats endpoint
pub async fn api_key_stats() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "api_keys": API_AUTH_MANAGER.get_api_key_stats(),
            "public_endpoints": [
                "/network/info",
                "/network/health",
                "/blocks",
                "/transactions",
                "/smart-rate/current",
                "/mining/stats",
                "/mining/detection/stats",
                "/api/info",
                "/events"
            ]
        }
    }))
}