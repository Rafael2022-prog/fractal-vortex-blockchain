use fractal_vortex_chain::crypto::fractal_hash::FractalHasher;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fractal_hash_basic() {
        let mut hasher = FractalHasher::new(3);
        let data = b"test data";
        let result = hasher.fractal_hash(data);
        
        assert_eq!(result.fractal_hash.len(), 32);
        assert_eq!(result.vortex_pattern.len(), 6);
        assert!(result.energy_signature > 0);
        assert_eq!(result.iteration_depth, 3);
    }

    #[test]
    fn test_fractal_hash_consistency() {
        let mut hasher = FractalHasher::new(2);
        let data = b"consistent test";
        
        let hash1 = hasher.fractal_hash(data);
        let hash2 = hasher.fractal_hash(data);
        
        assert_eq!(hash1.fractal_hash, hash2.fractal_hash);
        assert_eq!(hash1.vortex_pattern, hash2.vortex_pattern);
        assert_eq!(hash1.energy_signature, hash2.energy_signature);
    }

    #[test]
    fn test_fractal_hash_verification() {
        let mut hasher = FractalHasher::new(4);
        let data = b"verification test";
        
        let hash = hasher.fractal_hash(data);
        let verified = hasher.verify_hash(data, &hash);
        
        assert!(verified);
    }

    #[test]
    fn test_fractal_hash_different_data() {
        let mut hasher = FractalHasher::new(2);
        let data1 = b"test data 1";
        let data2 = b"test data 2";
        
        let hash1 = hasher.fractal_hash(data1);
        let hash2 = hasher.fractal_hash(data2);
        
        assert_ne!(hash1.fractal_hash, hash2.fractal_hash);
    }


}