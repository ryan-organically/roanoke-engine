//! Ship management and fleet system
//!
//! Handles ship creation, maintenance, and fleet operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{Ship, ShipClass, ShipGun, GunSize, GunSide, Faction, AmmoType, CargoItem, CargoType};
use super::crew::CrewRoster;

/// Ship manager handling all ships in the game world
#[derive(Debug, Default)]
pub struct ShipManager {
    pub ships: HashMap<u64, Ship>,
    pub ship_crews: HashMap<u64, CrewRoster>,
    next_ship_id: u64,
}

impl ShipManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new ship
    pub fn create_ship(&mut self, name: &str, class: ShipClass, faction: Faction) -> u64 {
        let id = self.next_ship_id;
        self.next_ship_id += 1;

        let ship = Ship::new(id, name, class, faction);
        self.ships.insert(id, ship);
        self.ship_crews.insert(id, CrewRoster::new());

        id
    }

    /// Create a fully equipped ship
    pub fn create_equipped_ship(
        &mut self,
        name: &str,
        class: ShipClass,
        faction: Faction,
    ) -> u64 {
        let id = self.create_ship(name, class, faction);

        // Equip with standard armament
        if let Some(ship) = self.ships.get_mut(&id) {
            let stats = class.base_stats();

            // Add guns based on ship class
            let gun_size = match stats.max_gun_size {
                GunSize::None => None,
                GunSize::Swivel => Some(GunSize::Swivel),
                GunSize::Falconet => Some(GunSize::Falconet),
                GunSize::Saker => Some(GunSize::Saker),
                GunSize::Minion => Some(GunSize::Minion),
                GunSize::Culverin => Some(GunSize::Culverin),
                GunSize::DemiCannon => Some(GunSize::DemiCannon),
                GunSize::Cannon => Some(GunSize::Cannon),
            };

            if let Some(size) = gun_size {
                let ports_per_side = stats.gun_ports / 2;
                for _ in 0..ports_per_side {
                    ship.guns.push(ShipGun::new(size, GunSide::Port));
                    ship.guns.push(ShipGun::new(size, GunSide::Starboard));
                }
            }

            // Add starting ammunition
            ship.ammo_stores.insert(AmmoType::RoundShot, stats.gun_ports * 20);
            ship.ammo_stores.insert(AmmoType::ChainShot, stats.gun_ports * 10);
            ship.ammo_stores.insert(AmmoType::Grapeshot, stats.gun_ports * 10);

            // Set crew to minimum
            ship.crew_count = stats.min_crew;
        }

        // Add crew roster
        if let Some(roster) = self.ship_crews.get_mut(&id) {
            let stats = class.base_stats();
            let crew_needed = stats.min_crew;

            // Add essential officers
            roster.recruit(super::crew::CrewRole::Captain);
            if crew_needed > 10 {
                roster.recruit(super::crew::CrewRole::FirstMate);
            }
            if crew_needed > 20 {
                roster.recruit(super::crew::CrewRole::Quartermaster);
                roster.recruit(super::crew::CrewRole::Navigator);
            }
            if crew_needed > 30 {
                roster.recruit(super::crew::CrewRole::Surgeon);
                roster.recruit(super::crew::CrewRole::Carpenter);
            }

            // Add gunners and sailors
            let gunners_needed = stats.gun_ports / 4;
            for _ in 0..gunners_needed {
                roster.recruit(super::crew::CrewRole::Gunner);
            }

            let sailors_remaining = crew_needed.saturating_sub(roster.count());
            for _ in 0..sailors_remaining {
                roster.recruit(super::crew::CrewRole::Sailor);
            }

            // Starting provisions
            roster.provisions_remaining = 30;
            roster.water_remaining = 20;
        }

        id
    }

    /// Get a ship by ID
    pub fn get_ship(&self, id: u64) -> Option<&Ship> {
        self.ships.get(&id)
    }

    /// Get a mutable ship by ID
    pub fn get_ship_mut(&mut self, id: u64) -> Option<&mut Ship> {
        self.ships.get_mut(&id)
    }

    /// Get ship's crew roster
    pub fn get_crew(&self, id: u64) -> Option<&CrewRoster> {
        self.ship_crews.get(&id)
    }

    /// Get mutable crew roster
    pub fn get_crew_mut(&mut self, id: u64) -> Option<&mut CrewRoster> {
        self.ship_crews.get_mut(&id)
    }

    /// Update all ships
    pub fn update(&mut self, delta_time: f32) {
        let sinking: Vec<u64> = self.ships
            .iter()
            .filter(|(_, ship)| ship.is_sinking())
            .map(|(id, _)| *id)
            .collect();

        for id in sinking {
            self.ships.remove(&id);
            self.ship_crews.remove(&id);
        }

        for ship in self.ships.values_mut() {
            ship.update(delta_time);
        }
    }

    /// Get all ships belonging to a faction
    pub fn ships_by_faction(&self, faction: Faction) -> Vec<&Ship> {
        self.ships.values().filter(|s| s.faction == faction).collect()
    }

    /// Get ships near a position
    pub fn ships_near(&self, position: [f32; 2], radius: f32) -> Vec<&Ship> {
        self.ships
            .values()
            .filter(|s| {
                let dx = s.position[0] - position[0];
                let dz = s.position[1] - position[1];
                (dx * dx + dz * dz).sqrt() <= radius
            })
            .collect()
    }

    /// Remove a ship (sunk or captured)
    pub fn remove_ship(&mut self, id: u64) {
        self.ships.remove(&id);
        self.ship_crews.remove(&id);
    }
}

