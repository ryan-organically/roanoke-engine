# Animal System Specification
## Roanoke Engine Wildlife & Creature Framework

This document specifies the architecture for dangerous wildlife in Roanoke Engine, designed to integrate with the existing chunk-based world generation and wgpu rendering pipeline.

---

## Table of Contents
1. [Overview](#overview)
2. [Core Data Structures](#core-data-structures)
3. [Entity Management](#entity-management)
4. [AI Behavior System](#ai-behavior-system)
5. [Spawning System](#spawning-system)
6. [Combat System](#combat-system)
7. [Status Effects](#status-effects)
8. [Pack Behavior](#pack-behavior)
9. [Rendering Integration](#rendering-integration)
10. [Audio Integration](#audio-integration)
11. [Persistence](#persistence)
12. [Implementation Phases](#implementation-phases)

---

## Overview

### Design Goals
- **Immersive Wildlife**: Animals feel like natural inhabitants, not video game enemies
- **Emergent Behavior**: Simple rules create complex, believable interactions
- **Performance**: Support 50+ active animals without frame drops
- **Integration**: Seamlessly work with chunk streaming and existing systems

### Architectural Approach
Given the engine does NOT use an ECS framework, we implement a lightweight **Agent-Based Entity System** with:
- Central `AnimalManager` owning all animal instances
- Component-like data structs composed into animal entities
- Behavior via hierarchical state machines (HFSM)
- Spatial hashing for efficient queries

---

## Core Data Structures

### Location: `roanoke_game/src/animals/mod.rs`

```rust
//! Animal system module
//!
//! Submodules:
//!   - types.rs      - Animal species definitions
//!   - entity.rs     - Animal entity struct
//!   - manager.rs    - Central animal management
//!   - behavior.rs   - AI state machines
//!   - spawner.rs    - Chunk-based spawning
//!   - combat.rs     - Damage and attacks
//!   - effects.rs    - Status effects
//!   - packs.rs      - Pack coordination
```

### Animal Species Definition

```rust
// types.rs

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Unique identifier for animal species
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimalSpecies {
    BlackBear,
    EasternCougar,
    GrayWolf,
    TimberRattlesnake,
    AmericanAlligator,
    WildBoar,
    Copperhead,
    RedWolf,
    Bobcat,
    Cottonmouth,
}

/// Base stats for an animal species (before difficulty modifiers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimalStats {
    pub health: f32,
    pub damage: f32,
    pub speed: f32,
    pub speed_in_water: Option<f32>,  // For aquatic animals
    pub detection_range: f32,
    pub attack_range: f32,
}

/// Time periods when animal is active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeOfDay {
    Dawn,   // 5:00 - 8:00
    Day,    // 8:00 - 17:00
    Dusk,   // 17:00 - 20:00
    Night,  // 20:00 - 5:00
    Any,    // Always active
}

/// Primary behavior archetype
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorType {
    Territorial,   // Defends area, attacks if approached
    Stalker,       // Follows prey, ambushes
    PackHunter,    // Coordinates with pack members
    Ambush,        // Waits hidden, strikes when close
    Aggressive,    // Attacks on sight
    Hidden,        // Camouflaged, defensive only
}

/// Aggression response pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggressionType {
    Defensive,   // Only attacks if threatened/approached
    Predatory,   // Hunts player as prey
    Aggressive,  // Attacks readily
    Territorial, // Attacks in territory
    Cautious,    // Evaluates threat before engaging
}

/// Habitat types for spawn filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Habitat {
    Forests,
    Mountains,
    Swamps,
    Rivers,
    Marshes,
    Plains,
    RockyAreas,
    Meadows,
    Fields,
    CoastalPlains,
    NearWater,
}

/// Attack definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackDef {
    pub name: String,
    pub damage: f32,
    pub cooldown: f32,
    pub effect: Option<StatusEffectType>,
    pub range_override: Option<f32>,  // If different from base attack_range
    pub animation: String,
}

/// Weakness types (affects damage taken)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weakness {
    Fire,
    LoudNoises,
    Cold,
    Spears,
    Boots,       // Reduced damage from boots (snakes)
    Dogs,
    LongWeapons,
}

/// Complete species definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimalSpeciesDef {
    pub id: AnimalSpecies,
    pub name: String,
    pub scientific_name: String,
    pub danger_level: u8,              // 1-10
    pub habitats: Vec<Habitat>,
    pub behavior: BehaviorType,
    pub stats: AnimalStats,
    pub attacks: Vec<AttackDef>,
    pub loot: Vec<String>,
    pub weakness: Weakness,
    pub spawn_rate: f32,               // 0.0 - 1.0
    pub active_times: Vec<TimeOfDay>,
    pub aggression: AggressionType,
    pub pack_size: Option<(u8, u8)>,   // (min, max) if pack animal
    pub flee_health: f32,              // HP threshold to flee
}
```

### Difficulty Modifiers

```rust
// types.rs (continued)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Survival,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyModifiers {
    pub health_multiplier: f32,
    pub damage_multiplier: f32,
    pub spawn_rate_multiplier: f32,
}

impl Difficulty {
    pub fn modifiers(&self) -> DifficultyModifiers {
        match self {
            Difficulty::Easy => DifficultyModifiers {
                health_multiplier: 0.75,
                damage_multiplier: 0.75,
                spawn_rate_multiplier: 0.8,
            },
            Difficulty::Normal => DifficultyModifiers {
                health_multiplier: 1.0,
                damage_multiplier: 1.0,
                spawn_rate_multiplier: 1.0,
            },
            Difficulty::Hard => DifficultyModifiers {
                health_multiplier: 1.5,
                damage_multiplier: 1.25,
                spawn_rate_multiplier: 1.3,
            },
            Difficulty::Survival => DifficultyModifiers {
                health_multiplier: 2.0,
                damage_multiplier: 1.5,
                spawn_rate_multiplier: 1.5,
            },
        }
    }
}
```

---

## Entity Management

### Animal Entity

```rust
// entity.rs

use glam::{Vec3, Quat};
use std::time::Instant;

/// Unique runtime identifier for animal instances
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimalId(pub u64);

/// Runtime state for a single animal
#[derive(Debug)]
pub struct Animal {
    // Identity
    pub id: AnimalId,
    pub species: AnimalSpecies,
    pub pack_id: Option<PackId>,

    // Transform
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,

    // Physics
    pub on_ground: bool,
    pub in_water: bool,

    // Combat state
    pub current_health: f32,
    pub max_health: f32,
    pub active_effects: Vec<ActiveStatusEffect>,
    pub attack_cooldowns: Vec<f32>,  // Indexed by attack
    pub last_damage_time: Option<Instant>,
    pub damage_source: Option<DamageSource>,

    // AI state
    pub behavior_state: BehaviorState,
    pub target: Option<Target>,
    pub home_position: Vec3,         // Spawn location for territorial behavior
    pub territory_radius: f32,
    pub awareness: f32,              // 0.0 = unaware, 1.0 = fully alert
    pub last_seen_player: Option<(Vec3, Instant)>,

    // Animation
    pub animation_state: AnimationState,
    pub animation_time: f32,

    // Spawning
    pub spawn_chunk: ChunkCoord,
    pub despawn_timer: Option<f32>,  // Countdown when far from player
}

/// What the animal is currently targeting
#[derive(Debug, Clone)]
pub enum Target {
    Player,
    Position(Vec3),
    Animal(AnimalId),
    FleeFrom(Vec3),
}

/// Who dealt damage
#[derive(Debug, Clone)]
pub enum DamageSource {
    Player,
    Animal(AnimalId),
    Environment,
    StatusEffect,
}
```

### Animal Manager

```rust
// manager.rs

use std::collections::HashMap;
use crate::spatial::SpatialHash;

/// Central manager for all animal entities
pub struct AnimalManager {
    // Entity storage
    animals: HashMap<AnimalId, Animal>,
    next_id: u64,

    // Spatial indexing (cell size = 16 units)
    spatial_hash: SpatialHash<AnimalId>,

    // Species data (loaded from config)
    species_defs: HashMap<AnimalSpecies, AnimalSpeciesDef>,

    // Pack tracking
    packs: HashMap<PackId, Pack>,
    next_pack_id: u64,

    // Global state
    difficulty: Difficulty,
    time_of_day: TimeOfDay,

    // Statistics
    total_spawned: u64,
    total_killed: u64,
}

impl AnimalManager {
    pub fn new(difficulty: Difficulty) -> Self { ... }

    /// Spawn a new animal, returns its ID
    pub fn spawn(&mut self, species: AnimalSpecies, position: Vec3, pack_id: Option<PackId>) -> AnimalId { ... }

    /// Despawn an animal
    pub fn despawn(&mut self, id: AnimalId) { ... }

    /// Get animal by ID
    pub fn get(&self, id: AnimalId) -> Option<&Animal> { ... }
    pub fn get_mut(&mut self, id: AnimalId) -> Option<&mut Animal> { ... }

    /// Query animals in radius
    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<AnimalId> { ... }

    /// Query animals in chunk
    pub fn query_chunk(&self, chunk: ChunkCoord) -> Vec<AnimalId> { ... }

    /// Main update tick
    pub fn update(&mut self, dt: f32, player: &Player, world: &World) { ... }

    /// Handle damage to animal
    pub fn damage_animal(&mut self, id: AnimalId, amount: f32, source: DamageSource) { ... }

    /// Set time of day (affects spawning and behavior)
    pub fn set_time_of_day(&mut self, time: TimeOfDay) { ... }
}
```

### Spatial Hashing

```rust
// spatial.rs

use std::collections::{HashMap, HashSet};
use glam::Vec3;

/// Spatial hash grid for efficient proximity queries
pub struct SpatialHash<T: Copy + Eq + std::hash::Hash> {
    cell_size: f32,
    cells: HashMap<(i32, i32), HashSet<T>>,
}

impl<T: Copy + Eq + std::hash::Hash> SpatialHash<T> {
    pub fn new(cell_size: f32) -> Self { ... }

    fn cell_coord(&self, pos: Vec3) -> (i32, i32) {
        ((pos.x / self.cell_size).floor() as i32,
         (pos.z / self.cell_size).floor() as i32)
    }

    pub fn insert(&mut self, id: T, pos: Vec3) { ... }
    pub fn remove(&mut self, id: T, pos: Vec3) { ... }
    pub fn update(&mut self, id: T, old_pos: Vec3, new_pos: Vec3) { ... }

    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<T> { ... }
}
```

---

## AI Behavior System

### Hierarchical Finite State Machine (HFSM)

```rust
// behavior.rs

/// High-level behavior states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorState {
    Idle(IdleState),
    Patrol(PatrolState),
    Alert(AlertState),
    Pursue(PursueState),
    Attack(AttackState),
    Flee(FleeState),
    Dead,
}

/// Idle sub-states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleState {
    Standing,
    Resting,
    Eating,
    Drinking,
    Grooming,
}

/// Patrol sub-states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatrolState {
    Walking,
    Investigating,
    Returning,  // Returning to territory
}

/// Alert sub-states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertState {
    Listening,
    Looking,
    Sniffing,
    Warning,  // Threat display (rattlesnake rattle, bear standing)
}

/// Pursue sub-states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PursueState {
    Chasing,
    Stalking,    // For stalker behavior type
    Circling,    // Pack flanking
    Closing,     // Final approach
}

/// Attack sub-states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackState {
    WindingUp,
    Striking,
    Recovering,
}

/// Flee sub-states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FleeState {
    Running,
    Hiding,
    Cornered,  // Will fight back
}
```

### Behavior Update Logic

```rust
// behavior.rs (continued)

pub struct BehaviorContext<'a> {
    pub animal: &'a Animal,
    pub species: &'a AnimalSpeciesDef,
    pub player: &'a Player,
    pub pack: Option<&'a Pack>,
    pub nearby_animals: &'a [AnimalId],
    pub dt: f32,
}

pub fn update_behavior(ctx: &BehaviorContext, animal: &mut Animal) {
    // Calculate player distance and visibility
    let player_dist = animal.position.distance(ctx.player.position);
    let can_see_player = check_line_of_sight(animal, ctx.player);
    let player_in_range = player_dist < ctx.species.stats.detection_range;

    // Update awareness (gradual alert/relax)
    update_awareness(animal, player_in_range, can_see_player, ctx.dt);

    // State machine transitions
    let new_state = match animal.behavior_state {
        BehaviorState::Idle(_) => {
            if animal.current_health <= ctx.species.flee_health {
                BehaviorState::Flee(FleeState::Running)
            } else if animal.awareness > 0.8 && should_attack(ctx) {
                BehaviorState::Pursue(PursueState::Chasing)
            } else if animal.awareness > 0.3 {
                BehaviorState::Alert(AlertState::Looking)
            } else {
                maybe_start_patrol(animal, ctx)
            }
        },

        BehaviorState::Alert(_) => {
            if animal.awareness < 0.2 {
                BehaviorState::Idle(IdleState::Standing)
            } else if animal.awareness > 0.9 && should_attack(ctx) {
                BehaviorState::Pursue(PursueState::Chasing)
            } else {
                animal.behavior_state  // Stay alert
            }
        },

        BehaviorState::Pursue(_) => {
            if animal.current_health <= ctx.species.flee_health {
                BehaviorState::Flee(FleeState::Running)
            } else if player_dist < ctx.species.stats.attack_range {
                BehaviorState::Attack(AttackState::WindingUp)
            } else if player_dist > ctx.species.stats.detection_range * 2.0 {
                BehaviorState::Alert(AlertState::Looking)
            } else {
                animal.behavior_state
            }
        },

        BehaviorState::Attack(sub) => {
            update_attack_state(animal, sub, ctx)
        },

        BehaviorState::Flee(sub) => {
            update_flee_state(animal, sub, ctx)
        },

        BehaviorState::Dead => BehaviorState::Dead,

        _ => animal.behavior_state,
    };

    animal.behavior_state = new_state;

    // Execute current state behavior
    execute_state(animal, ctx);
}

fn should_attack(ctx: &BehaviorContext) -> bool {
    match ctx.species.aggression {
        AggressionType::Predatory => true,
        AggressionType::Aggressive => true,
        AggressionType::Territorial => {
            let dist_from_home = ctx.animal.position.distance(ctx.animal.home_position);
            dist_from_home < ctx.animal.territory_radius
        },
        AggressionType::Defensive => {
            ctx.animal.last_damage_time.is_some()
        },
        AggressionType::Cautious => {
            // Only attack if confident (pack support, player wounded, etc.)
            ctx.pack.map(|p| p.members.len() >= 2).unwrap_or(false)
        },
    }
}
```

### Stalker Behavior (Cougar)

```rust
// behavior.rs (continued)

fn update_stalker_behavior(animal: &mut Animal, ctx: &BehaviorContext) {
    let player_dist = animal.position.distance(ctx.player.position);

    match animal.behavior_state {
        BehaviorState::Pursue(PursueState::Stalking) => {
            // Stay at detection range edge, follow player
            let ideal_dist = ctx.species.stats.detection_range * 0.8;

            // Move to flanking position
            let to_player = (ctx.player.position - animal.position).normalize();
            let flank_angle = (animal.id.0 as f32 * 0.1).sin() * 0.5;  // Vary by individual
            let flank_dir = rotate_y(to_player, flank_angle);

            // If player isn't looking, close distance
            let player_facing = player_forward(ctx.player);
            let angle_to_animal = to_player.dot(-player_facing);

            if angle_to_animal < 0.3 {  // Player not looking
                // Pounce opportunity!
                if player_dist < ctx.species.stats.attack_range * 1.5 {
                    animal.behavior_state = BehaviorState::Attack(AttackState::WindingUp);
                } else {
                    // Close in
                    animal.target = Some(Target::Position(ctx.player.position));
                    set_speed(animal, ctx.species.stats.speed * 0.7);  // Quiet approach
                }
            } else {
                // Freeze or retreat
                if player_dist < ideal_dist {
                    animal.target = Some(Target::FleeFrom(ctx.player.position));
                    set_speed(animal, ctx.species.stats.speed * 0.5);
                }
            }
        },
        _ => {}
    }
}
```

---

## Spawning System

### Chunk-Based Spawning

```rust
// spawner.rs

use crate::chunk_manager::{ChunkCoord, ChunkManager};
use noise::{NoiseFn, Perlin};

pub struct AnimalSpawner {
    spawn_noise: Perlin,
    spawn_density_base: f32,      // Animals per chunk
    max_animals_per_chunk: u8,
    min_player_distance: f32,     // Don't spawn too close
    max_active_animals: usize,    // Global cap
}

impl AnimalSpawner {
    /// Called when a chunk finishes loading
    pub fn on_chunk_loaded(
        &self,
        chunk: ChunkCoord,
        chunk_data: &LoadedChunk,
        manager: &mut AnimalManager,
        player_pos: Vec3,
        seed: u32,
    ) {
        // Check global cap
        if manager.animal_count() >= self.max_active_animals {
            return;
        }

        // Determine biome/habitat from chunk
        let habitats = determine_chunk_habitats(chunk, chunk_data, seed);

        // Get eligible species for this chunk
        let time = manager.time_of_day;
        let eligible: Vec<&AnimalSpeciesDef> = manager.species_defs.values()
            .filter(|s| {
                s.habitats.iter().any(|h| habitats.contains(h)) &&
                (s.active_times.contains(&time) || s.active_times.contains(&TimeOfDay::Any))
            })
            .collect();

        if eligible.is_empty() {
            return;
        }

        // Determine spawn positions in chunk
        let chunk_center = chunk_to_world(chunk);
        let spawn_positions = generate_spawn_positions(chunk, seed, 8);  // Up to 8 potential spots

        for pos in spawn_positions {
            // Distance check
            if pos.distance(player_pos) < self.min_player_distance {
                continue;
            }

            // Weighted species selection
            let species = select_species(&eligible, pos, seed, &manager.difficulty);

            if let Some(species) = species {
                // Pack spawning
                if let Some((min, max)) = species.pack_size {
                    let pack_size = seeded_range(seed + pos.x as u32, min, max);
                    let pack_id = manager.create_pack(species.id);

                    for i in 0..pack_size {
                        let offset = Vec3::new(
                            seeded_float(seed + i as u32) * 5.0 - 2.5,
                            0.0,
                            seeded_float(seed + i as u32 + 100) * 5.0 - 2.5,
                        );
                        manager.spawn(species.id, pos + offset, Some(pack_id));
                    }
                } else {
                    manager.spawn(species.id, pos, None);
                }
            }
        }
    }

    /// Called when chunk unloads
    pub fn on_chunk_unloaded(&self, chunk: ChunkCoord, manager: &mut AnimalManager) {
        // Mark animals for despawn (with grace period)
        for id in manager.query_chunk(chunk) {
            if let Some(animal) = manager.get_mut(id) {
                animal.despawn_timer = Some(30.0);  // 30 second grace period
            }
        }
    }
}

fn determine_chunk_habitats(chunk: ChunkCoord, data: &LoadedChunk, seed: u32) -> Vec<Habitat> {
    let mut habitats = Vec::new();

    // Sample heights in chunk to determine terrain type
    let avg_height = data.bounds.average_height();
    let moisture = sample_moisture(chunk, seed);
    let has_water = data.has_water_bodies();

    // Height-based classification
    if avg_height > 50.0 {
        habitats.push(Habitat::Mountains);
    }
    if avg_height < 30.0 && moisture > 0.6 {
        habitats.push(Habitat::Swamps);
        habitats.push(Habitat::Marshes);
    }
    if avg_height >= 20.0 && avg_height <= 45.0 {
        habitats.push(Habitat::Forests);
    }
    if avg_height < 25.0 && moisture < 0.4 {
        habitats.push(Habitat::Plains);
        habitats.push(Habitat::Fields);
    }

    // Feature-based
    if has_water {
        habitats.push(Habitat::Rivers);
        habitats.push(Habitat::NearWater);
    }
    if data.has_rocky_terrain() {
        habitats.push(Habitat::RockyAreas);
    }
    if avg_height < 15.0 && moisture > 0.3 {
        habitats.push(Habitat::CoastalPlains);
    }

    habitats
}
```

---

## Combat System

### Damage Calculation

```rust
// combat.rs

pub struct DamageEvent {
    pub target: DamageTarget,
    pub amount: f32,
    pub damage_type: DamageType,
    pub source: DamageSource,
    pub effect: Option<StatusEffectType>,
    pub knockback: Option<Vec3>,
}

pub enum DamageTarget {
    Player,
    Animal(AnimalId),
}

pub enum DamageType {
    Physical,
    Poison,
    Bleed,
}

impl AnimalManager {
    pub fn process_attack(&mut self, attacker_id: AnimalId, player: &mut Player) -> Option<DamageEvent> {
        let animal = self.get(attacker_id)?;
        let species = self.species_defs.get(&animal.species)?;

        // Check attack range
        let dist = animal.position.distance(player.position);

        // Select attack (prioritize special attacks off cooldown)
        let attack_idx = self.select_attack(attacker_id, dist)?;
        let attack = &species.attacks[attack_idx];

        // Check cooldown
        let animal_mut = self.get_mut(attacker_id)?;
        if animal_mut.attack_cooldowns[attack_idx] > 0.0 {
            return None;
        }

        // Calculate damage with difficulty modifier
        let base_damage = attack.damage;
        let modified_damage = base_damage * self.difficulty.modifiers().damage_multiplier;

        // Set cooldown
        animal_mut.attack_cooldowns[attack_idx] = attack.cooldown;

        // Create damage event
        Some(DamageEvent {
            target: DamageTarget::Player,
            amount: modified_damage,
            damage_type: DamageType::Physical,
            source: DamageSource::Animal(attacker_id),
            effect: attack.effect,
            knockback: calculate_knockback(&attack.name, animal.position, player.position),
        })
    }

    /// Player attacks animal
    pub fn player_attack(&mut self, id: AnimalId, damage: f32, weapon: &Weapon) -> bool {
        let animal = match self.get_mut(id) {
            Some(a) => a,
            None => return false,
        };

        let species = match self.species_defs.get(&animal.species) {
            Some(s) => s,
            None => return false,
        };

        // Weakness check
        let final_damage = if weapon_matches_weakness(weapon, species.weakness) {
            damage * 1.5  // 50% bonus damage
        } else {
            damage
        };

        animal.current_health -= final_damage;
        animal.last_damage_time = Some(Instant::now());
        animal.damage_source = Some(DamageSource::Player);

        // Alert nearby pack members
        if let Some(pack_id) = animal.pack_id {
            self.alert_pack(pack_id, animal.position);
        }

        // Check death
        if animal.current_health <= 0.0 {
            animal.behavior_state = BehaviorState::Dead;
            self.total_killed += 1;
            return true;  // Killed
        }

        false
    }
}

fn weapon_matches_weakness(weapon: &Weapon, weakness: Weakness) -> bool {
    match weakness {
        Weakness::Fire => weapon.has_fire_damage(),
        Weakness::Spears => weapon.weapon_type == WeaponType::Spear,
        Weakness::LongWeapons => weapon.reach > 2.0,
        Weakness::Boots => false,  // Passive defense
        _ => false,
    }
}
```

### Attack Definitions (From JSON Data)

```rust
// attacks.rs

pub fn get_species_attacks(species: AnimalSpecies) -> Vec<AttackDef> {
    match species {
        AnimalSpecies::BlackBear => vec![
            AttackDef {
                name: "claw_swipe".into(),
                damage: 25.0,
                cooldown: 2.0,
                effect: Some(StatusEffectType::Bleeding),
                range_override: None,
                animation: "bear_swipe".into(),
            },
            AttackDef {
                name: "bite".into(),
                damage: 30.0,
                cooldown: 3.0,
                effect: None,
                range_override: None,
                animation: "bear_bite".into(),
            },
        ],

        AnimalSpecies::EasternCougar => vec![
            AttackDef {
                name: "pounce".into(),
                damage: 35.0,
                cooldown: 4.0,
                effect: Some(StatusEffectType::Knockdown),
                range_override: Some(3.0),  // Extended range for leap
                animation: "cougar_pounce".into(),
            },
            AttackDef {
                name: "throat_bite".into(),
                damage: 40.0,
                cooldown: 5.0,
                effect: Some(StatusEffectType::Bleeding),
                range_override: None,
                animation: "cougar_bite".into(),
            },
        ],

        AnimalSpecies::GrayWolf => vec![
            AttackDef {
                name: "bite".into(),
                damage: 20.0,
                cooldown: 1.5,
                effect: None,
                range_override: None,
                animation: "wolf_bite".into(),
            },
            AttackDef {
                name: "pack_howl".into(),
                damage: 0.0,
                cooldown: 30.0,
                effect: Some(StatusEffectType::SummonPack),
                range_override: None,
                animation: "wolf_howl".into(),
            },
        ],

        AnimalSpecies::AmericanAlligator => vec![
            AttackDef {
                name: "death_roll".into(),
                damage: 50.0,
                cooldown: 6.0,
                effect: Some(StatusEffectType::Stun),
                range_override: None,
                animation: "gator_roll".into(),
            },
            AttackDef {
                name: "tail_whip".into(),
                damage: 25.0,
                cooldown: 3.0,
                effect: Some(StatusEffectType::Knockback),
                range_override: Some(2.5),  // Wide arc
                animation: "gator_tail".into(),
            },
            AttackDef {
                name: "crushing_bite".into(),
                damage: 40.0,
                cooldown: 4.0,
                effect: None,
                range_override: None,
                animation: "gator_bite".into(),
            },
        ],

        // ... other species
        _ => vec![],
    }
}
```

---

## Status Effects

```rust
// effects.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusEffectType {
    Bleeding,
    Poison,
    Knockdown,
    Knockback,
    Stun,
    Fear,
    Slow,
    SummonPack,
    Intimidate,
}

#[derive(Debug, Clone)]
pub struct StatusEffectDef {
    pub duration: f32,
    pub damage_per_second: f32,
    pub movement_modifier: f32,  // 1.0 = normal, 0.5 = half speed
    pub can_act: bool,
    pub description: &'static str,
}

impl StatusEffectType {
    pub fn definition(&self) -> StatusEffectDef {
        match self {
            StatusEffectType::Bleeding => StatusEffectDef {
                duration: 10.0,
                damage_per_second: 2.0,
                movement_modifier: 1.0,
                can_act: true,
                description: "Causes damage over time",
            },
            StatusEffectType::Poison => StatusEffectDef {
                duration: 15.0,
                damage_per_second: 3.0,
                movement_modifier: 0.8,  // Slight slow
                can_act: true,
                description: "Causes damage over time and slows movement",
            },
            StatusEffectType::Knockdown => StatusEffectDef {
                duration: 2.0,
                damage_per_second: 0.0,
                movement_modifier: 0.0,
                can_act: false,
                description: "Player is knocked to the ground",
            },
            StatusEffectType::Knockback => StatusEffectDef {
                duration: 0.5,
                damage_per_second: 0.0,
                movement_modifier: 0.0,  // Momentum takes over
                can_act: false,
                description: "Player is pushed back",
            },
            StatusEffectType::Stun => StatusEffectDef {
                duration: 3.0,
                damage_per_second: 0.0,
                movement_modifier: 0.0,
                can_act: false,
                description: "Player cannot move or act",
            },
            StatusEffectType::Fear => StatusEffectDef {
                duration: 2.0,
                damage_per_second: 0.0,
                movement_modifier: 1.2,  // Faster but erratic
                can_act: true,  // Can act but with penalties
                description: "Player movement is erratic",
            },
            StatusEffectType::Slow => StatusEffectDef {
                duration: 5.0,
                damage_per_second: 0.0,
                movement_modifier: 0.5,
                can_act: true,
                description: "Player movement speed reduced by 50%",
            },
            StatusEffectType::SummonPack => StatusEffectDef {
                duration: 0.0,  // Instant
                damage_per_second: 0.0,
                movement_modifier: 1.0,
                can_act: true,
                description: "Calls nearby pack members to assist",
            },
            StatusEffectType::Intimidate => StatusEffectDef {
                duration: 1.0,
                damage_per_second: 0.0,
                movement_modifier: 1.0,
                can_act: true,
                description: "Player accuracy reduced",
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActiveStatusEffect {
    pub effect_type: StatusEffectType,
    pub remaining_duration: f32,
    pub source: DamageSource,
}

pub fn update_status_effects(player: &mut Player, effects: &mut Vec<ActiveStatusEffect>, dt: f32) {
    let mut total_dps = 0.0;
    let mut speed_modifier = 1.0;
    let mut can_act = true;

    effects.retain_mut(|effect| {
        let def = effect.effect_type.definition();

        total_dps += def.damage_per_second;
        speed_modifier = speed_modifier.min(def.movement_modifier);
        can_act = can_act && def.can_act;

        effect.remaining_duration -= dt;
        effect.remaining_duration > 0.0
    });

    // Apply aggregated effects
    if total_dps > 0.0 {
        player.take_damage(total_dps * dt, DamageSource::StatusEffect);
    }
    player.speed_modifier = speed_modifier;
    player.can_act = can_act;
}
```

---

## Pack Behavior

```rust
// packs.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackId(pub u64);

#[derive(Debug)]
pub struct Pack {
    pub id: PackId,
    pub species: AnimalSpecies,
    pub members: Vec<AnimalId>,
    pub alpha: Option<AnimalId>,
    pub target: Option<Target>,
    pub formation: PackFormation,
    pub morale: f32,  // 0.0 = flee, 1.0 = aggressive
}

#[derive(Debug, Clone, Copy)]
pub enum PackFormation {
    Scattered,    // Default wandering
    Grouped,      // Close together
    Flanking,     // Surrounding target
    Retreating,   // Fleeing together
}

impl Pack {
    /// Update pack coordination
    pub fn update(&mut self, manager: &AnimalManager, player: &Player, dt: f32) {
        // Calculate morale based on pack health
        let total_health: f32 = self.members.iter()
            .filter_map(|id| manager.get(*id))
            .map(|a| a.current_health / a.max_health)
            .sum();
        let avg_health = total_health / self.members.len() as f32;

        // Count dead members
        let alive_count = self.members.iter()
            .filter(|id| manager.get(**id).map(|a| a.current_health > 0.0).unwrap_or(false))
            .count();

        // Morale drops with casualties and injuries
        self.morale = (avg_health * 0.5 + (alive_count as f32 / self.members.len() as f32) * 0.5)
            .clamp(0.0, 1.0);

        // Determine formation
        self.formation = if self.morale < 0.3 {
            PackFormation::Retreating
        } else if self.target.is_some() && self.morale > 0.6 {
            PackFormation::Flanking
        } else {
            PackFormation::Scattered
        };

        // Elect alpha (highest health member)
        self.alpha = self.members.iter()
            .filter_map(|id| manager.get(*id).map(|a| (*id, a.current_health)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| id);
    }

    /// Get position offset for pack member in formation
    pub fn get_formation_offset(&self, member_idx: usize) -> Vec3 {
        let angle = (member_idx as f32 / self.members.len() as f32) * std::f32::consts::TAU;
        let radius = match self.formation {
            PackFormation::Scattered => 10.0,
            PackFormation::Grouped => 3.0,
            PackFormation::Flanking => 8.0,
            PackFormation::Retreating => 5.0,
        };

        Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius)
    }
}

/// Gray Wolf pack howl summons nearby pack members
pub fn process_pack_howl(manager: &mut AnimalManager, howler_id: AnimalId) {
    let howler = match manager.get(howler_id) {
        Some(a) => a,
        None => return,
    };

    let pack_id = match howler.pack_id {
        Some(id) => id,
        None => return,
    };

    let howl_pos = howler.position;
    let howl_range = 100.0;  // Audible range

    // Alert all pack members
    if let Some(pack) = manager.packs.get(&pack_id) {
        for member_id in &pack.members {
            if *member_id != howler_id {
                if let Some(member) = manager.get_mut(*member_id) {
                    member.awareness = 1.0;
                    member.target = Some(Target::Position(howl_pos));
                    member.behavior_state = BehaviorState::Pursue(PursueState::Chasing);
                }
            }
        }
    }
}
```

---

## Rendering Integration

### Animal Pipeline

```rust
// In crates/croatoan_render/src/animal_pipeline.rs

pub struct AnimalPipeline {
    pipeline: wgpu::RenderPipeline,
    mesh_cache: HashMap<AnimalSpecies, AnimalMesh>,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
}

pub struct AnimalMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    skeleton: Option<Skeleton>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AnimalInstance {
    pub model_matrix: [[f32; 4]; 4],
    pub animation_data: [f32; 4],  // time, state, blend, _padding
    pub tint: [f32; 4],            // For damage flash, etc.
}

impl AnimalPipeline {
    pub fn update_instances(&mut self, animals: &[&Animal], queue: &wgpu::Queue) {
        let instances: Vec<AnimalInstance> = animals.iter()
            .filter(|a| a.behavior_state != BehaviorState::Dead || a.animation_time < 3.0)
            .map(|a| AnimalInstance {
                model_matrix: Mat4::from_rotation_translation(
                    a.rotation,
                    a.position,
                ).to_cols_array_2d(),
                animation_data: [
                    a.animation_time,
                    a.animation_state as u32 as f32,
                    0.0,
                    0.0,
                ],
                tint: if a.last_damage_time.map(|t| t.elapsed().as_secs_f32() < 0.1).unwrap_or(false) {
                    [1.0, 0.3, 0.3, 1.0]  // Damage flash
                } else {
                    [1.0, 1.0, 1.0, 1.0]
                },
            })
            .collect();

        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, species: AnimalSpecies, count: u32) {
        if let Some(mesh) = self.mesh_cache.get(&species) {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..count);
        }
    }
}
```

### Integration with ChunkManager

```rust
// In roanoke_game/src/chunk_manager.rs (additions)

pub struct LoadedChunk {
    pub terrain: TerrainPipeline,
    pub grass: Option<GrassPipeline>,
    pub trees: Option<TreePipeline>,
    pub detritus: Option<DetritusPipeline>,
    pub rocks: Vec<TreePipeline>,
    pub buildings: Vec<BuildingPipeline>,
    pub bounds: ChunkBounds,
    // NEW: Animal tracking per chunk (not rendering, just tracking)
    pub animal_ids: Vec<AnimalId>,
}
```

---

## Audio Integration

```rust
// audio.rs (additions)

pub enum AnimalSound {
    // Bear
    BearGrowl,
    BearRoar,
    BearWalking,

    // Cougar
    CougarScream,
    CougarGrowl,
    CougarPurr,

    // Wolf
    WolfHowl,
    WolfGrowl,
    WolfWhimper,
    WolfBark,

    // Snake
    SnakeRattle,
    SnakeHiss,
    SnakeStrike,

    // Alligator
    AlligatorBellow,
    AlligatorHiss,
    AlligatorSplash,

    // Boar
    BoarSqueal,
    BoarGrunt,
    BoarCharge,

    // Bobcat
    BobcatYowl,
    BobcatHiss,

    // Generic
    AnimalFootsteps,
    AnimalDeath,
    AnimalFlee,
}

impl AnimalManager {
    pub fn get_audio_events(&self) -> Vec<(Vec3, AnimalSound, f32)> {
        let mut events = Vec::new();

        for animal in self.animals.values() {
            // State-based sounds
            match animal.behavior_state {
                BehaviorState::Alert(AlertState::Warning) => {
                    let sound = match animal.species {
                        AnimalSpecies::TimberRattlesnake => AnimalSound::SnakeRattle,
                        AnimalSpecies::BlackBear => AnimalSound::BearGrowl,
                        AnimalSpecies::GrayWolf => AnimalSound::WolfGrowl,
                        _ => continue,
                    };
                    events.push((animal.position, sound, 1.0));
                },
                BehaviorState::Attack(AttackState::WindingUp) => {
                    // Attack sounds
                },
                _ => {},
            }
        }

        events
    }
}
```

---

## Persistence

```rust
// In roanoke_game/src/main.rs (SaveData additions)

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SaveData {
    seed: u32,
    player_pos: [f32; 3],
    player_rot: [f32; 2],
    inventory: Vec<String>,
    // NEW: Animal persistence
    animal_state: AnimalSaveState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AnimalSaveState {
    /// Active animals near player (serialized fully)
    active_animals: Vec<SerializedAnimal>,
    /// Killed animals (to prevent respawn)
    killed_animals: Vec<KilledAnimalRecord>,
    /// Pack data
    packs: Vec<SerializedPack>,
    /// Global statistics
    total_kills: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SerializedAnimal {
    species: AnimalSpecies,
    position: [f32; 3],
    health: f32,
    home_position: [f32; 3],
    pack_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct KilledAnimalRecord {
    species: AnimalSpecies,
    position: [f32; 3],
    kill_time: u64,  // Game time ticks
}
```

---

## Implementation Phases

### Phase 1: Foundation (Core Systems) ✅ COMPLETE
- [x] Create `roanoke_game/src/animals/` module structure
- [x] Implement `AnimalSpecies`, `AnimalStats`, core type definitions
- [x] Implement `SpatialHash` for efficient queries
- [x] Implement `AnimalManager` with basic CRUD operations
- [x] Add `AnimalId` generation and tracking

### Phase 2: Spawning ✅ COMPLETE
- [x] Implement `AnimalSpawner` with chunk integration
- [x] Add habitat detection from terrain data (noise-based)
- [x] Implement time-of-day filtering
- [x] Add pack spawning logic (wolves spawn in groups)
- [x] Integrate with `ChunkManager::on_chunk_loaded`

### Phase 3: Basic AI ✅ COMPLETE
- [x] Implement `BehaviorState` enum and transitions (HFSM)
- [x] Add idle/patrol behavior
- [x] Implement detection and awareness system
- [x] Add basic pursue and flee states
- [x] Integrate with player position updates

### Phase 4: Combat 🚧 PARTIAL
- [x] Implement attack system with cooldowns (defined in types.rs)
- [x] Add damage events and processing (damage_animal in manager.rs)
- [x] Implement status effects system (ActiveStatusEffect in entity.rs)
- [ ] Add weakness/resistance calculations
- [ ] Player damage feedback (screen effects, sounds)
- [ ] Connect combat.rs to player damage system

### Phase 5: Advanced AI ✅ COMPLETE
- [x] Implement stalker behavior (cougar) - follows at detection edge, pounces when back turned
- [x] Implement pack coordination (wolves) - morale, alpha election, alert propagation
- [x] Add territorial behavior (bear, boar) - defends home area
- [x] Implement ambush behavior (snakes, alligator) - waits hidden, strikes when close
- [x] Add flee behavior with cornered fighting

### Phase 6: Rendering ⏳ PLANNED
- [ ] Create `AnimalPipeline` in `croatoan_render`
- [ ] Implement basic mesh rendering (placeholder cubes initially)
- [ ] Add instance batching for performance
- [ ] Implement basic animation state machine
- [ ] Add damage flash and death effects

### Phase 7: Polish 🚧 PARTIAL
- [ ] Audio integration (growls, howls, footsteps)
- [ ] Persistence (save/load animal state)
- [x] Difficulty modifiers (Easy/Normal/Hard/Survival scaling)
- [ ] Loot drops on death
- [ ] Performance optimization and profiling
- [x] Debug UI integration (shows animal counts and nearby info)

---

## Species Reference Table

| Species | HP | DMG | SPD | Range | Danger | Behavior | Pack |
|---------|-----|-----|-----|-------|--------|----------|------|
| Black Bear | 150 | 25 | 35 | 20 | 7 | Territorial | No |
| Eastern Cougar | 100 | 30 | 50 | 30 | 8 | Stalker | No |
| Gray Wolf | 80 | 20 | 45 | 25 | 6 | Pack Hunter | 2-6 |
| Timber Rattlesnake | 30 | 15 | 15 | 10 | 5 | Ambush | No |
| American Alligator | 200 | 40 | 20/35 | 15 | 9 | Ambush | No |
| Wild Boar | 90 | 18 | 30 | 15 | 4 | Aggressive | No |
| Copperhead | 25 | 12 | 12 | 8 | 3 | Hidden | No |
| Red Wolf | 70 | 18 | 42 | 22 | 5 | Pack Hunter | 2-4 |
| Bobcat | 60 | 15 | 40 | 20 | 3 | Stalker | No |
| Cottonmouth | 35 | 16 | 10/18 | 12 | 4 | Aggressive | No |

---

## Configuration File Location

Animal data should be loaded from: `assets/data/animals.json`

This allows designers to tweak values without recompilation.

---

## Current Implementation Status

**Last Updated:** December 2024

### Summary

The animal system core is fully implemented and integrated into the game loop:

| Component | Status | Location |
|-----------|--------|----------|
| Types & Species | ✅ Complete | `animals/types.rs` |
| Entity Management | ✅ Complete | `animals/entity.rs`, `animals/manager.rs` |
| Spatial Queries | ✅ Complete | `animals/spatial.rs` |
| AI Behaviors | ✅ Complete | `animals/behavior.rs` |
| Spawning | ✅ Complete | `animals/spawner.rs` |
| Combat | 🚧 Partial | `animals/combat.rs` (not connected) |
| Rendering | ⏳ Planned | — |
| Audio | ⏳ Planned | — |

### Running the System

Animals spawn automatically when terrain chunks load. Debug information is visible in the Dev Stats panel (press F3):

```
Animals: 12 | Packs: 2 | Spawned: 45 | Killed: 3
Nearby (50m): 2
  Gray Wolf @ 23m - Pursue(Chasing)
  Gray Wolf @ 31m - Pursue(Circling)
```

### Documentation

For detailed API usage and implementation notes, see: `roanoke_game/src/animals/README.md`
