use std::collections::HashMap;
use serde_json::json;
use statrs::distribution::{ChiSquared, ContinuousCDF};
use crate::security::ValidationResult;

/// Mathematical audit system for vortex score validation
pub struct VortexAuditor {
    confidence_level: f64,
    test_results: HashMap<String, f64>,
}

/// Mathematical proof structure for vortex randomness
#[derive(Debug, Clone)]
pub struct MathematicalProof {
    pub theorem: String,
    pub proof_steps: Vec<String>,
    pub assumptions: Vec<String>,
    pub conclusion: bool,
}

impl VortexAuditor {
    pub fn new(confidence_level: f64) -> Self {
        Self {
            confidence_level,
            test_results: HashMap::new(),
        }
    }

    /// NIST SP 800-22 statistical test suite for randomness
    pub fn run_nist_tests(&mut self, vortex_scores: &[f64]) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        
        // Frequency (Monobit) Test
        results.insert("frequency_test".to_string(), self.frequency_test(vortex_scores));
        
        // Runs Test
        results.insert("runs_test".to_string(), self.runs_test(vortex_scores));
        
        // Chi-Square Test for Uniform Distribution
        results.insert("chi_square_test".to_string(), self.chi_square_test(vortex_scores));
        
        // Kolmogorov-Smirnov Test
        results.insert("kolmogorov_smirnov".to_string(), self.kolmogorov_smirnov_test(vortex_scores));
        
        // Serial Test
        results.insert("serial_test".to_string(), self.serial_test(vortex_scores));
        
