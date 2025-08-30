use serde::{Deserialize, Serialize};

/// Mining reward system untuk FVChain dengan mekanisme halving ala Bitcoin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningRewardSystem {
    /// Reward awal per blok dalam wei (18 decimals)
    initial_reward: u64,
    /// Interval halving dalam jumlah blok
    halving_interval: u64,
    /// Maximum number of halvings before rewards become negligible
    max_halvings: u32,
    /// Persentase alokasi untuk ekosistem (10%)
    ecosystem_percentage: u8,
    /// Timestamp genesis block
    genesis_timestamp: u64,
    /// Block time in seconds
    block_time: u64,
    /// Total minable supply in wei
    total_minable_supply: u128,
}

/// Reward distribution structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    /// Miner reward
    pub miner_reward: u128,
    /// Ecosystem allocation
    pub ecosystem_reward: u128,
    /// Total reward for this block
    pub total_reward: u128,
    /// Current halving epoch
    pub halving_epoch: u32,
    /// Blocks until next halving
    pub blocks_until_halving: u64,
}

/// Mining statistics and projections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningStats {
    /// Total supply mined so far
    pub total_mined: u128,
    /// Remaining minable supply
    pub remaining_supply: u128,
    /// Current annual emission rate
    pub annual_emission: u128,
    /// Estimated years until next halving
    pub years_until_halving: f64,
    /// Total ecosystem rewards distributed
    pub total_ecosystem_rewards: u128,
}

impl MiningRewardSystem {
    /// Create new mining reward system with Bitcoin-style parameters
    pub fn new() -> Self {
        Self {
            initial_reward: 625, // 6.25 FVC (stored as 625 for 2 decimal precision)
            halving_interval: 12_614_400, // 2 years in blocks (5 second blocks)
            max_halvings: 32, // ~64 years of mining
            ecosystem_percentage: 10,
            genesis_timestamp: 1754668800, // August 9, 2025 00:00:00 UTC
            block_time: 5, // 5 seconds per block
            total_minable_supply: 2_340_585_000u128, // 2.34B FVC direct value
        }
    }

    /// Calculate reward for a given block height
    pub fn calculate_reward(&self, block_height: u64) -> RewardDistribution {
        // Calculate which halving epoch we're in
        let halving_epoch = (block_height / self.halving_interval) as u32;
        
        // If we've exceeded max halvings, no more rewards
        if halving_epoch >= self.max_halvings {
            return RewardDistribution {
                miner_reward: 0,
                ecosystem_reward: 0,
                total_reward: 0,
                halving_epoch,
                blocks_until_halving: 0,
            };
        }

        // Calculate current reward (halved for each epoch)
        let current_reward = (self.initial_reward >> halving_epoch) as u128; // Bit shift for halving
        
        // Calculate ecosystem allocation (10%)
        let ecosystem_reward = (current_reward * self.ecosystem_percentage as u128) / 100;
        let miner_reward = current_reward - ecosystem_reward;
        
        // Calculate blocks until next halving
        let blocks_until_halving = self.halving_interval - (block_height % self.halving_interval);
        
        RewardDistribution {
            miner_reward,
            ecosystem_reward,
            total_reward: current_reward,
            halving_epoch,
            blocks_until_halving,
        }
    }

    /// Menghitung total supply yang sudah ditambang hingga block tertentu
    pub fn total_mined_supply(&self, block_height: u64) -> u128 {
        let mut total = 0u128;
        let mut current_height = 1u64;
        
        while current_height <= block_height {
            let halving_count = (current_height - 1) / self.halving_interval;
            if halving_count >= 64 {
                break; // Prevent overflow
            }
            
            let reward = self.initial_reward >> halving_count;
            let blocks_in_this_era = std::cmp::min(
                self.halving_interval,
                block_height - current_height + 1
            );
            
            // Check for overflow
            if let Some(era_total) = (reward as u128).checked_mul(blocks_in_this_era as u128) {
                if let Some(new_total) = total.checked_add(era_total) {
                    total = new_total;
                } else {
                    break; // Overflow protection
                }
            } else {
                break; // Overflow protection
            }
            
            current_height += blocks_in_this_era;
        }
        
        total
    }

    /// Calculate total supply that will be mined by a given block height
    pub fn calculate_total_supply(&self, block_height: u64) -> u128 {
        let mut total_supply = 0u128;
        let mut current_height = 1u64; // Start from block 1 (after genesis)
        
        while current_height <= block_height {
            let halving_epoch = ((current_height - 1) / self.halving_interval) as u32;
            
            if halving_epoch >= self.max_halvings {
                break;
            }
            
            let reward = self.initial_reward >> halving_epoch;
            let blocks_in_this_epoch = std::cmp::min(
                self.halving_interval,
                block_height - current_height + 1
            );
            
            // Check for overflow
            if let Some(era_total) = (reward as u128).checked_mul(blocks_in_this_epoch as u128) {
                if let Some(new_total) = total_supply.checked_add(era_total) {
                    total_supply = new_total;
                } else {
                    break; // Overflow protection
                }
            } else {
                break; // Overflow protection
            }
            
            current_height += blocks_in_this_epoch;
        }
        
        total_supply
    }

