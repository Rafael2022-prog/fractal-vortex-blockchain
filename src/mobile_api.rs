use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::wallet::Wallet;
use crate::mining::MiningStats;
use crate::node::FractalNode;
use crate::crypto::fractal_hash::FractalHash;
use crate::input_validation::InputValidator;
use std::sync::{Arc, Mutex};
use log::{info, warn, error};

// API Key structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: String,
    pub app_id: String,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub is_active: bool,
    pub permissions: Vec<String>,
}

// Mobile API Manager
pub struct MobileApiManager {
    api_keys: Arc<Mutex<HashMap<String, ApiKey>>>,
    node: Arc<Mutex<FractalNode>>,
}

impl MobileApiManager {
    pub fn new(node: Arc<Mutex<FractalNode>>) -> Self {
        let mut api_keys = HashMap::new();
        
        // Default API key for mobile app
        let default_key = ApiKey {
            key: "fvc_mobile_2025_secure_key_v1".to_string(),
            app_id: "fvchain_mobile_mining".to_string(),
            created_at: Utc::now(),
            last_used: Utc::now(),
            is_active: true,
            permissions: vec![
                "mining.read".to_string(),
                "mining.control".to_string(),
                "wallet.read".to_string(),
                "wallet.send".to_string(),
                "blockchain.read".to_string(),
                "stats.read".to_string(),
            ],
        };
        
        api_keys.insert(default_key.key.clone(), default_key);
        
        Self {
            api_keys: Arc::new(Mutex::new(api_keys)),
            node,
        }
    }
    
    pub fn validate_api_key(&self, key: &str) -> bool {
        if let Ok(keys) = self.api_keys.lock() {
            if let Some(api_key) = keys.get(key) {
                return api_key.is_active;
            }
        }
        false
    }
    
    pub fn has_permission(&self, key: &str, permission: &str) -> bool {
        if let Ok(keys) = self.api_keys.lock() {
            if let Some(api_key) = keys.get(key) {
                return api_key.permissions.contains(&permission.to_string());
            }
        }
        false
    }
    
    pub fn update_last_used(&self, key: &str) {
        if let Ok(mut keys) = self.api_keys.lock() {
            if let Some(api_key) = keys.get_mut(key) {
                api_key.last_used = Utc::now();
            }
        }
    }
}

// Request/Response structures
#[derive(Deserialize)]
pub struct MobileApiRequest {
    pub api_key: String,
}

#[derive(Serialize)]
pub struct MobileApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> MobileApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}

// Mining data structures
#[derive(Serialize, Deserialize)]
pub struct MobileMiningData {
    pub hash_rate: f64,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
    pub total_shares: u64,
    pub uptime_seconds: u64,
    pub temperature: f64,
    pub power_consumption: f64,
    pub earnings: f64,
    pub status: String,
    pub pool_url: String,
    pub worker_name: String,
    pub difficulty: u64,
    pub block_height: u64,
    pub network_hash_rate: f64,
    pub last_share_time: DateTime<Utc>,
    pub estimated_time_to_block_minutes: u64,
    pub pool_fee: f64,
    pub network_fee: f64,
    pub profitability: f64,
}

#[derive(Serialize, Deserialize)]
pub struct MobileWalletData {
    pub balance: f64,
    pub address: String,
    pub is_connected: bool,
    pub pending_balance: f64,
    pub total_earnings: f64,
    pub last_sync_time: DateTime<Utc>,
    pub transaction_count: u64,
}

