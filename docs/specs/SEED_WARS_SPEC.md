# Seed Wars - Wagered Survival Mode Specification

## Roanoke Engine - High-Stakes Battle Royale

**Status:** DRAFT
**Version:** 0.1.0

---

## Overview

Seed Wars is a 50-player, wagered survival mode combining Hunger Games tension with Rust's brutal resource competition. Players stake $5 USD or 5 Roanoke Coins to enter a procedurally-seeded server. Last survivor(s) take the pot.

### Design Pillars

1. **Real Stakes** - Entry fee makes every decision matter
2. **Extended Tension** - Multi-hour matches, not 20-minute sprints
3. **Disheartening Loss** - Death means losing your stake AND time invested
4. **Emergent Drama** - Alliances form, break, betray
5. **Seed Fairness** - Same procedural seed = same opportunity for all

---

## Core Loop

```
┌─────────────────────────────────────────────────────────────────┐
│                         SEED WARS FLOW                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   QUEUE (Lobby)                                                  │
│   ├── 50 players pay $5/5 coins                                 │
│   ├── Pot: $250 (minus 10% house rake = $225 to winners)        │
│   └── Seed revealed when lobby fills                            │
│                                                                  │
│   SPAWN (Day 0)                                                  │
│   ├── All players spawn simultaneously                          │
│   ├── Random positions around map perimeter                     │
│   ├── Naked, no items                                           │
│   └── 10-minute peace period (no PvP damage)                    │
│                                                                  │
│   SURVIVAL (Days 1-7)                                           │
│   ├── Gather, craft, build shelter                              │
│   ├── Hunt animals for food                                     │
│   ├── Form/break alliances                                      │
│   ├── Full PvP enabled after peace period                       │
│   └── Map shrinks every 24 in-game hours                        │
│                                                                  │
│   ENDGAME (Final Circle)                                        │
│   ├── Final zone forces confrontation                           │
│   ├── Last player/team standing wins                            │
│   └── Pot distributed based on placement                        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Entry & Wagering

### Queue System

```rust
pub struct SeedWarsQueue {
    pub queue_id: Uuid,
    pub entry_fee: WagerAmount,
    pub players: Vec<QueuedPlayer>,
    pub max_players: u32,          // 50
    pub min_players: u32,          // 40 (game starts with 40-50)
    pub queue_timeout: Duration,   // 15 minutes max wait
    pub seed: Option<u64>,         // Revealed when full
}

#[derive(Clone, Copy)]
pub enum WagerAmount {
    FiatUSD(u32),    // $5
    RoanokeCoin(u32), // 5 coins
}

impl WagerAmount {
    pub fn standard() -> Self {
        WagerAmount::FiatUSD(5)
    }

    pub fn to_pot_value(&self) -> u32 {
        match self {
            WagerAmount::FiatUSD(v) => *v,
            WagerAmount::RoanokeCoin(v) => *v, // 1 coin = $1 equivalent
        }
    }
}

pub struct QueuedPlayer {
    pub player_id: PlayerId,
    pub wager: WagerAmount,
    pub queue_time: Instant,
    pub escrow_tx: TransactionId, // Funds held in escrow
}
```

### Pot Distribution

```rust
pub struct PotDistribution {
    pub total_pot: u32,           // $250 for 50 players
    pub house_rake: f32,          // 10%
    pub player_pot: u32,          // $225 distributed to winners
}

impl PotDistribution {
    pub fn calculate(players: u32, entry_fee: u32) -> Self {
        let total = players * entry_fee;
        let rake = (total as f32 * 0.10) as u32;
        Self {
            total_pot: total,
            house_rake: 0.10,
            player_pot: total - rake,
        }
    }

    /// Winner-take-all for solo queue
    pub fn solo_payout(&self, placement: u32) -> u32 {
        match placement {
            1 => self.player_pot,  // $225
            _ => 0,
        }
    }

    /// Top-3 split for team queue
    pub fn team_payout(&self, placement: u32) -> u32 {
        match placement {
            1 => (self.player_pot as f32 * 0.60) as u32,  // $135
            2 => (self.player_pot as f32 * 0.25) as u32,  // $56
            3 => (self.player_pot as f32 * 0.15) as u32,  // $34
            _ => 0,
        }
    }
}
```

---

## Match Phases

### Phase 0: Spawn (10 minutes real-time)

```rust
pub struct SpawnPhase {
    pub duration: Duration,        // 10 minutes
    pub pvp_enabled: bool,         // false
    pub spawn_ring_radius: f32,    // 2000m from center
    pub spawn_spread: f32,         // ~125m between players
}