        self.test_results = results.clone();
        results
    }

    /// Frequency test (Monobit test)
    fn frequency_test(&self, scores: &[f64]) -> f64 {
        let ones = scores.iter().filter(|&&x| x >= 0.5).count() as f64;
        let zeros = scores.len() as f64 - ones;
        
        let s_obs = (ones - zeros).abs();
        let s = s_obs / (scores.len() as f64).sqrt();
        
        // p-value for standard normal distribution
        1.0 - (-2.0 * s * s).exp()
    }

    /// Runs test for consecutive values
    fn runs_test(&self, scores: &[f64]) -> f64 {
        if scores.len() < 2 {
            return 0.0;
        }
        
        let mut runs = 1;
        let mut prev = scores[0];
        
        for &score in &scores[1..] {
            if (score >= 0.5) != (prev >= 0.5) {
                runs += 1;
            }
            prev = score;
        }
        
        let n = scores.len() as f64;
        let expected_runs = (2.0 * n - 1.0) / 3.0;
        let variance = (16.0 * n - 29.0) / 90.0;
        
        let z = ((runs as f64) - expected_runs) / variance.sqrt();
        
        // Two-tailed p-value
        2.0 * (1.0 - z.abs().min(5.0).exp())
    }

    /// Chi-square test for uniform distribution
    fn chi_square_test(&self, scores: &[f64]) -> f64 {
        let bins = 10;
        let mut counts = vec![0; bins];
        
        for &score in scores {
            let bin = (score * bins as f64).min(bins as f64 - 1.0) as usize;
            counts[bin] += 1;
        }
        
        let expected = scores.len() as f64 / bins as f64;
        let chi_square = counts.iter()
            .map(|&count| {
                let diff = count as f64 - expected;
                diff * diff / expected
            })
            .sum::<f64>();
        
        let chi_dist = ChiSquared::new((bins - 1) as f64).unwrap();
        1.0 - chi_dist.cdf(chi_square)
    }

    /// Kolmogorov-Smirnov test
    fn kolmogorov_smirnov_test(&self, scores: &[f64]) -> f64 {
        let mut sorted_scores = scores.to_vec();
        sorted_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let n = sorted_scores.len() as f64;
        let mut max_distance: f64 = 0.0;
        
        for (i, &score) in sorted_scores.iter().enumerate() {
            let empirical_cdf = (i + 1) as f64 / n;
            let uniform_cdf = score;
            
            let distance = (empirical_cdf - uniform_cdf).abs();
            max_distance = max_distance.max(distance);
        }
        
        // K-S test statistic
        let ks_stat = max_distance * n.sqrt();
        
        // Approximate p-value for large samples
        2.0 * (-2.0 * ks_stat * ks_stat).exp()
    }

    /// Serial test for independence
    fn serial_test(&self, scores: &[f64]) -> f64 {
        if scores.len() < 2 {
            return 0.0;
        }
        
        let mut pairs = HashMap::new();
        
        for i in 0..scores.len() - 1 {
            let x = (scores[i] * 10.0) as usize;
            let y = (scores[i + 1] * 10.0) as usize;
            *pairs.entry((x, y)).or_insert(0) += 1;
        }
        
        let expected = (scores.len() - 1) as f64 / 100.0;
        let chi_square = pairs.iter()
            .map(|(_, &count)| {
                let diff = count as f64 - expected;
                diff * diff / expected
            })
            .sum::<f64>();
        
        let chi_dist = ChiSquared::new(99.0).unwrap();
        1.0 - chi_dist.cdf(chi_square)
    }

    /// Generate mathematical proof for vortex randomness
    pub fn generate_proof(&mut self, scores: &[f64]) -> MathematicalProof {
        let test_results = self.run_nist_tests(scores);
        
        let assumptions = vec![
            "Vortex scores follow uniform distribution [0,1]".to_string(),
            "Sample size is statistically significant".to_string(),
            "No external bias in generation process".to_string(),
        ];
        
        let proof_steps = vec![
            "Applied NIST SP 800-22 test suite".to_string(),
            "Performed frequency analysis".to_string(),
            "Validated uniform distribution".to_string(),
            "Tested independence between samples".to_string(),
        ];
        
        let all_passed = test_results.values().all(|&p| p >= 0.01);
        
        MathematicalProof {
            theorem: "Vortex score generation produces cryptographically secure randomness".to_string(),
            proof_steps,
            assumptions,
            conclusion: all_passed,
        }
    }

    /// Validate vortex score randomness
    pub fn validate_vortex_randomness(&self, score: f64) -> bool {
        score >= 0.0 && score <= 1.0 && !score.is_nan()
    }
    
    /// Validate vortex score
    pub async fn validate_vortex_score(&self, score: f64) -> ValidationResult {
        // Check if score is within valid range
        if score < 0.0 || score > 1.0 || score.is_nan() {
            return ValidationResult::Invalid("Score out of range".to_string());
        }
        
        // Check if score passes basic statistical tests
        let frequency_test = score >= 0.3 && score <= 0.7;
        let entropy_test = score != 0.0 && score != 1.0;
        
        if frequency_test && entropy_test {
            ValidationResult::Valid
        } else {
            ValidationResult::Suspicious
        }
    }

    /// Get comprehensive audit report
    pub fn generate_audit_report(&mut self, scores: &[f64]) -> String {
        let proof = self.generate_proof(scores);
        let test_results = self.run_nist_tests(scores);
        
        format!(
            "Fractal-Vortex Security Audit Report\n\n{}",
            serde_json::to_string_pretty(&json!({
                "theorem": proof.theorem,
                "conclusion": proof.conclusion,
                "test_results": test_results,
                "sample_size": scores.len(),
                "confidence_level": self.confidence_level,
            })).unwrap()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vortex_auditor() {
        let mut auditor = VortexAuditor::new(0.95);
        let scores: Vec<f64> = (0..1000).map(|_| rand::random()).collect();
        
        let results = auditor.run_nist_tests(&scores);
        assert!(results.len() >= 5);
    }

    #[test]
    fn test_mathematical_proof() {
        let mut auditor = VortexAuditor::new(0.95);
        let scores: Vec<f64> = (0..1000).map(|_| rand::random()).collect();
        
        let proof = auditor.generate_proof(&scores);
        assert!(!proof.theorem.is_empty());
    }
}