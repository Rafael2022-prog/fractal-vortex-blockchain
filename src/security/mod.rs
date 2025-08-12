//! Comprehensive Security Module for Fractal-Vortex Chain
//! Implements mathematical audit, formal verification, chaos testing, and monitoring

pub mod audit;
pub mod verification;
pub mod chaos_testing;
pub mod monitoring;
pub mod anomaly_detection;
pub mod integration;

pub use audit::{VortexAuditor, MathematicalProof};
pub use verification::{FormalVerifier, TLAProof};
pub use chaos_testing::{ChaosTester, FractalPropertyTest};
pub use monitoring::SecurityMonitor;
pub use anomaly_detection::{AnomalyDetector, AttackPattern};
pub use integration::{SecurityFramework, SecurityConfig, ValidationResult};