impl SpawnPhase {
    pub fn spawn_positions(player_count: u32, seed: u64) -> Vec<Vec3> {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut positions = Vec::new();

        let angle_step = (2.0 * PI) / player_count as f32;

        for i in 0..player_count {
            let base_angle = angle_step * i as f32;
            // Slight randomization to prevent camping known spots
            let angle = base_angle + rng.gen_range(-0.1..0.1);
            let radius = 2000.0 + rng.gen_range(-100.0..100.0);

            positions.push(Vec3::new(
                angle.cos() * radius,
                0.0, // Ground level
                angle.sin() * radius,
            ));
        }

        positions
    }
}
```

### Phase 1-7: Survival (7 in-game days)

```rust
pub struct SurvivalPhase {
    pub current_day: u32,
    pub day_length: Duration,      // 1 hour real-time = 1 in-game day
    pub circle: ShrinkingCircle,
    pub events: Vec<ScheduledEvent>,
}

pub struct ShrinkingCircle {
    pub center: Vec3,
    pub current_radius: f32,
    pub target_radius: f32,
    pub shrink_rate: f32,          // meters per second during shrink
    pub damage_per_second: f32,    // Outside circle damage
}

impl ShrinkingCircle {
    pub fn schedule() -> Vec<CirclePhase> {
        vec![
            // Day 1: Full map
            CirclePhase { day: 1, radius: 2500.0, damage: 1.0 },
            // Day 2: 80% map
            CirclePhase { day: 2, radius: 2000.0, damage: 2.0 },
            // Day 3: 60% map
            CirclePhase { day: 3, radius: 1500.0, damage: 3.0 },
            // Day 4: 40% map
            CirclePhase { day: 4, radius: 1000.0, damage: 5.0 },
            // Day 5: 25% map
            CirclePhase { day: 5, radius: 625.0, damage: 8.0 },
            // Day 6: 10% map
            CirclePhase { day: 6, radius: 250.0, damage: 12.0 },
            // Day 7: Final circle
            CirclePhase { day: 7, radius: 50.0, damage: 20.0 },
        ]
    }
}
```

### Survival Mechanics

```rust
/// Core survival needs
pub struct SurvivalNeeds {
    pub hunger: f32,               // 0.0 = starving, 100.0 = full
    pub thirst: f32,               // 0.0 = dying, 100.0 = hydrated
    pub warmth: f32,               // 0.0 = freezing, 100.0 = warm
    pub fatigue: f32,              // 0.0 = exhausted, 100.0 = rested
}

impl SurvivalNeeds {
    /// Decay rates per in-game hour
    pub const HUNGER_DECAY: f32 = 4.0;    // ~25 hours to starve
    pub const THIRST_DECAY: f32 = 6.0;    // ~17 hours to dehydrate
    pub const WARMTH_DECAY_NIGHT: f32 = 8.0;
    pub const FATIGUE_DECAY: f32 = 2.0;   // Need sleep every ~2 days

    pub fn update(&mut self, dt_hours: f32, conditions: &Conditions) {
        self.hunger -= Self::HUNGER_DECAY * dt_hours;
        self.thirst -= Self::THIRST_DECAY * dt_hours;

        if conditions.is_night && !conditions.has_shelter {
            self.warmth -= Self::WARMTH_DECAY_NIGHT * dt_hours;
        }

        if !conditions.is_sleeping {
            self.fatigue -= Self::FATIGUE_DECAY * dt_hours;
        }

        // Clamp all values
        self.hunger = self.hunger.clamp(0.0, 100.0);
        self.thirst = self.thirst.clamp(0.0, 100.0);
        self.warmth = self.warmth.clamp(0.0, 100.0);
        self.fatigue = self.fatigue.clamp(0.0, 100.0);
    }

