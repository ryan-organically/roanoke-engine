//! Central animal management system

use super::behavior::{update_behavior, BehaviorContext, BehaviorState};
use super::entity::{Animal, AnimalId, PackId, Target};
use super::spatial::SpatialHash;
use super::types::{AnimalSpecies, Difficulty, TimeOfDay};
use glam::Vec3;
use std::collections::HashMap;

/// Central manager for all animal entities
pub struct AnimalManager {
    // Entity storage
    animals: HashMap<AnimalId, Animal>,
    next_id: u64,

    // Spatial indexing (cell size = 16 units)
    spatial: SpatialHash<AnimalId>,

    // Pack tracking
    packs: HashMap<PackId, Pack>,
    next_pack_id: u64,

    // Global state
    pub difficulty: Difficulty,
    pub time_of_day: TimeOfDay,

    // Statistics
    pub total_spawned: u64,
    pub total_killed: u64,
}

/// Pack of animals that coordinate behavior
#[derive(Debug)]
pub struct Pack {
    pub id: PackId,
    pub species: AnimalSpecies,
    pub members: Vec<AnimalId>,
    pub alpha: Option<AnimalId>,
    pub morale: f32,
}

impl Pack {
    fn new(id: PackId, species: AnimalSpecies) -> Self {
        Self {
            id,
            species,
            members: Vec::new(),
            alpha: None,
            morale: 1.0,
        }
    }

