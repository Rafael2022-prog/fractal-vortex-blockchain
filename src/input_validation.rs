use regex::Regex;
use serde_json::Value;


// Input validation errors
#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidFormat(String),
    InvalidLength(String),
    InvalidRange(String),
    InvalidCharacters(String),
    Required(String),
    InvalidType(String),
}

impl ValidationError {
    pub fn to_string(&self) -> String {
        match self {
            ValidationError::InvalidFormat(msg) => format!("Invalid format: {}", msg),
            ValidationError::InvalidLength(msg) => format!("Invalid length: {}", msg),
            ValidationError::InvalidRange(msg) => format!("Invalid range: {}", msg),
            ValidationError::InvalidCharacters(msg) => format!("Invalid characters: {}", msg),
            ValidationError::Required(msg) => format!("Required field: {}", msg),
            ValidationError::InvalidType(msg) => format!("Invalid type: {}", msg),
        }
    }
}

// Input validator
pub struct InputValidator;

impl InputValidator {
    // Validate FVChain address format (NATIVE FORMAT: fvc + 36 hex + emyl)
    pub fn validate_fvchain_address(address: &str) -> Result<(), ValidationError> {
        if address.is_empty() {
            return Err(ValidationError::Required("Address is required".to_string()));
        }
        
        if !address.starts_with("fvc") {
            return Err(ValidationError::InvalidFormat(
                "Address must start with 'fvc'".to_string()
            ));
        }
        
        if !address.ends_with("emyl") {
            return Err(ValidationError::InvalidFormat(
                "Address must end with 'emyl'".to_string()
            ));
        }
        
        if address.len() != 43 {
            return Err(ValidationError::InvalidLength(
                "Address must be exactly 43 characters (fvc + 36 hex + emyl)".to_string()
            ));
        }
        
        // Extract hex part (remove fvc prefix and emyl suffix)
        let hex_part = &address[3..address.len()-4];
        if hex_part.len() != 36 {
            return Err(ValidationError::InvalidLength(
                "Hex part must be exactly 36 characters".to_string()
            ));
        }
        
        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidCharacters(
                "Address hex part can only contain hexadecimal characters (0-9, a-f, A-F)".to_string()
            ));
        }
        
        Ok(())
    }
    
    // Validate transaction hash format
    pub fn validate_transaction_hash(hash: &str) -> Result<(), ValidationError> {
        if hash.is_empty() {
            return Err(ValidationError::Required("Transaction hash is required".to_string()));
        }
        
        if hash.len() != 64 {
            return Err(ValidationError::InvalidLength(
                "Transaction hash must be exactly 64 characters".to_string()
            ));
        }
        
        let hash_part = hash;
        if !hash_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidCharacters(
                "Transaction hash can only contain hexadecimal characters".to_string()
            ));
        }
        
        Ok(())
    }
    
    // Validate device ID format
    pub fn validate_device_id(device_id: &str) -> Result<(), ValidationError> {
        if device_id.is_empty() {
            return Err(ValidationError::Required("Device ID is required".to_string()));
        }
        
        if device_id.len() < 8 || device_id.len() > 64 {
            return Err(ValidationError::InvalidLength(
                "Device ID must be between 8 and 64 characters".to_string()
            ));
        }
        
        let regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
        if !regex.is_match(device_id) {
            return Err(ValidationError::InvalidCharacters(
                "Device ID can only contain alphanumeric characters, underscores, and hyphens".to_string()
            ));
        }
        
        Ok(())
    }
    
    // Validate amount (must be positive and within reasonable range)
    pub fn validate_amount(amount: u64) -> Result<(), ValidationError> {
        if amount == 0 {
            return Err(ValidationError::InvalidRange(
                "Amount must be greater than 0".to_string()
            ));
        }
        
        // Maximum amount: 1 billion FVC (in smallest units)
        const MAX_AMOUNT: u64 = 1_000_000_000_000_000_000; // 1B FVC
        if amount > MAX_AMOUNT {
            return Err(ValidationError::InvalidRange(
                "Amount exceeds maximum allowed value".to_string()
            ));
        }
        
        Ok(())
    }
    
    // Validate block height
    pub fn validate_block_height(height: u64) -> Result<(), ValidationError> {
        // Maximum reasonable block height (for safety)
        const MAX_BLOCK_HEIGHT: u64 = 100_000_000;
        if height > MAX_BLOCK_HEIGHT {
            return Err(ValidationError::InvalidRange(
                "Block height exceeds maximum allowed value".to_string()
            ));
        }
        
        Ok(())
    }
    
    // Validate limit parameter for pagination
    pub fn validate_limit(limit: Option<usize>) -> Result<usize, ValidationError> {
        match limit {
            Some(l) => {
                if l == 0 {
                    return Err(ValidationError::InvalidRange(
                        "Limit must be greater than 0".to_string()
                    ));
                }
                if l > 1000 {
                    return Err(ValidationError::InvalidRange(
                        "Limit cannot exceed 1000".to_string()
                    ));
                }
                Ok(l)
            }
            None => Ok(50), // Default limit
        }
    }
    
    // Validate PIN hash format
    pub fn validate_pin_hash(pin_hash: &str) -> Result<(), ValidationError> {
        if pin_hash.is_empty() {
            return Err(ValidationError::Required("PIN hash is required".to_string()));
        }
        
        if pin_hash.len() != 64 {
            return Err(ValidationError::InvalidLength(
                "PIN hash must be exactly 64 characters (SHA-256)".to_string()
            ));
        }
        
        if !pin_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidCharacters(
                "PIN hash must be hexadecimal".to_string()
            ));
        }
        
        Ok(())
    }
    
    // Validate private key format
    pub fn validate_private_key(private_key: &str) -> Result<(), ValidationError> {
        if private_key.is_empty() {
            return Err(ValidationError::Required("Private key is required".to_string()));
        }
        
        // Check if it's encrypted (base64) or raw hex
        if private_key.len() == 64 {
            // Raw hex private key
            if !private_key.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(ValidationError::InvalidCharacters(
                    "Private key must be hexadecimal".to_string()
                ));
            }
        } else {
            // Encrypted private key (base64)
            let regex = Regex::new(r"^[A-Za-z0-9+/]+=*$").unwrap();
            if !regex.is_match(private_key) {
                return Err(ValidationError::InvalidFormat(
                    "Encrypted private key must be valid base64".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    // Validate JSON payload structure
    pub fn validate_json_structure(value: &Value, required_fields: &[&str]) -> Result<(), ValidationError> {
        if !value.is_object() {
            return Err(ValidationError::InvalidType(
                "Payload must be a JSON object".to_string()
            ));
        }
        
        let obj = value.as_object().unwrap();
        
        for field in required_fields {
            if !obj.contains_key(*field) {
                return Err(ValidationError::Required(
                    format!("Field '{}' is required", field)
                ));
            }
        }
        
        Ok(())
    }
    
    // Sanitize string input (remove potentially dangerous characters)
    pub fn sanitize_string(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || " -_.,@".contains(*c))
            .collect::<String>()
            .trim()
            .to_string()
    }
    
    // Validate and sanitize user input for logging
    pub fn sanitize_for_logging(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_ascii() && !c.is_control())
            .take(100) // Limit length for logging
            .collect()
    }
}

// Validation result helper
pub type ValidationResult<T> = Result<T, Vec<ValidationError>>;

// Batch validation helper
pub struct BatchValidator {
    errors: Vec<ValidationError>,
}

impl BatchValidator {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }
    
    pub fn validate<F>(&mut self, validation_fn: F) -> &mut Self
    where
        F: FnOnce() -> Result<(), ValidationError>,
    {
        if let Err(error) = validation_fn() {
            self.errors.push(error);
        }
        self
    }
    
    pub fn finish<T>(self, success_value: T) -> ValidationResult<T> {
        if self.errors.is_empty() {
            Ok(success_value)
        } else {
            Err(self.errors)
        }
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_fvchain_address() {
        // Valid address (fvc + 36 hex + emyl = 43 characters)
        assert!(InputValidator::validate_fvchain_address("fvc1234567890abcdef1234567890abcdef123456emyl").is_ok());
        
        // Invalid prefix
        assert!(InputValidator::validate_fvchain_address("btc1234567890abcdef1234567890abcdef123456emyl").is_err());
        
        // Invalid suffix
        assert!(InputValidator::validate_fvchain_address("fvc1234567890abcdef1234567890abcdef123456xyz").is_err());
        
        // Invalid length (too short)
        assert!(InputValidator::validate_fvchain_address("fvc123emyl").is_err());
        
        // Invalid length (too long)
        assert!(InputValidator::validate_fvchain_address("fvc1234567890abcdef1234567890abcdef1234567890emyl").is_err());
        
        // Invalid hex characters
        assert!(InputValidator::validate_fvchain_address("fvcGHIJKL7890abcdef1234567890abcdef123456emyl").is_err());
        
        // Invalid characters
        assert!(InputValidator::validate_fvchain_address("fvc1234567890ABCDEF1234567890abcdef123456").is_err());
    }
    
    #[test]
    fn test_validate_transaction_hash() {
        // Valid hash (64 hex characters)
        assert!(InputValidator::validate_transaction_hash("123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234").is_ok());
        
        // Invalid length (too short)
        assert!(InputValidator::validate_transaction_hash("123456789abcdef").is_err());
        
        // Invalid length (too long)
        assert!(InputValidator::validate_transaction_hash("123456789abcdef123456789abcdef123456789abcdef123456789abcdef12345").is_err());
        
        // Invalid characters (non-hex)
        assert!(InputValidator::validate_transaction_hash("123456789abcdef123456789abcdef123456789abcdef123456789abcdefghij").is_err());
        
        // Empty string
        assert!(InputValidator::validate_transaction_hash("").is_err());
    }
    
    #[test]
    fn test_validate_amount() {
        // Valid amount
        assert!(InputValidator::validate_amount(1000).is_ok());
        
        // Zero amount
        assert!(InputValidator::validate_amount(0).is_err());
        
        // Too large amount
        assert!(InputValidator::validate_amount(u64::MAX).is_err());
    }
}