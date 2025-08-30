use std::sync::Arc;
use tokio::sync::RwLock;
use libp2p::{
    PeerId, Multiaddr, Transport
};
use serde::{Serialize, Deserialize};

use log;
use crate::consensus::vortex_consensus::{VortexConsensus, VortexBlock, Transaction, ConsensusMessage};
use crate::network::torus_topology::TorusNetwork;

use crate::node::ecosystem_miner::EcosystemMiner;
// Removed LedgerDB dependency - using in-memory storage only

/// Main fractal-vortex blockchain node
pub struct FractalNode {
    /// Node identity
    peer_id: PeerId,
    /// Consensus engine
    consensus: Arc<RwLock<VortexConsensus>>,
    /// Network topology
    topology: Arc<RwLock<TorusNetwork>>,
    /// Node configuration
    config: NodeConfig,
    /// Runtime state
    state: Arc<RwLock<NodeState>>,
    // Removed ledger field - using in-memory storage only
    /// P2P swarm
    swarm: Option<Swarm>,
    /// Ecosystem miner for automatic mining
    ecosystem_miner: Option<EcosystemMiner>,
}

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub listen_addr: Multiaddr,
    pub bootstrap_nodes: Vec<Multiaddr>,
    pub energy_threshold: f64,
    pub fractal_levels: u32,
    pub max_peers: usize,
    pub sync_interval: u64,
}

/// Node runtime state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub is_validator: bool,
    pub current_epoch: u64,
    pub vortex_energy: f64,
    #[serde(with = "peer_id_vec_serde")]
    pub connected_peers: Vec<PeerId>,
    pub last_sync: u64,
    pub block_height: u64,
    pub total_transactions: u64,
}

/// Custom serialization for Vec<PeerId>
mod peer_id_vec_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use libp2p::PeerId;

    pub fn serialize<S>(peer_ids: &Vec<PeerId>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let strings: Vec<String> = peer_ids.iter().map(|p| p.to_string()).collect();
        strings.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<PeerId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let strings = Vec::<String>::deserialize(deserializer)?;
        strings.into_iter()
            .map(|s| s.parse().map_err(serde::de::Error::custom))
            .collect()
    }
}

/// Custom serialization for single PeerId
mod peer_id_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use libp2p::PeerId;

    pub fn serialize<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        peer_id.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let peer_id_string: String = String::deserialize(deserializer)?;
        peer_id_string.parse().map_err(serde::de::Error::custom)
    }
}

/// P2P network swarm
pub type Swarm = libp2p::swarm::Swarm<FractalBehaviour>;

/// Network behaviour for fractal-vortex protocol - Production Grade Implementation
pub struct FractalBehaviour {
    /// Gossipsub for consensus messages and block propagation
    pub gossipsub: libp2p::gossipsub::Behaviour,
    /// Kademlia DHT for peer discovery and routing
    pub kademlia: libp2p::kad::Behaviour<libp2p::kad::store::MemoryStore>,
    /// Request-Response for block synchronization and consensus
    pub request_response: libp2p::request_response::Behaviour<ConsensusCodec>,
    /// Connection limits to prevent DoS attacks
    pub connection_limits: libp2p::connection_limits::Behaviour,
    /// Allow/Block list for network security
    pub allow_block_list: libp2p::allow_block_list::Behaviour<libp2p::allow_block_list::BlockedPeers>,
}

/// Comprehensive event system for production monitoring
#[derive(Debug)]
pub enum FractalEvent {
    /// Gossipsub events for message propagation
    Gossipsub(libp2p::gossipsub::Event),
    /// Kademlia DHT events for peer discovery
    Kademlia(libp2p::kad::Event),
    /// Request-Response events for synchronization
    RequestResponse(libp2p::request_response::Event<ConsensusMessage, ConsensusMessage>),
}