    /// Penalties for low needs
    pub fn apply_penalties(&self, player: &mut Player) {
        // Starving: health drain
        if self.hunger < 10.0 {
            player.health -= 2.0; // per tick
        }

        // Dehydrated: stamina penalty
        if self.thirst < 20.0 {
            player.max_stamina *= 0.5;
        }

        // Freezing: health drain + slow
        if self.warmth < 15.0 {
            player.health -= 1.0;
            player.move_speed *= 0.7;
        }

        // Exhausted: aim sway, slow actions
        if self.fatigue < 10.0 {
            player.aim_sway *= 2.0;
            player.action_speed *= 0.6;
        }
    }
}
```

---

## Death & Elimination

### Permadeath

```rust
pub struct DeathEvent {
    pub victim: PlayerId,
    pub killer: Option<PlayerId>,
    pub cause: DeathCause,
    pub position: Vec3,
    pub timestamp: f64,
    pub survival_time: Duration,
    pub placement: u32,            // Out of 50
}

pub enum DeathCause {
    Player { killer_id: PlayerId, weapon: WeaponType },
    Starvation,
    Dehydration,
    Hypothermia,
    Fall,
    Wildlife { species: HostileSpecies },
    CircleDamage,
    Disconnect,  // Treated as death after 5 min
}

impl DeathEvent {
    /// What the dead player sees
    pub fn death_screen(&self) -> DeathScreen {
        DeathScreen {
            message: match &self.cause {
                DeathCause::Player { killer_id, .. } => {
                    format!("Killed by {}", get_player_name(*killer_id))
                }
                DeathCause::Starvation => "You starved to death.".into(),
                DeathCause::CircleDamage => "The storm consumed you.".into(),
                _ => "You died.".into(),
            },
            placement: format!("#{} of 50", self.placement),
            survival_time: format_duration(self.survival_time),
            payout: "$0", // No payout for losers
            spectate_option: true,
            return_to_lobby: true,
        }
    }
}
```

### The Sting of Loss

The goal is to make death *hurt* emotionally:

```rust
/// Post-death experience
pub struct PostDeathFlow {
    pub forced_spectate_time: Duration,  // 30 seconds minimum
    pub show_killer_inventory: bool,      // Salt in the wound
    pub show_killer_health: bool,         // "They only had 12 HP left"
    pub play_death_recap: bool,           // Slow-mo replay
}

impl PostDeathFlow {
    pub fn execute(&self, death: &DeathEvent, ui: &mut UI) {
        // Force them to watch for 30 seconds
        ui.show_death_screen(death);

        if let DeathCause::Player { killer_id, .. } = death.cause {
            // Show what killed them
            ui.show_killer_loadout(killer_id);

            // Show kill feed to all players
            broadcast_kill_notification(death);
        }

        // After forced watch, allow spectate or leave
        ui.enable_spectate_controls();
        ui.show_leave_button(); // "Return to Lobby ($5 lost)"
    }
}
```

---

## Alliance System

Temporary, breakable alliances add psychological depth.

```rust
pub struct Alliance {
    pub id: Uuid,
    pub members: Vec<PlayerId>,
    pub formed_at: f64,
    pub shared_markers: bool,     // See ally positions
    pub friendly_fire: bool,      // Can still betray
}

pub struct AllianceManager {
    pub alliances: HashMap<Uuid, Alliance>,
    pub player_alliance: HashMap<PlayerId, Uuid>,
    pub max_alliance_size: u32,   // 4 players max
}

impl AllianceManager {
    pub fn propose_alliance(&mut self, proposer: PlayerId, target: PlayerId) {
        // Must be within proximity to propose
        // Target gets notification to accept/decline
    }

    pub fn betray(&mut self, betrayer: PlayerId) {
        // Leave alliance silently
        // 30 second delay before they disappear from ally map
        // Creates "I have a bad feeling about this" moments
    }
}

/// Proximity-only alliance invites
pub const ALLIANCE_INVITE_RANGE: f32 = 5.0; // Must be close
```

---

## Loot & Crafting

### Starter Resources (Nothing)

```rust
pub struct PlayerInventory {
    pub slots: [Option<Item>; 20],
    pub equipped: EquippedItems,
}

