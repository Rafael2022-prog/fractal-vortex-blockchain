use axum::{
    routing::{get, post, delete},
    Router,
    response::{Json, IntoResponse},
    extract::{State, Path, Query, rejection::QueryRejection, rejection::JsonRejection},
    http::StatusCode,
};
use std::collections::HashMap;
use serde::Deserialize;
use serde_json::json;
use chrono::Utc;
use fractal_vortex_chain::mining::auto_detection::{MiningAutoDetection, AutoDetectionConfig, HeartbeatRequest};
// Mobile API functionality is now integrated directly in this server

use fractal_vortex_chain::rpc_storage::{RPCStorage, WalletTransaction};
use fractal_vortex_chain::storage::StorageError;
use fractal_vortex_chain::node::fractal_node::{FractalNode, NodeConfig};


use serde_json::Value;

use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use axum::http::Method;
use once_cell::sync::Lazy;
use log::info;

use std::time::Duration;
use axum::response::sse::{Event, Sse};
use futures_util::stream::{Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use tokio::sync::broadcast;
use std::convert::Infallible;
use chrono;
use std::io::Write;
use libp2p::Multiaddr;
use fractal_vortex_chain::wallet::key_manager::KeyManager;
use std::fs::OpenOptions;
use hex;
use std::sync::atomic::{AtomicUsize, Ordering};

// Multi-node architecture with 4 integrated nodes
static BLOCKCHAIN_NODES: Lazy<Arc<tokio::sync::Mutex<Vec<Option<FractalNode>>>>> = Lazy::new(|| {
    Arc::new(tokio::sync::Mutex::new(vec![None, None, None, None]))
});

// Node health status tracking
static NODE_HEALTH: Lazy<Arc<tokio::sync::RwLock<Vec<bool>>>> = Lazy::new(|| {
    Arc::new(tokio::sync::RwLock::new(vec![false, false, false, false]))
});

// Global ecosystem miner instance
static GLOBAL_ECOSYSTEM_MINER: Lazy<Arc<tokio::sync::Mutex<Option<fractal_vortex_chain::node::ecosystem_miner::EcosystemMiner>>>> = Lazy::new(|| {
    Arc::new(tokio::sync::Mutex::new(None))
});

// Round-robin load balancer counter
static LOAD_BALANCER_COUNTER: AtomicUsize = AtomicUsize::new(0);

// NodeManager for managing 4 blockchain nodes
struct NodeManager {
    nodes: Arc<tokio::sync::Mutex<Vec<Option<FractalNode>>>>,
    health: Arc<tokio::sync::RwLock<Vec<bool>>>,
    counter: Arc<AtomicUsize>,
}

impl NodeManager {
    fn new() -> Self {
        Self {
            nodes: BLOCKCHAIN_NODES.clone(),
            health: NODE_HEALTH.clone(),
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    // Get next healthy node using round-robin load balancing
    async fn get_healthy_node(&self) -> Option<usize> {
        let health = self.health.read().await;
        let healthy_nodes: Vec<usize> = health.iter()
            .enumerate()
            .filter_map(|(i, &is_healthy)| if is_healthy { Some(i) } else { None })
            .collect();
        
        if healthy_nodes.is_empty() {
            return None;
        }
        
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);
        Some(healthy_nodes[counter % healthy_nodes.len()])
    }
    
    // Execute operation on a healthy node
    async fn execute_on_node<F, R>(&self, operation: F) -> Result<R, String>
    where
        F: Fn(&FractalNode) -> R,
    {
        if let Some(node_index) = self.get_healthy_node().await {
            let nodes = self.nodes.lock().await;
            if let Some(Some(node)) = nodes.get(node_index) {
                return Ok(operation(node));
            }
        }
        Err("No healthy nodes available".to_string())
    }
    
    // Update node health status
    async fn update_node_health(&self, node_index: usize, is_healthy: bool) {
        let mut health = self.health.write().await;
        if node_index < health.len() {
            health[node_index] = is_healthy;
        }
    }
    
    // Get node health statistics
    async fn get_health_stats(&self) -> (usize, usize) {
        let health = self.health.read().await;
        let healthy_count = health.iter().filter(|&&h| h).count();
        let total_count = health.len();
        (healthy_count, total_count)
    }
}

// Global NodeManager instance
static NODE_MANAGER: Lazy<NodeManager> = Lazy::new(|| NodeManager::new());

// Legacy compatibility - get primary node (node 0)
static BLOCKCHAIN_NODE: Lazy<Arc<tokio::sync::Mutex<Option<FractalNode>>>> = Lazy::new(|| {
    Arc::new(tokio::sync::Mutex::new(None))
});

// Global broadcast channel for real-time updates
static BROADCAST: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
    let (tx, _rx) = broadcast::channel(1000);
    tx
});

static AUTO_DETECTION: Lazy<Arc<MiningAutoDetection>> = Lazy::new(|| {
    let config = AutoDetectionConfig {
        heartbeat_timeout: Duration::from_secs(30),
        check_interval: Duration::from_secs(10),
        grace_period: Duration::from_secs(60),
        max_retry_attempts: 3,
    };
    Arc::new(MiningAutoDetection::new(config))
});

#[derive(Clone)]
struct AppState {
    latest_block: Arc<RwLock<u64>>,
    total_transactions: Arc<RwLock<u64>>,
    active_nodes: Arc<RwLock<u32>>,
    _genesis_timestamp: u64,
}

// Storage functions using LevelDB via RPCStorage
async fn get_block_height() -> u64 {
    RPCStorage::get_block_height().await.unwrap_or(1)
}

async fn get_transaction_count() -> u64 {
    RPCStorage::get_transaction_count().await.unwrap_or(0)
}

#[allow(dead_code)]
async fn add_transaction(tx: WalletTransaction) {
    let _ = RPCStorage::add_transaction(&tx).await;
}

async fn get_latest_transactions(limit: usize) -> Vec<WalletTransaction> {
    RPCStorage::get_latest_transactions(limit).await.unwrap_or_default()
}

// Advanced calculation functions from rpc-server - OFFICIAL SMART RATE IMPLEMENTATION
async fn calculate_vortex_energy_rate() -> f64 {
    let block_height = get_block_height().await;
    let transaction_count = get_transaction_count().await;
    
    if block_height == 0 {
        return 369.0; // Base fractal energy for genesis
    }
    
    let base_energy = 369.0; // Base fractal energy constant
    let vortex_pattern = [1.0, 2.0, 4.0, 8.0, 7.0, 5.0];
    let pattern_index = (block_height % 6) as usize;
    let pattern_multiplier = vortex_pattern[pattern_index];
    
    // Network activity factor
    let network_activity = (transaction_count as f64 / block_height as f64).min(10.0);
    
    base_energy * pattern_multiplier * (1.0 + network_activity * 0.1)
}

async fn calculate_fractal_contribution_score() -> f64 {
    let block_height = get_block_height().await;
    let transaction_count = get_transaction_count().await;
    
    if block_height == 0 {
        return 0.0;
    }
    
    let sierpinski_dimension = 1.585;
    let block_contribution = (block_height as f64).log2() * sierpinski_dimension;
    let tx_contribution = (transaction_count as f64).sqrt() * 0.1;
    
    (block_contribution + tx_contribution).min(100.0)
}

async fn calculate_mathematical_efficiency_index() -> f64 {
    let block_height = get_block_height().await;
    
    if block_height == 0 {
        return 100.0; // Genesis block efficiency
    }
    
    let target_block_time = 5.0; // 5 seconds target
    let current_time = chrono::Utc::now().timestamp() as f64;
    let genesis_time = 1640995200.0; // FVChain genesis timestamp
    
    // Calculate actual average block time
    let elapsed_time = current_time - genesis_time;
    let actual_avg_time = elapsed_time / block_height as f64;
    
    // Time efficiency calculation
    let time_efficiency = (target_block_time / actual_avg_time).min(2.0);
    
    (time_efficiency * 50.0).min(100.0)
}

async fn calculate_network_harmony_factor() -> f64 {
    let block_height = get_block_height().await;
    let active_nodes = check_active_nodes().await;
    
    if block_height == 0 || active_nodes == 0 {
        return 0.0;
    }
    
    let golden_ratio = 1.618033988749;
    let node_harmony = (active_nodes as f64).log2() * golden_ratio;
    
    let network_consistency = if block_height > 0 {
        let consistency_factor = (block_height as f64 / active_nodes.max(1) as f64).min(10.0);
        consistency_factor * 0.1
    } else {
        0.0
    };
    
    ((node_harmony + network_consistency) * 10.0).min(100.0)
}

async fn calculate_smart_rate() -> f64 {
    let block_height = get_block_height().await;
    
    // Calculate all Smart Rate components
    let ver = calculate_vortex_energy_rate().await;
    let fcs = calculate_fractal_contribution_score().await;
    let mei = calculate_mathematical_efficiency_index().await;
    let nhf = calculate_network_harmony_factor().await;
    
    // Normalize components to 0-1 scale
    let ver_normalized = (ver / 5000.0).min(1.0);
    let fcs_normalized = fcs / 100.0;
    let mei_normalized = mei / 100.0;
    let nhf_normalized = nhf / 100.0;
    
    // Calculate weighted geometric mean with official weights
    // VER: 35%, FCS: 25%, MEI: 25%, NHF: 15%
    let weighted_product = ver_normalized.powf(0.35) *
                          fcs_normalized.powf(0.25) *
                          mei_normalized.powf(0.25) *
                          nhf_normalized.powf(0.15);
    
    // Apply base Smart Rate and vortex pattern
    let base_smart_rate = 1000.0;
    let smart_rate = base_smart_rate * weighted_product;
    
    // Apply vortex pattern based on block height
    let vortex_pattern = [1.0, 1.2, 1.4, 1.8, 1.7, 1.5];
    let pattern_index = (block_height % 6) as usize;
    let pattern_multiplier = vortex_pattern[pattern_index];
    
    smart_rate * pattern_multiplier
}

pub async fn check_active_nodes() -> u32 {
    // Get healthy blockchain nodes count from NODE_HEALTH
    let health = NODE_HEALTH.read().await;
    let healthy_count = health.iter().filter(|&&h| h).count() as u32;
    
    // If no healthy nodes from multi-node setup, fallback to single node check
    if healthy_count == 0 {
        let node_guard = BLOCKCHAIN_NODE.lock().await;
        if node_guard.is_some() { 1 } else { 0 }
    } else {
        healthy_count
    }
}

// Production mining function
async fn start_production_mining(device_id: &str, mining_address: &str) -> Result<(), String> {
    use fractal_vortex_chain::node::ecosystem_miner::EcosystemMiner;
    use fractal_vortex_chain::consensus::vortex_consensus::VortexConsensus;
    use fractal_vortex_chain::node::fractal_node::NodeState;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    
    let mut global_miner = GLOBAL_ECOSYSTEM_MINER.lock().await;
    
    // Check if miner is already running
    if let Some(ref miner) = *global_miner {
        if miner.is_running() {
            info!("Production mining already running for device: {} with address: {}", device_id, mining_address);
            return Ok(());
        }
    }
    
    // Create consensus and state for miner
    let consensus = Arc::new(RwLock::new(VortexConsensus::new(1000.0)));
    let state = Arc::new(RwLock::new(NodeState {
        is_validator: false,
        current_epoch: 0,
        vortex_energy: 1.0,
        connected_peers: Vec::new(),
        last_sync: 0,
        block_height: 0,
        total_transactions: 0,
    }));
    
    // Create and start ecosystem miner
    match EcosystemMiner::new(consensus, state).await {
        Ok(miner) => {
            match miner.start().await {
                Ok(_) => {
                    info!("Production mining started for device: {} with address: {}", device_id, mining_address);
                    *global_miner = Some(miner);
                    Ok(())
                },
                Err(e) => {
                    log::error!("Failed to start production mining: {}", e);
                    Err(format!("Mining start failed: {}", e))
                }
            }
        },
        Err(e) => {
            log::error!("Failed to create ecosystem miner: {}", e);
            Err(format!("Miner creation failed: {}", e))
        }
    }
}

#[derive(Deserialize)]
struct StartMinerRequest {
    address: String,
    device_id: String,
}

async fn start_miner(payload: Result<Json<StartMinerRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(req)) => start_miner_impl(req).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

async fn start_miner_impl(payload: StartMinerRequest) -> Json<Value> {
    match start_production_mining(&payload.device_id, &payload.address).await {
        Ok(_) => {
            // Save device session to track mining status
            let session_id = format!("session_{}_{}", payload.device_id, Utc::now().timestamp());
            let current_time = Utc::now().timestamp() as u64;
            
            if let Err(e) = RPCStorage::set_device_session(&payload.device_id, &session_id, current_time).await {
                log::error!("Failed to save device session: {}", e);
            }
            
            // Save device address for mining rewards
            if let Err(e) = RPCStorage::set_device_address(&payload.device_id, &payload.address).await {
                log::error!("Failed to save device address: {}", e);
            }
            
            // Register device to AUTO_DETECTION if not already registered
            AUTO_DETECTION.register_device(
                payload.device_id.clone(),
                session_id.clone(),
                payload.address.clone()
            ).await;
            
            // Start mining in AUTO_DETECTION
            let mining_started = AUTO_DETECTION.start_mining(&payload.device_id).await;
            if !mining_started {
                log::warn!("Failed to start mining in AUTO_DETECTION for device: {}", payload.device_id);
            }
            
            Json(json!({
                "success": true,
                "message": "Production mining started successfully",
                "device_id": payload.device_id,
                "mining_address": payload.address,
                "session_token": session_id
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": e
            }))
        }
    }
}

// Production mining stop function
async fn stop_production_mining(device_id: &str) -> Result<(), String> {
    let mut global_miner = GLOBAL_ECOSYSTEM_MINER.lock().await;
    
    if let Some(miner) = global_miner.take() {
        match miner.stop().await {
            Ok(_) => {
                info!("Production mining stopped for device: {}", device_id);
                Ok(())
            },
            Err(e) => {
                log::error!("Failed to stop production mining: {}", e);
                Err(format!("Mining stop failed: {}", e))
            }
        }
    } else {
        info!("No active mining to stop for device: {}", device_id);
        Ok(())
    }
}

#[derive(Deserialize)]
struct StopMinerRequest {
    device_id: String,
    #[allow(dead_code)]
    session_token: String,
}

#[derive(Deserialize)]
struct ResetMinerRequest {
    device_id: String,
}

async fn stop_miner(payload: Result<Json<StopMinerRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(req)) => stop_miner_impl(req).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

async fn stop_miner_impl(payload: StopMinerRequest) -> Json<Value> {
    // Stop production mining
    match stop_production_mining(&payload.device_id).await {
        Ok(_) => {
            // Remove device session to stop mining status
            if let Err(e) = RPCStorage::remove_device_session(&payload.device_id).await {
                log::error!("Failed to remove device session: {}", e);
            }
            
            // Stop mining in AUTO_DETECTION
            let mining_stopped = AUTO_DETECTION.stop_mining(&payload.device_id).await;
            if !mining_stopped {
                log::warn!("Failed to stop mining in AUTO_DETECTION for device: {}", payload.device_id);
            }
            
            Json(json!({
                "success": true,
                "message": "Production mining stopped successfully",
                "device_id": payload.device_id
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": e,
                "device_id": payload.device_id
            }))
        }
    }
}

