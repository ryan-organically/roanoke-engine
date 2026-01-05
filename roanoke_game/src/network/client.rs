//! WebSocket Client for joining multiplayer sessions

use crate::network::messages::*;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message;

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed(String),
}

/// Client state
struct ClientState {
    connection_state: ConnectionState,
    my_id: Option<PlayerId>,
    my_name: String,
    server_seed: Option<u32>,
    players: HashMap<PlayerId, PlayerInfo>,
    player_states: HashMap<PlayerId, PlayerSyncData>,
    ping_ms: f64,
    last_ping_time: Option<f64>,
}

/// Network client for joining games
pub struct NetworkClient {
    state: Arc<RwLock<ClientState>>,
    /// Channel to send messages to server
    outgoing_tx: Option<mpsc::UnboundedSender<NetMessage>>,
    /// Channel to receive messages (for game thread)
    pub incoming_rx: Option<mpsc::UnboundedReceiver<NetMessage>>,
    incoming_tx: mpsc::UnboundedSender<NetMessage>,
}

impl NetworkClient {
    pub fn new(player_name: String) -> Self {
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();

        Self {
            state: Arc::new(RwLock::new(ClientState {
                connection_state: ConnectionState::Disconnected,
                my_id: None,
                my_name: player_name,
                server_seed: None,
                players: HashMap::new(),
                player_states: HashMap::new(),
                ping_ms: 0.0,
                last_ping_time: None,
            })),
            outgoing_tx: None,
            incoming_rx: Some(incoming_rx),
            incoming_tx,
        }
    }