impl PlayerInventory {
    /// What players spawn with
    pub fn seed_wars_spawn() -> Self {
        Self {
            slots: [None; 20],  // NOTHING
            equipped: EquippedItems {
                weapon: None,
                armor: None,
                tool: None,
            },
        }
    }
}
```

### Scarcity-Focused Loot

```rust
pub struct SeedWarsLootTable {
    pub base_loot_density: f32,    // 0.3 = 30% of normal density
    pub weapon_rarity_mult: f32,   // 0.2 = weapons are rare
    pub food_rarity_mult: f32,     // 0.5 = food is scarce
}

impl SeedWarsLootTable {
    pub fn standard() -> Self {
        Self {
            base_loot_density: 0.3,
            weapon_rarity_mult: 0.2,
            food_rarity_mult: 0.5,
        }
    }
}

/// Point-of-interest loot zones
pub enum LootZone {
    Ruins,        // Medium loot, contested
    Settlement,   // High loot, very contested
    Wilderness,   // Sparse, safe early game
    Cache,        // Rare spawn, excellent loot
}
```

### Crafting Essentials

```rust
/// Minimal crafting for survival
pub enum CraftableItem {
    StoneAxe,          // Gather wood faster
    StoneKnife,        // Process meat, basic weapon
    WoodenSpear,       // Hunting, defense
    Campfire,          // Warmth, cooking
    LeanTo,            // Basic shelter
    WaterContainer,    // Carry water
    Bow,               // Ranged hunting
    Arrows(u32),       // Ammo
    RabbitSnare,       // Passive food
    FishingLine,       // River food
}
```

---

## Map & Seed System

### Procedural Fairness

```rust
pub struct SeedWarsSeed {
    pub seed: u64,
    pub map_size: f32,             // 5000m diameter
    pub biome_distribution: BiomeWeights,
    pub poi_count: u32,            // Points of interest
    pub cache_count: u32,          // Hidden high-loot spots
}

impl SeedWarsSeed {
    /// Seed is revealed only when lobby is full
    /// This prevents "seed sniping" (memorizing good seeds)
    pub fn generate() -> Self {
        Self {
            seed: rand::random(),
            map_size: 5000.0,
            biome_distribution: BiomeWeights::balanced(),
            poi_count: 8,
            cache_count: 3,
        }
    }

    /// Anyone can verify the seed was fair
    pub fn verification_hash(&self) -> String {
        // Cryptographic proof seed wasn't manipulated
        sha256(&self.seed.to_le_bytes())
    }
}
```

---

## Match Events

Scheduled events create tension and force movement.

```rust
pub enum ScheduledEvent {
    /// Supply drop with high-tier loot
    AirDrop {
        day: u32,
        position: Vec3,
        contents: Vec<Item>,
    },

    /// Weather forces shelter or die
    Blizzard {
        day: u32,
        duration: Duration,
        damage_per_second: f32,
    },

    /// Predator pack spawns, hunts players
    WolfPack {
        day: u32,
        count: u32,
        aggression: f32,
    },

    /// Bonus loot zone opens temporarily
    CacheReveal {
        day: u32,
        duration: Duration,
        position: Vec3,
    },
}

pub fn standard_event_schedule() -> Vec<ScheduledEvent> {
    vec![
        ScheduledEvent::AirDrop { day: 2, .. },
        ScheduledEvent::Blizzard { day: 3, duration: Duration::from_secs(600), .. },
        ScheduledEvent::WolfPack { day: 4, count: 6, .. },
        ScheduledEvent::AirDrop { day: 5, .. },
        ScheduledEvent::CacheReveal { day: 6, .. },
    ]
}
```

---

## Server Architecture

### Match Server

```rust
pub struct SeedWarsMatch {
    pub match_id: Uuid,
    pub seed: SeedWarsSeed,
    pub state: MatchState,
    pub players: HashMap<PlayerId, SeedWarsPlayer>,
    pub alive_count: u32,
    pub pot: PotDistribution,
    pub start_time: Instant,
    pub phase: MatchPhase,
}

pub enum MatchState {
    WaitingForPlayers,
    Starting { countdown: Duration },
    InProgress,
    Ending { winner: Option<PlayerId> },
    Complete,
}

pub enum MatchPhase {
    Spawn,
    Survival { day: u32 },
    FinalCircle,
}

impl SeedWarsMatch {
    pub fn tick(&mut self, dt: f32) {
        // Update survival needs
        for player in self.players.values_mut() {
            player.survival.update(dt, &player.conditions);
        }

        // Update circle
        self.update_circle(dt);

        // Check for winner
        if self.alive_count <= 1 {
            self.end_match();
        }

        // Process scheduled events
        self.process_events();
    }

