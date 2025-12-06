//! Naval Combat and Ship System
//!
//! A historically-grounded Age of Sail naval system modeling:
//! - Ship types from canoes to galleons
//! - Realistic sailing mechanics (wind, point of sail)
//! - Cannon warfare and damage systems
//! - Boarding combat
//! - Crew management
//!
//! Set in the colonial Chesapeake/Carolina region circa 1580s-1600s.

pub mod ships;
pub mod sailing;
pub mod combat;
pub mod crew;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// All ship classes available in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShipClass {
    // Small craft
    Canoe,          // 1-3 crew, no guns, native craft
    Rowboat,        // 2-4 crew, no guns
    Pinnace,        // 8-20 crew, 2-4 guns, versatile small ship

    // Medium vessels
    Shallop,        // 15-30 crew, 4-8 guns, coastal trader
    Sloop,          // 20-40 crew, 8-12 guns, fast raider
    Brigantine,     // 40-80 crew, 12-16 guns, pirate favorite

    // Large vessels
    Merchantman,    // 60-100 crew, 16-24 guns, cargo hauler
    Frigate,        // 100-200 crew, 24-36 guns, warship
    Galleon,        // 150-400 crew, 40-60 guns, treasure ship
    ManOfWar,       // 200-500 crew, 60-100 guns, ship of the line
}

