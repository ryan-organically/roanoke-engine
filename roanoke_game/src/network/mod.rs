//! Multiplayer Networking Module
//!
//! Provides WebSocket-based multiplayer for local/LAN play.
//!
//! Usage:
//!   Host:  cargo run -- --host 7878
//!   Join:  cargo run -- --join 192.168.1.X:7878

mod messages;
mod server;
mod client;
mod manager;
mod cli;
mod remote_renderer;

pub use messages::*;
pub use server::NetworkServer;
pub use client::NetworkClient;
pub use manager::{NetworkManager, NetworkMode, RemotePlayer};
pub use cli::{NetworkLaunchMode, print_usage};
pub use remote_renderer::{RemotePlayerOrb, PlayerNametag, RemotePlayerBatch};
