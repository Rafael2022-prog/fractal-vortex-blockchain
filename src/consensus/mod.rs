//! Fractal-Vortex consensus algorithms

pub mod fractal_vortex;
pub mod vortex_consensus;
pub mod mining_rewards;
pub mod difficulty_adjuster;
pub mod mining_engine;

pub use fractal_vortex::{FractalVortexConsensus, Block, FractalTopology, VortexMath};
pub use vortex_consensus::{VortexConsensus, VortexBlock, Transaction, ConsensusStats};
pub use mining_rewards::{MiningRewardSystem, RewardDistribution, MiningStats, HalvingEvent};
pub use difficulty_adjuster::DifficultyAdjuster;
pub use mining_engine::{MiningEngine, MiningResult};