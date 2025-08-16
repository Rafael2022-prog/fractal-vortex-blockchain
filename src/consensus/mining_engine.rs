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
    pub network_hashrate: f64,
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

    pub fn get_mining_info(&self) -> MiningStats {
        let reward = self.reward_system.calculate_reward(self.current_block_height);
        let blocks_per_day = 86400.0 / 5.0; // 86400 seconds / 5 second blocks
        let total_supply = self.reward_system.calculate_total_supply(self.current_block_height);
        
        // Estimate network hashrate based on difficulty
        let network_hashrate = (self.current_difficulty as f64 * 2.0f64.powi(32)) / 5.0;
        
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
            network_hashrate,
            difficulty: self.current_difficulty,
            block_reward: reward.total_reward as f64 / 1e9, // 9 decimals
            blocks_per_day,
            total_supply_mined: total_supply as f64 / 1e9, // 9 decimals
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
        assert_eq!(reward.total_reward, 6_250_000_000); // 6.25 FVC (9 decimals)
        assert_eq!(reward.miner_reward, 5_625_000_000); // 90% of 6.25 FVC (9 decimals)
        assert_eq!(reward.ecosystem_reward, 625_000_000); // 10% of 6.25 FVC (9 decimals)
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
        
        assert!(stats.network_hashrate > 0.0);
        assert!(stats.blocks_per_day > 0.0);
        assert_eq!(stats.difficulty, 10); // Check initial difficulty 10
    }
}