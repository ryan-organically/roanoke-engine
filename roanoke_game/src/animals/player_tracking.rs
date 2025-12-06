//! Animal-Player Relationship Tracking
//!
//! Tracks how animals perceive and remember the player for
//! more realistic and emergent wildlife behaviors.

use super::entity::AnimalId;
use super::types::AnimalSpecies;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracks how the player is perceived by wildlife
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerWildlifeReputation {
    /// Per-species fear/respect levels
    pub species_perception: HashMap<AnimalSpecies, SpeciesPerception>,
    /// Individual animal memories (for pack alphas, legendary beasts)
    pub individual_memories: HashMap<AnimalId, AnimalMemory>,
    /// Recent combat encounters
    pub recent_encounters: Vec<EncounterRecord>,
    /// Territory markers (areas where player has killed)
    pub territory_markers: Vec<TerritoryMarker>,
    /// Scent trails (movement history for tracking animals)
    pub scent_trail: Vec<(Vec3, f64)>,
    /// Player's current threat level
    pub threat_level: f32,
    /// Player's noise level (affects detection)
    pub noise_level: f32,
    /// Blood scent (attracts predators after kills)
    pub blood_scent: f32,
    /// Campfire protection radius
    pub fire_protection: Option<(Vec3, f32)>,
}

impl PlayerWildlifeReputation {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update after a game tick
    pub fn update(&mut self, dt: f32, player_pos: Vec3, game_time: f64) {
        // Decay threat level
        self.threat_level = (self.threat_level - 0.1 * dt).max(0.0);

        // Decay noise level
        self.noise_level = (self.noise_level - 0.5 * dt).max(0.0);

        // Decay blood scent
        self.blood_scent = (self.blood_scent - 0.05 * dt).max(0.0);

        // Update scent trail
        if let Some((last_pos, _)) = self.scent_trail.last() {
            if player_pos.distance(*last_pos) > 5.0 {
                self.scent_trail.push((player_pos, game_time));
            }
        } else {
            self.scent_trail.push((player_pos, game_time));
        }

        // Remove old scent trail entries (older than 30 game minutes)
        self.scent_trail.retain(|(_, time)| game_time - time < 0.5);

        // Keep only last 100 trail points
        if self.scent_trail.len() > 100 {
            self.scent_trail.drain(0..50);
        }

        // Remove old territory markers (older than 3 game days)
        self.territory_markers.retain(|m| game_time - m.timestamp < 72.0);

        // Remove old encounter records
        self.recent_encounters.retain(|e| game_time - e.timestamp < 24.0);
    }

    /// Record a kill
    pub fn record_kill(&mut self, species: AnimalSpecies, position: Vec3, was_stealth: bool, game_time: f64) {
        // Update species perception
        let perception = self.species_perception.entry(species).or_default();
        perception.kills += 1;
        perception.fear = (perception.fear + 20).min(100);
        perception.last_encounter = game_time;

        if was_stealth {
            perception.stealth_kills += 1;
            perception.fear += 10; // Extra fear from unseen death
        }

        // Increase threat level
        self.threat_level = (self.threat_level + 30.0).min(100.0);

        // Add blood scent
        self.blood_scent = (self.blood_scent + 50.0).min(100.0);

        // Add territory marker
        self.territory_markers.push(TerritoryMarker {
            position,
            timestamp: game_time,
            kill_count: 1,
            species,
        });

        // Merge nearby markers
        self.merge_nearby_markers(position, 50.0);

        // Record encounter
        self.recent_encounters.push(EncounterRecord {
            species,
            position,
            timestamp: game_time,
            outcome: EncounterOutcome::PlayerKilled,
        });
    }

    /// Record taking damage from an animal
    pub fn record_damage_taken(&mut self, species: AnimalSpecies, amount: f32, game_time: f64) {
        let perception = self.species_perception.entry(species).or_default();
        perception.damage_dealt_to_player += amount;
        perception.fear = (perception.fear - 5).max(0); // Animals less afraid if they hurt player
        perception.last_encounter = game_time;

        // Decrease threat level (player appears vulnerable)
        self.threat_level = (self.threat_level - 10.0).max(0.0);
    }

    /// Record fleeing from an animal
    pub fn record_fled(&mut self, species: AnimalSpecies, game_time: f64) {
        let perception = self.species_perception.entry(species).or_default();
        perception.fear = (perception.fear - 15).max(0); // Much less afraid
        perception.respect = (perception.respect - 10).max(-100);
        perception.last_encounter = game_time;

        self.recent_encounters.push(EncounterRecord {
            species,
            position: Vec3::ZERO, // Position not important for flee
            timestamp: game_time,
            outcome: EncounterOutcome::PlayerFled,
        });
    }