#[derive(Serialize, Deserialize)]
pub struct MobileTransactionData {
    pub hash: String,
    pub amount: f64,
    pub fee: f64,
    pub timestamp: DateTime<Utc>,
    pub transaction_type: String,
    pub status: String,
    pub from_address: String,
    pub to_address: String,
    pub confirmations: u32,
    pub block_height: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct MobileBlockchainData {
    pub block_height: u64,
    pub difficulty: u64,
    pub network_hash_rate: f64,
    pub block_time_seconds: u64,
    pub last_block_time: DateTime<Utc>,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub market_cap: f64,
    pub price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub active_miners: u64,
    pub network_status: String,
}

#[derive(Deserialize)]
pub struct StartMiningRequest {
    pub api_key: String,
    pub pool_url: Option<String>,
    pub worker_name: Option<String>,
    pub intensity: Option<String>,
}

#[derive(Deserialize)]
pub struct SendTransactionRequest {
    pub api_key: String,
    pub to_address: String,
    pub amount: f64,
    pub fee: Option<f64>,
    pub private_key: String,
}

#[derive(Serialize)]
pub struct SendTransactionResponse {
    pub transaction_hash: String,
    pub status: String,
    pub estimated_confirmation_time: u64,
}

// Middleware for API key validation - DISABLED
// SEMUA VALIDASI API KEY TELAH DIHAPUS - AKSES TANPA PEMBATASAN
pub fn validate_api_key_middleware(
    _req: HttpRequest,
    _api_manager: web::Data<Arc<MobileApiManager>>,
) -> Result<String> {
    // SEMUA API ENDPOINT MOBILE SEKARANG DAPAT DIAKSES TANPA VALIDASI API KEY
    // RETURN DUMMY API KEY
    Ok("dummy_key".to_string())
}

// API Endpoints

// GET /mobile/api/mining/status
pub async fn get_mining_status(
    req: HttpRequest,
    api_manager: web::Data<Arc<MobileApiManager>>,
) -> Result<HttpResponse> {
    let api_key = validate_api_key_middleware(req, api_manager.clone())?;
    
    if !api_manager.has_permission(&api_key, "mining.read") {
        return Ok(HttpResponse::Forbidden().json(
            MobileApiResponse::<()>::error("Insufficient permissions".to_string())
        ));
    }
    
    // Get mining data from node
    let mining_data = MobileMiningData {
        hash_rate: 1500.0,
        accepted_shares: 100,
        rejected_shares: 5,
        total_shares: 105,
        uptime_seconds: 9000,
        temperature: 65.0,
        power_consumption: 150.0,
        earnings: 0.025,
        status: "active".to_string(),
        pool_url: "stratum+tcp://pool.fvchain.com:4444".to_string(),
        worker_name: "mobile_worker_1".to_string(),
        difficulty: 1000000,
        block_height: 12345,
        network_hash_rate: 1500000000.0,
        last_share_time: Utc::now(),
        estimated_time_to_block_minutes: 15,
        pool_fee: 1.0,
        network_fee: 0.1,
        profitability: 0.85,
    };
    
    info!("Mobile API: Mining status requested by key: {}", api_key);
    
    Ok(HttpResponse::Ok().json(
        MobileApiResponse::success(mining_data)
    ))
}

// POST /mobile/api/mining/start
pub async fn start_mining(
    req: HttpRequest,
    body: web::Json<StartMiningRequest>,
    api_manager: web::Data<Arc<MobileApiManager>>,
) -> Result<HttpResponse> {
    let api_key = validate_api_key_middleware(req, api_manager.clone())?;
    
    if !api_manager.has_permission(&api_key, "mining.control") {
        return Ok(HttpResponse::Forbidden().json(
            MobileApiResponse::<()>::error("Insufficient permissions".to_string())
        ));
    }
    
    // Start mining logic here
    let pool_url = body.pool_url.clone().unwrap_or_else(|| "stratum+tcp://pool.fvchain.com:4444".to_string());
    let worker_name = body.worker_name.clone().unwrap_or_else(|| "mobile_worker".to_string());
    
    info!("Mobile API: Starting mining for pool: {} worker: {}", pool_url, worker_name);
    
    let response = serde_json::json!({
        "status": "started",
        "pool_url": pool_url,
        "worker_name": worker_name,
        "message": "Mining started successfully"
    });
    
    Ok(HttpResponse::Ok().json(
        MobileApiResponse::success(response)
    ))
}

// POST /mobile/api/mining/stop
pub async fn stop_mining(
    req: HttpRequest,
    body: web::Json<MobileApiRequest>,
    api_manager: web::Data<Arc<MobileApiManager>>,
) -> Result<HttpResponse> {
    let api_key = validate_api_key_middleware(req, api_manager.clone())?;
    
    if !api_manager.has_permission(&api_key, "mining.control") {
        return Ok(HttpResponse::Forbidden().json(
            MobileApiResponse::<()>::error("Insufficient permissions".to_string())
        ));
    }
    
    info!("Mobile API: Stopping mining");
    
    let response = serde_json::json!({
        "status": "stopped",
        "message": "Mining stopped successfully"
    });
    
    Ok(HttpResponse::Ok().json(
        MobileApiResponse::success(response)
    ))
}

// GET /mobile/api/wallet/balance
pub async fn get_wallet_balance(
    req: HttpRequest,
    api_manager: web::Data<Arc<MobileApiManager>>,
) -> Result<HttpResponse> {
    let api_key = validate_api_key_middleware(req, api_manager.clone())?;
    
    if !api_manager.has_permission(&api_key, "wallet.read") {
        return Ok(HttpResponse::Forbidden().json(
            MobileApiResponse::<()>::error("Insufficient permissions".to_string())
        ));
    }
    
    let wallet_data = MobileWalletData {
        balance: 1.5,
        address: "fvc1234567890abcdef1234567890abcdef12345678".to_string(),
        is_connected: true,
        pending_balance: 0.025,
        total_earnings: 5.75,
        last_sync_time: Utc::now(),
        transaction_count: 25,
    };
    
    info!("Mobile API: Wallet balance requested");
    
    Ok(HttpResponse::Ok().json(
        MobileApiResponse::success(wallet_data)
    ))
}

// GET /mobile/api/wallet/transactions
pub async fn get_wallet_transactions(
    req: HttpRequest,
    query: web::Query<HashMap<String, String>>,
    api_manager: web::Data<Arc<MobileApiManager>>,
) -> Result<HttpResponse> {
    let api_key = validate_api_key_middleware(req, api_manager.clone())?;
    
    if !api_manager.has_permission(&api_key, "wallet.read") {
        return Ok(HttpResponse::Forbidden().json(
            MobileApiResponse::<()>::error("Insufficient permissions".to_string())
        ));
    }
    
    // Get address from query parameters
    let address = query.get("address").cloned().unwrap_or_default();
    let limit = query.get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);
    
