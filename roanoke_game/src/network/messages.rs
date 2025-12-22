//! Network Message Types
//!
//! All messages are serialized with bincode for efficiency.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a player in the network session
pub type PlayerId = Uuid;

/// All possible network messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetMessage {
    // === Connection ===

    /// Client requests to join the session
    JoinRequest {
        player_name: String,
        version: u32,
    },

    /// Server accepts the join request
    JoinAccepted {
        your_id: PlayerId,
        server_seed: u32,
        players: Vec<PlayerInfo>,
    },

    /// Server rejects the join request
    JoinRejected {
        reason: String,
    },

    /// Notify all clients of a new player
    PlayerJoined {
        player: PlayerInfo,
    },

    /// Notify all clients of a player leaving
    PlayerLeft {
        player_id: PlayerId,
        reason: String,
    },

    // === State Sync ===

    /// Player position/rotation update (sent frequently)
    PlayerSync {
        player_id: PlayerId,
        position: [f32; 3],
        velocity: [f32; 3],
        yaw: f32,
        pitch: f32,
        on_ground: bool,
        timestamp: f64,
    },

    /// Batch sync of all players (server -> clients periodically)
    WorldSync {
        players: Vec<PlayerSyncData>,
        server_time: f64,
    },

    // === Actions ===

    /// Player performed an action (jump, attack, etc.)
    PlayerAction {
        player_id: PlayerId,
        action: PlayerActionType,
        position: [f32; 3],
        direction: [f32; 3],
    },

    // === Chat ===

    /// Chat message
    Chat {
        sender_id: PlayerId,
        sender_name: String,
        message: String,
    },

    // === Keepalive ===

    /// Ping to measure latency
    Ping {
        timestamp: f64,
    },

    /// Pong response
    Pong {
        timestamp: f64,
        server_time: f64,
    },
}

/// Basic player info for lobby/join
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub name: String,
    pub color: [f32; 3],
}

/// Compact player sync data for batch updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSyncData {
    pub id: PlayerId,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

/// Types of player actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerActionType {
    Jump,
    Attack,
    Interact,
    Crouch,
    Sprint,
}

impl NetMessage {
    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize message")
    }

    /// Deserialize message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

/// Network protocol version - increment when messages change
pub const PROTOCOL_VERSION: u32 = 1;

/// Default server port
pub const DEFAULT_PORT: u16 = 7878;

/// How often to send position updates (Hz)
pub const SYNC_RATE: f32 = 20.0;

/// How often server sends full world sync (Hz)
pub const WORLD_SYNC_RATE: f32 = 5.0;