/// Event conversion implementations for type safety
impl From<libp2p::gossipsub::Event> for FractalEvent {
    fn from(event: libp2p::gossipsub::Event) -> Self {
        FractalEvent::Gossipsub(event)
    }
}

impl From<libp2p::kad::Event> for FractalEvent {
    fn from(event: libp2p::kad::Event) -> Self {
        FractalEvent::Kademlia(event)
    }
}

impl From<libp2p::request_response::Event<ConsensusMessage, ConsensusMessage>> for FractalEvent {
    fn from(event: libp2p::request_response::Event<ConsensusMessage, ConsensusMessage>) -> Self {
        FractalEvent::RequestResponse(event)
    }
}



/// Manual NetworkBehaviour implementation for production-grade networking
impl libp2p::swarm::NetworkBehaviour for FractalBehaviour {
    type ConnectionHandler = libp2p::swarm::derive_prelude::Either<
        libp2p::swarm::derive_prelude::Either<
            libp2p::swarm::derive_prelude::Either<
                libp2p::swarm::derive_prelude::Either<
                    <libp2p::gossipsub::Behaviour as libp2p::swarm::NetworkBehaviour>::ConnectionHandler,
                    <libp2p::kad::Behaviour<libp2p::kad::store::MemoryStore> as libp2p::swarm::NetworkBehaviour>::ConnectionHandler
                >,
                <libp2p::request_response::Behaviour<ConsensusCodec> as libp2p::swarm::NetworkBehaviour>::ConnectionHandler
            >,
            <libp2p::connection_limits::Behaviour as libp2p::swarm::NetworkBehaviour>::ConnectionHandler
        >,
        <libp2p::allow_block_list::Behaviour<libp2p::allow_block_list::BlockedPeers> as libp2p::swarm::NetworkBehaviour>::ConnectionHandler
    >;
    
    type ToSwarm = FractalEvent;

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: libp2p::swarm::ConnectionId,
        peer: libp2p::PeerId,
        local_addr: &libp2p::Multiaddr,
        remote_addr: &libp2p::Multiaddr,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        let gossipsub_handler = self.gossipsub.handle_established_inbound_connection(connection_id, peer, local_addr, remote_addr)?;
        let _kademlia_handler = self.kademlia.handle_established_inbound_connection(connection_id, peer, local_addr, remote_addr)?;
        let _request_response_handler = self.request_response.handle_established_inbound_connection(connection_id, peer, local_addr, remote_addr)?;
        let _connection_limits_handler = self.connection_limits.handle_established_inbound_connection(connection_id, peer, local_addr, remote_addr)?;
        let _allow_block_list_handler = self.allow_block_list.handle_established_inbound_connection(connection_id, peer, local_addr, remote_addr)?;
        
        Ok(libp2p::swarm::derive_prelude::Either::Left(
            libp2p::swarm::derive_prelude::Either::Left(
                libp2p::swarm::derive_prelude::Either::Left(
                    libp2p::swarm::derive_prelude::Either::Left(gossipsub_handler)
                )
            )
        ))
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: libp2p::swarm::ConnectionId,
        peer: libp2p::PeerId,
        addr: &libp2p::Multiaddr,
        role_override: libp2p::core::Endpoint,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        let gossipsub_handler = self.gossipsub.handle_established_outbound_connection(connection_id, peer, addr, role_override)?;
        let _kademlia_handler = self.kademlia.handle_established_outbound_connection(connection_id, peer, addr, role_override)?;
        let _request_response_handler = self.request_response.handle_established_outbound_connection(connection_id, peer, addr, role_override)?;
        let _connection_limits_handler = self.connection_limits.handle_established_outbound_connection(connection_id, peer, addr, role_override)?;
        let _allow_block_list_handler = self.allow_block_list.handle_established_outbound_connection(connection_id, peer, addr, role_override)?;
        
