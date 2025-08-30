use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::time::interval;
use lazy_static::lazy_static;

// API metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    pub endpoint: String,
    pub method: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub timestamp: u64,
    pub client_ip: String,
    pub user_agent: Option<String>,
    pub api_key_used: Option<String>,
}

// Aggregated statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStats {
    pub endpoint: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub requests_per_minute: f64,
    pub error_rate: f64,
    pub last_request: u64,
}

// Security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    RateLimitExceeded {
        client_ip: String,
        endpoint: String,
        attempts: u32,
    },
    InvalidApiKey {
        client_ip: String,
        endpoint: String,
        provided_key: String,
    },
    SuspiciousActivity {
        client_ip: String,
        reason: String,
        details: HashMap<String, String>,
    },
    UnauthorizedAccess {
        client_ip: String,
        endpoint: String,
        user_agent: Option<String>,
    },
}

// API Monitor structure
pub struct ApiMonitor {
    metrics: Arc<Mutex<Vec<ApiMetrics>>>,
    endpoint_stats: Arc<Mutex<HashMap<String, EndpointStats>>>,
    security_events: Arc<Mutex<Vec<SecurityEvent>>>,
    start_time: Instant,
}

impl ApiMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            endpoint_stats: Arc::new(Mutex::new(HashMap::new())),
            security_events: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
        }
    }
    
    // Record API request metrics
    pub fn record_request(
        &self,
        endpoint: String,
        method: String,
        status_code: u16,
        response_time: Duration,
        client_ip: String,
        user_agent: Option<String>,
        api_key_used: Option<String>,
    ) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let metric = ApiMetrics {
            endpoint: endpoint.clone(),
            method,
            status_code,
            response_time_ms: response_time.as_millis() as u64,
            timestamp,
            client_ip,
            user_agent,
            api_key_used,
        };
        
        // Store metric
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.push(metric.clone());
            
            // Keep only last 10000 metrics to prevent memory issues
            if metrics.len() > 10000 {
                metrics.drain(0..1000);
            }
        }
        
        // Update endpoint statistics
        self.update_endpoint_stats(&metric);
        
        // Log important events
        if status_code >= 400 {
            println!(
                "[API_ERROR] {} {} - Status: {} - Response Time: {}ms - IP: {}",
                metric.method,
                metric.endpoint,
                metric.status_code,
                metric.response_time_ms,
                metric.client_ip
            );
        }
    }
    
    // Update endpoint statistics
    fn update_endpoint_stats(&self, metric: &ApiMetrics) {
        let mut stats = self.endpoint_stats.lock().unwrap();
        let endpoint_stat = stats.entry(metric.endpoint.clone()).or_insert(EndpointStats {
            endpoint: metric.endpoint.clone(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            min_response_time_ms: u64::MAX,
            max_response_time_ms: 0,
            requests_per_minute: 0.0,
            error_rate: 0.0,
            last_request: metric.timestamp,
        });
        
        endpoint_stat.total_requests += 1;
        
        if metric.status_code < 400 {
            endpoint_stat.successful_requests += 1;
        } else {
            endpoint_stat.failed_requests += 1;
        }
        
        // Update response time statistics
        endpoint_stat.min_response_time_ms = endpoint_stat.min_response_time_ms.min(metric.response_time_ms);
        endpoint_stat.max_response_time_ms = endpoint_stat.max_response_time_ms.max(metric.response_time_ms);
        
        // Calculate average response time
        endpoint_stat.avg_response_time_ms = (
            endpoint_stat.avg_response_time_ms * (endpoint_stat.total_requests - 1) as f64 +
            metric.response_time_ms as f64
        ) / endpoint_stat.total_requests as f64;
        
        // Calculate error rate
        endpoint_stat.error_rate = (endpoint_stat.failed_requests as f64 / endpoint_stat.total_requests as f64) * 100.0;
        
        endpoint_stat.last_request = metric.timestamp;
    }
    
    // Record security event
    pub fn record_security_event(&self, event: SecurityEvent) {
        let mut events = self.security_events.lock().unwrap();
        
        // Log security event
        match &event {
            SecurityEvent::RateLimitExceeded { client_ip, endpoint, attempts } => {
                println!(
                    "[SECURITY_ALERT] Rate limit exceeded - IP: {} - Endpoint: {} - Attempts: {}",
                    client_ip, endpoint, attempts
                );
            }
            SecurityEvent::InvalidApiKey { client_ip, endpoint, provided_key } => {
                println!(
                    "[SECURITY_ALERT] Invalid API key - IP: {} - Endpoint: {} - Key: {}...",
                    client_ip, endpoint, &provided_key[..8.min(provided_key.len())]
                );
            }
            SecurityEvent::SuspiciousActivity { client_ip, reason, .. } => {
                println!(
                    "[SECURITY_ALERT] Suspicious activity - IP: {} - Reason: {}",
                    client_ip, reason
                );
            }
            SecurityEvent::UnauthorizedAccess { client_ip, endpoint, .. } => {
                println!(
                    "[SECURITY_ALERT] Unauthorized access - IP: {} - Endpoint: {}",
                    client_ip, endpoint
                );
            }
        }
        
        events.push(event);
        
        // Keep only last 1000 security events
        if events.len() > 1000 {
            events.drain(0..100);
        }
    }
    
    // Get endpoint statistics
    pub fn get_endpoint_stats(&self) -> HashMap<String, EndpointStats> {
        let stats = self.endpoint_stats.lock().unwrap();
        let _current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut result = HashMap::new();
        
        for (endpoint, stat) in stats.iter() {
            let mut updated_stat = stat.clone();
            
            // Calculate requests per minute based on uptime
            let uptime_minutes = self.start_time.elapsed().as_secs() as f64 / 60.0;
            if uptime_minutes > 0.0 {
                updated_stat.requests_per_minute = stat.total_requests as f64 / uptime_minutes;
            }
            
            result.insert(endpoint.clone(), updated_stat);
        }
        
        result
    }
    
    // Get recent security events
    pub fn get_recent_security_events(&self, limit: usize) -> Vec<SecurityEvent> {
        let events = self.security_events.lock().unwrap();
        events.iter().rev().take(limit).cloned().collect()
    }
    
    // Get system health status
    pub fn get_health_status(&self) -> serde_json::Value {
        let stats = self.get_endpoint_stats();
        let total_requests: u64 = stats.values().map(|s| s.total_requests).sum();
        let total_errors: u64 = stats.values().map(|s| s.failed_requests).sum();
        let avg_response_time: f64 = if !stats.is_empty() {
            stats.values().map(|s| s.avg_response_time_ms).sum::<f64>() / stats.len() as f64
        } else {
            0.0
        };
        
        let uptime_seconds = self.start_time.elapsed().as_secs();
        let error_rate = if total_requests > 0 {
            (total_errors as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        serde_json::json!({
            "status": if error_rate < 5.0 && avg_response_time < 1000.0 { "healthy" } else { "degraded" },
            "uptime_seconds": uptime_seconds,
            "total_requests": total_requests,
            "total_errors": total_errors,
            "error_rate_percent": error_rate,
            "avg_response_time_ms": avg_response_time,
            "active_endpoints": stats.len(),
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        })
    }
    
    // Start background monitoring tasks
    pub fn start_background_tasks(&self) {
        let stats = Arc::clone(&self.endpoint_stats);
        let events = Arc::clone(&self.security_events);
        
        // Periodic statistics logging
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes
            
            loop {
                interval.tick().await;
                
                let stats_guard = stats.lock().unwrap();
                let events_guard = events.lock().unwrap();
                
                println!("[API_STATS] Active endpoints: {}, Recent security events: {}", 
                    stats_guard.len(), events_guard.len());
                
                // Log top endpoints by request count
                let mut sorted_stats: Vec<_> = stats_guard.values().collect();
                sorted_stats.sort_by(|a, b| b.total_requests.cmp(&a.total_requests));
                
                for (i, stat) in sorted_stats.iter().take(5).enumerate() {
                    println!(
                        "[API_STATS] Top {} - {}: {} requests, {:.1}% error rate, {:.1}ms avg response",
                        i + 1,
                        stat.endpoint,
                        stat.total_requests,
                        stat.error_rate,
                        stat.avg_response_time_ms
                    );
                }
                
                drop(stats_guard);
                drop(events_guard);
            }
        });
    }
}