    /// Record using fire (scares animals)
    pub fn record_fire_use(&mut self, position: Vec3, radius: f32) {
        self.fire_protection = Some((position, radius));

        // All species become more afraid
        for perception in self.species_perception.values_mut() {
            perception.fear = (perception.fear + 5).min(100);
        }
    }

    /// Clear fire protection
    pub fn clear_fire(&mut self) {
        self.fire_protection = None;
    }

    /// Make noise (alerts animals)
    pub fn make_noise(&mut self, amount: f32) {
        self.noise_level = (self.noise_level + amount).min(100.0);
    }

    /// Get fear level for a species
    pub fn get_fear_level(&self, species: AnimalSpecies) -> i32 {
        self.species_perception.get(&species).map(|p| p.fear).unwrap_or(0)
    }

    /// Get respect level for a species
    pub fn get_respect_level(&self, species: AnimalSpecies) -> i32 {
        self.species_perception.get(&species).map(|p| p.respect).unwrap_or(0)
    }

    /// Check if animals should avoid an area
    pub fn is_dangerous_area(&self, position: Vec3) -> bool {
        self.territory_markers.iter()
            .any(|m| m.position.distance(position) < 100.0 && m.kill_count >= 3)
    }

    /// Get scent trail for tracking animals
    pub fn get_scent_at(&self, position: Vec3, max_age: f64, current_time: f64) -> Option<Vec3> {
        self.scent_trail.iter()
            .filter(|(_, time)| current_time - time < max_age)
            .min_by(|(pos, _), (pos2, _)| {
                pos.distance(position).partial_cmp(&pos2.distance(position)).unwrap_or(std::cmp::Ordering::Equal)
            })
            .filter(|(pos, _)| pos.distance(position) < 20.0)
            .map(|(pos, _)| *pos)
    }

    /// Check if player is protected by fire
    pub fn is_fire_protected(&self, check_pos: Vec3) -> bool {
        if let Some((fire_pos, radius)) = self.fire_protection {
            fire_pos.distance(check_pos) <= radius
        } else {
            false
        }
    }

    /// Merge nearby territory markers
    fn merge_nearby_markers(&mut self, center: Vec3, radius: f32) {
        let nearby: Vec<usize> = self.territory_markers.iter()
            .enumerate()
            .filter(|(_, m)| m.position.distance(center) < radius)
            .map(|(i, _)| i)
            .collect();

        if nearby.len() > 1 {
            // Sum up kills
            let total_kills: u32 = nearby.iter()
                .map(|&i| self.territory_markers[i].kill_count)
                .sum();

            // Keep only the first marker with combined kills
            if let Some(&first_idx) = nearby.first() {
                self.territory_markers[first_idx].kill_count = total_kills;
                self.territory_markers[first_idx].position = center;

                // Remove others (in reverse to preserve indices)
                for &idx in nearby.iter().skip(1).rev() {
                    self.territory_markers.remove(idx);
                }
            }
        }
    }

    /// Calculate animal detection modifier based on player state
    pub fn detection_modifier(&self) -> f32 {
        let mut modifier = 1.0;

        // Noise increases detection
        modifier += self.noise_level / 100.0;

        // Blood scent attracts predators
        modifier += self.blood_scent / 200.0;

        // Threat level makes animals more cautious
        modifier -= self.threat_level / 300.0;

        modifier.max(0.2)
    }

    /// Get behavior modifier for a species encountering player
    pub fn behavior_modifier(&self, species: AnimalSpecies) -> AnimalBehaviorMod {
        let perception = self.species_perception.get(&species);

        match perception {
            Some(p) if p.fear > 70 => AnimalBehaviorMod::Flee,
            Some(p) if p.fear > 50 && p.kills > 3 => AnimalBehaviorMod::AvoidArea,
            Some(p) if p.fear < 20 && p.respect < -30 => AnimalBehaviorMod::Aggressive,
            Some(p) if p.respect > 50 => AnimalBehaviorMod::Cautious,
            _ => AnimalBehaviorMod::Normal,
        }
    }

    /// Store memory for individual animal (alphas, legendaries)
    pub fn remember_animal(&mut self, id: AnimalId, memory: AnimalMemory) {
        self.individual_memories.insert(id, memory);
    }

    /// Get memory of specific animal
    pub fn get_animal_memory(&self, id: AnimalId) -> Option<&AnimalMemory> {
        self.individual_memories.get(&id)
    }
}

/// Per-species perception of the player
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpeciesPerception {
    /// Number of this species killed
    pub kills: u32,
    /// Stealth kills (scarier)
    pub stealth_kills: u32,
    /// Fear level (0-100)
    pub fear: i32,
    /// Respect level (-100 to 100)
    pub respect: i32,
    /// Damage dealt to player by this species
    pub damage_dealt_to_player: f32,
    /// Last encounter time
    pub last_encounter: f64,
}