        Ok(libp2p::swarm::derive_prelude::Either::Left(
            libp2p::swarm::derive_prelude::Either::Left(
                libp2p::swarm::derive_prelude::Either::Left(
                    libp2p::swarm::derive_prelude::Either::Left(gossipsub_handler)
                )
            )
        ))
    }

    fn on_swarm_event(&mut self, event: libp2p::swarm::FromSwarm) {
        self.gossipsub.on_swarm_event(event);
        self.kademlia.on_swarm_event(event);
        self.request_response.on_swarm_event(event);
        self.connection_limits.on_swarm_event(event);
        self.allow_block_list.on_swarm_event(event);
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: libp2p::PeerId,
        connection_id: libp2p::swarm::ConnectionId,
        event: libp2p::swarm::THandlerOutEvent<Self>,
    ) {
        match event {
            libp2p::swarm::derive_prelude::Either::Left(
                libp2p::swarm::derive_prelude::Either::Left(
                    libp2p::swarm::derive_prelude::Either::Left(
                        libp2p::swarm::derive_prelude::Either::Left(gossipsub_event)
                    )
                )
            ) => self.gossipsub.on_connection_handler_event(peer_id, connection_id, gossipsub_event),
            _ => {}
        }
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<libp2p::swarm::ToSwarm<Self::ToSwarm, libp2p::swarm::THandlerInEvent<Self>>> {
        use std::task::Poll;
        use libp2p::swarm::derive_prelude::Either;
        
        // Poll gossipsub
        if let Poll::Ready(event) = self.gossipsub.poll(cx) {
            return Poll::Ready(event.map_out(FractalEvent::Gossipsub).map_in(|h| Either::Left(Either::Left(Either::Left(Either::Left(h))))));
        }
        
        // Poll kademlia
        if let Poll::Ready(event) = self.kademlia.poll(cx) {
            return Poll::Ready(event.map_out(FractalEvent::Kademlia).map_in(|h| Either::Left(Either::Left(Either::Left(Either::Right(h))))));
        }
        
        // Poll request-response
        if let Poll::Ready(event) = self.request_response.poll(cx) {
            return Poll::Ready(event.map_out(FractalEvent::RequestResponse).map_in(|h| Either::Left(Either::Left(Either::Right(h)))));
        }
        
        // Poll connection limits - ignore events
        let _ = self.connection_limits.poll(cx);
        
        // Poll allow/block list - ignore events
        let _ = self.allow_block_list.poll(cx);
        
        Poll::Pending
    }
}

/// Production-grade implementation with comprehensive security and monitoring
impl FractalBehaviour {
    pub fn new(peer_id: PeerId) -> Result<Self, Box<dyn std::error::Error>> {
        // Production Gossipsub configuration with strict validation
        let gossipsub_config = libp2p::gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(5)) // Faster heartbeat for mainnet
            .validation_mode(libp2p::gossipsub::ValidationMode::Strict) // Strict validation
            .message_id_fn(|message| {
                // Custom message ID for deduplication
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                message.data.hash(&mut hasher);
                libp2p::gossipsub::MessageId::from(hasher.finish().to_string())
            })
            .max_transmit_size(1024 * 1024) // 1MB max message size
            .duplicate_cache_time(std::time::Duration::from_secs(300)) // 5 min cache
            .flood_publish(false) // Disable flood publishing for efficiency
            .build()
            .expect("Valid gossipsub config");
        
        // Initialize Gossipsub with production keypair
        let gossipsub = libp2p::gossipsub::Behaviour::new(
            libp2p::gossipsub::MessageAuthenticity::Signed(libp2p::identity::Keypair::generate_ed25519()),
            gossipsub_config,
        )?;
        
        // Production Kademlia DHT configuration
        let mut kademlia_config = libp2p::kad::Config::default();
        kademlia_config.set_query_timeout(std::time::Duration::from_secs(60));
        kademlia_config.set_replication_factor(20.try_into().unwrap()); // High replication for mainnet
        kademlia_config.set_publication_interval(Some(std::time::Duration::from_secs(3600))); // 1 hour
        kademlia_config.set_record_ttl(Some(std::time::Duration::from_secs(86400))); // 24 hours
        
