use axum::{
    routing::{get, post},
    Router,
    response::Json,
    extract::{State, Path, Query},
};
use serde::Deserialize;
use fractal_vortex_chain::wallet::key_manager::KeyManager;
use fractal_vortex_chain::storage::LedgerDB;
use rand::random;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use axum::http::Method;
use std::sync::atomic::{AtomicBool, Ordering};
use once_cell::sync::Lazy;
use tokio::sync::RwLock as TokioRwLock;

use std::time::Duration;
use axum::response::sse::{Event, Sse};
use futures_util::stream::{Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use tokio::sync::broadcast;
use std::convert::Infallible;


#[derive(Clone)]
struct AppState {
    latest_block: Arc<RwLock<u64>>,
    total_transactions: Arc<RwLock<u64>>,
    active_nodes: Arc<RwLock<u64>>,
}

#[allow(dead_code)]
async fn get_latest_block(State(state): State<AppState>) -> Json<Value> {
    let block_height = *state.latest_block.read().await;
    Json(serde_json::json!({
        "height": block_height,
        "hash": format!("0x{:064x}", block_height + 12345),
        "timestamp": chrono::Utc::now().timestamp(),
        "transaction_count": 200,
        "miner": "Validator-Node-0",
        "size": 1000000
    }))
}

async fn get_blockchain_info(State(state): State<AppState>) -> Json<Value> {
    let latest_block = *state.latest_block.read().await;
    let total_tx = *state.total_transactions.read().await;
    let active_nodes = *state.active_nodes.read().await;
    
    Json(serde_json::json!({
        "latest_block_height": latest_block,
        "transaction_count": total_tx,
        "active_nodes": active_nodes,
        "avg_block_time": 15,
        "total_supply": 1000000000.0,
        "circulating_supply": 950000000.0
    }))
}

#[derive(Deserialize)]
struct LimitQuery {
    limit: Option<usize>,
}

async fn get_blocks(State(state): State<AppState>, Query(query): Query<LimitQuery>) -> Json<Value> {
    let latest = *state.latest_block.read().await;
    let limit = query.limit.unwrap_or(10).min(50);
    let blocks: Vec<Value> = (0..limit).map(|i| {
        let height = latest.saturating_sub(i as u64);
        serde_json::json!({
            "hash": format!("0x{:064x}", height + 123456789),
            "parent_hash": format!("0x{:064x}", height + 123456788),
            "height": height,
            "timestamp": chrono::Utc::now().timestamp() - (i as i64 * 15),
            "data": "Fractal Vortex Block Data",
            "fractal_level": 7,
            "vortex_seed": 42,
            "size": 1000000 + (i * 250000),
            "transaction_count": 200 + (i * 47),
            "miner": format!("0x{:040x}", i + 1234),
            "reward": 6.25
        })
    }).collect();
    
    Json(serde_json::json!({"blocks": blocks}))
}

async fn get_transactions(Query(query): Query<LimitQuery>) -> Json<Value> {
    let limit = query.limit.unwrap_or(20).min(100);
    let txs = TRANSACTIONS.read().await;
    let start = txs.len().saturating_sub(limit);
    let slice = &txs[start..];
    let transactions: Vec<Value> = slice.iter().rev().map(|t| {
        serde_json::json!({
            "hash": t.hash,
            "from": t.from,
            "to": t.to,
            "amount": (t.amount as f64) / 1_000_000.0, // convert micro -> FVC for API
            "timestamp": t.timestamp,
        })
    }).collect();
    Json(serde_json::json!({"transactions": transactions}))
}

async fn get_transaction(Path(hash): Path<String>) -> Json<Value> {
    Json(serde_json::json!({
        "hash": hash,
        "from": "0x1234567890123456789012345678901234567890",
        "to": "0x0987654321098765432109876543210987654321",
        "amount": 2.5,
        "fee": 0.001,
        "nonce": 42,
        "timestamp": chrono::Utc::now().timestamp(),
        "signature": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        "block_hash": "0x1111111111111111111111111111111111111111111111111111111111111111",
        "block_height": 1247839,
        "status": "confirmed"
    }))
}

async fn get_block(Path(height): Path<u64>) -> Json<Value> {
    Json(serde_json::json!({
        "hash": format!("0x{:064x}", height + 123456789),
        "parent_hash": format!("0x{:064x}", height + 123456788),
        "height": height,
        "timestamp": chrono::Utc::now().timestamp() - ((1247839 - height as i64) * 15),
        "data": "Fractal Vortex Block Data",
        "fractal_level": 7,
        "vortex_seed": 42,
        "size": 1000000,
        "transaction_count": 200,
        "miner": "0x1234567890123456789012345678901234567890",
        "reward": 6.25
    }))
}

static MINER_RUNNING: AtomicBool = AtomicBool::new(false);
static LEDGER: Lazy<Arc<LedgerDB>> = Lazy::new(|| {
    Arc::new(LedgerDB::open("./ledger").expect("Failed to open ledger DB"))
});


static BROADCAST: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
    let (tx, _rx) = broadcast::channel(100);
    tx
});

