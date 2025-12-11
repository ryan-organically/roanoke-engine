# Multiplayer Hunting Events Specification

## Roanoke Engine - Cooperative Hunting Framework

This document specifies the architecture for multiplayer hunting events in Roanoke Engine, enabling cooperative group hunts with shared objectives, synchronized animal behavior, and coordinated rewards.

---

## Table of Contents

1. [Overview](#overview)
2. [Core Data Structures](#core-data-structures)
3. [Event Types](#event-types)
4. [Herd Discovery System](#herd-discovery-system)
5. [Tracking Phase](#tracking-phase)
6. [Hunt Coordination](#hunt-coordination)
7. [Animal Behavior During Events](#animal-behavior-during-events)
8. [Reward Distribution](#reward-distribution)
9. [Network Synchronization](#network-synchronization)
10. [Integration with Existing Systems](#integration-with-existing-systems)
11. [Implementation Phases](#implementation-phases)

---

## Overview

### Design Goals

- **Authentic Group Hunts**: Recreate historical communal hunting traditions
- **Coordinated Gameplay**: Require teamwork for optimal success
- **Living Herds**: Realistic herd behavior with flight, scatter, and regrouping
- **Shared Discovery**: Finding herds triggers events for nearby players
- **Fair Rewards**: Contribution-based loot distribution
- **Network Efficient**: Minimal bandwidth for synchronized animal state

### Event Philosophy

Multiplayer hunting events should feel like emergent discoveries rather than scripted missions. Players stumble upon animal signs, track herds, coordinate approach strategies, and execute hunts with roles determined organically by positioning and timing.

### Relationship to Existing Systems

| System | Integration |
|--------|-------------|
| Docile Fauna (DOCILE_FAUNA_SPEC.md) | Herd spawning, flee behavior, harvest tables |
| Hunting Skill Tree (HUNTING_SKILL_TREE_SPEC.md) | Tracking skills, loot bonuses, role abilities |
| Animal System (ANIMAL_SYSTEM_SPEC.md) | Predator interference, AI state machines |
| Faction System (FACTION_SYSTEM_SPEC.md) | Hunting ground territories, reputation rewards |

---

## Core Data Structures

### Location: `roanoke_game/src/multiplayer/hunting_events.rs`

```rust
//! Multiplayer hunting event system
//!
//! Submodules:
//!   - events.rs       - Event lifecycle management
//!   - discovery.rs    - Herd discovery mechanics
//!   - tracking.rs     - Group tracking phase
//!   - coordination.rs - Hunt execution & roles
//!   - rewards.rs      - Contribution & loot distribution
//!   - network.rs      - State synchronization
```

### Hunt Event State

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle state of a multiplayer hunting event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HuntEventState {
    /// Herd discovered, players can join
    Discovery,
    /// Tracking phase - following signs to herd
    Tracking,
    /// Active hunt in progress
    Active,
    /// Hunt completed, distributing rewards
    Harvesting,
    /// Event concluded
    Completed,
    /// Herd escaped, event failed
    Failed,
}

/// A multiplayer hunting event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntEvent {
    pub id: Uuid,
    pub state: HuntEventState,
    pub herd: HerdInfo,
    pub participants: Vec<HuntParticipant>,
    pub discovery_time: f64,
    pub tracking_progress: f32,
    pub hunt_area: HuntArea,
    pub difficulty: HuntDifficulty,
    pub modifiers: Vec<HuntModifier>,
}

/// Information about the target herd
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerdInfo {
    pub species: DocileSpecies,
    pub initial_count: u32,
    pub current_count: u32,
    pub herd_leader: Option<EntityId>,
    pub position: Vec3,
    pub movement_direction: Vec3,
    pub alert_level: f32,          // 0.0 = grazing, 1.0 = fleeing
    pub scattered: bool,
    pub quality_tier: HerdQuality,
}

/// Quality tier affects trophy potential and loot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HerdQuality {
    Common,       // Standard herd
    Prime,        // Healthy, well-fed animals
    Exceptional,  // Trophy specimens present
    Legendary,    // Rare spawn, unique trophies
}

/// Hunt difficulty based on herd and conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HuntDifficulty {
    Easy,      // Small herd, open terrain
    Normal,    // Standard conditions
    Hard,      // Large herd, dense cover
    Expert,    // Wary animals, difficult terrain
    Legendary, // Maximum challenge, unique rewards
}

/// Geographic bounds of the hunt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntArea {
    pub center: Vec3,
    pub radius: f32,
    pub terrain_type: TerrainType,
    pub cover_density: f32,        // 0.0 = open, 1.0 = dense forest
    pub escape_routes: Vec<EscapeRoute>,
}

/// Possible escape paths for fleeing herd
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscapeRoute {
    pub direction: Vec3,
    pub terrain_advantage: f32,    // How favorable for animal escape
    pub can_be_blocked: bool,
}
```

### Participant Tracking

```rust
/// A player participating in a hunt event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntParticipant {
    pub player_id: PlayerId,
    pub role: HuntRole,
    pub contribution: HuntContribution,
    pub position: Vec3,
    pub ready_status: bool,
    pub skills: ParticipantSkills,
}

/// Roles players can assume during hunts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HuntRole {
    /// Initiates the hunt, coordinates group
    Leader,
    /// Expert tracker, finds and follows signs
    Tracker,
    /// Positions to cut off escape routes
    Flanker,
    /// Primary shooter/attacker
    Striker,
    /// Provides cover fire, support
    Support,
    /// Processes kills, carries loot
    Harvester,
    /// Observes from distance, spots movement
    Scout,
}

impl HuntRole {
    pub fn max_per_hunt(&self) -> u32 {
        match self {
            Self::Leader => 1,
            Self::Tracker => 2,
            Self::Flanker => 4,
            Self::Striker => 4,
            Self::Support => 3,
            Self::Harvester => 2,
            Self::Scout => 2,
        }
    }

    pub fn contribution_multiplier(&self) -> f32 {
        match self {
            Self::Leader => 1.2,     // Bonus for coordination
            Self::Tracker => 1.1,    // Bonus for finding herd
            Self::Striker => 1.0,    // Standard
            Self::Flanker => 1.0,    // Standard
            Self::Support => 0.9,    // Slightly less direct
            Self::Harvester => 0.85, // Post-kill focus
            Self::Scout => 0.8,      // Observation focus
        }
    }
}

/// Contribution tracking for reward distribution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HuntContribution {
    pub discovery_credit: f32,     // Found signs, spotted herd
    pub tracking_credit: f32,      // Successfully followed trail
    pub position_credit: f32,      // Good flanking/blocking
    pub kills: u32,                // Direct kills
    pub assists: u32,              // Damage without kill
    pub scare_events: u32,         // Accidentally spooked herd (negative)
    pub harvest_credit: f32,       // Processed kills
    pub time_active: f32,          // Seconds actively participating
}

impl HuntContribution {
    pub fn total_score(&self) -> f32 {
        let positive = self.discovery_credit * 50.0
            + self.tracking_credit * 30.0
            + self.position_credit * 20.0
            + self.kills as f32 * 100.0
            + self.assists as f32 * 40.0
            + self.harvest_credit * 25.0
            + (self.time_active / 60.0) * 5.0; // 5 points per minute

        let negative = self.scare_events as f32 * 25.0;

        (positive - negative).max(0.0)
    }
}
```

---

## Event Types

### Herd Hunt (Primary)

The core multiplayer hunting experience - tracking and hunting a group of animals.

```rust
/// Herd hunt event configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerdHuntConfig {
    pub species: DocileSpecies,
    pub min_herd_size: u32,
    pub max_herd_size: u32,
    pub min_players: u32,
    pub max_players: u32,
    pub time_limit: Option<f32>,   // Seconds, None = unlimited
    pub success_threshold: f32,    // % of herd that must be taken
}

/// Supported herd hunt types
pub const HERD_HUNT_CONFIGS: &[HerdHuntConfig] = &[
    // Elk Hunt - The flagship experience
    HerdHuntConfig {
        species: DocileSpecies::Elk,
        min_herd_size: 6,
        max_herd_size: 15,
        min_players: 2,
        max_players: 8,
        time_limit: Some(1800.0),  // 30 minutes
        success_threshold: 0.25,   // Take at least 25%
    },
    // Deer Drive - Classic colonial hunt
    HerdHuntConfig {
        species: DocileSpecies::WhiteTailedDeer,
        min_herd_size: 4,
        max_herd_size: 10,
        min_players: 2,
        max_players: 6,
        time_limit: Some(1200.0),  // 20 minutes
        success_threshold: 0.30,
    },
    // Turkey Roundup - Easier group hunt
    HerdHuntConfig {
        species: DocileSpecies::WildTurkey,
        min_herd_size: 5,
        max_herd_size: 12,
        min_players: 2,
        max_players: 4,
        time_limit: Some(900.0),   // 15 minutes
        success_threshold: 0.40,
    },
    // Bison Hunt - Epic scale (future)
    HerdHuntConfig {
        species: DocileSpecies::AmericanBison,
        min_herd_size: 10,
        max_herd_size: 30,
        min_players: 4,
        max_players: 12,
        time_limit: Some(2700.0),  // 45 minutes
        success_threshold: 0.15,
    },
];
```

### Special Event Types

```rust
/// Special hunting event variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecialHuntEvent {
    /// Track and hunt a specific trophy animal
    TrophyHunt {
        target_species: DocileSpecies,
        trophy_traits: Vec<TrophyTrait>,
        tracking_difficulty: f32,
    },

    /// Protect herd from predator pack
    PredatorDefense {
        herd_species: DocileSpecies,
        predator_species: HostileSpecies,
        predator_count: u32,
    },

    /// Drive animals into prepared kill zone
    DriveHunt {
        species: DocileSpecies,
        drive_distance: f32,
        kill_zone: KillZone,
    },

    /// Night hunt with limited visibility
    NightHunt {
        species: DocileSpecies,
        moon_phase: MoonPhase,
        torch_allowed: bool,
    },

    /// Seasonal migration interception
    MigrationIntercept {
        species: DocileSpecies,
        migration_path: Vec<Vec3>,
        window_duration: f32,
    },
}

/// Trophy animal special traits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrophyTrait {
    GiantAntlers,      // 2x antler size
    AlbinoColoring,    // White fur/feathers
    MelanisticColoring,// Black fur/feathers
    Scarred,           // Battle-worn veteran
    Massive,           // 1.5x body size
    Ancient,           // Very old specimen
    Piebald,           // Mixed coloring
}
```

---

## Herd Discovery System

### Discovery Triggers

Players discover herds through various means, each granting different preparation time.

```rust
/// How the herd was discovered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Found fresh tracks, droppings, or rubs
    TrackingSign,
    /// Heard animal calls
    AudioCue,
    /// Direct visual sighting
    VisualSpotting,
    /// NPC provided information
    NPCIntel,
    /// Found animal bedding area
    BeddingArea,
    /// Spotted from elevated position
    ScoutingPost,
    /// Following water source
    WaterHole,
    /// Seasonal grazing area
    GrazingMeadow,
}