/// Memory of individual animal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimalMemory {
    pub species: AnimalSpecies,
    pub is_alpha: bool,
    pub is_legendary: bool,
    pub encounters: u32,
    pub damage_to_player: f32,
    pub damage_from_player: f32,
    pub last_seen_position: Vec3,
    pub last_seen_time: f64,
    pub escaped: bool,
}

/// Territory marker where player has killed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryMarker {
    pub position: Vec3,
    pub timestamp: f64,
    pub kill_count: u32,
    pub species: AnimalSpecies,
}

/// Record of an encounter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterRecord {
    pub species: AnimalSpecies,
    pub position: Vec3,
    pub timestamp: f64,
    pub outcome: EncounterOutcome,
}

/// Outcome of an encounter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterOutcome {
    PlayerKilled,
    AnimalKilled,
    PlayerFled,
    AnimalFled,
    Standoff,
}

/// Behavior modification based on player reputation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimalBehaviorMod {
    Normal,
    Aggressive,  // More likely to attack
    Cautious,    // Keeps distance but follows
    AvoidArea,   // Stays away from player's territory
    Flee,        // Always runs
}

/// Animal den/lair tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenInfo {
    pub id: u64,
    pub position: Vec3,
    pub species: AnimalSpecies,
    pub discovered: bool,
    pub marked_on_map: bool,
    pub population: u8,
    pub last_spawn: f64,
    pub is_legendary_lair: bool,
}

/// Legendary animal tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendaryAnimal {
    pub id: String,
    pub name: String,
    pub species: AnimalSpecies,
    pub position: Vec3,
    pub is_spawned: bool,
    pub is_killed: bool,
    pub encounter_count: u32,
    pub damage_dealt: f32,
    pub rewards: Vec<String>,
}

impl Default for LegendaryAnimal {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            species: AnimalSpecies::BlackBear,
            position: Vec3::ZERO,
            is_spawned: false,
            is_killed: false,
            encounter_count: 0,
            damage_dealt: 0.0,
            rewards: Vec::new(),
        }
    }
}

/// Legendary animals in the game
pub fn create_legendary_animals() -> Vec<LegendaryAnimal> {
    vec![
        LegendaryAnimal {
            id: "ghost_cougar".to_string(),
            name: "The Ghost of the Mountain".to_string(),
            species: AnimalSpecies::EasternCougar,
            position: Vec3::new(500.0, 150.0, 500.0), // Mountain peak
            rewards: vec!["ghost_cougar_pelt".to_string(), "invisibility_cloak".to_string()],
            ..Default::default()
        },
        LegendaryAnimal {
            id: "fenrir".to_string(),
            name: "Fenrir, the Giant Wolf".to_string(),
            species: AnimalSpecies::GrayWolf,
            position: Vec3::new(-300.0, 50.0, -400.0), // Deep forest
            rewards: vec!["fenrir_fang".to_string(), "wolf_spirit_token".to_string()],
            ..Default::default()
        },
        LegendaryAnimal {
            id: "swamp_king".to_string(),
            name: "The Swamp King".to_string(),
            species: AnimalSpecies::AmericanAlligator,
            position: Vec3::new(200.0, 0.0, -600.0), // Deep swamp
            rewards: vec!["swamp_king_hide".to_string(), "impenetrable_armor".to_string()],
            ..Default::default()
        },
        LegendaryAnimal {
            id: "old_silverback".to_string(),
            name: "Old Silverback".to_string(),
            species: AnimalSpecies::BlackBear,
            position: Vec3::new(-500.0, 100.0, 300.0), // Ancient cave
            rewards: vec!["silverback_pelt".to_string(), "bear_spirit_token".to_string()],
            ..Default::default()
        },
        LegendaryAnimal {
            id: "serpent_mother".to_string(),
            name: "The Serpent Mother".to_string(),
            species: AnimalSpecies::TimberRattlesnake,
            position: Vec3::new(100.0, 30.0, 200.0), // Hidden grotto
            rewards: vec!["serpent_mother_skin".to_string(), "poison_mastery_token".to_string()],
            ..Default::default()
        },
    ]
}

/// Pack relationship with player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackRelationship {
    pub pack_id: u64,
    pub species: AnimalSpecies,
    pub alpha_killed: bool,
    pub members_killed: u32,
    pub fear_level: i32,
    pub last_encounter: f64,
    /// Pack will actively hunt player if true
    pub vendetta: bool,
}

impl PackRelationship {
    /// Check if pack will attack player on sight
    pub fn will_attack(&self) -> bool {
        self.vendetta && !self.alpha_killed
    }

    /// Check if pack will flee from player
    pub fn will_flee(&self) -> bool {
        self.alpha_killed && self.members_killed >= 2
    }
}