impl ShipClass {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Canoe => "Canoe",
            Self::Rowboat => "Rowboat",
            Self::Pinnace => "Pinnace",
            Self::Shallop => "Shallop",
            Self::Sloop => "Sloop",
            Self::Brigantine => "Brigantine",
            Self::Merchantman => "Merchantman",
            Self::Frigate => "Frigate",
            Self::Galleon => "Galleon",
            Self::ManOfWar => "Man-of-War",
        }
    }

    pub fn base_stats(&self) -> ShipStats {
        match self {
            Self::Canoe => ShipStats {
                hull_hp: 20,
                max_crew: 3,
                min_crew: 1,
                base_speed: 4.0,
                maneuverability: 0.95,
                cargo_capacity: 50,
                gun_ports: 0,
                max_gun_size: GunSize::None,
                draft: 0.3,
            },
            Self::Rowboat => ShipStats {
                hull_hp: 30,
                max_crew: 6,
                min_crew: 2,
                base_speed: 3.0,
                maneuverability: 0.9,
                cargo_capacity: 100,
                gun_ports: 0,
                max_gun_size: GunSize::Swivel,
                draft: 0.5,
            },
            Self::Pinnace => ShipStats {
                hull_hp: 80,
                max_crew: 25,
                min_crew: 6,
                base_speed: 8.0,
                maneuverability: 0.85,
                cargo_capacity: 500,
                gun_ports: 4,
                max_gun_size: GunSize::Saker,
                draft: 1.5,
            },
            Self::Shallop => ShipStats {
                hull_hp: 120,
                max_crew: 35,
                min_crew: 8,
                base_speed: 7.0,
                maneuverability: 0.75,
                cargo_capacity: 1000,
                gun_ports: 8,
                max_gun_size: GunSize::Minion,
                draft: 2.0,
            },
            Self::Sloop => ShipStats {
                hull_hp: 150,
                max_crew: 50,
                min_crew: 12,
                base_speed: 10.0,
                maneuverability: 0.8,
                cargo_capacity: 800,
                gun_ports: 12,
                max_gun_size: GunSize::Culverin,
                draft: 2.5,
            },
            Self::Brigantine => ShipStats {
                hull_hp: 200,
                max_crew: 100,
                min_crew: 20,
                base_speed: 9.0,
                maneuverability: 0.7,
                cargo_capacity: 1500,
                gun_ports: 16,
                max_gun_size: GunSize::Culverin,
                draft: 3.0,
            },
            Self::Merchantman => ShipStats {
                hull_hp: 300,
                max_crew: 120,
                min_crew: 30,
                base_speed: 6.0,
                maneuverability: 0.5,
                cargo_capacity: 5000,
                gun_ports: 24,
                max_gun_size: GunSize::DemiCannon,
                draft: 4.0,
            },
            Self::Frigate => ShipStats {
                hull_hp: 400,
                max_crew: 250,
                min_crew: 60,
                base_speed: 8.0,
                maneuverability: 0.6,
                cargo_capacity: 2000,
                gun_ports: 36,
                max_gun_size: GunSize::DemiCannon,
                draft: 4.5,
            },
            Self::Galleon => ShipStats {
                hull_hp: 600,
                max_crew: 400,
                min_crew: 100,
                base_speed: 5.0,
                maneuverability: 0.4,
                cargo_capacity: 8000,
                gun_ports: 60,
                max_gun_size: GunSize::Cannon,
                draft: 6.0,
            },
            Self::ManOfWar => ShipStats {
                hull_hp: 800,
                max_crew: 600,
                min_crew: 150,
                base_speed: 4.5,
                maneuverability: 0.35,
                cargo_capacity: 3000,
                gun_ports: 100,
                max_gun_size: GunSize::Cannon,
                draft: 7.0,
            },
        }
    }

    /// Get purchase price in pounds sterling
    pub fn base_price(&self) -> u32 {
        match self {
            Self::Canoe => 5,
            Self::Rowboat => 20,
            Self::Pinnace => 200,
            Self::Shallop => 400,
            Self::Sloop => 800,
            Self::Brigantine => 1500,
            Self::Merchantman => 3000,
            Self::Frigate => 8000,
            Self::Galleon => 20000,
            Self::ManOfWar => 50000,
        }
    }

    /// Maintenance cost per month
    pub fn monthly_upkeep(&self) -> u32 {
        match self {
            Self::Canoe => 0,
            Self::Rowboat => 1,
            Self::Pinnace => 10,
            Self::Shallop => 25,
            Self::Sloop => 50,
            Self::Brigantine => 100,
            Self::Merchantman => 150,
            Self::Frigate => 400,
            Self::Galleon => 800,
            Self::ManOfWar => 1500,
        }
    }

    /// Historical context/description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Canoe => "A native dugout canoe, light and maneuverable in shallow waters.",
            Self::Rowboat => "A simple oar-powered boat for short coastal journeys.",
            Self::Pinnace => "A small, versatile sailing vessel used for exploration and coastal work.",
            Self::Shallop => "A light boat used for fishing and coastal trade in the colonies.",
            Self::Sloop => "A fast, single-masted vessel favored by pirates and smugglers.",
            Self::Brigantine => "A two-masted vessel combining square and fore-and-aft sails.",
            Self::Merchantman => "A large cargo vessel designed for transatlantic trade.",
            Self::Frigate => "A fast warship used for escort and patrol duties.",
            Self::Galleon => "A large, multi-decked sailing ship used for war and treasure transport.",
            Self::ManOfWar => "The most powerful warship of the era, bristling with cannon.",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShipStats {
    pub hull_hp: u32,
    pub max_crew: u32,
    pub min_crew: u32,
    pub base_speed: f32,       // Knots at optimal conditions
    pub maneuverability: f32,  // 0-1, turning rate
    pub cargo_capacity: u32,   // Weight units
    pub gun_ports: u32,        // Maximum guns
    pub max_gun_size: GunSize,
    pub draft: f32,            // Depth in meters (affects shallow water)
}

/// Gun sizes from colonial era
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GunSize {
    None,
    Swivel,       // 0.5-1 lb shot, anti-personnel
    Falconet,     // 2 lb shot
    Saker,        // 5 lb shot
    Minion,       // 8 lb shot
    Culverin,     // 18 lb shot
    DemiCannon,  // 24 lb shot
    Cannon,       // 42 lb shot
}

