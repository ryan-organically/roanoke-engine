//! Horse Stable System
//!
//! Manages collection of owned horses, stabling, and breeding.

use super::entity::{Horse, HorseId, HerdId, OwnershipState};
use super::taming::HorseTamingSystem;
use super::training::TrainingSystem;
use super::types::{HorseSpecies, HorseGender, HorseAge, HorseCoat};
use super::encephalon::{ThreatInfo, EnvironmentContext};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum horses a player can own
pub const MAX_OWNED_HORSES: usize = 12;

/// A stabled horse (not currently in the world)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabledHorse {
    pub id: HorseId,
    pub name: String,
    pub species: HorseSpecies,
    pub coat: HorseCoat,
    pub gender: HorseGender,
    pub age: HorseAge,
    pub level: u8,
    pub health_percent: f32,
    pub stamina_percent: f32,
    pub bond_level: f32,
}

impl StabledHorse {
    pub fn from_horse(horse: &Horse) -> Self {
        Self {
            id: horse.id,
            name: horse.name.clone(),
            species: horse.species,
            coat: horse.coat,
            gender: horse.gender,
            age: horse.age,
            level: horse.level,
            health_percent: horse.health / horse.max_health,
            stamina_percent: horse.stamina / horse.max_stamina,
            bond_level: horse.bond_level,
        }
    }
}

/// Manages all horse-related systems
#[derive(Debug)]
pub struct Stable {
    /// All active horses in the world (wild + tamed)
    pub active_horses: HashMap<HorseId, Horse>,
    /// Stabled horses (not in world)
    pub stabled_horses: HashMap<HorseId, StabledHorse>,
    /// Wild herds
    pub herds: HashMap<HerdId, HerdInfo>,
    /// Current active mount
    pub active_mount: Option<HorseId>,
    /// Taming system
    pub taming: HorseTamingSystem,
    /// Training system
    pub training: TrainingSystem,
    /// Next IDs
    next_horse_id: u64,
    next_herd_id: u64,
}

/// Information about a wild herd
#[derive(Debug, Clone)]
pub struct HerdInfo {
    pub id: HerdId,
    pub species: HorseSpecies,
    pub members: Vec<HorseId>,
    pub center: Vec3,
    pub territory_radius: f32,
}

impl Default for Stable {
    fn default() -> Self {
        Self::new()
    }
}

impl Stable {
    pub fn new() -> Self {
        Self {
            active_horses: HashMap::new(),
            stabled_horses: HashMap::new(),
            herds: HashMap::new(),
            active_mount: None,
            taming: HorseTamingSystem::new(),
            training: TrainingSystem::new(),
            next_horse_id: 1,
            next_herd_id: 1,
        }
    }

    /// Spawn a wild horse
    pub fn spawn_wild_horse(
        &mut self,
        species: HorseSpecies,
        position: Vec3,
        chunk: (i32, i32),
    ) -> HorseId {
        let id = HorseId(self.next_horse_id);
        self.next_horse_id += 1;

        let personality_seed = id.0.wrapping_mul(0x9E3779B97F4A7C15);
        let horse = Horse::new_wild(id, species, position, chunk, personality_seed);

        self.active_horses.insert(id, horse);
        id
    }

    /// Spawn a wild herd
    pub fn spawn_herd(
        &mut self,
        species: HorseSpecies,
        center: Vec3,
        chunk: (i32, i32),
        size: usize,
    ) -> HerdId {
        let herd_id = HerdId(self.next_herd_id);
        self.next_herd_id += 1;

        let mut members = Vec::new();

        for i in 0..size {
            // Spread horses around the center
            let angle = (i as f32 / size as f32) * std::f32::consts::TAU;
            let radius = 5.0 + (i as f32 % 3.0) * 3.0;
            let offset = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
            let pos = center + offset;

            let horse_id = self.spawn_wild_horse(species, pos, chunk);

            // Set herd info
            if let Some(horse) = self.active_horses.get_mut(&horse_id) {
                horse.herd_id = Some(herd_id);
                horse.home_position = center;
                horse.territory_radius = 80.0;
            }

            members.push(horse_id);
        }

        let herd = HerdInfo {
            id: herd_id,
            species,
            members,
            center,
            territory_radius: 100.0,
        };

        self.herds.insert(herd_id, herd);
        herd_id
    }