    fn end_match(&mut self) {
        self.state = MatchState::Ending {
            winner: self.find_last_alive()
        };

        // Distribute pot
        if let Some(winner) = self.find_last_alive() {
            self.payout_winner(winner);
        }
    }
}
```

---

## Anti-Cheat & Integrity

### Wagered Match Security

```rust
pub struct MatchIntegrity {
    /// Server-authoritative for all combat
    pub combat_authority: Authority::Server,

    /// Position validation
    pub max_speed: f32,
    pub teleport_detection: bool,

    /// Input validation
    pub action_rate_limits: bool,

    /// Match recording for disputes
    pub full_replay_recording: bool,
}

/// Disconnect handling
pub struct DisconnectPolicy {
    pub grace_period: Duration,    // 5 minutes to reconnect
    pub afk_timeout: Duration,     // 3 minutes of no input
    pub disconnect_as_death: bool, // After grace period
}
```

---

## Economy Integration

### Payment Flow

```rust
pub enum EntryMethod {
    /// Direct USD payment
    Stripe { payment_intent: String },

    /// Roanoke Coin (on-chain or off-chain)
    RoanokeCoin {
        amount: u32,
        source: CoinSource,
    },

    /// Tournament ticket (purchased/earned)
    Ticket { ticket_id: Uuid },
}

pub enum CoinSource {
    Wallet { address: String },
    InGameBalance,
}

pub struct PayoutMethod {
    pub usd_payout: Option<StripeTransfer>,
    pub coin_payout: Option<CoinTransfer>,
    pub tax_withholding: f32,  // For US players >$600/year
}
```

---

## Spectator Mode

```rust
pub struct SpectatorMode {
    pub dead_players: Vec<PlayerId>,
    pub external_viewers: Vec<ViewerId>,
    pub stream_delay: Duration,    // 2 minute delay to prevent ghosting
    pub free_cam: bool,            // After match ends
}

impl SpectatorMode {
    pub fn allowed_views(&self, spectator: PlayerId) -> Vec<SpectatorView> {
        vec![
            SpectatorView::FollowPlayer { player_id: any_alive },
            SpectatorView::MapOverview,
            // No free cam until match ends (anti-ghost)
        ]
    }
}
```

---

## Queue Types

### Solo Queue (Free-for-all)

- 50 players
- Winner take all ($225)
- No alliances allowed (or alliance = team queue)

### Duo Queue

- 25 teams of 2
- Team shares entry ($5 each = $10/team, $250 pot)
- Last team standing splits pot
- Both members must die for elimination

### Squad Queue (Future)

- 10 teams of 5
- Higher stakes ($10/player = $500 pot)
- Coordination emphasis

---

## Minimal Implementation Checklist

### Phase 1: Core Loop

- [ ] Queue system with escrow
- [ ] Match creation with seed
- [ ] Spawn phase (no items, spread positions)
- [ ] Basic survival needs (hunger, thirst)
- [ ] Permadeath with elimination tracking
- [ ] Winner detection and payout

### Phase 2: Map & Circle

- [ ] Shrinking circle implementation
- [ ] Circle damage application
- [ ] Day/night cycle (1 hour = 1 day)
- [ ] Basic biome generation from seed

### Phase 3: Survival Depth

- [ ] Temperature/warmth system
- [ ] Crafting essentials
- [ ] Hunting integration (existing fauna)
- [ ] Shelter building

### Phase 4: Polish

- [ ] Death screen with emotional impact
- [ ] Spectator mode
- [ ] Match replay recording
- [ ] Leaderboards (lifetime earnings)

---

## Files to Create

```
roanoke_game/src/seed_wars/
├── mod.rs
├── queue.rs           // Matchmaking and wagering
├── match_state.rs     // Match lifecycle
├── survival.rs        // Hunger/thirst/warmth
├── circle.rs          // Shrinking zone
├── loot.rs            // Scarcity tables
├── alliance.rs        // Temporary teams
├── death.rs           // Elimination handling
├── payout.rs          // Pot distribution
└── spectate.rs        // Dead player viewing
```

---

*End of Seed Wars Specification*
