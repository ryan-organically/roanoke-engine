//! WebSocket Server for hosting multiplayer sessions

use crate::network::messages::*;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message;

/// Connected client state
struct ConnectedClient {
    id: PlayerId,
    info: PlayerInfo,
    addr: SocketAddr,
    sender: mpsc::UnboundedSender<NetMessage>,
}

/// Server state shared across async tasks
struct ServerState {
    clients: HashMap<PlayerId, ConnectedClient>,
    player_states: HashMap<PlayerId, PlayerSyncData>,
    seed: u32,
    start_time: std::time::Instant,
}

/// Network server for hosting games
pub struct NetworkServer {
    port: u16,
    state: Arc<RwLock<ServerState>>,
    shutdown_tx: Option<broadcast::Sender<()>>,
    /// Channel to receive messages from clients (for game thread)
    pub incoming_rx: Option<mpsc::UnboundedReceiver<(PlayerId, NetMessage)>>,
    incoming_tx: mpsc::UnboundedSender<(PlayerId, NetMessage)>,
}

impl NetworkServer {
    pub fn new(port: u16, seed: u32) -> Self {
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();

        Self {
            port,
            state: Arc::new(RwLock::new(ServerState {
                clients: HashMap::new(),
                player_states: HashMap::new(),
                seed,
                start_time: std::time::Instant::now(),
            })),
            shutdown_tx: None,
            incoming_rx: Some(incoming_rx),
            incoming_tx,
        }
    }

    /// Start the server (call from tokio runtime)
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        log::info!("Server listening on {}", addr);

        let (shutdown_tx, _) = broadcast::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        let state = self.state.clone();
        let incoming_tx = self.incoming_tx.clone();