    /// Update all active horses
    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Option<Vec3>,
        threats: &[ThreatInfo],
        environment: &EnvironmentContext,
    ) {
        // Collect positions for herd awareness
        let positions: Vec<(HorseId, Vec3)> = self.active_horses
            .iter()
            .map(|(id, h)| (*id, h.position))
            .collect();

        // Update each horse
        for (id, horse) in self.active_horses.iter_mut() {
            // Get nearby horse positions (excluding self)
            let nearby: Vec<Vec3> = positions
                .iter()
                .filter(|(other_id, pos)| {
                    *other_id != *id && pos.distance(horse.position) < 50.0
                })
                .map(|(_, pos)| *pos)
                .collect();

            horse.update(dt, player_pos, &nearby, threats, environment);
        }

        // Update training if active
        if let Some(mount_id) = self.active_mount {
            if let Some(horse) = self.active_horses.get_mut(&mount_id) {
                self.training.update(horse, dt);
            }
        }

        // Clean up dead horses
        self.active_horses.retain(|_, h| h.is_alive());
    }

    /// Get an active horse by ID
    pub fn get_horse(&self, id: HorseId) -> Option<&Horse> {
        self.active_horses.get(&id)
    }

    /// Get a mutable active horse by ID
    pub fn get_horse_mut(&mut self, id: HorseId) -> Option<&mut Horse> {
        self.active_horses.get_mut(&id)
    }

    /// Add a tamed horse to ownership
    pub fn claim_horse(&mut self, id: HorseId, name: String) -> Result<(), &'static str> {
        let owned_count = self.active_horses
            .values()
            .filter(|h| h.ownership == OwnershipState::Owned)
            .count()
            + self.stabled_horses.len();

        if owned_count >= MAX_OWNED_HORSES {
            return Err("Maximum horses owned");
        }

        if let Some(horse) = self.active_horses.get_mut(&id) {
            horse.ownership = OwnershipState::Owned;
            horse.name = name;
            Ok(())
        } else {
            Err("Horse not found")
        }
    }

    /// Stable a horse (remove from world)
    pub fn stable_horse(&mut self, id: HorseId) -> Result<(), &'static str> {
        if self.active_mount == Some(id) {
            return Err("Cannot stable mounted horse");
        }

        if let Some(horse) = self.active_horses.remove(&id) {
            if horse.ownership != OwnershipState::Owned {
                // Put it back, can't stable wild horses
                self.active_horses.insert(id, horse);
                return Err("Can only stable owned horses");
            }

            let stabled = StabledHorse::from_horse(&horse);
            self.stabled_horses.insert(id, stabled);
            Ok(())
        } else {
            Err("Horse not found")
        }
    }

    /// Retrieve a horse from stable
    pub fn retrieve_horse(&mut self, id: HorseId, position: Vec3, chunk: (i32, i32)) -> Result<(), &'static str> {
        if let Some(stabled) = self.stabled_horses.remove(&id) {
            // Recreate horse from stabled info
            let mut horse = Horse::new_wild(
                stabled.id,
                stabled.species,
                position,
                chunk,
                stabled.id.0,
            );

            horse.name = stabled.name;
            horse.ownership = OwnershipState::Owned;
            horse.level = stabled.level;
            horse.bond_level = stabled.bond_level;
            horse.health = horse.max_health * stabled.health_percent;
            horse.stamina = horse.max_stamina * stabled.stamina_percent;
            horse.coat = stabled.coat;
            horse.gender = stabled.gender;
            horse.age = stabled.age;

            self.active_horses.insert(id, horse);
            Ok(())
        } else {
            Err("Horse not in stable")
        }
    }

    /// Mount a horse
    pub fn mount_horse(&mut self, id: HorseId) -> Result<(), &'static str> {
        if self.active_mount.is_some() {
            return Err("Already mounted");
        }

        if let Some(horse) = self.active_horses.get_mut(&id) {
            if horse.mount() {
                self.active_mount = Some(id);
                Ok(())
            } else {
                Err("Horse refused to be mounted")
            }
        } else {
            Err("Horse not found")
        }
    }

    /// Dismount current horse
    pub fn dismount(&mut self) {
        if let Some(id) = self.active_mount {
            if let Some(horse) = self.active_horses.get_mut(&id) {
                horse.dismount();
            }
            self.active_mount = None;
        }
    }

    /// Get current mount
    pub fn current_mount(&self) -> Option<&Horse> {
        self.active_mount.and_then(|id| self.active_horses.get(&id))
    }

    /// Get mutable current mount
    pub fn current_mount_mut(&mut self) -> Option<&mut Horse> {
        self.active_mount.and_then(|id| self.active_horses.get_mut(&id))
    }

    /// Get all owned horses (active + stabled)
    pub fn owned_horses(&self) -> Vec<HorseId> {
        let mut owned: Vec<HorseId> = self.active_horses
            .iter()
            .filter(|(_, h)| h.ownership == OwnershipState::Owned)
            .map(|(id, _)| *id)
            .collect();

        owned.extend(self.stabled_horses.keys().copied());
        owned
    }

    /// Get horses near a position
    pub fn horses_near(&self, pos: Vec3, radius: f32) -> Vec<&Horse> {
        self.active_horses
            .values()
            .filter(|h| h.position.distance(pos) < radius)
            .collect()
    }

    /// Get wild horses near a position
    pub fn wild_horses_near(&self, pos: Vec3, radius: f32) -> Vec<&Horse> {
        self.active_horses
            .values()
            .filter(|h| {
                h.ownership == OwnershipState::Wild
                    && h.position.distance(pos) < radius
            })
            .collect()
    }

    /// Despawn horses far from position
    pub fn despawn_distant(&mut self, pos: Vec3, max_distance: f32) {
        self.active_horses.retain(|_, h| {
            // Keep owned horses regardless of distance
            if h.ownership == OwnershipState::Owned {
                return true;
            }
            h.position.distance(pos) < max_distance
        });
    }

    /// Get statistics
    pub fn stats(&self) -> StableStats {
        let active_owned = self.active_horses
            .values()
            .filter(|h| h.ownership == OwnershipState::Owned)
            .count();

        StableStats {
            active_horses: self.active_horses.len(),
            stabled_horses: self.stabled_horses.len(),
            owned_horses: active_owned + self.stabled_horses.len(),
            wild_herds: self.herds.len(),
            currently_mounted: self.active_mount.is_some(),
        }
    }
}

/// Statistics about the stable
#[derive(Debug, Clone)]
pub struct StableStats {
    pub active_horses: usize,
    pub stabled_horses: usize,
    pub owned_horses: usize,
    pub wild_herds: usize,
    pub currently_mounted: bool,
}
