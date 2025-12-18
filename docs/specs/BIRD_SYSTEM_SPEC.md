# Bird System Specification

## Roanoke Engine Avian Framework - Flight, Flocking & Perching

This document specifies the architecture for birds in Roanoke Engine, extending the existing animal system with 3D flight mechanics, perching behaviors, and flock dynamics.

---

## Table of Contents

1. [Overview](#overview)
2. [Core Data Structures](#core-data-structures)
3. [Species Definitions](#species-definitions)
4. [Flight Physics System](#flight-physics-system)
5. [Wing Animation System](#wing-animation-system)
6. [Behavior State Machine](#behavior-state-machine)
7. [Perching System](#perching-system)
8. [Flock Behavior (Boids)](#flock-behavior-boids)
9. [Spawning Integration](#spawning-integration)
10. [Rendering Pipeline](#rendering-pipeline)
11. [Audio Integration](#audio-integration)
12. [Performance Considerations](#performance-considerations)
13. [Implementation Phases](#implementation-phases)
14. [Testing Checklist](#testing-checklist)

---

## Overview

### Design Goals

- **Believable Flight**: Physics-based flight with lift, drag, and banking
- **Atmospheric Life**: Birds add vertical dimension to world ambiance
- **Hunting Targets**: Some species huntable (turkeys, ducks, geese)
- **Ecosystem Depth**: Raptors hunt small fauna, songbirds flee predators
- **Performance**: Support 100+ birds visible without frame drops

### System Architecture

```
                           BIRD SYSTEM
                               |
       +-----------------------+-----------------------+
       |                       |                       |
       v                       v                       v
  FLIGHT PHYSICS         BEHAVIOR AI            FLOCK MANAGER
  (lift, drag, bank)     (HFSM states)          (boids algorithm)
       |                       |                       |
       v                       v                       v
  WING ANIMATION         PERCH SYSTEM           SPATIAL HASH
  (flap, glide, fold)    (tree branches)        (neighbor queries)
       |                       |                       |
       +----------+------------+----------+------------+
                  |                       |
                  v                       v
            RENDER PIPELINE          AUDIO SYSTEM
            (instanced birds)        (calls, wings)
```

### Relationship to Existing Systems

| System | Integration Point |
|--------|-------------------|
| AnimalManager | Birds stored alongside quadrupeds |
| Behavior HFSM | New `Flying` state variant added |
| Spawner | Aerial spawn points + perch spawning |
| Rendering | New bird shader with wing animation |
| Terrain | Perch point queries from tree system |

---

## Core Data Structures

### Location: `roanoke_game/src/animals/birds/`

```rust
//! Bird system module
//!
//! Submodules:
//!   - species.rs     - Bird species and stats
//!   - flight.rs      - Flight physics component
//!   - wing_ik.rs     - Wing bone animation
//!   - perch.rs       - Perching locations and logic
//!   - flock.rs       - Boids flocking behavior
//!   - behavior.rs    - Bird-specific AI states
```

### BirdSpecies Enum

```rust
// species.rs

use serde::{Deserialize, Serialize};

/// All bird species in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BirdSpecies {
    // === Songbirds (Small, ambient) ===
    NorthernCardinal,
    BlueJay,
    AmericanRobin,
    CarolinaWren,
    EasternBluebird,

    // === Corvids (Medium, intelligent) ===
    AmericanCrow,
    CommonRaven,

    // === Raptors (Predatory) ===
    RedTailedHawk,
    BarredOwl,
    GreatHornedOwl,
    BaldEagle,
    Osprey,

    // === Game Birds (Huntable) ===
    WildTurkey,
    BobwhiteQuail,
    MourningDove,

    // === Waterfowl (Aquatic) ===
    WoodDuck,
    CanadaGoose,
    Mallard,
    GreatBlueHeron,

    // === Shorebirds ===
    BlackSkimmer,
    SandPiper,

    // === Tiny Birds ===
    RubyThroatedHummingbird,
}

/// Category determines flight style and behavior patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BirdCategory {
    Songbird,       // Small, fast wingbeats, perches often
    Corvid,         // Medium, intelligent, soaring + flapping
    Raptor,         // Large, soaring hunters, dive attacks
    GameBird,       // Ground-dwelling, burst flight only
    Waterfowl,      // Water landing/takeoff, V-formation
    Shorebird,      // Low coastal flight, beach walking
    Hummingbird,    // Hover capability, rapid wingbeats
}

impl BirdSpecies {
    pub fn category(&self) -> BirdCategory {
        match self {
            Self::NorthernCardinal | Self::BlueJay | Self::AmericanRobin |
            Self::CarolinaWren | Self::EasternBluebird => BirdCategory::Songbird,

            Self::AmericanCrow | Self::CommonRaven => BirdCategory::Corvid,

            Self::RedTailedHawk | Self::BarredOwl | Self::GreatHornedOwl |
            Self::BaldEagle | Self::Osprey => BirdCategory::Raptor,

            Self::WildTurkey | Self::BobwhiteQuail |
            Self::MourningDove => BirdCategory::GameBird,

            Self::WoodDuck | Self::CanadaGoose | Self::Mallard |
            Self::GreatBlueHeron => BirdCategory::Waterfowl,

            Self::BlackSkimmer | Self::SandPiper => BirdCategory::Shorebird,

            Self::RubyThroatedHummingbird => BirdCategory::Hummingbird,
        }
    }

    pub fn can_hover(&self) -> bool {
        matches!(self.category(), BirdCategory::Hummingbird)
    }

    pub fn can_soar(&self) -> bool {
        matches!(self.category(),
            BirdCategory::Raptor | BirdCategory::Corvid | BirdCategory::Waterfowl)
    }

    pub fn is_nocturnal(&self) -> bool {
        matches!(self, Self::BarredOwl | Self::GreatHornedOwl)
    }

    pub fn is_predator(&self) -> bool {
        matches!(self.category(), BirdCategory::Raptor)
    }
}
```

### Bird Stats

```rust
// species.rs (continued)

#[derive(Debug, Clone)]
pub struct BirdStats {
    // Physical
    pub wingspan: f32,          // Meters, affects lift and turn radius
    pub body_mass: f32,         // Kg, affects momentum and stall speed
    pub body_length: f32,       // Meters, for collision/rendering

    // Flight Performance
    pub cruise_speed: f32,      // m/s normal flight
    pub max_speed: f32,         // m/s diving/fleeing
    pub stall_speed: f32,       // m/s minimum to maintain lift
    pub climb_rate: f32,        // m/s vertical gain
    pub turn_rate: f32,         // radians/s at cruise speed
    pub flap_frequency: f32,    // Hz wingbeats

    // Behavior
    pub preferred_altitude: f32,    // Meters above ground
    pub detection_range: f32,       // Threat awareness
    pub flee_trigger_range: f32,    // Distance to start fleeing
    pub territorial_radius: f32,    // For songbirds

    // Hunting (raptors only)
    pub dive_speed: Option<f32>,
    pub prey_species: Vec<DocileSpecies>,  // What they hunt
}

impl BirdSpecies {
    pub fn stats(&self) -> BirdStats {
        match self {
            Self::NorthernCardinal => BirdStats {
                wingspan: 0.28,
                body_mass: 0.045,
                body_length: 0.22,
                cruise_speed: 8.0,
                max_speed: 14.0,
                stall_speed: 4.0,
                climb_rate: 3.0,
                turn_rate: 4.0,
                flap_frequency: 12.0,
                preferred_altitude: 8.0,
                detection_range: 15.0,
                flee_trigger_range: 10.0,
                territorial_radius: 20.0,
                dive_speed: None,
                prey_species: vec![],
            },

            Self::RedTailedHawk => BirdStats {
                wingspan: 1.3,
                body_mass: 1.2,
                body_length: 0.56,
                cruise_speed: 12.0,
                max_speed: 45.0,  // Dive speed
                stall_speed: 6.0,
                climb_rate: 4.0,
                turn_rate: 1.5,
                flap_frequency: 3.0,
                preferred_altitude: 40.0,
                detection_range: 100.0,  // Excellent vision
                flee_trigger_range: 30.0,
                territorial_radius: 200.0,
                dive_speed: Some(45.0),
                prey_species: vec![
                    DocileSpecies::EasternCottontail,
                    DocileSpecies::GraySquirrel,
                ],
            },

            Self::WildTurkey => BirdStats {
                wingspan: 1.4,
                body_mass: 5.0,
                body_length: 0.9,
                cruise_speed: 10.0,
                max_speed: 20.0,
                stall_speed: 8.0,  // Heavy, needs speed
                climb_rate: 2.0,
                turn_rate: 1.0,
                flap_frequency: 4.0,
                preferred_altitude: 0.0,  // Ground bird
                detection_range: 25.0,
                flee_trigger_range: 15.0,
                territorial_radius: 0.0,
                dive_speed: None,
                prey_species: vec![],
            },

            Self::RubyThroatedHummingbird => BirdStats {
                wingspan: 0.11,
                body_mass: 0.003,
                body_length: 0.08,
                cruise_speed: 12.0,
                max_speed: 20.0,
                stall_speed: 0.0,  // Can hover!
                climb_rate: 8.0,
                turn_rate: 10.0,  // Extremely agile
                flap_frequency: 53.0,  // 53 beats/second
                preferred_altitude: 3.0,
                detection_range: 8.0,
                flee_trigger_range: 5.0,
                territorial_radius: 15.0,
                dive_speed: None,
                prey_species: vec![],
            },

            // ... additional species configurations
            _ => Self::default_stats(),
        }
    }

    fn default_stats() -> BirdStats {
        BirdStats {
            wingspan: 0.4,
            body_mass: 0.2,
            body_length: 0.25,
            cruise_speed: 10.0,
            max_speed: 18.0,
            stall_speed: 5.0,
            climb_rate: 3.0,
            turn_rate: 2.5,
            flap_frequency: 8.0,
            preferred_altitude: 12.0,
            detection_range: 20.0,
            flee_trigger_range: 12.0,
            territorial_radius: 0.0,
            dive_speed: None,
            prey_species: vec![],
        }
    }
}
```

---

## Flight Physics System

### Flight State Component

```rust
// flight.rs

/// Core flight state for a bird
#[derive(Debug, Clone)]
pub struct BirdFlight {
    // Kinematics
    pub velocity: Vec3,         // World-space velocity
    pub angular_velocity: Vec3, // Roll/pitch/yaw rates

    // Flight Mode
    pub mode: FlightMode,
    pub glide_factor: f32,      // 0.0 = flapping, 1.0 = gliding

    // Wing State
    pub wing_phase: f32,        // 0.0-1.0 flap cycle
    pub wing_fold: f32,         // 0.0 = extended, 1.0 = tucked (diving)

    // Energy
    pub stamina: f32,           // 0.0-1.0, depletes while climbing

    // Targets
    pub altitude_target: f32,
    pub heading_target: f32,    // Radians, world Y-up
    pub speed_target: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightMode {
    Perched,        // Not flying, on a surface
    TakingOff,      // Transition from perched to flying
    Flapping,       // Active wingbeats, gaining/maintaining altitude
    Gliding,        // Passive flight, slowly losing altitude
    Soaring,        // Using thermals/updrafts to gain altitude
    Diving,         // Rapid descent, wings tucked
    Hovering,       // Stationary in air (hummingbirds only)
    Landing,        // Approach to perch
    Swimming,       // On water surface (waterfowl)
    Walking,        // Ground locomotion (turkeys, etc.)
}
```

### Physics Update

```rust
// flight.rs (continued)

/// Flight physics constants
pub struct FlightConfig {
    pub gravity: f32,           // 9.81 m/s^2
    pub air_density: f32,       // 1.225 kg/m^3
    pub lift_coefficient: f32,  // ~1.0 for bird wings
    pub drag_coefficient: f32,  // ~0.1-0.3
    pub thermal_strength: f32,  // Updraft velocity
}

impl Default for FlightConfig {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            air_density: 1.225,
            lift_coefficient: 1.0,
            drag_coefficient: 0.15,
            thermal_strength: 2.0,
        }
    }
}

pub fn update_bird_flight(
    flight: &mut BirdFlight,
    stats: &BirdStats,
    config: &FlightConfig,
    terrain_height: f32,
    in_thermal: bool,
    dt: f32,
) {
    let speed = flight.velocity.length();
    let forward = flight.velocity.normalize_or_zero();

    match flight.mode {
        FlightMode::Perched | FlightMode::Walking => {
            // No flight physics
            return;
        }

        FlightMode::Flapping => {
            // Active flight - generate lift and thrust
            let wing_area = stats.wingspan * stats.wingspan * 0.15;

            // Lift = 0.5 * rho * v^2 * Cl * A
            let lift_force = 0.5 * config.air_density
                * speed * speed
                * config.lift_coefficient
                * wing_area
                * (1.0 - flight.wing_fold);

            // Thrust from flapping (simplified)
            let thrust = stats.body_mass * config.gravity * 1.2;  // Slight excess for climb

            // Apply forces
            let lift_vec = Vec3::Y * lift_force / stats.body_mass;
            let thrust_vec = forward * thrust / stats.body_mass;
            let gravity_vec = Vec3::NEG_Y * config.gravity;
            let drag_vec = -forward * config.drag_coefficient * speed * speed / stats.body_mass;

            flight.velocity += (lift_vec + thrust_vec + gravity_vec + drag_vec) * dt;

            // Drain stamina while climbing
            if flight.velocity.y > 0.0 {
                flight.stamina -= dt * 0.1;
            } else {
                flight.stamina = (flight.stamina + dt * 0.05).min(1.0);
            }

            // Advance wing flap cycle
            flight.wing_phase = (flight.wing_phase + dt * stats.flap_frequency).fract();
        }

        FlightMode::Gliding => {
            // Passive flight - no thrust, gradual descent
            let wing_area = stats.wingspan * stats.wingspan * 0.15;

            let lift_force = 0.5 * config.air_density
                * speed * speed
                * config.lift_coefficient * 0.8  // Less efficient than flapping
                * wing_area;

            let lift_vec = Vec3::Y * lift_force / stats.body_mass;
            let gravity_vec = Vec3::NEG_Y * config.gravity;
            let drag_vec = -forward * config.drag_coefficient * 0.5 * speed * speed / stats.body_mass;

            flight.velocity += (lift_vec + gravity_vec + drag_vec) * dt;

            // Recover stamina while gliding
            flight.stamina = (flight.stamina + dt * 0.15).min(1.0);

            // Wings held steady
            flight.wing_phase = 0.5;  // Mid-position
        }

        FlightMode::Soaring => {
            // Use thermal updrafts
            let thermal_lift = if in_thermal {
                Vec3::Y * config.thermal_strength
            } else {
                Vec3::ZERO
            };

            let gravity_vec = Vec3::NEG_Y * config.gravity;
            let drag_vec = -forward * config.drag_coefficient * 0.3 * speed * speed / stats.body_mass;

            flight.velocity += (thermal_lift + gravity_vec + drag_vec) * dt;

            // Circle in thermal
            flight.angular_velocity.y = 0.5;  // Gentle turn

            // Full stamina recovery
            flight.stamina = (flight.stamina + dt * 0.2).min(1.0);
        }

        FlightMode::Diving => {
            // Tucked wings, minimal drag, rapid descent
            let gravity_vec = Vec3::NEG_Y * config.gravity;
            let drag_vec = -forward * config.drag_coefficient * 0.05 * speed * speed / stats.body_mass;

            flight.velocity += (gravity_vec + drag_vec) * dt;

            // Clamp to max dive speed
            if let Some(max_dive) = stats.dive_speed {
                if speed > max_dive {
                    flight.velocity = flight.velocity.normalize() * max_dive;
                }
            }

            flight.wing_fold = 1.0;  // Wings tucked
        }

        FlightMode::Hovering => {
            // Hummingbird only - requires enormous energy
            flight.velocity = flight.velocity.lerp(Vec3::ZERO, dt * 5.0);
            flight.wing_phase = (flight.wing_phase + dt * 53.0).fract();  // 53 Hz!
            flight.stamina -= dt * 0.3;  // Drains fast
        }

        FlightMode::TakingOff => {
            // Burst of vertical thrust
            let thrust = Vec3::new(0.0, 8.0, 0.0) + forward * 5.0;
            flight.velocity += thrust * dt;
            flight.wing_phase = (flight.wing_phase + dt * stats.flap_frequency * 1.5).fract();

            // Transition to flapping once airborne
            let height_above_ground = flight.velocity.y;
            if height_above_ground > 2.0 {
                flight.mode = FlightMode::Flapping;
            }
        }

        FlightMode::Landing => {
            // Reduce speed, extend wings for maximum drag
            let target_vel = forward * stats.stall_speed * 0.8;
            flight.velocity = flight.velocity.lerp(target_vel, dt * 2.0);
            flight.wing_fold = 0.0;
            flight.wing_phase = 0.3;  // Wings forward for braking
        }

        FlightMode::Swimming => {
            // Waterfowl on water - bobbing motion
            flight.velocity = flight.velocity.lerp(Vec3::ZERO, dt * 2.0);
            // Water physics handled elsewhere
        }
    }

    // Stall check
    if speed < stats.stall_speed && !matches!(flight.mode,
        FlightMode::Hovering | FlightMode::Perched | FlightMode::Walking | FlightMode::Swimming)
    {
        // Stalling - nose drops
        flight.velocity.y -= config.gravity * dt * 2.0;
    }

    // Ground collision
    let current_height = flight.velocity.y;  // Approximate
    if terrain_height > current_height - 0.5 {
        flight.mode = FlightMode::Walking;
        flight.velocity.y = 0.0;
    }

    // Auto-switch between flapping and gliding based on stamina
    if flight.stamina < 0.2 && flight.mode == FlightMode::Flapping {
        flight.mode = FlightMode::Gliding;
    }
    if flight.stamina > 0.8 && flight.mode == FlightMode::Gliding
        && flight.velocity.y < -1.0  // Losing altitude
    {
        flight.mode = FlightMode::Flapping;
    }
}
```

### Banking and Turning

```rust
// flight.rs (continued)

/// Update bird orientation based on velocity and turn input
pub fn update_bird_orientation(
    rotation: &mut Quat,
    flight: &mut BirdFlight,
    turn_input: f32,  // -1.0 to 1.0
    stats: &BirdStats,
    dt: f32,
) {
    let speed = flight.velocity.length();

    // Turn rate scales with speed (slower = tighter turns, but less bank)
    let speed_factor = (speed / stats.cruise_speed).clamp(0.5, 1.5);
    let turn_rate = stats.turn_rate * turn_input * speed_factor;

    // Bank into turn
    let target_bank = turn_input * 0.6;  // ~35 degrees max bank
    let current_bank = rotation.to_euler(EulerRot::YXZ).2;
    let bank = lerp(current_bank, target_bank, dt * 3.0);

    // Pitch based on climb/descent
    let climb_rate = flight.velocity.y / speed.max(0.1);
    let target_pitch = climb_rate.clamp(-0.5, 0.5);  // ~30 degrees max

    // Yaw from turn rate
    let yaw_delta = turn_rate * dt;

    // Build new rotation
    let (current_yaw, _, _) = rotation.to_euler(EulerRot::YXZ);
    *rotation = Quat::from_euler(EulerRot::YXZ, current_yaw + yaw_delta, target_pitch, bank);

    // Rotate velocity to match heading
    let forward = *rotation * Vec3::NEG_Z;
    flight.velocity = forward * speed;
}
```

---

## Wing Animation System

### Wing Bone Structure

```
BIRD SKELETON HIERARCHY:

Root
└── Body
    ├── Neck
    │   └── Head
    │       └── Beak
    ├── Tail
    │   └── TailFeathers
    ├── Wing_L
    │   ├── Wing_L_Shoulder
    │   ├── Wing_L_Elbow
    │   ├── Wing_L_Wrist
    │   └── Wing_L_Primaries (feathers)
    ├── Wing_R
    │   └── (mirror of Wing_L)
    ├── Leg_L
    │   ├── Leg_L_Upper
    │   ├── Leg_L_Lower
    │   └── Leg_L_Foot
    └── Leg_R
        └── (mirror of Leg_L)
```

### Wing IK Component

```rust
// wing_ik.rs

/// Per-wing IK state
#[derive(Debug, Clone, Default)]
pub struct WingIK {
    pub shoulder_angle: f32,    // Forward/back rotation
    pub elbow_angle: f32,       // Wing bend
    pub wrist_angle: f32,       // Wingtip angle
    pub feather_spread: f32,    // 0.0 = closed, 1.0 = spread
}

/// Full bird IK state
#[derive(Debug, Clone)]
pub struct BirdIK {
    pub left_wing: WingIK,
    pub right_wing: WingIK,
    pub left_leg: TwoBoneIK,    // Reuse from quadruped system
    pub right_leg: TwoBoneIK,
    pub tail_angle: f32,        // Up/down for steering
    pub neck_curve: f32,        // Head position
}

/// Calculate wing pose from flight state
pub fn calculate_wing_pose(
    flight: &BirdFlight,
    stats: &BirdStats,
    dt: f32,
) -> (WingIK, WingIK) {
    let phase = flight.wing_phase;
    let fold = flight.wing_fold;

    match flight.mode {
        FlightMode::Flapping => {
            // Sinusoidal wing motion
            let downstroke = (phase * std::f32::consts::TAU).sin();

            // Shoulder drives main wing motion
            let shoulder = downstroke * 0.8 * (1.0 - fold);

            // Elbow bends more on upstroke
            let elbow = if phase < 0.5 {
                0.2  // Downstroke - extended
            } else {
                0.5  // Upstroke - bent
            } * (1.0 - fold);

            // Wrist follows with delay
            let wrist_phase = (phase - 0.1).fract();
            let wrist = (wrist_phase * std::f32::consts::TAU).sin() * 0.3;

            // Feathers spread on downstroke
            let spread = if phase < 0.5 { 1.0 } else { 0.6 };

            let wing = WingIK {
                shoulder_angle: shoulder,
                elbow_angle: elbow,
                wrist_angle: wrist,
                feather_spread: spread * (1.0 - fold),
            };

            (wing.clone(), wing)  // Symmetric for straight flight
        }

        FlightMode::Gliding | FlightMode::Soaring => {
            // Wings held steady, extended
            let wing = WingIK {
                shoulder_angle: 0.1,   // Slight dihedral
                elbow_angle: 0.15,     // Slight bend
                wrist_angle: 0.0,
                feather_spread: 0.9,   // Mostly spread
            };
            (wing.clone(), wing)
        }

        FlightMode::Diving => {
            // Wings tucked close to body
            let wing = WingIK {
                shoulder_angle: -0.3,  // Swept back
                elbow_angle: 1.2,      // Tightly bent
                wrist_angle: 0.5,      // Wingtips in
                feather_spread: 0.1,   // Closed
            };
            (wing.clone(), wing)
        }

        FlightMode::Hovering => {
            // Rapid figure-8 motion (hummingbird)
            let hover_phase = flight.wing_phase * 2.0;  // Double speed
            let figure_8 = (hover_phase * std::f32::consts::TAU).sin();

            let wing = WingIK {
                shoulder_angle: figure_8 * 0.6,
                elbow_angle: 0.3,
                wrist_angle: -figure_8 * 0.4,  // Counter-rotation
                feather_spread: 0.8,
            };
            (wing.clone(), wing)
        }

        FlightMode::Perched | FlightMode::Walking => {
            // Wings folded at sides
            let wing = WingIK {
                shoulder_angle: -0.5,  // Back
                elbow_angle: 1.5,      // Fully bent
                wrist_angle: 0.8,
                feather_spread: 0.0,   // Closed
            };
            (wing.clone(), wing)
        }

        FlightMode::TakingOff => {
            // Powerful downstrokes
            let downstroke = (phase * std::f32::consts::TAU).sin();
            let wing = WingIK {
                shoulder_angle: downstroke * 1.0,  // Exaggerated
                elbow_angle: 0.1,
                wrist_angle: downstroke * 0.2,
                feather_spread: 1.0,
            };
            (wing.clone(), wing)
        }

        FlightMode::Landing => {
            // Wings forward and spread for braking
            let wing = WingIK {
                shoulder_angle: 0.4,   // Forward
                elbow_angle: 0.0,      // Extended
                wrist_angle: -0.2,     // Cupped
                feather_spread: 1.0,   // Full spread
            };
            (wing.clone(), wing)
        }

        FlightMode::Swimming => {
            // Wings tucked, may paddle
            let wing = WingIK {
                shoulder_angle: -0.3,
                elbow_angle: 1.3,
                wrist_angle: 0.5,
                feather_spread: 0.2,
            };
            (wing.clone(), wing)
        }
    }
}

/// Apply asymmetric wing angles for banking
pub fn apply_bank_to_wings(
    left: &mut WingIK,
    right: &mut WingIK,
    bank_angle: f32,  // Positive = banking right
) {
    // Inside wing (lower) more tucked
    // Outside wing (higher) more extended
    let bank_factor = bank_angle.abs().min(0.5);

    if bank_angle > 0.0 {
        // Banking right - right wing lower
        right.shoulder_angle -= bank_factor * 0.3;
        right.elbow_angle += bank_factor * 0.2;
        left.shoulder_angle += bank_factor * 0.2;
    } else {
        // Banking left - left wing lower
        left.shoulder_angle -= bank_factor * 0.3;
        left.elbow_angle += bank_factor * 0.2;
        right.shoulder_angle += bank_factor * 0.2;
    }
}
```

---

## Behavior State Machine

### Bird-Specific HFSM States

```rust
// behavior.rs

/// Bird behavior states (extends base animal HFSM)
#[derive(Debug, Clone, PartialEq)]
pub enum BirdBehaviorState {
    // === Grounded States ===
    Perched(PerchedState),
    Foraging,           // Ground feeding (turkeys, sparrows)
    Walking,            // Ground locomotion
    Bathing,            // In water/dust

    // === Flight States ===
    Cruising,           // Normal flight, no specific goal
    Migrating(Vec3),    // Flying toward destination
    Fleeing(FleeState),
    Hunting(HuntState), // Raptors only

    // === Social States ===
    Flocking,           // Following flock leader
    Singing,            // Territorial/mating calls
    Nesting,            // At nest site

    // === Transitions ===
    TakingOff,
    Landing(PerchPoint),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PerchedState {
    Resting,
    Preening,
    LookingAround,
    Calling,
    Sleeping,       // Nocturnal birds during day
}

#[derive(Debug, Clone, PartialEq)]
pub enum FleeState {
    BurstFlight,    // Initial rapid escape
    Evasive,        // Erratic flight pattern
    GainingAltitude,
    SeekingCover,   // Flying to dense trees
}

#[derive(Debug, Clone, PartialEq)]
pub enum HuntState {
    Searching,      // Circling, looking for prey
    Tracking,       // Following specific target
    Stooping,       // Diving attack
    Striking,       // Impact moment
    Carrying,       // Flying with prey
}
```

### Behavior Update Logic

```rust
// behavior.rs (continued)

pub struct BirdBehaviorContext<'a> {
    pub player_pos: Vec3,
    pub player_velocity: Vec3,
    pub terrain: &'a Terrain,
    pub nearby_birds: &'a [BirdId],
    pub nearby_prey: &'a [(DocileSpeciesId, Vec3)],
    pub available_perches: &'a [PerchPoint],
    pub time_of_day: f32,  // 0.0-24.0 hours
    pub dt: f32,
}

pub fn update_bird_behavior(
    bird: &mut Bird,
    ctx: &BirdBehaviorContext,
) -> Option<BirdBehaviorState> {
    let stats = bird.species.stats();
    let dist_to_player = (bird.position - ctx.player_pos).length();

    // Priority 1: Flee from threats
    if dist_to_player < stats.flee_trigger_range {
        if !matches!(bird.behavior, BirdBehaviorState::Fleeing(_)) {
            bird.flight.mode = FlightMode::Flapping;
            return Some(BirdBehaviorState::Fleeing(FleeState::BurstFlight));
        }
    }

    // Priority 2: Hunt (raptors)
    if bird.species.is_predator() {
        if let Some((prey_id, prey_pos)) = find_viable_prey(bird, ctx.nearby_prey, &stats) {
            return Some(BirdBehaviorState::Hunting(HuntState::Tracking));
        }
    }

    // Priority 3: Maintain flock cohesion
    if should_follow_flock(bird, ctx.nearby_birds) {
        return Some(BirdBehaviorState::Flocking);
    }

    // Normal behavior based on current state
    match &bird.behavior {
        BirdBehaviorState::Perched(perched) => {
            update_perched_behavior(bird, perched, ctx)
        }

        BirdBehaviorState::Cruising => {
            // Random wandering flight
            if rand::random::<f32>() < 0.001 {
                // Occasionally land
                if let Some(perch) = find_nearest_perch(bird.position, ctx.available_perches) {
                    return Some(BirdBehaviorState::Landing(perch));
                }
            }
            None
        }

        BirdBehaviorState::Fleeing(flee_state) => {
            update_flee_behavior(bird, flee_state, ctx)
        }

        BirdBehaviorState::Hunting(hunt_state) => {
            update_hunt_behavior(bird, hunt_state, ctx)
        }

        BirdBehaviorState::Flocking => {
            // Handled by flock manager
            None
        }

        _ => None,
    }
}

fn update_perched_behavior(
    bird: &mut Bird,
    state: &PerchedState,
    ctx: &BirdBehaviorContext,
) -> Option<BirdBehaviorState> {
    // Time-based transitions
    let should_sleep = bird.species.is_nocturnal() != is_night(ctx.time_of_day);

    if should_sleep {
        return Some(BirdBehaviorState::Perched(PerchedState::Sleeping));
    }

    // Random state transitions while perched
    match state {
        PerchedState::Resting => {
            if rand::random::<f32>() < 0.01 {
                return Some(BirdBehaviorState::Perched(PerchedState::LookingAround));
            }
            if rand::random::<f32>() < 0.005 {
                return Some(BirdBehaviorState::Perched(PerchedState::Preening));
            }
            if rand::random::<f32>() < 0.002 {
                // Take off
                bird.flight.mode = FlightMode::TakingOff;
                return Some(BirdBehaviorState::TakingOff);
            }
        }

        PerchedState::LookingAround => {
            // Head movement handled in animation
            if rand::random::<f32>() < 0.02 {
                return Some(BirdBehaviorState::Perched(PerchedState::Resting));
            }
        }

        PerchedState::Preening => {
            if rand::random::<f32>() < 0.03 {
                return Some(BirdBehaviorState::Perched(PerchedState::Resting));
            }
        }

        PerchedState::Calling => {
            // Trigger audio
            // Short duration then return to resting
            if rand::random::<f32>() < 0.05 {
                return Some(BirdBehaviorState::Perched(PerchedState::Resting));
            }
        }

        PerchedState::Sleeping => {
            // Wake up at appropriate time
            if !should_sleep {
                return Some(BirdBehaviorState::Perched(PerchedState::Resting));
            }
        }
    }

    None
}

fn update_flee_behavior(
    bird: &mut Bird,
    state: &FleeState,
    ctx: &BirdBehaviorContext,
) -> Option<BirdBehaviorState> {
    let dist_to_player = (bird.position - ctx.player_pos).length();
    let stats = bird.species.stats();

    // Safe distance reached
    if dist_to_player > stats.detection_range * 2.0 {
        bird.flight.mode = FlightMode::Gliding;
        return Some(BirdBehaviorState::Cruising);
    }

    match state {
        FleeState::BurstFlight => {
            // Initial panic - fly directly away
            let flee_dir = (bird.position - ctx.player_pos).normalize();
            bird.flight.velocity = flee_dir * stats.max_speed * 0.9;
            bird.flight.mode = FlightMode::Flapping;

            // Transition to evasive after initial burst
            if rand::random::<f32>() < 0.05 {
                return Some(BirdBehaviorState::Fleeing(FleeState::Evasive));
            }
        }

        FleeState::Evasive => {
            // Erratic direction changes
            if rand::random::<f32>() < 0.1 {
                let random_turn = (rand::random::<f32>() - 0.5) * 2.0;
                bird.flight.heading_target += random_turn;
            }

            // Try to gain altitude
            if bird.position.y < stats.preferred_altitude * 2.0 {
                return Some(BirdBehaviorState::Fleeing(FleeState::GainingAltitude));
            }
        }

        FleeState::GainingAltitude => {
            bird.flight.altitude_target = stats.preferred_altitude * 3.0;
            bird.flight.mode = FlightMode::Flapping;

            if bird.position.y > stats.preferred_altitude * 2.5 {
                return Some(BirdBehaviorState::Fleeing(FleeState::Evasive));
            }
        }

        FleeState::SeekingCover => {
            // Find dense tree canopy
            // Implementation depends on terrain/tree system
        }
    }

    None
}
```

---

## Perching System

### Perch Point Definition

```rust
// perch.rs

/// A location where a bird can land
#[derive(Debug, Clone)]
pub struct PerchPoint {
    pub position: Vec3,
    pub normal: Vec3,           // Surface direction (up for branches)
    pub forward: Vec3,          // Direction bird faces when perched
    pub perch_type: PerchType,
    pub radius: f32,            // Size of perching surface
    pub height_above_ground: f32,
    pub occupied_by: Option<BirdId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerchType {
    TreeBranch,
    TreeTop,
    Rock,
    Fence,
    Rooftop,
    Ground,
    WaterSurface,   // For waterfowl
    CliffLedge,
    PowerLine,      // If applicable to setting
}

impl PerchType {
    /// Which bird categories can use this perch type
    pub fn valid_for(&self, category: BirdCategory) -> bool {
        match self {
            Self::TreeBranch => matches!(category,
                BirdCategory::Songbird | BirdCategory::Corvid | BirdCategory::Raptor),
            Self::TreeTop => matches!(category,
                BirdCategory::Raptor | BirdCategory::Corvid),
            Self::Rock => true,  // All birds
            Self::Fence => matches!(category,
                BirdCategory::Songbird | BirdCategory::Corvid | BirdCategory::GameBird),
            Self::Rooftop => matches!(category,
                BirdCategory::Corvid | BirdCategory::Raptor),
            Self::Ground => matches!(category,
                BirdCategory::GameBird | BirdCategory::Shorebird | BirdCategory::Waterfowl),
            Self::WaterSurface => matches!(category,
                BirdCategory::Waterfowl | BirdCategory::Shorebird),
            Self::CliffLedge => matches!(category,
                BirdCategory::Raptor),
            Self::PowerLine => matches!(category,
                BirdCategory::Songbird | BirdCategory::Corvid),
        }
    }

    /// Preferred by this category (for spawn selection)
    pub fn preference_for(&self, category: BirdCategory) -> f32 {
        match (self, category) {
            (Self::TreeBranch, BirdCategory::Songbird) => 1.0,
            (Self::TreeTop, BirdCategory::Raptor) => 1.0,
            (Self::WaterSurface, BirdCategory::Waterfowl) => 1.0,
            (Self::Ground, BirdCategory::GameBird) => 1.0,
            _ => 0.5,
        }
    }
}
```

### Perch Point Generation

```rust
// perch.rs (continued)

/// Generate perch points from world features
pub fn generate_perch_points_for_chunk(
    chunk: &Chunk,
    trees: &[TreeInstance],
    rocks: &[RockInstance],
    water_bodies: &[WaterBody],
) -> Vec<PerchPoint> {
    let mut perches = Vec::new();

    // Tree branches and tops
    for tree in trees {
        // Tree top perch (for raptors)
        perches.push(PerchPoint {
            position: tree.position + Vec3::Y * tree.height,
            normal: Vec3::Y,
            forward: random_horizontal_dir(),
            perch_type: PerchType::TreeTop,
            radius: 0.3,
            height_above_ground: tree.height,
            occupied_by: None,
        });

        // Branch perches (multiple per tree)
        let branch_count = (tree.height / 2.0) as usize;
        for i in 0..branch_count {
            let height = tree.height * 0.4 + (i as f32 / branch_count as f32) * tree.height * 0.5;
            let angle = i as f32 * 2.4;  // Golden angle for distribution
            let radius = tree.canopy_radius * 0.7;

            let offset = Vec3::new(
                angle.cos() * radius,
                height,
                angle.sin() * radius,
            );

            perches.push(PerchPoint {
                position: tree.position + offset,
                normal: Vec3::Y,
                forward: -offset.normalize(),  // Face outward
                perch_type: PerchType::TreeBranch,
                radius: 0.1,
                height_above_ground: height,
                occupied_by: None,
            });
        }
    }

    // Rock perches
    for rock in rocks {
        if rock.height > 0.5 {  // Only on larger rocks
            perches.push(PerchPoint {
                position: rock.position + Vec3::Y * rock.height,
                normal: Vec3::Y,
                forward: random_horizontal_dir(),
                perch_type: PerchType::Rock,
                radius: rock.radius * 0.5,
                height_above_ground: rock.height,
                occupied_by: None,
            });
        }
    }

    // Water surface perches (for waterfowl)
    for water in water_bodies {
        // Generate several floating perch points on water
        let count = (water.surface_area / 100.0) as usize;
        for _ in 0..count.min(10) {
            let pos = water.random_surface_point();
            perches.push(PerchPoint {
                position: pos,
                normal: Vec3::Y,
                forward: random_horizontal_dir(),
                perch_type: PerchType::WaterSurface,
                radius: 1.0,
                height_above_ground: 0.0,
                occupied_by: None,
            });
        }
    }

    // Ground perches (clearings, paths)
    for _ in 0..5 {
        let pos = chunk.random_ground_point();
        if chunk.is_clear_area(pos, 2.0) {
            perches.push(PerchPoint {
                position: pos,
                normal: Vec3::Y,
                forward: random_horizontal_dir(),
                perch_type: PerchType::Ground,
                radius: 2.0,
                height_above_ground: 0.0,
                occupied_by: None,
            });
        }
    }

    perches
}
```

### Landing Approach

```rust
// perch.rs (continued)

/// Calculate landing approach path
pub fn calculate_landing_approach(
    bird_pos: Vec3,
    bird_vel: Vec3,
    perch: &PerchPoint,
    stats: &BirdStats,
) -> LandingPath {
    let to_perch = perch.position - bird_pos;
    let distance = to_perch.length();
    let direction = to_perch.normalize();

    // Approach from above and behind the perch facing direction
    let approach_height = perch.position.y + 3.0;
    let approach_offset = -perch.forward * 5.0 + Vec3::Y * 3.0;
    let approach_point = perch.position + approach_offset;

    // Glide slope (typically 1:4 to 1:6 for birds)
    let glide_ratio = 5.0;

    LandingPath {
        waypoints: vec![
            approach_point,
            perch.position + Vec3::Y * 1.0,  // Flare point
            perch.position,
        ],
        target_speed: stats.stall_speed * 1.2,
        flare_distance: 2.0,
    }
}

pub struct LandingPath {
    pub waypoints: Vec<Vec3>,
    pub target_speed: f32,
    pub flare_distance: f32,
}

/// Update bird position during landing
pub fn update_landing(
    bird: &mut Bird,
    perch: &PerchPoint,
    landing_path: &LandingPath,
    dt: f32,
) -> bool {  // Returns true when landed
    let to_perch = perch.position - bird.position;
    let distance = to_perch.length();

    if distance < 0.2 {
        // Landed!
        bird.position = perch.position;
        bird.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, perch.forward);
        bird.flight.mode = FlightMode::Perched;
        bird.flight.velocity = Vec3::ZERO;
        return true;
    }

    // Follow path
    let target = if distance > landing_path.flare_distance {
        landing_path.waypoints[0]
    } else {
        perch.position
    };

    let to_target = (target - bird.position).normalize();
    bird.flight.velocity = to_target * landing_path.target_speed;

    // Reduce speed as we approach
    let speed_factor = (distance / 5.0).clamp(0.3, 1.0);
    bird.flight.velocity *= speed_factor;

    // Update position
    bird.position += bird.flight.velocity * dt;

    // Orient toward target
    if bird.flight.velocity.length() > 0.1 {
        let forward = bird.flight.velocity.normalize();
        bird.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, forward);
    }

    false
}
```

---

## Flock Behavior (Boids)

### Flock Data Structure

```rust
// flock.rs

/// A group of birds flying together
#[derive(Debug)]
pub struct Flock {
    pub id: FlockId,
    pub species: BirdSpecies,
    pub members: Vec<BirdId>,
    pub leader: Option<BirdId>,
    pub center: Vec3,
    pub velocity: Vec3,
    pub formation: FlockFormation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlockFormation {
    Cluster,        // Tight ball (sparrows, starlings)
    Vee,            // V-formation (geese, ducks)
    Line,           // Single file
    Scattered,      // Loose grouping (crows)
    Murmuration,    // Dynamic swirling (starlings)
}

impl BirdSpecies {
    pub fn flock_formation(&self) -> FlockFormation {
        match self.category() {
            BirdCategory::Waterfowl => FlockFormation::Vee,
            BirdCategory::Songbird => FlockFormation::Cluster,
            BirdCategory::Corvid => FlockFormation::Scattered,
            _ => FlockFormation::Cluster,
        }
    }

    pub fn flock_size_range(&self) -> (usize, usize) {
        match self {
            Self::CanadaGoose => (5, 30),
            Self::AmericanCrow => (10, 50),
            Self::NorthernCardinal => (2, 8),
            Self::MourningDove => (3, 12),
            _ => (3, 15),
        }
    }
}
```

### Boids Algorithm

```rust
// flock.rs (continued)

/// Boids steering parameters
pub struct BoidsConfig {
    pub separation_radius: f32,     // Avoid getting too close
    pub alignment_radius: f32,      // Match neighbors' velocity
    pub cohesion_radius: f32,       // Stay near flock center
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub max_steering_force: f32,
    pub formation_weight: f32,      // Follow formation positions
}

impl Default for BoidsConfig {
    fn default() -> Self {
        Self {
            separation_radius: 2.0,
            alignment_radius: 8.0,
            cohesion_radius: 15.0,
            separation_weight: 1.5,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            max_steering_force: 5.0,
            formation_weight: 0.5,
        }
    }
}

/// Calculate boids steering for a single bird
pub fn calculate_boids_steering(
    bird: &Bird,
    flock: &Flock,
    all_birds: &HashMap<BirdId, Bird>,
    config: &BoidsConfig,
) -> Vec3 {
    let mut separation = Vec3::ZERO;
    let mut alignment = Vec3::ZERO;
    let mut cohesion = Vec3::ZERO;

    let mut sep_count = 0;
    let mut align_count = 0;
    let mut cohesion_count = 0;

    for &member_id in &flock.members {
        if member_id == bird.id {
            continue;
        }

        let Some(other) = all_birds.get(&member_id) else { continue };
        let to_other = other.position - bird.position;
        let dist = to_other.length();

        // Separation - steer away from close neighbors
        if dist < config.separation_radius && dist > 0.01 {
            separation -= to_other.normalize() / dist;
            sep_count += 1;
        }

        // Alignment - match velocity of nearby birds
        if dist < config.alignment_radius {
            alignment += other.flight.velocity;
            align_count += 1;
        }

        // Cohesion - steer toward center of nearby birds
        if dist < config.cohesion_radius {
            cohesion += other.position;
            cohesion_count += 1;
        }
    }

    // Average and weight each component
    let mut steering = Vec3::ZERO;

    if sep_count > 0 {
        separation /= sep_count as f32;
        steering += separation.normalize_or_zero() * config.separation_weight;
    }

    if align_count > 0 {
        alignment /= align_count as f32;
        let align_steer = alignment - bird.flight.velocity;
        steering += align_steer.normalize_or_zero() * config.alignment_weight;
    }

    if cohesion_count > 0 {
        cohesion /= cohesion_count as f32;
        let cohesion_steer = cohesion - bird.position;
        steering += cohesion_steer.normalize_or_zero() * config.cohesion_weight;
    }

    // Formation-specific steering
    steering += calculate_formation_steering(bird, flock, all_birds, config);

    // Clamp to max force
    if steering.length() > config.max_steering_force {
        steering = steering.normalize() * config.max_steering_force;
    }

    steering
}

/// Calculate position in formation
fn calculate_formation_steering(
    bird: &Bird,
    flock: &Flock,
    all_birds: &HashMap<BirdId, Bird>,
    config: &BoidsConfig,
) -> Vec3 {
    let Some(leader_id) = flock.leader else {
        return Vec3::ZERO;
    };

    let Some(leader) = all_birds.get(&leader_id) else {
        return Vec3::ZERO;
    };

    // Find bird's index in flock
    let index = flock.members.iter().position(|&id| id == bird.id).unwrap_or(0);

    if index == 0 {
        return Vec3::ZERO;  // Leader doesn't follow formation
    }

    let target_pos = match flock.formation {
        FlockFormation::Vee => {
            // V-formation: alternating left/right behind leader
            let side = if index % 2 == 0 { 1.0 } else { -1.0 };
            let row = (index + 1) / 2;
            let offset = Vec3::new(
                side * row as f32 * 3.0,  // Spread sideways
                -row as f32 * 0.5,        // Slightly lower each row
                -row as f32 * 4.0,        // Behind leader
            );

            let leader_forward = leader.flight.velocity.normalize_or_zero();
            let leader_right = leader_forward.cross(Vec3::Y).normalize();

            leader.position
                + leader_forward * offset.z
                + leader_right * offset.x
                + Vec3::Y * offset.y
        }

        FlockFormation::Cluster => {
            // Tight cluster around center
            flock.center
        }

        FlockFormation::Scattered => {
            // Loose grouping, just general direction
            flock.center + random_offset(5.0)
        }

        FlockFormation::Line => {
            // Single file behind leader
            let behind = -leader.flight.velocity.normalize_or_zero() * (index as f32 * 3.0);
            leader.position + behind
        }

        FlockFormation::Murmuration => {
            // Dynamic - handled by pure boids, no fixed positions
            return Vec3::ZERO;
        }
    };

    let to_target = target_pos - bird.position;
    to_target.normalize_or_zero() * config.formation_weight
}
```

### Flock Manager

```rust
// flock.rs (continued)

pub struct FlockManager {
    pub flocks: HashMap<FlockId, Flock>,
    next_id: u32,
}

impl FlockManager {
    pub fn new() -> Self {
        Self {
            flocks: HashMap::new(),
            next_id: 0,
        }
    }

    /// Create a new flock from spawned birds
    pub fn create_flock(&mut self, species: BirdSpecies, members: Vec<BirdId>) -> FlockId {
        let id = FlockId(self.next_id);
        self.next_id += 1;

        let leader = members.first().copied();

        self.flocks.insert(id, Flock {
            id,
            species,
            members,
            leader,
            center: Vec3::ZERO,
            velocity: Vec3::ZERO,
            formation: species.flock_formation(),
        });

        id
    }

    /// Update all flocks
    pub fn update(&mut self, birds: &HashMap<BirdId, Bird>) {
        for flock in self.flocks.values_mut() {
            // Remove dead/despawned birds
            flock.members.retain(|id| birds.contains_key(id));

            if flock.members.is_empty() {
                continue;
            }

            // Update center and velocity
            let mut center = Vec3::ZERO;
            let mut velocity = Vec3::ZERO;

            for &id in &flock.members {
                if let Some(bird) = birds.get(&id) {
                    center += bird.position;
                    velocity += bird.flight.velocity;
                }
            }

            let count = flock.members.len() as f32;
            flock.center = center / count;
            flock.velocity = velocity / count;

            // Elect new leader if needed
            if flock.leader.map_or(true, |id| !flock.members.contains(&id)) {
                flock.leader = flock.members.first().copied();
            }
        }

        // Remove empty flocks
        self.flocks.retain(|_, flock| !flock.members.is_empty());
    }
}
```

---

## Spawning Integration

### Bird Spawning Rules

```rust
// In spawner.rs additions

/// Spawn configuration for birds
pub struct BirdSpawnConfig {
    pub species: BirdSpecies,
    pub spawn_mode: BirdSpawnMode,
    pub habitats: Vec<Habitat>,
    pub time_of_day: TimeRange,      // When active
    pub altitude_range: (f32, f32),   // Flight spawn height
    pub density: f32,                 // Birds per chunk
}

#[derive(Debug, Clone, Copy)]
pub enum BirdSpawnMode {
    Perched,            // Spawn on a perch point
    Flying,             // Spawn in flight
    Ground,             // Spawn walking (turkeys)
    Water,              // Spawn floating (ducks)
    Flock(usize),       // Spawn as flock of N birds
}

impl BirdSpecies {
    pub fn spawn_config(&self) -> BirdSpawnConfig {
        let stats = self.stats();

        match self {
            Self::NorthernCardinal => BirdSpawnConfig {
                species: *self,
                spawn_mode: BirdSpawnMode::Perched,
                habitats: vec![Habitat::Forests, Habitat::Meadows],
                time_of_day: TimeRange::new(6.0, 18.0),  // Diurnal
                altitude_range: (3.0, 15.0),
                density: 0.5,
            },

            Self::RedTailedHawk => BirdSpawnConfig {
                species: *self,
                spawn_mode: BirdSpawnMode::Flying,
                habitats: vec![Habitat::Plains, Habitat::Forests, Habitat::Mountains],
                time_of_day: TimeRange::new(8.0, 17.0),
                altitude_range: (30.0, 80.0),
                density: 0.05,  // Rare
            },

            Self::BarredOwl => BirdSpawnConfig {
                species: *self,
                spawn_mode: BirdSpawnMode::Perched,
                habitats: vec![Habitat::Forests, Habitat::Swamps],
                time_of_day: TimeRange::new(19.0, 5.0),  // Nocturnal
                altitude_range: (8.0, 20.0),
                density: 0.1,
            },

            Self::WildTurkey => BirdSpawnConfig {
                species: *self,
                spawn_mode: BirdSpawnMode::Ground,
                habitats: vec![Habitat::Forests, Habitat::Meadows],
                time_of_day: TimeRange::new(6.0, 18.0),
                altitude_range: (0.0, 0.0),  // Ground only
                density: 0.3,
            },

            Self::CanadaGoose => BirdSpawnConfig {
                species: *self,
                spawn_mode: BirdSpawnMode::Flock(12),
                habitats: vec![Habitat::NearWater, Habitat::Marshes],
                time_of_day: TimeRange::all_day(),
                altitude_range: (0.0, 50.0),
                density: 0.2,
            },

            Self::AmericanCrow => BirdSpawnConfig {
                species: *self,
                spawn_mode: BirdSpawnMode::Flock(8),
                habitats: vec![Habitat::Forests, Habitat::Plains, Habitat::Fields],
                time_of_day: TimeRange::new(6.0, 19.0),
                altitude_range: (10.0, 40.0),
                density: 0.4,
            },

            _ => BirdSpawnConfig::default_for(*self),
        }
    }
}

/// Spawn birds for a newly loaded chunk
pub fn spawn_birds_for_chunk(
    chunk: &Chunk,
    perches: &[PerchPoint],
    time_of_day: f32,
    rng: &mut impl Rng,
) -> Vec<BirdSpawn> {
    let mut spawns = Vec::new();
    let habitat = chunk.primary_habitat();

    // Determine eligible species for this chunk
    let eligible: Vec<BirdSpecies> = BirdSpecies::all()
        .filter(|s| {
            let config = s.spawn_config();
            config.habitats.contains(&habitat)
                && config.time_of_day.contains(time_of_day)
        })
        .collect();

    for species in eligible {
        let config = species.spawn_config();
        let count = (config.density * rng.gen::<f32>() * 2.0) as usize;

        for _ in 0..count {
            match config.spawn_mode {
                BirdSpawnMode::Perched => {
                    // Find suitable perch
                    let category = species.category();
                    let valid_perches: Vec<_> = perches.iter()
                        .filter(|p| p.perch_type.valid_for(category) && p.occupied_by.is_none())
                        .collect();

                    if let Some(perch) = valid_perches.choose(rng) {
                        spawns.push(BirdSpawn {
                            species,
                            position: perch.position,
                            flight_mode: FlightMode::Perched,
                            flock_id: None,
                        });
                    }
                }

                BirdSpawnMode::Flying => {
                    let altitude = rng.gen_range(config.altitude_range.0..config.altitude_range.1);
                    let pos = chunk.random_point() + Vec3::Y * altitude;

                    spawns.push(BirdSpawn {
                        species,
                        position: pos,
                        flight_mode: FlightMode::Gliding,
                        flock_id: None,
                    });
                }

                BirdSpawnMode::Ground => {
                    let pos = chunk.random_ground_point();
                    spawns.push(BirdSpawn {
                        species,
                        position: pos,
                        flight_mode: FlightMode::Walking,
                        flock_id: None,
                    });
                }

                BirdSpawnMode::Water => {
                    if let Some(water_pos) = chunk.random_water_point() {
                        spawns.push(BirdSpawn {
                            species,
                            position: water_pos,
                            flight_mode: FlightMode::Swimming,
                            flock_id: None,
                        });
                    }
                }

                BirdSpawnMode::Flock(size) => {
                    let flock_size = rng.gen_range(size/2..size*2);
                    let center = chunk.random_point();
                    let altitude = rng.gen_range(config.altitude_range.0..config.altitude_range.1);

                    // Create flock members
                    for i in 0..flock_size {
                        let offset = Vec3::new(
                            rng.gen_range(-5.0..5.0),
                            rng.gen_range(-2.0..2.0),
                            rng.gen_range(-5.0..5.0),
                        );

                        spawns.push(BirdSpawn {
                            species,
                            position: center + Vec3::Y * altitude + offset,
                            flight_mode: FlightMode::Flapping,
                            flock_id: Some(FlockId(chunk.id as u32)),  // Temporary
                        });
                    }
                }
            }
        }
    }

    spawns
}

pub struct BirdSpawn {
    pub species: BirdSpecies,
    pub position: Vec3,
    pub flight_mode: FlightMode,
    pub flock_id: Option<FlockId>,
}
```

---

## Rendering Pipeline

### Bird Instance Data

```rust
// In animal_model_pipeline.rs additions

/// Per-bird instance data for GPU
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BirdInstance {
    // Transform
    pub model_matrix: [[f32; 4]; 4],

    // Wing animation
    pub left_wing_angles: [f32; 4],   // shoulder, elbow, wrist, spread
    pub right_wing_angles: [f32; 4],

    // Body pose
    pub tail_angle: f32,
    pub neck_angle: f32,

    // Material
    pub color_tint: [f32; 3],
    pub wetness: f32,
}

impl BirdInstance {
    pub fn from_bird(bird: &Bird, wing_ik: &(WingIK, WingIK)) -> Self {
        let (left, right) = wing_ik;

        Self {
            model_matrix: bird.transform_matrix().to_cols_array_2d(),
            left_wing_angles: [
                left.shoulder_angle,
                left.elbow_angle,
                left.wrist_angle,
                left.feather_spread,
            ],
            right_wing_angles: [
                right.shoulder_angle,
                right.elbow_angle,
                right.wrist_angle,
                right.feather_spread,
            ],
            tail_angle: bird.ik.tail_angle,
            neck_angle: bird.ik.neck_curve,
            color_tint: bird.species.color_tint(),
            wetness: if bird.flight.mode == FlightMode::Swimming { 0.8 } else { 0.0 },
        }
    }
}
```

### Bird Shader

```wgsl
// assets/shaders/bird.wgsl

struct BirdInstance {
    model_matrix: mat4x4<f32>,
    left_wing_angles: vec4<f32>,   // shoulder, elbow, wrist, spread
    right_wing_angles: vec4<f32>,
    tail_neck: vec2<f32>,          // tail_angle, neck_angle
    color_wetness: vec4<f32>,      // rgb tint, wetness
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) bone_index: u32,     // Which bone this vertex belongs to
    @location(4) bone_weight: f32,    // Blend weight
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) wetness: f32,
}

// Bone indices
const BONE_BODY: u32 = 0u;
const BONE_WING_L_SHOULDER: u32 = 1u;
const BONE_WING_L_ELBOW: u32 = 2u;
const BONE_WING_L_WRIST: u32 = 3u;
const BONE_WING_R_SHOULDER: u32 = 4u;
const BONE_WING_R_ELBOW: u32 = 5u;
const BONE_WING_R_WRIST: u32 = 6u;
const BONE_TAIL: u32 = 7u;
const BONE_NECK: u32 = 8u;
const BONE_HEAD: u32 = 9u;

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<storage, read> instances: array<BirdInstance>;

fn rotation_x(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, c, -s),
        vec3(0.0, s, c)
    );
}

fn rotation_z(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3(c, -s, 0.0),
        vec3(s, c, 0.0),
        vec3(0.0, 0.0, 1.0)
    );
}

@vertex
fn vs_main(
    in: VertexInput,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    let inst = instances[instance_idx];
    var pos = in.position;
    var normal = in.normal;

    // Apply bone transformations based on bone_index
    switch in.bone_index {
        case BONE_WING_L_SHOULDER: {
            let rot = rotation_z(inst.left_wing_angles.x);  // shoulder
            pos = rot * pos;
            normal = rot * normal;
        }
        case BONE_WING_L_ELBOW: {
            let rot = rotation_z(inst.left_wing_angles.x) * rotation_x(inst.left_wing_angles.y);
            pos = rot * pos;
            normal = rot * normal;
        }
        case BONE_WING_L_WRIST: {
            let rot = rotation_z(inst.left_wing_angles.x)
                    * rotation_x(inst.left_wing_angles.y)
                    * rotation_z(inst.left_wing_angles.z);
            pos = rot * pos;
            normal = rot * normal;
        }
        case BONE_WING_R_SHOULDER: {
            let rot = rotation_z(-inst.right_wing_angles.x);  // Mirrored
            pos = rot * pos;
            normal = rot * normal;
        }
        // ... similar for right wing bones
        case BONE_TAIL: {
            let rot = rotation_x(inst.tail_neck.x);
            pos = rot * pos;
            normal = rot * normal;
        }
        case BONE_NECK, BONE_HEAD: {
            let rot = rotation_x(inst.tail_neck.y);
            pos = rot * pos;
            normal = rot * normal;
        }
        default: {}
    }

    // Transform to world space
    let world_pos = (inst.model_matrix * vec4(pos, 1.0)).xyz;
    let world_normal = normalize((inst.model_matrix * vec4(normal, 0.0)).xyz);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4(world_pos, 1.0);
    out.world_pos = world_pos;
    out.world_normal = world_normal;
    out.uv = in.uv;
    out.wetness = inst.color_wetness.w;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let albedo = textureSample(diffuse_texture, texture_sampler, in.uv);

    // Basic lighting
    let light_dir = normalize(vec3(0.5, 1.0, 0.3));
    let ndotl = max(dot(in.world_normal, light_dir), 0.0);
    let ambient = 0.3;
    let diffuse = ndotl * 0.7;

    var color = albedo.rgb * (ambient + diffuse);

    // Wetness darkens and adds specular
    if in.wetness > 0.0 {
        color *= mix(1.0, 0.7, in.wetness);
        // Add specular sheen for wet feathers
        let view_dir = normalize(camera.position - in.world_pos);
        let half_vec = normalize(light_dir + view_dir);
        let spec = pow(max(dot(in.world_normal, half_vec), 0.0), 32.0);
        color += vec3(spec * in.wetness * 0.5);
    }

    // Fog
    let dist = length(camera.position - in.world_pos);
    let fog_factor = 1.0 - exp(-dist * 0.005);
    color = mix(color, fog_color.rgb, fog_factor);

    return vec4(color, albedo.a);
}
```

### LOD System

```rust
// In render integration

/// Bird LOD levels
pub enum BirdLOD {
    Full,       // Full model with wing animation
    Simplified, // Reduced poly, baked wing pose
    Billboard,  // 2D sprite
    Point,      // Single colored point
}

pub fn get_bird_lod(distance: f32, bird: &Bird) -> BirdLOD {
    let base_distance = match bird.species.category() {
        BirdCategory::Raptor => 150.0,   // Large, visible far
        BirdCategory::Waterfowl => 100.0,
        BirdCategory::GameBird => 80.0,
        BirdCategory::Corvid => 60.0,
        BirdCategory::Songbird => 40.0,
        BirdCategory::Hummingbird => 20.0,  // Tiny
        BirdCategory::Shorebird => 50.0,
    };

    match distance {
        d if d < base_distance => BirdLOD::Full,
        d if d < base_distance * 2.0 => BirdLOD::Simplified,
        d if d < base_distance * 4.0 => BirdLOD::Billboard,
        _ => BirdLOD::Point,
    }
}
```

---

## Audio Integration

### Bird Sound Types

```rust
// audio/birds.rs

#[derive(Debug, Clone, Copy)]
pub enum BirdSoundType {
    // Vocalizations
    Song,           // Territorial/mating song
    Call,           // Contact call
    Alarm,          // Warning call
    Distress,       // When fleeing

    // Physical sounds
    WingFlap,       // During flight
    WingBurst,      // Takeoff
    Landing,        // Perch impact
    Foraging,       // Ground scratching

    // Species-specific
    Gobble,         // Turkey
    Honk,           // Goose
    Caw,            // Crow
    Hoot,           // Owl
    Screech,        // Hawk
}

pub struct BirdSoundConfig {
    pub sound_type: BirdSoundType,
    pub file: &'static str,
    pub volume: f32,
    pub pitch_variance: f32,    // Random pitch adjustment
    pub min_interval: f32,      // Seconds between plays
    pub distance_falloff: f32,  // How far sound travels
    pub is_looping: bool,
}

impl BirdSpecies {
    pub fn sounds(&self) -> Vec<BirdSoundConfig> {
        match self {
            Self::NorthernCardinal => vec![
                BirdSoundConfig {
                    sound_type: BirdSoundType::Song,
                    file: "sounds/birds/cardinal_song.ogg",
                    volume: 0.6,
                    pitch_variance: 0.1,
                    min_interval: 8.0,
                    distance_falloff: 30.0,
                    is_looping: false,
                },
                BirdSoundConfig {
                    sound_type: BirdSoundType::Call,
                    file: "sounds/birds/cardinal_chip.ogg",
                    volume: 0.4,
                    pitch_variance: 0.15,
                    min_interval: 3.0,
                    distance_falloff: 20.0,
                    is_looping: false,
                },
            ],

            Self::AmericanCrow => vec![
                BirdSoundConfig {
                    sound_type: BirdSoundType::Caw,
                    file: "sounds/birds/crow_caw.ogg",
                    volume: 0.8,
                    pitch_variance: 0.2,
                    min_interval: 2.0,
                    distance_falloff: 80.0,
                    is_looping: false,
                },
            ],

            Self::BarredOwl => vec![
                BirdSoundConfig {
                    sound_type: BirdSoundType::Hoot,
                    file: "sounds/birds/barred_owl_hoot.ogg",
                    volume: 0.7,
                    pitch_variance: 0.05,
                    min_interval: 15.0,
                    distance_falloff: 100.0,
                    is_looping: false,
                },
            ],

            // Generic wing sounds for all species
            _ => vec![],
        }
    }

    pub fn wing_sound(&self) -> Option<&'static str> {
        match self.category() {
            BirdCategory::Raptor => Some("sounds/birds/large_wingflap.ogg"),
            BirdCategory::Waterfowl => Some("sounds/birds/heavy_wingflap.ogg"),
            BirdCategory::GameBird => Some("sounds/birds/burst_wingflap.ogg"),
            BirdCategory::Corvid => Some("sounds/birds/medium_wingflap.ogg"),
            BirdCategory::Songbird => Some("sounds/birds/small_wingflap.ogg"),
            BirdCategory::Hummingbird => Some("sounds/birds/hum.ogg"),
            BirdCategory::Shorebird => Some("sounds/birds/small_wingflap.ogg"),
        }
    }
}
```

### Sound Trigger Logic

```rust
// audio/birds.rs (continued)

pub fn update_bird_audio(
    bird: &Bird,
    audio: &mut AudioManager,
    player_pos: Vec3,
    dt: f32,
) {
    let distance = (bird.position - player_pos).length();

    // Don't process audio for distant birds
    if distance > 150.0 {
        return;
    }

    // Wing sounds during flight
    if matches!(bird.flight.mode, FlightMode::Flapping | FlightMode::TakingOff) {
        if let Some(wing_sound) = bird.species.wing_sound() {
            let flap_phase = bird.flight.wing_phase;

            // Trigger on downstroke (phase 0.0-0.1)
            if flap_phase < 0.1 && bird.last_wing_phase > 0.9 {
                audio.play_3d(
                    wing_sound,
                    bird.position,
                    0.3 / (distance / 20.0).max(1.0),
                );
            }
        }
    }

    // Burst sound on takeoff
    if bird.flight.mode == FlightMode::TakingOff && bird.last_flight_mode != FlightMode::TakingOff {
        if let Some(wing_sound) = bird.species.wing_sound() {
            audio.play_3d(wing_sound, bird.position, 0.8);
        }
    }

    // Vocalization based on behavior state
    match &bird.behavior {
        BirdBehaviorState::Perched(PerchedState::Calling) => {
            if let Some(call_config) = bird.species.sounds()
                .iter()
                .find(|s| matches!(s.sound_type, BirdSoundType::Song | BirdSoundType::Call))
            {
                audio.play_3d(
                    call_config.file,
                    bird.position,
                    call_config.volume / (distance / call_config.distance_falloff).max(1.0),
                );
            }
        }

        BirdBehaviorState::Fleeing(_) => {
            // Alarm calls
            if let Some(alarm) = bird.species.sounds()
                .iter()
                .find(|s| matches!(s.sound_type, BirdSoundType::Alarm))
            {
                audio.play_3d(alarm.file, bird.position, alarm.volume);
            }
        }

        _ => {}
    }
}
```

---

## Performance Considerations

### Budgets

| Resource | Budget | Notes |
|----------|--------|-------|
| Active birds | 200 | Full simulation |
| Visible birds | 100 | With LOD |
| Flocks | 20 | With boids |
| Audio sources | 10 | Closest birds only |
| Full-detail models | 30 | Within LOD range |

### Optimization Strategies

```rust
// Performance optimizations

/// Update frequency based on distance
pub fn get_update_frequency(distance: f32) -> UpdateFrequency {
    match distance {
        d if d < 30.0 => UpdateFrequency::EveryFrame,
        d if d < 80.0 => UpdateFrequency::Every2Frames,
        d if d < 150.0 => UpdateFrequency::Every4Frames,
        _ => UpdateFrequency::Every8Frames,
    }
}

/// Spatial partitioning for flock queries
pub struct BirdSpatialHash {
    cell_size: f32,
    cells: HashMap<(i32, i32, i32), Vec<BirdId>>,
}

impl BirdSpatialHash {
    pub fn new() -> Self {
        Self {
            cell_size: 20.0,  // Larger cells for flying creatures
            cells: HashMap::new(),
        }
    }

    pub fn query_sphere(&self, center: Vec3, radius: f32) -> Vec<BirdId> {
        // 3D spatial query for neighbors
        let min_cell = self.pos_to_cell(center - Vec3::splat(radius));
        let max_cell = self.pos_to_cell(center + Vec3::splat(radius));

        let mut results = Vec::new();
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    if let Some(birds) = self.cells.get(&(x, y, z)) {
                        results.extend(birds.iter().copied());
                    }
                }
            }
        }
        results
    }
}

/// Batch processing for boids
pub fn update_flocks_batched(
    flocks: &mut [Flock],
    birds: &mut HashMap<BirdId, Bird>,
    spatial: &BirdSpatialHash,
    config: &BoidsConfig,
    dt: f32,
) {
    // Pre-compute all steering forces
    let mut steering_forces: HashMap<BirdId, Vec3> = HashMap::new();

    for flock in flocks.iter() {
        for &bird_id in &flock.members {
            if let Some(bird) = birds.get(&bird_id) {
                let steering = calculate_boids_steering(bird, flock, birds, config);
                steering_forces.insert(bird_id, steering);
            }
        }
    }

    // Apply all forces
    for (bird_id, steering) in steering_forces {
        if let Some(bird) = birds.get_mut(&bird_id) {
            bird.flight.velocity += steering * dt;
        }
    }
}
```

---

## Implementation Phases

### Phase 1: Core Flight (Week 1)
- [ ] Add `BirdSpecies` enum to `types.rs`
- [ ] Create `flight.rs` with `BirdFlight` component
- [ ] Implement basic flight physics (lift, drag, gravity)
- [ ] Add `FlightMode` state machine
- [ ] Test with single bird spawned in air

### Phase 2: Wing Animation (Week 2)
- [ ] Create `wing_ik.rs` with `WingIK` struct
- [ ] Implement wing pose calculations per flight mode
- [ ] Create bird shader with wing bone animation
- [ ] Import or create test bird model (crow or hawk)

### Phase 3: Behavior AI (Week 3)
- [ ] Create `birds/behavior.rs` with `BirdBehaviorState`
- [ ] Implement perched behavior states
- [ ] Implement flee behavior
- [ ] Implement cruising/wandering
- [ ] Add raptor hunting behavior

### Phase 4: Perching System (Week 4)
- [ ] Create `perch.rs` with `PerchPoint` struct
- [ ] Generate perch points from tree system
- [ ] Implement landing approach calculation
- [ ] Implement takeoff logic
- [ ] Test perch → fly → perch cycle

### Phase 5: Flocking (Week 5)
- [ ] Create `flock.rs` with `Flock` and `FlockManager`
- [ ] Implement boids algorithm
- [ ] Add V-formation for geese
- [ ] Add cluster formation for songbirds
- [ ] Test flocks of 10-20 birds

### Phase 6: Spawning (Week 6)
- [ ] Integrate with existing spawner system
- [ ] Add habitat-based bird spawning
- [ ] Implement time-of-day filtering
- [ ] Add flock spawning
- [ ] Balance spawn densities

### Phase 7: Polish (Week 7)
- [ ] Add audio triggers
- [ ] Implement LOD system
- [ ] Performance profiling and optimization
- [ ] Add remaining species configurations
- [ ] Final integration testing

---

## Testing Checklist

### Flight Physics
- [ ] Bird maintains altitude in level flight
- [ ] Bird stalls when speed drops below threshold
- [ ] Gliding bird gradually loses altitude
- [ ] Diving bird accelerates correctly
- [ ] Hummingbird can hover stationary
- [ ] Banking turns look natural

### Wing Animation
- [ ] Wing flap cycle is smooth
- [ ] Wings fold during dive
- [ ] Wings spread during landing
- [ ] Asymmetric wing positions when banking
- [ ] Perched birds have folded wings

### Behavior
- [ ] Birds flee from player within range
- [ ] Perched birds transition between idle states
- [ ] Raptors circle and hunt
- [ ] Birds land on valid perches
- [ ] Birds take off when disturbed

### Flocking
- [ ] Flock stays together
- [ ] V-formation maintained by geese
- [ ] No collisions between flock members
- [ ] Flock responds to threats as group
- [ ] New members join nearby flock

### Performance
- [ ] 100 birds at 60 FPS
- [ ] No GC spikes from boids
- [ ] LOD transitions are smooth
- [ ] Distant birds culled properly
- [ ] Audio doesn't overload

### Integration
- [ ] Birds spawn in correct habitats
- [ ] Nocturnal birds active at night
- [ ] Raptors hunt docile fauna
- [ ] Birds avoid water (except waterfowl)
- [ ] Save/load preserves bird state
