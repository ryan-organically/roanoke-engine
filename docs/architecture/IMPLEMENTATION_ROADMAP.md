# Implementation Roadmap: Primordia to Endgame

## Philosophy

**Endgame specs are north stars, not starting points.**

Each feature must:
1. Be implementable NOW with current engine
2. Provide immediate value
3. Be a foundation for the next step
4. Not require rework when endgame arrives

---

## The Ladder

```
ENDGAME (Year 2+)
│ Cheat-proof architecture
│ Distributed compute network
│ Post-quantum cryptography
│ Self-sustaining economy
│
├── MILESTONE 4: Trustless Wagered Play
│   │ Full determinism
│   │ On-chain settlement
│   │ Fraud proofs
│   │
│   ├── MILESTONE 3: Ranked Competitive
│   │   │ Server-authoritative state
│   │   │ Replay/dispute system
│   │   │ Skill-based matchmaking
│   │   │
│   │   ├── MILESTONE 2: Ping-Fair Combat
│   │   │   │ Input lockstep
│   │   │   │ Simultaneous resolution
│   │   │   │ No reaction advantage
│   │   │   │
│   │   │   ├── MILESTONE 1: Basic Multiplayer ◄── WE ARE HERE
│   │   │   │   │ WebSocket connectivity
│   │   │   │   │ Position sync
│   │   │   │   │ See other players
│   │   │   │   │
│   │   │   │   └── CURRENT: Single Player
```

---

## Milestone 1: Basic Multiplayer (DONE - Scaffolded)

**Status:** Network module created, needs integration

**What exists:**
- WebSocket server/client
- Position sync messages
- NetworkManager API

**Immediate next:**
- Integrate into main.rs
- Test with 2 players on LAN
- Render other players as capsules

---

## Milestone 2: Ping-Fair Combat

**The Problem:**

```
Traditional netcode (ping advantage):

  Player A (20ms ping)          Player B (150ms ping)
  ─────────────────────────────────────────────────────

  T=0:    A sees B, shoots
  T=20:   Server receives A's shot
  T=20:   Server sends "you're hit" to B
  T=170:  B finally sees they're dead

  B never had a chance to react.
  A's 130ms advantage is insurmountable.

  This is why competitive players obsess over ping.
```

**The Solution: Input Lockstep**

```
Ping-fair combat (simultaneous resolution):

  Player A (20ms ping)          Player B (150ms ping)
  ─────────────────────────────────────────────────────

  TURN N (Input Window: 200ms)
  ───────────────────────────
  T=0:     Window opens. Both see current state.
  T=20:    A decides to shoot, sends input
  T=150:   B decides to dodge, sends input
  T=200:   Window closes. Both inputs locked.

  RESOLUTION
  ──────────
  T=200:   Server has both inputs
  T=200:   Resolves simultaneously:
           - A shot at position X
           - B dodged to position Y
           - Did shot hit where B WAS or where B IS?
           - Answer: Neither knew the other's action

  T=220:   A sees result
  T=350:   B sees result

  Both had equal 200ms to make their decision.
  Ping only affects when you SEE results, not outcomes.
```

### Implementation: Combat Tick System

```rust
/// Combat operates on discrete ticks, not continuous time
pub struct CombatTick {
    pub tick_number: u64,
    pub inputs: HashMap<PlayerId, CombatInput>,
    pub state_before: CombatState,
    pub state_after: CombatState,
}

/// Combat input for a single tick
#[derive(Clone, Serialize, Deserialize)]
pub struct CombatInput {
    pub player_id: PlayerId,
    pub tick: u64,

    /// Movement intention
    pub movement: MovementInput,

    /// Combat action (if any)
    pub action: Option<CombatAction>,

    /// When this input was created (client time)
    pub client_timestamp: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MovementInput {
    /// Direction relative to facing
    Move { forward: i8, strafe: i8 },  // -1, 0, or 1

    /// Dodge in direction (costs stamina)
    Dodge { direction: DodgeDirection },

    /// Stay still
    Hold,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum CombatAction {
    /// Basic attack with current weapon
    Attack { aim_direction: [i16; 2] },

    /// Block/parry (timing matters)
    Block,

    /// Use ability
    Ability { ability_id: u32, target: ActionTarget },

    /// Use item
    UseItem { slot: u8 },
}
```

### The Tick Loop

