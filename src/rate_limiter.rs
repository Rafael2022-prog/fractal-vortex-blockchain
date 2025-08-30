use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};


#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_limit: u32,
    pub cleanup_interval: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 180, // Balanced for public API access
            burst_limit: 60, // Reasonable burst limit for legitimate usage
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

#[derive(Debug, Clone)]
struct ClientInfo {
    request_count: u32,
    last_request: Instant,
    first_request_in_window: Instant,
}

impl ClientInfo {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            request_count: 1,
            last_request: now,
            first_request_in_window: now,
        }
    }



    fn update(&mut self, config: &RateLimitConfig) -> bool {
        let now = Instant::now();
        let window_duration = Duration::from_secs(60);
        
        // Check if we need to reset the window
        if now.duration_since(self.first_request_in_window) >= window_duration {
            self.request_count = 1;
            self.first_request_in_window = now;
            self.last_request = now;
            return true;
        }
        
        // For dashboard endpoints, be more lenient with burst limits
        // Only check rate limit per minute, not burst limit for normal operations
        if self.request_count >= config.requests_per_minute {
            return false;
        }
        
        // Only apply burst limit if requests are coming too fast (within 1 second)
        if now.duration_since(self.last_request) < Duration::from_millis(100) {
            if self.request_count >= config.burst_limit {
                return false;
            }
        }
        
        self.request_count += 1;
        self.last_request = now;
        true
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    clients: Arc<Mutex<HashMap<IpAddr, ClientInfo>>>,
    config: RateLimitConfig,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            config,
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn check_rate_limit(&self, ip: IpAddr) -> bool {
        let mut clients = self.clients.lock().unwrap();
        
        match clients.get_mut(&ip) {
            Some(client_info) => {
                client_info.update(&self.config)
            }
            None => {
                clients.insert(ip, ClientInfo::new());
                true
            }
        }
    }

    pub fn get_rate_limit_response(&self) -> Response {
        let error_response = json!({
            "success": false,
            "error": {
                "code": "RATE_LIMIT_EXCEEDED",
                "message": "Rate limit exceeded. Please try again later.",
                "details": {
                    "requests_per_minute": self.config.requests_per_minute,
                    "burst_limit": self.config.burst_limit
                }
            }
        });

        (StatusCode::TOO_MANY_REQUESTS, Json(error_response)).into_response()
    }

    pub fn cleanup_old_entries(&self) {
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        let now = Instant::now();
        
        if now.duration_since(*last_cleanup) < self.config.cleanup_interval {
            return;
        }
        
        let mut clients = self.clients.lock().unwrap();
        let cutoff = now - Duration::from_secs(300); // Remove entries older than 5 minutes
        
        clients.retain(|_, client_info| {
            client_info.last_request > cutoff
        });
        
        *last_cleanup = now;
    }

    pub fn get_stats(&self) -> Value {
        let clients = self.clients.lock().unwrap();
        json!({
            "active_clients": clients.len(),
            "config": {
                "requests_per_minute": self.config.requests_per_minute,
                "burst_limit": self.config.burst_limit
            }
        })
    }
}

// Global rate limiter instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_RATE_LIMITER: RateLimiter = RateLimiter::new(RateLimitConfig::default());
}

// Endpoint-specific rate limiters
#[derive(Debug)]
pub struct EndpointRateLimiters {
    pub mining: RateLimiter,
    pub wallet: RateLimiter,
    pub admin: RateLimiter,
}

impl Default for EndpointRateLimiters {
    fn default() -> Self {
        Self {
            mining: RateLimiter::new(RateLimitConfig {
                requests_per_minute: 60, // Reduced for mining operations
                burst_limit: 15, // Lower burst for mining
                cleanup_interval: Duration::from_secs(300),
            }),
            wallet: RateLimiter::new(RateLimitConfig {
                requests_per_minute: 120, // Moderate for wallet operations
                burst_limit: 25, // Reasonable burst for wallet
                cleanup_interval: Duration::from_secs(300),
            }),
            admin: RateLimiter::new(RateLimitConfig {
                requests_per_minute: 5, // Very strict for admin
                burst_limit: 1, // Minimal burst for admin
                cleanup_interval: Duration::from_secs(300),
            }),
        }
    }
}

impl EndpointRateLimiters {
    pub fn get_all_stats(&self) -> Value {
        json!({
            "mining": self.mining.get_stats(),
            "wallet": self.wallet.get_stats(),
            "admin": self.admin.get_stats()
        })
    }
}

lazy_static::lazy_static! {
    pub static ref ENDPOINT_RATE_LIMITERS: EndpointRateLimiters = EndpointRateLimiters::default();
}

// Rate limiting middleware
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let ip = addr.ip();
    
    // Perform cleanup periodically
    GLOBAL_RATE_LIMITER.cleanup_old_entries();
    
    // Check rate limit
    if !GLOBAL_RATE_LIMITER.check_rate_limit(ip) {
        return GLOBAL_RATE_LIMITER.get_rate_limit_response();
    }
    
    // Check endpoint-specific rate limits
    let path = request.uri().path();
    let endpoint_limiter = match path {
        p if p.contains("/mining/") || p.contains("/miner/") => Some(&ENDPOINT_RATE_LIMITERS.mining),
        p if p.contains("/wallet/") || p.contains("/device/") => Some(&ENDPOINT_RATE_LIMITERS.wallet),
        p if p.contains("/admin/") => Some(&ENDPOINT_RATE_LIMITERS.admin),
        _ => None,
    };
    
    if let Some(limiter) = endpoint_limiter {
        if !limiter.check_rate_limit(ip) {
            return limiter.get_rate_limit_response();
        }
    }
    
    next.run(request).await
}

// Helper function to get client IP from request
pub fn get_client_ip(request: &Request<axum::body::Body>) -> Option<IpAddr> {
    // Try to get IP from X-Forwarded-For header (for reverse proxy setups)
    if let Some(forwarded_for) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }
    
    // Try to get IP from X-Real-IP header
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_rate_limiter_basic() {
        let config = RateLimitConfig {
            requests_per_minute: 5,
            burst_limit: 3,
            cleanup_interval: Duration::from_secs(60),
        };
        let limiter = RateLimiter::new(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        // First few requests should pass
        assert!(limiter.check_rate_limit(ip));
        assert!(limiter.check_rate_limit(ip));
        assert!(limiter.check_rate_limit(ip));
        
        // Should hit burst limit
        assert!(!limiter.check_rate_limit(ip));
    }

    #[test]
    fn test_client_info_window_reset() {
        let config = RateLimitConfig::default();
        let mut client = ClientInfo::new();
        
        // Simulate old timestamp
        client.first_request_in_window = Instant::now() - Duration::from_secs(70);
        
        // Should allow request in new window
        assert!(client.update(&config));
        assert_eq!(client.request_count, 1);
    }
}