impl GunSize {
    pub fn weight(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::Swivel => 100,
            Self::Falconet => 200,
            Self::Saker => 800,
            Self::Minion => 1200,
            Self::Culverin => 2500,
            Self::DemiCannon => 3500,
            Self::Cannon => 5000,
        }
    }

    pub fn crew_required(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::Swivel => 1,
            Self::Falconet => 2,
            Self::Saker => 3,
            Self::Minion => 4,
            Self::Culverin => 6,
            Self::DemiCannon => 8,
            Self::Cannon => 10,
        }
    }

    pub fn reload_time(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Swivel => 5.0,
            Self::Falconet => 10.0,
            Self::Saker => 20.0,
            Self::Minion => 30.0,
            Self::Culverin => 45.0,
            Self::DemiCannon => 60.0,
            Self::Cannon => 90.0,
        }
    }

    pub fn base_damage(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::Swivel => 5,
            Self::Falconet => 10,
            Self::Saker => 20,
            Self::Minion => 30,
            Self::Culverin => 50,
            Self::DemiCannon => 70,
            Self::Cannon => 100,
        }
    }

    pub fn range(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Swivel => 100.0,
            Self::Falconet => 200.0,
            Self::Saker => 400.0,
            Self::Minion => 500.0,
            Self::Culverin => 800.0,
            Self::DemiCannon => 600.0, // Heavier, shorter range
            Self::Cannon => 500.0,
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::Swivel => 10,
            Self::Falconet => 25,
            Self::Saker => 80,
            Self::Minion => 120,
            Self::Culverin => 300,
            Self::DemiCannon => 500,
            Self::Cannon => 800,
        }
    }
}

/// Ammunition types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AmmoType {
    RoundShot,    // Hull damage
    ChainShot,    // Rigging/sail damage
    Grapeshot,    // Crew damage
    BarShot,      // Mast damage
    HotShot,      // Fire damage
}

impl AmmoType {
    pub fn hull_damage_modifier(&self) -> f32 {
        match self {
            Self::RoundShot => 1.0,
            Self::ChainShot => 0.2,
            Self::Grapeshot => 0.1,
            Self::BarShot => 0.3,
            Self::HotShot => 0.8,
        }
    }

    pub fn rigging_damage_modifier(&self) -> f32 {
        match self {
            Self::RoundShot => 0.2,
            Self::ChainShot => 1.5,
            Self::Grapeshot => 0.1,
            Self::BarShot => 1.2,
            Self::HotShot => 0.5,
        }
    }

    pub fn crew_damage_modifier(&self) -> f32 {
        match self {
            Self::RoundShot => 0.3,
            Self::ChainShot => 0.4,
            Self::Grapeshot => 2.0,
            Self::BarShot => 0.2,
            Self::HotShot => 0.3,
        }
    }

    pub fn fire_chance(&self) -> f32 {
        match self {
            Self::RoundShot => 0.01,
            Self::ChainShot => 0.0,
            Self::Grapeshot => 0.0,
            Self::BarShot => 0.0,
            Self::HotShot => 0.25,
        }
    }

    pub fn cost(&self) -> u32 {
        match self {
            Self::RoundShot => 1,
            Self::ChainShot => 3,
            Self::Grapeshot => 2,
            Self::BarShot => 4,
            Self::HotShot => 5,
        }
    }
}

/// Individual ship instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub id: u64,
    pub name: String,
    pub class: ShipClass,

    // Current state
    pub hull_hp: u32,
    pub max_hull_hp: u32,
    pub sail_hp: u32,           // Affects speed
    pub max_sail_hp: u32,

    // Position and movement
    pub position: [f32; 2],     // World coordinates
    pub heading: f32,           // Radians, 0 = North
    pub current_speed: f32,     // Current knots
    pub anchor_down: bool,

    // Crew
    pub crew_count: u32,
    pub crew_morale: f32,       // 0-100
    pub crew_experience: f32,   // 0-1

    // Armament
    pub guns: Vec<ShipGun>,
    pub ammo_stores: HashMap<AmmoType, u32>,

    // Cargo
    pub cargo: Vec<CargoItem>,
    pub current_cargo_weight: u32,

    // Damage states
    pub is_on_fire: bool,
    pub fire_intensity: f32,
    pub flooding_rate: f32,
    pub mast_damage: f32,       // 0-1, affects sail HP

    // Flags
    pub faction: Faction,
    pub is_player_ship: bool,
}