    /// Update pack state based on member health
    pub fn update(&mut self, animals: &HashMap<AnimalId, Animal>) {
        // Remove dead members
        self.members
            .retain(|id| animals.get(id).map(|a| a.is_alive()).unwrap_or(false));

        if self.members.is_empty() {
            return;
        }

        // Calculate morale based on health
        let total_health: f32 = self
            .members
            .iter()
            .filter_map(|id| animals.get(id))
            .map(|a| a.current_health / a.max_health)
            .sum();
        let avg_health = total_health / self.members.len() as f32;

        // Morale based on health and pack size
        let size_factor = (self.members.len() as f32 / 4.0).min(1.0);
        self.morale = (avg_health * 0.6 + size_factor * 0.4).clamp(0.0, 1.0);

        // Elect alpha (highest health)
        self.alpha = self
            .members
            .iter()
            .filter_map(|id| animals.get(id).map(|a| (*id, a.current_health)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| id);
    }
}

impl AnimalManager {
    /// Create a new animal manager
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            animals: HashMap::new(),
            next_id: 1,
            spatial: SpatialHash::new(16.0),
            packs: HashMap::new(),
            next_pack_id: 1,
            difficulty,
            time_of_day: TimeOfDay::Day,
            total_spawned: 0,
            total_killed: 0,
        }
    }

    /// Spawn a new animal at the given position
    pub fn spawn(
        &mut self,
        species: AnimalSpecies,
        position: Vec3,
        chunk: (i32, i32),
        pack_id: Option<PackId>,
    ) -> AnimalId {
        let id = AnimalId(self.next_id);
        self.next_id += 1;

        // Calculate health with difficulty modifier
        let base_health = species.base_stats().health;
        let modified_health = base_health * self.difficulty.health_multiplier();

        let mut animal = Animal::new(id, species, position, modified_health, chunk);
        animal.pack_id = pack_id;

        // Add to pack if specified
        if let Some(pack_id) = pack_id {
            if let Some(pack) = self.packs.get_mut(&pack_id) {
                pack.members.push(id);
            }
        }

        self.spatial.insert(id, position);
        self.animals.insert(id, animal);
        self.total_spawned += 1;

        id
    }

    /// Create a new pack and return its ID
    pub fn create_pack(&mut self, species: AnimalSpecies) -> PackId {
        let id = PackId(self.next_pack_id);
        self.next_pack_id += 1;
        self.packs.insert(id, Pack::new(id, species));
        id
    }

    /// Despawn an animal
    pub fn despawn(&mut self, id: AnimalId) {
        if let Some(animal) = self.animals.remove(&id) {
            self.spatial.remove(id);

            // Remove from pack
            if let Some(pack_id) = animal.pack_id {
                if let Some(pack) = self.packs.get_mut(&pack_id) {
                    pack.members.retain(|&mid| mid != id);
                }
            }
        }
    }

    /// Get animal by ID
    pub fn get(&self, id: AnimalId) -> Option<&Animal> {
        self.animals.get(&id)
    }

    /// Get mutable animal by ID
    pub fn get_mut(&mut self, id: AnimalId) -> Option<&mut Animal> {
        self.animals.get_mut(&id)
    }

    /// Query animals in radius
    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<AnimalId> {
        self.spatial.query_radius(center, radius)
    }

    /// Query animals in chunk
    pub fn query_chunk(&self, chunk_x: i32, chunk_z: i32, chunk_size: f32) -> Vec<AnimalId> {
        self.spatial.query_chunk(chunk_x, chunk_z, chunk_size)
    }

    /// Get total number of animals
    pub fn animal_count(&self) -> usize {
        self.animals.len()
    }

    /// Set time of day (affects spawning and behavior)
    pub fn set_time_of_day(&mut self, time: TimeOfDay) {
        self.time_of_day = time;
    }

    /// Main update tick
    pub fn update(&mut self, dt: f32, player_pos: Vec3, player_velocity: Vec3) {
        // Collect IDs to update (avoid borrow issues)
        let ids: Vec<AnimalId> = self.animals.keys().copied().collect();

        // Update each animal
        for id in ids {
            // Get nearby animals for context
            let pos = match self.animals.get(&id) {
                Some(a) => a.position,
                None => continue,
            };
            let nearby = self.spatial.query_radius(pos, 50.0);

            let ctx = BehaviorContext {
                player_pos,
                player_velocity,
                dt,
                nearby_animals: &nearby,
            };

            // Update behavior
            if let Some(animal) = self.animals.get_mut(&id) {
                if animal.is_alive() {
                    update_behavior(animal, &ctx);

                    // Update cooldowns and effects
                    animal.update_cooldowns(dt);
                    animal.update_effects(dt);

                    // Update animation
                    animal.update_animation(dt);

                    // Apply movement
                    let new_pos = animal.position + animal.velocity * dt;
                    animal.position = new_pos;

                    // Update spatial hash
                    self.spatial.update(id, new_pos);

                    // Update despawn timer
                    if let Some(timer) = &mut animal.despawn_timer {
                        *timer -= dt;
                    }
                }
            }
        }

        // Update packs
        for pack in self.packs.values_mut() {
            pack.update(&self.animals);
        }

        // Despawn animals with expired timers or dead for too long
        let to_despawn: Vec<AnimalId> = self
            .animals
            .iter()
            .filter(|(_, animal)| {
                animal.despawn_timer.map(|t| t <= 0.0).unwrap_or(false)
                    || (animal.is_dead() && animal.animation_time > 60.0) // Despawn corpses after 60s
            })
            .map(|(&id, _)| id)
            .collect();

        for id in to_despawn {
            self.despawn(id);
        }

        // Clean up empty packs
        self.packs.retain(|_, pack| !pack.members.is_empty());
    }

    /// Get all animals for rendering
    pub fn animals_iter(&self) -> impl Iterator<Item = &Animal> {
        self.animals.values()
    }

    /// Get animals near player for rendering (optimization)
    pub fn animals_near(&self, center: Vec3, radius: f32) -> Vec<&Animal> {
        self.spatial
            .query_radius(center, radius)
            .iter()
            .filter_map(|id| self.animals.get(id))
            .collect()
    }

    /// Alert pack when one member is attacked
    pub fn alert_pack(&mut self, pack_id: PackId, threat_pos: Vec3) {
        if let Some(pack) = self.packs.get(&pack_id) {
            let members = pack.members.clone();
            for member_id in members {
                if let Some(animal) = self.animals.get_mut(&member_id) {
                    animal.awareness = 1.0;
                    animal.target = Some(Target::Player);
                    if animal.behavior_state == BehaviorState::Idle
                        || matches!(animal.behavior_state, BehaviorState::Alert(_))
                    {
                        animal.behavior_state =
                            BehaviorState::Pursue(super::behavior::PursueState::Chasing);
                    }
                    animal.last_seen_player = Some((threat_pos, std::time::Instant::now()));
                }
            }
        }
    }

    /// Apply damage to an animal from the player
    pub fn damage_animal(&mut self, id: AnimalId, damage: f32) -> bool {
        // Get info we need before mutating
        let (pack_id, pos) = match self.animals.get(&id) {
            Some(a) => (a.pack_id, a.position),
            None => return false,
        };

        // Apply damage
        let (was_alive, is_dead) = {
            let animal = match self.animals.get_mut(&id) {
                Some(a) => a,
                None => return false,
            };
            let was_alive = animal.is_alive();
            animal.take_damage(damage, super::entity::DamageSource::Player);
            (was_alive, animal.is_dead())
        };

        // Alert pack (now that we've released the borrow)
        if let Some(pack_id) = pack_id {
            self.alert_pack(pack_id, pos);
        }

        if was_alive && is_dead {
            self.total_killed += 1;
            return true; // Killed
        }

        false
    }

    /// Get debug info string
    pub fn debug_info(&self) -> String {
        format!(
            "Animals: {} | Packs: {} | Spawned: {} | Killed: {}",
            self.animals.len(),
            self.packs.len(),
            self.total_spawned,
            self.total_killed
        )
    }
}