impl DiscoveryMethod {
    /// Time before herd moves away (seconds)
    pub fn discovery_window(&self) -> f32 {
        match self {
            Self::TrackingSign => 600.0,    // 10 min, old signs
            Self::AudioCue => 300.0,        // 5 min, they're nearby
            Self::VisualSpotting => 120.0,  // 2 min, they may have seen you
            Self::NPCIntel => 1800.0,       // 30 min, general area
            Self::BeddingArea => 900.0,     // 15 min, they'll return
            Self::ScoutingPost => 450.0,    // 7.5 min, good overview
            Self::WaterHole => 1200.0,      // 20 min, predictable
            Self::GrazingMeadow => 1500.0,  // 25 min, feeding time
        }
    }

    /// Bonus XP for this discovery type
    pub fn discovery_xp(&self) -> u32 {
        match self {
            Self::TrackingSign => 50,
            Self::AudioCue => 30,
            Self::VisualSpotting => 20,
            Self::NPCIntel => 10,
            Self::BeddingArea => 60,
            Self::ScoutingPost => 40,
            Self::WaterHole => 35,
            Self::GrazingMeadow => 25,
        }
    }
}

/// Discovery event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerdDiscovery {
    pub discoverer: PlayerId,
    pub method: DiscoveryMethod,
    pub estimated_herd_size: HerdSizeEstimate,
    pub estimated_direction: Vec3,
    pub confidence: f32,           // How accurate the estimate is
    pub sign_freshness: f32,       // 0.0 = old, 1.0 = fresh
    pub discovery_location: Vec3,
}

