use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;
use serde::{Deserialize, Serialize};
use log::{info, warn};

/// Struktur untuk tracking koneksi device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConnection {
    pub device_id: String,
    pub last_heartbeat: u64, // timestamp in seconds
    pub session_token: String,
    pub wallet_address: String,
    pub is_mining: bool,
    pub connection_count: u32,
    pub last_activity: u64, // timestamp in seconds
}

/// Konfigurasi untuk auto detection
#[derive(Debug, Clone)]
pub struct AutoDetectionConfig {
    /// Timeout untuk heartbeat (default: 30 detik)
    pub heartbeat_timeout: Duration,
    /// Interval untuk checking koneksi (default: 10 detik)
    pub check_interval: Duration,
    /// Grace period sebelum menghentikan mining (default: 60 detik)
    pub grace_period: Duration,
    /// Maximum retry attempts untuk reconnection
    pub max_retry_attempts: u32,
}

impl Default for AutoDetectionConfig {
    fn default() -> Self {
        Self {
            heartbeat_timeout: Duration::from_secs(45), // Increased from 30 to 45 seconds
            check_interval: Duration::from_secs(10),
            grace_period: Duration::from_secs(90), // Increased from 60 to 90 seconds
            max_retry_attempts: 3,
        }
    }
}

/// Request untuk heartbeat
#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    pub device_id: String,
    pub session_token: String,
    pub timestamp: u64,
}

/// Response untuk heartbeat
#[derive(Debug, Serialize)]
pub struct HeartbeatResponse {
    pub success: bool,
    pub server_time: u64,
    pub mining_status: bool,
    pub message: String,
}

/// Manager untuk auto detection mining
pub struct MiningAutoDetection {
    connections: Arc<Mutex<HashMap<String, DeviceConnection>>>,
    config: AutoDetectionConfig,
    mining_callback: Option<Arc<dyn Fn(String, bool) + Send + Sync>>,
}

