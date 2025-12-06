# Naval Combat & Ship Battles Specification

## Roanoke Engine - Age of Sail Warfare

This document specifies the comprehensive naval combat system for Roanoke, including ship types, sailing mechanics, cannon warfare, boarding actions, and the unique challenges of colonial-era maritime warfare.

---

## Table of Contents

1. [Overview](#overview)
2. [Ship Types & Classes](#ship-types--classes)
3. [Sailing Mechanics](#sailing-mechanics)
4. [Naval Combat System](#naval-combat-system)
5. [Cannon Warfare](#cannon-warfare)
6. [Boarding Actions](#boarding-actions)
7. [Ship Damage & Repair](#ship-damage--repair)
8. [Crew Management](#crew-management)
9. [Weather & Sea Conditions](#weather--sea-conditions)
10. [Naval AI](#naval-ai)
11. [Piracy & Privateering](#piracy--privateering)
12. [Port & Docking](#port--docking)
13. [Data Structures](#data-structures)

---

## Overview

### Design Philosophy

Naval warfare in the late 16th century was a brutal affair of wooden ships, black powder, and steel nerves. The Spanish treasure fleets, English privateers, and Native American canoes all plied the waters near Roanoke. This system captures the tension of age-of-sail combat while remaining accessible and exciting.

### Historical Context

- **1585-1590**: Roanoke Colony period
- **Spanish Armada**: 1588 - Naval warfare at its peak
- **English Privateers**: Drake, Hawkins, Raleigh
- **Native Watercraft**: Dugout canoes, skilled river navigation

### Core Features

| Feature | Description |
|---------|-------------|
| Ship Classes | 8 distinct vessel types from canoes to galleons |
| Wind Mechanics | Realistic sailing with wind direction and speed |
| Cannon Combat | Broadside warfare with multiple ammo types |
| Boarding | Close-quarters combat on ship decks |
| Crew System | Manage sailors, gunners, and marines |
| Storm Sailing | Navigate hurricanes and storms |
| Ship Customization | Upgrade and modify your vessel |

---

## Ship Types & Classes

### Ship Classification

```rust
// ships/types.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShipClass {
    // Native/Small Craft
    Canoe,              // 1-4 crew, no guns
    Pinnace,            // 10-20 crew, 4-8 guns

    // Light Ships
    Sloop,              // 20-40 crew, 8-14 guns
    Brigantine,         // 30-60 crew, 12-20 guns

    // Medium Ships
    Caravel,            // 20-30 crew, 8-12 guns (exploration)
    Fluyt,              // 40-60 crew, 10-20 guns (cargo)
    Barque,             // 50-80 crew, 20-30 guns

    // Heavy Ships
    Frigate,            // 100-150 crew, 28-44 guns
    Galleon,            // 150-300 crew, 50-74 guns

    // Special
    Fireship,           // Unmanned, explosive
    WarCanoe,           // Native war vessel
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipDef {
    pub class: ShipClass,
    pub name: &'static str,

    // Dimensions
    pub length: f32,           // meters
    pub beam: f32,             // width
    pub draft: f32,            // depth below water
    pub displacement: f32,     // tons

    // Crew
    pub min_crew: u32,
    pub optimal_crew: u32,
    pub max_crew: u32,
    pub max_marines: u32,

    // Armament
    pub gun_ports: GunPorts,
    pub max_gun_weight: u32,   // Heaviest cannon allowed

    // Performance
    pub max_speed: f32,        // knots
    pub acceleration: f32,
    pub turn_rate: f32,        // degrees per second
    pub handling: f32,         // 0-1, affects responsiveness

    // Durability
    pub hull_points: f32,
    pub sail_points: f32,
    pub armor_rating: f32,

    // Cargo
    pub cargo_capacity: u32,   // tons

    // Cost
    pub purchase_price: u32,
    pub daily_upkeep: u32,

    // Sailing characteristics
    pub best_point_of_sail: PointOfSail,
    pub into_wind_capability: f32,  // How close to wind it can sail
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GunPorts {
    pub bow_chasers: u32,
    pub stern_chasers: u32,
    pub port_broadside: u32,
    pub starboard_broadside: u32,
    pub total: u32,
}

pub fn get_ship_def(class: ShipClass) -> ShipDef {
    match class {
        ShipClass::Canoe => ShipDef {
            class: ShipClass::Canoe,
            name: "Dugout Canoe",

            length: 6.0,
            beam: 1.0,
            draft: 0.3,
            displacement: 0.5,

            min_crew: 1,
            optimal_crew: 2,
            max_crew: 4,
            max_marines: 2,

            gun_ports: GunPorts::none(),
            max_gun_weight: 0,

            max_speed: 4.0,  // Paddled
            acceleration: 2.0,
            turn_rate: 45.0,
            handling: 1.0,

            hull_points: 50.0,
            sail_points: 0.0,
            armor_rating: 0.0,

            cargo_capacity: 2,
            purchase_price: 50,
            daily_upkeep: 0,

            best_point_of_sail: PointOfSail::Any,  // Paddled
            into_wind_capability: 1.0,  // Can paddle into wind
        },

        ShipClass::Pinnace => ShipDef {
            class: ShipClass::Pinnace,
            name: "Pinnace",

            length: 15.0,
            beam: 5.0,
            draft: 2.0,
            displacement: 40.0,

            min_crew: 8,
            optimal_crew: 15,
            max_crew: 25,
            max_marines: 10,

            gun_ports: GunPorts {
                bow_chasers: 2,
                stern_chasers: 0,
                port_broadside: 2,
                starboard_broadside: 2,
                total: 6,
            },
            max_gun_weight: 6,  // Falconet or saker

            max_speed: 8.0,
            acceleration: 0.8,
            turn_rate: 15.0,
            handling: 0.9,

            hull_points: 200.0,
            sail_points: 100.0,
            armor_rating: 0.1,

            cargo_capacity: 20,
            purchase_price: 2000,
            daily_upkeep: 20,

            best_point_of_sail: PointOfSail::BeamReach,
            into_wind_capability: 0.6,
        },

        ShipClass::Sloop => ShipDef {
            class: ShipClass::Sloop,
            name: "Sloop",

            length: 20.0,
            beam: 7.0,
            draft: 2.5,
            displacement: 100.0,

            min_crew: 15,
            optimal_crew: 30,
            max_crew: 50,
            max_marines: 20,

            gun_ports: GunPorts {
                bow_chasers: 2,
                stern_chasers: 2,
                port_broadside: 5,
                starboard_broadside: 5,
                total: 14,
            },
            max_gun_weight: 9,

            max_speed: 11.0,
            acceleration: 0.6,
            turn_rate: 12.0,
            handling: 0.85,

            hull_points: 350.0,
            sail_points: 150.0,
            armor_rating: 0.15,

            cargo_capacity: 50,
            purchase_price: 8000,
            daily_upkeep: 50,

            best_point_of_sail: PointOfSail::CloseHauled,
            into_wind_capability: 0.5,
        },

        ShipClass::Galleon => ShipDef {
            class: ShipClass::Galleon,
            name: "Galleon",

            length: 40.0,
            beam: 12.0,
            draft: 5.0,
            displacement: 500.0,

            min_crew: 100,
            optimal_crew: 200,
            max_crew: 350,
            max_marines: 100,

            gun_ports: GunPorts {
                bow_chasers: 4,
                stern_chasers: 4,
                port_broadside: 30,
                starboard_broadside: 30,
                total: 68,
            },
            max_gun_weight: 42,  // Full cannons

            max_speed: 7.0,
            acceleration: 0.2,
            turn_rate: 3.0,
            handling: 0.4,

            hull_points: 2000.0,
            sail_points: 400.0,
            armor_rating: 0.5,

            cargo_capacity: 400,
            purchase_price: 100000,
            daily_upkeep: 300,

            best_point_of_sail: PointOfSail::BroadReach,
            into_wind_capability: 0.35,
        },

        // ... other ships
    }
}
```

### Ship Instance

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub id: ShipId,
    pub name: String,
    pub class: ShipClass,
    pub owner: Owner,

    // Position and movement
    pub position: Vec3,
    pub heading: f32,         // Degrees, 0 = North
    pub velocity: Vec3,
    pub current_speed: f32,

    // Sail state
    pub sail_state: SailState,
    pub rudder_position: f32,  // -1.0 to 1.0

    // Health
    pub hull_health: f32,
    pub sail_health: f32,
    pub mast_health: [f32; 3],  // Fore, main, mizzen
    pub flooding: f32,          // 0.0 - 1.0

    // Armament
    pub cannons: Vec<Cannon>,
    pub ammo: AmmoStore,

    // Crew
    pub crew: CrewComplement,
    pub morale: f32,

    // Cargo
    pub cargo: Vec<CargoItem>,
    pub cargo_weight: f32,

    // Flags and state
    pub flags: ShipFlags,
    pub combat_state: CombatState,
    pub damage_state: DamageState,
}

#[derive(Debug, Clone, Copy)]
pub enum SailState {
    Furled,           // No sails
    BattleSails,      // Reduced for combat maneuverability
    PlainSails,       // Normal sailing
    FullSails,        // Maximum speed
    Damaged,          // Sails destroyed
}

impl SailState {
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            Self::Furled => 0.0,
            Self::BattleSails => 0.5,
            Self::PlainSails => 0.8,
            Self::FullSails => 1.0,
            Self::Damaged => 0.2,
        }
    }

    pub fn turn_multiplier(&self) -> f32 {
        match self {
            Self::Furled => 0.3,     // Drift only
            Self::BattleSails => 1.0, // Best maneuverability
            Self::PlainSails => 0.8,
            Self::FullSails => 0.6,   // Hard to turn at full speed
            Self::Damaged => 0.4,
        }
    }
}
```

---

## Sailing Mechanics

### Wind System

```rust
// ships/sailing.rs

#[derive(Debug, Clone)]
pub struct WindState {
    pub direction: f32,        // Degrees, where wind is coming FROM
    pub speed: f32,            // knots
    pub gusts: f32,            // Gust intensity multiplier
    pub variability: f32,      // How much direction varies
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointOfSail {
    InIrons,          // Pointing into wind - stalled
    CloseHauled,      // 30-50 degrees off wind
    CloseReach,       // 50-70 degrees
    BeamReach,        // 70-110 degrees - optimal for most ships
    BroadReach,       // 110-150 degrees
    Running,          // 150-180 degrees - wind from behind
    Any,              // Paddled/rowed craft
}

impl PointOfSail {
    pub fn from_relative_angle(angle: f32) -> Self {
        let abs_angle = angle.abs();
        match abs_angle {
            a if a < 30.0 => Self::InIrons,
            a if a < 50.0 => Self::CloseHauled,
            a if a < 70.0 => Self::CloseReach,
            a if a < 110.0 => Self::BeamReach,
            a if a < 150.0 => Self::BroadReach,
            _ => Self::Running,
        }
    }

    pub fn base_speed_factor(&self) -> f32 {
        match self {
            Self::InIrons => 0.0,
            Self::CloseHauled => 0.4,
            Self::CloseReach => 0.7,
            Self::BeamReach => 1.0,
            Self::BroadReach => 0.9,
            Self::Running => 0.7,
            Self::Any => 1.0,
        }
    }
}

pub struct SailingCalculator;

impl SailingCalculator {
    /// Calculate effective speed based on wind and heading
    pub fn calculate_speed(
        ship: &Ship,
        wind: &WindState,
        def: &ShipDef,
    ) -> f32 {
        // Get relative wind angle
        let relative_wind = normalize_angle(wind.direction - ship.heading);
        let point_of_sail = PointOfSail::from_relative_angle(relative_wind);

        // Can this ship sail at this point?
        if point_of_sail == PointOfSail::InIrons {
            return 0.0;
        }

        // Check if ship can sail this close to wind
        let min_angle = (1.0 - def.into_wind_capability) * 90.0;
        if relative_wind.abs() < min_angle {
            return 0.0;  // Can't sail this close to wind
        }

        // Base speed from point of sail
        let base_factor = point_of_sail.base_speed_factor();

        // Adjust for ship's best point of sail
        let ship_bonus = if point_of_sail == def.best_point_of_sail {
            1.15
        } else {
            1.0
        };

        // Wind speed factor (ships have max hull speed)
        let wind_factor = (wind.speed / 15.0).min(1.0);  // Optimal at 15 knots

        // Strong winds can be dangerous
        let danger_factor = if wind.speed > 30.0 {
            0.8 - (wind.speed - 30.0) * 0.02
        } else {
            1.0
        };

        // Sail state
        let sail_factor = ship.sail_state.speed_multiplier();

        // Damage factor
        let damage_factor = ship.sail_health / 100.0;

        // Crew factor (need minimum crew to sail)
        let crew_factor = (ship.crew.sailors as f32 / def.optimal_crew as f32).min(1.0);

        // Cargo weight penalty
        let load_factor = 1.0 - (ship.cargo_weight / (def.cargo_capacity as f32 * 2.0));

        def.max_speed
            * base_factor
            * ship_bonus
            * wind_factor
            * danger_factor
            * sail_factor
            * damage_factor
            * crew_factor
            * load_factor.max(0.3)
    }

    /// Calculate turn rate
    pub fn calculate_turn_rate(
        ship: &Ship,
        wind: &WindState,
        def: &ShipDef,
    ) -> f32 {
        let base_rate = def.turn_rate;

        // Sail state
        let sail_factor = ship.sail_state.turn_multiplier();

        // Speed factor - faster = harder to turn
        let speed_factor = 1.0 - (ship.current_speed / def.max_speed) * 0.4;

        // Rudder damage
        let rudder_factor = if ship.damage_state.rudder_damaged { 0.3 } else { 1.0 };

        // Crew
        let crew_factor = (ship.crew.sailors as f32 / def.optimal_crew as f32).min(1.0);

        base_rate * sail_factor * speed_factor * rudder_factor * crew_factor
    }

    /// Simulate tacking (turning through the wind)
    pub fn attempt_tack(
        ship: &mut Ship,
        wind: &WindState,
        def: &ShipDef,
    ) -> TackResult {
        // Can't tack if in irons
        if ship.current_speed < 2.0 {
            return TackResult::Stalled;
        }

        // Crew skill check
        let skill_factor = ship.crew.average_skill();

        // Wind strength affects difficulty
        let wind_difficulty = if wind.speed > 20.0 {
            0.7
        } else if wind.speed < 5.0 {
            0.8  // Light winds = sluggish tack
        } else {
            1.0
        };

        // Ship handling
        let handling = def.handling;

        let success_chance = skill_factor * wind_difficulty * handling;

        if rand::random::<f32>() < success_chance {
            TackResult::Success
        } else {
            // Failed tack - stuck in irons
            ship.current_speed *= 0.3;
            TackResult::InIrons
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TackResult {
    Success,
    InIrons,
    Stalled,
}
```

### Ship Movement Update

```rust
impl Ship {
    pub fn update_movement(&mut self, wind: &WindState, dt: f32) {
        let def = get_ship_def(self.class);

        // Calculate target speed
        let target_speed = SailingCalculator::calculate_speed(self, wind, &def);

        // Accelerate/decelerate toward target
        let speed_diff = target_speed - self.current_speed;
        let accel = def.acceleration * if speed_diff > 0.0 { 1.0 } else { 2.0 };  // Slowing is faster
        self.current_speed += speed_diff.clamp(-accel * dt, accel * dt);

        // Apply rudder turn
        if self.rudder_position.abs() > 0.01 {
            let turn_rate = SailingCalculator::calculate_turn_rate(self, wind, &def);
            let turn = turn_rate * self.rudder_position * dt;
            self.heading = normalize_angle(self.heading + turn);
        }

        // Update velocity
        let heading_rad = self.heading.to_radians();
        self.velocity = Vec3::new(
            heading_rad.sin() * self.current_speed * 0.514,  // knots to m/s
            0.0,
            heading_rad.cos() * self.current_speed * 0.514,
        );

        // Apply wind drift (leeway)
        let leeway = self.calculate_leeway(wind, &def);
        self.velocity += leeway;

        // Update position
        self.position += self.velocity * dt;

        // Apply wave effects
        self.apply_wave_motion(dt);
    }

    fn calculate_leeway(&self, wind: &WindState, def: &ShipDef) -> Vec3 {
        // Sideways drift from wind
        let wind_rad = wind.direction.to_radians();
        let wind_vec = Vec3::new(wind_rad.sin(), 0.0, wind_rad.cos());

        // Ships with deeper draft drift less
        let drift_resistance = def.draft / 5.0;

        let drift_speed = wind.speed * 0.05 * (1.0 - drift_resistance);

        // Drift perpendicular to wind
        Vec3::new(-wind_vec.z, 0.0, wind_vec.x) * drift_speed * 0.514
    }
}
```

---

## Naval Combat System

### Combat States

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatState {
    Peace,              // Normal sailing
    BattleStations,     // Crew at guns, sails reduced
    Engaged,            // In combat
    Fleeing,            // Attempting escape
    Surrendered,        // Struck colors
    Sinking,            // Catastrophic damage
}

#[derive(Debug, Clone)]
pub struct NavalCombat {
    pub participants: Vec<ShipId>,
    pub combat_state: CombatPhase,
    pub start_time: f64,
    pub elapsed: f32,

    // Tracking
    pub shots_fired: u32,
    pub hits_landed: u32,
    pub damage_dealt: HashMap<ShipId, f32>,
    pub casualties: HashMap<ShipId, u32>,

    // Boarding
    pub boarding_in_progress: Option<BoardingAction>,
}

#[derive(Debug, Clone, Copy)]
pub enum CombatPhase {
    Approaching,        // Ships closing
    Opening,            // First volleys
    CloseAction,        // Point-blank range
    Boarding,           // Ships grappled
    Breaking,           // Ships separating
    Pursuit,            // Chase phase
    Concluded,          // Combat over
}

impl NavalCombat {
    pub fn check_engagement(ship_a: &Ship, ship_b: &Ship) -> Option<Self> {
        let distance = ship_a.position.distance(ship_b.position);

        // Maximum engagement distance
        if distance > 1500.0 {
            return None;
        }

        // Check if either ship is hostile
        if !are_hostile(ship_a.owner, ship_b.owner) {
            return None;
        }

        Some(Self {
            participants: vec![ship_a.id, ship_b.id],
            combat_state: CombatPhase::Approaching,
            start_time: get_game_time(),
            elapsed: 0.0,
            shots_fired: 0,
            hits_landed: 0,
            damage_dealt: HashMap::new(),
            casualties: HashMap::new(),
            boarding_in_progress: None,
        })
    }

    pub fn update(&mut self, ships: &mut [Ship], dt: f32) {
        self.elapsed += dt;

        // Update phase based on distances
        let (ship_a, ship_b) = get_two_ships(ships, self.participants[0], self.participants[1]);
        let distance = ship_a.position.distance(ship_b.position);

        self.combat_state = match (self.combat_state, distance) {
            (_, d) if d > 1200.0 => CombatPhase::Approaching,
            (CombatPhase::Approaching, d) if d < 800.0 => CombatPhase::Opening,
            (CombatPhase::Opening, d) if d < 300.0 => CombatPhase::CloseAction,
            (_, d) if d < 30.0 && self.boarding_in_progress.is_some() => CombatPhase::Boarding,
            (CombatPhase::Boarding, d) if d > 50.0 => CombatPhase::Breaking,
            (CombatPhase::Breaking, d) if d > 800.0 => CombatPhase::Pursuit,
            (phase, _) => phase,
        };

        // Check for combat conclusion
        for ship in ships.iter() {
            if self.participants.contains(&ship.id) {
                if ship.combat_state == CombatState::Surrendered
                    || ship.combat_state == CombatState::Sinking
                {
                    self.combat_state = CombatPhase::Concluded;
                }
            }
        }
    }
}
```

### Targeting System

```rust
#[derive(Debug, Clone)]
pub struct TargetingSolution {
    pub target: ShipId,
    pub aim_point: AimPoint,
    pub range: f32,
    pub bearing: f32,           // Relative to our heading
    pub deflection: f32,        // Lead angle for moving target
    pub elevation: f32,         // Cannon elevation
    pub hit_probability: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum AimPoint {
    Hull,               // Damage hull, cause flooding
    Waterline,          // Maximum flooding damage
    Deck,               // Kill crew, damage equipment
    Masts,              // Disable sailing
    Rigging,            // Slow the enemy
    Rudder,             // Disable steering
}

impl TargetingSolution {
    pub fn calculate(
        shooter: &Ship,
        target: &Ship,
        aim_point: AimPoint,
        cannon: &Cannon,
    ) -> Self {
        let to_target = target.position - shooter.position;
        let range = to_target.length();
        let bearing = to_target.x.atan2(to_target.z).to_degrees() - shooter.heading;

        // Calculate lead for moving target
        let time_of_flight = range / cannon.muzzle_velocity;
        let target_movement = target.velocity * time_of_flight;
        let deflection = target_movement.x.atan2(target_movement.z).to_degrees();

        // Calculate elevation
        let elevation = Self::calculate_elevation(range, cannon, aim_point);

        // Hit probability
        let hit_prob = Self::calculate_hit_probability(
            range, cannon, shooter, target, aim_point
        );

        Self {
            target: target.id,
            aim_point,
            range,
            bearing: normalize_angle(bearing),
            deflection,
            elevation,
            hit_probability: hit_prob,
        }
    }

    fn calculate_elevation(range: f32, cannon: &Cannon, aim: AimPoint) -> f32 {
        // Basic ballistic calculation
        let g = 9.81;
        let v = cannon.muzzle_velocity;
        let height_offset = match aim {
            AimPoint::Waterline => -2.0,
            AimPoint::Hull => 0.0,
            AimPoint::Deck => 3.0,
            AimPoint::Masts => 15.0,
            AimPoint::Rigging => 10.0,
            AimPoint::Rudder => -1.0,
        };

        // Simplified elevation: arcsin(range * g / v²) / 2
        let base_angle = ((range * g) / (v * v)).asin() / 2.0;
        base_angle.to_degrees() + (height_offset / range).atan().to_degrees()
    }

    fn calculate_hit_probability(
        range: f32,
        cannon: &Cannon,
        shooter: &Ship,
        target: &Ship,
        aim: AimPoint,
    ) -> f32 {
        // Base accuracy from range
        let range_factor = match range {
            r if r < 100.0 => 0.9,
            r if r < 300.0 => 0.7,
            r if r < 600.0 => 0.5,
            r if r < 1000.0 => 0.3,
            _ => 0.1,
        };

        // Cannon quality
        let cannon_factor = cannon.accuracy;

        // Crew skill
        let crew_skill = shooter.crew.gunnery_skill();

        // Target size
        let target_size = get_ship_def(target.class).length / 30.0;

        // Target movement
        let movement_penalty = (target.current_speed / 10.0).min(0.3);

        // Sea state penalty
        let sea_penalty = 0.0;  // TODO: From wave height

        // Aim difficulty
        let aim_penalty = match aim {
            AimPoint::Hull => 0.0,
            AimPoint::Waterline => 0.1,
            AimPoint::Deck => 0.1,
            AimPoint::Masts => 0.3,
            AimPoint::Rigging => 0.4,
            AimPoint::Rudder => 0.35,
        };

        (range_factor * cannon_factor * crew_skill * target_size
            - movement_penalty - sea_penalty - aim_penalty).clamp(0.05, 0.95)
    }
}
```

---

## Cannon Warfare

### Cannon Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CannonType {
    Swivel,             // Anti-personnel, 0.5 lb
    Falconet,           // Light, 2 lb
    Falcon,             // Light, 3 lb
    Minion,             // 4 lb
    Saker,              // Medium, 6 lb
    Culverin,           // Long range, 18 lb
    DemiCannon,         // Heavy, 32 lb
    Cannon,             // Full cannon, 42 lb
    Carronade,          // Short range, devastating, 32 lb (anachronistic)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cannon {
    pub cannon_type: CannonType,
    pub position: CannonPosition,
    pub loaded_ammo: Option<AmmoType>,
    pub reload_progress: f32,     // 0.0 - 1.0
    pub heat: f32,                // Affects reload, explosion risk
    pub condition: f32,           // Damage state
    pub crew_assigned: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum CannonPosition {
    BowChaser { slot: u32 },
    SternChaser { slot: u32 },
    PortBroadside { deck: u32, slot: u32 },
    StarboardBroadside { deck: u32, slot: u32 },
    Swivel { slot: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CannonDef {
    pub cannon_type: CannonType,
    pub name: &'static str,

    pub weight: u32,               // pounds of shot
    pub cannon_weight: u32,        // weight of cannon in lbs
    pub range_effective: f32,      // meters
    pub range_maximum: f32,
    pub muzzle_velocity: f32,      // m/s
    pub reload_time: f32,          // seconds
    pub crew_required: u32,

    pub hull_damage: f32,
    pub sail_damage: f32,
    pub crew_damage: f32,

    pub accuracy: f32,             // 0.0 - 1.0
    pub penetration: f32,          // vs armor
}

pub fn get_cannon_def(cannon_type: CannonType) -> CannonDef {
    match cannon_type {
        CannonType::Swivel => CannonDef {
            cannon_type: CannonType::Swivel,
            name: "Swivel Gun",
            weight: 1,
            cannon_weight: 50,
            range_effective: 100.0,
            range_maximum: 300.0,
            muzzle_velocity: 200.0,
            reload_time: 10.0,
            crew_required: 1,
            hull_damage: 5.0,
            sail_damage: 10.0,
            crew_damage: 25.0,
            accuracy: 0.8,
            penetration: 0.1,
        },

        CannonType::Saker => CannonDef {
            cannon_type: CannonType::Saker,
            name: "Saker",
            weight: 6,
            cannon_weight: 1500,
            range_effective: 500.0,
            range_maximum: 1500.0,
            muzzle_velocity: 400.0,
            reload_time: 45.0,
            crew_required: 3,
            hull_damage: 25.0,
            sail_damage: 15.0,
            crew_damage: 15.0,
            accuracy: 0.6,
            penetration: 0.4,
        },

        CannonType::Culverin => CannonDef {
            cannon_type: CannonType::Culverin,
            name: "Culverin",
            weight: 18,
            cannon_weight: 4000,
            range_effective: 800.0,
            range_maximum: 2500.0,
            muzzle_velocity: 450.0,
            reload_time: 60.0,
            crew_required: 4,
            hull_damage: 50.0,
            sail_damage: 25.0,
            crew_damage: 20.0,
            accuracy: 0.5,
            penetration: 0.6,
        },

        CannonType::Cannon => CannonDef {
            cannon_type: CannonType::Cannon,
            name: "Full Cannon",
            weight: 42,
            cannon_weight: 6000,
            range_effective: 600.0,
            range_maximum: 2000.0,
            muzzle_velocity: 400.0,
            reload_time: 90.0,
            crew_required: 6,
            hull_damage: 100.0,
            sail_damage: 40.0,
            crew_damage: 30.0,
            accuracy: 0.4,
            penetration: 0.9,
        },

        // ... other cannon types
    }
}
```

### Ammunition Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmmoType {
    RoundShot,      // Standard ball - hull damage
    ChainShot,      // Two balls + chain - rigging damage
    BarShot,        // Bar connecting balls - mast damage
    GrapeShot,      // Small balls - crew killer
    Canister,       // Like grapeshot, close range
    HotShot,        // Heated ball - fire starter
    DoubleShot,     // Two balls - close range devastation
}

impl AmmoType {
    pub fn damage_modifiers(&self) -> DamageModifiers {
        match self {
            Self::RoundShot => DamageModifiers {
                hull: 1.0,
                sail: 0.3,
                crew: 0.2,
                fire_chance: 0.0,
                penetration: 1.0,
            },
            Self::ChainShot => DamageModifiers {
                hull: 0.1,
                sail: 2.0,
                crew: 0.1,
                fire_chance: 0.0,
                penetration: 0.1,
            },
            Self::BarShot => DamageModifiers {
                hull: 0.2,
                sail: 1.5,
                crew: 0.2,
                fire_chance: 0.0,
                penetration: 0.2,
            },
            Self::GrapeShot => DamageModifiers {
                hull: 0.05,
                sail: 0.5,
                crew: 3.0,
                fire_chance: 0.0,
                penetration: 0.05,
            },
            Self::Canister => DamageModifiers {
                hull: 0.1,
                sail: 0.3,
                crew: 4.0,
                fire_chance: 0.0,
                penetration: 0.05,
            },
            Self::HotShot => DamageModifiers {
                hull: 0.8,
                sail: 0.5,
                crew: 0.2,
                fire_chance: 0.4,
                penetration: 0.9,
            },
            Self::DoubleShot => DamageModifiers {
                hull: 1.8,
                sail: 0.2,
                crew: 0.3,
                fire_chance: 0.0,
                penetration: 1.5,
            },
        }
    }

    pub fn effective_range_modifier(&self) -> f32 {
        match self {
            Self::RoundShot => 1.0,
            Self::ChainShot => 0.6,
            Self::BarShot => 0.7,
            Self::GrapeShot => 0.3,
            Self::Canister => 0.2,
            Self::HotShot => 0.8,
            Self::DoubleShot => 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DamageModifiers {
    pub hull: f32,
    pub sail: f32,
    pub crew: f32,
    pub fire_chance: f32,
    pub penetration: f32,
}
```

### Firing Mechanics

```rust
pub struct CannonFireSystem;

impl CannonFireSystem {
    pub fn fire_broadside(
        ship: &mut Ship,
        side: BroadsideSide,
        target: &TargetingSolution,
    ) -> BroadsideResult {
        let def = get_ship_def(ship.class);
        let cannons_on_side: Vec<&mut Cannon> = ship.cannons.iter_mut()
            .filter(|c| c.position.matches_side(side) && c.is_ready())
            .collect();

        let mut result = BroadsideResult::default();

        for cannon in cannons_on_side {
            let fire_result = Self::fire_cannon(cannon, target, &ship.crew);
            result.add(fire_result);
        }

        // Morale boost from successful broadside
        if result.hits > 0 {
            ship.morale = (ship.morale + 0.02 * result.hits as f32).min(1.0);
        }

        result
    }

    fn fire_cannon(
        cannon: &mut Cannon,
        target: &TargetingSolution,
        crew: &CrewComplement,
    ) -> ShotResult {
        let cannon_def = get_cannon_def(cannon.cannon_type);

        // Can't fire if not loaded
        let ammo_type = match cannon.loaded_ammo.take() {
            Some(ammo) => ammo,
            None => return ShotResult::NotLoaded,
        };

        // Start reload
        cannon.reload_progress = 0.0;

        // Heat increases
        cannon.heat += 0.2;

        // Calculate hit
        let effective_range = cannon_def.range_effective * ammo_type.effective_range_modifier();
        let range_penalty = if target.range > effective_range {
            (target.range / effective_range - 1.0).min(0.5)
        } else {
            0.0
        };

        let hit_chance = target.hit_probability - range_penalty;

        if rand::random::<f32>() > hit_chance {
            return ShotResult::Miss;
        }

        // Calculate damage
        let mods = ammo_type.damage_modifiers();
        let base_damage = match target.aim_point {
            AimPoint::Hull | AimPoint::Waterline => cannon_def.hull_damage * mods.hull,
            AimPoint::Deck => cannon_def.crew_damage * mods.crew,
            AimPoint::Masts | AimPoint::Rigging => cannon_def.sail_damage * mods.sail,
            AimPoint::Rudder => cannon_def.hull_damage * 0.5,
        };

        // Critical hit check
        let is_critical = rand::random::<f32>() < 0.1;

        ShotResult::Hit {
            damage: base_damage * if is_critical { 2.0 } else { 1.0 },
            damage_type: target.aim_point.to_damage_type(),
            fire_started: rand::random::<f32>() < mods.fire_chance,
            critical: is_critical,
        }
    }

    pub fn reload_cannon(cannon: &mut Cannon, crew: &CrewComplement, ammo: AmmoType, dt: f32) {
        let cannon_def = get_cannon_def(cannon.cannon_type);

        // Need sufficient crew
        if cannon.crew_assigned < cannon_def.crew_required {
            return;  // Can't reload
        }

        // Calculate reload speed
        let crew_efficiency = (cannon.crew_assigned as f32 / cannon_def.crew_required as f32).min(1.5);
        let skill_bonus = crew.gunnery_skill();
        let heat_penalty = if cannon.heat > 0.5 { 1.0 + (cannon.heat - 0.5) } else { 1.0 };

        let reload_rate = crew_efficiency * skill_bonus / heat_penalty;
        let reload_progress = dt / cannon_def.reload_time * reload_rate;

        cannon.reload_progress = (cannon.reload_progress + reload_progress).min(1.0);

        if cannon.reload_progress >= 1.0 {
            cannon.loaded_ammo = Some(ammo);
        }

        // Cannon cools
        cannon.heat = (cannon.heat - 0.01 * dt).max(0.0);
    }
}

#[derive(Debug, Clone)]
pub enum ShotResult {
    NotLoaded,
    Miss,
    Hit {
        damage: f32,
        damage_type: DamageType,
        fire_started: bool,
        critical: bool,
    },
}

#[derive(Debug, Clone, Default)]
pub struct BroadsideResult {
    pub shots_fired: u32,
    pub hits: u32,
    pub damage_dealt: f32,
    pub fires_started: u32,
    pub criticals: u32,
}
```

---

## Boarding Actions

### Boarding Mechanics

```rust
#[derive(Debug, Clone)]
pub struct BoardingAction {
    pub attacker: ShipId,
    pub defender: ShipId,
    pub phase: BoardingPhase,

    // Forces
    pub attacker_forces: BoardingForce,
    pub defender_forces: BoardingForce,

    // State
    pub grappled: bool,
    pub attacker_morale: f32,
    pub defender_morale: f32,

    // Zones controlled
    pub zones: BoardingZones,

    // Combat resolution
    pub elapsed: f32,
    pub casualties: (u32, u32),  // (attacker, defender)
}

#[derive(Debug, Clone, Copy)]
pub enum BoardingPhase {
    Approaching,        // Ships maneuvering to grapple
    Grappling,          // Throwing hooks, securing lines
    Swinging,           // Crossing to enemy deck
    MainDeck,           // Fighting on main deck
    BelowDecks,         // Clearing below
    QuarterDeck,        // Fighting for command
    Secured,            // Victory
    Repelled,           // Failed boarding
}

#[derive(Debug, Clone)]
pub struct BoardingForce {
    pub marines: u32,
    pub sailors: u32,
    pub officers: u32,
    pub quality: f32,        // Training/experience
    pub equipment: BoardingEquipment,
}

#[derive(Debug, Clone)]
pub struct BoardingEquipment {
    pub cutlasses: u32,
    pub pistols: u32,
    pub muskets: u32,
    pub pikes: u32,
    pub grenades: u32,
    pub grappling_hooks: u32,
}

#[derive(Debug, Clone)]
pub struct BoardingZones {
    pub bow: ZoneControl,
    pub main_deck: ZoneControl,
    pub quarter_deck: ZoneControl,
    pub below_decks: ZoneControl,
}

#[derive(Debug, Clone, Copy)]
pub enum ZoneControl {
    Defender,
    Contested,
    Attacker,
}

impl BoardingAction {
    pub fn initiate(attacker: &Ship, defender: &Ship) -> Self {
        Self {
            attacker: attacker.id,
            defender: defender.id,
            phase: BoardingPhase::Approaching,

            attacker_forces: BoardingForce::from_ship(attacker),
            defender_forces: BoardingForce::from_ship(defender),

            grappled: false,
            attacker_morale: attacker.morale,
            defender_morale: defender.morale,

            zones: BoardingZones {
                bow: ZoneControl::Defender,
                main_deck: ZoneControl::Defender,
                quarter_deck: ZoneControl::Defender,
                below_decks: ZoneControl::Defender,
            },

            elapsed: 0.0,
            casualties: (0, 0),
        }
    }

    pub fn update(&mut self, attacker: &mut Ship, defender: &mut Ship, dt: f32) {
        self.elapsed += dt;

        match self.phase {
            BoardingPhase::Approaching => {
                // Check if close enough to grapple
                let distance = attacker.position.distance(defender.position);
                if distance < 30.0 {
                    self.phase = BoardingPhase::Grappling;
                }
            },

            BoardingPhase::Grappling => {
                // Roll for grapple success
                let grapple_skill = self.attacker_forces.quality;
                let cut_skill = self.defender_forces.quality;

                if rand::random::<f32>() < grapple_skill * 0.3 * dt {
                    self.grappled = true;
                    self.phase = BoardingPhase::Swinging;
                }

                // Defender can cut lines
                if self.grappled && rand::random::<f32>() < cut_skill * 0.1 * dt {
                    self.grappled = false;
                }
            },

            BoardingPhase::Swinging => {
                // Men crossing to enemy ship
                let crossers = (self.attacker_forces.total() as f32 * 0.1 * dt) as u32;
                // Some may fall/be shot
                let losses = (crossers as f32 * (1.0 - self.attacker_forces.quality) * 0.3) as u32;

                self.casualties.0 += losses;
                self.attacker_forces.marines = self.attacker_forces.marines.saturating_sub(losses);

                // After enough cross, fighting begins
                if self.elapsed > 10.0 {
                    self.phase = BoardingPhase::MainDeck;
                }
            },

            BoardingPhase::MainDeck => {
                self.resolve_zone_combat(&mut self.zones.main_deck, dt);

                if self.zones.main_deck == ZoneControl::Attacker {
                    self.phase = BoardingPhase::QuarterDeck;
                }

                if self.attacker_morale < 0.2 {
                    self.phase = BoardingPhase::Repelled;
                }
            },

            BoardingPhase::QuarterDeck => {
                self.resolve_zone_combat(&mut self.zones.quarter_deck, dt);

                if self.zones.quarter_deck == ZoneControl::Attacker {
                    // Victory check
                    if self.defender_morale < 0.3 {
                        self.phase = BoardingPhase::Secured;
                    } else {
                        self.phase = BoardingPhase::BelowDecks;
                    }
                }

                if self.attacker_morale < 0.2 {
                    self.phase = BoardingPhase::Repelled;
                }
            },

            BoardingPhase::BelowDecks => {
                self.resolve_zone_combat(&mut self.zones.below_decks, dt);

                if self.zones.below_decks == ZoneControl::Attacker
                    && self.defender_morale < 0.2 {
                    self.phase = BoardingPhase::Secured;
                }
            },

            BoardingPhase::Secured => {
                // Boarding succeeded
                defender.combat_state = CombatState::Surrendered;
            },

            BoardingPhase::Repelled => {
                // Boarding failed
                self.grappled = false;
            },
        }
    }

    fn resolve_zone_combat(&mut self, zone: &mut ZoneControl, dt: f32) {
        let attacker_power = self.attacker_forces.combat_power() * self.attacker_morale;
        let defender_power = self.defender_forces.combat_power() * self.defender_morale;

        let ratio = attacker_power / (defender_power + 0.01);

        // Casualties based on power difference
        let attacker_casualties = (defender_power * 0.02 * dt) as u32;
        let defender_casualties = (attacker_power * 0.02 * dt) as u32;

        self.casualties.0 += attacker_casualties;
        self.casualties.1 += defender_casualties;

        // Morale damage
        self.attacker_morale -= attacker_casualties as f32 * 0.01;
        self.defender_morale -= defender_casualties as f32 * 0.01;

        // Zone control shifts
        if ratio > 2.0 && rand::random::<f32>() < 0.3 * dt {
            *zone = match zone {
                ZoneControl::Defender => ZoneControl::Contested,
                ZoneControl::Contested => ZoneControl::Attacker,
                ZoneControl::Attacker => ZoneControl::Attacker,
            };
        } else if ratio < 0.5 && rand::random::<f32>() < 0.3 * dt {
            *zone = match zone {
                ZoneControl::Attacker => ZoneControl::Contested,
                ZoneControl::Contested => ZoneControl::Defender,
                ZoneControl::Defender => ZoneControl::Defender,
            };
        }
    }
}

impl BoardingForce {
    pub fn combat_power(&self) -> f32 {
        let base = (self.marines as f32 * 1.5
            + self.sailors as f32 * 0.8
            + self.officers as f32 * 2.0);

        let equipment = (self.equipment.cutlasses as f32 * 0.1
            + self.equipment.pistols as f32 * 0.15
            + self.equipment.muskets as f32 * 0.2
            + self.equipment.pikes as f32 * 0.12
            + self.equipment.grenades as f32 * 0.5) / self.total() as f32;

        base * self.quality * (1.0 + equipment)
    }

    pub fn total(&self) -> u32 {
        self.marines + self.sailors + self.officers
    }
}
```

---

## Ship Damage & Repair

### Damage System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageState {
    // Hull
    pub hull_breaches: Vec<HullBreach>,
    pub flooding_rate: f32,
    pub flooding_level: f32,

    // Sails and rigging
    pub mast_damage: [f32; 3],     // Fore, main, mizzen
    pub sail_damage: f32,
    pub rigging_integrity: f32,

    // Systems
    pub rudder_damaged: bool,
    pub rudder_destroyed: bool,
    pub pumps_operational: u32,
    pub pumps_damaged: u32,

    // Fires
    pub active_fires: Vec<Fire>,
    pub powder_magazine_risk: f32,

    // Crew casualties
    pub crew_killed: u32,
    pub crew_wounded: u32,
}

#[derive(Debug, Clone)]
pub struct HullBreach {
    pub position: Vec3,
    pub size: BreachSize,
    pub below_waterline: bool,
    pub patched: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum BreachSize {
    Minor,          // Slow leak
    Moderate,       // Significant flooding
    Major,          // Rapid flooding
    Catastrophic,   // Sinking
}

impl BreachSize {
    pub fn flooding_rate(&self) -> f32 {
        match self {
            Self::Minor => 0.1,
            Self::Moderate => 0.5,
            Self::Major => 2.0,
            Self::Catastrophic => 10.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fire {
    pub position: Vec3,
    pub intensity: f32,     // 0.0 - 1.0
    pub spread_rate: f32,
    pub being_fought: bool,
}

impl DamageState {
    pub fn apply_hit(&mut self, damage: f32, damage_type: DamageType, position: Vec3) {
        match damage_type {
            DamageType::Hull => {
                // Create breach if damage is severe enough
                if damage > 30.0 {
                    let size = if damage > 80.0 {
                        BreachSize::Major
                    } else if damage > 50.0 {
                        BreachSize::Moderate
                    } else {
                        BreachSize::Minor
                    };

                    self.hull_breaches.push(HullBreach {
                        position,
                        size,
                        below_waterline: position.y < 1.0,
                        patched: false,
                    });
                }
            },

            DamageType::Sail => {
                self.sail_damage += damage * 0.01;
                self.rigging_integrity -= damage * 0.005;
            },

            DamageType::Mast => {
                // Damage random mast
                let mast = rand::random::<usize>() % 3;
                self.mast_damage[mast] += damage * 0.01;

                // Mast falls if too damaged
                if self.mast_damage[mast] >= 1.0 {
                    self.on_mast_falls(mast);
                }
            },

            DamageType::Crew => {
                // Crew casualties handled elsewhere
            },

            DamageType::Rudder => {
                if damage > 40.0 {
                    self.rudder_damaged = true;
                }
                if damage > 80.0 {
                    self.rudder_destroyed = true;
                }
            },
        }
    }

    pub fn update(&mut self, ship: &mut Ship, dt: f32) {
        // Update flooding
        self.update_flooding(ship, dt);

        // Update fires
        self.update_fires(ship, dt);

        // Check for catastrophic events
        self.check_catastrophic(ship);
    }

    fn update_flooding(&mut self, ship: &mut Ship, dt: f32) {
        // Calculate flooding rate from breaches
        let breach_rate: f32 = self.hull_breaches.iter()
            .filter(|b| b.below_waterline && !b.patched)
            .map(|b| b.size.flooding_rate())
            .sum();

        self.flooding_rate = breach_rate;

        // Pumps reduce flooding
        let pump_rate = self.pumps_operational as f32 * 0.3;
        let net_rate = breach_rate - pump_rate;

        self.flooding_level = (self.flooding_level + net_rate * dt).clamp(0.0, 1.0);

        // Flooding affects ship
        if self.flooding_level > 0.3 {
            ship.current_speed *= 1.0 - (self.flooding_level - 0.3);
        }

        if self.flooding_level >= 1.0 {
            ship.combat_state = CombatState::Sinking;
        }
    }

    fn update_fires(&mut self, ship: &mut Ship, dt: f32) {
        for fire in &mut self.active_fires {
            if fire.being_fought {
                fire.intensity -= 0.1 * dt;
            } else {
                fire.intensity += fire.spread_rate * dt;
            }

            // Damage from fire
            if fire.intensity > 0.5 {
                ship.hull_health -= fire.intensity * 0.5 * dt;

                // Risk to powder magazine
                if fire.position.distance(ship.powder_magazine_position()) < 5.0 {
                    self.powder_magazine_risk += fire.intensity * 0.1 * dt;
                }
            }
        }

        // Remove extinguished fires
        self.active_fires.retain(|f| f.intensity > 0.0);

        // Powder magazine explosion
        if self.powder_magazine_risk >= 1.0 {
            self.powder_magazine_explodes(ship);
        }
    }

    fn powder_magazine_explodes(&mut self, ship: &mut Ship) {
        // Catastrophic explosion
        ship.hull_health = 0.0;
        ship.combat_state = CombatState::Sinking;
        // All crew killed
        ship.crew.sailors = 0;
        ship.crew.gunners = 0;
        ship.crew.marines = 0;
    }

    fn on_mast_falls(&mut self, mast: usize) {
        // Mast falling causes additional damage and crew casualties
        self.sail_damage += 0.3;
        self.crew_killed += rand::random::<u32>() % 5 + 1;
    }
}
```

### Repair System

```rust
#[derive(Debug, Clone)]
pub struct RepairSystem {
    pub repair_crew: u32,
    pub repair_materials: RepairMaterials,
}

#[derive(Debug, Clone, Default)]
pub struct RepairMaterials {
    pub planking: u32,
    pub canvas: u32,
    pub rope: u32,
    pub tar: u32,
    pub nails: u32,
}

impl RepairSystem {
    pub fn repair_in_combat(&mut self, ship: &mut Ship, dt: f32, priority: RepairPriority) {
        // Limited repairs possible during combat
        let repair_rate = self.repair_crew as f32 * 0.1;

        match priority {
            RepairPriority::Flooding => {
                self.patch_breaches(&mut ship.damage_state, repair_rate, dt);
            },
            RepairPriority::Fires => {
                self.fight_fires(&mut ship.damage_state, repair_rate, dt);
            },
            RepairPriority::Sails => {
                self.repair_sails(&mut ship.damage_state, repair_rate, dt);
            },
            RepairPriority::Guns => {
                self.repair_guns(&mut ship.cannons, repair_rate, dt);
            },
        }
    }

    fn patch_breaches(&mut self, damage: &mut DamageState, rate: f32, dt: f32) {
        // Can only patch minor/moderate breaches in combat
        for breach in &mut damage.hull_breaches {
            if breach.patched { continue; }

            if matches!(breach.size, BreachSize::Minor | BreachSize::Moderate) {
                if self.repair_materials.planking > 0 && self.repair_materials.tar > 0 {
                    // Progress toward patching
                    if rand::random::<f32>() < rate * 0.01 * dt {
                        breach.patched = true;
                        self.repair_materials.planking -= 1;
                        self.repair_materials.tar -= 1;
                    }
                }
            }
        }
    }

    fn fight_fires(&mut self, damage: &mut DamageState, rate: f32, dt: f32) {
        // Assign crew to fight fires
        let crew_per_fire = (rate / damage.active_fires.len().max(1) as f32) as u32;

        for fire in &mut damage.active_fires {
            if crew_per_fire >= 2 {
                fire.being_fought = true;
            }
        }
    }

    fn repair_sails(&mut self, damage: &mut DamageState, rate: f32, dt: f32) {
        if self.repair_materials.canvas > 0 && self.repair_materials.rope > 0 {
            let repair_amount = rate * 0.001 * dt;
            damage.sail_damage = (damage.sail_damage - repair_amount).max(0.0);

            // Consume materials slowly
            if rand::random::<f32>() < 0.01 * dt {
                self.repair_materials.canvas -= 1;
                self.repair_materials.rope -= 1;
            }
        }
    }

    pub fn drydock_repairs(&mut self, ship: &mut Ship, days: u32) {
        // Full repairs at port
        // Much faster and can repair major damage

        ship.hull_health = 1.0;
        ship.sail_health = 1.0;
        ship.damage_state.hull_breaches.clear();
        ship.damage_state.flooding_level = 0.0;
        ship.damage_state.mast_damage = [0.0; 3];
        ship.damage_state.sail_damage = 0.0;
        ship.damage_state.rigging_integrity = 1.0;
        ship.damage_state.rudder_damaged = false;
        ship.damage_state.rudder_destroyed = false;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RepairPriority {
    Flooding,
    Fires,
    Sails,
    Guns,
}
```

---

## Crew Management

### Crew Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewComplement {
    // Core crew
    pub captain: Option<CrewMember>,
    pub officers: Vec<CrewMember>,
    pub sailors: u32,
    pub gunners: u32,
    pub marines: u32,

    // Specialists
    pub surgeon: Option<CrewMember>,
    pub carpenter: Option<CrewMember>,
    pub navigator: Option<CrewMember>,
    pub quartermaster: Option<CrewMember>,

    // Status
    pub total: u32,
    pub wounded: u32,
    pub sick: u32,
    pub dead: u32,

    // Morale factors
    pub pay_owed: u32,
    pub days_at_sea: u32,
    pub last_port_visit: f64,
    pub rations_level: RationsLevel,
    pub discipline: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewMember {
    pub id: u64,
    pub name: String,
    pub role: CrewRole,
    pub skills: CrewSkills,
    pub health: f32,
    pub loyalty: f32,
    pub experience: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum CrewRole {
    Captain,
    FirstMate,
    Bosun,
    MasterGunner,
    Surgeon,
    Carpenter,
    Navigator,
    Quartermaster,
    Sailor,
    Gunner,
    Marine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewSkills {
    pub sailing: f32,
    pub gunnery: f32,
    pub combat: f32,
    pub navigation: f32,
    pub repair: f32,
    pub medicine: f32,
}

impl CrewComplement {
    pub fn gunnery_skill(&self) -> f32 {
        let base = 0.5;
        let gunner_bonus = (self.gunners as f32 / (self.gunners + self.sailors) as f32) * 0.3;
        let master_bonus = if self.has_master_gunner() { 0.2 } else { 0.0 };

        (base + gunner_bonus + master_bonus).min(1.0)
    }

    pub fn sailing_skill(&self) -> f32 {
        let base = 0.4;
        let crew_ratio = self.sailors as f32 / self.total as f32;
        let officer_bonus = self.officers.len() as f32 * 0.05;
        let nav_bonus = if self.navigator.is_some() { 0.15 } else { 0.0 };

        (base + crew_ratio * 0.3 + officer_bonus + nav_bonus).min(1.0)
    }

    pub fn average_skill(&self) -> f32 {
        (self.gunnery_skill() + self.sailing_skill()) / 2.0
    }

    pub fn morale(&self) -> f32 {
        let mut morale = 0.5;

        // Pay
        if self.pay_owed > 0 {
            morale -= (self.pay_owed as f32 / 1000.0).min(0.3);
        }

        // Time at sea
        let sea_fatigue = (self.days_at_sea as f32 / 60.0).min(0.2);
        morale -= sea_fatigue;

        // Rations
        morale += match self.rations_level {
            RationsLevel::Full => 0.1,
            RationsLevel::Normal => 0.0,
            RationsLevel::Reduced => -0.1,
            RationsLevel::Minimal => -0.25,
            RationsLevel::None => -0.5,
        };

        // Casualties affect morale
        let casualty_ratio = (self.wounded + self.dead) as f32 / self.total as f32;
        morale -= casualty_ratio * 0.5;

        // Discipline
        morale += (self.discipline - 0.5) * 0.2;

        morale.clamp(0.0, 1.0)
    }

    fn has_master_gunner(&self) -> bool {
        self.officers.iter().any(|o| o.role == CrewRole::MasterGunner)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RationsLevel {
    Full,       // Extra rations
    Normal,     // Standard
    Reduced,    // 2/3 rations
    Minimal,    // 1/3 rations
    None,       // Starvation
}
```

---

## Weather & Sea Conditions

### Sea State

```rust
#[derive(Debug, Clone)]
pub struct SeaState {
    pub wave_height: f32,       // meters
    pub wave_period: f32,       // seconds
    pub wave_direction: f32,    // degrees
    pub swell_height: f32,
    pub swell_direction: f32,
    pub current_speed: f32,
    pub current_direction: f32,
    pub visibility: f32,        // meters
}

impl SeaState {
    pub fn from_wind(wind: &WindState) -> Self {
        // Sea state develops from wind
        let wave_height = wind.speed * 0.1;  // Simplified
        let wave_period = 0.8 * (wave_height.sqrt() * 2.0 + 3.0);

        Self {
            wave_height,
            wave_period,
            wave_direction: wind.direction,
            swell_height: wave_height * 0.5,
            swell_direction: wind.direction + 20.0,
            current_speed: wind.speed * 0.02,
            current_direction: wind.direction + 45.0,
            visibility: 10000.0,  // 10km clear
        }
    }

    pub fn beaufort_scale(&self) -> u8 {
        match self.wave_height {
            h if h < 0.1 => 0,   // Calm
            h if h < 0.3 => 1,   // Light air
            h if h < 0.6 => 2,   // Light breeze
            h if h < 1.0 => 3,   // Gentle breeze
            h if h < 1.5 => 4,   // Moderate breeze
            h if h < 2.5 => 5,   // Fresh breeze
            h if h < 4.0 => 6,   // Strong breeze
            h if h < 5.5 => 7,   // High wind
            h if h < 7.5 => 8,   // Gale
            h if h < 10.0 => 9,  // Strong gale
            h if h < 12.5 => 10, // Storm
            h if h < 16.0 => 11, // Violent storm
            _ => 12,             // Hurricane
        }
    }

    pub fn sailing_modifier(&self) -> f32 {
        // Rough seas slow ships and make sailing harder
        match self.beaufort_scale() {
            0..=3 => 1.0,
            4 => 0.95,
            5 => 0.9,
            6 => 0.8,
            7 => 0.65,
            8 => 0.5,
            9 => 0.3,
            10 => 0.15,
            _ => 0.05,
        }
    }

    pub fn gunnery_modifier(&self) -> f32 {
        // Rolling seas make aiming difficult
        match self.beaufort_scale() {
            0..=2 => 1.0,
            3 => 0.95,
            4 => 0.85,
            5 => 0.7,
            6 => 0.5,
            7 => 0.3,
            8 => 0.15,
            _ => 0.05,
        }
    }

    pub fn capsize_risk(&self, ship: &Ship) -> f32 {
        let def = get_ship_def(ship.class);

        // Smaller ships at higher risk
        let size_factor = def.displacement / 500.0;

        // Wave height vs ship size
        let wave_factor = self.wave_height / (def.beam * 0.5);

        // Sail state - full sails in storm = dangerous
        let sail_factor = match ship.sail_state {
            SailState::FullSails if self.beaufort_scale() > 6 => 2.0,
            SailState::PlainSails if self.beaufort_scale() > 7 => 1.5,
            _ => 1.0,
        };

        // Damage increases risk
        let damage_factor = 1.0 + (1.0 - ship.hull_health) * 0.5;

        let risk = (wave_factor / size_factor) * sail_factor * damage_factor;
        risk.clamp(0.0, 1.0)
    }
}
```

### Storm Sailing

```rust
pub struct StormSailingSystem;

impl StormSailingSystem {
    pub fn update_in_storm(
        ship: &mut Ship,
        storm: &Storm,
        sea: &SeaState,
        dt: f32,
    ) {
        // Forced sail reduction
        if sea.beaufort_scale() >= 8 && ship.sail_state != SailState::BattleSails {
            // Must reduce sail or risk mast loss
            if rand::random::<f32>() < 0.1 * dt {
                ship.damage_state.mast_damage[1] += 0.1;  // Damage main mast
            }
        }

        // Wave impacts
        if sea.wave_height > 5.0 {
            // Waves breaking over deck
            ship.crew.wounded += (rand::random::<u32>() % 2);
            ship.flooding_level += 0.01 * dt;
        }

        // Lightning strikes
        if storm.is_supernatural && rand::random::<f32>() < storm.lightning_rate * dt {
            Self::lightning_strike(ship);
        }

        // Capsize check
        let capsize_risk = sea.capsize_risk(ship);
        if rand::random::<f32>() < capsize_risk * dt * 0.01 {
            ship.combat_state = CombatState::Sinking;
        }

        // Crew morale
        ship.morale -= 0.01 * dt * (sea.beaufort_scale() as f32 / 12.0);
    }

    fn lightning_strike(ship: &mut Ship) {
        // Lightning hits mast or starts fire
        if rand::random::<bool>() {
            let mast = rand::random::<usize>() % 3;
            ship.damage_state.mast_damage[mast] += 0.3;
        } else {
            ship.damage_state.active_fires.push(Fire {
                position: Vec3::new(0.0, 5.0, 0.0),
                intensity: 0.3,
                spread_rate: 0.1,
                being_fought: false,
            });
        }

        // Crew casualties
        ship.crew.wounded += rand::random::<u32>() % 3;
    }
}
```

---

## Naval AI

### Ship AI

```rust
#[derive(Debug, Clone)]
pub struct ShipAI {
    pub ship_id: ShipId,
    pub behavior: AIBehavior,
    pub state: AIState,
    pub target: Option<ShipId>,

    // Tactics
    pub engagement_distance: f32,
    pub preferred_side: BroadsideSide,
    pub aggression: f32,
    pub skill_level: f32,

    // Memory
    pub known_enemies: Vec<ShipId>,
    pub threat_assessment: HashMap<ShipId, f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum AIBehavior {
    Patrol,             // Patrol area
    Escort,             // Protect another ship
    Hunt,               // Seek and destroy enemies
    Merchant,           // Travel route, avoid combat
    Pirate,             // Attack weak targets
    Flee,               // Run away
    Defend,             // Defend position
}

#[derive(Debug, Clone, Copy)]
pub enum AIState {
    Idle,
    Patrolling,
    Pursuing,
    Engaging,
    Maneuvering,
    Broadside,
    Boarding,
    Fleeing,
    Damaged,
    Surrendering,
}

impl ShipAI {
    pub fn update(
        &mut self,
        ship: &mut Ship,
        nearby_ships: &[&Ship],
        wind: &WindState,
        dt: f32,
    ) -> Vec<AICommand> {
        let mut commands = vec![];

        // Update threat assessment
        self.assess_threats(ship, nearby_ships);

        // State machine
        self.state = match (&self.behavior, &self.state) {
            (AIBehavior::Hunt, AIState::Idle) => {
                if let Some(target) = self.find_target(nearby_ships) {
                    self.target = Some(target);
                    AIState::Pursuing
                } else {
                    AIState::Patrolling
                }
            },

            (_, AIState::Pursuing) => {
                if let Some(target_id) = self.target {
                    if let Some(target) = nearby_ships.iter().find(|s| s.id == target_id) {
                        let distance = ship.position.distance(target.position);

                        if distance < self.engagement_distance {
                            AIState::Engaging
                        } else {
                            // Generate pursuit commands
                            commands.push(self.pursue_target(ship, target, wind));
                            AIState::Pursuing
                        }
                    } else {
                        self.target = None;
                        AIState::Idle
                    }
                } else {
                    AIState::Idle
                }
            },

            (_, AIState::Engaging) => {
                if let Some(target_id) = self.target {
                    if let Some(target) = nearby_ships.iter().find(|s| s.id == target_id) {
                        // Combat maneuvering
                        commands.extend(self.combat_maneuvers(ship, target, wind));

                        // Firing decisions
                        if let Some(fire_cmd) = self.decide_fire(ship, target) {
                            commands.push(fire_cmd);
                        }

                        // Boarding check
                        if self.should_board(ship, target) {
                            commands.push(AICommand::InitiateBoarding(target.id));
                            AIState::Boarding
                        } else {
                            AIState::Engaging
                        }
                    } else {
                        AIState::Idle
                    }
                } else {
                    AIState::Idle
                }
            },

            (_, AIState::Damaged) => {
                // Attempt to disengage
                if ship.hull_health < 0.3 || ship.damage_state.flooding_level > 0.5 {
                    commands.push(AICommand::SetSails(SailState::FullSails));
                    commands.push(AICommand::TurnAway);
                    AIState::Fleeing
                } else {
                    AIState::Engaging
                }
            },

            (_, AIState::Fleeing) => {
                commands.push(self.flee_command(ship, nearby_ships, wind));
                AIState::Fleeing
            },

            _ => self.state,
        };

        // Check for surrender conditions
        if self.should_surrender(ship) {
            self.state = AIState::Surrendering;
            commands.push(AICommand::Surrender);
        }

        commands
    }

    fn combat_maneuvers(&self, ship: &Ship, target: &Ship, wind: &WindState) -> Vec<AICommand> {
        let mut commands = vec![];

        let to_target = target.position - ship.position;
        let distance = to_target.length();
        let bearing = to_target.x.atan2(to_target.z).to_degrees();
        let relative_bearing = normalize_angle(bearing - ship.heading);

        // Try to get broadside
        let optimal_angle = if self.preferred_side == BroadsideSide::Port {
            -90.0
        } else {
            90.0
        };

        let angle_diff = relative_bearing - optimal_angle;

        if angle_diff.abs() > 10.0 {
            commands.push(AICommand::Rudder(if angle_diff > 0.0 { -0.5 } else { 0.5 }));
        }

        // Maintain optimal distance
        if distance > self.engagement_distance * 0.8 {
            commands.push(AICommand::SetSails(SailState::PlainSails));
        } else if distance < self.engagement_distance * 0.4 {
            commands.push(AICommand::SetSails(SailState::BattleSails));
        }

        commands
    }

    fn decide_fire(&self, ship: &Ship, target: &Ship) -> Option<AICommand> {
        let distance = ship.position.distance(target.position);

        // Check if target is in broadside arc
        let to_target = target.position - ship.position;
        let bearing = to_target.x.atan2(to_target.z).to_degrees();
        let relative = normalize_angle(bearing - ship.heading);

        let (in_port_arc, in_starboard_arc) = (
            relative > 60.0 && relative < 120.0,
            relative < -60.0 && relative > -120.0,
        );

        if in_port_arc && ship.has_loaded_cannons(BroadsideSide::Port) {
            Some(AICommand::FireBroadside(BroadsideSide::Port, AimPoint::Hull))
        } else if in_starboard_arc && ship.has_loaded_cannons(BroadsideSide::Starboard) {
            Some(AICommand::FireBroadside(BroadsideSide::Starboard, AimPoint::Hull))
        } else {
            None
        }
    }

    fn should_board(&self, ship: &Ship, target: &Ship) -> bool {
        let distance = ship.position.distance(target.position);

        distance < 50.0
            && ship.crew.marines + ship.crew.sailors > target.crew.marines + target.crew.sailors
            && self.aggression > 0.7
    }

    fn should_surrender(&self, ship: &Ship) -> bool {
        ship.hull_health < 0.15
            || ship.damage_state.flooding_level > 0.7
            || ship.morale < 0.1
            || (ship.crew.sailors + ship.crew.marines) < 10
    }
}

#[derive(Debug, Clone)]
pub enum AICommand {
    SetSails(SailState),
    Rudder(f32),
    TurnAway,
    FireBroadside(BroadsideSide, AimPoint),
    FireChasers(AimPoint),
    InitiateBoarding(ShipId),
    Surrender,
    RepairPriority(RepairPriority),
}
```

---

## Piracy & Privateering

### Prize System

```rust
#[derive(Debug, Clone)]
pub struct PrizeSystem {
    pub captured_ships: Vec<CapturedShip>,
    pub total_value: u32,
    pub contraband: Vec<Contraband>,
}

#[derive(Debug, Clone)]
pub struct CapturedShip {
    pub ship: Ship,
    pub captured_from: Faction,
    pub capture_date: f64,
    pub prize_value: u32,
    pub prize_crew_assigned: u32,
}

#[derive(Debug, Clone)]
pub struct Contraband {
    pub item_type: ContrabanType,
    pub quantity: u32,
    pub origin: Faction,
    pub legal_in: Vec<Faction>,
    pub value: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum ContrabandType {
    Gold,
    Silver,
    Gems,
    Tobacco,
    Sugar,
    Slaves,     // Historical accuracy - can refuse to engage
    Weapons,
    Ammunition,
    SpanishDocuments,
}

impl PrizeSystem {
    pub fn capture_ship(&mut self, ship: Ship, crew_available: u32) -> CaptureResult {
        let def = get_ship_def(ship.class);

        // Need minimum prize crew
        let min_crew = def.min_crew;
        if crew_available < min_crew {
            return CaptureResult::InsufficientCrew;
        }

        let prize_value = self.calculate_prize_value(&ship);

        self.captured_ships.push(CapturedShip {
            ship,
            captured_from: Faction::Spain,  // Based on context
            capture_date: get_game_time(),
            prize_value,
            prize_crew_assigned: min_crew,
        });

        CaptureResult::Success {
            value: prize_value,
            crew_needed: min_crew,
        }
    }

    fn calculate_prize_value(&self, ship: &Ship) -> u32 {
        let def = get_ship_def(ship.class);

        let hull_value = (def.purchase_price as f32 * ship.hull_health) as u32;
        let cargo_value: u32 = ship.cargo.iter().map(|c| c.value).sum();
        let cannon_value: u32 = ship.cannons.iter()
            .map(|c| get_cannon_def(c.cannon_type).cannon_weight * 2)
            .sum();

        hull_value + cargo_value + cannon_value
    }
}

pub enum CaptureResult {
    Success { value: u32, crew_needed: u32 },
    InsufficientCrew,
    ShipSunk,
}
```

---

## Port & Docking

### Port System

```rust
#[derive(Debug, Clone)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    pub position: Vec3,
    pub faction: Faction,
    pub size: PortSize,

    // Facilities
    pub shipyard: Option<Shipyard>,
    pub market: Market,
    pub tavern: Option<Tavern>,
    pub governor: Option<Governor>,

    // State
    pub ships_in_port: Vec<ShipId>,
    pub dock_capacity: u32,
    pub trade_goods: Vec<TradeGood>,
    pub prices: HashMap<GoodType, u32>,
}

#[derive(Debug, Clone)]
pub struct Shipyard {
    pub available_ships: Vec<ShipClass>,
    pub can_repair: bool,
    pub can_upgrade: bool,
    pub repair_rate: f32,
    pub repair_cost_multiplier: f32,
}

#[derive(Debug, Clone)]
pub struct Tavern {
    pub crew_available: u32,
    pub crew_quality: f32,
    pub rumors: Vec<Rumor>,
}

impl Port {
    pub fn dock_ship(&mut self, ship: &mut Ship) -> DockResult {
        if self.ships_in_port.len() >= self.dock_capacity as usize {
            return DockResult::NoSpace;
        }

        // Check if allowed to dock (faction relations)
        if !self.can_dock(ship.owner) {
            return DockResult::Denied;
        }

        // Docking fee
        let fee = self.calculate_dock_fee(ship);

        self.ships_in_port.push(ship.id);

        DockResult::Success { fee }
    }

    pub fn hire_crew(&mut self, count: u32) -> Vec<CrewMember> {
        let tavern = match &mut self.tavern {
            Some(t) => t,
            None => return vec![],
        };

        let hired = count.min(tavern.crew_available);
        tavern.crew_available -= hired;

        (0..hired).map(|_| {
            CrewMember {
                id: rand::random(),
                name: generate_crew_name(),
                role: CrewRole::Sailor,
                skills: CrewSkills {
                    sailing: tavern.crew_quality * rand::random::<f32>(),
                    gunnery: tavern.crew_quality * rand::random::<f32>() * 0.5,
                    combat: tavern.crew_quality * rand::random::<f32>() * 0.7,
                    navigation: 0.1,
                    repair: 0.2,
                    medicine: 0.0,
                },
                health: 1.0,
                loyalty: 0.5,
                experience: 0,
            }
        }).collect()
    }
}
```

---

## Data Structures

### Core Types

```rust
// ships/mod.rs

pub mod types;
pub mod sailing;
pub mod combat;
pub mod damage;
pub mod crew;
pub mod ai;
pub mod ports;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShipId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Faction {
    English,
    Spanish,
    French,
    Dutch,
    Pirate,
    Native,
    Player,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BroadsideSide {
    Port,
    Starboard,
}

#[derive(Debug, Clone, Copy)]
pub enum DamageType {
    Hull,
    Sail,
    Mast,
    Crew,
    Rudder,
}
```

---

## Implementation Priority

### Phase 1: Ship Foundation
- [ ] Ship class definitions
- [ ] Basic sailing mechanics
- [ ] Wind system

### Phase 2: Movement
- [ ] Speed calculation
- [ ] Turning/maneuvering
- [ ] Points of sail

### Phase 3: Combat Basics
- [ ] Cannon definitions
- [ ] Firing mechanics
- [ ] Damage system

### Phase 4: Advanced Combat
- [ ] Targeting system
- [ ] Ammo types
- [ ] Broadside volleys

### Phase 5: Boarding
- [ ] Grappling mechanics
- [ ] Boarding combat
- [ ] Capture system

### Phase 6: Crew
- [ ] Crew management
- [ ] Skills and morale
- [ ] Casualties

### Phase 7: AI
- [ ] Basic ship AI
- [ ] Combat maneuvers
- [ ] Fleet coordination

### Phase 8: Ports
- [ ] Docking system
- [ ] Repairs
- [ ] Trading

### Phase 9: Weather
- [ ] Storm sailing
- [ ] Sea state effects
- [ ] Hurricane survival

---

*The seas around Roanoke are treacherous, patrolled by Spanish galleons and plagued by storms. Master the art of naval warfare, and the New World's riches are yours for the taking.*