#[derive(Deserialize)]
struct StartMinerRequest {
    address: String,
}

async fn start_miner(Json(payload): Json<StartMinerRequest>) -> Json<Value> {
    // If already running, just respond
    if MINER_RUNNING.swap(true, Ordering::SeqCst) {
        return Json(serde_json::json!({"status": "running"}));
    }

    // Spawn async mining loop
    tokio::spawn({
        let ledger = LEDGER.clone();
        let address = payload.address.clone();
        async move {
        while MINER_RUNNING.load(Ordering::SeqCst) {
            {
                // Persist mining reward to ledger (6.25 FVC = 6_250_000 micro-FVC)
                let current = ledger.get_balance(&address).await.unwrap_or(0);
                let new_balance = current + 6_250_000;
                let _ = ledger.set_balance(&address, new_balance).await;
                // Catat reward mining sebagai transaksi
                let reward_tx = WalletTransaction {
                    from: "0x0000000000000000000000000000000000000000".to_string(),
                    to: address.clone(),
                    amount: 6_250_000,
                    nonce: 0,
                    fee: 0,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    signature: None,
                    hash: Some(format!("0xrew{:x}", random::<u64>())),
                };
                {
                    let mut txs = TRANSACTIONS.write().await;
                    txs.push(reward_tx);
                }
                let msg = serde_json::json!({"event":"balance","balance":new_balance}).to_string();
                let _ = BROADCAST.send(msg);
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }});

    Json(serde_json::json!({"status": "running", "address": payload.address}))
}

async fn stop_miner() -> Json<Value> {
    MINER_RUNNING.store(false, Ordering::SeqCst);
    Json(serde_json::json!({"status": "stopped"}))
}

async fn miner_status() -> Json<Value> {
    let running = MINER_RUNNING.load(Ordering::SeqCst);
    Json(serde_json::json!({
        "running": running,
        "reward": if running { 12.5 } else { 0.0 },
        "hash_rate": if running { 1234 } else { 0 },
        "balance": 0
    }))
}

#[derive(Deserialize)]
struct SendRequest {
    from: String,
    to: String,
    amount: u64,
    private_key: String,
}

#[derive(Deserialize)]
struct AddressQuery {
    address: String,
}

#[derive(Deserialize)]
struct WalletTransaction {
    from: String,
    to: String,
    amount: u64,
    nonce: u64,
    fee: u64,
    timestamp: u64,
    signature: Option<Vec<u8>>,    
    hash: Option<String>,
}

// In-memory transaction storage (simple, non-persistent)
static TRANSACTIONS: Lazy<TokioRwLock<Vec<WalletTransaction>>> = Lazy::new(|| {
    TokioRwLock::new(Vec::new())
});

async fn post_transaction(State(state): State<AppState>, Json(_tx): Json<WalletTransaction>) -> Json<Value> {
    // Generate pseudo-random tx hash
    let tx_hash = format!("0x{:016x}", random::<u64>());

    // Increase total transaction counter
    {
        let mut total_tx = state.total_transactions.write().await;
        *total_tx += 1;
    }

    // Broadcast new transaction event (best-effort)
    let _ = BROADCAST.send(serde_json::json!({"event":"new_tx","tx_hash":tx_hash}).to_string());

    Json(serde_json::json!({"hash": tx_hash}))
}

async fn wallet_create() -> Json<Value> {
    let km = KeyManager::new();
    // Give 10 FVC test funds (10_000_000 micro)
    let _ = LEDGER.set_balance(&km.get_address(), 10_000_000).await;
    Json(serde_json::json!({
        "address": km.get_address(),
        "private_key": km.get_private_key()
    }))
}

async fn wallet_balance(Query(query): Query<AddressQuery>) -> Json<Value> {
    let balance = LEDGER.get_balance(&query.address).await.unwrap_or(0);
    Json(serde_json::json!({
        "address": query.address,
        "balance": balance
    }))
}

async fn sse_endpoint() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = BROADCAST.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|msg| async move { msg.ok() })
        .map(|data| Ok::<_, Infallible>(Event::default().data(data)));
    Sse::new(stream)
}

