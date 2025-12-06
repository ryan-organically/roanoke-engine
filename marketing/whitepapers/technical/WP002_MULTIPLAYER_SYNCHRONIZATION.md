# WHITEPAPER WP-002
## Multiplayer Synchronization in Roanoke
### Scalable Netcode for Persistent Virtual Worlds

---

**Document Classification:** Public Technical Documentation
**Version:** 1.0
**Authors:** Roanoke Engine Team
**Date:** 2025
**Abstract:** This whitepaper presents the Roanoke Engine's approach to multiplayer synchronization at scale. We detail our hybrid authority model, interest management system, and state synchronization protocols that enable thousands of concurrent players in a persistent, modifiable world while maintaining sub-100ms perceived latency.

---

## 1. Introduction

### 1.1 The Scale Challenge

Traditional multiplayer architectures face fundamental limits:

| Architecture | Max CCU | Latency | Persistence | Modification |
|--------------|---------|---------|-------------|--------------|
| P2P | 16 | Variable | None | Full |
| Dedicated Server | 64-128 | Low | Session | Full |
| MMO Sharded | 3,000 | Medium | Full | Limited |
| Cloud Instance | 100 | Low | Session | Full |

**Roanoke Requirements:**
- 10,000+ concurrent players per world
- Full world modification (building, terrain)
- Persistent across sessions
- Sub-100ms perceived latency
- Player-hosted servers possible

### 1.2 Our Approach

The Roanoke multiplayer architecture combines:

1. **Spatial partitioning** with dynamic load balancing
2. **Hybrid authority** for responsiveness and security
3. **Interest management** for bandwidth efficiency
4. **Delta compression** for state synchronization
5. **Conflict resolution** for concurrent modifications

---

## 2. Architecture Overview

### 2.1 System Components

```
┌─────────────────────────────────────────────────────────────────────┐
│                    ROANOKE MULTIPLAYER ARCHITECTURE                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   CLIENT LAYER                                                       │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│   │   Client A   │  │   Client B   │  │   Client N   │             │
│   │  (Predict)   │  │  (Predict)   │  │  (Predict)   │             │
│   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘             │
│          │                 │                 │                      │
├──────────┼─────────────────┼─────────────────┼──────────────────────┤
│          │                 │                 │                      │
│   EDGE LAYER              │                 │                      │
│   ┌──────┴─────────────────┴─────────────────┴───────┐             │
│   │              EDGE RELAY NETWORK                   │             │
│   │  (Geographic distribution, UDP relay, DDoS)       │             │
│   └──────────────────────┬───────────────────────────┘             │
│                          │                                          │
├──────────────────────────┼──────────────────────────────────────────┤
│                          │                                          │
│   SIMULATION LAYER       │                                          │
│   ┌──────────────────────┴───────────────────────────┐             │
│   │              WORLD COORDINATOR                    │             │
│   └──┬───────────────┬───────────────┬───────────────┘             │
│      │               │               │                              │
│   ┌──┴────┐      ┌───┴───┐      ┌────┴───┐                         │
│   │ Zone  │      │ Zone  │      │ Zone   │                         │
│   │Server │      │Server │      │Server  │                         │
│   │ (0,0) │      │ (1,0) │      │ (0,1)  │                         │
│   └───────┘      └───────┘      └────────┘                         │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   PERSISTENCE LAYER                                                  │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │                 DISTRIBUTED DATABASE                         │   │
│   │   (World state, player data, modifications)                  │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Zone Server Model

The world is divided into zones, each managed by a dedicated server process:

```rust
pub struct ZoneServer {
    zone_id: ZoneId,
    bounds: AABB,
    entities: EntityStore,
    terrain_modifications: ModificationLog,
    connected_clients: HashMap<PlayerId, ClientConnection>,
    neighbor_zones: Vec<ZoneConnection>,
}

pub struct ZoneId {
    x: i32,
    z: i32,
    layer: u8,  // For vertical stacking (caves, etc.)
}

impl ZoneServer {
    pub const ZONE_SIZE: f32 = 512.0;  // meters
    pub const MAX_ENTITIES: usize = 10_000;
    pub const MAX_PLAYERS: usize = 200;