/// Estimated herd size (imprecise by design)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HerdSizeEstimate {
    Few,       // 2-4 animals
    Several,   // 5-8 animals
    Many,      // 9-15 animals
    Large,     // 16-25 animals
    Massive,   // 26+ animals
}
```

### Sign Types

```rust
/// Types of animal signs players can find
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimalSign {
    /// Hoofprints/footprints in soft ground
    Tracks {
        species: DocileSpecies,
        freshness: f32,
        count_estimate: u32,
        direction: Vec3,
    },

    /// Animal droppings
    Scat {
        species: DocileSpecies,
        freshness: f32,
        diet_indicator: DietIndicator,
    },

    /// Tree/vegetation damage
    Rub {
        species: DocileSpecies,
        freshness: f32,
        height: f32,  // Indicates animal size
    },

    /// Flattened vegetation where animals rested
    Bedding {
        species: DocileSpecies,
        count: u32,
        warmth: f32,  // Recent = warm
    },

    /// Grazing/browsing damage
    Browse {
        species: DocileSpecies,
        freshness: f32,
        height: f32,
    },

    /// Fur/feathers caught on vegetation
    Hair {
        species: DocileSpecies,
        quality: HerdQuality,  // Color indicates health
    },

    /// Disturbed water/mud
    Wallow {
        species: DocileSpecies,
        freshness: f32,
        size: f32,
    },
}

/// What the animal has been eating
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DietIndicator {
    Grass,
    Acorns,
    Berries,
    Bark,
    Crops,     // Near farms
    Mixed,
}
```

### Broadcasting Discovery

When a player discovers a herd, nearby players are notified.

```rust
/// Broadcast hunt opportunity to nearby players
pub fn broadcast_herd_discovery(
    discovery: &HerdDiscovery,
    world: &World,
    network: &NetworkManager,
) -> HuntEvent {
    // Find nearby players who can join
    let broadcast_radius = match discovery.method {
        DiscoveryMethod::VisualSpotting => 200.0,  // Quiet discovery
        DiscoveryMethod::AudioCue => 400.0,        // Heard calls
        _ => 300.0,                                 // Standard
    };

    let discoverer_pos = world.get_player_position(discovery.discoverer);
    let nearby_players = world.find_players_in_radius(discoverer_pos, broadcast_radius);

    // Create the hunt event
    let event = HuntEvent {
        id: Uuid::new_v4(),
        state: HuntEventState::Discovery,
        herd: estimate_herd_from_discovery(discovery, world),
        participants: vec![HuntParticipant {
            player_id: discovery.discoverer,
            role: HuntRole::Leader,  // Discoverer becomes leader
            contribution: HuntContribution {
                discovery_credit: 1.0,
                ..Default::default()
            },
            position: discoverer_pos,
            ready_status: false,
            skills: get_player_hunting_skills(discovery.discoverer),
        }],
        discovery_time: world.game_time(),
        tracking_progress: 0.0,
        hunt_area: calculate_hunt_area(discovery, world),
        difficulty: estimate_difficulty(discovery, world),
        modifiers: calculate_modifiers(discovery, world),
    };

    // Send invitations to nearby players
    for player_id in nearby_players {
        if player_id != discovery.discoverer {
            network.send_hunt_invitation(player_id, &event);
        }
    }

    event
}

/// Hunt invitation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntInvitation {
    pub event_id: Uuid,
    pub leader: PlayerId,
    pub species: DocileSpecies,
    pub estimated_size: HerdSizeEstimate,
    pub difficulty: HuntDifficulty,
    pub distance: f32,
    pub time_remaining: f32,
}
```

---

## Tracking Phase

Once players join, the group enters the tracking phase.

### Tracking Mechanics

```rust
/// Tracking phase state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingState {
    pub signs_found: Vec<FoundSign>,
    pub estimated_herd_position: Vec3,
    pub position_confidence: f32,     // 0.0 = lost, 1.0 = pinpointed
    pub trail_age: f32,               // How old the trail is
    pub herd_alert_level: f32,        // Are they aware of pursuers?
    pub tracking_participants: Vec<PlayerId>,
}

/// A sign that was found during tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundSign {
    pub sign: AnimalSign,
    pub location: Vec3,
    pub finder: PlayerId,
    pub timestamp: f64,
}