```rust
/// Server-side combat tick manager
pub struct CombatTickManager {
    pub current_tick: u64,
    pub tick_duration: Duration,  // 200ms = 5 ticks/sec
    pub input_buffer: HashMap<u64, HashMap<PlayerId, CombatInput>>,
    pub participants: Vec<PlayerId>,
}

impl CombatTickManager {
    /// Called when input arrives from player
    pub fn receive_input(&mut self, input: CombatInput) {
        // Input is for which tick?
        let target_tick = self.current_tick + 1;  // Next tick

        // Buffer it
        self.input_buffer
            .entry(target_tick)
            .or_default()
            .insert(input.player_id, input);
    }

    /// Called every tick_duration
    pub fn process_tick(&mut self, state: &mut CombatState) -> TickResult {
        self.current_tick += 1;

        // Get all inputs for this tick (or empty/default for missing)
        let inputs = self.collect_inputs_for_tick(self.current_tick);

        // Resolve simultaneously
        let result = resolve_tick(state, &inputs);

        // Broadcast result to all
        result
    }

    fn collect_inputs_for_tick(&mut self, tick: u64) -> HashMap<PlayerId, CombatInput> {
        let mut inputs = self.input_buffer.remove(&tick).unwrap_or_default();

        // Fill in missing inputs with "no action"
        for player in &self.participants {
            inputs.entry(*player).or_insert_with(|| CombatInput {
                player_id: *player,
                tick,
                movement: MovementInput::Hold,
                action: None,
                client_timestamp: 0.0,
            });
        }

        inputs
    }
}
```

### Simultaneous Resolution

```rust
/// Resolve all combat actions simultaneously
pub fn resolve_tick(
    state: &mut CombatState,
    inputs: &HashMap<PlayerId, CombatInput>,
) -> TickResult {
    let mut result = TickResult::new();

    // PHASE 1: Collect all intended movements
    let mut intended_positions: HashMap<PlayerId, Vec3> = HashMap::new();
    for (player_id, input) in inputs {
        let current_pos = state.get_position(*player_id);
        let intended = calculate_movement(current_pos, &input.movement, state);
        intended_positions.insert(*player_id, intended);
    }

    // PHASE 2: Collect all attacks (before movement resolves)
    let mut attacks: Vec<Attack> = Vec::new();
    for (player_id, input) in inputs {
        if let Some(CombatAction::Attack { aim_direction }) = &input.action {
            let attacker_pos = state.get_position(*player_id);
            attacks.push(Attack {
                attacker: *player_id,
                origin: attacker_pos,
                direction: aim_direction_to_vec(*aim_direction),
                weapon: state.get_equipped_weapon(*player_id),
            });
        }
    }

    // PHASE 3: Collect all blocks
    let blockers: HashSet<PlayerId> = inputs.iter()
        .filter(|(_, input)| matches!(input.action, Some(CombatAction::Block)))
        .map(|(id, _)| *id)
        .collect();

    // PHASE 4: Resolve attacks against CURRENT positions (before movement)
    // This is key: you attack where they WERE, they dodge where they WILL BE
    // Neither knew the other's action
    for attack in &attacks {
        for (target_id, input) in inputs {
            if *target_id == attack.attacker { continue; }

            let target_pos = state.get_position(*target_id);  // Position at START of tick
            let target_intended = intended_positions[target_id];  // Position at END of tick

            // Did attack connect?
            let hit = check_attack_hit(attack, target_pos, &state);

            if hit {
                // Was target blocking?
                if blockers.contains(target_id) {
                    result.add_event(CombatEvent::Blocked {
                        attacker: attack.attacker,
                        blocker: *target_id,
                    });
                } else {
                    // Was target dodging?
                    let was_dodging = matches!(input.movement, MovementInput::Dodge { .. });

                    if was_dodging {
                        // Dodge vs Attack: Timing-based
                        // Both committed simultaneously, so it's a fair contest
                        // Use deterministic resolution based on inputs
                        let dodge_success = resolve_dodge_vs_attack(input, attack);

                        if dodge_success {
                            result.add_event(CombatEvent::Dodged {
                                attacker: attack.attacker,
                                dodger: *target_id,
                            });
                        } else {
                            let damage = calculate_damage(attack, state);
                            result.add_event(CombatEvent::Hit {
                                attacker: attack.attacker,
                                target: *target_id,
                                damage,
                            });
                        }
                    } else {
                        // Not blocking, not dodging = hit
                        let damage = calculate_damage(attack, state);
                        result.add_event(CombatEvent::Hit {
                            attacker: attack.attacker,
                            target: *target_id,
                            damage,
                        });
                    }
                }
            }
        }
    }

    // PHASE 5: Apply movements
    for (player_id, intended_pos) in intended_positions {
        state.set_position(player_id, intended_pos);
    }

    // PHASE 6: Apply damage
    for event in &result.events {
        if let CombatEvent::Hit { target, damage, .. } = event {
            state.apply_damage(*target, *damage);
        }
    }

    result
}
```