impl MiningAutoDetection {
    /// Membuat instance baru dari MiningAutoDetection
    pub fn new(config: AutoDetectionConfig) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            config,
            mining_callback: None,
        }
    }

    /// Set callback untuk start/stop mining
    pub fn set_mining_callback<F>(&mut self, callback: F)
    where
        F: Fn(String, bool) + Send + Sync + 'static,
    {
        self.mining_callback = Some(Arc::new(callback));
    }

    /// Register device connection
    pub async fn register_device(&self, device_id: String, session_token: String, wallet_address: String) {
        let mut connections = self.connections.lock().await;
        let connection = DeviceConnection {
            device_id: device_id.clone(),
            last_heartbeat: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            session_token,
            wallet_address,
            is_mining: false,
            connection_count: 1,
            last_activity: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        };
        
        connections.insert(device_id.clone(), connection);
        info!("Device registered for auto-detection: {}", device_id);
    }

    /// Update heartbeat untuk device
    pub async fn update_heartbeat(&self, request: HeartbeatRequest) -> HeartbeatResponse {
        let mut connections = self.connections.lock().await;
        
        if let Some(connection) = connections.get_mut(&request.device_id) {
            // Validasi session token
            if connection.session_token != request.session_token {
                warn!("Invalid session token for device: {}", request.device_id);
                return HeartbeatResponse {
                    success: false,
                    server_time: chrono::Utc::now().timestamp() as u64,
                    mining_status: false,
                    message: "Invalid session token".to_string(),
                };
            }

            // Update heartbeat
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            connection.last_heartbeat = now;
            connection.last_activity = now;
            connection.connection_count += 1;

            info!("Heartbeat updated for device: {}", request.device_id);
            
            HeartbeatResponse {
                success: true,
                server_time: chrono::Utc::now().timestamp() as u64,
                mining_status: connection.is_mining,
                message: "Heartbeat received".to_string(),
            }
        } else {
            warn!("Device not found for heartbeat: {}", request.device_id);
            HeartbeatResponse {
                success: false,
                server_time: chrono::Utc::now().timestamp() as u64,
                mining_status: false,
                message: "Device not registered".to_string(),
            }
        }
    }

    /// Start mining untuk device
    pub async fn start_mining(&self, device_id: &str) -> bool {
        let mut connections = self.connections.lock().await;
        
        if let Some(connection) = connections.get_mut(device_id) {
            connection.is_mining = true;
            connection.last_activity = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            info!("Mining started for device: {}", device_id);
            true
        } else {
            warn!("Cannot start mining - device not found: {}", device_id);
            false
        }
    }

    /// Stop mining untuk device
    pub async fn stop_mining(&self, device_id: &str) -> bool {
        let mut connections = self.connections.lock().await;
        
        if let Some(connection) = connections.get_mut(device_id) {
            connection.is_mining = false;
            connection.last_activity = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            info!("Mining stopped for device: {}", device_id);
            true
        } else {
            warn!("Cannot stop mining - device not found: {}", device_id);
            false
        }
    }

    /// Remove device dari monitoring
    pub async fn remove_device(&self, device_id: &str) {
        let mut connections = self.connections.lock().await;
        connections.remove(device_id);
        info!("Device removed from auto-detection: {}", device_id);
    }

    /// Unregister device completely (alias for remove_device for consistency)
    pub async fn unregister_device(&self, device_id: &str) {
        let mut connections = self.connections.lock().await;
        
        if let Some(connection) = connections.get(device_id) {
            // Stop mining if active
            if connection.is_mining {
                if let Some(callback) = &self.mining_callback {
                    callback(device_id.to_string(), false);
                }
            }
        }
        
        connections.remove(device_id);
        info!("Device unregistered from auto-detection: {}", device_id);
    }

    /// Get status device
    pub async fn get_device_status(&self, device_id: &str) -> Option<DeviceConnection> {
        let connections = self.connections.lock().await;
        connections.get(device_id).cloned()
    }

    /// Start monitoring loop
    pub async fn start_monitoring(&self) {
        let connections = Arc::clone(&self.connections);
        let config = self.config.clone();
        let callback = self.mining_callback.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(config.check_interval);
            
            loop {
                interval.tick().await;
                
                let mut connections_guard = connections.lock().await;
                let mut devices_to_stop = Vec::new();
                let mut devices_to_remove = Vec::new();
                
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                
                for (device_id, connection) in connections_guard.iter() {
                    let time_since_heartbeat = Duration::from_secs(now.saturating_sub(connection.last_heartbeat));
                    
                    // Check jika device sudah timeout
                    if time_since_heartbeat > config.heartbeat_timeout {
                        if connection.is_mining {
                            // Grace period sebelum stop mining
                            if time_since_heartbeat > config.grace_period {
                                devices_to_stop.push(device_id.clone());
                                warn!("Device {} timed out, stopping mining", device_id);
                            } else {
                                warn!("Device {} in grace period, mining continues", device_id);
                            }
                        } else {
                            // Jika tidak mining dan sudah timeout, remove device
                            if time_since_heartbeat > config.grace_period * 2 {
                                devices_to_remove.push(device_id.clone());
                            }
                        }
                    }
                }
                
                // Stop mining untuk devices yang timeout
                for device_id in &devices_to_stop {
                    if let Some(connection) = connections_guard.get_mut(device_id) {
                        connection.is_mining = false;
                        
                        // Call callback untuk stop mining
                        if let Some(ref callback) = callback {
                            callback(device_id.clone(), false);
                        }
                    }
                }
                
                // Remove devices yang sudah tidak aktif
                for device_id in &devices_to_remove {
                    connections_guard.remove(device_id);
                    info!("Removed inactive device: {}", device_id);
                }
                
                drop(connections_guard);
                
                if !devices_to_stop.is_empty() || !devices_to_remove.is_empty() {
                    info!("Auto-detection check completed: {} stopped, {} removed", 
                          devices_to_stop.len(), devices_to_remove.len());
                }
            }
        });
        
        info!("Mining auto-detection monitoring started");
    }

    /// Get semua active connections
    pub async fn get_active_connections(&self) -> Vec<DeviceConnection> {
        let connections = self.connections.lock().await;
        connections.values().cloned().collect()
    }

    /// Get statistics
    pub async fn get_statistics(&self) -> HashMap<String, u64> {
        let connections = self.connections.lock().await;
        let total_devices = connections.len() as u64;
        let mining_devices = connections.values().filter(|c| c.is_mining).count() as u64;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let active_devices = connections.values()
            .filter(|c| Duration::from_secs(now.saturating_sub(c.last_heartbeat)) < self.config.heartbeat_timeout)
            .count() as u64;
        
        let mut stats = HashMap::new();
        stats.insert("total_devices".to_string(), total_devices);
        stats.insert("mining_devices".to_string(), mining_devices);
        stats.insert("active_devices".to_string(), active_devices);
        stats.insert("inactive_devices".to_string(), total_devices - active_devices);
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // tokio::time::sleep import removed - not used in current tests

    #[tokio::test]
    async fn test_device_registration() {
        let detection = MiningAutoDetection::new(AutoDetectionConfig::default());
        
        detection.register_device(
            "test_device".to_string(),
            "test_session".to_string(),
            "test_wallet".to_string()
        ).await;
        
        let status = detection.get_device_status("test_device").await;
        assert!(status.is_some());
        assert_eq!(status.unwrap().device_id, "test_device");
    }

    #[tokio::test]
    async fn test_heartbeat_update() {
        let detection = MiningAutoDetection::new(AutoDetectionConfig::default());
        
        detection.register_device(
            "test_device".to_string(),
            "test_session".to_string(),
            "test_wallet".to_string()
        ).await;
        
        let request = HeartbeatRequest {
            device_id: "test_device".to_string(),
            session_token: "test_session".to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        
        let response = detection.update_heartbeat(request).await;
        assert!(response.success);
    }

    #[tokio::test]
    async fn test_mining_control() {
        let detection = MiningAutoDetection::new(AutoDetectionConfig::default());
        
        detection.register_device(
            "test_device".to_string(),
            "test_session".to_string(),
            "test_wallet".to_string()
        ).await;
        
        // Start mining
        let started = detection.start_mining("test_device").await;
        assert!(started);
        
        let status = detection.get_device_status("test_device").await;
        assert!(status.unwrap().is_mining);
        
        // Stop mining
        let stopped = detection.stop_mining("test_device").await;
        assert!(stopped);
        
        let status = detection.get_device_status("test_device").await;
        assert!(!status.unwrap().is_mining);
    }
}