/// Update tracking progress
pub fn update_tracking(
    event: &mut HuntEvent,
    tracking: &mut TrackingState,
    dt: f32,
    world: &World,
) {
    // Trail gets colder over time
    tracking.trail_age += dt;
    tracking.position_confidence -= dt * 0.01;

    // Move estimated position based on herd movement
    let herd_speed = get_species_travel_speed(event.herd.species);
    let drift = event.herd.movement_direction * herd_speed * dt;
    tracking.estimated_herd_position += drift;

    // Check if trackers find new signs
    for participant in &event.participants {
        if tracking.tracking_participants.contains(&participant.player_id) {
            let player_pos = world.get_player_position(participant.player_id);

            // Check for nearby signs
            if let Some(sign) = find_nearby_sign(player_pos, &event.herd, world) {
                tracking.signs_found.push(FoundSign {
                    sign: sign.clone(),
                    location: player_pos,
                    finder: participant.player_id,
                    timestamp: world.game_time(),
                });

                // Update confidence based on sign freshness
                let freshness_boost = sign.freshness() * 0.15;
                tracking.position_confidence =
                    (tracking.position_confidence + freshness_boost).min(1.0);

                // Update estimated position
                if let Some(direction) = sign.direction_hint() {
                    tracking.estimated_herd_position =
                        player_pos + direction * estimate_distance(&sign);
                }

                // Reset trail age with fresh sign
                tracking.trail_age = tracking.trail_age.min(sign.age());
            }
        }
    }

    // Transition to Active when confidence is high enough
    if tracking.position_confidence >= 0.8 {
        event.state = HuntEventState::Active;
    }

    // Fail if trail goes completely cold
    if tracking.position_confidence <= 0.0 || tracking.trail_age > 1800.0 {
        event.state = HuntEventState::Failed;
    }
}
```

### Skill-Based Tracking Bonuses

```rust
/// Apply tracking skill bonuses
pub fn apply_tracking_skills(
    player: &HuntParticipant,
    sign: &AnimalSign,
) -> TrackingBonus {
    let skills = &player.skills;

    TrackingBonus {
        // Expert Tracker: Read more from signs
        info_quality: if skills.has_expert_tracker { 1.5 } else { 1.0 },

        // Animal Lore: Better direction estimates
        direction_accuracy: if skills.has_animal_lore { 0.9 } else { 0.7 },

        // Sign Reading: See older signs
        max_sign_age: if skills.has_sign_reading { 3600.0 } else { 1800.0 },

        // Wind Reading: Account for scent spread
        scent_tracking: skills.has_wind_reading,

        // Herd Behavior: Predict movement
        prediction_bonus: if skills.has_herd_behavior { 0.2 } else { 0.0 },
    }
}
```

---

## Hunt Coordination

### Position Assignment

```rust
/// Coordinate hunter positions before engaging
pub fn assign_hunt_positions(
    event: &HuntEvent,
    world: &World,
) -> HuntFormation {
    let herd_pos = event.herd.position;
    let herd_dir = event.herd.movement_direction;
    let escape_routes = &event.hunt_area.escape_routes;

    let mut formation = HuntFormation::default();

    // Sort participants by role priority
    let mut sorted_participants = event.participants.clone();
    sorted_participants.sort_by_key(|p| match p.role {
        HuntRole::Flanker => 0,   // Position first
        HuntRole::Scout => 1,
        HuntRole::Striker => 2,
        HuntRole::Support => 3,
        HuntRole::Tracker => 4,
        HuntRole::Harvester => 5,
        HuntRole::Leader => 6,
    });

    // Assign flankers to escape routes
    let mut flanker_idx = 0;
    for participant in sorted_participants.iter() {
        if participant.role == HuntRole::Flanker && flanker_idx < escape_routes.len() {
            let route = &escape_routes[flanker_idx];
            let block_pos = herd_pos + route.direction * 50.0; // 50m ahead
            formation.positions.insert(participant.player_id, HuntPosition {
                target: block_pos,
                facing: -route.direction,
                role_objective: "Block escape route",
                ready_distance: 10.0,
            });
            flanker_idx += 1;
        }
    }

    // Position strikers in approach arc
    let approach_dir = -herd_dir.normalize();
    let mut striker_angle = -45.0_f32.to_radians();
    let angle_step = 30.0_f32.to_radians();

    for participant in sorted_participants.iter() {
        if participant.role == HuntRole::Striker {
            let rotated = rotate_vec3_y(approach_dir, striker_angle);
            let strike_pos = herd_pos + rotated * 40.0;
            formation.positions.insert(participant.player_id, HuntPosition {
                target: strike_pos,
                facing: (herd_pos - strike_pos).normalize(),
                role_objective: "Primary attack position",
                ready_distance: 5.0,
            });
            striker_angle += angle_step;
        }
    }

    // Support behind strikers
    for participant in sorted_participants.iter() {
        if participant.role == HuntRole::Support {
            let support_pos = herd_pos + approach_dir * 60.0;
            formation.positions.insert(participant.player_id, HuntPosition {
                target: support_pos,
                facing: (herd_pos - support_pos).normalize(),
                role_objective: "Provide covering fire",
                ready_distance: 8.0,
            });
        }
    }

    // Scouts on high ground
    for participant in sorted_participants.iter() {
        if participant.role == HuntRole::Scout {
            if let Some(vantage) = find_vantage_point(herd_pos, world) {
                formation.positions.insert(participant.player_id, HuntPosition {
                    target: vantage,
                    facing: (herd_pos - vantage).normalize(),
                    role_objective: "Observe and report movement",
                    ready_distance: 15.0,
                });
            }
        }
    }

    formation
}

/// Hunt formation data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HuntFormation {
    pub positions: HashMap<PlayerId, HuntPosition>,
    pub engagement_signal: Option<EngagementSignal>,
}

/// Individual position assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntPosition {
    pub target: Vec3,
    pub facing: Vec3,
    pub role_objective: &'static str,
    pub ready_distance: f32,
}

/// Signal to begin the hunt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngagementSignal {
    LeaderCall,      // Leader gives verbal command
    FirstShot,       // Whoever gets first shot
    Countdown,       // Synchronized timer
    HerdMoves,       // When herd starts moving
}
```

### Communication System

```rust
/// Quick communication commands during hunt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HuntCommand {
    // Movement
    Hold,            // Stay in position
    Advance,         // Move forward
    Retreat,         // Pull back
    Flank,           // Go around
    Encircle,        // Surround

    // Status
    Ready,           // In position
    Spotted,         // Visual on herd
    Moving,          // Herd is moving
    Scattered,       // Herd has scattered
    Regrouping,      // Herd reforming

    // Action
    Engage,          // Begin attack
    HoldFire,        // Stop shooting
    FinishIt,        // Kill wounded
    Harvest,         // Begin processing
    Abort,           // Abandon hunt
}

/// Send command to hunt group
pub fn send_hunt_command(
    event: &HuntEvent,
    sender: PlayerId,
    command: HuntCommand,
    network: &NetworkManager,
) {
    // Only leader can send certain commands
    let requires_leader = matches!(command,
        HuntCommand::Engage |
        HuntCommand::Abort |
        HuntCommand::Encircle
    );

    let is_leader = event.participants.iter()
        .any(|p| p.player_id == sender && p.role == HuntRole::Leader);

    if requires_leader && !is_leader {
        return; // Ignore non-leader commands
    }

    let message = HuntCommandMessage {
        event_id: event.id,
        sender,
        command,
        timestamp: std::time::Instant::now(),
    };

    for participant in &event.participants {
        network.send_hunt_command(participant.player_id, &message);
    }
}
```

---

## Animal Behavior During Events

### Herd AI During Hunt

```rust
/// Herd behavior states during multiplayer hunt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HerdHuntState {
    /// Normal grazing behavior
    Grazing,
    /// Noticed something, heads up
    Alert,
    /// Nervous, preparing to flee
    Nervous,
    /// Running as a group
    Fleeing,
    /// Herd has split up
    Scattered,
    /// Animals regrouping after scare
    Regrouping,
    /// Cornered, may fight or freeze
    Cornered,
    /// Ultimate escape, stamina exhausted
    Exhausted,
}

