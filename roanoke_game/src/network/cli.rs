//! Command-line argument parsing for multiplayer mode

use std::env;

/// Network launch configuration
#[derive(Debug, Clone)]
pub enum NetworkLaunchMode {
    /// Single player (no networking)
    Offline,
    /// Host a game on specified port
    Host { port: u16 },
    /// Join a game at specified address
    Join { address: String, player_name: String },
}

impl NetworkLaunchMode {
    /// Parse network mode from command line arguments
    ///
    /// Usage:
    ///   cargo run                          # Offline (single-player)
    ///   cargo run -- --host 7878           # Host on port 7878
    ///   cargo run -- --join 192.168.1.5:7878 --name "Player1"
    pub fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--host" | "-h" => {
                    let port = if i + 1 < args.len() {
                        args[i + 1].parse().unwrap_or(7878)
                    } else {
                        7878
                    };
                    return Self::Host { port };
                }
                "--join" | "-j" => {
                    let address = if i + 1 < args.len() {
                        args[i + 1].clone()
                    } else {
                        "127.0.0.1:7878".to_string()
                    };

                    // Look for --name
                    let mut player_name = format!("Player_{}", rand::random::<u16>());
                    for j in (i + 1)..args.len() {
                        if args[j] == "--name" || args[j] == "-n" {
                            if j + 1 < args.len() {
                                player_name = args[j + 1].clone();
                            }
                            break;
                        }
                    }

                    return Self::Join { address, player_name };
                }
                _ => {}
            }
            i += 1;
        }

        Self::Offline
    }

    /// Check if we're in offline mode
    pub fn is_offline(&self) -> bool {
        matches!(self, Self::Offline)
    }

    /// Check if we're hosting
    pub fn is_host(&self) -> bool {
        matches!(self, Self::Host { .. })
    }

    /// Check if we're a client
    pub fn is_client(&self) -> bool {
        matches!(self, Self::Join { .. })
    }

    /// Get description for logging
    pub fn description(&self) -> String {
        match self {
            Self::Offline => "Offline (Single Player)".to_string(),
            Self::Host { port } => format!("Hosting on port {}", port),
            Self::Join { address, player_name } => format!("Joining {} as {}", address, player_name),
        }
    }
}

/// Print usage help
pub fn print_usage() {
    println!("Roanoke Engine - Multiplayer Options");
    println!("=====================================");
    println!();
    println!("Usage:");
    println!("  cargo run                                    # Single player");
    println!("  cargo run -- --host <port>                   # Host a game");
    println!("  cargo run -- --join <ip:port> --name <name>  # Join a game");
    println!();
    println!("Examples:");
    println!("  cargo run -- --host 7878");
    println!("  cargo run -- --join 192.168.1.5:7878 --name \"Hunter\"");
    println!();
}
