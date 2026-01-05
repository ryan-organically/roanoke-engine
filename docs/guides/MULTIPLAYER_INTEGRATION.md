# Multiplayer Integration Guide

## Quick Start

Once integrated, you can test multiplayer with:

```bash
# Terminal 1 - Host
cargo run -- --host 7878

# Terminal 2 - Join
cargo run -- --join 127.0.0.1:7878 --name "Player2"
```

## Integration Steps

### Step 1: Add mod declaration

In `main.rs`, add after the other mod declarations (~line 62):

```rust
mod network;
```

### Step 2: Add to SharedState struct

Add this field to `SharedState` (~line 409, before the closing brace):

```rust
    // Multiplayer Networking
    network: network::NetworkManager,
```

### Step 3: Initialize NetworkManager

In the `SharedState` initialization in `main()` (~line 1477), add:

```rust
    // Parse network mode from CLI args
    let net_mode = network::NetworkLaunchMode::from_args();
    println!("[NET] Launch mode: {}", net_mode.description());
```

Then when initializing SharedState, replace the seed initialization:

```rust
    // Before SharedState creation, determine seed and network
    let (network_manager, effective_seed) = match &net_mode {
        network::NetworkLaunchMode::Offline => {
            (network::NetworkManager::offline(12345), 12345u32)
        }
        network::NetworkLaunchMode::Host { port } => {
            let seed: u32 = rand::random();
            match network::NetworkManager::host(*port, seed) {
                Ok(nm) => (nm, seed),
                Err(e) => {
                    eprintln!("[NET] Failed to start host: {}", e);
                    (network::NetworkManager::offline(seed), seed)
                }
            }
        }
        network::NetworkLaunchMode::Join { address, player_name } => {
            match network::NetworkManager::join(address, player_name) {
                Ok(nm) => {
                    let seed = nm.seed();
                    (nm, seed)
                }
                Err(e) => {
                    eprintln!("[NET] Failed to join: {}", e);
                    (network::NetworkManager::offline(12345), 12345)
                }
            }
        }
    };
```

Then in SharedState initialization:

```rust
    seed: effective_seed,
    // ... other fields ...
    network: network_manager,
```

### Step 4: Send position updates

In the game loop where player position updates (~line 4033), add after player.update():

```rust
    // Send position to network
    state.network.send_position(
        state.player.position,
        state.player.velocity,
        state.player.yaw,
        state.player.pitch,
        state.player.on_ground,
    );
```

### Step 5: Update network each frame

Add to the game update section (after player update):

```rust
    // Update network
    let game_time = state.game_progression.game_time;
    state.network.update(delta, game_time);
```

### Step 6: Render remote players (basic capsules)

Add a simple remote player rendering section. For now, use the animal orb pipeline as colored markers:

```rust
    // Render remote players as colored orbs
    if state.network.is_online() {
        for (id, remote) in state.network.remote_players() {
            // Add orb at remote player position
            let orb = OrbInstance {
                position: remote.position,
                color: remote.color,
                scale: 1.0,
                pulse: 0.0,
            };
            // Add to orb batch for rendering
            // (integrate with existing AnimalOrbPipeline)
        }
    }
```

### Step 7: Display network status in UI

Add to the debug UI section:

```rust
    if state.network.is_online() {
        ui.label(format!("Network: {}", state.network.status_string()));
        for (_, player) in state.network.remote_players() {
            ui.label(format!("  {} at ({:.0}, {:.0}, {:.0})",
                player.name,
                player.position.x,
                player.position.y,
                player.position.z
            ));
        }
    }
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        main.rs                               │
│                                                              │
│  ┌─────────────────┐      ┌─────────────────────────────┐   │
│  │  SharedState    │      │     Game Loop               │   │
│  │                 │      │                             │   │
│  │  network: ──────┼──────┼──▶ network.update()        │   │
│  │    NetworkMgr   │      │    network.send_position() │   │
│  │                 │      │    network.remote_players()│   │
│  └─────────────────┘      └─────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   network/manager.rs                         │
│                                                              │
│  NetworkManager {                                            │
│    mode: Host | Client | Offline                             │
│    server: Option<NetworkServer>                             │
│    client: Option<NetworkClient>                             │
│    remote_players: HashMap<PlayerId, RemotePlayer>          │
│  }                                                           │
└──────────────────────┬───────────────────┬──────────────────┘
                       │                   │
          ┌────────────▼────────┐ ┌───────▼──────────┐
          │  network/server.rs  │ │ network/client.rs│
          │                     │ │                  │
          │  WebSocket Server   │ │ WebSocket Client │
          │  - Accept conns     │ │ - Connect        │
          │  - Broadcast sync   │ │ - Receive sync   │
          │  - Handle messages  │ │ - Send updates   │
          └─────────────────────┘ └──────────────────┘
```

## Message Flow

### Hosting

```
Host starts → Server binds to port → Clients connect → Server assigns IDs
                                           ↓
                   Client sends PlayerSync ← Server broadcasts to others
                                           ↓
                              All clients see remote positions
```

### Joining

```
Client connects → Sends JoinRequest → Server validates
                                           ↓
                   Server sends JoinAccepted (with seed, player list)
                                           ↓
                   Client uses server's seed for terrain generation
                                           ↓
                            Everyone sees same world!
```

## Testing Locally

1. Build the project: `cargo build`
2. Open two terminals
3. In terminal 1: `cargo run -- --host 7878`
4. In terminal 2: `cargo run -- --join 127.0.0.1:7878 --name "Friend"`
5. Both should see the same terrain (same seed)
6. Walk around - you should see each other as markers

## LAN Testing

1. Find your local IP: `ipconfig` (Windows) or `ifconfig` (Mac/Linux)
2. Host on your machine: `cargo run -- --host 7878`
3. On another computer, same network: `cargo run -- --join 192.168.1.X:7878 --name "Friend"`

## Files Created

```
roanoke_game/src/network/
├── mod.rs          # Module exports
├── messages.rs     # NetMessage enum, PlayerSync, etc.
├── server.rs       # WebSocket server for hosting
├── client.rs       # WebSocket client for joining
├── manager.rs      # NetworkManager (unified interface)
└── cli.rs          # Command-line argument parsing
```

## Next Steps

After basic integration works:

1. **Player Models**: Render remote players as actual player models instead of orbs
2. **Interpolation**: Improve position smoothing for 60Hz rendering from 20Hz updates
3. **Chat UI**: Add text chat interface
4. **Actions**: Sync attacks, jumps, interactions
5. **Animal Sync**: Host controls animal spawns, clients see same animals
6. **Combat**: Hit registration, damage sync