### Client-Side Experience

```rust
/// Client-side combat input handling
pub struct CombatInputHandler {
    pub current_tick: u64,
    pub pending_input: Option<CombatInput>,
    pub last_sent_tick: u64,

    /// Buffer for smooth rendering
    pub state_buffer: VecDeque<CombatState>,
}

impl CombatInputHandler {
    /// Called when player presses attack
    pub fn queue_attack(&mut self, aim: [i16; 2]) {
        // Store for next tick submission
        if let Some(input) = &mut self.pending_input {
            input.action = Some(CombatAction::Attack { aim_direction: aim });
        }
    }

    /// Called when movement keys change
    pub fn update_movement(&mut self, forward: i8, strafe: i8) {
        if let Some(input) = &mut self.pending_input {
            input.movement = MovementInput::Move { forward, strafe };
        }
    }

    /// Called each frame - handles tick timing
    pub fn update(&mut self, dt: f32, network: &NetworkManager) {
        // Check if it's time to send input for next tick
        if self.should_send_input() {
            let input = self.pending_input.take().unwrap_or_default();
            network.send_combat_input(input);
            self.last_sent_tick = self.current_tick + 1;

            // Prepare input buffer for next tick
            self.pending_input = Some(CombatInput::default());
        }

        // Interpolate rendering between confirmed states
        self.interpolate_state(dt);
    }
}
```

### Why This Works

```
┌─────────────────────────────────────────────────────────────────┐
│                   PING-FAIR COMBAT                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Traditional (ping advantage):                                  │
│  ├── Fast ping = see and react before slow ping                │
│  ├── Shot hits where target IS                                 │
│  └── Network speed = combat speed                               │
│                                                                  │
│  Lockstep (ping-fair):                                          │
│  ├── All inputs collected before resolution                    │
│  ├── Shot hits where target WAS GOING TO BE                    │
│  ├── Neither knew the other's action                           │
│  └── Network speed = visual latency only                        │
│                                                                  │
│  Key insight:                                                   │
│  "Attack" and "Dodge" are SIMULTANEOUS decisions               │
│  Not "I saw you attack so I dodged"                            │
│  But "We both committed to our actions blindly"                 │
│                                                                  │
│  This is like chess boxing:                                     │
│  Both fighters commit to their punch at the same time          │
│  Whoever reads the opponent better wins                        │
│  Not whoever has faster internet                                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Practical Tick Rate Considerations

```
TICK RATE OPTIONS:
─────────────────
5 ticks/sec  (200ms windows) - Very strategic, card-game feel
10 ticks/sec (100ms windows) - Tactical, souls-like combat
20 ticks/sec (50ms windows)  - Action, hunting feel
30 ticks/sec (33ms windows)  - Fast action (pushing latency limits)

FOR ROANOKE:
────────────
Combat encounters: 10-15 ticks/sec
├── Bow draw time: ~1 second (10-15 ticks to full draw)
├── Melee swing: ~0.5 seconds (5-7 ticks)
├── Dodge roll: ~0.3 seconds (3-5 ticks)
└── Feels deliberate and tactical, not twitchy

Exploration/non-combat: Continuous (standard netcode)
├── Position sync 20Hz
├── No combat lockstep needed
└── Responsive movement feel
```

---

## Implementation Steps

### Step 1: Combat State Separation

```rust
/// Separate combat state from exploration state
pub struct GameState {
    /// Standard exploration (continuous)
    pub exploration: ExplorationState,

    /// Tick-based combat (when in combat)
    pub combat: Option<CombatInstance>,
}

pub struct CombatInstance {
    pub participants: Vec<PlayerId>,
    pub tick_manager: CombatTickManager,
    pub state: CombatState,
    pub started_at: Instant,
}

