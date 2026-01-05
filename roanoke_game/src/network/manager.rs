//! Network Manager - Unified interface for multiplayer
//!
//! Handles both host and client modes with a simple API.

use crate::network::client::{ConnectionState, NetworkClient};
use crate::network::messages::*;
use crate::network::server::NetworkServer;
use glam::Vec3;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

/// Network mode
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkMode {
    /// Single player (no networking)
    Offline,
    /// Hosting a game
    Host { port: u16 },
    /// Connected to a host
    Client { address: String },
}

/// Remote player data for rendering
#[derive(Debug, Clone)]
pub struct RemotePlayer {
    pub id: PlayerId,
    pub name: String,
    pub color: [f32; 3],
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    /// Interpolation target position
    pub target_position: Vec3,
    /// Last update time
    pub last_update: f64,
}

impl RemotePlayer {
    /// Interpolate position for smooth rendering
    pub fn interpolate(&mut self, dt: f32) {
        let lerp_speed = 10.0;
        self.position = self.position.lerp(self.target_position, (lerp_speed * dt).min(1.0));
    }
}

/// Main network manager
pub struct NetworkManager {
    mode: NetworkMode,
    runtime: Option<Runtime>,
    server: Option<Arc<RwLock<NetworkServer>>>,
    client: Option<Arc<RwLock<NetworkClient>>>,
    remote_players: HashMap<PlayerId, RemotePlayer>,
    my_id: Option<PlayerId>,
    seed: u32,
    last_sync_time: f64,
    chat_messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub message: String,
    pub timestamp: f64,
}

impl NetworkManager {
    /// Create offline (single-player) manager
    pub fn offline(seed: u32) -> Self {
        Self {
            mode: NetworkMode::Offline,
            runtime: None,
            server: None,
            client: None,
            remote_players: HashMap::new(),
            my_id: None,
            seed,
            last_sync_time: 0.0,
            chat_messages: Vec::new(),
        }
    }

    /// Create and start as host
    pub fn host(port: u16, seed: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let runtime = Runtime::new()?;

        let server = Arc::new(RwLock::new(NetworkServer::new(port, seed)));

        // Start server
        {
            let server_clone = server.clone();
            runtime.block_on(async {
                let mut s = server_clone.write().await;
                s.start().await.map_err(|e| e.to_string())
            }).map_err(|e: String| -> Box<dyn std::error::Error> { e.into() })?;
        }

        let my_id = PlayerId::new_v4();

        log::info!("Started hosting on port {} with seed {}", port, seed);

        Ok(Self {
            mode: NetworkMode::Host { port },
            runtime: Some(runtime),
            server: Some(server),
            client: None,
            remote_players: HashMap::new(),
            my_id: Some(my_id),
            seed,
            last_sync_time: 0.0,
            chat_messages: Vec::new(),
        })
    }

    /// Create and connect as client
    pub fn join(address: &str, player_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let runtime = Runtime::new()?;

        let client = Arc::new(RwLock::new(NetworkClient::new(player_name.to_string())));

        // Connect to server
        let seed = {
            let client_clone = client.clone();
            let addr = address.to_string();
            let result: Result<u32, String> = runtime.block_on(async {
                let mut c = client_clone.write().await;
                c.connect(&addr).await.map_err(|e| e.to_string())?;

                // Wait for connection
                let mut attempts = 0;
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    let state = c.connection_state().await;
                    match state {
                        ConnectionState::Connected => break,
                        ConnectionState::Failed(reason) => {
                            return Err(format!("Connection failed: {}", reason));
                        }
                        _ => {
                            attempts += 1;
                            if attempts > 50 {
                                return Err("Connection timeout".to_string());
                            }
                        }
                    }
                }

                // Get seed from server
                Ok(c.server_seed().await.unwrap_or(42))
            });
            result.map_err(|e: String| -> Box<dyn std::error::Error> { e.into() })?
        };

        let my_id = runtime.block_on(async {
            client.read().await.my_id().await
        });

        log::info!("Connected to {} with seed {}", address, seed);