/// Shipyard for purchasing and upgrading ships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipyard {
    pub name: String,
    pub position: [f32; 2],
    pub available_classes: Vec<ShipClass>,
    pub price_modifier: f32,
    pub repair_rate: f32,
}

impl Shipyard {
    pub fn new(name: &str, position: [f32; 2]) -> Self {
        Self {
            name: name.to_string(),
            position,
            available_classes: vec![
                ShipClass::Rowboat,
                ShipClass::Pinnace,
                ShipClass::Shallop,
                ShipClass::Sloop,
            ],
            price_modifier: 1.0,
            repair_rate: 10.0, // Hull HP per day
        }
    }

    pub fn major_shipyard(name: &str, position: [f32; 2]) -> Self {
        Self {
            name: name.to_string(),
            position,
            available_classes: vec![
                ShipClass::Rowboat,
                ShipClass::Pinnace,
                ShipClass::Shallop,
                ShipClass::Sloop,
                ShipClass::Brigantine,
                ShipClass::Merchantman,
                ShipClass::Frigate,
            ],
            price_modifier: 0.9,
            repair_rate: 20.0,
        }
    }

    /// Get purchase price for a ship class
    pub fn get_price(&self, class: ShipClass) -> u32 {
        if !self.available_classes.contains(&class) {
            return 0;
        }
        (class.base_price() as f32 * self.price_modifier) as u32
    }

    /// Calculate repair cost
    pub fn repair_cost(&self, ship: &Ship) -> u32 {
        let damage = ship.max_hull_hp - ship.hull_hp;
        let sail_damage = ship.max_sail_hp - ship.sail_hp;
        (damage + sail_damage) * 2
    }

    /// Calculate repair time in days
    pub fn repair_time(&self, ship: &Ship) -> f32 {
        let damage = (ship.max_hull_hp - ship.hull_hp) as f32;
        damage / self.repair_rate
    }

    /// Repair a ship (returns days needed)
    pub fn repair_ship(&self, ship: &mut Ship) -> f32 {
        let time = self.repair_time(ship);
        ship.hull_hp = ship.max_hull_hp;
        ship.sail_hp = ship.max_sail_hp;
        ship.mast_damage = 0.0;
        ship.is_on_fire = false;
        ship.fire_intensity = 0.0;
        ship.flooding_rate = 0.0;
        time
    }
}

/// A player's fleet of ships
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Fleet {
    pub ship_ids: Vec<u64>,
    pub flagship_id: Option<u64>,
    pub name: String,
    pub formation: FleetFormation,
}

impl Fleet {
    pub fn new(name: &str) -> Self {
        Self {
            ship_ids: Vec::new(),
            flagship_id: None,
            name: name.to_string(),
            formation: FleetFormation::Line,
        }
    }

    pub fn add_ship(&mut self, id: u64) {
        if !self.ship_ids.contains(&id) {
            self.ship_ids.push(id);
            if self.flagship_id.is_none() {
                self.flagship_id = Some(id);
            }
        }
    }

    pub fn remove_ship(&mut self, id: u64) {
        self.ship_ids.retain(|&i| i != id);
        if self.flagship_id == Some(id) {
            self.flagship_id = self.ship_ids.first().copied();
        }
    }

    pub fn set_flagship(&mut self, id: u64) {
        if self.ship_ids.contains(&id) {
            self.flagship_id = Some(id);
        }
    }