#[allow(dead_code)]
async fn reset_miner(payload: Result<Json<ResetMinerRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(req)) => reset_miner_impl(req).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

#[allow(dead_code)]
async fn reset_miner_impl(payload: ResetMinerRequest) -> Json<Value> {
    let device_id = &payload.device_id;
    
    // Reset failed attempts and lockout
    if let Err(e) = RPCStorage::set_device_failed_attempts(device_id, 0).await {
        log::error!("Failed to reset failed attempts for device {}: {}", device_id, e);
    }
    
    if let Err(e) = RPCStorage::set_device_lockout(device_id, 0).await {
        log::error!("Failed to reset lockout for device {}: {}", device_id, e);
    }
    
    // Clear mining session to reset mining status
    if let Err(e) = RPCStorage::remove_device_session(device_id).await {
        log::error!("Failed to remove device session for {}: {}", device_id, e);
    }
    
    Json(json!({
        "success": true,
        "message": "Miner reset successfully - session cleared",
        "device_id": device_id
    }))
}

// Blockchain info endpoint
#[allow(dead_code)]
async fn get_blockchain_info(State(state): State<AppState>) -> Json<Value> {
    let latest_block = *state.latest_block.read().await;
    let total_transactions = *state.total_transactions.read().await;
    let active_nodes = *state.active_nodes.read().await;
    
    Json(json!({
        "latest_block": latest_block,
        "total_transactions": total_transactions,
        "active_nodes": active_nodes,
        "network_status": "active",
        "smart_rate": calculate_smart_rate().await,
        "vortex_energy_rate": calculate_vortex_energy_rate().await,
        "fractal_contribution_score": calculate_fractal_contribution_score().await
    }))
}

#[derive(Deserialize)]
struct LimitQuery {
    #[serde(deserialize_with = "deserialize_limit")]
    limit: Option<usize>,
}

fn deserialize_limit<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct LimitVisitor;

    impl<'de> Visitor<'de> for LimitVisitor {
        type Value = Option<usize>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a positive integer or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(LimitValueVisitor)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    struct LimitValueVisitor;

    impl<'de> Visitor<'de> for LimitValueVisitor {
        type Value = Option<usize>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a positive integer")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value < 0 {
                Err(E::custom("limit must be non-negative"))
            } else if value > 1000 {
                Ok(Some(1000))
            } else {
                Ok(Some(value as usize))
            }
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value > 1000 {
                Ok(Some(1000))
            } else {
                Ok(Some(value as usize))
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value.parse::<usize>() {
                Ok(num) => {
                    if num > 1000 {
                        Ok(Some(1000))
                    } else {
                        Ok(Some(num))
                    }
                }
                Err(_) => Err(E::custom("invalid number format")),
            }
        }
    }

    deserializer.deserialize_option(LimitVisitor)
}

fn handle_json_rejection(rejection: JsonRejection) -> (StatusCode, Json<Value>) {
    let (status, error_message) = match rejection {
        JsonRejection::JsonDataError(err) => {
            (StatusCode::BAD_REQUEST, format!("Invalid JSON data: {}", err))
        }
        JsonRejection::JsonSyntaxError(err) => {
            (StatusCode::BAD_REQUEST, format!("JSON syntax error: {}", err))
        }
        JsonRejection::MissingJsonContentType(_) => {
            (StatusCode::BAD_REQUEST, "Missing 'Content-Type: application/json' header".to_string())
        }
        JsonRejection::BytesRejection(err) => {
            (StatusCode::BAD_REQUEST, format!("Failed to read request body: {}", err))
        }
        _ => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Unknown JSON processing error".to_string())
        }
    };

    let json_response = Json(json!({
        "success": false,
        "error": error_message
    }));

    (status, json_response)
}

async fn get_blocks(State(_state): State<AppState>, query: Result<Query<LimitQuery>, QueryRejection>) -> impl IntoResponse {
    let limit = match query {
        Ok(Query(q)) => q.limit.unwrap_or(10),
        Err(_) => 10,
    };

    let capped_limit = std::cmp::min(limit, 100);
    
    match RPCStorage::get_latest_blocks(capped_limit).await {
        Ok(blocks) => {
            // Calculate Smart Rate and vPoW indicators once for all blocks
            let ver = calculate_vortex_energy_rate().await;
            let fcs = calculate_fractal_contribution_score().await;
            let mei = calculate_mathematical_efficiency_index().await;
            let nhf = calculate_network_harmony_factor().await;
            let smart_rate = calculate_smart_rate().await;
            
            let block_data: Vec<Value> = blocks.into_iter().map(|block| {
                json!({
                    "height": block.height,
                    "hash": block.hash,
                    "previous_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                    "timestamp": block.timestamp,
                    "transactions": block.transactions,
                    "transaction_count": block.transaction_count,
                    "miner": block.miner,
                    "difficulty": block.difficulty,
                    "nonce": block.nonce,
                    "block_time": 5.0,
                    "smart_rate": smart_rate,
                    "vortex_energy_rate": ver,
                    "fractal_contribution_score": fcs,
                    "mathematical_efficiency_index": mei,
                    "network_harmony_factor": nhf
                })
            }).collect();
            
            Json(json!({
                "success": true,
                "blocks": block_data,
                "count": block_data.len()
            })).into_response()
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to retrieve blocks: {}", e)
            })).into_response()
        }
    }
}

async fn get_transactions(State(_state): State<AppState>, query: Result<Query<LimitQuery>, QueryRejection>) -> impl IntoResponse {
    let limit = match query {
        Ok(Query(q)) => q.limit.unwrap_or(10),
        Err(_) => 10,
    };

    let capped_limit = std::cmp::min(limit, 100);
    let transactions = get_latest_transactions(capped_limit).await;
    
    Json(json!({
        "success": true,
        "transactions": transactions,
        "count": transactions.len()
    })).into_response()
}

#[allow(dead_code)]
async fn get_transaction(Path(hash): Path<String>) -> Json<Value> {
    match RPCStorage::get_transaction(&hash).await {
        Ok(Some(tx)) => Json(json!({ "success": true, "transaction": tx })),
        Ok(None) => Json(json!({ "success": false, "error": "Transaction not found" })),
        Err(e) => Json(json!({ "success": false, "error": format!("Database error: {}", e) }))
    }
}

