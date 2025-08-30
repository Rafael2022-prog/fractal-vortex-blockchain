use crate::consensus::{DifficultyAdjuster, MiningRewardSystem, RewardDistribution};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct MiningEngine {
    difficulty_adjuster: DifficultyAdjuster,
    reward_system: MiningRewardSystem,
    current_difficulty: u64,
    current_block_height: u64,
    block_times: Vec<u64>,
    last_block_timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct MiningResult {
    pub block_hash: String,
    pub nonce: u64,
    pub difficulty: u64,
    pub reward_distribution: RewardDistribution,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct MiningStats {
    pub network_smart_rate: f64,
    pub difficulty: u64,
    pub block_reward: f64,
    pub blocks_per_day: f64,
    pub total_supply_mined: f64,
    pub next_difficulty_estimate: u64,
}

impl MiningEngine {
    pub fn new() -> Self {
        let difficulty_adjuster = DifficultyAdjuster::new(5, 2023); // 5 second blocks, adjust every 2023 blocks
        let reward_system = MiningRewardSystem::new();
        
        Self {
            difficulty_adjuster,
            reward_system,
            current_difficulty: 10, // Initial difficulty set to 10
            current_block_height: 0,
            block_times: Vec::new(),
            last_block_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    // Calculate Smart Rate based on blockchain metrics
    fn calculate_smart_rate(&self) -> f64 {
        // Calculate individual components
        let vortex_energy = self.calculate_vortex_energy_rate();
        let fractal_score = self.calculate_fractal_contribution_score();
        let efficiency_index = self.calculate_mathematical_efficiency_index();
        let harmony_factor = self.calculate_network_harmony_factor();
        
        // Normalize components to 0-1 range
        let normalized_ver = (vortex_energy / 1000.0).min(1.0);
        let normalized_fcs = (fractal_score / 1000.0).min(1.0);
        let normalized_mei = (efficiency_index / 1000.0).min(1.0);
        let normalized_nhf = (harmony_factor / 1000.0).min(1.0);
        
        // Weights for each component (sum = 1.0)
        let w_ver = 0.30; // Vortex Energy Rate weight
        let w_fcs = 0.25; // Fractal Contribution Score weight
        let w_mei = 0.25; // Mathematical Efficiency Index weight
        let w_nhf = 0.20; // Network Harmony Factor weight
        
        // Calculate weighted geometric mean
        let weighted_geometric_mean = (normalized_ver.powf(w_ver) * 
                                     normalized_fcs.powf(w_fcs) * 
                                     normalized_mei.powf(w_mei) * 
                                     normalized_nhf.powf(w_nhf)).max(0.001);
        
        // Base Smart Rate (in Smart Steps per Second)
        let base_smart_rate = 1000.0;
        
        // Vortex Pattern based on block height
        let vortex_pattern = 1.0 + (0.1 * ((self.current_block_height as f64 * 0.618).sin() + 1.0));
        
        // Final Smart Rate calculation
        let smart_rate = base_smart_rate * weighted_geometric_mean * vortex_pattern;
        
        // Round to 2 decimal places
        (smart_rate * 100.0).round() / 100.0
    }
    
    fn calculate_vortex_energy_rate(&self) -> f64 {
        if self.current_block_height == 0 {
            return 0.0;
        }
        
        // Simulate transaction throughput (simplified for mining engine)
        let throughput: f64 = 1.0; // 1 transaction per block average
        
        // Energy efficiency factor
        let energy_efficiency: f64 = 0.85; // 85% efficiency
        
        // Network load factor
        let network_load = (throughput / 10.0).min(1.0); // Normalize to max 10 tx/block
        
        // Vortex Energy Rate calculation
        let ver = throughput * energy_efficiency * (1.0 + network_load) * 100.0;
        
        // Ensure minimum value and round
        ver.max(1.0).min(1000.0)
    }
    
    fn calculate_fractal_contribution_score(&self) -> f64 {
        if self.current_block_height <= 1 {
            return 0.0;
        }
        
        // Fractal depth based on block height
        let fractal_depth = (self.current_block_height as f64).log2().floor();
        
        // Transaction density (simplified for mining engine)
        let tx_density = 1.0; // 1 transaction per block average
        
        // Fractal pattern strength
        let pattern_strength = (fractal_depth / 20.0).min(1.0); // Normalize to max depth 20
        
        // Contribution score calculation
        let fcs = tx_density * (1.0 + pattern_strength) * fractal_depth * 10.0;
        
        // Ensure minimum value and cap at 1000
        fcs.max(1.0).min(1000.0)
    }
    
    fn calculate_mathematical_efficiency_index(&self) -> f64 {
        if self.current_block_height == 0 {
            return 0.0;
        }
        
        // Processing efficiency (simplified for mining engine)
        let throughput = 1.0; // 1 transaction per block average
        
        // Mathematical constants
        let golden_ratio = 1.618033988749; // φ (phi)
        let euler_number = std::f64::consts::E; // e
        
        // Computational complexity factor
        let complexity_factor = (self.current_block_height as f64).log10() / 10.0; // Normalize log scale
        
        // Mathematical efficiency calculation
        let mei = throughput * golden_ratio * (1.0 + complexity_factor) * euler_number * 10.0;
        
        // Ensure minimum value and cap at 1000
        mei.max(1.0).min(1000.0)
    }
    
    fn calculate_network_harmony_factor(&self) -> f64 {
        if self.current_block_height == 0 {
            return 0.0;
        }
        
        // Block production consistency
        let block_consistency = if self.current_block_height > 10 {
            let recent_blocks = std::cmp::min(self.current_block_height, 100);
            recent_blocks as f64 / 100.0
        } else {
            self.current_block_height as f64 / 10.0
        };
        
        // Transaction flow smoothness (simplified for mining engine)
        let tx_flow: f64 = 1.0 / 10.0; // 1 tx per block, normalized to expected 10 tx/block
        let transaction_smoothness = tx_flow.min(1.0);
        
        // Network synchronization (simulated)
        let sync_factor = 0.95; // 95% network sync
        
        // Harmony calculation with weighted average
        let harmony = (block_consistency * 0.4 + transaction_smoothness * 0.4 + sync_factor * 0.2) * 1000.0;
        
        // Ensure minimum value and cap at 1000
        harmony.max(1.0).min(1000.0)
    }

    pub fn get_mining_info(&self) -> MiningStats {
        let reward = self.reward_system.calculate_reward(self.current_block_height);
        let blocks_per_day = 86400.0 / 5.0; // 86400 seconds / 5 second blocks
        let total_supply = self.reward_system.calculate_total_supply(self.current_block_height);
        
        // Calculate Smart Rate based on blockchain metrics
        let smart_rate = self.calculate_smart_rate();
        
        // Estimate next difficulty
        let next_difficulty = if self.block_times.len() >= 2023 {
            self.difficulty_adjuster.calculate_new_difficulty(
                self.current_difficulty,
                &self.block_times[self.block_times.len()-2023..]
            )
        } else {
            self.current_difficulty
        };

        MiningStats {
            network_smart_rate: smart_rate,
            difficulty: self.current_difficulty,
            block_reward: reward.total_reward as f64 / 1e6, // 6 decimals
            blocks_per_day,
            total_supply_mined: total_supply as f64 / 1e6, // 6 decimals
            next_difficulty_estimate: next_difficulty,
        }
    }

    pub fn add_block(&mut self, block_time: u64) -> Result<MiningResult, String> {
        // Validate block time
        if !self.difficulty_adjuster.validate_block_time(block_time) {
            return Err("Invalid block time".to_string());
        }

        // Add block time to history
        self.block_times.push(block_time);
        
        // Update block height
        self.current_block_height += 1;
        
        // Adjust difficulty if needed
        if self.current_block_height % 2023 == 0 && self.block_times.len() >= 2023 {
            let recent_times = &self.block_times[self.block_times.len()-2023..];
            self.current_difficulty = self.difficulty_adjuster.calculate_new_difficulty(
                self.current_difficulty,
                recent_times
            );
        }

        // Calculate reward distribution
        let reward_distribution = self.reward_system.calculate_reward(self.current_block_height);
        
        // Generate block hash (simplified for demonstration)
        let block_hash = format!("{:x}", self.current_block_height);
        let nonce = self.current_block_height;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(MiningResult {
            block_hash,
            nonce,
            difficulty: self.current_difficulty,
            reward_distribution,
            timestamp,
        })
    }

    pub fn get_block_reward(&self, block_height: u64) -> RewardDistribution {
        self.reward_system.calculate_reward(block_height)
    }

    pub fn get_difficulty(&self) -> u64 {
        self.current_difficulty
    }

    pub fn get_block_height(&self) -> u64 {
        self.current_block_height
    }

    pub fn get_mining_stats(&self) -> crate::consensus::mining_rewards::MiningStats {
        self.reward_system.get_mining_stats(self.current_block_height)
    }

    pub fn estimate_mining_duration(&self, target_blocks: u64) -> f64 {
        let _target_height = self.current_block_height + target_blocks;
        let expected_time = target_blocks as f64 * 5.0; // 5 seconds per block
        expected_time / 86400.0 // Convert to days
    }

    pub fn reset_to_genesis(&mut self) {
        self.current_block_height = 0;
        self.current_difficulty = 10; // Reset to initial difficulty 10
        self.block_times.clear();
        self.last_block_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_engine_initialization() {
        let engine = MiningEngine::new();
        assert_eq!(engine.current_difficulty, 10); // Check initial difficulty 10
        assert_eq!(engine.current_block_height, 0);
    }

    #[test]
    fn test_mining_reward_calculation() {
        let engine = MiningEngine::new();
        
        let reward = engine.get_block_reward(1);
        assert_eq!(reward.total_reward, 6); // 6 FVC direct value
        assert_eq!(reward.miner_reward, 6); // 90% of 6 FVC (rounded)
        assert_eq!(reward.ecosystem_reward, 0); // 10% of 6 FVC (rounded down to 0)
    }

    #[test]
    fn test_difficulty_adjustment() {
        let mut engine = MiningEngine::new();
        
        // Simulate fast blocks
        for _ in 0..2023 {
            engine.add_block(4).unwrap(); // 4 seconds instead of 5
        }
        
        let new_difficulty = engine.current_difficulty;
        assert!(new_difficulty > 10, "Difficulty should increase with fast blocks");
    }

    #[test]
    fn test_mining_stats() {
        let engine = MiningEngine::new();
        let stats = engine.get_mining_info();
        
        assert!(stats.network_smart_rate > 0.0);
        assert!(stats.blocks_per_day > 0.0);
        assert_eq!(stats.difficulty, 10); // Check initial difficulty 10
    }
}