/// Update herd behavior during hunt event
pub fn update_herd_behavior(
    event: &mut HuntEvent,
    herd_entities: &mut [FaunaEntity],
    hunters: &[Vec3],
    world: &World,
    dt: f32,
) {
    let herd = &mut event.herd;

    // Calculate threat level from all hunters
    let threat_level = calculate_threat_level(herd, hunters);
    herd.alert_level = (herd.alert_level + threat_level * dt).min(1.0);

    // Determine herd state
    let herd_state = determine_herd_state(herd);

    match herd_state {
        HerdHuntState::Grazing => {
            // Normal behavior, slow alert decay
            herd.alert_level = (herd.alert_level - dt * 0.05).max(0.0);
        }

        HerdHuntState::Alert => {
            // Heads up, looking around
            for entity in herd_entities.iter_mut() {
                entity.behavior_state = FaunaBehavior::Alert;
                entity.look_at_threat(nearest_hunter(entity.position, hunters));
            }
        }

        HerdHuntState::Nervous => {
            // Preparing to flee, some may bolt early
            let bolt_chance = 0.01 * dt; // 1% per second
            for entity in herd_entities.iter_mut() {
                if rand::random::<f32>() < bolt_chance {
                    entity.behavior_state = FaunaBehavior::Fleeing;
                    entity.flee_direction = calculate_flee_direction(
                        entity.position,
                        hunters,
                        &event.hunt_area.escape_routes
                    );
                }
            }
        }

        HerdHuntState::Fleeing => {
            // Group flight behavior
            let leader = find_herd_leader(herd_entities);
            let flee_dir = calculate_group_flee_direction(
                herd.position,
                hunters,
                &event.hunt_area.escape_routes,
            );

            for entity in herd_entities.iter_mut() {
                entity.behavior_state = FaunaBehavior::Fleeing;
                // Follow leader with some variation
                entity.flee_direction = flee_dir + random_spread(0.2);
                entity.apply_flee_movement(dt);
            }

            // Update herd center position
            herd.position = calculate_herd_center(herd_entities);
            herd.movement_direction = flee_dir;
        }

        HerdHuntState::Scattered => {
            herd.scattered = true;
            // Each animal flees independently
            for entity in herd_entities.iter_mut() {
                let personal_flee = calculate_flee_direction(
                    entity.position,
                    hunters,
                    &event.hunt_area.escape_routes,
                );
                entity.flee_direction = personal_flee;
                entity.apply_flee_movement(dt);
            }
        }

        HerdHuntState::Regrouping => {
            // Animals try to find each other
            let center = calculate_herd_center(herd_entities);
            for entity in herd_entities.iter_mut() {
                if !entity.is_fleeing() {
                    entity.move_toward(center, dt);
                }
            }
            herd.scattered = check_herd_scattered(herd_entities);
        }

        HerdHuntState::Cornered => {
            // No escape, some may freeze
            for entity in herd_entities.iter_mut() {
                if rand::random::<f32>() < 0.3 {
                    entity.behavior_state = FaunaBehavior::Frozen;
                } else {
                    entity.behavior_state = FaunaBehavior::Panicked;
                }
            }
        }

        HerdHuntState::Exhausted => {
            // Stamina depleted, slow movement
            for entity in herd_entities.iter_mut() {
                entity.speed_multiplier = 0.3;
                entity.behavior_state = FaunaBehavior::Exhausted;
            }
        }
    }
}

/// Calculate threat level from hunter positions
fn calculate_threat_level(herd: &HerdInfo, hunters: &[Vec3]) -> f32 {
    let mut threat = 0.0;

    for hunter_pos in hunters {
        let distance = (herd.position - *hunter_pos).length();
        let species_detect = get_species_detection_range(herd.species);

        if distance < species_detect {
            // Closer = more threatening
            let proximity_threat = 1.0 - (distance / species_detect);

            // Multiple hunters compound threat
            threat += proximity_threat * 0.5;
        }
    }

    threat.min(1.0)
}

/// Determine which escape route to use
fn calculate_group_flee_direction(
    herd_pos: Vec3,
    hunters: &[Vec3],
    escape_routes: &[EscapeRoute],
) -> Vec3 {
    // Average hunter position
    let threat_center = hunters.iter()
        .fold(Vec3::ZERO, |acc, p| acc + *p) / hunters.len() as f32;

    // Flee away from threat
    let away_from_threat = (herd_pos - threat_center).normalize();

    // Find best escape route
    let mut best_route = away_from_threat;
    let mut best_score = 0.0;

    for route in escape_routes {
        // Score based on angle from threat and terrain advantage
        let angle_score = route.direction.dot(away_from_threat);
        let terrain_score = route.terrain_advantage;

        // Check if route is blocked by hunters
        let blocked = hunters.iter().any(|h| {
            let to_hunter = (*h - herd_pos).normalize();
            to_hunter.dot(route.direction) > 0.8 &&
            (*h - herd_pos).length() < 60.0
        });

        if blocked && route.can_be_blocked {
            continue; // Skip blocked routes
        }

        let score = angle_score * 0.6 + terrain_score * 0.4;
        if score > best_score {
            best_score = score;
            best_route = route.direction;
        }
    }

    best_route
}
```

---

## Reward Distribution

### Contribution-Based Loot

```rust
/// Calculate rewards for all participants
pub fn distribute_hunt_rewards(
    event: &HuntEvent,
    kills: &[HuntKill],
    world: &mut World,
) -> Vec<ParticipantReward> {
    let mut rewards = Vec::new();

    // Calculate total contribution score
    let total_contribution: f32 = event.participants.iter()
        .map(|p| p.contribution.total_score() * p.role.contribution_multiplier())
        .sum();

    if total_contribution <= 0.0 {
        return rewards;
    }

    // Generate base loot pool from kills
    let loot_pool = generate_loot_pool(kills, event.herd.quality_tier);

    for participant in &event.participants {
        let contribution = participant.contribution.total_score()
            * participant.role.contribution_multiplier();
        let share_percent = contribution / total_contribution;

        // Base XP based on contribution
        let base_xp = calculate_base_hunt_xp(event) as f32;
        let xp_earned = (base_xp * share_percent * 1.5).round() as u32; // 1.5x bonus for coop

        // Loot allocation
        let loot_share = allocate_loot(&loot_pool, share_percent, &participant.role);

        // Bonus rewards for exceptional performance
        let bonuses = calculate_performance_bonuses(participant, event);

        rewards.push(ParticipantReward {
            player_id: participant.player_id,
            xp_earned,
            hunting_xp: (xp_earned as f32 * 0.8) as u32,
            loot: loot_share,
            bonuses,
            contribution_rank: 0, // Set after sorting
            achievements: check_hunt_achievements(participant, event),
        });
    }

    // Sort by contribution and assign ranks
    rewards.sort_by(|a, b| b.xp_earned.cmp(&a.xp_earned));
    for (i, reward) in rewards.iter_mut().enumerate() {
        reward.contribution_rank = i + 1;
    }

    rewards
}

