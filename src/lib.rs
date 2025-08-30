//! # Fractal-Vortex Chain (FVC)
//! 
//! A revolutionary blockchain architecture that combines fractal self-similarity
//! with vortex mathematics for unprecedented scalability and energy efficiency.

/// Mathematical constants for fractal-vortex calculations
pub mod math {
    /// Golden ratio for optimal fractal distribution
    pub const GOLDEN_RATIO: f64 = 1.6180339887498948482045868343656;
    
    /// Sierpinski triangle fractal dimension
    pub const SIERPINSKI_DIMENSION: f64 = 1.584962500721156;
    
    /// Vortex mathematics sequence (1-2-4-8-7-5)
    pub const VORTEX_SEQUENCE: [u8; 6] = [1, 2, 4, 8, 7, 5];
    
    /// Torus surface area constant
    pub const TORUS_SURFACE: f64 = 39.47841760435743; // 4π²Rr
}

/// Node health utilities
pub mod node_health {
    use std::sync::atomic::{AtomicU32, Ordering};
    
    static CACHED_ACTIVE_NODES: AtomicU32 = AtomicU32::new(1);
    
    /// Update cached active nodes count
    pub fn update_active_nodes_count(count: u32) {
        CACHED_ACTIVE_NODES.store(count, Ordering::Relaxed);
    }
    
    /// Get cached active nodes count
    pub fn get_active_nodes_count() -> u32 {
        CACHED_ACTIVE_NODES.load(Ordering::Relaxed)
    }
}

/// Utility functions for fractal-vortex operations
pub mod utils {
    use super::math::*;
    
    /// Calculate digital root using vortex mathematics
    pub fn digital_root(n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        let mut sum = n;
        while sum >= 10 {
            sum = sum.to_string()
                .chars()
                .map(|c| c.to_digit(10).unwrap() as u64)
                .sum();
        }
        sum
    }
    
    /// Calculate vortex energy from data
    pub fn vortex_energy(data: &[u8]) -> f64 {
        let mut energy = 0.0;
        for (i, &byte) in data.iter().enumerate() {
            let vortex_val = VORTEX_SEQUENCE[i % VORTEX_SEQUENCE.len()] as f64;
            energy += (byte as f64 * vortex_val) / (i as f64 + 1.0);
        }
        energy.abs()
    }
    
    /// Generate Sierpinski triangle coordinates
    pub fn sierpinski_coordinates(level: u32) -> Vec<(f64, f64)> {
        let mut coords = Vec::new();
        let _size = 2.0f64.powi(level as i32);
        
        for i in 0..=level {
            for j in 0..=(2u32.pow(i)) {
                let x = (j as f64) / (2u32.pow(i) as f64);
                let y = (i as f64) / (level as f64);
                coords.push((x, y));
            }
        }
        
        coords
    }
    
    /// Calculate torus coordinates from seed
    pub fn torus_coordinates(seed: u64) -> (f64, f64, f64) {
        let phi = (seed as f64 * GOLDEN_RATIO) % (2.0 * std::f64::consts::PI);
        let theta = (seed as f64 / GOLDEN_RATIO) % (2.0 * std::f64::consts::PI);
        let radius = 1.0 + (seed as f64 / 1000.0).sin() * 0.5;
        (phi, theta, radius)
    }
}

/// Core types and structures
pub mod types {
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Block {
        pub hash: String,
        pub height: u64,
        pub timestamp: u64,
        pub parent_hash: String,
        pub transactions: Vec<Transaction>,
        pub fractal_complexity: f64,
        pub vortex_energy: f64,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Transaction {
        pub hash: String,
        pub from: String,
        pub to: String,
        pub amount: u64,
        pub nonce: u64,
        pub timestamp: u64,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Validator {
        pub id: String,
        pub address: String,
        pub stake: u64,
        pub fractal_energy: f64,
    }
}

/// Cryptographic primitives
pub mod crypto;

/// Wallet implementation
pub mod wallet;

/// Persistent storage module
pub mod storage;

/// Security framework
pub mod security;

/// Shared utilities and common types
pub mod shared;

/// Node implementation for fractal-vortex blockchain
pub mod node;

/// Consensus algorithms for fractal-vortex blockchain
pub mod consensus;

/// Network topology and routing
pub mod network;

/// Mining and auto-detection
pub mod mining;
pub mod rate_limiter;
pub mod rpc_storage;
pub mod api_auth;
pub mod input_validation;
pub mod api_monitoring;

/// Version information
pub const VERSION: &str = "1.0.0";
pub const CHAIN_ID: &str = "fractal-vortex-mainnet";