    /// Connect to a server
    pub async fn connect(&mut self, addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = if addr.starts_with("ws://") {
            addr.to_string()
        } else {
            format!("ws://{}", addr)
        };

        log::info!("Connecting to {}", url);

        {
            let mut state = self.state.write().await;
            state.connection_state = ConnectionState::Connecting;
        }

        let (ws_stream, _) = tokio_tungstenite::connect_async(&url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<NetMessage>();
        self.outgoing_tx = Some(outgoing_tx.clone());

        // Send join request
        let player_name = {
            let state = self.state.read().await;
            state.my_name.clone()
        };

        let join_msg = NetMessage::JoinRequest {
            player_name,
            version: PROTOCOL_VERSION,
        };
        let bytes = join_msg.to_bytes();
        ws_sender.send(Message::Binary(bytes)).await?;

        let state = self.state.clone();
        let incoming_tx = self.incoming_tx.clone();

        // Spawn sender task
        tokio::spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                let bytes = msg.to_bytes();
                if ws_sender.send(Message::Binary(bytes)).await.is_err() {
                    break;
                }
            }
        });

        // Spawn receiver task
        tokio::spawn(async move {
            while let Some(result) = ws_receiver.next().await {
                let msg = match result {
                    Ok(Message::Binary(data)) => match NetMessage::from_bytes(&data) {
                        Ok(m) => m,
                        Err(e) => {
                            log::warn!("Invalid message from server: {}", e);
                            continue;
                        }
                    },
                    Ok(Message::Close(_)) => {
                        let mut s = state.write().await;
                        s.connection_state = ConnectionState::Disconnected;
                        break;
                    }
                    Ok(_) => continue,
                    Err(e) => {
                        log::error!("WebSocket error: {}", e);
                        let mut s = state.write().await;
                        s.connection_state = ConnectionState::Failed(e.to_string());
                        break;
                    }
                };

                // Process message
                match &msg {
                    NetMessage::JoinAccepted { your_id, server_seed, players } => {
                        let mut s = state.write().await;
                        s.my_id = Some(*your_id);
                        s.server_seed = Some(*server_seed);
                        s.connection_state = ConnectionState::Connected;
                        for p in players {
                            s.players.insert(p.id, p.clone());
                        }
                        log::info!("Joined server as {} (seed: {})", your_id, server_seed);
                    }

                    NetMessage::JoinRejected { reason } => {
                        let mut s = state.write().await;
                        s.connection_state = ConnectionState::Failed(reason.clone());
                        log::error!("Join rejected: {}", reason);
                        break;
                    }

                    NetMessage::PlayerJoined { player } => {
                        let mut s = state.write().await;
                        log::info!("Player {} joined", player.name);
                        s.players.insert(player.id, player.clone());
                    }

                    NetMessage::PlayerLeft { player_id, reason } => {
                        let mut s = state.write().await;
                        if let Some(p) = s.players.remove(player_id) {
                            log::info!("Player {} left: {}", p.name, reason);
                        }
                        s.player_states.remove(player_id);
                    }

                    NetMessage::PlayerSync { player_id, position, velocity, yaw, pitch, on_ground, .. } => {
                        let mut s = state.write().await;
                        // Don't update our own state from server
                        if Some(*player_id) != s.my_id {
                            s.player_states.insert(*player_id, PlayerSyncData {
                                id: *player_id,
                                position: *position,
                                velocity: *velocity,
                                yaw: *yaw,
                                pitch: *pitch,
                                on_ground: *on_ground,
                            });
                        }
                    }

                    NetMessage::WorldSync { players, .. } => {
                        let mut s = state.write().await;
                        for p in players {
                            if Some(p.id) != s.my_id {
                                s.player_states.insert(p.id, p.clone());
                            }
                        }
                    }

                    NetMessage::Pong { timestamp, .. } => {
                        let mut s = state.write().await;
                        if let Some(send_time) = s.last_ping_time {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64();
                            s.ping_ms = (now - send_time) * 1000.0;
                        }
                        s.last_ping_time = Some(*timestamp);
                    }

                    _ => {}
                }

                // Forward to game thread
                let _ = incoming_tx.send(msg);
            }
        });

        Ok(())
    }

    /// Send a message to the server
    pub fn send(&self, msg: NetMessage) {
        if let Some(tx) = &self.outgoing_tx {
            let _ = tx.send(msg);
        }
    }

    /// Send our position update
    pub fn send_position(&self, position: [f32; 3], velocity: [f32; 3], yaw: f32, pitch: f32, on_ground: bool) {
        if let Some(tx) = &self.outgoing_tx {
            let msg = NetMessage::PlayerSync {
                player_id: PlayerId::nil(), // Server knows who we are
                position,
                velocity,
                yaw,
                pitch,
                on_ground,
                timestamp: 0.0,
            };
            let _ = tx.send(msg);
        }
    }

    /// Send a chat message
    pub fn send_chat(&self, message: String) {
        self.send(NetMessage::Chat {
            sender_id: PlayerId::nil(),
            sender_name: String::new(),
            message,
        });
    }

    /// Send ping
    pub async fn send_ping(&self) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        {
            let mut s = self.state.write().await;
            s.last_ping_time = Some(timestamp);
        }

        self.send(NetMessage::Ping { timestamp });
    }

    /// Get connection state
    pub async fn connection_state(&self) -> ConnectionState {
        self.state.read().await.connection_state.clone()
    }

    /// Get our player ID
    pub async fn my_id(&self) -> Option<PlayerId> {
        self.state.read().await.my_id
    }

    /// Get server seed (for terrain generation)
    pub async fn server_seed(&self) -> Option<u32> {
        self.state.read().await.server_seed
    }

    /// Get all remote player states
    pub async fn get_player_states(&self) -> HashMap<PlayerId, PlayerSyncData> {
        self.state.read().await.player_states.clone()
    }

    /// Get player info
    pub async fn get_players(&self) -> HashMap<PlayerId, PlayerInfo> {
        self.state.read().await.players.clone()
    }

    /// Get current ping
    pub async fn ping_ms(&self) -> f64 {
        self.state.read().await.ping_ms
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.state.read().await.connection_state == ConnectionState::Connected
    }
}