    pub fn tick(&mut self, delta: f32) {
        // 1. Process incoming client inputs
        self.process_client_inputs();

        // 2. Run simulation
        self.simulate(delta);

        // 3. Handle zone transitions
        self.handle_transitions();

        // 4. Synchronize with neighbors
        self.sync_neighbors();

        // 5. Send state updates to clients
        self.broadcast_state();
    }
}
```

### 2.3 World Coordinator

The coordinator manages zone servers and handles global concerns:

```rust
pub struct WorldCoordinator {
    zones: HashMap<ZoneId, ZoneServerHandle>,
    player_locations: HashMap<PlayerId, ZoneId>,
    global_events: EventQueue,
    load_balancer: LoadBalancer,
}

impl WorldCoordinator {
    pub fn spawn_zone(&mut self, zone_id: ZoneId) -> Result<ZoneServerHandle> {
        // Find least-loaded server machine
        let machine = self.load_balancer.select_machine()?;

        // Spawn zone server process
        let handle = machine.spawn_zone(zone_id)?;

        // Load persisted modifications from database
        handle.load_modifications(&self.database)?;

        self.zones.insert(zone_id, handle);
        Ok(handle)
    }

    pub fn handle_player_transition(
        &mut self,
        player_id: PlayerId,
        from_zone: ZoneId,
        to_zone: ZoneId,
    ) {
        // Ensure target zone is running
        if !self.zones.contains_key(&to_zone) {
            self.spawn_zone(to_zone);
        }

        // Transfer player entity
        let entity_data = self.zones[&from_zone].extract_player(player_id);
        self.zones[&to_zone].insert_player(player_id, entity_data);

        // Update tracking
        self.player_locations.insert(player_id, to_zone);
    }
}
```

---

## 3. Client-Side Prediction

### 3.1 Input Prediction

Clients predict their movement locally for responsive controls:

```rust
pub struct PredictionState {
    pending_inputs: VecDeque<TimestampedInput>,
    last_confirmed_tick: u64,
    predicted_state: PlayerState,
    confirmed_state: PlayerState,
}

impl PredictionState {
    pub fn apply_input(&mut self, input: PlayerInput, tick: u64) {
        // Store for reconciliation
        self.pending_inputs.push_back(TimestampedInput { input, tick });

        // Apply to predicted state
        self.predicted_state.apply_movement(input);
    }

    pub fn reconcile(&mut self, server_state: PlayerState, server_tick: u64) {
        self.confirmed_state = server_state.clone();
        self.last_confirmed_tick = server_tick;

        // Discard inputs before server tick
        while let Some(front) = self.pending_inputs.front() {
            if front.tick <= server_tick {
                self.pending_inputs.pop_front();
            } else {
                break;
            }
        }

        // Re-apply remaining inputs to server state
        self.predicted_state = server_state;
        for input in &self.pending_inputs {
            self.predicted_state.apply_movement(input.input);
        }
    }
}
```

### 3.2 Entity Interpolation

Remote entities are interpolated between known states:

```rust
pub struct InterpolationBuffer {
    states: VecDeque<(f64, EntityState)>,
    interpolation_delay: f64,  // Typically 100ms
}

impl InterpolationBuffer {
    pub fn add_state(&mut self, timestamp: f64, state: EntityState) {
        self.states.push_back((timestamp, state));

        // Keep only recent history
        while self.states.len() > 20 {
            self.states.pop_front();
        }
    }

    pub fn interpolate(&self, render_time: f64) -> EntityState {
        let target_time = render_time - self.interpolation_delay;

        // Find surrounding states
        let (before, after) = self.find_surrounding_states(target_time);

        match (before, after) {
            (Some(b), Some(a)) => {
                let t = (target_time - b.0) / (a.0 - b.0);
                EntityState::lerp(&b.1, &a.1, t as f32)
            }
            (Some(b), None) => {
                // Extrapolate if no future state
                b.1.extrapolate(target_time - b.0)
            }
            _ => EntityState::default(),
        }
    }
}
```

---

## 4. Interest Management

### 4.1 Relevance Calculation

Not all entities are relevant to all clients:

```rust
pub struct InterestManager {
    player_interests: HashMap<PlayerId, InterestSet>,
}

pub struct InterestSet {
    fully_relevant: HashSet<EntityId>,      // Full updates
    partially_relevant: HashSet<EntityId>,   // Reduced updates
    irrelevant: HashSet<EntityId>,           // No updates
}