        // Spawn accept loop
        tokio::spawn(async move {
            let mut shutdown_rx = shutdown_tx.subscribe();

            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                log::info!("New connection from {}", addr);
                                let state = state.clone();
                                let incoming_tx = incoming_tx.clone();
                                tokio::spawn(handle_connection(stream, addr, state, incoming_tx));
                            }
                            Err(e) => {
                                log::error!("Accept error: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        log::info!("Server shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Broadcast a message to all connected clients
    pub async fn broadcast(&self, msg: NetMessage) {
        let state = self.state.read().await;
        for client in state.clients.values() {
            let _ = client.sender.send(msg.clone());
        }
    }

    /// Broadcast to all except one player
    pub async fn broadcast_except(&self, msg: NetMessage, except: PlayerId) {
        let state = self.state.read().await;
        for (id, client) in state.clients.iter() {
            if *id != except {
                let _ = client.sender.send(msg.clone());
            }
        }
    }

    /// Update local player state and broadcast
    pub async fn update_player(&self, player_id: PlayerId, sync: PlayerSyncData) {
        {
            let mut state = self.state.write().await;
            state.player_states.insert(player_id, sync.clone());
        }

        // Broadcast to other clients
        let msg = NetMessage::PlayerSync {
            player_id,
            position: sync.position,
            velocity: sync.velocity,
            yaw: sync.yaw,
            pitch: sync.pitch,
            on_ground: sync.on_ground,
            timestamp: self.server_time().await,
        };

        self.broadcast_except(msg, player_id).await;
    }

    /// Get all remote player states
    pub async fn get_player_states(&self) -> HashMap<PlayerId, PlayerSyncData> {
        let state = self.state.read().await;
        state.player_states.clone()
    }

    /// Get connected player count
    pub async fn player_count(&self) -> usize {
        let state = self.state.read().await;
        state.clients.len()
    }

    /// Get server time
    pub async fn server_time(&self) -> f64 {
        let state = self.state.read().await;
        state.start_time.elapsed().as_secs_f64()
    }

    /// Shutdown the server
    pub fn shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
    }
}

/// Handle a single client connection
async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    state: Arc<RwLock<ServerState>>,
    incoming_tx: mpsc::UnboundedSender<(PlayerId, NetMessage)>,
) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("WebSocket handshake failed for {}: {}", addr, e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (client_tx, mut client_rx) = mpsc::unbounded_channel::<NetMessage>();

    let mut player_id: Option<PlayerId> = None;

    // Spawn sender task
    let sender_task = tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            let bytes = msg.to_bytes();
            if ws_sender.send(Message::Binary(bytes)).await.is_err() {
                break;
            }
        }
    });

    // Receive loop
    while let Some(result) = ws_receiver.next().await {
        let msg = match result {
            Ok(Message::Binary(data)) => match NetMessage::from_bytes(&data) {
                Ok(m) => m,
                Err(e) => {
                    log::warn!("Invalid message from {}: {}", addr, e);
                    continue;
                }
            },
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(e) => {
                log::warn!("WebSocket error from {}: {}", addr, e);
                break;
            }
        };

        match msg {
            NetMessage::JoinRequest { player_name, version } => {
                if version != PROTOCOL_VERSION {
                    let reject = NetMessage::JoinRejected {
                        reason: format!("Version mismatch: server={}, client={}", PROTOCOL_VERSION, version),
                    };
                    let _ = client_tx.send(reject);
                    break;
                }

                let new_id = PlayerId::new_v4();
                let color = random_color(&new_id);

                let info = PlayerInfo {
                    id: new_id,
                    name: player_name.clone(),
                    color,
                };

                // Get current state
                let (seed, existing_players) = {
                    let mut s = state.write().await;

                    // Add client
                    s.clients.insert(new_id, ConnectedClient {
                        id: new_id,
                        info: info.clone(),
                        addr,
                        sender: client_tx.clone(),
                    });

                    // Initialize player state
                    s.player_states.insert(new_id, PlayerSyncData {
                        id: new_id,
                        position: [0.0, 50.0, 0.0],
                        velocity: [0.0, 0.0, 0.0],
                        yaw: 0.0,
                        pitch: 0.0,
                        on_ground: false,
                    });

                    let players: Vec<PlayerInfo> = s.clients.values()
                        .filter(|c| c.id != new_id)
                        .map(|c| c.info.clone())
                        .collect();

                    (s.seed, players)
                };

                // Send accept
                let accept = NetMessage::JoinAccepted {
                    your_id: new_id,
                    server_seed: seed,
                    players: existing_players,
                };
                let _ = client_tx.send(accept);

                // Broadcast join to others
                let join_msg = NetMessage::PlayerJoined { player: info.clone() };
                {
                    let s = state.read().await;
                    for (id, client) in s.clients.iter() {
                        if *id != new_id {
                            let _ = client.sender.send(join_msg.clone());
                        }
                    }
                }

                player_id = Some(new_id);
                log::info!("Player {} ({}) joined from {}", player_name, new_id, addr);
            }

            NetMessage::PlayerSync { position, velocity, yaw, pitch, on_ground, .. } => {
                if let Some(pid) = player_id {
                    // Update state
                    {
                        let mut s = state.write().await;
                        if let Some(ps) = s.player_states.get_mut(&pid) {
                            ps.position = position;
                            ps.velocity = velocity;
                            ps.yaw = yaw;
                            ps.pitch = pitch;
                            ps.on_ground = on_ground;
                        }
                    }

                    // Broadcast to others
                    let broadcast_msg = NetMessage::PlayerSync {
                        player_id: pid,
                        position,
                        velocity,
                        yaw,
                        pitch,
                        on_ground,
                        timestamp: state.read().await.start_time.elapsed().as_secs_f64(),
                    };

                    let s = state.read().await;
                    for (id, client) in s.clients.iter() {
                        if *id != pid {
                            let _ = client.sender.send(broadcast_msg.clone());
                        }
                    }

                    // Forward to game thread
                    let _ = incoming_tx.send((pid, NetMessage::PlayerSync {
                        player_id: pid,
                        position,
                        velocity,
                        yaw,
                        pitch,
                        on_ground,
                        timestamp: 0.0,
                    }));
                }
            }

            NetMessage::PlayerAction { action, position, direction, .. } => {
                if let Some(pid) = player_id {
                    let msg = NetMessage::PlayerAction {
                        player_id: pid,
                        action,
                        position,
                        direction,
                    };

                    // Broadcast to all including sender (for confirmation)
                    let s = state.read().await;
                    for client in s.clients.values() {
                        let _ = client.sender.send(msg.clone());
                    }
                }
            }

            NetMessage::Chat { message, .. } => {
                if let Some(pid) = player_id {
                    let s = state.read().await;
                    let sender_name = s.clients.get(&pid)
                        .map(|c| c.info.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    drop(s);

                    let msg = NetMessage::Chat {
                        sender_id: pid,
                        sender_name,
                        message,
                    };

                    let s = state.read().await;
                    for client in s.clients.values() {
                        let _ = client.sender.send(msg.clone());
                    }
                }
            }

            NetMessage::Ping { timestamp } => {
                let pong = NetMessage::Pong {
                    timestamp,
                    server_time: state.read().await.start_time.elapsed().as_secs_f64(),
                };
                let _ = client_tx.send(pong);
            }

            _ => {}
        }
    }

    // Client disconnected
    if let Some(pid) = player_id {
        let player_name = {
            let mut s = state.write().await;
            let name = s.clients.get(&pid).map(|c| c.info.name.clone()).unwrap_or_default();
            s.clients.remove(&pid);
            s.player_states.remove(&pid);
            name
        };

        log::info!("Player {} ({}) disconnected", player_name, pid);

        // Broadcast disconnect
        let msg = NetMessage::PlayerLeft {
            player_id: pid,
            reason: "Disconnected".to_string(),
        };

        let s = state.read().await;
        for client in s.clients.values() {
            let _ = client.sender.send(msg.clone());
        }
    }

    sender_task.abort();
}

/// Generate a deterministic color from player ID
fn random_color(id: &PlayerId) -> [f32; 3] {
    let bytes = id.as_bytes();
    [
        (bytes[0] as f32 / 255.0) * 0.5 + 0.5,
        (bytes[1] as f32 / 255.0) * 0.5 + 0.5,
        (bytes[2] as f32 / 255.0) * 0.5 + 0.5,
    ]
}