/// A single kill during the hunt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntKill {
    pub entity_id: EntityId,
    pub species: DocileSpecies,
    pub killer: PlayerId,
    pub assisters: Vec<PlayerId>,
    pub quality: AnimalQuality,
    pub trophy_traits: Vec<TrophyTrait>,
    pub location: Vec3,
    pub method: KillMethod,
}

/// How the animal was killed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KillMethod {
    BowShot,
    Firearm,
    Trap,
    Melee,
    CliffFall,  // Drove off cliff
    DrownedFlee,// Fled into deep water
}

/// Rewards for a participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantReward {
    pub player_id: PlayerId,
    pub xp_earned: u32,
    pub hunting_xp: u32,
    pub loot: Vec<LootItem>,
    pub bonuses: Vec<BonusReward>,
    pub contribution_rank: usize,
    pub achievements: Vec<HuntAchievement>,
}

/// Bonus rewards for exceptional performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BonusReward {
    /// Discoverer bonus
    DiscoveryBonus { xp: u32 },
    /// First blood
    FirstKillBonus { xp: u32 },
    /// Most kills
    TopKillerBonus { xp: u32, extra_loot: LootItem },
    /// Perfect positioning
    TacticalBonus { xp: u32 },
    /// No accidental scares
    StealthBonus { xp: u32 },
    /// Harvested everything
    HarvesterBonus { xp: u32, extra_materials: Vec<LootItem> },
    /// Led successful hunt
    LeadershipBonus { xp: u32, reputation: i32 },
}

/// Loot pool generation from kills
fn generate_loot_pool(kills: &[HuntKill], quality: HerdQuality) -> LootPool {
    let mut pool = LootPool::default();

    let quality_multiplier = match quality {
        HerdQuality::Common => 1.0,
        HerdQuality::Prime => 1.25,
        HerdQuality::Exceptional => 1.5,
        HerdQuality::Legendary => 2.0,
    };

    for kill in kills {
        let harvest = get_species_harvest(kill.species);

        pool.meat += (harvest.meat as f32 * quality_multiplier) as u32;
        pool.hides += harvest.hide;
        pool.bones += harvest.bones;

        if let Some(antlers) = harvest.antlers {
            if kill.trophy_traits.contains(&TrophyTrait::GiantAntlers) {
                pool.trophy_antlers += 1;
            } else {
                pool.antlers += antlers;
            }
        }

        // Trophy items
        for trait_type in &kill.trophy_traits {
            pool.trophies.push(TrophyItem {
                species: kill.species,
                trait_type: *trait_type,
                quality: kill.quality,
            });
        }
    }

    pool
}

/// Allocate loot based on share percentage and role
fn allocate_loot(pool: &LootPool, share: f32, role: &HuntRole) -> Vec<LootItem> {
    let mut loot = Vec::new();

    // Harvesters get material bonuses
    let material_bonus = if *role == HuntRole::Harvester { 1.2 } else { 1.0 };

    // Calculate share amounts
    let meat_share = (pool.meat as f32 * share * material_bonus).round() as u32;
    let hide_share = (pool.hides as f32 * share).round() as u32;
    let bone_share = (pool.bones as f32 * share * material_bonus).round() as u32;

    if meat_share > 0 {
        loot.push(LootItem::Material {
            material_type: MaterialType::Meat,
            amount: meat_share
        });
    }
    if hide_share > 0 {
        loot.push(LootItem::Material {
            material_type: MaterialType::Hide,
            amount: hide_share
        });
    }
    if bone_share > 0 {
        loot.push(LootItem::Material {
            material_type: MaterialType::Bone,
            amount: bone_share
        });
    }

    // Trophies go to highest contributor or killer
    // Handled separately in trophy allocation phase

    loot
}
```

### Trophy Allocation

```rust
/// Special trophy allocation rules
pub fn allocate_trophies(
    trophies: &[TrophyItem],
    kills: &[HuntKill],
    participants: &[HuntParticipant],
) -> HashMap<PlayerId, Vec<TrophyItem>> {
    let mut allocation: HashMap<PlayerId, Vec<TrophyItem>> = HashMap::new();

    for (i, trophy) in trophies.iter().enumerate() {
        // Find the kill this trophy came from
        let kill = kills.iter()
            .find(|k| k.trophy_traits.iter()
                .any(|t| *t == trophy.trait_type && k.species == trophy.species));

        if let Some(kill) = kill {
            // Killer gets first choice
            let recipient = if participants.iter()
                .any(|p| p.player_id == kill.killer)
            {
                kill.killer
            } else if !kill.assisters.is_empty() {
                // Random assister if killer left
                kill.assisters[i % kill.assisters.len()]
            } else {
                // Top contributor
                participants.iter()
                    .max_by(|a, b| a.contribution.total_score()
                        .partial_cmp(&b.contribution.total_score())
                        .unwrap_or(std::cmp::Ordering::Equal))
                    .map(|p| p.player_id)
                    .unwrap_or(kill.killer)
            };

            allocation.entry(recipient)
                .or_default()
                .push(trophy.clone());
        }
    }

    allocation
}
```

---

## Network Synchronization

### State Sync Protocol

```rust
/// Network messages for hunt synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HuntNetMessage {
    /// Server: New hunt event available
    EventCreated(HuntEvent),

    /// Client: Request to join hunt
    JoinRequest { event_id: Uuid, preferred_role: HuntRole },

    /// Server: Join approved with assigned role
    JoinApproved { event_id: Uuid, assigned_role: HuntRole },

    /// Server: Hunt state update (periodic)
    StateUpdate(HuntStateSnapshot),

    /// Client: Player position update
    PositionUpdate { position: Vec3, ready: bool },

    /// Any: Hunt command broadcast
    Command(HuntCommandMessage),

    /// Server: Herd position/state update
    HerdUpdate(HerdStateSnapshot),

    /// Server: Animal killed
    KillConfirmed(HuntKill),

    /// Server: Hunt concluded
    HuntComplete(HuntResult),

    /// Server: Hunt failed
    HuntFailed { reason: FailReason },

    /// Client: Leave hunt
    LeaveHunt { event_id: Uuid },
}