impl Ship {
    pub fn new(id: u64, name: &str, class: ShipClass, faction: Faction) -> Self {
        let stats = class.base_stats();

        Self {
            id,
            name: name.to_string(),
            class,
            hull_hp: stats.hull_hp,
            max_hull_hp: stats.hull_hp,
            sail_hp: 100,
            max_sail_hp: 100,
            position: [0.0, 0.0],
            heading: 0.0,
            current_speed: 0.0,
            anchor_down: true,
            crew_count: stats.min_crew,
            crew_morale: 70.0,
            crew_experience: 0.3,
            guns: Vec::new(),
            ammo_stores: HashMap::new(),
            cargo: Vec::new(),
            current_cargo_weight: 0,
            is_on_fire: false,
            fire_intensity: 0.0,
            flooding_rate: 0.0,
            mast_damage: 0.0,
            faction,
            is_player_ship: false,
        }
    }

    /// Get effective speed based on damage
    pub fn effective_speed(&self, wind_modifier: f32) -> f32 {
        let stats = self.class.base_stats();
        let sail_factor = self.sail_hp as f32 / self.max_sail_hp as f32;
        let crew_factor = (self.crew_count as f32 / stats.min_crew as f32).min(1.0);
        let mast_factor = 1.0 - self.mast_damage;

        stats.base_speed * sail_factor * crew_factor * mast_factor * wind_modifier
    }

    /// Get effective maneuverability
    pub fn effective_maneuverability(&self) -> f32 {
        let stats = self.class.base_stats();
        let crew_factor = (self.crew_count as f32 / stats.min_crew as f32).min(1.0);
        let sail_factor = self.sail_hp as f32 / self.max_sail_hp as f32;

        stats.maneuverability * crew_factor.sqrt() * sail_factor.sqrt()
    }

    /// Check if ship can enter shallow water
    pub fn can_enter_shallows(&self, depth: f32) -> bool {
        self.class.base_stats().draft < depth
    }

    /// Get available gun count
    pub fn ready_guns(&self) -> u32 {
        self.guns.iter().filter(|g| g.is_loaded && g.ready_to_fire()).count() as u32
    }

    /// Apply damage to the ship
    pub fn take_damage(&mut self, damage: u32, ammo: AmmoType) {
        // Hull damage
        let hull_dmg = (damage as f32 * ammo.hull_damage_modifier()) as u32;
        self.hull_hp = self.hull_hp.saturating_sub(hull_dmg);

        // Rigging damage
        let sail_dmg = (damage as f32 * ammo.rigging_damage_modifier()) as u32;
        self.sail_hp = self.sail_hp.saturating_sub(sail_dmg);

        // Fire chance
        if rand_float() < ammo.fire_chance() {
            self.is_on_fire = true;
            self.fire_intensity = 0.3;
        }

        // Flooding if hull badly damaged
        if self.hull_hp < self.max_hull_hp / 4 {
            self.flooding_rate += 0.01;
        }
    }

    /// Apply crew casualties
    pub fn take_crew_damage(&mut self, casualties: u32) {
        self.crew_count = self.crew_count.saturating_sub(casualties);
        self.crew_morale -= casualties as f32 * 2.0;
        self.crew_morale = self.crew_morale.max(0.0);
    }