/// Transition into combat mode
impl GameState {
    pub fn enter_combat(&mut self, participants: Vec<PlayerId>) {
        self.combat = Some(CombatInstance {
            participants: participants.clone(),
            tick_manager: CombatTickManager::new(participants),
            state: CombatState::from_exploration(&self.exploration),
            started_at: Instant::now(),
        });
    }

    pub fn exit_combat(&mut self) {
        if let Some(combat) = self.combat.take() {
            // Apply combat results back to exploration state
            self.exploration.apply_combat_results(&combat.state);
        }
    }
}
```

### Step 2: Network Integration

```rust
/// New message types for combat
#[derive(Serialize, Deserialize)]
pub enum CombatNetMessage {
    /// Enter combat with these players
    CombatStart {
        instance_id: u64,
        participants: Vec<PlayerId>,
        initial_state: CombatState,
    },

    /// Player's input for a tick
    TickInput {
        instance_id: u64,
        input: CombatInput,
    },

    /// Server's resolution of a tick
    TickResult {
        instance_id: u64,
        tick: u64,
        events: Vec<CombatEvent>,
        new_state: CombatState,
    },

    /// Combat ended
    CombatEnd {
        instance_id: u64,
        result: CombatOutcome,
    },
}
```

### Step 3: Input Buffering

```rust
/// Handle network jitter with input buffer
pub struct InputBuffer {
    /// Inputs waiting to be processed
    pending: VecDeque<CombatInput>,

    /// How many ticks ahead to buffer (higher = more jitter tolerance, more latency)
    buffer_ticks: u64,
}

impl InputBuffer {
    /// Add input, possibly for future tick
    pub fn add(&mut self, input: CombatInput) {
        // Insert in tick order
        let pos = self.pending.iter()
            .position(|i| i.tick > input.tick)
            .unwrap_or(self.pending.len());
        self.pending.insert(pos, input);
    }

    /// Get inputs for specific tick
    pub fn get_for_tick(&mut self, tick: u64) -> Option<CombatInput> {
        self.pending.iter()
            .position(|i| i.tick == tick)
            .map(|i| self.pending.remove(i).unwrap())
    }
}
```

---

## Testing Plan

### Local Testing (Week 1)

```bash
# Two terminals, same machine
Terminal 1: cargo run -- --host 7878
Terminal 2: cargo run -- --join 127.0.0.1:7878

# Test: Both players attack simultaneously
# Expected: Neither has advantage, hits resolve fairly
```

### Simulated Latency Testing (Week 2)

```bash
# Use network simulation tool (e.g., clumsy on Windows, tc on Linux)
# Add 150ms latency to one client

# Test: High-latency player should have equal combat outcomes
# Expected: Visual delay only, no combat disadvantage
```

### Metrics to Track

```rust
pub struct CombatFairnessMetrics {
    /// Win rate by ping bracket
    pub wins_by_ping: HashMap<PingBracket, WinStats>,

    /// Average "first hit" by ping (should be equal)
    pub first_hit_by_ping: HashMap<PingBracket, f32>,

    /// Input latency (time from keypress to server receipt)
    pub input_latencies: Vec<Duration>,

    /// Tick desync events
    pub desync_count: u64,
}

pub enum PingBracket {
    Low,     // 0-50ms
    Medium,  // 50-100ms
    High,    // 100-200ms
    VeryHigh, // 200ms+
}
```

---

## What This Enables

```
MILESTONE 2 COMPLETE → Unlocks:
├── Fair PvP encounters during hunting
├── Competitive arena modes
├── Tournament-viable combat
└── Foundation for ranked play

DOES NOT REQUIRE:
├── Full determinism (that's Milestone 4)
├── Cryptographic commitments (that's endgame)
├── Blockchain anything
└── Complex anti-cheat

IS A STEPPING STONE TO:
├── Adding commit-reveal (cryptographic fairness)
├── Server-authoritative validation
├── Eventually: cheat-proof architecture
```

---

## Files to Create

```
roanoke_game/src/combat/
├── mod.rs
├── tick_manager.rs      // Tick timing and synchronization
├── input.rs             // Combat input types
├── resolution.rs        // Simultaneous resolution logic
├── state.rs             // Combat-specific state
└── network.rs           // Combat message handling
```

---

*This is the immediate next step. Ping-fair combat is achievable with current engine, provides real value, and builds toward endgame.*