    /// Get mining statistics for current state
    pub fn get_mining_stats(&self, current_block_height: u64) -> MiningStats {
        let total_mined = self.calculate_total_supply(current_block_height);
        let max_supply = self.calculate_max_minable_supply();
        let remaining_supply = max_supply.saturating_sub(total_mined);
        
        let current_reward = self.calculate_reward(current_block_height);
        let blocks_per_year = (365.25 * 24.0 * 3600.0 / self.block_time as f64) as u64;
        let annual_emission = current_reward.total_reward * blocks_per_year as u128;
        
        let years_until_halving = (current_reward.blocks_until_halving as f64 * self.block_time as f64) / (365.25 * 24.0 * 3600.0);
        
        // Calculate total ecosystem rewards (10% of all mined supply)
        let total_ecosystem_rewards = (total_mined * self.ecosystem_percentage as u128) / 100;
        
        MiningStats {
            total_mined,
            remaining_supply,
            annual_emission,
            years_until_halving,
            total_ecosystem_rewards,
        }
    }

    /// Calculate maximum minable supply (sum of all rewards across all halvings)
    pub fn calculate_max_minable_supply(&self) -> u128 {
        let mut total = 0u128;
        
        for epoch in 0..self.max_halvings {
            let reward = self.initial_reward >> epoch;
            let blocks_in_epoch = self.halving_interval;
            total += (reward as u128) * (blocks_in_epoch as u128);
        }
        
        total
    }

    /// Get halving schedule for documentation/planning
    pub fn get_halving_schedule(&self) -> Vec<HalvingEvent> {
        let mut schedule = Vec::new();
        
        for epoch in 0..self.max_halvings {
            let block_height = epoch as u64 * self.halving_interval;
            let reward = self.initial_reward >> epoch;
            let timestamp = self.genesis_timestamp + (block_height * self.block_time);
            
            // Convert to human readable date
            let years_from_genesis = (block_height * self.block_time) as f64 / (365.25 * 24.0 * 3600.0);
            
            schedule.push(HalvingEvent {
                epoch,
                block_height,
                reward_before: if epoch == 0 { self.initial_reward as u128 } else { (self.initial_reward >> (epoch - 1)) as u128 },
                reward_after: reward as u128,
                timestamp,
                years_from_genesis,
                estimated_date: format!("August 9, {}", 2025 + years_from_genesis as u32),
            });
        }
        
        schedule
    }

    /// Validate if a reward claim is correct for given block height
    pub fn validate_reward_claim(&self, block_height: u64, claimed_reward: u128) -> bool {
        let expected = self.calculate_reward(block_height);
        claimed_reward == expected.total_reward
    }

    /// Get ecosystem allocation address (should match genesis config)
    pub fn get_ecosystem_address(&self) -> &str {
        "fvc3333333333333333333333333333333333333333"
    }
}

/// Halving event information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalvingEvent {
    pub epoch: u32,
    pub block_height: u64,
    pub reward_before: u128,
    pub reward_after: u128,
    pub timestamp: u64,
    pub years_from_genesis: f64,
    pub estimated_date: String,
}

/// Utility functions for reward calculations
pub mod utils {

    
    /// Convert FVC to display format (already direct value)
    pub fn fvc_to_display(fvc: u128) -> f64 {
        fvc as f64
    }
    
    /// Convert display value to FVC (direct conversion)
    pub fn display_to_fvc(fvc: f64) -> u128 {
        fvc as u128
    }
    
    /// Format large numbers with commas
    pub fn format_number(num: u128) -> String {
        let s = num.to_string();
        let mut result = String::new();
        
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        
        result.chars().rev().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_initial_reward() {
        let system = MiningRewardSystem::new();
        let reward = system.calculate_reward(1);
        
        assert_eq!(reward.total_reward, 6); // 6 FVC direct value
        assert_eq!(reward.halving_epoch, 0);
        assert_eq!(reward.ecosystem_reward, 0); // 0.6 FVC (10%) rounded down to 0
        assert_eq!(reward.miner_reward, 6); // 5.4 FVC (90%) rounded up to 6
    }
    
    #[test]
    fn test_first_halving() {
        let system = MiningRewardSystem::new();
        let reward = system.calculate_reward(12_614_401); // First block after halving
        
        assert_eq!(reward.total_reward, 3); // 3 FVC direct value
        assert_eq!(reward.halving_epoch, 1);
    }
    
    #[test]
    fn test_max_supply_calculation() {
        let system = MiningRewardSystem::new();
        let max_supply = system.calculate_max_minable_supply();
        
        // Should be approximately 2.34 billion FVC direct value
        assert!(max_supply > 2_340_000_000);
        assert!(max_supply < 2_341_000_000);
    }
}