async fn wallet_send(Json(payload): Json<SendRequest>) -> Json<Value> {
    let amount = payload.amount;
    // Debit sender
    let sender_balance = LEDGER.get_balance(&payload.from).await.unwrap_or(0);
    if sender_balance < amount {
        return Json(serde_json::json!({"error":"Insufficient balance"}));
    }
    let _ = LEDGER.set_balance(&payload.from, sender_balance - amount).await;
    // Credit receiver
    let receiver_balance = LEDGER.get_balance(&payload.to).await.unwrap_or(0);
    let _ = LEDGER.set_balance(&payload.to, receiver_balance + amount).await;
    let tx_hash = format!("0xtx{:x}", random::<u64>());
    // Build transaction record
    let tx = WalletTransaction {
        from: payload.from.clone(),
        to: payload.to.clone(),
        amount,
        nonce: 0,
        fee: 0,
        timestamp: chrono::Utc::now().timestamp() as u64,
        signature: None,
        hash: Some(tx_hash.clone()),
    };
    {
        let mut txs = TRANSACTIONS.write().await;
        txs.push(tx);
    }
    // Broadcast transaction event
    let _ = BROADCAST.send(serde_json::json!({"event":"transfer","tx_hash":tx_hash}).to_string());
    Json(serde_json::json!({"tx_hash": tx_hash}))
}

async fn get_address(Path(address): Path<String>) -> Json<Value> {
    Json(serde_json::json!({
        "address": address,
        "balance": 100.5,
        "transaction_count": 25,
        "first_seen": chrono::Utc::now().timestamp() - 86400,
        "last_seen": chrono::Utc::now().timestamp()
    }))
}

#[tokio::main]
async fn main() {
    let state = AppState {
        latest_block: Arc::new(RwLock::new(1247839)),
        total_transactions: Arc::new(RwLock::new(1247839)),
        active_nodes: Arc::new(RwLock::new(4)),
    };

    // Added CORS layer configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);
    
    // Applied CORS layer to router
    let app = Router::new()
        .route("/network/info", get(get_blockchain_info))
        .route("/blocks/latest", get(get_blocks))
        .route("/blocks", get(get_blocks))
        .route("/transactions", get(get_transactions))
        .route("/transactions/latest", get(get_transactions))
        .route("/transaction", post(post_transaction))
        .route("/transaction/:hash", get(get_transaction))
        .route("/block/:height", get(get_block))
        .route("/miner/start", post(start_miner))
        .route("/miner/stop", post(stop_miner))
        .route("/miner/status", get(miner_status))
        // Wallet routes
        .route("/wallet/create", get(wallet_create).post(wallet_create))
        .route("/wallet/balance", get(wallet_balance))
        .route("/wallet/send", post(wallet_send))
        .route("/address/:address", get(get_address))
        .route("/events", get(sse_endpoint))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    
    println!("🌐 RPC Server running on http://localhost:8080");
    
    axum::serve(listener, app).await.unwrap();
}