/// Compact herd state for network sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerdStateSnapshot {
    pub position: Vec3,
    pub movement_dir: Vec3,
    pub alert_level: f32,
    pub state: HerdHuntState,
    pub remaining_count: u32,
    pub scattered: bool,
    pub timestamp: f64,
}

/// Compact hunt state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntStateSnapshot {
    pub event_id: Uuid,
    pub state: HuntEventState,
    pub tracking_progress: f32,
    pub participant_positions: Vec<(PlayerId, Vec3)>,
    pub kills_so_far: u32,
    pub time_elapsed: f32,
    pub timestamp: f64,
}

/// Sync frequency configuration
pub const SYNC_CONFIG: SyncConfig = SyncConfig {
    // Full state sync every 5 seconds
    full_state_interval: 5.0,
    // Herd position update every 0.5 seconds during active hunt
    herd_update_interval: 0.5,
    // Player position interpolation
    position_lerp_speed: 10.0,
    // Maximum prediction time before resync
    max_prediction_ms: 500,
};
```

### Authority and Prediction

```rust
/// Server-authoritative hunt management
pub struct HuntAuthority {
    pub event_id: Uuid,
    pub is_host: bool,
    pub herd_authority: bool,  // Server controls herd
    pub kill_authority: bool,  // Server confirms kills
}

/// Client-side prediction for smooth gameplay
pub fn predict_herd_position(
    last_known: &HerdStateSnapshot,
    current_time: f64,
) -> Vec3 {
    let dt = (current_time - last_known.timestamp) as f32;

    // Don't predict too far
    if dt > SYNC_CONFIG.max_prediction_ms as f32 / 1000.0 {
        return last_known.position;
    }

    // Predict based on state
    let speed = match last_known.state {
        HerdHuntState::Grazing => 0.5,
        HerdHuntState::Alert => 0.0,    // Stationary
        HerdHuntState::Nervous => 1.0,
        HerdHuntState::Fleeing => 15.0, // Full speed
        HerdHuntState::Scattered => 12.0,
        HerdHuntState::Regrouping => 3.0,
        HerdHuntState::Cornered => 0.0,
        HerdHuntState::Exhausted => 4.0,
    };

    last_known.position + last_known.movement_dir * speed * dt
}

/// Reconcile prediction with server state
pub fn reconcile_herd_state(
    predicted: Vec3,
    server: &HerdStateSnapshot,
) -> Vec3 {
    let error = (predicted - server.position).length();

    if error > 5.0 {
        // Snap to server position if too far off
        server.position
    } else {
        // Smooth interpolation
        predicted.lerp(server.position, 0.3)
    }
}
```

---

## Integration with Existing Systems

### Hunting Skill Tree Integration

```rust
/// Skills that affect multiplayer hunts
pub struct MultplayerHuntSkills {
    // From Tracking branch
    pub expert_tracker: bool,      // Better sign reading
    pub animal_lore: bool,         // Predict herd movement
    pub wind_reading: bool,        // Scent tracking
    pub herd_behavior: bool,       // Know when they'll scatter

    // From Weapon Mastery
    pub clean_kill: bool,          // Higher quality harvest
    pub quick_draw: bool,          // Faster engagement

    // From Stealth
    pub silent_approach: bool,     // Less alert buildup
    pub camouflage: bool,          // Reduced detection range

    // From Harvest
    pub field_dressing: bool,      // More materials
    pub trophy_hunter: bool,       // Better trophy quality
}