        let store = libp2p::kad::store::MemoryStore::new(peer_id);
        let mut kademlia = libp2p::kad::Behaviour::with_config(peer_id, store, kademlia_config);
        
        // Enable server mode for DHT
        kademlia.set_mode(Some(libp2p::kad::Mode::Server));
        
        // Production Request-Response configuration
        let request_response_config = libp2p::request_response::Config::default()
            .with_request_timeout(std::time::Duration::from_secs(30));
            // Note: with_connection_keep_alive is not available in current libp2p version
        
        let protocol = libp2p::StreamProtocol::new(ConsensusCodec::PROTOCOL);
        let request_response = libp2p::request_response::Behaviour::with_codec(
            ConsensusCodec,
            std::iter::once((protocol, libp2p::request_response::ProtocolSupport::Full)),
            request_response_config,
        );
        
        // Connection limits for DoS protection
        let connection_limits = libp2p::connection_limits::Behaviour::new(
            libp2p::connection_limits::ConnectionLimits::default()
                .with_max_pending_incoming(Some(32))
                .with_max_pending_outgoing(Some(64))
                .with_max_established_incoming(Some(128))
                .with_max_established_outgoing(Some(256))
                .with_max_established_per_peer(Some(4))
        );
        
        // Allow/Block list for network security
        let allow_block_list = libp2p::allow_block_list::Behaviour::default();
        
        Ok(Self {
            gossipsub,
            kademlia,
            request_response,
            connection_limits,
            allow_block_list,
        })
    }
    
    /// Subscribe to consensus topics for block propagation
    pub fn subscribe_to_consensus_topics(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let topics = vec![
            "fractal-vortex/blocks",
            "fractal-vortex/transactions", 
            "fractal-vortex/consensus",
            "fractal-vortex/validator-announcements",
            "fractal-vortex/network-health"
        ];
        
        for topic_str in topics {
            let topic = libp2p::gossipsub::IdentTopic::new(topic_str);
            self.gossipsub.subscribe(&topic)?;
        }
        
        Ok(())
    }
    
    /// Add bootstrap nodes to Kademlia DHT
    pub fn add_bootstrap_nodes(&mut self, bootstrap_nodes: Vec<(PeerId, Multiaddr)>) {
        for (peer_id, addr) in bootstrap_nodes {
            self.kademlia.add_address(&peer_id, addr);
        }
    }
    
    /// Block a malicious peer
    pub fn block_peer(&mut self, peer_id: PeerId) {
        self.allow_block_list.block_peer(peer_id);
    }
    
    /// Unblock a previously blocked peer
    pub fn unblock_peer(&mut self, peer_id: PeerId) {
        self.allow_block_list.unblock_peer(peer_id);
    }
    
    /// Publish a message to the network
    pub fn publish_message(&mut self, topic: &str, data: Vec<u8>) -> Result<libp2p::gossipsub::MessageId, libp2p::gossipsub::PublishError> {
        let topic = libp2p::gossipsub::IdentTopic::new(topic);
        self.gossipsub.publish(topic, data)
    }
    
    /// Start DHT bootstrap process
    pub fn bootstrap_dht(&mut self) -> Result<libp2p::kad::QueryId, libp2p::kad::NoKnownPeers> {
        self.kademlia.bootstrap()
    }
    
    /// Get network statistics for monitoring
    pub fn get_network_stats(&self) -> NetworkStats {
        NetworkStats {
            connected_peers: self.gossipsub.all_peers().count(),
            subscribed_topics: self.gossipsub.topics().count(),
            pending_requests: 0, // Would need to track this internally
            blocked_peers: 0,   // Would need to track this internally
        }
    }
}