// Global API monitor instance
lazy_static! {
    pub static ref GLOBAL_API_MONITOR: ApiMonitor = ApiMonitor::new();
}

// Helper function to get client IP from request
pub fn extract_client_ip(headers: &axum::http::HeaderMap) -> String {
    // Check for forwarded headers first (behind proxy)
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    "unknown".to_string()
}

// Helper function to extract user agent
pub fn extract_user_agent(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_monitor_creation() {
        let monitor = ApiMonitor::new();
        let stats = monitor.get_endpoint_stats();
        assert!(stats.is_empty());
    }
    
    #[test]
    fn test_record_request() {
        let monitor = ApiMonitor::new();
        
        monitor.record_request(
            "/api/test".to_string(),
            "GET".to_string(),
            200,
            Duration::from_millis(100),
            "127.0.0.1".to_string(),
            Some("test-agent".to_string()),
            None,
        );
        
        let stats = monitor.get_endpoint_stats();
        assert_eq!(stats.len(), 1);
        assert!(stats.contains_key("/api/test"));
        
        let endpoint_stat = &stats["/api/test"];
        assert_eq!(endpoint_stat.total_requests, 1);
        assert_eq!(endpoint_stat.successful_requests, 1);
        assert_eq!(endpoint_stat.failed_requests, 0);
    }
    
    #[test]
    fn test_security_event_recording() {
        let monitor = ApiMonitor::new();
        
        let event = SecurityEvent::RateLimitExceeded {
            client_ip: "127.0.0.1".to_string(),
            endpoint: "/api/test".to_string(),
            attempts: 5,
        };
        
        monitor.record_security_event(event);
        
        let events = monitor.get_recent_security_events(10);
        assert_eq!(events.len(), 1);
    }
}