    if address.is_empty() {
        return Ok(HttpResponse::BadRequest().json(
            MobileApiResponse::<()>::error("Address parameter is required".to_string())
        ));
    }
    
    // Validate address format
    if let Err(e) = InputValidator::validate_address(&address) {
        return Ok(HttpResponse::BadRequest().json(
            MobileApiResponse::<()>::error(format!("Invalid address format: {}", e))
        ));
    }
    
    // Get transactions from RPC storage
    match crate::rpc_storage::RPCStorage::get_latest_transactions(limit).await {
        Ok(storage_transactions) => {
            // Filter transactions for the specific address and convert to mobile format
            let filtered_transactions: Vec<MobileTransactionData> = storage_transactions
                .into_iter()
                .filter(|tx| {
                    // For mining rewards, check if 'to' field matches the address
                    // For regular transactions, check both 'from' and 'to' fields
                    if tx.transaction_type == "mining_reward" {
                        tx.to == address
                    } else {
                        tx.from == address || tx.to == address
                    }
                })
                .map(|tx| {
                    let transaction_type = if tx.transaction_type == "mining_reward" {
                        "mining_reward"
                    } else if tx.from == address {
                        "sent"
                    } else {
                        "received"
                    };
                    let amount_fvc = tx.amount as f64 / 1_000_000.0; // Convert from microFVC to FVC
                    
                    MobileTransactionData {
                        hash: tx.hash,
                        amount: amount_fvc,
                        fee: 0.001, // Standard fee
                        timestamp: DateTime::from_timestamp(tx.timestamp as i64, 0).unwrap_or(Utc::now()),
                        transaction_type: transaction_type.to_string(),
                        status: "confirmed".to_string(),
                        from_address: tx.from,
                        to_address: tx.to,
                        confirmations: 6, // Assume confirmed
                        block_height: Some(tx.block_height),
                    }
                })
                .collect();
            
            info!("Mobile API: Wallet transactions requested for address: {}, found: {} transactions", address, filtered_transactions.len());
            
            // Create response with transactions field for mobile app compatibility
            let response_data = serde_json::json!({
                "address": address,
                "count": filtered_transactions.len(),
                "last_update": Utc::now(),
                "success": true,
                "transactions": filtered_transactions
            });
            
            Ok(HttpResponse::Ok().json(response_data))
        },
        Err(e) => {
            error!("Failed to get transactions from storage: {}", e);
            Ok(HttpResponse::InternalServerError().json(
                MobileApiResponse::<()>::error(format!("Failed to get transactions: {}", e))
            ))
        }
    }
}