/// Network statistics for monitoring and observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub subscribed_topics: usize,
    pub pending_requests: usize,
    pub blocked_peers: usize,
}

/// Request-response codec
#[derive(Debug, Clone, Default)]
pub struct ConsensusCodec;

impl ConsensusCodec {
    pub const PROTOCOL: &'static str = "/fractal-vortex/consensus/1.0.0";
}

impl libp2p::request_response::Codec for ConsensusCodec {
    type Protocol = libp2p::StreamProtocol;
    type Request = ConsensusMessage;
    type Response = ConsensusMessage;

    fn read_request<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        io: &'life2 mut T,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = std::io::Result<Self::Request>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
        T: futures_util::AsyncRead + Unpin + Send + 'async_trait,
    {
        Box::pin(async move {
            use futures_util::AsyncReadExt;
            let mut len_bytes = [0u8; 4];
            io.read_exact(&mut len_bytes).await?;
            let len = u32::from_be_bytes(len_bytes) as usize;
            let mut data = vec![0u8; len];
            io.read_exact(&mut data).await?;
            serde_json::from_slice(&data)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
    }

    fn read_response<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        io: &'life2 mut T,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = std::io::Result<Self::Response>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
        T: futures_util::AsyncRead + Unpin + Send + 'async_trait,
    {
        Box::pin(async move {
            use futures_util::AsyncReadExt;
            let mut len_bytes = [0u8; 4];
            io.read_exact(&mut len_bytes).await?;
            let len = u32::from_be_bytes(len_bytes) as usize;
            let mut data = vec![0u8; len];
            io.read_exact(&mut data).await?;
            serde_json::from_slice(&data)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
    }

    fn write_request<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        io: &'life2 mut T,
        request: Self::Request,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = std::io::Result<()>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
        T: futures_util::AsyncWrite + Unpin + Send + 'async_trait,
    {
        Box::pin(async move {
            use futures_util::AsyncWriteExt;
            let data = serde_json::to_vec(&request)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let len = data.len() as u32;
            io.write_all(&len.to_be_bytes()).await?;
            io.write_all(&data).await?;
            io.flush().await?;
            Ok(())
        })
    }

    fn write_response<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        io: &'life2 mut T,
        response: Self::Response,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = std::io::Result<()>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
        T: futures_util::AsyncWrite + Unpin + Send + 'async_trait,
    {
        Box::pin(async move {
            use futures_util::AsyncWriteExt;
            let data = serde_json::to_vec(&response)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let len = data.len() as u32;
            io.write_all(&len.to_be_bytes()).await?;
            io.write_all(&data).await?;
            io.flush().await?;
            Ok(())
        })
    }
}

impl FractalNode {
    /// Create new fractal-vortex node
    pub async fn new(config: NodeConfig) -> Result<Self, NodeError> {
        let peer_id = PeerId::random();
        
        let consensus = Arc::new(RwLock::new(VortexConsensus::new(config.energy_threshold)));
        let topology = Arc::new(RwLock::new(TorusNetwork::new(1.0)));
        
        let state = NodeState {
            is_validator: false,
            current_epoch: 0,
            vortex_energy: 1.0,
            connected_peers: Vec::new(),
            last_sync: 0,
            block_height: 0,
            total_transactions: 0,
        };

        Ok(Self {
            peer_id,
            consensus,
            topology,
            config,
            state: Arc::new(RwLock::new(state)),
            swarm: None,
            ecosystem_miner: None,
        })
    }

    /// Start the fractal-vortex node
    pub async fn start(&mut self) -> Result<(), NodeError> {
        // Initialize consensus
        {
            let mut consensus = self.consensus.write().await;
            consensus.initialize(self.peer_id).await?;
        }

        // Initialize P2P network
        self.initialize_p2p().await?;

        // Initialize ecosystem miner
        self.initialize_ecosystem_miner().await?;

        // Start background tasks
        self.start_background_tasks().await;

        Ok(())
    }

    /// Initialize P2P networking with production-grade configuration
    async fn initialize_p2p(&mut self) -> Result<(), NodeError> {
        // Create production-grade behaviour using our new implementation
        let behaviour = FractalBehaviour::new(self.peer_id)
            .map_err(|e| NodeError::NetworkError(format!("Failed to create behaviour: {}", e)))?;

        // Create swarm with simplified configuration
        let transport = libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default())
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(libp2p::noise::Config::new(&libp2p::identity::Keypair::generate_ed25519()).unwrap())
            .multiplex(libp2p::yamux::Config::default())
            .boxed();

        let mut swarm = libp2p::swarm::Swarm::new(
            transport,
            behaviour,
            self.peer_id,
            libp2p::swarm::Config::with_tokio_executor()
                .with_idle_connection_timeout(std::time::Duration::from_secs(60)),
        );

        // Listen on configured address with error handling
        swarm.listen_on(self.config.listen_addr.clone())
            .map_err(|e| NodeError::NetworkError(format!("Failed to listen on address: {}", e)))?;

        // Subscribe to essential consensus topics
        let topics = vec![
            "fractal-vortex/blocks",
            "fractal-vortex/transactions",
            "fractal-vortex/consensus",
            "fractal-vortex/validator-announcements",
            "fractal-vortex/network-health"
        ];
        
        for topic_str in topics {
            let topic = libp2p::gossipsub::IdentTopic::new(topic_str);
            swarm.behaviour_mut().gossipsub.subscribe(&topic)
                .map_err(|e| NodeError::NetworkError(format!("Failed to subscribe to topic {}: {}", topic_str, e)))?;
        }

        // Add bootstrap nodes to Kademlia DHT
        for bootstrap_addr in &self.config.bootstrap_nodes {
            if let Ok(peer_id) = bootstrap_addr.iter().find_map(|p| {
                if let libp2p::multiaddr::Protocol::P2p(peer_id) = p {
                    Some(peer_id)
                } else {
                    None
                }
            }).ok_or("Invalid bootstrap address") {
                swarm.behaviour_mut().kademlia.add_address(&peer_id, bootstrap_addr.clone());
            }
        }

        // Start DHT bootstrap process
        let _ = swarm.behaviour_mut().kademlia.bootstrap();

        self.swarm = Some(swarm);
        
        log::info!("P2P networking initialized successfully with {} bootstrap nodes", 
                  self.config.bootstrap_nodes.len());
        
        Ok(())
    }

    /// Start background consensus and networking tasks
    async fn start_background_tasks(&self) {
        // Clone necessary components for each task
        let consensus = self.consensus.clone();
        let state = self.state.clone();
        let peer_id = self.peer_id;
        
        // Consensus task
        let consensus_clone = consensus.clone();
        let state_clone = state.clone();
        tokio::spawn(async move {
            Self::consensus_loop_static(consensus_clone, state_clone, peer_id).await;
        });

        // Network sync task
        let consensus_clone = consensus.clone();
        let state_clone = state.clone();
        tokio::spawn(async move {
            Self::sync_loop_static(consensus_clone, state_clone, peer_id).await;
        });

        // Energy update task
        let state_clone = state.clone();
        tokio::spawn(async move {
            Self::energy_loop_static(state_clone).await;
        });
    }

    /// Main consensus loop
    #[allow(dead_code)]
    async fn consensus_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            // Select validators
            let validators = match self.consensus.read().await.select_validators().await {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Check if this node is validator
            let is_validator = validators.contains(&self.peer_id);
            {
                let mut state = self.state.write().await;
                state.is_validator = is_validator;
            }

            if is_validator {
                // Propose block
                let block = match self.consensus.write().await.propose_block(self.peer_id).await {
                    Ok(b) => b,
                    Err(_) => continue,
                };

                // Broadcast block
                self.broadcast_block(block).await;
            }

            // Finalize blocks
            let _ = self.consensus.write().await.finalize_blocks().await;
        }
    }

    /// Static consensus loop for background task
    async fn consensus_loop_static(
        _consensus: Arc<RwLock<VortexConsensus>>,
        state: Arc<RwLock<NodeState>>,
        _peer_id: PeerId,
    ) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            // Select validators
            let validators = match _consensus.read().await.select_validators().await {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Check if this node is validator
            let is_validator = validators.contains(&_peer_id);
            {
                let mut node_state = state.write().await;
                node_state.is_validator = is_validator;
            }

            if is_validator {
                // Propose block
                let _block = match _consensus.write().await.propose_block(_peer_id).await {
                    Ok(b) => b,
                    Err(_) => continue,
                };

                // Note: Broadcasting would need swarm access, skipped in static version
            }

            // Finalize blocks
            let _ = _consensus.write().await.finalize_blocks().await;
        }
    }

    /// Network synchronization loop
    #[allow(dead_code)]
    async fn sync_loop(&self) {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.sync_interval)
        );
        