    /// Update ship state over time
    pub fn update(&mut self, delta_time: f32) {
        // Fire spreads and damages
        if self.is_on_fire {
            self.fire_intensity += 0.01 * delta_time;
            let fire_dmg = (self.fire_intensity * 5.0 * delta_time) as u32;
            self.hull_hp = self.hull_hp.saturating_sub(fire_dmg);
            self.sail_hp = self.sail_hp.saturating_sub(fire_dmg * 2);

            // Fire can be put out by crew
            if self.crew_count > 10 && rand_float() < 0.1 * delta_time {
                self.fire_intensity -= 0.1;
                if self.fire_intensity <= 0.0 {
                    self.is_on_fire = false;
                    self.fire_intensity = 0.0;
                }
            }
        }

        // Flooding causes damage
        if self.flooding_rate > 0.0 {
            let flood_dmg = (self.flooding_rate * 10.0 * delta_time) as u32;
            self.hull_hp = self.hull_hp.saturating_sub(flood_dmg);

            // Crew can pump water
            if self.crew_count > 5 {
                self.flooding_rate = (self.flooding_rate - 0.005 * delta_time).max(0.0);
            }
        }

        // Update gun cooldowns
        for gun in &mut self.guns {
            gun.update(delta_time);
        }

        // Morale recovery
        if self.crew_morale < 70.0 && !self.is_on_fire && self.flooding_rate == 0.0 {
            self.crew_morale += 0.1 * delta_time;
        }
    }

    /// Check if ship is sinking
    pub fn is_sinking(&self) -> bool {
        self.hull_hp == 0 || self.flooding_rate > 0.5
    }

    /// Check if ship is crippled (can't fight/sail effectively)
    pub fn is_crippled(&self) -> bool {
        self.sail_hp < 20 || self.crew_count < self.class.base_stats().min_crew / 2
    }

    /// Get current cargo weight
    pub fn cargo_weight(&self) -> u32 {
        self.current_cargo_weight
    }

    /// Check if can add cargo
    pub fn can_add_cargo(&self, weight: u32) -> bool {
        self.cargo_weight() + weight <= self.class.base_stats().cargo_capacity
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipGun {
    pub size: GunSize,
    pub is_loaded: bool,
    pub cooldown: f32,
    pub condition: f32,  // 0-1, affects accuracy
    pub side: GunSide,
}

impl ShipGun {
    pub fn new(size: GunSize, side: GunSide) -> Self {
        Self {
            size,
            is_loaded: false,
            cooldown: 0.0,
            condition: 1.0,
            side,
        }
    }

    pub fn ready_to_fire(&self) -> bool {
        self.is_loaded && self.cooldown <= 0.0
    }

    pub fn fire(&mut self) {
        self.is_loaded = false;
        self.cooldown = self.size.reload_time();
        self.condition -= 0.01; // Wear
    }

    pub fn reload(&mut self) {
        self.is_loaded = true;
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.cooldown > 0.0 {
            self.cooldown -= delta_time;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GunSide {
    Port,       // Left
    Starboard,  // Right
    Bow,        // Front (chase guns)
    Stern,      // Rear (chase guns)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoItem {
    pub item_type: CargoType,
    pub quantity: u32,
    pub weight: u32,
    pub value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CargoType {
    // Trade goods
    Tobacco,
    Furs,
    Timber,
    Fish,
    Grain,
    Sugar,
    Rum,
    Cotton,
    Indigo,
    // Valuable
    Gold,
    Silver,
    Spices,
    Silk,
    // Supplies
    Provisions,
    Water,
    Gunpowder,
    Shot,
    Rope,
    Sailcloth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    English,
    Spanish,
    French,
    Dutch,
    Native,
    Pirate,
    Player,
    Neutral,
}

impl Faction {
    pub fn is_hostile_to(&self, other: &Faction) -> bool {
        match (self, other) {
            (Self::Pirate, _) | (_, Self::Pirate) => true,
            (Self::English, Self::Spanish) | (Self::Spanish, Self::English) => true,
            (Self::French, Self::English) | (Self::English, Self::French) => true,
            (Self::Spanish, Self::French) | (Self::French, Self::Spanish) => true,
            _ => false,
        }
    }
}

// Simple random for damage calculations
fn rand_float() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}
