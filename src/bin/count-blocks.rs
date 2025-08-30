//! Binary untuk menghitung dan menganalisis blok yang tersimpan di database

use fractal_vortex_chain::rpc_storage::RPCStorage;
use std::collections::HashMap;
use fractal_vortex_chain::storage::StorageError;
use fractal_vortex_chain::rpc_storage::Block;

/// Count all blocks stored in the database
async fn count_all_stored_blocks() -> Result<(u64, Vec<u64>), StorageError> {
    let mut stored_blocks = Vec::new();
    let latest_height = RPCStorage::get_block_height().await.unwrap_or(0);
    
    println!("🔍 Scanning blocks from height 0 to {}...", latest_height);
    
    // Check blocks from 0 to latest_height
    for height in 0..=latest_height {
        if height % 100 == 0 {
            println!("   Checking height {}...", height);
        }
        
        if let Ok(Some(_block)) = RPCStorage::get_block_by_height(height).await {
            stored_blocks.push(height);
        }
    }
    
    let count = stored_blocks.len() as u64;
    Ok((count, stored_blocks))
}

/// Get detailed information about all stored blocks
async fn get_all_stored_blocks_info() -> Result<HashMap<u64, Block>, StorageError> {
    let mut blocks_map = HashMap::new();
    let latest_height = RPCStorage::get_block_height().await.unwrap_or(0);
    
    // Check blocks from 0 to latest_height
    for height in 0..=latest_height {
        if let Ok(Some(block)) = RPCStorage::get_block_by_height(height).await {
            blocks_map.insert(height, block);
        }
    }
    
    Ok(blocks_map)
}

/// Print detailed storage analysis
async fn analyze_block_storage() {
    println!("🔍 Analyzing blockchain storage...");
    
    match count_all_stored_blocks().await {
        Ok((count, stored_heights)) => {
            let latest_height = RPCStorage::get_block_height().await.unwrap_or(0);
            
            println!("\n📊 Storage Analysis Results:");
            println!("   Latest Block Height: {}", latest_height);
            println!("   Total Stored Blocks: {}", count);
            
            if latest_height > 0 {
                println!("   Storage Efficiency: {:.2}%", (count as f64 / (latest_height + 1) as f64) * 100.0);
            }
            
            if stored_heights.len() <= 20 {
                println!("   Stored Block Heights: {:?}", stored_heights);
            } else {
                println!("   First 10 Stored Heights: {:?}", &stored_heights[0..10]);
                println!("   Last 10 Stored Heights: {:?}", &stored_heights[stored_heights.len()-10..]);
            }
            
            // Find gaps in storage
            let mut gaps = Vec::new();
            for height in 0..=latest_height {
                if !stored_heights.contains(&height) {
                    gaps.push(height);
                }
            }
            
            if gaps.len() > 0 {
                println!("   Missing Block Heights: {} gaps found", gaps.len());
                if gaps.len() <= 20 {
                    println!("   Missing Heights: {:?}", gaps);
                } else {
                    println!("   First 10 Missing: {:?}", &gaps[0..10]);
                    println!("   Last 10 Missing: {:?}", &gaps[gaps.len()-10..]);
                }
            } else {
                println!("   ✅ No gaps found - all blocks stored sequentially");
            }
            
            // Get detailed info about stored blocks
            println!("\n📋 Detailed Block Information:");
            match get_all_stored_blocks_info().await {
                Ok(blocks_info) => {
                    let mut total_transactions = 0u64;
                    for (height, block) in &blocks_info {
                        total_transactions += block.transaction_count;
                        if blocks_info.len() <= 10 {
                            println!("   Block #{}: {} transactions, miner: {}", 
                                height, block.transaction_count, block.miner);
                        }
                    }
                    
                    if blocks_info.len() > 10 {
                        println!("   (Showing summary for {} blocks)", blocks_info.len());
                    }
                    
                    println!("   Total Transactions in Stored Blocks: {}", total_transactions);
                    
                    if count > 0 {
                        println!("   Average Transactions per Block: {:.2}", 
                            total_transactions as f64 / count as f64);
                    }
                },
                Err(e) => {
                    println!("   ❌ Error getting block details: {:?}", e);
                }
            }
        },
        Err(e) => {
            println!("❌ Error analyzing storage: {:?}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    println!("🚀 Fractal Vortex Chain - Block Storage Analyzer");
    println!("================================================\n");
    
    // Get current transaction count for comparison
    match RPCStorage::get_transaction_count().await {
        Ok(tx_count) => {
            println!("📈 Current Transaction Count: {}", tx_count);
        },
        Err(e) => {
            println!("❌ Error getting transaction count: {:?}", e);
        }
    }
    
    // Analyze block storage
    analyze_block_storage().await;
    
    println!("\n✅ Analysis complete!");
}