/// Apply skill bonuses to hunt
pub fn apply_multiplayer_hunt_skills(
    participant: &mut HuntParticipant,
    event: &HuntEvent,
) {
    let skills = &participant.skills;

    // Tracking bonuses
    if skills.expert_tracker {
        // +25% tracking progress contribution
        participant.contribution.tracking_credit *= 1.25;
    }

    // Stealth bonuses
    if skills.silent_approach {
        // Reduce scare penalty
        participant.contribution.scare_events =
            (participant.contribution.scare_events as f32 * 0.5) as u32;
    }

    // Harvest bonuses applied in reward phase
}
```

### Faction Integration

```rust
/// Faction effects on multiplayer hunts
pub fn get_faction_hunt_bonuses(
    faction: Faction,
    hunt_location: Vec3,
) -> FactionHuntBonus {
    match faction {
        Faction::Cherokee => FactionHuntBonus {
            // Cherokee are expert communal hunters
            coordination_bonus: 1.15,
            territory_bonus: is_in_cherokee_territory(hunt_location),
            special_technique: Some(HuntTechnique::DriveHunt),
        },
        Faction::Powhatan => FactionHuntBonus {
            // Powhatan excel at river hunts
            coordination_bonus: 1.1,
            territory_bonus: is_near_river(hunt_location),
            special_technique: Some(HuntTechnique::WaterDrive),
        },
        Faction::English => FactionHuntBonus {
            // English prefer organized hunts
            coordination_bonus: 1.0,
            territory_bonus: is_in_english_territory(hunt_location),
            special_technique: Some(HuntTechnique::BeatersAndStands),
        },
        _ => FactionHuntBonus::default(),
    }
}
```

---

## Implementation Phases

### Phase 1: Core Hunt Loop

**Goal**: Basic multiplayer elk hunt functional

- [ ] Herd spawning with multiplayer visibility
- [ ] Discovery broadcast to nearby players
- [ ] Join/leave hunt events
- [ ] Basic tracking phase (follow waypoints)
- [ ] Simple flee behavior on engagement
- [ ] Kill registration and basic rewards
- [ ] Network state sync (position + herd state)

**Files**:
- `roanoke_game/src/multiplayer/hunting_events/mod.rs`
- `roanoke_game/src/multiplayer/hunting_events/events.rs`
- `roanoke_game/src/multiplayer/hunting_events/network.rs`

### Phase 2: Tracking System

**Goal**: Meaningful tracking gameplay

- [ ] Animal sign generation along herd path
- [ ] Sign inspection UI
- [ ] Tracking skill checks
- [ ] Trail confidence system
- [ ] Herd position estimation
- [ ] Direction indicators

**Files**:
- `roanoke_game/src/multiplayer/hunting_events/tracking.rs`
- `roanoke_game/src/multiplayer/hunting_events/signs.rs`

### Phase 3: Hunt Coordination

**Goal**: Team tactics matter

- [ ] Role assignment system
- [ ] Position recommendations
- [ ] Quick command system
- [ ] Formation visualization
- [ ] Ready check system
- [ ] Engagement signals

**Files**:
- `roanoke_game/src/multiplayer/hunting_events/coordination.rs`
- `roanoke_game/src/multiplayer/hunting_events/commands.rs`

### Phase 4: Advanced Herd AI

**Goal**: Realistic herd behavior

- [ ] Multi-state herd behavior
- [ ] Scatter/regroup mechanics
- [ ] Escape route evaluation
- [ ] Blocking detection
- [ ] Stamina system
- [ ] Leader following

**Files**:
- `roanoke_game/src/multiplayer/hunting_events/herd_ai.rs`
- `roanoke_game/src/fauna/group_behavior.rs` (modifications)

### Phase 5: Rewards and Progression

**Goal**: Fair, compelling rewards

- [ ] Contribution tracking
- [ ] Loot pool generation
- [ ] Share calculation
- [ ] Trophy allocation
- [ ] Achievement system
- [ ] XP distribution
- [ ] Skill tree integration

**Files**:
- `roanoke_game/src/multiplayer/hunting_events/rewards.rs`
- `roanoke_game/src/multiplayer/hunting_events/achievements.rs`

### Phase 6: Special Events

**Goal**: Variety and replayability

- [ ] Trophy hunts
- [ ] Night hunts
- [ ] Migration intercepts
- [ ] Drive hunts
- [ ] Predator defense
- [ ] Seasonal modifiers

**Files**:
- `roanoke_game/src/multiplayer/hunting_events/special.rs`
- `roanoke_game/src/multiplayer/hunting_events/modifiers.rs`

---

## Appendix: Elk Herd Specifics

Since "locating a herd of elk" was the primary example, here are elk-specific details:

### Elk Species Definition

```rust
/// Elk (Wapiti) - Primary large game for group hunts
pub const ELK_DEFINITION: DocileSpeciesDef = DocileSpeciesDef {
    id: DocileSpecies::Elk,
    name: "Elk",
    scientific_name: "Cervus canadensis",
    category: FaunaCategory::LargeMammal,
    behavior: DocileBehavior::Skittish,
    stats: FaunaStats {
        health: 120.0,           // Tougher than deer
        speed: 40.0,             // Fast but slightly slower than deer
        swim_speed: Some(15.0),  // Can cross rivers
        glide_speed: None,
        detection_range: 45.0,   // Very alert
        flee_range: 30.0,        // Start running earlier
        stamina_time: 25.0,      // Can run longer
    },
    habitats: vec![
        Habitat::Meadows,
        Habitat::MountainMeadows,
        Habitat::ForestEdges,
        Habitat::RiverValleys,
    ],
    grouping: GroupingDef {
        group_type: GroupType::Herd,
        size_min: 6,
        size_max: 20,
        flees_together: true,
    },
    harvest: HarvestDef {
        meat: 15,                // Substantial
        hide: 2,                 // Large hide
        antlers: Some(2),        // Bull elk only
        bones: 8,
        sinew: Some(4),
        tallow: Some(3),
        ..Default::default()
    },
    spawn_rate: 0.15,            // Rarer than deer
    active_times: vec![TimeOfDay::Dawn, TimeOfDay::Day, TimeOfDay::Dusk],
    seasonal_behavior: SeasonalBehavior {
        spring: SeasonState::CalvingGrounds,
        summer: SeasonState::HighMeadows,
        fall: SeasonState::Rut,           // Bulls bugling
        winter: SeasonState::ValleyHerds, // Congregate in valleys
    },
    flight_response: FlightResponse {
        trigger_distance: 25.0,
        zigzag_pattern: false,   // Run straight
        jump_obstacles: true,
        warns_others: true,      // Bark alarm call
        herd_stampede: true,     // Group flight
        river_crossing: true,    // Will cross water
    },
    unique_behavior: Some(UniqueBehavior::BugleCall), // Fall rut
};
```

### Elk Hunt Event Configuration

```rust
pub const ELK_HUNT_CONFIG: HerdHuntConfig = HerdHuntConfig {
    species: DocileSpecies::Elk,
    min_herd_size: 6,
    max_herd_size: 15,
    min_players: 2,
    max_players: 8,
    time_limit: Some(1800.0),  // 30 minutes
    success_threshold: 0.25,   // Take at least 25% for success

    // Elk-specific modifiers
    detection_multiplier: 1.2,  // More alert
    flee_speed_multiplier: 1.1, // Faster escape
    scatter_resistance: 0.8,    // Tend to stay together
};
```

### Elk Sign Types

```rust
/// Elk-specific tracking signs
pub enum ElkSign {
    /// Large hoofprints, distinctive split
    Tracks {
        freshness: f32,
        bull_or_cow: Option<Gender>,
        count_estimate: u32,
    },

    /// Elk droppings (larger than deer)
    Pellets {
        freshness: f32,
        group_size_hint: HerdSizeEstimate,
    },

    /// Bull rubs on trees (fall only)
    AntlerRub {
        height: f32,  // Indicates bull size
        freshness: f32,
    },

    /// Bugling heard (fall only)
    BugleCall {
        direction: Vec3,
        distance_estimate: f32,
    },

    /// Wallows (mud baths)
    Wallow {
        freshness: f32,
        hair_present: bool,
    },

    /// Bedding areas
    Beds {
        count: u32,
        warmth: f32,
    },
}
```

---

*End of Multiplayer Hunting Events Specification*