#[allow(dead_code)]
async fn get_block(Path(height): Path<u64>) -> Json<Value> {
    match RPCStorage::get_block_by_height(height).await {
        Ok(Some(block)) => {
            Json(json!({
                "success": true,
                "block": {
                    "height": block.height,
                    "hash": block.hash,
                    "previous_hash": block.parent_hash,
                    "timestamp": block.timestamp,
                    "transactions": block.transactions,
                    "miner": block.miner,
                    "difficulty": block.difficulty,
                    "nonce": block.nonce
                }
            }))
        },
        Ok(None) => {
            Json(json!({
                "success": false,
                "error": "Block not found"
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "found": false,
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

#[allow(dead_code)]
async fn verify_block_hash(Path(height): Path<u64>) -> Json<Value> {
    match RPCStorage::get_block_by_height(height).await {
        Ok(Some(block)) => {
            use fractal_vortex_chain::crypto::fractal_hash::FractalHasher;
            
            let mut hasher = FractalHasher::new(3);
            
            // Serialize block data for hashing
            let mut block_data = Vec::new();
            block_data.extend_from_slice(&height.to_le_bytes());
            block_data.extend_from_slice(block.parent_hash.as_bytes());
            block_data.extend_from_slice(&block.timestamp.to_le_bytes());
            
            // Serialize transactions
            for tx in &block.transactions {
                block_data.extend_from_slice(tx.hash.as_bytes());
            }
            
            block_data.extend_from_slice(block.miner.as_bytes());
            block_data.extend_from_slice(&block.difficulty.to_le_bytes());
            block_data.extend_from_slice(&block.nonce.to_le_bytes());
            
            let vortex_hash = hasher.fractal_hash(&block_data);
            let calculated_hash = format!("0x{}", hex::encode(vortex_hash.fractal_hash));
            
            let is_valid = calculated_hash == block.hash;
            
            Json(json!({
                "success": true,
                "height": height,
                "stored_hash": block.hash,
                "calculated_hash": calculated_hash,
                "is_valid": is_valid
            }))
        },
        Ok(None) => {
            Json(json!({
                "success": false,
                "error": "Block not found"
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

#[allow(dead_code)]
async fn verify_all_blocks() -> Json<Value> {
    let current_height = get_block_height().await;
    let mut verification_results = Vec::new();
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for height in 1..=current_height {
        match RPCStorage::get_block_by_height(height).await {
            Ok(Some(block)) => {
                use fractal_vortex_chain::crypto::fractal_hash::FractalHasher;
                
                let mut hasher = FractalHasher::new(3);
                // Serialize block data for hashing
                let mut block_data = Vec::new();
                block_data.extend_from_slice(&height.to_le_bytes());
                block_data.extend_from_slice(block.parent_hash.as_bytes());
                block_data.extend_from_slice(&block.timestamp.to_le_bytes());
                block_data.extend_from_slice(&block.nonce.to_le_bytes());
                block_data.extend_from_slice(&block.difficulty.to_le_bytes());
                block_data.extend_from_slice(block.miner.as_bytes());
                
                let calculated_hash = hasher.fractal_hash(&block_data);
                
                let calculated_hash_string = format!("0x{}", calculated_hash.fractal_hash.iter().map(|b| format!("{:02x}", b)).collect::<String>());
                let is_valid = calculated_hash_string == block.hash;
                
                if is_valid {
                    valid_count += 1;
                } else {
                    invalid_count += 1;
                    verification_results.push(json!({
                        "height": height,
                        "stored_hash": block.hash,
                        "calculated_hash": calculated_hash_string,
                        "is_valid": false
                    }));
                }
            },
            Ok(None) => {
                invalid_count += 1;
                verification_results.push(json!({
                    "height": height,
                    "error": "Block not found",
                    "is_valid": false
                }));
            },
            Err(e) => {
                invalid_count += 1;
                verification_results.push(json!({
                    "height": height,
                    "error": format!("Database error: {}", e),
                    "is_valid": false
                }));
            }
        }
    }
    
    Json(json!({
        "success": true,
        "total_blocks": current_height,
        "valid_blocks": valid_count,
        "invalid_blocks": invalid_count,
        "invalid_details": verification_results
    }))
}

// SSE endpoints
async fn sse_endpoint() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = BROADCAST.subscribe();
    let stream = BroadcastStream::new(rx)
        .map(|result| {
            match result {
                Ok(data) => Ok(Event::default().data(data)),
                Err(_) => Ok(Event::default().data("heartbeat")),
            }
        });
    
    Sse::new(stream)
}

async fn blocks_sse_endpoint() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = BROADCAST.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|result| async move {
            match result {
                Ok(data) => {
                    if data.contains("new_block") {
                        Some(Ok(Event::default().data(data)))
                    } else {
                        None
                    }
                },
                Err(_) => None,
            }
        });
    
    Sse::new(stream)
}

async fn transactions_sse_endpoint() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = BROADCAST.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|result| async move {
            match result {
                Ok(data) => {
                    if data.contains("new_transaction") {
                        Some(Ok(Event::default().data(data)))
                    } else {
                        None
                    }
                },
                Err(_) => None,
            }
        });
    
    Sse::new(stream)
}

// Initialize blockchain node
// Initialize 4 blockchain nodes with different configurations
async fn initialize_blockchain_nodes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Initializing blockchain nodes configuration...");
    let base_p2p_port: u16 = std::env::var("P2P_PORT")
        .unwrap_or_else(|_| "30333".to_string())
        .parse()
        .unwrap_or(30333);
    let mining_address = std::env::var("MINING_ADDRESS")
        .unwrap_or_else(|_| "FVCminer1234567890abcdef".to_string());

    println!("🔒 Acquiring node locks...");
    let mut nodes_guard = BLOCKCHAIN_NODES.lock().await;
    let mut health_guard = NODE_HEALTH.write().await;
    println!("✅ Node locks acquired successfully");
    
    // Initialize 4 nodes with different configurations
    println!("🚀 Starting initialization of 4 blockchain nodes...");
    for i in 0usize..4 {
        let node_id = format!("node-{}", i);
        let p2p_port = base_p2p_port + i as u16;
        println!("🔄 Initializing {} on port {}...", node_id, p2p_port);
        
        let bootstrap_nodes = if i == 0 { 
            vec![] 
        } else { 
            // Connect subsequent nodes to the first node
            vec![format!("/ip4/127.0.0.1/tcp/{}", base_p2p_port).parse::<Multiaddr>()?]
        };
        
        let bootstrap_count = bootstrap_nodes.len();
        
        let config = NodeConfig {
            listen_addr: format!("/ip4/0.0.0.0/tcp/{}", p2p_port).parse::<Multiaddr>()?,
            bootstrap_nodes,
            energy_threshold: 1000.0 + (i as f64 * 100.0), // Different thresholds
            fractal_levels: 5 + (i % 3) as u32, // Varying fractal levels (5-7)
            max_peers: 50 + (i * 10), // Different peer limits
            sync_interval: 30 + (i * 5) as u64, // Different sync intervals
        };

        match FractalNode::new(config).await {
            Ok(node) => {
                // Start production mining for each node
                use fractal_vortex_chain::node::ecosystem_miner::EcosystemMiner;
                let consensus = node.get_consensus();
                let state = node.get_state();
                
                if let Ok(miner) = EcosystemMiner::new(consensus, state).await {
                    if let Err(e) = miner.start().await {
                        log::warn!("Failed to start mining for {}: {}", node_id, e);
                    }
                }
                
                println!("✅ Blockchain {} initialized on port {}", node_id, p2p_port);
                println!("🔗 {} connected to network with {} bootstrap nodes", node_id, bootstrap_count);
                info!("✅ Blockchain {} initialized on port {}", node_id, p2p_port);
                info!("🔗 {} connected to network with {} bootstrap nodes", node_id, bootstrap_count);
                
                nodes_guard[i] = Some(node);
                health_guard[i] = true;
                
                // Set primary node for legacy compatibility
                if i == 0 {
                    let mut primary_guard = BLOCKCHAIN_NODE.lock().await;
                    *primary_guard = nodes_guard[0].clone();
                }
            }
            Err(e) => {
                log::error!("❌ Failed to initialize {}: {}", node_id, e);
                health_guard[i] = false;
            }
        }
    }
    
    // Calculate health stats directly without using NODE_MANAGER to avoid deadlock
    let healthy_count = health_guard.iter().filter(|&&h| h).count();
    let total_count = health_guard.len();
    
    // Update cached active nodes count
    let active_nodes = if healthy_count > 0 { healthy_count as u32 } else { 1 };
    fractal_vortex_chain::node_health::update_active_nodes_count(active_nodes);
    
    println!("🚀 Multi-node initialization complete: {}/{} nodes healthy", healthy_count, total_count);
    println!("⚡ Production mining started on {} nodes with address: {}", healthy_count, mining_address);
    info!("🚀 Multi-node initialization complete: {}/{} nodes healthy", healthy_count, total_count);
    info!("⚡ Production mining started on {} nodes with address: {}", healthy_count, mining_address);
    info!("📊 Initial active nodes count: {}", active_nodes);
    
    // Start health monitoring task
    tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            monitor_node_health().await;
        }
    });
    
    Ok(())
}

// Legacy compatibility function
#[allow(dead_code)]
async fn initialize_blockchain_node() -> Result<(), Box<dyn std::error::Error>> {
    initialize_blockchain_nodes().await
}

// Health monitoring for all nodes
async fn monitor_node_health() {
    let nodes = BLOCKCHAIN_NODES.lock().await;
    let mut health_guard = NODE_HEALTH.write().await;
    let mut health_changed = false;
    
    for (i, node_opt) in nodes.iter().enumerate() {
        let is_healthy = match node_opt {
            Some(node) => {
                // Check if node is responsive (simplified health check)
                // In production, this would include more comprehensive checks
                let _state = node.get_state();
                // If we can get the state, node is considered healthy
                true
            }
            None => false,
        };
        
        if health_guard[i] != is_healthy {
            health_guard[i] = is_healthy;
            health_changed = true;
            let status = if is_healthy { "HEALTHY" } else { "UNHEALTHY" };
            log::info!("🔄 Node-{} status changed to: {}", i, status);
        }
    }
    
    // Update cached active nodes count if health changed
    if health_changed {
        let healthy_count = health_guard.iter().filter(|&&h| h).count() as u32;
        let active_nodes = if healthy_count > 0 { healthy_count } else { 1 };
        fractal_vortex_chain::node_health::update_active_nodes_count(active_nodes);
        log::info!("📊 Updated active nodes count: {}", active_nodes);
    }
}

// Device endpoints
async fn device_verify(State(_state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let device_id = payload["device_id"].as_str().unwrap_or("unknown");
    let address = payload["address"].as_str().unwrap_or("");
    
    // Check if device has access to this wallet
    let authorized = match RPCStorage::get_device_wallet(device_id).await {
        Ok(Some(wallet_data)) => {
            // Check if the wallet address matches
            wallet_data.get("address").and_then(|v| v.as_str()).unwrap_or("") == address
        },
        Ok(None) => {
            // If no wallet data found, allow access for now (backward compatibility)
            true
        },
        Err(_) => {
            // If database error, allow access for now (backward compatibility)
            true
        }
    };
    
    Json(json!({
        "success": true,
        "device_id": device_id,
        "address": address,
        "verified": true,
        "authorized": authorized
    }))
}

async fn device_validate(State(_state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let device_id = payload["device_id"].as_str().unwrap_or("unknown");
    
    Json(json!({
        "success": true,
        "device_id": device_id,
        "valid": true
    }))
}

async fn device_wallet(State(_state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let device_id = payload["device_id"].as_str().unwrap_or("unknown");
    
    Json(json!({
        "success": true,
        "device_id": device_id,
        "wallet_status": "active"
    }))
}

async fn device_register(State(_state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let device_id = payload["device_id"].as_str().unwrap_or("unknown");
    let device_name = payload["device_name"].as_str().unwrap_or("Unknown Device");
    
    // Log device registration
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("device_registrations.log")
    {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let _ = writeln!(file, "[{}] Device registered: {} ({})", timestamp, device_id, device_name);
    }
    
    Json(json!({
        "success": true,
        "device_id": device_id,
        "device_name": device_name,
        "registered": true,
        "timestamp": Utc::now().timestamp()
    }))
}

#[allow(dead_code)]
async fn device_get_wallet(State(_state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let device_id = payload["device_id"].as_str().unwrap_or("unknown");
    
    match RPCStorage::get_device_wallet(device_id).await {
        Ok(Some(wallet_data)) => {
            Json(json!({
                "success": true,
                "device_id": device_id,
                "wallet": wallet_data
            }))
        },
        Ok(None) => {
            Json(json!({
                "success": false,
                "error": "Wallet not found for device",
                "device_id": device_id
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Database error: {}", e),
                "device_id": device_id
            }))
        }
    }
}

#[allow(dead_code)]
async fn device_save_wallet(State(_state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let device_id = payload["device_id"].as_str().unwrap_or("unknown");
    let wallet_data = payload["wallet"].clone();
    
    match RPCStorage::save_device_wallet(device_id, &wallet_data.to_string()).await {
        Ok(_) => {
            Json(json!({
                "success": true,
                "device_id": device_id,
                "message": "Wallet saved successfully"
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to save wallet: {}", e),
                "device_id": device_id
            }))
        }
    }
}

// Save device wallet address (used by mobile mining app)
async fn device_save_wallet_address(State(_state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let device_id = payload["device_id"].as_str().unwrap_or("");
    let wallet_address = payload["wallet_address"].as_str().unwrap_or("");
    
    if device_id.is_empty() {
        return Json(json!({
            "success": false,
            "error": "device_id is required"
        }));
    }
    
    if wallet_address.is_empty() {
        return Json(json!({
            "success": false,
            "error": "wallet_address is required"
        }));
    }
    
    // Validate FVChain native address format (43 characters: fvc + 36 hex + emyl)
    if wallet_address.len() != 43 {
        return Json(json!({
            "success": false,
            "error": "Invalid wallet address format: must be 43 characters"
        }));
    }
    
    if !wallet_address.starts_with("fvc") {
        return Json(json!({
            "success": false,
            "error": "Invalid wallet address format: must start with 'fvc'"
        }));
    }
    
    if !wallet_address.ends_with("emyl") {
        return Json(json!({
            "success": false,
            "error": "Invalid wallet address format: must end with 'emyl'"
        }));
    }
    
    // Check middle part is hexadecimal
    let middle_part = &wallet_address[3..39];
    if !middle_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Json(json!({
            "success": false,
            "error": "Invalid wallet address format: middle part must be hexadecimal"
        }));
    }
    
    // Save device address mapping
    match RPCStorage::set_device_address(device_id, wallet_address).await {
        Ok(_) => {
            Json(json!({
                "success": true,
                "device_id": device_id,
                "wallet_address": wallet_address,
                "message": "Device wallet address saved successfully"
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to save device address: {}", e),
                "device_id": device_id
            }))
        }
    }
}

#[derive(Deserialize)]
struct StorePrivateKeyRequest {
    encrypted_private_key: String,
    #[allow(dead_code)]
    metadata: Option<serde_json::Value>,
}

#[allow(dead_code)]
async fn device_store_private_key(Path(device_id): Path<String>, Json(payload): Json<StorePrivateKeyRequest>) -> Json<Value> {
    match RPCStorage::set_device_private_key(&device_id, &payload.encrypted_private_key).await {
        Ok(_) => {
            Json(json!({
                "success": true,
                "message": "Private key stored successfully",
                "device_id": device_id
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to store private key: {}", e)
            }))
        }
    }
}

#[allow(dead_code)]
async fn device_get_private_key(Path(device_id): Path<String>) -> Json<Value> {
    match RPCStorage::get_device_private_key(&device_id).await {
        Ok(Some(encrypted_private_key)) => {
            Json(json!({
                "success": true,
                "encrypted_private_key": encrypted_private_key,
                "device_id": device_id
            }))
        },
        Ok(None) => {
            Json(json!({
                "success": false,
                "error": "Private key not found for device"
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to get private key: {}", e)
            }))
        }
    }
}

#[allow(dead_code)]
async fn device_has_private_key(Path(device_id): Path<String>) -> Json<Value> {
    match RPCStorage::get_device_private_key(&device_id).await {
        Ok(Some(_)) => {
            Json(json!({
                "success": true,
                "has_private_key": true,
                "device_id": device_id
            }))
        },
        Ok(None) => {
            Json(json!({
                "success": true,
                "has_private_key": false,
                "device_id": device_id
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to check private key: {}", e)
            }))
        }
    }
}

#[allow(dead_code)]
async fn device_remove_private_key(Path(device_id): Path<String>) -> Json<Value> {
    match RPCStorage::remove_device_private_key(&device_id).await {
        Ok(_) => {
            Json(json!({
                "success": true,
                "message": "Private key removed successfully",
                "device_id": device_id
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to remove private key: {}", e)
            }))
        }
    }
}

#[derive(Deserialize)]
struct MigratePlaintextRequest {
    plaintext_private_key: String,
    #[allow(dead_code)]
    pin_hash: String,
}

#[allow(dead_code)]
async fn device_migrate_plaintext(Path(device_id): Path<String>, Json(payload): Json<MigratePlaintextRequest>) -> Json<Value> {
    // This would encrypt the plaintext private key and store it
    // For now, we'll just store it as-is (in production, implement proper encryption)
    match RPCStorage::set_device_private_key(&device_id, &payload.plaintext_private_key).await {
        Ok(_) => {
            Json(json!({
                "success": true,
                "message": "Private key migrated successfully",
                "device_id": device_id
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to migrate private key: {}", e)
            }))
        }
    }
}

#[allow(dead_code)]
async fn device_clear_data(Path(device_id): Path<String>) -> Json<Value> {
    // Clear all device data including private key, wallet data, sessions, etc.
    let mut errors = Vec::new();
    
    // Remove private key
    if let Err(e) = RPCStorage::remove_device_private_key(&device_id).await {
        errors.push(format!("Failed to remove private key: {}", e));
    }
    
    // Remove wallet data
    if let Err(e) = RPCStorage::remove_device_wallet(&device_id).await {
        errors.push(format!("Failed to remove wallet data: {}", e));
    }
    
    // Remove session
    if let Err(e) = RPCStorage::remove_device_session(&device_id).await {
        errors.push(format!("Failed to remove session: {}", e));
    }
    
    // Remove address
    if let Err(e) = RPCStorage::remove_device_address(&device_id).await {
        errors.push(format!("Failed to remove address: {}", e));
    }
    
    if errors.is_empty() {
        Json(json!({
            "success": true,
            "message": "All device data cleared successfully",
            "device_id": device_id
        }))
    } else {
        Json(json!({
            "success": false,
            "error": format!("Some operations failed: {}", errors.join(", ")),
            "device_id": device_id
        }))
    }
}

#[derive(Deserialize)]
struct DeviceSessionRequest {
    device_id: String,
}

#[allow(dead_code)]
async fn device_get_session(payload: Result<Json<DeviceSessionRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(req)) => device_get_session_impl(req).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

#[allow(dead_code)]
async fn device_get_session_impl(payload: DeviceSessionRequest) -> Json<Value> {
    let device_id = &payload.device_id;
    
    match RPCStorage::get_device_session(device_id).await {
        Ok(Some((session_id, timestamp))) => {
            let current_time = Utc::now().timestamp() as u64;
            let session_age = current_time.saturating_sub(timestamp);
            let session_valid = session_age < 86400; // 24 hours
            
            if session_valid {
                Json(json!({
                    "success": true,
                    "found": true,
                    "device_id": device_id,
                    "session_token": session_id,
                    "timestamp": timestamp,
                    "valid": true
                }))
            } else {
                // Session expired, create new one
                let new_session_id = format!("session_{}_{}", device_id, current_time);
                match RPCStorage::set_device_session(device_id, &new_session_id, current_time).await {
                    Ok(_) => {
                        Json(json!({
                            "success": true,
                            "found": true,
                            "device_id": device_id,
                            "session_token": new_session_id,
                            "timestamp": current_time,
                            "valid": true,
                            "renewed": true
                        }))
                    },
                    Err(e) => {
                        Json(json!({
                            "success": false,
                            "found": false,
                            "error": format!("Failed to create new session: {}", e)
                        }))
                    }
                }
            }
        },
        Ok(None) => {
            // No session exists, create new one
            let current_time = Utc::now().timestamp() as u64;
            let new_session_id = format!("session_{}_{}", device_id, current_time);
            match RPCStorage::set_device_session(device_id, &new_session_id, current_time).await {
                Ok(_) => {
                    Json(json!({
                        "success": true,
                        "found": true,
                        "device_id": device_id,
                        "session_token": new_session_id,
                        "timestamp": current_time,
                        "valid": true,
                        "created": true
                    }))
                },
                Err(e) => {
                    Json(json!({
                        "success": false,
                        "found": false,
                        "error": format!("Failed to create session: {}", e)
                    }))
                }
            }
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

// Get device FVC address by device_id
async fn device_get_address(Query(params): Query<std::collections::HashMap<String, String>>) -> Json<Value> {
    let device_id = params.get("device_id").cloned().unwrap_or_default();
    
    if device_id.is_empty() {
        return Json(json!({
            "success": false,
            "error": "device_id parameter is required"
        }));
    }
    
    match RPCStorage::get_device_address(&device_id).await {
        Ok(Some(address)) => {
            Json(json!({
                "success": true,
                "device_id": device_id,
                "address": address
            }))
        },
        Ok(None) => {
            Json(json!({
                "success": false,
                "error": "No address found for device",
                "device_id": device_id
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

#[allow(dead_code)]
async fn device_get_session_by_id(Path(device_id): Path<String>) -> Json<Value> {
    match RPCStorage::get_device_session(&device_id).await {
        Ok(Some((session_id, timestamp))) => {
            let current_time = Utc::now().timestamp() as u64;
            let session_age = current_time.saturating_sub(timestamp);
            let session_valid = session_age < 86400; // 24 hours
            
            Json(json!({
                "success": true,
                "device_id": device_id,
                "session_id": session_id,
                "timestamp": timestamp,
                "valid": session_valid,
                "age_seconds": session_age
            }))
        },
        Ok(None) => {
            Json(json!({
                "success": false,
                "error": "No session found for device",
                "device_id": device_id
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

// PIN management endpoints
#[allow(dead_code)]
async fn device_pin_status(Path(device_id): Path<String>) -> Json<Value> {
    match RPCStorage::get_device_pin(&device_id).await {
        Ok(_) => Json(json!({
            "success": true,
            "device_id": device_id,
            "pin_exists": true
        })),
        Err(StorageError::NotFound(_)) => Json(json!({
            "success": true,
            "device_id": device_id,
            "pin_exists": false
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }))
    }
}

#[derive(Deserialize)]
struct CreatePinRequest {
    #[allow(dead_code)]
    pin_hash: String,
}

#[allow(dead_code)]
async fn device_create_pin(Path(device_id): Path<String>, Json(payload): Json<CreatePinRequest>) -> Json<Value> {
    let timestamp = Utc::now().timestamp() as u64;
    
    match RPCStorage::set_device_pin(&device_id, &payload.pin_hash).await {
        Ok(_) => {
            if let Err(e) = RPCStorage::set_device_failed_attempts(&device_id, 0).await {
                log::error!("Failed to reset failed attempts for device {}: {}", device_id, e);
            }
            
            if let Err(e) = RPCStorage::set_device_lockout(&device_id, 0).await {
                log::error!("Failed to reset lockout for device {}: {}", device_id, e);
            }
            
            Json(json!({
                "success": true,
                "device_id": device_id,
                "message": "PIN created successfully",
                "timestamp": timestamp
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to create PIN: {}", e)
            }))
        }
    }
}

#[derive(Deserialize)]
struct SetupPinRequest {
    #[allow(dead_code)]
    pin_hash: String,
    #[allow(dead_code)]
    timestamp: u64,
}

#[allow(dead_code)]
async fn device_setup_pin(Path(device_id): Path<String>, Json(payload): Json<SetupPinRequest>) -> Json<Value> {
    match RPCStorage::set_device_pin(&device_id, &payload.pin_hash).await {
        Ok(_) => {
            Json(json!({
                "success": true,
                "device_id": device_id,
                "message": "PIN setup completed"
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to setup PIN: {}", e)
            }))
        }
    }
}

#[derive(Deserialize)]
struct VerifyPinRequest {
    pin_hash: String,
}

async fn device_verify_pin(Path(device_id): Path<String>, payload: Result<Json<VerifyPinRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(payload)) => device_verify_pin_impl(device_id, payload).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

async fn device_verify_pin_impl(device_id: String, payload: VerifyPinRequest) -> Json<Value> {
    // Check if device is locked out
    match RPCStorage::get_device_lockout(&device_id).await {
        Ok(lockout_until) => {
            let current_time = Utc::now().timestamp() as u64;
            if lockout_until > current_time {
                let remaining_seconds = lockout_until - current_time;
                return Json(json!({
                    "success": false,
                    "error": "Device is locked out",
                    "lockout_remaining_seconds": remaining_seconds
                }));
            }
        },
        Err(e) => {
            log::error!("Failed to check lockout status for device {}: {}", device_id, e);
        }
    }
    
    // Get stored PIN data
    match RPCStorage::get_device_pin(&device_id).await {
        Ok(stored_hash) => {
            // Frontend already sends hashed PIN, so we compare directly
            if payload.pin_hash == stored_hash {
                // PIN is correct, reset failed attempts
                if let Err(e) = RPCStorage::set_device_failed_attempts(&device_id, 0).await {
                    log::error!("Failed to reset failed attempts for device {}: {}", device_id, e);
                }
                
                Json(json!({
                    "success": true,
                    "device_id": device_id,
                    "message": "PIN verified successfully"
                }))
            } else {
                // PIN is incorrect, increment failed attempts
                let failed_attempts = RPCStorage::get_device_failed_attempts(&device_id).await.unwrap_or(0);
                let new_attempts = failed_attempts + 1;
                
                if let Err(e) = RPCStorage::set_device_failed_attempts(&device_id, new_attempts).await {
                    log::error!("Failed to update failed attempts for device {}: {}", device_id, e);
                }
                
                // Lock device if too many failed attempts
                if new_attempts >= 5 {
                    let lockout_until = Utc::now().timestamp() as u64 + 300; // 5 minutes
                    if let Err(e) = RPCStorage::set_device_lockout(&device_id, lockout_until).await {
                        log::error!("Failed to set lockout for device {}: {}", device_id, e);
                    }
                    
                    Json(json!({
                        "success": false,
                        "error": "Too many failed attempts. Device locked for 5 minutes.",
                        "failed_attempts": new_attempts,
                        "lockout_until": lockout_until
                    }))
                } else {
                    Json(json!({
                        "success": false,
                        "error": "Invalid PIN",
                        "failed_attempts": new_attempts,
                        "remaining_attempts": 5 - new_attempts
                    }))
                }
            }
        },
        Err(StorageError::NotFound(_)) => {
            Json(json!({
                "success": false,
                "error": "No PIN set for this device"
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

#[allow(dead_code)]
async fn device_clear_lockout(Path(device_id): Path<String>) -> Json<Value> {
    // Reset failed attempts
    if let Err(e) = RPCStorage::set_device_failed_attempts(&device_id, 0).await {
        log::error!("Failed to reset failed attempts for device {}: {}", device_id, e);
    }
    
    // Clear lockout
    let _ = RPCStorage::set_device_lockout(&device_id, 0).await;
    
    Json(json!({
        "success": true,
        "device_id": device_id,
        "message": "Device lockout cleared successfully"
    }))
}

#[derive(Deserialize)]
struct DeviceLoginRequest {
    pin_hash: String,
}

#[allow(dead_code)]
async fn device_login(Path(device_id): Path<String>, Json(payload): Json<DeviceLoginRequest>) -> Json<Value> {
    // Verify PIN first
    let pin_verification = device_verify_pin_impl(device_id.clone(), VerifyPinRequest { pin_hash: payload.pin_hash }).await;
    
    if pin_verification["success"].as_bool().unwrap_or(false) {
        // PIN verified, create session
        let current_time = Utc::now().timestamp() as u64;
        let session_id = format!("session_{}_{}", device_id, current_time);
        
        match RPCStorage::set_device_session(&device_id, &session_id, current_time).await {
            Ok(_) => {
                Json(json!({
                    "success": true,
                    "device_id": device_id,
                    "session_id": session_id,
                    "message": "Login successful"
                }))
            },
            Err(e) => {
                Json(json!({
                    "success": false,
                    "error": format!("Failed to create session: {}", e)
                }))
            }
        }
    } else {
        pin_verification
    }
}

#[allow(dead_code)]
async fn device_reset_pin(Path(device_id): Path<String>) -> Json<Value> {
    match RPCStorage::remove_device_pin(&device_id).await {
        Ok(_) => {
            // Also reset failed attempts and lockout
            let _ = RPCStorage::set_device_failed_attempts(&device_id, 0).await;
            let _ = RPCStorage::set_device_lockout(&device_id, 0).await;
            
            Json(json!({
                "success": true,
                "device_id": device_id,
                "message": "PIN reset successfully"
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to reset PIN: {}", e)
            }))
        }
    }
}

// Additional endpoints from rpc-server
#[derive(Deserialize)]
#[allow(dead_code)]
struct DeviceStatusRequest {
    device_id: String,
}

// Removed unused miner_status functions and structs to fix dead_code warnings

// Wallet endpoints
#[derive(Deserialize)]
struct DeviceBalanceRequest {
    #[allow(dead_code)]
    device_id: String,
    #[allow(dead_code)]
    address: String,
}

#[allow(dead_code)]
async fn device_balance(payload: Result<Json<DeviceBalanceRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(req)) => device_balance_impl(req).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

#[allow(dead_code)]
async fn device_balance_impl(payload: DeviceBalanceRequest) -> Json<Value> {
    match RPCStorage::get_balance(&payload.address).await {
        Ok(balance) => {
            Json(json!({
                "success": true,
                "device_id": payload.device_id,
                "address": payload.address,
                "balance": balance
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to get balance: {}", e)
            }))
        }
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SendRequest {
    from: String,
    to: String,
    amount: u64,
    private_key: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DeviceSendRequest {
    from: String,
    to: String,
    amount: u64,
    private_key: String,
    device_id: String,
}

#[derive(Deserialize)]
struct AddressQuery {
    address: String,
}

#[derive(Deserialize)]
struct WalletBalanceRequest {
    address: String,
    device_id: String,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct TransactionRequest {
    from: String,
    to: String,
    amount: u64,
}

#[allow(dead_code)]
async fn post_transaction(State(state): State<AppState>, Json(_tx_req): Json<TransactionRequest>) -> Json<Value> {
    let mut total_transactions = state.total_transactions.write().await;
    *total_transactions += 1;
    
    Json(json!({
        "success": true,
        "message": "Transaction posted",
        "total_transactions": *total_transactions
    }))
}

#[allow(dead_code)]
async fn wallet_create(Query(query): Query<HashMap<String, String>>) -> Json<Value> {
    let device_id = query.get("device_id").cloned().unwrap_or_else(|| "unknown".to_string());
    
    let key_manager = KeyManager::new();
    let private_key = hex::encode(key_manager.get_private_key());
    let public_key = hex::encode(key_manager.get_public_key());
    let address = key_manager.get_address();
    
    Json(json!({
        "success": true,
        "device_id": device_id,
        "address": address,
        "private_key": private_key,
        "public_key": public_key
    }))
}

#[derive(Deserialize)]
struct WalletCreateRequest {
    #[allow(dead_code)]
    device_id: String,
    #[allow(dead_code)]
    pin_hash: String,
}

#[derive(Deserialize)]
struct WalletTransactionsRequest {
    address: String,
    limit: Option<usize>,
    transaction_type: Option<String>,
}

#[allow(dead_code)]
async fn wallet_create_post(Json(payload): Json<WalletCreateRequest>) -> Json<Value> {
    let key_manager = KeyManager::new();
    let private_key = hex::encode(key_manager.get_private_key());
    let public_key = hex::encode(key_manager.get_public_key());
    let address = key_manager.get_address();
    
    // Save wallet data
    let wallet_data = json!({
        "address": address,
        "private_key": private_key,
        "public_key": public_key,
        "imported": false,
        "timestamp": Utc::now().timestamp()
    });
    
    match RPCStorage::save_device_wallet(&payload.device_id, &wallet_data.to_string()).await {
        Ok(_) => {
            // CRITICAL FIX: Save device address mapping for mining rewards
            if let Err(e) = RPCStorage::set_device_address(&payload.device_id, &address).await {
                println!("Warning: Failed to save device address mapping: {}", e);
            }
            
            // Set initial balance for new wallet (10 FVC = 10,000,000 microFVC)
            if let Err(e) = RPCStorage::set_balance(&address, 10000000).await { // 10 FVC initial balance
                println!("Warning: Failed to set initial balance: {}", e);
            }
            
            Json(json!({
                "success": true,
                "device_id": payload.device_id,
                "address": address,
                "private_key": private_key,
                "public_key": public_key,
                "balance": 10000000u64
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to save wallet data: {}", e)
            }))
        }
    }
}

#[allow(dead_code)]
async fn wallet_balance(payload: Result<Json<AddressQuery>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(query)) => wallet_balance_impl(query).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

#[allow(dead_code)]
async fn wallet_balance_impl(query: AddressQuery) -> Json<Value> {
    match RPCStorage::get_balance(&query.address).await {
        Ok(balance) => Json(json!({
            "success": true,
            "address": query.address,
            "balance": balance
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to get balance: {}", e)
        }))
    }
}

async fn device_send(State(state): State<AppState>, payload: Result<Json<DeviceSendRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(req)) => device_send_impl(State(state), req).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

async fn device_send_impl(State(_state): State<AppState>, payload: DeviceSendRequest) -> Json<Value> {
    // Validate addresses
    if payload.from.len() < 20 || payload.to.len() < 20 {
        return Json(json!({
            "success": false,
            "error": "Invalid address format"
        }));
    }
    
    // Check sender balance including transaction fee
    let fee = 1000; // 0.001 FVC fee
    let total_required = payload.amount + fee;
    
    match RPCStorage::get_balance(&payload.from).await {
        Ok(balance) => {
            if balance < total_required {
                return Json(json!({
                    "success": false,
                    "error": format!("Insufficient balance. Required: {} (including {} fee), Available: {}", total_required, fee, balance)
                }));
            }
        },
        Err(e) => {
            return Json(json!({
                "success": false,
                "error": format!("Failed to check balance: {}", e)
            }));
        }
    }
    
    // Create transaction
    let tx = WalletTransaction {
        hash: format!("tx_{}", Utc::now().timestamp()),
        from: payload.from.clone(),
        to: payload.to.clone(),
        amount: payload.amount,
        timestamp: Utc::now().timestamp() as u64,
        transaction_type: "device_transfer".to_string(),
        block_height: 1,
    };
    
    // Process transaction
    match RPCStorage::add_transaction(&tx).await {
        Ok(_) => {
            // Get current balances safely
            let sender_current_balance = RPCStorage::get_balance(&payload.from).await.unwrap_or(0);
            let receiver_current_balance = RPCStorage::get_balance(&payload.to).await.unwrap_or(0);
            
            // Double-check balance before updating (safety check)
            if sender_current_balance < total_required {
                return Json(json!({
                    "success": false,
                    "error": "Insufficient balance including transaction fee"
                }));
            }
            
            // Update balances safely
            let sender_new_balance = sender_current_balance - total_required;
            let receiver_new_balance = receiver_current_balance + payload.amount;
            
            match RPCStorage::set_balance(&payload.from, sender_new_balance).await {
                Ok(_) => {},
                Err(e) => {
                    log::error!("Failed to update sender balance: {}", e);
                    return Json(json!({
                        "success": false,
                        "error": "Failed to update sender balance"
                    }));
                }
            }
            
            match RPCStorage::set_balance(&payload.to, receiver_new_balance).await {
                Ok(_) => {},
                Err(e) => {
                    log::error!("Failed to update receiver balance: {}", e);
                    return Json(json!({
                        "success": false,
                        "error": "Failed to update receiver balance"
                    }));
                }
            }
            
            // Broadcast transaction
            let _ = BROADCAST.send(json!({
                "type": "new_transaction",
                "transaction": tx
            }).to_string());
            
            Json(json!({
                "success": true,
                "device_id": payload.device_id,
                "transaction_hash": tx.hash,
                "from": payload.from,
                "to": payload.to,
                "amount": payload.amount
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Transaction failed: {}", e)
            }))
        }
    }
}

async fn wallet_send(State(state): State<AppState>, payload: Result<Json<SendRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(req)) => wallet_send_impl(State(state), req).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

async fn wallet_send_impl(State(_state): State<AppState>, payload: SendRequest) -> Json<Value> {
    // Validate addresses
    if payload.from.len() < 20 || payload.to.len() < 20 {
        return Json(json!({
            "success": false,
            "error": "Invalid address format"
        }));
    }
    
    // Check sender balance
    match RPCStorage::get_balance(&payload.from).await {
        Ok(balance) => {
            if balance < payload.amount {
                return Json(json!({
                    "success": false,
                    "error": "Insufficient balance"
                }));
            }
        },
        Err(e) => {
            return Json(json!({
                "success": false,
                "error": format!("Failed to check balance: {}", e)
            }));
        }
    }
    
    // Create transaction
    let tx = WalletTransaction {
        hash: format!("tx_{}", Utc::now().timestamp()),
        from: payload.from.clone(),
        to: payload.to.clone(),
        amount: payload.amount,
        timestamp: Utc::now().timestamp() as u64,
        transaction_type: "transfer".to_string(),
        block_height: 1,
    };
    
    // Process transaction
    match RPCStorage::add_transaction(&tx).await {
        Ok(_) => {
            // Update balances
            let sender_new_balance = RPCStorage::get_balance(&payload.from).await.unwrap_or(0) - payload.amount - 1000; // 0.001 FVC fee
            let receiver_new_balance = RPCStorage::get_balance(&payload.to).await.unwrap_or(0) + payload.amount;
            
            if let Err(e) = RPCStorage::set_balance(&payload.from, sender_new_balance).await {
                log::error!("Failed to update sender balance: {}", e);
            }
            
            if let Err(e) = RPCStorage::set_balance(&payload.to, receiver_new_balance).await {
                log::error!("Failed to update receiver balance: {}", e);
            }
            
            // Broadcast transaction
            let _ = BROADCAST.send(json!({
                "type": "new_transaction",
                "transaction": tx
            }).to_string());
            
            Json(json!({
                "success": true,
                "transaction_hash": tx.hash,
                "from": payload.from,
                "to": payload.to,
                "amount": payload.amount
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Transaction failed: {}", e)
            }))
        }
    }
}

#[allow(dead_code)]
async fn get_address(Path(address): Path<String>) -> Json<Value> {
    match RPCStorage::get_balance(&address).await {
        Ok(balance) => Json(json!({ "success": true, "address": address, "balance": balance })),
        Err(e) => Json(json!({ "success": false, "error": format!("Failed to get address info: {}", e) }))
    }
}

// Admin endpoints
#[derive(Deserialize)]
struct AdminSetBalanceRequest {
    #[allow(dead_code)]
    address: String,
    #[allow(dead_code)]
    balance: u64,
    #[allow(dead_code)]
    admin_key: String,
}

#[allow(dead_code)]
async fn admin_set_balance(payload: Result<Json<AdminSetBalanceRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(req)) => admin_set_balance_impl(req).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

#[allow(dead_code)]
async fn admin_set_balance_impl(payload: AdminSetBalanceRequest) -> Json<Value> {
    // Simple admin key check (in production, use proper authentication)
    if payload.admin_key != "admin123" {
        return Json(json!({
            "success": false,
            "error": "Invalid admin key"
        }));
    }
    
    match RPCStorage::set_balance(&payload.address, payload.balance).await {
        Ok(_) => Json(json!({
            "success": true,
            "message": "Balance updated",
            "address": payload.address,
            "new_balance": payload.balance
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to set balance: {}", e)
        }))
    }
}

#[derive(Deserialize)]
struct AdminInitializeEcosystemRequest {
    admin_key: String,
}

#[allow(dead_code)]
async fn admin_initialize_ecosystem(payload: Result<Json<AdminInitializeEcosystemRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(req)) => admin_initialize_ecosystem_impl(req).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

#[allow(dead_code)]
async fn admin_initialize_ecosystem_impl(payload: AdminInitializeEcosystemRequest) -> Json<Value> {
    // Simple admin key check
    if payload.admin_key != "admin123" {
        return Json(json!({
            "success": false,
            "error": "Invalid admin key"
        }));
    }
    
    // Initialize ecosystem wallets with initial balances
    let ecosystem_wallets = vec![
        ("FVCowner1234567890abcdef", 1000000000000u64), // 1M FVC
        ("FVCdeveloper1234567890ab", 500000000000u64),   // 500K FVC
        ("FVCmaintenance123456789", 250000000000u64),    // 250K FVC
        ("FVCfeepool1234567890abc", 100000000000u64),    // 100K FVC
    ];
    
    for (address, balance) in ecosystem_wallets {
        if let Err(e) = RPCStorage::set_balance(address, balance).await {
            log::error!("Failed to initialize wallet {}: {}", address, e);
        }
    }
    
    Json(json!({
        "success": true,
        "message": "Ecosystem initialized successfully"
    }))
}

#[allow(dead_code)]
async fn get_network_health(State(state): State<AppState>) -> Json<Value> {
    let latest_block = *state.latest_block.read().await;
    let total_transactions = *state.total_transactions.read().await;
    let active_nodes = *state.active_nodes.read().await;
    
    let health_score = if latest_block > 0 && active_nodes > 0 {
        let block_rate = latest_block as f64 / (Utc::now().timestamp() as f64 / 3600.0); // blocks per hour
        let tx_rate = total_transactions as f64 / latest_block as f64; // tx per block
        let node_factor = (active_nodes as f64 / 10.0).min(1.0); // normalize to 0-1
        
        ((block_rate + tx_rate + node_factor) / 3.0 * 100.0).min(100.0)
    } else {
        0.0
    };
    
    Json(json!({
        "success": true,
        "network_health": {
            "health_score": health_score,
            "latest_block": latest_block,
            "total_transactions": total_transactions,
            "active_nodes": active_nodes,
            "status": if health_score > 80.0 { "excellent" } else if health_score > 60.0 { "good" } else if health_score > 40.0 { "fair" } else { "poor" }
        }
    }))
}

// Mining heartbeat and detection
async fn mining_heartbeat(payload: Result<Json<HeartbeatRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(req)) => mining_heartbeat_impl(req).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

async fn mining_heartbeat_impl(payload: HeartbeatRequest) -> Json<Value> {
    let response = AUTO_DETECTION.update_heartbeat(payload).await;
    
    Json(json!({
        "success": response.success,
        "device_id": response.server_time,
        "mining_status": response.mining_status,
        "message": response.message,
        "timestamp": Utc::now().timestamp()
    }))
}

async fn mining_detection_stats() -> Json<Value> {
    let stats = AUTO_DETECTION.get_statistics().await;
    
    Json(json!({
        "success": true,
        "detection_stats": stats
    }))
}

// Reward estimation endpoint based on Smart Rate
async fn reward_estimation(Query(params): Query<HashMap<String, String>>) -> Json<Value> {
    let device_id = match params.get("device_id") {
        Some(id) => id,
        None => {
            return Json(json!({
                "success": false,
                "error": "device_id parameter is required"
            }));
        }
    };
    
    // Calculate Smart Rate components
    let smart_rate = calculate_smart_rate().await;
    let vortex_energy = calculate_vortex_energy_rate().await;
    let fractal_score = calculate_fractal_contribution_score().await;
    let efficiency_index = calculate_mathematical_efficiency_index().await;
    let harmony_factor = calculate_network_harmony_factor().await;
    
    // Calculate reward based on Smart Rate contribution
    // Base reward calculation: 1 SS/S should yield approximately 600-800 FVC per day
    // For 50-150 SS/S range, this gives 30,000-120,000 FVC per day
    let base_reward_per_ss = 600.0; // Base FVC per SS/S per day
    let smart_rate_multiplier = if smart_rate > 100.0 {
        1.2 // Bonus for high Smart Rate
    } else if smart_rate > 50.0 {
        1.0 // Standard rate
    } else {
        0.8 // Reduced rate for low Smart Rate
    };
    
    let estimated_daily_reward = smart_rate * base_reward_per_ss * smart_rate_multiplier;
    
    // Apply network difficulty adjustment
    let network_difficulty_factor = 1.0; // Can be adjusted based on network conditions
    let final_estimated_reward = estimated_daily_reward * network_difficulty_factor;
    
    Json(json!({
        "success": true,
        "data": {
            "device_id": device_id,
            "smart_rate": smart_rate,
            "smart_rate_unit": "ss/s",
            "estimated_daily_reward": final_estimated_reward,
            "base_reward_per_ss": base_reward_per_ss,
            "smart_rate_multiplier": smart_rate_multiplier,
            "network_difficulty_factor": network_difficulty_factor,
            "components": {
                "vortex_energy_rate": vortex_energy,
                "fractal_contribution_score": fractal_score,
                "mathematical_efficiency_index": efficiency_index,
                "network_harmony_factor": harmony_factor
            }
        }
    }))
}

// Additional device endpoints
async fn miner_register(State(_state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let device_id = payload["device_id"].as_str().unwrap_or("unknown");
    let session_token = payload["session_token"].as_str().unwrap_or("");
    let wallet_address = payload["wallet_address"].as_str().unwrap_or("");
    
    if device_id == "unknown" || session_token.is_empty() {
        return Json(json!({
            "success": false,
            "error": "device_id and session_token are required"
        }));
    }
    
    // Validate wallet address format (FVChain native format: fvc + 36 hex + emyl)
    if !wallet_address.is_empty() {
        if wallet_address.len() != 43 {
            return Json(json!({
                "success": false,
                "error": "Alamat wallet harus 43 karakter (fvc + 36 hex + emyl)"
            }));
        }
        
        if !wallet_address.starts_with("fvc") || !wallet_address.ends_with("emyl") {
            return Json(json!({
                "success": false,
                "error": "Format alamat wallet tidak valid. Gunakan format FVChain native (fvcxxxxxxxxemyl)"
            }));
        }
        
        // Validate hex characters in the middle (36 characters between fvc and emyl)
        let hex_part = &wallet_address[3..39];
        if hex_part.len() != 36 || !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Json(json!({
                "success": false,
                "error": "Alamat wallet mengandung karakter tidak valid. Harus berupa hex (0-9, a-f)"
            }));
        }
    }
    
    // Save device session to track mining status
    let current_time = Utc::now().timestamp() as u64;
    if let Err(e) = RPCStorage::set_device_session(device_id, session_token, current_time).await {
        log::error!("Failed to save device session during registration: {}", e);
        return Json(json!({
            "success": false,
            "error": format!("Failed to register device session: {}", e)
        }));
    }
    
    // Also save wallet address for this device
    if !wallet_address.is_empty() {
        if let Err(e) = RPCStorage::set_device_address(device_id, wallet_address).await {
            log::error!("Failed to save device address: {}", e);
            return Json(json!({
                "success": false,
                "error": format!("Failed to save device address: {}", e)
            }));
        }
    }
    
    // Register device to AUTO_DETECTION for heartbeat monitoring
    AUTO_DETECTION.register_device(
        device_id.to_string(),
        session_token.to_string(),
        wallet_address.to_string()
    ).await;
    
    log::info!("Device registered successfully: {} with session: {}", device_id, session_token);
    
    Json(json!({
        "success": true,
        "device_id": device_id,
        "session_token": session_token,
        "wallet_address": wallet_address,
        "registered": true,
        "timestamp": current_time
    }))
}

#[allow(dead_code)]
async fn miner_update(State(_state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let device_id = payload["device_id"].as_str().unwrap_or("unknown");
    
    Json(json!({
        "success": true,
        "device_id": device_id,
        "updated": true
    }))
}

// Device unregistration endpoint
async fn miner_unregister(State(_state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let device_id = payload["device_id"].as_str().unwrap_or("unknown");
    
    if device_id == "unknown" {
        return Json(json!({
            "success": false,
            "error": "device_id is required"
        }));
    }
    
    // Stop mining if currently active
    if let Err(e) = stop_production_mining(device_id).await {
        log::warn!("Failed to stop mining during unregistration for device {}: {}", device_id, e);
    }
    
    // Remove device from AUTO_DETECTION
    AUTO_DETECTION.unregister_device(device_id).await;
    
    // Remove device session
    if let Err(e) = RPCStorage::remove_device_session(device_id).await {
        log::error!("Failed to remove device session during unregistration: {}", e);
    }
    
    // Remove device address mapping
    if let Err(e) = RPCStorage::remove_device_address(device_id).await {
        log::error!("Failed to remove device address during unregistration: {}", e);
    }
    
    // Remove device PIN if exists
    if let Err(e) = RPCStorage::remove_device_pin(device_id).await {
        log::warn!("Failed to remove device PIN during unregistration: {}", e);
    }
    
    // Reset failed attempts and lockout
    let _ = RPCStorage::set_device_failed_attempts(device_id, 0).await;
    let _ = RPCStorage::set_device_lockout(device_id, 0).await;
    
    // Remove device private key if exists
    let _ = RPCStorage::remove_device_private_key(device_id).await;
    
    // Remove device registration data
    if let Err(e) = RPCStorage::remove_device_registration(device_id).await {
        log::warn!("Failed to remove device registration data: {}", e);
    }
    
    log::info!("Device unregistered successfully: {}", device_id);
    
    Json(json!({
        "success": true,
        "device_id": device_id,
        "unregistered": true,
        "message": "Device has been successfully unregistered and all associated data has been cleared",
        "timestamp": Utc::now().timestamp()
    }))
}

// Wallet import endpoint




// Get block by height
async fn get_block_by_height(Path(height): Path<u64>) -> Json<Value> {
    match RPCStorage::get_block_by_height(height).await {
        Ok(Some(block)) => Json(json!({
            "success": true,
            "block": block
        })),
        Ok(None) => Json(json!({
            "success": false,
            "error": "Block not found"
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to get block: {}", e)
        }))
    }
}

// Get transaction by hash
async fn get_transaction_by_hash(Path(hash): Path<String>) -> Json<Value> {
    match RPCStorage::get_transaction(&hash).await {
        Ok(Some(transaction)) => Json(json!({
            "success": true,
            "transaction": transaction
        })),
        Ok(None) => Json(json!({
            "success": false,
            "error": "Transaction not found"
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to get transaction: {}", e)
        }))
    }
}

// Get balance for address
async fn get_balance(Path(address): Path<String>) -> Json<Value> {
    match RPCStorage::get_balance(&address).await {
        Ok(balance) => Json(json!({
            "success": true,
            "balance": balance
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to get balance: {}", e)
        }))
    }
}

/// Check if wallet address exists on the server
async fn wallet_check_address(Path(address): Path<String>) -> Json<Value> {
    // Check if address exists in storage by trying to get balance
    match RPCStorage::get_balance(&address).await {
        Ok(_balance) => {
            // Address exists, try to get associated device_id
            match RPCStorage::get_device_id_by_address(&address).await {
                Ok(Some(device_id)) => Json(json!({
                    "success": true,
                    "exists": true,
                    "address": address,
                    "device_id": device_id,
                    "status": "found"
                })),
                Ok(None) => {
                    // Address exists but no device_id found
                    Json(json!({
                        "success": true,
                        "exists": true,
                        "address": address,
                        "device_id": null,
                        "status": "found_no_device"
                    }))
                },
                Err(e) => {
                    // Error getting device_id
                    Json(json!({
                        "success": false,
                        "error": format!("Failed to get device_id: {}", e),
                        "address": address
                    }))
                }
            }
        },
        Err(_) => {
            // Address doesn't exist
            Json(json!({
                "success": true,
                "exists": false,
                "address": address,
                "device_id": null,
                "status": "not_found"
            }))
        }
    }
}

// Get wallet transactions
async fn wallet_transactions(payload: Result<Json<WalletTransactionsRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(req)) => wallet_transactions_impl(req).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

async fn wallet_transactions_impl(request: WalletTransactionsRequest) -> Json<Value> {
    // Validate address format
    if request.address.is_empty() {
        return Json(json!({
            "success": false,
            "message": "Address is required",
            "transactions": [],
            "total_count": 0
        }));
    }

    let limit = request.limit.unwrap_or(5).min(50); // Max 50 transactions
    let transaction_type = request.transaction_type.unwrap_or_default();

    // Get transactions from storage
    match RPCStorage::get_latest_transactions(limit * 2).await { // Get more to filter
        Ok(all_transactions) => {
            // Filter transactions for the specific address
            let mut filtered_transactions: Vec<serde_json::Value> = all_transactions
                .into_iter()
                .filter(|tx| {
                    // Check if transaction involves this address
                    let involves_address = tx.to == request.address || tx.from == request.address;
                    
                    // Filter by transaction type if specified
                    if !transaction_type.is_empty() {
                        involves_address && tx.transaction_type == transaction_type
                    } else {
                        involves_address
                    }
                })
                .take(limit)
                .map(|tx| {
                    // Convert to mobile-friendly format
                    let amount = if tx.to == request.address {
                        tx.amount as f64 / 1_000_000.0 // Incoming transaction (positive)
                    } else {
                        -(tx.amount as f64 / 1_000_000.0) // Outgoing transaction (negative)
                    };
                    
                    let status = if tx.transaction_type == "mining_reward" {
                        "confirmed"
                    } else {
                        "confirmed" // All stored transactions are confirmed
                    };
                    
                    json!({
                        "hash": tx.hash,
                        "amount": amount,
                        "timestamp": chrono::DateTime::from_timestamp(tx.timestamp as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .to_rfc3339(),
                        "status": status,
                        "type": tx.transaction_type,
                        "from": tx.from,
                        "to": tx.to,
                        "block_height": tx.block_height
                    })
                })
                .collect();

            // Sort by timestamp (newest first)
            filtered_transactions.sort_by(|a, b| {
                let timestamp_a = a["timestamp"].as_str().unwrap_or("");
                let timestamp_b = b["timestamp"].as_str().unwrap_or("");
                timestamp_b.cmp(timestamp_a)
            });

            Json(json!({
                "success": true,
                "message": "Transactions retrieved successfully",
                "transactions": filtered_transactions,
                "total_count": filtered_transactions.len()
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "message": format!("Failed to get transactions: {}", e),
                "transactions": [],
                "total_count": 0
            }))
        }
    }
}

// Get wallet balance with device validation
async fn wallet_get_balance(payload: Result<Json<WalletBalanceRequest>, JsonRejection>) -> impl IntoResponse {
    match payload {
        Ok(Json(request)) => wallet_get_balance_impl(request).await.into_response(),
        Err(rejection) => {
            let (status, json_response) = handle_json_rejection(rejection);
            (status, json_response).into_response()
        }
    }
}

async fn wallet_get_balance_impl(request: WalletBalanceRequest) -> Json<Value> {
    // Validate address format
    if request.address.is_empty() || request.device_id.is_empty() {
        return Json(json!({
            "success": false,
            "error": "Address and device_id are required"
        }));
    }
    
    // Get balance from storage
    match RPCStorage::get_balance(&request.address).await {
        Ok(balance) => Json(json!({
            "success": true,
            "address": request.address,
            "balance": balance,
            "device_id": request.device_id,
            "timestamp": chrono::Utc::now().timestamp()
        })),
        Err(e) => {
            // Check if it's a wallet not found error
            if e.to_string().contains("not found") {
                Json(json!({
                    "success": false,
                    "error": "wallet_not_found",
                    "message": "Wallet address not found in storage"
                }))
            } else {
                Json(json!({
                    "success": false,
                    "error": format!("Failed to get balance: {}", e)
                }))
            }
        }
    }
}

// Get network info
async fn get_network_info() -> Json<Value> {
    match RPCStorage::get_network_info().await {
        Ok(info) => Json(json!({
            "success": true,
            "data": info
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to get network info: {}", e)
        }))
    }
}

// Get stats
async fn get_stats() -> Json<Value> {
    match RPCStorage::get_stats().await {
        Ok(stats) => Json(json!({
            "success": true,
            "data": stats
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to get stats: {}", e)
        }))
    }
}

// Get miner status for specific device
async fn get_miner_status(Query(params): Query<std::collections::HashMap<String, String>>) -> Json<Value> {
    let device_id = params.get("device_id").cloned().unwrap_or_default();
    
    if device_id.is_empty() {
        return Json(json!({
            "success": false,
            "error": "device_id parameter is required"
        }));
    }
    
    // Check global miner status first
    let global_miner = GLOBAL_ECOSYSTEM_MINER.lock().await;
    let is_mining_active = if let Some(ref miner) = *global_miner {
        miner.is_running()
    } else {
        false
    };
    drop(global_miner); // Release the lock early
    
    // Calculate Smart Rate and reward estimation
    let smart_rate = calculate_smart_rate().await;
    let vortex_energy = calculate_vortex_energy_rate().await;
    let fractal_score = calculate_fractal_contribution_score().await;
    let efficiency_index = calculate_mathematical_efficiency_index().await;
    let harmony_factor = calculate_network_harmony_factor().await;
    
    // Calculate estimated daily reward for this device
    let block_reward = 6.25; // FVC per block
    let blocks_per_day = 17280.0; // 86400 seconds / 5 seconds per block
    let active_devices = RPCStorage::get_all_active_devices().await.unwrap_or_default().len() as f64;
    let device_share = if active_devices > 0.0 { 1.0 / active_devices } else { 1.0 };
    let estimated_daily_reward = block_reward * blocks_per_day * device_share * 0.9; // 90% goes to miners
    
    // Get device session to check mining status
    match RPCStorage::get_device_session(&device_id).await {
        Ok(Some((session_token, started_at))) => {
            // Validate session is not inactive (5 minutes = 300 seconds) or expired (24 hours = 86400 seconds)
            let current_time = chrono::Utc::now().timestamp() as u64;
            let inactive_time = current_time.saturating_sub(started_at);
            
            if inactive_time > 300 || inactive_time > 86400 {
                // Session expired, remove it and return stopped status
                if let Err(e) = RPCStorage::remove_device_session(&device_id).await {
                    log::error!("Failed to remove expired session for device {}: {}", device_id, e);
                }
                
                return Json(json!({
                    "success": true,
                    "mining_active": false,
                    "mining_type": "stopped",
                    "blockchain_connected": false,
                    "reward": 0,
                    "blocks_found": 0,
                    "device_id": device_id,
                    "uptime": 0,
                    "started_at": current_time,
                    "session_token": null,
                    "smart_rate": 0.0,
                    "smart_rate_unit": "ss/s",
                    "estimated_daily_reward": 0.0,
                    "vortex_energy_rate": 0.0,
                    "fractal_contribution_score": 0.0,
                    "mathematical_efficiency_index": 0.0,
                    "network_harmony_factor": 0.0,
                    "active_miners": active_devices as u32,
                    "message": if inactive_time > 86400 { "Session expired" } else { "Device inactive" }
                }));
            }
            
            // Get device wallet address and balance for actual earnings
            let actual_earnings = match RPCStorage::get_device_address(&device_id).await {
                Ok(Some(address)) => {
                    match RPCStorage::get_balance(&address).await {
                        Ok(balance) => balance as f64 / 1_000_000.0, // Convert from satoshis to FVC
                        Err(_) => 0.0
                    }
                },
                _ => 0.0
            };
            
            // Calculate blocks found based on earnings (6.25 FVC per block)
            let blocks_found = if actual_earnings > 0.0 {
                (actual_earnings / 6.25).floor() as u32
            } else {
                0
            };
            
            // Device has active and valid mining session
            Json(json!({
                "success": true,
                "mining_active": is_mining_active,
                "mining_type": if is_mining_active { "blockchain" } else { "stopped" },
                "blockchain_connected": is_mining_active,
                "reward": actual_earnings,
                "blocks_found": blocks_found,
                "device_id": device_id,
                "uptime": chrono::Utc::now().timestamp() as u64 - started_at,
                "started_at": started_at,
                "session_token": session_token,
                "smart_rate": if is_mining_active { smart_rate } else { 0.0 },
                "smart_rate_unit": "ss/s",
                "estimated_daily_reward": if is_mining_active { estimated_daily_reward } else { 0.0 },
                "vortex_energy_rate": if is_mining_active { vortex_energy } else { 0.0 },
                "fractal_contribution_score": if is_mining_active { fractal_score } else { 0.0 },
                "mathematical_efficiency_index": if is_mining_active { efficiency_index } else { 0.0 },
                "network_harmony_factor": if is_mining_active { harmony_factor } else { 0.0 },
                "active_miners": active_devices as u32
            }))
        },
        Ok(None) => {
            // No active mining session, but check global miner status
            Json(json!({
                "success": true,
                "mining_active": is_mining_active,
                "mining_type": if is_mining_active { "blockchain" } else { "stopped" },
                "blockchain_connected": is_mining_active,
                "reward": 0,
                "blocks_found": 0,
                "device_id": device_id,
                "uptime": 0,
                "started_at": chrono::Utc::now().timestamp(),
                "session_token": null,
                "smart_rate": if is_mining_active { smart_rate } else { 0.0 },
                "smart_rate_unit": "ss/s",
                "estimated_daily_reward": 0.0,
                "vortex_energy_rate": 0.0,
                "fractal_contribution_score": 0.0,
                "mathematical_efficiency_index": 0.0,
                "network_harmony_factor": 0.0,
                "active_miners": active_devices as u32
            }))
        },
        Err(e) => {
            log::error!("Failed to get device session for {}: {}", device_id, e);
            Json(json!({
                "success": false,
                "error": format!("Failed to get mining status: {}", e)
            }))
        }
    }
}

// Get mining status
async fn get_mining_status() -> Json<Value> {
    Json(json!({
        "success": true,
        "mining": false,
        "difficulty": 1000000
    }))
}

// Device session
async fn device_session(Path(device_id): Path<String>) -> Json<Value> {
    Json(json!({
        "success": true,
        "device_id": device_id,
        "session_active": true,
        "last_seen": chrono::Utc::now().timestamp()
    }))
}

// API Info endpoint
async fn api_info() -> Json<Value> {
    Json(json!({
        "name": "Fractal Vortex Chain RPC Server",
        "version": "1.0.0",
        "description": "Blockchain Layer 1 dengan vPoW (Virtual Proof of Work) dan Smart Rate",
        "status": "running",
        "endpoints": {
            "blockchain": [
                "GET /blocks",
                "GET /blocks/:height",
                "GET /transactions",
                "GET /transaction/:hash",
                "GET /balance/:address",
                "GET /network/info",
                "GET /stats"
            ],
            "mining": [
                "POST /mining/start",
                "POST /mining/stop",
                "GET /mining/status",
                "POST /mining/heartbeat",
                "GET /mining/detection/stats",
                "GET /miner/status",
                "POST /miner/register"
            ],
            "wallet": [
                "GET /wallet/create",
                "POST /wallet/send",
                "GET /wallet/balance/:address"
            ],
            "device": [
                "POST /device/verify",
                "POST /device/validate",
                "POST /device/wallet",
                "POST /device/register",
                "GET /device/session/:device_id",
                "POST /device/send"
            ],
            "admin": [
                "GET /admin/rate-limit/stats"
            ],
            "events": [
                "GET /events",
                "GET /events/blocks",
                "GET /events/transactions"
            ]
        },
        "features": [
            "vPoW (Virtual Proof of Work)",
            "Smart Rate Algorithm",
            "Fractal Hash Function",
            "Device Isolation",
            "Real-time SSE Events",
            "Rate Limiting",
            "CORS Support"
        ],
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

// Rate limiting stats
async fn rate_limit_stats() -> Json<Value> {
    Json(json!({
        "success": true,
        "message": "Rate limiting disabled",
        "global_rate_limiter": {},
        "endpoint_rate_limiters": {}
    }))
}

// Get monitoring statistics
async fn get_monitoring_stats() -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let response = serde_json::json!({
        "status": "success",
        "data": {
            "endpoint_stats": {},
            "total_endpoints": 0,
            "message": "API monitoring disabled",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }
    });
    
    Ok(Json(response))
}

// Get health status
async fn get_health_status() -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let response = serde_json::json!({
        "status": "success",
        "data": {
            "status": "healthy",
            "message": "API monitoring disabled"
        }
    });
    
    Ok(Json(response))
}

// Get recent security events
async fn get_security_events() -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let response = serde_json::json!({
        "status": "success",
        "data": {
            "security_events": [],
            "message": "API monitoring disabled",
            "count": 0,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }
    });
    
    Ok(Json(response))
}

// Multi-Node Monitoring API Functions

// Get status of all 4 nodes
async fn get_nodes_status() -> Json<Value> {
    let (healthy_count, total_count) = NODE_MANAGER.get_health_stats().await;
    let health = NODE_HEALTH.read().await;
    
    let mut nodes_status = Vec::new();
    for (index, is_healthy) in health.iter().enumerate() {
        nodes_status.push(json!({
            "node_id": index,
            "status": if *is_healthy { "healthy" } else { "unhealthy" },
            "port": 8000 + index as u16,
            "last_check": chrono::Utc::now().timestamp()
        }));
    }
    
    Json(json!({
        "success": true,
        "api_version": "1.0",
        "cluster_status": {
            "total_nodes": total_count,
            "healthy_nodes": healthy_count,
            "unhealthy_nodes": total_count - healthy_count,
            "cluster_health": if healthy_count > 0 { "operational" } else { "critical" }
        },
        "nodes": nodes_status,
        "load_balancer": {
            "algorithm": "round_robin",
            "current_counter": LOAD_BALANCER_COUNTER.load(Ordering::Relaxed)
        },
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

// Get detailed information about a specific node
async fn get_node_details(Path(node_id): Path<usize>) -> Json<Value> {
    if node_id >= 4 {
        return Json(json!({
            "success": false,
            "error": "Invalid node ID. Valid range: 0-3",
            "api_version": "1.0"
        }));
    }
    
    let health = NODE_HEALTH.read().await;
    let is_healthy = health.get(node_id).copied().unwrap_or(false);
    
    Json(json!({
        "success": true,
        "api_version": "1.0",
        "node": {
            "node_id": node_id,
            "status": if is_healthy { "healthy" } else { "unhealthy" },
            "port": 8000 + node_id as u16,
            "config": {
                "energy_threshold": match node_id {
                    0 => 1000,
                    1 => 1200,
                    2 => 1500,
                    3 => 1800,
                    _ => 1000
                },
                "fractal_levels": match node_id {
                    0 => 5,
                    1 => 6,
                    2 => 7,
                    3 => 8,
                    _ => 5
                },
                "max_peers": match node_id {
                    0 => 50,
                    1 => 75,
                    2 => 100,
                    3 => 125,
                    _ => 50
                }
            },
            "last_health_check": chrono::Utc::now().timestamp()
        }
    }))
}

// Restart a specific node
async fn restart_node(Path(node_id): Path<usize>) -> Json<Value> {
    if node_id >= 4 {
        return Json(json!({
            "success": false,
            "error": "Invalid node ID. Valid range: 0-3",
            "api_version": "1.0"
        }));
    }
    
    // Mark node as unhealthy during restart
    NODE_MANAGER.update_node_health(node_id, false).await;
    
    // Simulate restart process (in production, this would restart the actual node)
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Mark node as healthy after restart
    NODE_MANAGER.update_node_health(node_id, true).await;
    
    Json(json!({
        "success": true,
        "api_version": "1.0",
        "message": format!("Node {} restarted successfully", node_id),
        "node_id": node_id,
        "restart_timestamp": chrono::Utc::now().timestamp()
    }))
}

// Get cluster metrics and performance statistics
async fn get_cluster_metrics() -> Json<Value> {
    let (healthy_count, total_count) = NODE_MANAGER.get_health_stats().await;
    let load_balancer_counter = LOAD_BALANCER_COUNTER.load(Ordering::Relaxed);
    
    // Calculate uptime percentage
    let uptime_percentage = if total_count > 0 {
        (healthy_count as f64 / total_count as f64) * 100.0
    } else {
        0.0
    };
    
    Json(json!({
        "success": true,
        "api_version": "1.0",
        "cluster_metrics": {
            "uptime_percentage": uptime_percentage,
            "total_requests_distributed": load_balancer_counter,
            "average_requests_per_node": if total_count > 0 { load_balancer_counter / total_count } else { 0 },
            "cluster_efficiency": if healthy_count > 2 { "optimal" } else if healthy_count > 0 { "degraded" } else { "critical" }
        },
        "performance": {
            "load_distribution": "round_robin",
            "failover_enabled": true,
            "auto_recovery": true,
            "health_check_interval": "30s"
        },
        "resource_usage": {
            "active_mining_sessions": healthy_count * 10, // Estimated
            "total_hash_rate": format!("{} TH/s", healthy_count * 25), // Estimated
            "network_connections": healthy_count * 50 // Estimated
        },
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

async fn create_app() -> Router {
    let state = AppState {
        latest_block: Arc::new(RwLock::new(1)),
        total_transactions: Arc::new(RwLock::new(0)),
        active_nodes: Arc::new(RwLock::new(1)),
        _genesis_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    Router::new()
        // Root endpoint
        .route("/", get(api_info))
        
        // Blockchain endpoints - Consolidated to /api/v1/blockchain/*
        .route("/api/v1/blockchain/blocks", get(get_blocks))
        .route("/api/v1/blockchain/blocks/:height", get(get_block_by_height))
        .route("/api/v1/blockchain/transactions", get(get_transactions))
        .route("/api/v1/blockchain/transactions/:hash", get(get_transaction_by_hash))
        .route("/api/v1/blockchain/network/info", get(get_network_info))
        .route("/api/v1/blockchain/stats", get(get_stats))
        
        // Legacy blockchain endpoints (for backward compatibility)
        .route("/blocks", get(get_blocks))
        .route("/blocks/:height", get(get_block_by_height))
        .route("/transactions", get(get_transactions))
        .route("/transaction/:hash", get(get_transaction_by_hash))
        .route("/network/info", get(get_network_info))
        .route("/stats", get(get_stats))
        
        // Mining endpoints - Consolidated to /api/v1/mining/*
        .route("/api/v1/mining/start", post(start_miner))
        .route("/api/v1/mining/stop", post(stop_miner))
        .route("/api/v1/mining/status", get(get_miner_status))
        .route("/api/v1/mining/heartbeat", post(mining_heartbeat))
        .route("/api/v1/mining/detection/stats", get(mining_detection_stats))
        .route("/api/v1/mining/reward/estimation", get(reward_estimation))
        .route("/api/v1/mining/register", post(miner_register))
        .route("/api/v1/mining/unregister", post(miner_unregister))
        .route("/api/v1/mining/reset", post(reset_miner))
        
        // Legacy mining endpoints (for backward compatibility - will be deprecated)
        .route("/mining/start", post(start_miner))
        .route("/mining/stop", post(stop_miner))
        .route("/mining/status", get(get_mining_status))
        .route("/miner/status", get(get_miner_status))
        
        // Wallet endpoints - Consolidated to /api/v1/wallet/*
        .route("/api/v1/wallet/create", post(wallet_create_post))
        .route("/api/v1/wallet/send", post(wallet_send))
        .route("/api/v1/wallet/balance/:address", get(get_balance))
        .route("/api/v1/wallet/check/:address", get(wallet_check_address))
        .route("/api/v1/wallet/transactions", post(wallet_transactions))
        
        // Legacy wallet endpoints (for backward compatibility)
        .route("/wallet/create", get(wallet_create))
        .route("/wallet/create", post(wallet_create_post))
        .route("/wallet/send", post(wallet_send))
        .route("/wallet/balance/:address", get(get_balance))
        .route("/balance/:address", get(get_balance))
        
        // Device endpoints - Consolidated to /api/v1/device/*
        .route("/api/v1/device/verify", post(device_verify))
        .route("/api/v1/device/validate", post(device_validate))
        .route("/api/v1/device/register", post(device_register))
        .route("/api/v1/device/session/:device_id", get(device_session))
        .route("/api/v1/device/pin-status/:device_id", get(device_pin_status))
        .route("/api/v1/device/create-pin/:device_id", post(device_create_pin))
        .route("/api/v1/device/setup-pin/:device_id", post(device_setup_pin))
        .route("/api/v1/device/verify-pin/:device_id", post(device_verify_pin))
        .route("/api/v1/device/login/:device_id", post(device_login))
        .route("/api/v1/device/reset-pin/:device_id", post(device_reset_pin))
        .route("/api/v1/device/wallet/get", post(device_get_wallet))
        .route("/api/v1/device/wallet/save", post(device_save_wallet))
        .route("/api/v1/device/save-wallet", post(device_save_wallet_address))
        .route("/api/v1/device/wallet/send", post(device_send))
        .route("/api/v1/device/address", get(device_get_address))
        .route("/api/v1/device/private-key/:device_id", post(device_store_private_key))
        .route("/api/v1/device/private-key/:device_id", get(device_get_private_key))
        .route("/api/v1/device/private-key/:device_id", delete(device_remove_private_key))
        .route("/api/v1/device/clear-data/:device_id", delete(device_clear_data))
        .route("/api/v1/device/heartbeat", post(device_heartbeat))
        
        // Legacy device endpoints (for backward compatibility)
        .route("/device/verify", post(device_verify))
        .route("/device/validate", post(device_validate))
        .route("/device/register", post(device_register))
        .route("/device/session/:device_id", get(device_session))
        .route("/device/pin-status/:device_id", get(device_pin_status))
        .route("/device/verify-pin/:device_id", post(device_verify_pin))
        .route("/device/send", post(device_send))
        .route("/device/heartbeat", post(device_heartbeat))
        
        // Admin endpoints - Consolidated to /api/v1/admin/* (removed unused endpoints)
        .route("/api/v1/admin/monitoring/stats", get(get_monitoring_stats))
        .route("/api/v1/admin/monitoring/health", get(get_health_status))
        .route("/api/v1/admin/monitoring/security-events", get(get_security_events))
        
        // Multi-Node Cluster Management endpoints - /api/v1/cluster/*
        .route("/api/v1/cluster/nodes/status", get(get_nodes_status))
        .route("/api/v1/cluster/nodes/:node_id", get(get_node_details))
        .route("/api/v1/cluster/nodes/:node_id/restart", post(restart_node))
        .route("/api/v1/cluster/metrics", get(get_cluster_metrics))
        
        // Legacy admin endpoints (for backward compatibility)
        .route("/admin/monitoring/stats", get(get_monitoring_stats))
        .route("/admin/monitoring/health", get(get_health_status))
        .route("/admin/monitoring/security-events", get(get_security_events))
        
        // Legacy cluster endpoints (for backward compatibility)
        .route("/cluster/nodes/status", get(get_nodes_status))
        .route("/cluster/nodes/:node_id", get(get_node_details))
        .route("/cluster/metrics", get(get_cluster_metrics))
        
        // SSE endpoints - Consolidated to /api/v1/events/*
        .route("/api/v1/events/stream", get(sse_endpoint))
        .route("/api/v1/events/blocks", get(blocks_sse_endpoint))
        .route("/api/v1/events/transactions", get(transactions_sse_endpoint))
        
        // Legacy SSE endpoints (for backward compatibility)
        .route("/events", get(sse_endpoint))
        .route("/events/blocks", get(blocks_sse_endpoint))
        .route("/events/transactions", get(transactions_sse_endpoint))
        
        // Mobile API endpoints - Standardized to /api/v1/mobile/* (no rate limiting)
        .route("/api/v1/mobile/mining/status", get(mobile_mining_status))
        .route("/api/v1/mobile/mining/start", post(mobile_mining_start))
        .route("/api/v1/mobile/mining/stop", post(mobile_mining_stop))
        .route("/api/v1/mobile/wallet/balance", get(mobile_wallet_balance))
        .route("/api/v1/mobile/wallet/transactions", get(mobile_wallet_transactions))
        .route("/api/v1/mobile/wallet/send", post(mobile_wallet_send))
        .route("/api/v1/mobile/blockchain/info", get(mobile_blockchain_info))
        .route("/api/v1/mobile/stats", get(mobile_stats))
        
        // Legacy mobile API endpoints (for backward compatibility)
        .route("/api/mobile/mining/status", get(mobile_mining_status))
        .route("/api/mobile/mining/start", post(mobile_mining_start))
        .route("/api/mobile/mining/stop", post(mobile_mining_stop))
        .route("/api/mobile/wallet/balance", get(mobile_wallet_balance))
        .route("/api/mobile/wallet/transactions", get(mobile_wallet_transactions))
        .route("/api/mobile/wallet/transactions", post(mobile_wallet_transactions_post))
        .route("/api/mobile/wallet/send", post(mobile_wallet_send))
        .route("/api/mobile/blockchain/info", get(mobile_blockchain_info))
        .route("/api/mobile/stats", get(mobile_stats))
        
        // Mobile app specific endpoints (for mobile mining flutter app)
        .route("/mobile/api/health", get(mobile_health_check))
        .route("/mobile/api/mining/status", get(mobile_mining_status))
        .route("/mobile/api/mining/start", post(mobile_mining_start))
        .route("/mobile/api/mining/stop", post(mobile_mining_stop))
        .route("/mobile/api/wallet/balance", get(mobile_wallet_balance))
        .route("/mobile/api/wallet/transactions", get(mobile_wallet_transactions))
        .route("/mobile/api/wallet/transactions", post(mobile_wallet_transactions_post))
        .route("/mobile/api/wallet/send", post(mobile_wallet_send))
        .route("/mobile/api/blockchain/info", get(mobile_blockchain_info))
        .route("/mobile/api/stats", get(mobile_stats))
        
        // Debug endpoints (temporary)
        .route("/debug/registry", get(debug_transaction_registry))
        .route("/debug/active-devices", get(debug_active_devices))
        .route("/debug/all-transactions", get(debug_all_transactions))
        
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any) // Allow all origins for optimal mining experience
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS])
                .allow_headers([
                    "Content-Type".parse().unwrap(),
                    "Authorization".parse().unwrap(),
                    "X-Requested-With".parse().unwrap(),
                    "X-Real-IP".parse().unwrap(),
                    "X-Forwarded-For".parse().unwrap(),
                    "X-API-Key".parse().unwrap(), // Add X-API-Key for mobile mining app
                    "User-Agent".parse().unwrap(),
                    "Accept".parse().unwrap(),
                    "Accept-Language".parse().unwrap(),
                    "Cache-Control".parse().unwrap(),
                ])
                .allow_credentials(false) // No credentials required for optimal access
                .max_age(Duration::from_secs(86400)) // Cache preflight for 24 hours for better performance
        )
}

// Mobile API Handler Functions
async fn mobile_mining_status(Query(params): Query<HashMap<String, String>>) -> Json<Value> {
    let device_id = params.get("device_id").cloned().unwrap_or_default();
    
    // Get mining status from existing function
    let status_response = get_miner_status(Query(params)).await;
    
    // Convert to mobile-optimized format
    Json(json!({
        "success": true,
        "data": {
            "is_mining": status_response.get("mining_active").unwrap_or(&json!(false)),
            "smart_rate": status_response.get("smart_rate").unwrap_or(&json!(0.0)),
            "smart_rate_unit": status_response.get("smart_rate_unit").unwrap_or(&json!("ss/s")),
            "blocks_mined": status_response.get("blocks_mined").unwrap_or(&json!(0)),
            "earnings": status_response.get("estimated_daily_reward").unwrap_or(&json!(0.0)),
            "device_id": device_id,
            "last_update": Utc::now().to_rfc3339()
        }
    }))
}

async fn mobile_mining_start(Json(payload): Json<Value>) -> Json<Value> {
    let device_id = payload.get("device_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let address = payload.get("address")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    
    let request = StartMinerRequest {
        address: address.to_string(),
        device_id: device_id.to_string(),
    };
    
    let result = start_miner_impl(request).await;
    
    // Convert to mobile-optimized format and include session token
    Json(json!({
        "success": result.get("success").unwrap_or(&json!(false)),
        "message": result.get("message").unwrap_or(&json!("Unknown error")),
        "data": {
            "device_id": device_id,
            "mining_address": address,
            "session_token": result.get("session_token"),
            "started_at": Utc::now().to_rfc3339()
        }
    }))
}

async fn mobile_mining_stop(Json(payload): Json<Value>) -> Json<Value> {
    let device_id = payload.get("device_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    
    let request = StopMinerRequest {
        device_id: device_id.to_string(),
        session_token: "mobile_session".to_string(),
    };
    
    let result = stop_miner_impl(request).await;
    
    // Convert to mobile-optimized format
    Json(json!({
        "success": result.get("success").unwrap_or(&json!(false)),
        "message": result.get("message").unwrap_or(&json!("Unknown error")),
        "data": {
            "device_id": device_id,
            "stopped_at": Utc::now().to_rfc3339()
        }
    }))
}

async fn mobile_wallet_balance(Query(params): Query<HashMap<String, String>>) -> Json<Value> {
    let address = params.get("address").cloned().unwrap_or_default();
    let device_id = params.get("device_id").cloned().unwrap_or_default();
    
    if address.is_empty() {
        return Json(json!({
            "success": false,
            "error": "Address parameter required"
        }));
    }
    
    let balance_response = get_balance(Path(address.clone())).await;
    
    Json(json!({
        "success": true,
        "data": {
            "address": address,
            "balance": balance_response.get("balance").unwrap_or(&json!(0)),
            "device_id": device_id,
            "last_update": Utc::now().to_rfc3339()
        }
    }))
}

async fn mobile_wallet_transactions(Query(params): Query<HashMap<String, String>>) -> Json<Value> {
    let address = params.get("address").cloned().unwrap_or_default();
    let limit = params.get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);
    
    // If no address provided, return empty result
    if address.is_empty() {
        return Json(json!({
            "success": true,
            "transactions": [],
            "count": 0,
            "address": "",
            "last_update": Utc::now().to_rfc3339()
        }));
    }
    
    let transactions = get_latest_transactions(limit).await;
    
    // Filter transactions for the specific address
    let filtered_transactions: Vec<_> = transactions.into_iter()
        .filter(|tx| {
            // Convert search address to hex-encoded format for database comparison
            let search_address_hex = if address.starts_with("fvc") {
                // Convert fvc address to hex-encoded format
                hex::encode(address.as_bytes())
            } else {
                // If it's already hex, use as is, otherwise encode it
                if address.chars().all(|c| c.is_ascii_hexdigit()) && address.len() % 2 == 0 {
                    address.clone()
                } else {
                    hex::encode(address.as_bytes())
                }
            };
            
            // Also prepare the original address for comparison
            let search_address_plain = if address.starts_with("fvc") {
                address.clone()
            } else {
                format!("fvc{}", address)
            };
            
            // Handle both hex-encoded and plain addresses in database
            let tx_from_decoded = if tx.from.len() > 40 {
                // Try to decode hex if it's longer than normal address
                hex::decode(&tx.from).ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or(tx.from.clone())
            } else {
                tx.from.clone()
            };
            
            let tx_to_decoded = if tx.to.len() > 40 {
                // Try to decode hex if it's longer than normal address
                hex::decode(&tx.to).ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or(tx.to.clone())
            } else {
                tx.to.clone()
            };
            
            // Special handling for mining rewards - check 'to' field only
            if tx.transaction_type == "mining_reward" {
                // Check both hex-encoded and decoded formats
                tx.to == search_address_hex || 
                tx_to_decoded == address || 
                tx_to_decoded == search_address_plain ||
                tx.to == address ||
                tx.to == search_address_plain
            } else {
                // For regular transactions, check both from and to
                tx.from == search_address_hex || tx.to == search_address_hex ||
                tx_from_decoded == address || tx_to_decoded == address ||
                tx_from_decoded == search_address_plain || tx_to_decoded == search_address_plain ||
                tx.from == address || tx.to == address ||
                tx.from == search_address_plain || tx.to == search_address_plain
            }
        })
        .map(|tx| {
            // Decode hex addresses for display
            let display_from = if tx.from.len() > 40 {
                hex::decode(&tx.from).ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or(tx.from.clone())
            } else {
                tx.from.clone()
            };
            
            let display_to = if tx.to.len() > 40 {
                hex::decode(&tx.to).ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or(tx.to.clone())
            } else {
                tx.to.clone()
            };
            
            json!({
                "hash": tx.hash,
                "from": display_from,
                "to": display_to,
                "amount": tx.amount,
                "timestamp": tx.timestamp,
                "type": if tx.transaction_type == "mining_reward" {
                    "mining_reward"
                } else if display_from == address || tx.from == address {
                    "sent"
                } else {
                    "received"
                }
            })
        })
        .collect();
    
    Json(json!({
        "success": true,
        "transactions": filtered_transactions,
        "count": filtered_transactions.len(),
        "address": address,
        "last_update": Utc::now().to_rfc3339()
    }))
}

// POST version of mobile_wallet_transactions for mobile app compatibility
async fn mobile_wallet_transactions_post(Json(payload): Json<WalletTransactionsRequest>) -> Json<Value> {
    let address = payload.address;
    let limit = payload.limit.unwrap_or(10);
    let transaction_type = payload.transaction_type;
    
    // If no address provided, return empty result
    if address.is_empty() {
        return Json(json!({
            "success": true,
            "transactions": [],
            "total_count": 0,
            "address": "",
            "last_update": Utc::now().to_rfc3339()
        }));
    }
    
    let transactions = get_latest_transactions(limit * 2).await; // Get more to ensure we have enough after filtering
    
    // Filter transactions for the specific address and transaction type
    let filtered_transactions: Vec<_> = transactions.into_iter()
        .filter(|tx| {
            // Filter by transaction type if specified
            let type_matches = if let Some(ref tx_type) = transaction_type {
                tx.transaction_type == *tx_type
            } else {
                true
            };
            
            // Simple address matching - addresses are stored in plain fvc format
            let address_matches = if tx.transaction_type == "mining_reward" {
                // For mining rewards, check 'to' field only
                tx.to == address
            } else {
                // For regular transactions, check both from and to
                tx.from == address || tx.to == address
            };
            
            type_matches && address_matches
        })
        .take(limit) // Limit the results
        .map(|tx| {
            // Decode hex addresses for display
            let display_from = if tx.from.len() > 40 {
                hex::decode(&tx.from).ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or(tx.from.clone())
            } else {
                tx.from.clone()
            };
            
            let display_to = if tx.to.len() > 40 {
                hex::decode(&tx.to).ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or(tx.to.clone())
            } else {
                tx.to.clone()
            };
            
            json!({
                "hash": tx.hash,
                "from": display_from,
                "to": display_to,
                "amount": tx.amount,
                "timestamp": tx.timestamp,
                "transaction_type": tx.transaction_type,
                "type": if tx.transaction_type == "mining_reward" {
                    "mining_reward"
                } else if display_from == address || tx.from == address {
                    "sent"
                } else {
                    "received"
                }
            })
        })
        .collect();
    
    Json(json!({
        "success": true,
        "transactions": filtered_transactions,
        "total_count": filtered_transactions.len(),
        "address": address,
        "last_update": Utc::now().to_rfc3339()
    }))
}

// Debug endpoint to check transaction registry
async fn debug_transaction_registry() -> Json<Value> {
    match RPCStorage::get_transaction_count().await {
        Ok(count) => {
            // Get all transactions to see what's in storage
            let transactions = get_latest_transactions(100).await;
            let tx_hashes: Vec<String> = transactions.iter().map(|tx| tx.hash.clone()).collect();
            
            Json(json!({
                "success": true,
                "storage_count": count,
                "transactions_found": transactions.len(),
                "transaction_hashes": tx_hashes,
                "last_10_transactions": transactions.iter().take(10).collect::<Vec<_>>()
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to get transaction count: {}", e)
            }))
        }
    }
}

async fn debug_active_devices() -> Json<Value> {
    // Get all active devices
    let active_devices = match RPCStorage::get_all_active_devices().await {
        Ok(devices) => devices,
        Err(_) => vec![],
    };
    
    // Get session registry through a helper function
    let session_registry = RPCStorage::get_session_registry().await.unwrap_or_default();
    
    // Get device addresses and session details
    let mut device_info = Vec::new();
    for device_id in &active_devices {
        let address = RPCStorage::get_device_address(device_id).await.unwrap_or(None);
        let session = RPCStorage::get_device_session(device_id).await.unwrap_or(None);
        
        device_info.push(json!({
            "device_id": device_id,
            "address": address,
            "session": session,
            "in_registry": session_registry.contains(device_id)
        }));
    }
    
    // Also check all devices in session registry
    let mut registry_info = Vec::new();
    for device_id in &session_registry {
        let address = RPCStorage::get_device_address(device_id).await.unwrap_or(None);
        let session = RPCStorage::get_device_session(device_id).await.unwrap_or(None);
        
        registry_info.push(json!({
            "device_id": device_id,
            "address": address,
            "session": session,
            "is_active": active_devices.contains(device_id)
        }));
    }
    
    Json(json!({
        "active_devices_count": active_devices.len(),
        "active_devices": active_devices,
        "device_addresses": device_info,
        "session_registry_count": session_registry.len(),
        "session_registry": session_registry,
        "registry_details": registry_info,
        "timestamp": Utc::now().to_rfc3339()
    }))
}

// Debug endpoint to check all transactions
async fn debug_all_transactions() -> Json<Value> {
    let transactions = get_latest_transactions(50).await;
    Json(json!({
        "success": true,
        "total_transactions": transactions.len(),
        "transactions": transactions.iter().map(|tx| json!({
            "hash": tx.hash,
            "from": tx.from,
            "to": tx.to,
            "amount": tx.amount,
            "timestamp": tx.timestamp,
            "transaction_type": tx.transaction_type,
            "block_height": tx.block_height
        })).collect::<Vec<_>>()
    }))
}

async fn mobile_wallet_send(Json(payload): Json<Value>) -> Json<Value> {
    let from = payload.get("from")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let to = payload.get("to")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let amount = payload.get("amount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let private_key = payload.get("private_key")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let device_id = payload.get("device_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    
    let request = DeviceSendRequest {
        from: from.to_string(),
        to: to.to_string(),
        amount,
        private_key: private_key.to_string(),
        device_id: device_id.to_string(),
    };
    
    let state = AppState {
        latest_block: Arc::new(RwLock::new(1)),
        total_transactions: Arc::new(RwLock::new(0)),
        active_nodes: Arc::new(RwLock::new(1)),
        _genesis_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    let result = device_send_impl(State(state), request).await;
    
    // Convert to mobile-optimized format
    Json(json!({
        "success": result.get("success").unwrap_or(&json!(false)),
        "message": result.get("message").unwrap_or(&json!("Unknown error")),
        "data": {
            "transaction_hash": result.get("transaction_hash"),
            "from": from,
            "to": to,
            "amount": amount,
            "device_id": device_id,
            "timestamp": Utc::now().to_rfc3339()
        }
    }))
}

async fn mobile_blockchain_info() -> Json<Value> {
    // Use RPCStorage as single source of truth for consistency with API v1
    match RPCStorage::get_network_info().await {
        Ok(network_info) => {
            Json(json!({
                "success": true,
                "data": {
                    "chain_id": network_info.get("chain_id").unwrap_or(&json!("FVChain")),
                    "latest_block_height": network_info.get("latest_block_height").unwrap_or(&json!(1)),
                    "transaction_count": network_info.get("transaction_count").unwrap_or(&json!(0)),
                    "active_nodes": network_info.get("active_nodes").unwrap_or(&json!(1)),
                    "network_smart_rate": network_info.get("network_smart_rate").unwrap_or(&json!(0.0)),
                    "total_supply": network_info.get("total_supply").unwrap_or(&json!(0)),
                    "circulating_supply": network_info.get("circulating_supply").unwrap_or(&json!(0)),
                    "avg_block_time": network_info.get("avg_block_time").unwrap_or(&json!(5)),
                    "difficulty": network_info.get("difficulty").unwrap_or(&json!(1.0)),
                    "last_update": Utc::now().to_rfc3339()
                }
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to get blockchain info: {}", e)
            }))
        }
    }
}

async fn mobile_stats() -> Json<Value> {
    let stats = get_stats().await;
    let network_info = get_network_info().await;
    
    Json(json!({
        "success": true,
        "data": {
            "blockchain": {
                "latest_block": stats.get("latest_block").unwrap_or(&json!(1)),
                "total_transactions": stats.get("total_transactions").unwrap_or(&json!(0)),
                "network_smart_rate": stats.get("network_smart_rate").unwrap_or(&json!(0.0)),
                "smart_rate_unit": stats.get("smart_rate_unit").unwrap_or(&json!("ss/s"))
            },
            "network": {
                "active_nodes": stats.get("active_nodes").unwrap_or(&json!(1)),
                "chain_id": network_info.get("chain_id").unwrap_or(&json!("FVChain")),
                "difficulty": stats.get("difficulty").unwrap_or(&json!(1.0))
            },
            "mining": {
                "vortex_energy_rate": stats.get("vortex_energy_rate").unwrap_or(&json!(0.0)),
                "fractal_contribution_score": stats.get("fractal_contribution_score").unwrap_or(&json!(0.0)),
                "mathematical_efficiency_index": stats.get("mathematical_efficiency_index").unwrap_or(&json!(0.0))
            },
            "last_update": Utc::now().to_rfc3339()
        }
    }))
}

// Mobile API health check endpoint
async fn mobile_health_check() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "status": "healthy",
            "timestamp": Utc::now().to_rfc3339(),
            "version": "1.0.0",
            "api_version": "v1",
            "uptime_seconds": 86400,
            "services": {
                "mining": "operational",
                "wallet": "operational",
                "blockchain": "operational",
                "database": "operational"
            }
        },
        "error": null,
        "timestamp": Utc::now().to_rfc3339()
    }))
}

// Device heartbeat to update activity timestamp
async fn device_heartbeat(Json(payload): Json<Value>) -> Json<Value> {
    let device_id = payload["device_id"].as_str().unwrap_or("");
    let session_token = payload["session_token"].as_str().unwrap_or("");
    
    if device_id.is_empty() || session_token.is_empty() {
        return Json(json!({
            "success": false,
            "error": "Missing required fields: device_id, session_token"
        }));
    }
    
    // Verify device session and update timestamp
    match RPCStorage::get_device_session(device_id).await {
        Ok(Some((stored_token, _))) => {
            if stored_token != session_token {
                return Json(json!({
                    "success": false,
                    "error": "Invalid session token"
                }));
            }
            
            // Update session timestamp
            let current_time = chrono::Utc::now().timestamp() as u64;
            if let Err(e) = RPCStorage::set_device_session(device_id, session_token, current_time).await {
                return Json(json!({
                    "success": false,
                    "error": format!("Failed to update heartbeat: {}", e)
                }));
            }
            
            Json(json!({
                "success": true,
                "message": "Heartbeat updated",
                "timestamp": current_time
            }))
        }
        Ok(None) => {
            Json(json!({
                "success": false,
                "error": "Device session not found"
            }))
        }
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Failed to verify session: {}", e)
            }))
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Starting Fractal Vortex Chain Integrated Node RPC Server...");
    
    // Initialize storage
    if let Err(e) = RPCStorage::create_genesis_block().await {
        eprintln!("Failed to create genesis block: {}", e);
    }
    
    // Initialize 4 blockchain nodes and start ecosystem mining
    println!("🔄 Starting multi-node blockchain initialization...");
    if let Err(e) = initialize_blockchain_nodes().await {
        eprintln!("❌ Failed to initialize blockchain nodes: {}", e);
        std::process::exit(1);
    } else {
        println!("✅ Multi-node blockchain cluster and ecosystem mining initialized successfully");
    }
    
    // Start AUTO_DETECTION monitoring
    println!("🔍 Starting AUTO_DETECTION monitoring...");
    AUTO_DETECTION.start_monitoring().await;
    println!("✅ AUTO_DETECTION monitoring started successfully");
    
    // Start session cleanup scheduler
    let cleanup_handle = tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // Every 5 minutes
        loop {
            interval.tick().await;
            
            // Cleanup expired sessions (older than 24 hours)
            if let Err(e) = RPCStorage::cleanup_old_sessions(86400).await {
                log::error!("Failed to cleanup old sessions: {}", e);
            } else {
                log::info!("Session cleanup completed");
            }
            
            // Also cleanup inactive device sessions (no heartbeat for 5 minutes)
            match RPCStorage::get_all_active_devices().await {
                Ok(devices) => {
                    let current_time = chrono::Utc::now().timestamp() as u64;
                    for device_id in devices {
                        if let Ok(Some((_, last_activity))) = RPCStorage::get_device_session(&device_id).await {
                            let inactive_time = current_time.saturating_sub(last_activity);
                            
                            // Remove sessions with no heartbeat for 5 minutes (300 seconds)
                            if inactive_time > 300 {
                                if let Err(e) = RPCStorage::remove_device_session(&device_id).await {
                                    log::error!("Failed to remove inactive device session {}: {}", device_id, e);
                                } else {
                                    log::info!("Removed inactive device session: {} (inactive for {} seconds)", device_id, inactive_time);
                                }
                            }
                            // Also remove sessions older than 24 hours regardless of activity
                            else if inactive_time > 86400 {
                                if let Err(e) = RPCStorage::remove_device_session(&device_id).await {
                                    log::error!("Failed to remove expired device session {}: {}", device_id, e);
                                } else {
                                    log::info!("Removed expired device session: {} (age: {} seconds)", device_id, inactive_time);
                                }
                            }
                        }
                    }
                }
                Err(e) => log::error!("Failed to get active devices for cleanup: {}", e),
            }
        }
    });
    
    // Start the server
    let app = create_app().await;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    
    println!("🚀 Fractal Vortex Chain RPC Server running on http://0.0.0.0:8080");
    println!("🧹 Session cleanup scheduler started (every 5 minutes)");
    
    // Run server and cleanup scheduler concurrently
    tokio::select! {
        _ = axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()) => {},
        _ = cleanup_handle => {},
    }
}
