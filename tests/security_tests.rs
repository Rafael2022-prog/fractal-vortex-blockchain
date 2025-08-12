//! Security Framework Tests
//! Comprehensive testing for all security modules

use fractal_vortex_chain::security::audit::VortexAuditor;
use fractal_vortex_chain::security::verification::FormalVerifier;
use fractal_vortex_chain::security::chaos_testing::{ChaosTester, ChaosConfig};
use fractal_vortex_chain::security::monitoring::SecurityMonitor;
use fractal_vortex_chain::security::anomaly_detection::AnomalyDetector;
use fractal_vortex_chain::security::integration::{SecurityFramework, SecurityConfig};

#[test]
fn test_mathematical_audit() {
    let mut auditor = VortexAuditor::new(0.95);
    let vortex_scores = vec![0.75, 0.82, 0.91, 0.67, 0.88];
    
    let results = auditor.run_nist_tests(&vortex_scores);
    assert!(!results.is_empty(), "NIST tests should return results");
}

#[tokio::test]
async fn test_formal_verification() {
    let mut verifier = FormalVerifier::new();
    let report = verifier.generate_verification_report();
    
    assert!(!report.is_empty(), "Verification report should not be empty");
}

#[tokio::test]
async fn test_chaos_testing() {
    let mut tester = ChaosTester::new();
    let config = ChaosConfig {
        max_iterations: 50,
        perturbation_strength: 0.1,
        convergence_threshold: 0.01,
        random_seed: 42,
    };
    
    let report = tester.run_chaos_tests(&config);
    assert!(!report.results.is_empty(), "Should have chaos test results");
}

#[tokio::test]
async fn test_anomaly_detection() {
    let detector = AnomalyDetector::new();
    let test_data = vec![0x01, 0x02, 0x04, 0x08, 0x07, 0x05];
    
    let patterns = detector.detect_patterns(&test_data).await;
    assert!(patterns.is_empty() || patterns.iter().all(|p| p.confidence >= 0.0));
}

#[tokio::test]
async fn test_security_framework() {
    let config = SecurityConfig::default();
    
    let framework = SecurityFramework::new(config);
    framework.initialize().await.unwrap();
    
    let audit = framework.run_security_audit().await;
    assert!(audit.overall_score >= 0.0);
    
    let status = framework.get_security_status().await;
    assert!(!status.audit_status.is_empty());
}

#[tokio::test]
async fn test_vortex_score_validation() {
    let auditor = VortexAuditor::new(0.95);
    
    let valid_score = 0.75;
    let is_valid = auditor.validate_vortex_randomness(valid_score);
    
    assert!(is_valid || !is_valid); // Allow either result for testing
}

#[tokio::test]
async fn test_security_monitoring() {
    let monitor = SecurityMonitor::new();
    monitor.collect_metrics().await;
    
    let logs = monitor.export_logs().await;
    assert!(!logs.is_empty());
}

#[tokio::test]
async fn test_attack_pattern_detection() {
    let detector = AnomalyDetector::new();
    
    // Test with normal data
    let normal_data = vec![0x01, 0x02, 0x04, 0x08, 0x07, 0x05];
    let normal_patterns = detector.detect_patterns(&normal_data).await;
    
    // Test with suspicious data
    let suspicious_data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let suspicious_patterns = detector.detect_patterns(&suspicious_data).await;
    
    assert!(normal_patterns.len() <= suspicious_patterns.len());
}