        Ok(Self {
            mode: NetworkMode::Client { address: address.to_string() },
            runtime: Some(runtime),
            server: None,
            client: Some(client),
            remote_players: HashMap::new(),
            my_id,
            seed,
            last_sync_time: 0.0,
            chat_messages: Vec::new(),
        })
    }

    /// Get the world seed (from server if client)
    pub fn seed(&self) -> u32 {
        self.seed
    }

    /// Get our player ID
    pub fn my_id(&self) -> Option<PlayerId> {
        self.my_id
    }

    /// Check if we're the host
    pub fn is_host(&self) -> bool {
        matches!(self.mode, NetworkMode::Host { .. })
    }

    /// Check if we're online (host or client)
    pub fn is_online(&self) -> bool {
        !matches!(self.mode, NetworkMode::Offline)
    }

    /// Get network mode
    pub fn mode(&self) -> &NetworkMode {
        &self.mode
    }

    /// Get connected player count (including self)
    pub fn player_count(&self) -> usize {
        if self.is_online() {
            self.remote_players.len() + 1
        } else {
            1
        }
    }

    /// Update - call every frame
    pub fn update(&mut self, dt: f32, game_time: f64) {
        if !self.is_online() {
            return;
        }

        // Process incoming messages
        self.process_messages();

        // Interpolate remote players
        for player in self.remote_players.values_mut() {
            player.interpolate(dt);
        }
    }

    /// Process incoming network messages
    fn process_messages(&mut self) {
        // Collect client messages first
        let client_messages: Vec<NetMessage> = if let (Some(runtime), Some(client)) = (&self.runtime, &self.client) {
            runtime.block_on(async {
                let mut c = client.write().await;
                let mut messages = Vec::new();
                if let Some(rx) = &mut c.incoming_rx {
                    while let Ok(msg) = rx.try_recv() {
                        messages.push(msg);
                    }
                }
                messages
            })
        } else {
            Vec::new()
        };

        // Process client messages
        for msg in client_messages {
            self.handle_message(msg);
        }

        // Collect server messages and states
        let (server_messages, server_states): (Vec<(PlayerId, NetMessage)>, HashMap<PlayerId, PlayerSyncData>) =
            if let (Some(runtime), Some(server)) = (&self.runtime, &self.server) {
                let msgs = runtime.block_on(async {
                    let mut s = server.write().await;
                    let mut messages = Vec::new();
                    if let Some(rx) = &mut s.incoming_rx {
                        while let Ok(msg) = rx.try_recv() {
                            messages.push(msg);
                        }
                    }
                    messages
                });

                let states = runtime.block_on(async {
                    server.read().await.get_player_states().await
                });

                (msgs, states)
            } else {
                (Vec::new(), HashMap::new())
            };

        // Process server messages
        for (player_id, msg) in server_messages {
            self.handle_message_from(player_id, msg);
        }

        // Update remote player states from server
        for (id, state) in server_states {
            if Some(id) != self.my_id {
                if let Some(player) = self.remote_players.get_mut(&id) {
                    player.target_position = Vec3::from_array(state.position);
                    player.velocity = Vec3::from_array(state.velocity);
                    player.yaw = state.yaw;
                    player.pitch = state.pitch;
                    player.on_ground = state.on_ground;
                }
            }
        }
    }

    /// Handle a network message
    fn handle_message(&mut self, msg: NetMessage) {
        match msg {
            NetMessage::PlayerJoined { player } => {
                log::info!("Player joined: {}", player.name);
                self.remote_players.insert(player.id, RemotePlayer {
                    id: player.id,
                    name: player.name,
                    color: player.color,
                    position: Vec3::ZERO,
                    velocity: Vec3::ZERO,
                    yaw: 0.0,
                    pitch: 0.0,
                    on_ground: false,
                    target_position: Vec3::ZERO,
                    last_update: 0.0,
                });
            }

            NetMessage::PlayerLeft { player_id, .. } => {
                self.remote_players.remove(&player_id);
            }

            NetMessage::PlayerSync { player_id, position, velocity, yaw, pitch, on_ground, timestamp } => {
                if Some(player_id) != self.my_id {
                    if let Some(player) = self.remote_players.get_mut(&player_id) {
                        player.target_position = Vec3::from_array(position);
                        player.velocity = Vec3::from_array(velocity);
                        player.yaw = yaw;
                        player.pitch = pitch;
                        player.on_ground = on_ground;
                        player.last_update = timestamp;
                    } else {
                        // Unknown player - might have missed join message
                        self.remote_players.insert(player_id, RemotePlayer {
                            id: player_id,
                            name: format!("Player_{}", &player_id.to_string()[..4]),
                            color: [0.5, 0.5, 0.5],
                            position: Vec3::from_array(position),
                            velocity: Vec3::from_array(velocity),
                            yaw,
                            pitch,
                            on_ground,
                            target_position: Vec3::from_array(position),
                            last_update: timestamp,
                        });
                    }
                }
            }

            NetMessage::Chat { sender_name, message, .. } => {
                self.chat_messages.push(ChatMessage {
                    sender: sender_name,
                    message,
                    timestamp: 0.0,
                });
                // Keep last 50 messages
                if self.chat_messages.len() > 50 {
                    self.chat_messages.remove(0);
                }
            }

            _ => {}
        }
    }

    /// Handle message from a specific player (server mode)
    fn handle_message_from(&mut self, player_id: PlayerId, msg: NetMessage) {
        match msg {
            NetMessage::PlayerSync { position, velocity, yaw, pitch, on_ground, .. } => {
                if let Some(player) = self.remote_players.get_mut(&player_id) {
                    player.target_position = Vec3::from_array(position);
                    player.velocity = Vec3::from_array(velocity);
                    player.yaw = yaw;
                    player.pitch = pitch;
                    player.on_ground = on_ground;
                }
            }
            _ => {}
        }
    }

    /// Send our position to the network
    pub fn send_position(&self, position: Vec3, velocity: Vec3, yaw: f32, pitch: f32, on_ground: bool) {
        if !self.is_online() {
            return;
        }

        let pos_arr = position.to_array();
        let vel_arr = velocity.to_array();

        if let (Some(runtime), Some(client)) = (&self.runtime, &self.client) {
            runtime.block_on(async {
                let c = client.read().await;
                c.send_position(pos_arr, vel_arr, yaw, pitch, on_ground);
            });
        }

        if let (Some(runtime), Some(server)) = (&self.runtime, &self.server) {
            if let Some(my_id) = self.my_id {
                let sync = PlayerSyncData {
                    id: my_id,
                    position: pos_arr,
                    velocity: vel_arr,
                    yaw,
                    pitch,
                    on_ground,
                };
                runtime.block_on(async {
                    server.read().await.update_player(my_id, sync).await;
                });
            }
        }
    }

    /// Send a chat message
    pub fn send_chat(&self, message: &str) {
        if !self.is_online() {
            return;
        }

        if let (Some(runtime), Some(client)) = (&self.runtime, &self.client) {
            let msg = message.to_string();
            runtime.block_on(async {
                client.read().await.send_chat(msg);
            });
        }

        // TODO: Server chat broadcast
    }

    /// Get all remote players for rendering
    pub fn remote_players(&self) -> &HashMap<PlayerId, RemotePlayer> {
        &self.remote_players
    }

    /// Get chat messages
    pub fn chat_messages(&self) -> &[ChatMessage] {
        &self.chat_messages
    }

    /// Get connection status string
    pub fn status_string(&self) -> String {
        match &self.mode {
            NetworkMode::Offline => "Offline".to_string(),
            NetworkMode::Host { port } => format!("Hosting on port {} ({} players)", port, self.player_count()),
            NetworkMode::Client { address } => format!("Connected to {} ({} players)", address, self.player_count()),
        }
    }
}

impl Drop for NetworkManager {
    fn drop(&mut self) {
        // Shutdown server if hosting
        if let Some(server) = &self.server {
            if let Some(runtime) = &self.runtime {
                runtime.block_on(async {
                    server.read().await.shutdown();
                });
            }
        }
    }
}