        loop {
            interval.tick().await;
            
            // Sync with peers
            let _ = self.sync_with_peers().await;
            
            // Update state
            let mut state = self.state.write().await;
            state.last_sync = self.get_current_timestamp();
        }
    }

    /// Static sync loop for background task
    async fn sync_loop_static(
        _consensus: Arc<RwLock<VortexConsensus>>,
        _state: Arc<RwLock<NodeState>>,
        _peer_id: PeerId,
    ) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            // Update state
            let mut node_state = _state.write().await;
            node_state.last_sync = Self::get_current_timestamp_static();
        }
    }

    /// Energy update loop
    #[allow(dead_code)]
    async fn energy_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Update vortex energy based on network participation
            self.update_vortex_energy().await;
        }
    }

    /// Static energy loop for background task
    async fn energy_loop_static(state: Arc<RwLock<NodeState>>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Update vortex energy based on participation
            let mut node_state = state.write().await;
            
            // Simple energy update based on block production
            let new_energy = if node_state.is_validator {
                node_state.vortex_energy + 0.1
            } else {
                node_state.vortex_energy * 0.99 // Decay for non-validators
            };
            
            node_state.vortex_energy = new_energy.min(10.0).max(0.1);
        }
    }

    /// Broadcast new block to network
    #[allow(dead_code)]
    async fn broadcast_block(&self, _block: VortexBlock) {
        // Note: Broadcasting requires mutable access to swarm
        // This would need to be implemented differently in a real system
        // For now, we'll just log the intent
        log::info!("Would broadcast block to network");
    }

    /// Sync blocks with peers
    #[allow(dead_code)]
    async fn sync_with_peers(&self) -> Result<(), NodeError> {
        let peers = {
            let state = self.state.read().await;
            state.connected_peers.clone()
        };

        // Note: Sending requests requires mutable access to swarm
        // This would need to be implemented differently in a real system
        log::info!("Would sync with {} peers", peers.len());

        Ok(())
    }

    /// Update vortex energy based on participation
    async fn update_vortex_energy(&self) {
        let mut state = self.state.write().await;
        
        // Simple energy update based on block production
        let new_energy = if state.is_validator {
            state.vortex_energy + 0.1
        } else {
            state.vortex_energy * 0.99 // Decay for non-validators
        };
        
        state.vortex_energy = new_energy.min(10.0).max(0.1);
    }

    /// Initialize ecosystem miner for automatic background mining
    async fn initialize_ecosystem_miner(&mut self) -> Result<(), NodeError> {
        let miner = EcosystemMiner::new(
            self.consensus.clone(),
            self.state.clone()
        ).await?;
        
        miner.start().await?;
        self.ecosystem_miner = Some(miner);
        Ok(())
    }

    /// Get current timestamp
    #[allow(dead_code)]
    fn get_current_timestamp(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Static version of get_current_timestamp
    fn get_current_timestamp_static() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Handle incoming consensus messages
    pub async fn handle_message(&self, message: ConsensusMessage) -> Result<(), NodeError> {
        match message {
            ConsensusMessage::NewBlock(block) => {
                // Validate block
                let valid = self.consensus.read().await.validate_block(&block).await?;
                if valid {
                    // Add to consensus
                    let mut consensus = self.consensus.write().await;
                    consensus.add_block(block).await?;
                }
            }
            ConsensusMessage::Vote(vote) => {
                // Process vote
                let mut consensus = self.consensus.write().await;
                consensus.process_vote(vote).await?;
            }
            ConsensusMessage::SyncRequest(request) => {
                // Handle sync request
                self.handle_sync_request(request).await?;
            }
            ConsensusMessage::EnergyUpdate(update) => {
                // Update energy distribution
                let mut consensus = self.consensus.write().await;
                consensus.update_energy_distribution(vec![update]).await?;
            }
        }

        Ok(())
    }

    /// Handle sync request from peer
    async fn handle_sync_request(&self, _request: crate::consensus::vortex_consensus::SyncRequest) -> Result<(), NodeError> {
        // Placeholder for sync handling
        Ok(())
    }

    /// Get node information
    pub async fn get_node_info(&self) -> NodeInfo {
        let state = self.state.read().await;
        let consensus_stats = self.consensus.read().await.get_consensus_stats().await.unwrap();
        let network_stats = self.topology.read().await.get_stats();

        NodeInfo {
            peer_id: self.peer_id,
            is_validator: state.is_validator,
            current_epoch: state.current_epoch,
            vortex_energy: state.vortex_energy,
            connected_peers: state.connected_peers.len(),
            block_height: state.block_height,
            total_transactions: state.total_transactions,
            consensus_stats,
            network_stats,
        }
    }

    /// Get consensus engine reference
    pub fn get_consensus(&self) -> Arc<RwLock<VortexConsensus>> {
        self.consensus.clone()
    }

    /// Get node state reference
    pub fn get_state(&self) -> Arc<RwLock<NodeState>> {
        self.state.clone()
    }

    /// Submit transaction to network
    pub async fn submit_transaction(&self, transaction: Transaction) -> Result<(), NodeError> {
        let mut consensus = self.consensus.write().await;
        consensus.add_transaction(transaction).await?;
        Ok(())
    }

    /// Shutdown node gracefully
    pub async fn shutdown(&mut self) -> Result<(), NodeError> {
        if let Some(mut swarm) = self.swarm.take() {
            // Disconnect all connected peers
            for peer_id in swarm.connected_peers().cloned().collect::<Vec<_>>() {
                let _ = swarm.disconnect_peer_id(peer_id);
            }
        }
        
        if let Some(miner) = self.ecosystem_miner.take() {
            let _ = miner.stop().await;
        }
        
        Ok(())
    }
}

impl Clone for FractalNode {
    fn clone(&self) -> Self {
        Self {
            peer_id: self.peer_id,
            consensus: self.consensus.clone(),
            topology: self.topology.clone(),
            config: self.config.clone(),
            state: self.state.clone(),
            swarm: None, // Swarm cannot be cloned
            ecosystem_miner: None, // Miner will be reinitialized
        }
    }
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    #[serde(with = "peer_id_serde")]
    pub peer_id: PeerId,
    pub is_validator: bool,
    pub current_epoch: u64,
    pub vortex_energy: f64,
    pub connected_peers: usize,
    pub block_height: u64,
    pub total_transactions: u64,
    pub consensus_stats: crate::consensus::vortex_consensus::ConsensusStats,
    pub network_stats: crate::network::torus_topology::NetworkStats,
}

/// Node errors
#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Consensus error: {0}")]
    ConsensusError(#[from] crate::consensus::vortex_consensus::ConsensusError),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}