impl InterestManager {
    pub fn calculate_interest(
        &self,
        player: &Player,
        entity: &Entity,
    ) -> InterestLevel {
        let distance = player.position.distance(entity.position);
        let in_view = player.frustum.contains(entity.bounds);
        let importance = entity.importance();

        // Base interest on distance
        let distance_score = 1.0 - (distance / MAX_INTEREST_DISTANCE).min(1.0);

        // Boost for entities in view
        let view_boost = if in_view { 0.3 } else { 0.0 };

        // Boost for important entities (other players, objectives)
        let importance_boost = importance * 0.2;

        let score = distance_score + view_boost + importance_boost;

        if score > 0.7 {
            InterestLevel::Full
        } else if score > 0.3 {
            InterestLevel::Partial
        } else {
            InterestLevel::None
        }
    }
}
```

### 4.2 Update Frequency Scaling

Update rates scale with interest level:

| Interest Level | Update Rate | Data Included |
|----------------|-------------|---------------|
| Full | 20 Hz | All state |
| Partial | 5 Hz | Position, essential state |
| None | 0 Hz | Nothing (culled) |
| Critical | 60 Hz | Combat participants |

```rust
pub fn should_send_update(
    entity: &Entity,
    interest: InterestLevel,
    last_update: Instant,
) -> bool {
    let min_interval = match interest {
        InterestLevel::Critical => Duration::from_millis(16),
        InterestLevel::Full => Duration::from_millis(50),
        InterestLevel::Partial => Duration::from_millis(200),
        InterestLevel::None => return false,
    };

    last_update.elapsed() >= min_interval
}
```

---

## 5. State Synchronization

### 5.1 Delta Compression

Only changed state is transmitted:

```rust
pub struct DeltaEncoder {
    baseline: EntityState,
    field_versions: HashMap<FieldId, u32>,
}

impl DeltaEncoder {
    pub fn encode_delta(&mut self, current: &EntityState) -> DeltaPacket {
        let mut changes = Vec::new();

        // Compare each field
        if current.position != self.baseline.position {
            changes.push(FieldDelta::Position(current.position));
        }
        if current.rotation != self.baseline.rotation {
            changes.push(FieldDelta::Rotation(current.rotation));
        }
        if current.health != self.baseline.health {
            changes.push(FieldDelta::Health(current.health));
        }
        // ... other fields

        // Update baseline
        self.baseline = current.clone();

        DeltaPacket {
            entity_id: current.id,
            sequence: self.next_sequence(),
            changes,
        }
    }

    pub fn encode_full(&mut self, current: &EntityState) -> FullStatePacket {
        self.baseline = current.clone();
        FullStatePacket {
            entity_id: current.id,
            sequence: self.next_sequence(),
            state: current.clone(),
        }
    }
}
```

### 5.2 Quantization

Floating-point values are quantized for bandwidth efficiency:

```rust
pub fn quantize_position(pos: Vec3) -> QuantizedPosition {
    QuantizedPosition {
        // 20 bits per axis = ~1mm precision over 1km
        x: ((pos.x + 500.0) * 1000.0) as u32,
        y: ((pos.y + 500.0) * 1000.0) as u32,
        z: ((pos.z + 500.0) * 1000.0) as u32,
    }
}

pub fn quantize_rotation(rot: Quat) -> QuantizedRotation {
    // Smallest-three encoding: 2 bits for largest component, 10 bits for others
    let (largest_idx, _) = rot.as_array()
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
        .unwrap();

    let mut components = [0u16; 3];
    let mut j = 0;
    for i in 0..4 {
        if i != largest_idx {
            // Map [-1/√2, 1/√2] to [0, 1023]
            components[j] = ((rot.as_array()[i] * 0.7071 + 0.5) * 1023.0) as u16;
            j += 1;
        }
    }

    QuantizedRotation {
        largest: largest_idx as u8,
        a: components[0],
        b: components[1],
        c: components[2],
    }
}
```

### 5.3 Reliable vs Unreliable Channels

Different data uses appropriate transport:

| Data Type | Channel | Ordering | Reliability |
|-----------|---------|----------|-------------|
| Position/Rotation | Unreliable | None | Best-effort |
| Animation State | Unreliable Sequenced | Sequenced | Best-effort |
| Health/Status | Reliable Sequenced | Sequenced | Guaranteed |
| Chat/Commands | Reliable Ordered | Ordered | Guaranteed |
| World Modification | Reliable Ordered | Ordered | Guaranteed |

```rust
pub enum NetworkChannel {
    UnreliableUnordered,    // Latest-only position
    UnreliableSequenced,    // Animation states
    ReliableSequenced,      // Status changes
    ReliableOrdered,        // Critical events
}