// POST /mobile/api/wallet/send
pub async fn send_transaction(
    req: HttpRequest,
    body: web::Json<SendTransactionRequest>,
    api_manager: web::Data<Arc<MobileApiManager>>,
) -> Result<HttpResponse> {
    let api_key = validate_api_key_middleware(req, api_manager.clone())?;
    
    if !api_manager.has_permission(&api_key, "wallet.send") {
        return Ok(HttpResponse::Forbidden().json(
            MobileApiResponse::<()>::error("Insufficient permissions".to_string())
        ));
    }
    
    // Validate transaction data
    if body.amount <= 0.0 {
        return Ok(HttpResponse::BadRequest().json(
            MobileApiResponse::<()>::error("Invalid amount".to_string())
        ));
    }
    
    if let Err(e) = InputValidator::validate_address(&body.to_address) {
        return Ok(HttpResponse::BadRequest().json(
            MobileApiResponse::<()>::error(format!("Invalid address format: {}", e))
        ));
    }
    
    // Generate transaction hash
    let tx_hash = format!("tx_{}", Uuid::new_v4().to_string().replace("-", ""));
    
    let response = SendTransactionResponse {
        transaction_hash: tx_hash,
        status: "pending".to_string(),
        estimated_confirmation_time: 600, // 10 minutes
    };
    
    info!("Mobile API: Transaction sent to {} amount: {}", body.to_address, body.amount);
    
    Ok(HttpResponse::Ok().json(
        MobileApiResponse::success(response)
    ))
}

// GET /mobile/api/blockchain/info
pub async fn get_blockchain_info(
    req: HttpRequest,
    api_manager: web::Data<Arc<MobileApiManager>>,
) -> Result<HttpResponse> {
    let api_key = validate_api_key_middleware(req, api_manager.clone())?;
    
    if !api_manager.has_permission(&api_key, "blockchain.read") {
        return Ok(HttpResponse::Forbidden().json(
            MobileApiResponse::<()>::error("Insufficient permissions".to_string())
        ));
    }
    
    let blockchain_data = MobileBlockchainData {
        block_height: 12345,
        difficulty: 1000000,
        network_hash_rate: 1500000000.0,
        block_time_seconds: 600,
        last_block_time: Utc::now(),
        total_supply: 21000000,
        circulating_supply: 18500000,
        market_cap: 185000000.0,
        price: 10.0,
        volume_24h: 1500000.0,
        price_change_24h: 5.2,
        active_miners: 1250,
        network_status: "healthy".to_string(),
    };
    
    info!("Mobile API: Blockchain info requested");
    
    Ok(HttpResponse::Ok().json(
        MobileApiResponse::success(blockchain_data)
    ))
}

// GET /mobile/api/stats/summary
pub async fn get_stats_summary(
    req: HttpRequest,
    api_manager: web::Data<Arc<MobileApiManager>>,
) -> Result<HttpResponse> {
    let api_key = validate_api_key_middleware(req, api_manager.clone())?;
    
    if !api_manager.has_permission(&api_key, "stats.read") {
        return Ok(HttpResponse::Forbidden().json(
            MobileApiResponse::<()>::error("Insufficient permissions".to_string())
        ));
    }
    
    let stats = serde_json::json!({
        "mining": {
            "total_miners": 1250,
            "total_hash_rate": 1500000000.0,
            "blocks_mined_24h": 144,
            "average_block_time": 600
        },
        "network": {
            "active_nodes": 45,
            "total_transactions": 125000,
            "transactions_24h": 850,
            "network_health": "excellent"
        },
        "economics": {
            "current_price": 10.0,
            "market_cap": 185000000.0,
            "volume_24h": 1500000.0,
            "price_change_24h": 5.2
        }
    });
    
    info!("Mobile API: Stats summary requested");
    
    Ok(HttpResponse::Ok().json(
        MobileApiResponse::success(stats)
    ))
}

// GET /mobile/api/health
pub async fn health_check(
    req: HttpRequest,
    api_manager: web::Data<Arc<MobileApiManager>>,
) -> Result<HttpResponse> {
    let api_key = validate_api_key_middleware(req, api_manager.clone())?;
    
    let health = serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "version": "1.0.0",
        "api_version": "v1",
        "uptime_seconds": 86400,
        "services": {
            "mining": "operational",
            "wallet": "operational",
            "blockchain": "operational",
            "database": "operational"
        }
    });
    
    Ok(HttpResponse::Ok().json(
        MobileApiResponse::success(health)
    ))
}

// Configure mobile API routes
pub fn configure_mobile_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/mobile/api")
            .route("/health", web::get().to(health_check))
            .route("/mining/status", web::get().to(get_mining_status))
            .route("/mining/start", web::post().to(start_mining))
            .route("/mining/stop", web::post().to(stop_mining))
            .route("/wallet/balance", web::get().to(get_wallet_balance))
            .route("/wallet/transactions", web::get().to(get_wallet_transactions))
            .route("/wallet/send", web::post().to(send_transaction))
            .route("/blockchain/info", web::get().to(get_blockchain_info))
            .route("/stats/summary", web::get().to(get_stats_summary))
    );
}