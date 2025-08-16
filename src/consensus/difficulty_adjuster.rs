#[derive(Debug, Clone)]
pub struct DifficultyAdjuster {
    target_block_time: u64,
    adjustment_interval: u64,
    max_adjustment_factor: f64,
}

impl DifficultyAdjuster {
    pub fn new(target_block_time: u64, adjustment_interval: u64) -> Self {
        Self {
            target_block_time,
            adjustment_interval,
            max_adjustment_factor: 4.0, // Max 4x increase or 1/4 decrease
        }
    }

    pub fn calculate_new_difficulty(
        &self,
        current_difficulty: u64,
        actual_times: &[u64], // Actual block times in seconds
    ) -> u64 {
        if actual_times.len() < self.adjustment_interval as usize {
            return current_difficulty;
        }

        let expected_time = self.target_block_time * self.adjustment_interval;
        let actual_time: u64 = actual_times.iter().sum();

        // Calculate adjustment factor
        let mut adjustment_factor = expected_time as f64 / actual_time as f64;
        
        // Clamp adjustment factor to prevent extreme changes
        adjustment_factor = adjustment_factor.clamp(
            1.0 / self.max_adjustment_factor,
            self.max_adjustment_factor,
        );

        // Calculate new difficulty
        let new_difficulty = (current_difficulty as f64 * adjustment_factor).round() as u64;
        
        // Ensure minimum difficulty of 1
        new_difficulty.max(1)
    }

    pub fn validate_block_time(&self, block_time: u64) -> bool {
        // Allow some flexibility in block times
        block_time > 0 && block_time < self.target_block_time * 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_adjustment() {
        let adjuster = DifficultyAdjuster::new(5, 2016);
        
        // Test when blocks are too fast (difficulty should increase)
        let fast_times = vec![4; 2016];
        let new_diff = adjuster.calculate_new_difficulty(1000000, &fast_times);
        assert!(new_diff > 1000000);
        
        // Test when blocks are too slow (difficulty should decrease)
        let slow_times = vec![6; 2016];
        let new_diff = adjuster.calculate_new_difficulty(1000000, &slow_times);
        assert!(new_diff < 1000000);
        
        // Test when blocks are perfect (difficulty should stay same)
        let perfect_times = vec![5; 2016];
        let new_diff = adjuster.calculate_new_difficulty(1000000, &perfect_times);
        assert_eq!(new_diff, 1000000);
    }

    #[test]
    fn test_adjustment_limits() {
        let adjuster = DifficultyAdjuster::new(5, 2016);
        
        // Test extreme fast blocks
        let very_fast = vec![1; 2016];
        let new_diff = adjuster.calculate_new_difficulty(1000000, &very_fast);
        assert!(new_diff <= 4000000); // Max 4x increase
        
        // Test extreme slow blocks
        let very_slow = vec![20; 2016];
        let new_diff = adjuster.calculate_new_difficulty(1000000, &very_slow);
        assert!(new_diff >= 250000); // Max 1/4 decrease
    }
}