impl NetworkChannel {
    pub fn send(&self, packet: &[u8], connection: &Connection) -> Result<()> {
        match self {
            NetworkChannel::UnreliableUnordered => {
                connection.send_unreliable(packet)
            }
            NetworkChannel::UnreliableSequenced => {
                connection.send_unreliable_sequenced(packet)
            }
            NetworkChannel::ReliableSequenced => {
                connection.send_reliable_sequenced(packet)
            }
            NetworkChannel::ReliableOrdered => {
                connection.send_reliable_ordered(packet)
            }
        }
    }
}
```

---

## 6. World Modification

### 6.1 Building System Synchronization

Player constructions must be synchronized reliably:

```rust
pub struct BuildAction {
    player_id: PlayerId,
    action_type: BuildActionType,
    position: Vec3i,
    block_type: BlockType,
    timestamp: u64,
    signature: ActionSignature,  // Prevent tampering
}

pub enum BuildActionType {
    Place,
    Remove,
    Modify,
}

impl ZoneServer {
    pub fn process_build_action(&mut self, action: BuildAction) -> BuildResult {
        // Verify player can perform action
        if !self.verify_build_permission(&action) {
            return BuildResult::Denied(DenyReason::NoPermission);
        }

        // Check placement validity
        if !self.is_valid_placement(&action) {
            return BuildResult::Denied(DenyReason::InvalidPlacement);
        }

        // Apply modification
        self.terrain_modifications.apply(&action);

        // Persist to database (async)
        self.database.queue_modification(action.clone());

        // Broadcast to interested clients
        self.broadcast_to_zone(BuildEvent::Applied(action));

        BuildResult::Success
    }
}
```

### 6.2 Conflict Resolution

Concurrent modifications use last-write-wins with vector clocks:

```rust
pub struct ModificationLog {
    modifications: BTreeMap<VectorClock, Modification>,
    local_clock: VectorClock,
}

impl ModificationLog {
    pub fn apply(&mut self, modification: Modification) {
        let clock = self.local_clock.increment();

        // Check for conflicts
        if let Some(conflict) = self.find_conflict(&modification) {
            // Resolve using deterministic rule
            let winner = self.resolve_conflict(&modification, &conflict);
            if winner != modification {
                return;  // Our modification loses
            }
        }

        self.modifications.insert(clock, modification);
    }

    fn resolve_conflict(
        &self,
        a: &Modification,
        b: &Modification,
    ) -> &Modification {
        // Deterministic: higher player ID wins ties
        if a.timestamp != b.timestamp {
            if a.timestamp > b.timestamp { a } else { b }
        } else {
            if a.player_id > b.player_id { a } else { b }
        }
    }
}
```

---

## 7. Zone Transitions

### 7.1 Seamless Handoff

Players transition between zones without loading screens:

```rust
pub struct ZoneTransition {
    overlap_distance: f32,  // Pre-load zone data
    handoff_distance: f32,  // Authority transfer point
}

impl Client {
    pub fn check_zone_transition(&mut self) {
        let current_zone = self.current_zone;
        let position = self.player.position;

        // Check if approaching zone boundary
        for neighbor in current_zone.neighbors() {
            let distance_to_boundary = neighbor.distance_to_boundary(position);

            if distance_to_boundary < self.overlap_distance {
                // Start receiving updates from neighbor zone
                self.subscribe_to_zone(neighbor);
            }

            if distance_to_boundary < 0.0 {
                // Crossed into neighbor zone
                self.transition_to_zone(neighbor);
            }
        }
    }

