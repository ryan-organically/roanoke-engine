# Multiplayer Demo Testability Specification

## Roanoke Engine - Demo Testing Framework with Proximity Chat

This document specifies the architecture for testing multiplayer functionality locally without requiring full server infrastructure, including proximity-based voice/text chat.

---

## Table of Contents

1. [Overview](#overview)
2. [Demo Testing Modes](#demo-testing-modes)
3. [Local Multi-Client Architecture](#local-multi-client-architecture)
4. [Proximity Chat System](#proximity-chat-system)
5. [Test Scenarios](#test-scenarios)
6. [Network Simulation](#network-simulation)
7. [Implementation Plan](#implementation-plan)

---

## Overview

### Goals

- **Zero-Infrastructure Testing**: Run multiplayer demos on a single machine
- **Proximity Chat Validation**: Test distance-based audio attenuation locally
- **Realistic Simulation**: Simulate latency, packet loss, and player count
- **Demo-Ready**: Polished enough for investor/press demonstrations
- **Developer Workflow**: Fast iteration without server deployment

### Non-Goals (for Demo)

- Production-scale (200+ players)
- Persistent world state
- Anti-cheat systems
- Cross-region play

---

## Demo Testing Modes

### Mode 1: Split-Screen Local (2-4 Players)

Single game instance with multiple viewports and input devices.

```rust
/// Split-screen configuration
pub struct SplitScreenConfig {
    pub player_count: u32,             // 2-4
    pub layout: SplitLayout,
    pub input_assignments: Vec<InputDevice>,
}

pub enum SplitLayout {
    Horizontal2,      // Top/bottom
    Vertical2,        // Left/right
    Quadrant4,        // 2x2 grid
    ThreeOneBottom,   // 3 top, 1 bottom
}

pub enum InputDevice {
    KeyboardWASD,
    KeyboardArrows,
    Gamepad(u32),     // Controller index
}
```

**Use Case**: Quick testing, local couch co-op demo, input binding verification.

### Mode 2: Localhost Multi-Instance (2-8 Players)

Multiple game instances on same machine connected via localhost.

```rust
/// Localhost testing configuration
pub struct LocalhostTestConfig {
    pub server_port: u16,              // Default: 7878
    pub client_ports: Vec<u16>,        // Auto-assigned if empty
    pub headless_server: bool,         // Run server without window
    pub auto_spawn_clients: u32,       // Spawn N client windows
    pub window_arrangement: WindowArrangement,
}

pub enum WindowArrangement {
    Tiled,            // Automatic grid tiling
    Stacked,          // Overlapping (manual arrangement)
    PrimarySecondary, // One large + small thumbnails
}
```

**Use Case**: Network code testing, proximity chat, player sync validation.

### Mode 3: LAN Party (2-16 Players)

Multiple machines on local network.

```rust
/// LAN discovery and connection
pub struct LANConfig {
    pub broadcast_port: u16,           // Discovery broadcast
    pub game_port: u16,                // Game traffic
    pub server_name: String,
    pub max_players: u32,
    pub password: Option<String>,
}

/// Auto-discovery message
#[derive(Serialize, Deserialize)]
pub struct LANBroadcast {
    pub server_name: String,
    pub player_count: u32,
    pub max_players: u32,
    pub game_version: String,
    pub map_name: String,
}
```

**Use Case**: Press demos, investor showcases, playtesting sessions.

### Mode 4: Simulated Players (Bots)

AI-controlled players for stress testing and solo development.

```rust
/// Bot configuration
pub struct BotConfig {
    pub count: u32,
    pub behavior: BotBehavior,
    pub spawn_delay: f32,              // Stagger spawns
    pub chat_frequency: f32,           // Messages per minute
}

pub enum BotBehavior {
    Idle,              // Stand still
    Wander,            // Random movement
    Follow(PlayerId),  // Follow a player
    Hunt,              // Engage in hunting behavior
    Stress,            // Rapid random actions
}
```

**Use Case**: Performance testing, solo proximity chat testing, load simulation.

---

## Local Multi-Client Architecture

### Process Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    LOCAL DEMO ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                   HOST PROCESS                           │   │
│   │  ┌─────────────┐  ┌─────────────┐  ┌────────────────┐  │   │
│   │  │   Server    │  │   Client    │  │   Proximity    │  │   │
│   │  │   Logic     │  │   (Host)    │  │   Chat Mixer   │  │   │
│   │  └──────┬──────┘  └──────┬──────┘  └───────┬────────┘  │   │
│   │         │                │                  │           │   │
│   │         └────────────────┼──────────────────┘           │   │
│   │                          │                               │   │
│   │                    localhost:7878                        │   │
│   └──────────────────────────┼──────────────────────────────┘   │
│                              │                                   │
│   ┌──────────────────────────┼──────────────────────────────┐   │
│   │            LOCAL NETWORK (loopback)                      │   │
│   └──────────────────────────┼──────────────────────────────┘   │
│                              │                                   │
│   ┌─────────────┐    ┌──────┴──────┐    ┌─────────────┐        │
│   │  Client 2   │    │  Client 3   │    │  Client N   │        │
│   │  (Window)   │    │  (Window)   │    │  (Window)   │        │
│   └─────────────┘    └─────────────┘    └─────────────┘        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Command-Line Launch

```bash
# Launch host (server + client)
roanoke_game --demo-host --players 4

# Launch additional clients (auto-connect to host)
roanoke_game --demo-client --host localhost:7878

# Launch with bots
roanoke_game --demo-host --players 2 --bots 6

# Launch headless server + separate client windows
roanoke_game --demo-server --headless
roanoke_game --demo-client --window-title "Player 1"
roanoke_game --demo-client --window-title "Player 2"
```

### Demo Network Manager

```rust
/// Simplified network manager for demo/testing
pub struct DemoNetworkManager {
    mode: DemoMode,
    connections: HashMap<PlayerId, DemoConnection>,
    local_player: PlayerId,
    server_state: Option<DemoServerState>,
    chat_system: ProximityChatSystem,
}

pub enum DemoMode {
    Host { port: u16 },
    Client { server_addr: SocketAddr },
    Offline,
}

pub struct DemoConnection {
    player_id: PlayerId,
    address: SocketAddr,
    latency_sim: LatencySimulator,
    last_state: PlayerState,
    voice_stream: Option<VoiceStream>,
}

impl DemoNetworkManager {
    pub fn tick(&mut self, dt: f32) {
        match &self.mode {
            DemoMode::Host { .. } => {
                self.process_client_inputs();
                self.broadcast_state();
                self.update_proximity_chat();
            }
            DemoMode::Client { .. } => {
                self.send_input();
                self.receive_state();
                self.receive_voice();
            }
            DemoMode::Offline => {}
        }
    }

    pub fn spawn_test_players(&mut self, count: u32) {
        for i in 0..count {
            let bot_id = PlayerId::new_bot(i);
            self.connections.insert(bot_id, DemoConnection::new_bot());
        }
    }
}
```

---

## Proximity Chat System

### Core Design

Proximity chat attenuates voice/text based on distance between players.

```rust
/// Proximity chat configuration
pub struct ProximityChatConfig {
    /// Maximum distance at which chat is audible
    pub max_range: f32,                // Default: 50.0 meters

    /// Distance at which full volume is heard
    pub full_volume_range: f32,        // Default: 10.0 meters

    /// Attenuation curve
    pub falloff: AttenuationCurve,

    /// Enable directional audio (stereo panning)
    pub directional: bool,

    /// Occlusion (walls block sound)
    pub occlusion_enabled: bool,

    /// Text chat also proximity-limited
    pub text_proximity: bool,
}

pub enum AttenuationCurve {
    Linear,           // Simple linear falloff
    InverseSquare,    // Realistic physics-based
    Logarithmic,      // Perceptually smooth
    Custom(Vec<(f32, f32)>),  // Custom curve points
}

impl Default for ProximityChatConfig {
    fn default() -> Self {
        Self {
            max_range: 50.0,
            full_volume_range: 10.0,
            falloff: AttenuationCurve::Logarithmic,
            directional: true,
            occlusion_enabled: true,
            text_proximity: true,
        }
    }
}
```

### Voice Chat Architecture

```rust
/// Voice chat system
pub struct VoiceChatSystem {
    config: ProximityChatConfig,
    local_capture: Option<AudioCapture>,
    remote_streams: HashMap<PlayerId, VoiceStream>,
    mixer: ProximityMixer,
}

pub struct VoiceStream {
    player_id: PlayerId,
    audio_buffer: RingBuffer<f32>,
    current_position: Vec3,
    current_volume: f32,
    current_pan: f32,
    muted: bool,
}

pub struct ProximityMixer {
    output_buffer: Vec<f32>,
    sample_rate: u32,
}

impl VoiceChatSystem {
    pub fn update(&mut self, local_pos: Vec3, world: &World) {
        for stream in self.remote_streams.values_mut() {
            // Calculate distance
            let distance = (local_pos - stream.current_position).length();

            // Calculate volume based on distance
            stream.current_volume = self.calculate_volume(distance);

            // Calculate stereo panning
            if self.config.directional {
                stream.current_pan = self.calculate_pan(local_pos, stream.current_position);
            }

            // Apply occlusion if enabled
            if self.config.occlusion_enabled {
                let occlusion = self.calculate_occlusion(local_pos, stream.current_position, world);
                stream.current_volume *= occlusion;
            }
        }
    }

    fn calculate_volume(&self, distance: f32) -> f32 {
        if distance <= self.config.full_volume_range {
            return 1.0;
        }
        if distance >= self.config.max_range {
            return 0.0;
        }

        let t = (distance - self.config.full_volume_range)
              / (self.config.max_range - self.config.full_volume_range);

        match &self.config.falloff {
            AttenuationCurve::Linear => 1.0 - t,
            AttenuationCurve::InverseSquare => 1.0 / (1.0 + t * t * 4.0),
            AttenuationCurve::Logarithmic => 1.0 - t.ln().max(0.0) / 4.0_f32.ln(),
            AttenuationCurve::Custom(points) => self.sample_curve(points, t),
        }
    }

    fn calculate_pan(&self, listener: Vec3, source: Vec3) -> f32 {
        // Calculate angle from listener's forward to source
        let to_source = (source - listener).normalize();
        // Simplified: use X component for stereo pan (-1 = left, 1 = right)
        to_source.x.clamp(-1.0, 1.0)
    }

    fn calculate_occlusion(&self, listener: Vec3, source: Vec3, world: &World) -> f32 {
        // Raycast from listener to source
        let ray = Ray::new(listener, (source - listener).normalize());
        let max_dist = (source - listener).length();

        let hits = world.raycast(ray, max_dist, CollisionGroup::TERRAIN | CollisionGroup::BUILDINGS);

        // Each wall reduces volume by 50%
        0.5_f32.powi(hits.len() as i32)
    }
}
```

### Text Chat with Proximity

```rust
/// Proximity text chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender: PlayerId,
    pub sender_name: String,
    pub content: String,
    pub position: Vec3,
    pub timestamp: f64,
    pub channel: ChatChannel,
}

pub enum ChatChannel {
    Proximity,         // Distance-based
    Party,             // Hunt group only
    Global,            // Everyone (admin/dev only)
    Whisper(PlayerId), // Private message
}

/// Display chat with distance-based styling
pub fn render_chat_message(
    message: &ChatMessage,
    local_pos: Vec3,
    config: &ProximityChatConfig,
) -> ChatDisplay {
    let distance = (local_pos - message.position).length();

    if distance > config.max_range && config.text_proximity {
        return ChatDisplay::Hidden;
    }

    // Fade text based on distance
    let alpha = if distance <= config.full_volume_range {
        1.0
    } else {
        let t = (distance - config.full_volume_range)
              / (config.max_range - config.full_volume_range);
        1.0 - t
    };

    ChatDisplay::Visible {
        text: message.content.clone(),
        sender: message.sender_name.clone(),
        alpha,
        // Far messages appear smaller
        scale: 0.7 + alpha * 0.3,
    }
}
```

### Voice Chat Testing Tools

```rust
/// Debug visualization for proximity chat
pub struct ProximityChatDebug {
    pub show_ranges: bool,
    pub show_volumes: bool,
    pub show_occlusion_rays: bool,
    pub show_pan_indicators: bool,
}

impl ProximityChatDebug {
    pub fn render(&self, gizmos: &mut Gizmos, chat_system: &VoiceChatSystem, local_pos: Vec3) {
        if self.show_ranges {
            // Draw full volume range (green)
            gizmos.circle(local_pos, chat_system.config.full_volume_range, Color::GREEN.with_alpha(0.3));
            // Draw max range (yellow)
            gizmos.circle(local_pos, chat_system.config.max_range, Color::YELLOW.with_alpha(0.2));
        }

        for stream in chat_system.remote_streams.values() {
            if self.show_volumes {
                // Show volume as sphere size
                let radius = stream.current_volume * 2.0;
                gizmos.sphere(stream.current_position, radius, Color::CYAN);
                // Volume percentage label
                gizmos.text(
                    stream.current_position + Vec3::Y * 2.0,
                    format!("{:.0}%", stream.current_volume * 100.0),
                );
            }

            if self.show_occlusion_rays {
                let color = if stream.current_volume > 0.0 { Color::GREEN } else { Color::RED };
                gizmos.line(local_pos, stream.current_position, color);
            }

            if self.show_pan_indicators {
                // Arrow showing pan direction
                let pan_offset = Vec3::new(stream.current_pan * 2.0, 0.0, 0.0);
                gizmos.arrow(stream.current_position, stream.current_position + pan_offset, Color::MAGENTA);
            }
        }
    }
}
```

---

## Test Scenarios

### Scenario 1: Basic Connectivity

**Purpose**: Verify players can join and see each other.

```rust
pub struct ConnectivityTest {
    pub steps: Vec<TestStep>,
}

impl ConnectivityTest {
    pub fn new() -> Self {
        Self {
            steps: vec![
                TestStep::LaunchHost,
                TestStep::WaitForServer(Duration::from_secs(2)),
                TestStep::LaunchClient(1),
                TestStep::WaitForJoin(Duration::from_secs(5)),
                TestStep::VerifyPlayerCount(2),
                TestStep::VerifyPlayersVisible,
                TestStep::LaunchClient(2),
                TestStep::WaitForJoin(Duration::from_secs(5)),
                TestStep::VerifyPlayerCount(3),
                TestStep::MovePlayer(1, Vec3::new(10.0, 0.0, 0.0)),
                TestStep::VerifyPositionSync(Duration::from_millis(100)),
            ],
        }
    }
}
```

### Scenario 2: Proximity Chat Range

**Purpose**: Verify voice attenuation at various distances.

```rust
pub struct ProximityChatTest {
    pub distances: Vec<f32>,
    pub expected_volumes: Vec<f32>,
}

impl ProximityChatTest {
    pub fn new(config: &ProximityChatConfig) -> Self {
        Self {
            distances: vec![0.0, 5.0, 10.0, 25.0, 40.0, 50.0, 60.0],
            expected_volumes: vec![
                1.0,   // 0m - full volume
                1.0,   // 5m - within full range
                1.0,   // 10m - edge of full range
                0.6,   // 25m - medium attenuation
                0.25,  // 40m - heavy attenuation
                0.0,   // 50m - max range cutoff
                0.0,   // 60m - beyond range
            ],
        }
    }

    pub fn run(&self, chat_system: &mut VoiceChatSystem) -> TestResult {
        for (distance, expected) in self.distances.iter().zip(&self.expected_volumes) {
            let actual = chat_system.calculate_volume(*distance);
            let tolerance = 0.1;

            if (actual - expected).abs() > tolerance {
                return TestResult::Failed(format!(
                    "At {}m: expected {:.2}, got {:.2}",
                    distance, expected, actual
                ));
            }
        }
        TestResult::Passed
    }
}
```

### Scenario 3: Hunt Coordination

**Purpose**: Test multiplayer hunting event with proximity chat coordination.

```rust
pub struct HuntCoordinationTest {
    pub player_count: u32,
    pub herd_position: Vec3,
}

impl HuntCoordinationTest {
    pub fn run(&self) -> Vec<TestAssertion> {
        vec![
            // Phase 1: Discovery
            TestAssertion::new("Discovery broadcast reaches all players within 300m"),
            TestAssertion::new("Players outside 300m do not receive broadcast"),

            // Phase 2: Coordination (proximity chat critical)
            TestAssertion::new("Leader's voice audible to all hunt members"),
            TestAssertion::new("Flankers can communicate with nearby flankers"),
            TestAssertion::new("Distant scouts have attenuated voice"),

            // Phase 3: Hunt commands
            TestAssertion::new("HuntCommand::Engage reaches all participants"),
            TestAssertion::new("Position updates sync within 100ms"),

            // Phase 4: Completion
            TestAssertion::new("Kill notifications broadcast to all"),
            TestAssertion::new("Reward distribution calculates correctly"),
        ]
    }
}
```

### Scenario 4: Stress Test

**Purpose**: Find performance limits.

```rust
pub struct StressTestConfig {
    pub player_count: u32,           // Target player count
    pub bot_count: u32,              // AI players
    pub voice_streams: u32,          // Concurrent voice streams
    pub message_rate: f32,           // Chat messages per second
    pub movement_chaos: bool,        // Random movement
    pub target_fps: u32,             // Minimum acceptable FPS
    pub target_latency_ms: u32,      // Maximum acceptable latency
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            player_count: 8,
            bot_count: 24,
            voice_streams: 8,
            message_rate: 10.0,
            movement_chaos: true,
            target_fps: 30,
            target_latency_ms: 150,
        }
    }
}
```

---

## Network Simulation

### Latency and Packet Loss

```rust
/// Simulate real network conditions
pub struct LatencySimulator {
    pub enabled: bool,
    pub min_latency_ms: u32,
    pub max_latency_ms: u32,
    pub packet_loss_percent: f32,
    pub jitter_ms: u32,
    pub duplicate_percent: f32,
    pending_packets: VecDeque<(Instant, Vec<u8>)>,
}

impl LatencySimulator {
    pub fn send(&mut self, packet: Vec<u8>) {
        if !self.enabled {
            self.immediate_send(packet);
            return;
        }

        // Simulate packet loss
        if rand::random::<f32>() < self.packet_loss_percent / 100.0 {
            return; // Dropped
        }

        // Calculate delay with jitter
        let base_latency = rand::thread_rng()
            .gen_range(self.min_latency_ms..=self.max_latency_ms);
        let jitter = rand::thread_rng()
            .gen_range(0..=self.jitter_ms);
        let delay = Duration::from_millis((base_latency + jitter) as u64);

        let deliver_at = Instant::now() + delay;
        self.pending_packets.push_back((deliver_at, packet.clone()));

        // Simulate duplicate packets
        if rand::random::<f32>() < self.duplicate_percent / 100.0 {
            let extra_delay = delay + Duration::from_millis(rand::thread_rng().gen_range(5..20));
            self.pending_packets.push_back((Instant::now() + extra_delay, packet));
        }
    }

    pub fn receive(&mut self) -> Vec<Vec<u8>> {
        let now = Instant::now();
        let mut ready = Vec::new();

        while let Some((deliver_at, _)) = self.pending_packets.front() {
            if *deliver_at <= now {
                ready.push(self.pending_packets.pop_front().unwrap().1);
            } else {
                break;
            }
        }

        ready
    }
}

/// Preset network conditions
pub enum NetworkPreset {
    Perfect,      // 0ms latency, 0% loss
    LAN,          // 1-5ms latency, 0% loss
    GoodInternet, // 20-50ms latency, 0.1% loss
    AverageWifi,  // 40-100ms latency, 1% loss
    BadWifi,      // 80-200ms latency, 5% loss
    Mobile4G,     // 50-150ms latency, 2% loss, high jitter
    Satellite,    // 500-700ms latency, 0.5% loss
}

impl NetworkPreset {
    pub fn to_config(&self) -> LatencySimulator {
        match self {
            NetworkPreset::Perfect => LatencySimulator {
                enabled: false,
                ..Default::default()
            },
            NetworkPreset::LAN => LatencySimulator {
                enabled: true,
                min_latency_ms: 1,
                max_latency_ms: 5,
                packet_loss_percent: 0.0,
                jitter_ms: 1,
                ..Default::default()
            },
            NetworkPreset::GoodInternet => LatencySimulator {
                enabled: true,
                min_latency_ms: 20,
                max_latency_ms: 50,
                packet_loss_percent: 0.1,
                jitter_ms: 10,
                ..Default::default()
            },
            NetworkPreset::BadWifi => LatencySimulator {
                enabled: true,
                min_latency_ms: 80,
                max_latency_ms: 200,
                packet_loss_percent: 5.0,
                jitter_ms: 50,
                duplicate_percent: 0.5,
                ..Default::default()
            },
            // ... other presets
            _ => LatencySimulator::default()
        }
    }
}
```

---

## Implementation Plan

### Phase 1: Local Host/Client (Foundation)

**Goal**: Two instances on localhost can connect and see each other.

- [ ] Add networking dependencies (`tokio`, `quinn` or `tokio-tungstenite`)
- [ ] Create `DemoNetworkManager` with Host/Client modes
- [ ] Implement player position sync
- [ ] Add `--demo-host` and `--demo-client` CLI flags
- [ ] Basic connection/disconnection handling

**Dependencies to add (Cargo.toml)**:
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
tokio-tungstenite = "0.21"  # WebSocket for simplicity
uuid = { version = "1.0", features = ["v4", "serde"] }
```

**Files**:
- `roanoke_game/src/multiplayer/mod.rs`
- `roanoke_game/src/multiplayer/demo_network.rs`
- `roanoke_game/src/multiplayer/messages.rs`

### Phase 2: Proximity Text Chat

**Goal**: Text chat with distance-based visibility.

- [ ] Implement `ChatMessage` struct
- [ ] Add chat UI panel (egui)
- [ ] Implement proximity filtering
- [ ] Add chat display with alpha fade
- [ ] Keybinding for chat input (Enter)

**Files**:
- `roanoke_game/src/multiplayer/chat.rs`
- `roanoke_game/src/ui/chat_panel.rs`

### Phase 3: Voice Chat Infrastructure

**Goal**: Voice capture and playback framework.

- [ ] Add audio capture (`cpal` crate)
- [ ] Implement `VoiceChatSystem`
- [ ] Add `VoiceStream` per remote player
- [ ] Implement `ProximityMixer`
- [ ] Push-to-talk keybinding (V)

**Dependencies**:
```toml
[dependencies]
cpal = "0.15"
opus = "0.3"  # Audio compression
```

**Files**:
- `roanoke_game/src/multiplayer/voice.rs`
- `roanoke_game/src/multiplayer/audio_capture.rs`

### Phase 4: Proximity Audio Processing

**Goal**: Distance-based volume, panning, occlusion.

- [ ] Implement volume attenuation curves
- [ ] Add stereo panning based on direction
- [ ] Implement occlusion raycasting
- [ ] Add debug visualization
- [ ] Tune falloff curves

**Files**:
- `roanoke_game/src/multiplayer/proximity_audio.rs`

### Phase 5: Network Simulation

**Goal**: Test under realistic conditions.

- [ ] Implement `LatencySimulator`
- [ ] Add network presets
- [ ] Add debug overlay for network stats
- [ ] Implement jitter buffer for voice
- [ ] Test and tune for bad conditions

**Files**:
- `roanoke_game/src/multiplayer/network_sim.rs`

### Phase 6: Bots and Stress Testing

**Goal**: Test at scale without humans.

- [ ] Implement bot spawning
- [ ] Add bot behaviors
- [ ] Create stress test scenarios
- [ ] Performance profiling
- [ ] Optimize bottlenecks

**Files**:
- `roanoke_game/src/multiplayer/bots.rs`
- `roanoke_game/src/multiplayer/stress_test.rs`

---

## Demo UI Overlay

```rust
/// In-game demo control panel
pub struct DemoOverlay {
    pub visible: bool,
    pub network_stats: NetworkStats,
    pub player_list: Vec<PlayerListEntry>,
    pub chat_debug: ProximityChatDebug,
}

pub struct NetworkStats {
    pub ping_ms: u32,
    pub packet_loss: f32,
    pub bandwidth_up: f32,
    pub bandwidth_down: f32,
    pub connected_players: u32,
}

pub struct PlayerListEntry {
    pub name: String,
    pub ping: u32,
    pub distance: f32,
    pub voice_volume: f32,
    pub is_talking: bool,
}

impl DemoOverlay {
    pub fn render(&self, ui: &mut egui::Ui) {
        if !self.visible { return; }

        egui::Window::new("Demo Control")
            .show(ui.ctx(), |ui| {
                // Network stats
                ui.heading("Network");
                ui.label(format!("Ping: {}ms", self.network_stats.ping_ms));
                ui.label(format!("Players: {}", self.network_stats.connected_players));

                // Player list
                ui.heading("Players");
                for player in &self.player_list {
                    ui.horizontal(|ui| {
                        if player.is_talking {
                            ui.label("🎤");
                        }
                        ui.label(&player.name);
                        ui.label(format!("{}m", player.distance as u32));
                        ui.label(format!("vol:{:.0}%", player.voice_volume * 100.0));
                    });
                }

                // Debug toggles
                ui.heading("Debug");
                ui.checkbox(&mut self.chat_debug.show_ranges, "Show chat ranges");
                ui.checkbox(&mut self.chat_debug.show_volumes, "Show volumes");
                ui.checkbox(&mut self.chat_debug.show_occlusion_rays, "Show occlusion");
            });
    }
}
```

---

## Appendix: Quick Reference

### CLI Flags

| Flag | Description |
|------|-------------|
| `--demo-host` | Start as host (server + client) |
| `--demo-client` | Start as client |
| `--host <addr>` | Server address to connect to |
| `--port <port>` | Port to use (default: 7878) |
| `--bots <n>` | Spawn N bot players |
| `--network <preset>` | Network simulation preset |
| `--headless` | Run server without window |
| `--player-name <name>` | Set player display name |

### Keybindings (Demo Mode)

| Key | Action |
|-----|--------|
| `Enter` | Open chat input |
| `V` (hold) | Push-to-talk |
| `Tab` | Show player list |
| `F3` | Toggle demo overlay |
| `F4` | Toggle proximity debug |

### Default Ports

| Service | Port |
|---------|------|
| Game server | 7878 |
| Voice chat | 7879 |
| LAN discovery | 7880 (UDP broadcast) |

---

*End of Multiplayer Demo Testability Specification*