    pub fn ship_count(&self) -> usize {
        self.ship_ids.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FleetFormation {
    #[default]
    Line,       // Ships in a line
    Column,     // Single file
    Wedge,      // V-formation
    Scattered,  // Spread out
}

/// Trade route for cargo ships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRoute {
    pub name: String,
    pub waypoints: Vec<[f32; 2]>,
    pub goods_exported: Vec<CargoType>,
    pub goods_imported: Vec<CargoType>,
    pub danger_level: f32,  // 0-1, pirate/enemy activity
}

impl TradeRoute {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            waypoints: Vec::new(),
            goods_exported: Vec::new(),
            goods_imported: Vec::new(),
            danger_level: 0.1,
        }
    }

    pub fn add_waypoint(&mut self, position: [f32; 2]) {
        self.waypoints.push(position);
    }

    pub fn total_distance(&self) -> f32 {
        let mut total = 0.0;
        for i in 1..self.waypoints.len() {
            let dx = self.waypoints[i][0] - self.waypoints[i-1][0];
            let dz = self.waypoints[i][1] - self.waypoints[i-1][1];
            total += (dx * dx + dz * dz).sqrt();
        }
        total
    }
}

/// Upgrade options for ships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipUpgrade {
    ReinforcedHull,      // +20% hull HP
    CopperSheathing,     // +10% speed, prevent barnacles
    ExtendedGunPorts,    // +2 gun capacity
    ImprovedRigging,     // +15% maneuverability
    LargerHold,          // +20% cargo capacity
    ReinforcedMasts,     // Masts take less damage
    SwivelsAdded,        // Anti-personnel on rails
}

impl ShipUpgrade {
    pub fn cost(&self) -> u32 {
        match self {
            Self::ReinforcedHull => 500,
            Self::CopperSheathing => 800,
            Self::ExtendedGunPorts => 400,
            Self::ImprovedRigging => 350,
            Self::LargerHold => 600,
            Self::ReinforcedMasts => 450,
            Self::SwivelsAdded => 200,
        }
    }

    pub fn installation_time(&self) -> f32 {
        match self {
            Self::ReinforcedHull => 14.0,
            Self::CopperSheathing => 21.0,
            Self::ExtendedGunPorts => 10.0,
            Self::ImprovedRigging => 7.0,
            Self::LargerHold => 14.0,
            Self::ReinforcedMasts => 10.0,
            Self::SwivelsAdded => 3.0,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::ReinforcedHull => "Thicker hull planking for increased durability",
            Self::CopperSheathing => "Copper plating prevents fouling and increases speed",
            Self::ExtendedGunPorts => "Additional gun ports for more firepower",
            Self::ImprovedRigging => "Better rigging for improved handling",
            Self::LargerHold => "Expanded cargo hold capacity",
            Self::ReinforcedMasts => "Stronger masts resistant to damage",
            Self::SwivelsAdded => "Swivel guns added to rails for boarding defense",
        }
    }
}

pub fn apply_upgrade(ship: &mut Ship, upgrade: ShipUpgrade) {
    match upgrade {
        ShipUpgrade::ReinforcedHull => {
            ship.max_hull_hp = (ship.max_hull_hp as f32 * 1.2) as u32;
            ship.hull_hp = ship.max_hull_hp;
        }
        ShipUpgrade::CopperSheathing => {
            // Speed bonus handled in sailing calculations
        }
        ShipUpgrade::ExtendedGunPorts => {
            // Add 2 gun mounts
            let max_size = ship.class.base_stats().max_gun_size;
            ship.guns.push(ShipGun::new(max_size, GunSide::Port));
            ship.guns.push(ShipGun::new(max_size, GunSide::Starboard));
        }
        ShipUpgrade::ImprovedRigging => {
            // Maneuverability bonus handled in sailing
        }
        ShipUpgrade::LargerHold => {
            // Cargo bonus handled in cargo system
        }
        ShipUpgrade::ReinforcedMasts => {
            // Damage reduction handled in combat
        }
        ShipUpgrade::SwivelsAdded => {
            ship.guns.push(ShipGun::new(GunSize::Swivel, GunSide::Port));
            ship.guns.push(ShipGun::new(GunSize::Swivel, GunSide::Starboard));
            ship.guns.push(ShipGun::new(GunSize::Swivel, GunSide::Bow));
            ship.guns.push(ShipGun::new(GunSize::Swivel, GunSide::Stern));
        }
    }
}