    fn transition_to_zone(&mut self, new_zone: ZoneId) {
        // Notify old zone of departure
        self.current_zone_connection.send(PlayerDeparting {
            player_id: self.player_id,
            destination: new_zone,
        });

        // Switch primary zone
        self.current_zone = new_zone;

        // Confirm arrival to new zone
        self.zone_connections[&new_zone].send(PlayerArriving {
            player_id: self.player_id,
            state: self.player.state.clone(),
        });

        // Clean up old subscriptions
        self.cleanup_distant_zones();
    }
}
```

### 7.2 Entity Handoff

Entities (NPCs, animals) also transition between zones:

```rust
impl ZoneServer {
    pub fn handle_entity_transition(&mut self, entity: Entity) {
        let target_zone = self.calculate_target_zone(entity.position);

        if target_zone != self.zone_id {
            // Serialize entity state
            let entity_data = entity.serialize();

            // Send to coordinator for handoff
            self.coordinator.transfer_entity(
                entity.id,
                self.zone_id,
                target_zone,
                entity_data,
            );

            // Remove from local simulation
            self.entities.remove(entity.id);
        }
    }
}
```

---

## 8. Security Considerations

### 8.1 Authority Model

The server maintains authority over critical state:

| State Type | Authority | Validation |
|------------|-----------|------------|
| Movement | Client-predicted, Server-validated | Speed limits, collision |
| Combat | Server-authoritative | Hit detection server-side |
| Inventory | Server-authoritative | No client modification |
| Building | Server-authoritative | Permission checking |
| Chat | Server-authoritative | Content filtering |

### 8.2 Anti-Cheat Integration

```rust
pub struct InputValidator {
    max_speed: f32,
    max_turn_rate: f32,
    action_rate_limits: HashMap<ActionType, RateLimit>,
}

impl InputValidator {
    pub fn validate(&self, input: &PlayerInput, player: &Player) -> ValidationResult {
        // Check movement speed
        let requested_speed = input.movement.length();
        if requested_speed > self.max_speed {
            return ValidationResult::Invalid(Reason::SpeedExceeded);
        }

        // Check action rate limits
        if let Some(limit) = self.action_rate_limits.get(&input.action_type) {
            if !limit.check(&player.action_history) {
                return ValidationResult::Invalid(Reason::RateLimitExceeded);
            }
        }

        // Check for impossible state transitions
        if !self.is_valid_state_transition(&player.state, &input) {
            return ValidationResult::Invalid(Reason::InvalidTransition);
        }

        ValidationResult::Valid
    }
}
```

---

## 9. Bandwidth Analysis

### 9.1 Per-Player Bandwidth

**Typical scenario: 50 relevant entities**

| Data Type | Size | Frequency | Bandwidth |
|-----------|------|-----------|-----------|
| Own state | 64 B | 60 Hz | 3.8 KB/s |
| Full entities (10) | 64 B | 20 Hz | 12.8 KB/s |
| Partial entities (30) | 24 B | 5 Hz | 3.6 KB/s |
| Events/chat | Variable | Variable | ~1 KB/s |
| **Total Download** | | | **~21 KB/s** |
| **Total Upload** | | | **~5 KB/s** |

### 9.2 Zone Server Bandwidth

**Scenario: 200 players, 1000 entities**

| Direction | Calculation | Bandwidth |
|-----------|-------------|-----------|
| To Clients | 200 × 21 KB/s | 4.2 MB/s |
| From Clients | 200 × 5 KB/s | 1.0 MB/s |
| Inter-Zone | ~500 KB/s | 0.5 MB/s |
| **Total** | | **~6 MB/s** |

---

## 10. Performance Benchmarks

### 10.1 Latency Measurements

| Metric | Target | Achieved |
|--------|--------|----------|
| Client → Server (edge) | <50ms | 12ms (median) |
| State broadcast latency | <100ms | 35ms (median) |
| Zone transition time | <500ms | 180ms (median) |
| Modification persistence | <1s | 200ms (median) |

### 10.2 Scalability Tests

| Players per Zone | Tick Rate | CPU Usage | Bandwidth |
|------------------|-----------|-----------|-----------|
| 50 | 60 Hz | 15% | 1.5 MB/s |
| 100 | 60 Hz | 35% | 3.2 MB/s |
| 200 | 60 Hz | 75% | 6.1 MB/s |
| 300 | 40 Hz | 90% | 7.8 MB/s |

*Test hardware: 8-core server, 32GB RAM, 1Gbps network*

---

## 11. Conclusion

The Roanoke multiplayer architecture demonstrates that persistent, modifiable virtual worlds can scale to thousands of concurrent players while maintaining responsive gameplay. Through careful combination of client-side prediction, interest management, and efficient state synchronization, we achieve the "feel" of a dedicated server with the scale of an MMO.

The system's modular design enables both cloud-hosted worlds and community-run servers, supporting Roanoke's mission to empower players with ownership of their experiences.

---

*© 2025 Roanoke Interactive, Inc. | Technical Whitepaper WP-002*
