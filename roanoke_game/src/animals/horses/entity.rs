//! Horse Entity - Runtime state for individual horses
//!
//! Represents a single horse in the game world, whether wild, being tamed,
//! or fully owned by the player.

use super::encephalon::{
    HorseEncephalon, EnvironmentContext, ThreatInfo, DecisionState,
    EmotionalState, PlayerInteraction,
};
use super::perks::HorsePerkTree;
use super::taming::TamingProgress;
use super::training::{TrainingSkills, TrainingSkill, SkillLevel};
use super::types::{
    HorseSpecies, HorseCoat, HorseGender, HorseAge, HorseStats,
    HerdType, HorseEquipmentSlot, EquipmentQuality,
};
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Unique identifier for horses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HorseId(pub u64);

/// Unique identifier for herds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HerdId(pub u64);

/// Horse ownership state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnershipState {
    /// Wild horse, not tamed
    Wild,
    /// Currently being tamed
    BeingTamed,
    /// Owned by player
    Owned,
    /// Temporarily borrowed/rented
    Borrowed,
    /// Stabled away
    Stabled,
}

/// Mount state when player is riding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MountState {
    #[default]
    NotMounted,
    Mounting,
    Mounted,
    Dismounting,
}

/// Movement gait for horses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Gait {
    #[default]
    Standing,
    Walking,
    Trotting,
    Cantering,
    Galloping,
    Swimming,
    Rearing,
    Bucking,
}

impl Gait {
    /// Get speed multiplier for this gait
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            Self::Standing => 0.0,
            Self::Walking => 0.3,
            Self::Trotting => 0.5,
            Self::Cantering => 0.75,
            Self::Galloping => 1.0,
            Self::Swimming => 0.4,
            Self::Rearing | Self::Bucking => 0.0,
        }
    }

    /// Get stamina drain rate
    pub fn stamina_drain(&self) -> f32 {
        match self {
            Self::Standing => 0.0,
            Self::Walking => 0.002,
            Self::Trotting => 0.005,
            Self::Cantering => 0.01,
            Self::Galloping => 0.025,
            Self::Swimming => 0.02,
            Self::Rearing | Self::Bucking => 0.015,
        }
    }
}

/// Equipment item on a horse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorseEquipment {
    pub slot: HorseEquipmentSlot,
    pub name: String,
    pub quality: EquipmentQuality,
    pub durability: f32,
    pub max_durability: f32,
    pub stat_bonuses: HashMap<String, f32>,
}

/// Complete runtime state for a horse
#[derive(Debug)]
pub struct Horse {
    // === Identity ===
    pub id: HorseId,
    pub name: String,
    pub species: HorseSpecies,
    pub coat: HorseCoat,
    pub gender: HorseGender,
    pub age: HorseAge,

    // === Physical State ===
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub in_water: bool,
    pub current_gait: Gait,

    // === Vital Stats ===
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub max_stamina: f32,

    // === AI Brain ===
    pub encephalon: HorseEncephalon,

    // === Ownership & Bonding ===
    pub ownership: OwnershipState,
    pub mount_state: MountState,
    pub bond_level: f32,            // 0.0-1.0 overall bond with player
    pub trust_level: f32,           // 0.0-1.0 trust in player
    pub respect_level: f32,         // 0.0-1.0 respect for player commands
    pub taming_progress: Option<TamingProgress>,

    // === Training & Skills ===
    pub training_skills: TrainingSkills,
    pub experience: u32,
    pub level: u8,

    // === Perks ===
    pub perk_tree: HorsePerkTree,
    pub perk_points: u8,

    // === Equipment ===
    pub equipment: HashMap<HorseEquipmentSlot, HorseEquipment>,
    pub saddled: bool,
    pub bridled: bool,

    // === Herd State ===
    pub herd_id: Option<HerdId>,
    pub herd_role: HerdRole,

    // === Spawn Info ===
    pub spawn_chunk: (i32, i32),
    pub home_position: Vec3,
    pub territory_radius: f32,

    // === Timestamps ===
    #[allow(dead_code)]
    pub spawn_time: Instant,
    pub last_fed: Option<Instant>,
    pub last_groomed: Option<Instant>,
    pub last_ridden: Option<Instant>,

    // === Cached Stats ===
    effective_stats: HorseStats,
    stats_dirty: bool,
}

/// Role within a herd
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HerdRole {
    #[default]
    Member,
    Lead,
    Stallion,
    Mare,
    Foal,
    Scout,
}

impl Horse {
    /// Create a new wild horse
    pub fn new_wild(
        id: HorseId,
        species: HorseSpecies,
        position: Vec3,
        chunk: (i32, i32),
        personality_seed: u64,
    ) -> Self {
        let base_stats = species.base_stats();
        let coats = species.available_coats();
        let coat = coats[(personality_seed as usize) % coats.len()];

        // Determine gender and age from seed
        let gender = if personality_seed % 3 == 0 {
            HorseGender::Stallion
        } else {
            HorseGender::Mare
        };
        let age = match personality_seed % 10 {
            0 => HorseAge::Foal,
            1 => HorseAge::Yearling,
            2..=4 => HorseAge::Young,
            5..=8 => HorseAge::Prime,
            _ => HorseAge::Mature,
        };

        let age_mods = age.stat_multipliers();
        let (strength_mod, speed_mod, _) = gender.stat_modifiers();

        Self {
            id,
            name: format!("Wild {}", species.name()),
            species,
            coat,
            gender,
            age,

            position,
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            on_ground: true,
            in_water: false,
            current_gait: Gait::Standing,

            health: base_stats.health * age_mods.health,
            max_health: base_stats.health * age_mods.health,
            stamina: base_stats.stamina * age_mods.stamina,
            max_stamina: base_stats.stamina * age_mods.stamina,

            encephalon: HorseEncephalon::new(species, personality_seed),

            ownership: OwnershipState::Wild,
            mount_state: MountState::NotMounted,
            bond_level: 0.0,
            trust_level: 0.0,
            respect_level: 0.0,
            taming_progress: None,

            training_skills: TrainingSkills::new(),
            experience: 0,
            level: 1,

            perk_tree: HorsePerkTree::new(),
            perk_points: 0,

            equipment: HashMap::new(),
            saddled: false,
            bridled: false,

            herd_id: None,
            herd_role: HerdRole::Member,

            spawn_chunk: chunk,
            home_position: position,
            territory_radius: 100.0,

            spawn_time: Instant::now(),
            last_fed: None,
            last_groomed: None,
            last_ridden: None,

            effective_stats: HorseStats {
                health: base_stats.health * age_mods.health,
                stamina: base_stats.stamina * age_mods.stamina,
                speed: base_stats.speed * age_mods.speed * speed_mod,
                acceleration: base_stats.acceleration,
                strength: base_stats.strength * age_mods.strength * strength_mod,
                agility: base_stats.agility,
                swim_speed: base_stats.swim_speed,
                carry_capacity: base_stats.carry_capacity * age_mods.strength,
                courage: base_stats.courage,
            },
            stats_dirty: false,
        }
    }

    /// Update horse state
    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Option<Vec3>,
        nearby_horses: &[Vec3],
        threats: &[ThreatInfo],
        environment: &EnvironmentContext,
    ) {
        // Update AI brain
        self.encephalon.update(
            dt,
            self.position,
            player_pos,
            nearby_horses,
            threats,
            environment,
        );

        // Update stamina based on gait
        let drain = self.current_gait.stamina_drain() * dt;
        self.stamina = (self.stamina - drain).max(0.0);

        // Stamina recovery when standing/walking
        if matches!(self.current_gait, Gait::Standing | Gait::Walking) {
            self.stamina = (self.stamina + 0.01 * dt).min(self.max_stamina);
        }

        // Process behavior
        self.process_behavior(dt, player_pos);

        // Update position based on velocity
        self.position += self.velocity * dt;

        // Recalculate stats if needed
        if self.stats_dirty {
            self.recalculate_stats();
        }
    }

    /// Process AI behavior into movement
    fn process_behavior(&mut self, dt: f32, player_pos: Option<Vec3>) {
        let decision = self.encephalon.decision_state;
        let emotion = self.encephalon.emotional_state;

        // Determine gait based on decision state
        self.current_gait = match decision {
            DecisionState::Idle | DecisionState::Resting => Gait::Standing,
            DecisionState::Grazing | DecisionState::SeekingWater => Gait::Walking,
            DecisionState::Alert | DecisionState::Assessing => Gait::Standing,
            DecisionState::Following | DecisionState::Investigating => Gait::Trotting,
            DecisionState::SeekingHerd | DecisionState::PlayingWithHerd => Gait::Cantering,
            DecisionState::PreparingToFlee => Gait::Walking,
            DecisionState::Fleeing => Gait::Galloping,
            DecisionState::Defending => Gait::Trotting,
        };

        // Calculate movement direction based on decision
        let speed = self.effective_speed();
        let gait_mult = self.current_gait.speed_multiplier();

        match decision {
            DecisionState::Fleeing => {
                // Flee from threats
                if let Some(threat) = &self.encephalon.awareness.highest_threat {
                    let away_dir = (self.position - threat.position).normalize_or_zero();
                    self.velocity = away_dir * speed * gait_mult;
                    self.look_at(self.position + away_dir);
                }
            }
            DecisionState::Following => {
                // Follow player if bonded
                if let Some(player) = player_pos {
                    let to_player = player - self.position;
                    let dist = to_player.length();
                    if dist > 5.0 {
                        let dir = to_player.normalize_or_zero();
                        self.velocity = dir * speed * gait_mult;
                        self.look_at(player);
                    } else {
                        self.velocity = Vec3::ZERO;
                        self.current_gait = Gait::Standing;
                    }
                }
            }
            DecisionState::Grazing => {
                // Slow wandering while grazing
                self.encephalon.needs.feed(0.001 * dt);
                if rand_deterministic(self.id.0, 0.02 * dt) {
                    let angle = rand_angle(self.id.0);
                    let wander_dir = Vec3::new(angle.cos(), 0.0, angle.sin());
                    self.velocity = wander_dir * speed * 0.2;
                } else {
                    self.velocity = Vec3::ZERO;
                }
            }
            DecisionState::SeekingHerd => {
                // Move toward nearest horse
                if let Some(dist) = self.encephalon.awareness.nearest_horse_distance {
                    if dist > 20.0 {
                        // Would need actual position to move toward
                        let wander_angle = rand_angle(self.id.0);
                        let dir = Vec3::new(wander_angle.cos(), 0.0, wander_angle.sin());
                        self.velocity = dir * speed * gait_mult;
                    }
                }
            }
            _ => {
                // Slow down when idle/standing
                self.velocity = self.velocity * 0.9;
            }
        }

        // Emotional overrides for special behaviors
        if emotion == EmotionalState::Panicked && self.current_gait != Gait::Galloping {
            self.current_gait = Gait::Galloping;
            if self.velocity.length_squared() < 0.1 {
                // Pick random flee direction
                let angle = rand_angle(self.id.0);
                let flee_dir = Vec3::new(angle.cos(), 0.0, angle.sin());
                self.velocity = flee_dir * speed;
            }
        }
    }

    /// Calculate effective speed with all modifiers
    pub fn effective_speed(&self) -> f32 {
        let mut speed = self.effective_stats.speed;

        // Stamina affects speed
        if self.stamina < self.max_stamina * 0.2 {
            speed *= 0.6;
        } else if self.stamina < self.max_stamina * 0.5 {
            speed *= 0.85;
        }

        // Health affects speed
        if self.health < self.max_health * 0.3 {
            speed *= 0.5;
        }

        // Bond affects willingness to perform
        if self.ownership == OwnershipState::Owned {
            speed *= 0.9 + self.bond_level * 0.2;
        }

        // Emotional state affects speed
        speed *= self.encephalon.movement_modifier();

        speed
    }

    /// Recalculate effective stats from base + modifiers
    fn recalculate_stats(&mut self) {
        let base = self.species.base_stats();
        let age_mods = self.age.stat_multipliers();
        let (strength_mod, speed_mod, _) = self.gender.stat_modifiers();

        self.effective_stats = HorseStats {
            health: base.health * age_mods.health,
            stamina: base.stamina * age_mods.stamina,
            speed: base.speed * age_mods.speed * speed_mod,
            acceleration: base.acceleration,
            strength: base.strength * age_mods.strength * strength_mod,
            agility: base.agility,
            swim_speed: base.swim_speed,
            carry_capacity: base.carry_capacity * age_mods.strength,
            courage: base.courage,
        };

        // Apply equipment bonuses
        for equip in self.equipment.values() {
            let quality_mult = equip.quality.bonus_multiplier();
            for (stat, bonus) in &equip.stat_bonuses {
                match stat.as_str() {
                    "speed" => self.effective_stats.speed += bonus * quality_mult,
                    "stamina" => self.effective_stats.stamina += bonus * quality_mult,
                    "strength" => self.effective_stats.strength += bonus * quality_mult,
                    "health" => self.effective_stats.health += bonus * quality_mult,
                    _ => {}
                }
            }
        }

        // Apply training skill bonuses
        let skills = &self.training_skills;
        if let Some(level) = skills.get_skill(TrainingSkill::Endurance) {
            self.effective_stats.stamina *= 1.0 + level.bonus();
        }
        if let Some(level) = skills.get_skill(TrainingSkill::Speed) {
            self.effective_stats.speed *= 1.0 + level.bonus();
        }
        if let Some(level) = skills.get_skill(TrainingSkill::Strength) {
            self.effective_stats.strength *= 1.0 + level.bonus();
        }

        // Apply perk bonuses
        self.effective_stats = self.perk_tree.apply_stat_bonuses(self.effective_stats);

        self.stats_dirty = false;
    }

    /// Face toward a target position
    pub fn look_at(&mut self, target: Vec3) {
        let direction = (target - self.position).normalize_or_zero();
        if direction.length_squared() > 0.001 {
            let yaw = direction.z.atan2(direction.x);
            self.rotation = Quat::from_rotation_y(-yaw + std::f32::consts::FRAC_PI_2);
        }
    }

    /// Feed the horse
    pub fn feed(&mut self, food_quality: f32) {
        self.encephalon.needs.feed(food_quality);
        self.last_fed = Some(Instant::now());

        // Positive interaction
        self.encephalon.record_player_interaction(PlayerInteraction::Fed, true);

        // Improve bond
        self.bond_level = (self.bond_level + 0.02 * food_quality).min(1.0);
        self.trust_level = (self.trust_level + 0.01 * food_quality).min(1.0);
    }

    /// Groom the horse
    pub fn groom(&mut self) {
        self.last_groomed = Some(Instant::now());
        self.encephalon.record_player_interaction(PlayerInteraction::Groomed, true);
        self.bond_level = (self.bond_level + 0.03).min(1.0);
        self.trust_level = (self.trust_level + 0.02).min(1.0);
    }

    /// Pet/calm the horse
    pub fn pet(&mut self) -> bool {
        let trust = self.encephalon.player_trust();
        if trust > 0.2 || self.ownership == OwnershipState::Owned {
            self.encephalon.record_player_interaction(PlayerInteraction::Petted, true);
            self.bond_level = (self.bond_level + 0.01).min(1.0);
            self.encephalon.emotion_weights.calm += 0.05;
            self.encephalon.emotion_weights.fear =
                (self.encephalon.emotion_weights.fear - 0.02).max(0.0);
            true
        } else {
            self.encephalon.record_player_interaction(PlayerInteraction::Petted, false);
            false
        }
    }

    /// Attempt to mount the horse
    pub fn mount(&mut self) -> bool {
        if !self.saddled && self.ownership != OwnershipState::Owned {
            return false;
        }
        if !self.age.can_ride() {
            return false;
        }

        let readiness = self.encephalon.taming_readiness();
        let can_mount = self.ownership == OwnershipState::Owned
            || (readiness > 0.6 && self.trust_level > 0.4);

        if can_mount {
            self.mount_state = MountState::Mounting;
            self.encephalon.record_player_interaction(PlayerInteraction::Mounted, true);
            self.last_ridden = Some(Instant::now());
            true
        } else {
            // Horse may buck or flee
            self.encephalon.record_player_interaction(PlayerInteraction::Mounted, false);
            if self.encephalon.personality.stubbornness > 0.5 {
                self.current_gait = Gait::Bucking;
            } else {
                self.current_gait = Gait::Galloping;
                self.encephalon.decision_state = DecisionState::Fleeing;
            }
            false
        }
    }

    /// Dismount from horse
    pub fn dismount(&mut self) {
        self.mount_state = MountState::Dismounting;
        self.encephalon.record_player_interaction(PlayerInteraction::Dismounted, true);
    }

    /// Equip an item on the horse
    pub fn equip(&mut self, equipment: HorseEquipment) {
        let slot = equipment.slot;
        self.equipment.insert(slot, equipment);
        self.stats_dirty = true;

        // Update convenience flags
        self.saddled = self.equipment.contains_key(&HorseEquipmentSlot::Saddle);
        self.bridled = self.equipment.contains_key(&HorseEquipmentSlot::Bridle);
    }

    /// Remove equipment from a slot
    pub fn unequip(&mut self, slot: HorseEquipmentSlot) -> Option<HorseEquipment> {
        let item = self.equipment.remove(&slot);
        if item.is_some() {
            self.stats_dirty = true;
            self.saddled = self.equipment.contains_key(&HorseEquipmentSlot::Saddle);
            self.bridled = self.equipment.contains_key(&HorseEquipmentSlot::Bridle);
        }
        item
    }

    /// Add experience and potentially level up
    pub fn add_experience(&mut self, amount: u32) {
        let multiplier = self.age.stat_multipliers().experience_gain;
        self.experience += (amount as f32 * multiplier) as u32;

        // Level up thresholds
        let next_level_xp = (self.level as u32) * 1000;
        if self.experience >= next_level_xp && self.level < 50 {
            self.level += 1;
            self.perk_points += 1;
            self.experience -= next_level_xp;
        }
    }

    /// Get current effective stats
    pub fn stats(&self) -> &HorseStats {
        &self.effective_stats
    }

    /// Check if horse is alive
    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    /// Take damage
    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
        self.encephalon.emotion_weights.fear += 0.2;
        self.encephalon.emotion_weights.calm = 0.0;
    }

    /// Heal the horse
    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
        self.encephalon.record_player_interaction(PlayerInteraction::Healed, true);
        self.bond_level = (self.bond_level + 0.02).min(1.0);
    }

    /// Get orb render color (species + coat blend)
    pub fn orb_color(&self) -> [f32; 3] {
        let species_color = self.species.orb_color();
        let coat_color = self.coat.color();
        // Blend 30% species, 70% coat
        [
            species_color[0] * 0.3 + coat_color[0] * 0.7,
            species_color[1] * 0.3 + coat_color[1] * 0.7,
            species_color[2] * 0.3 + coat_color[2] * 0.7,
        ]
    }

    /// Get orb scale for rendering
    pub fn orb_scale(&self) -> f32 {
        let base = self.species.orb_scale();
        let age_scale = match self.age {
            HorseAge::Foal => 0.5,
            HorseAge::Yearling => 0.7,
            HorseAge::Young => 0.9,
            HorseAge::Prime => 1.0,
            HorseAge::Mature => 1.0,
            HorseAge::Elder => 0.95,
        };
        base * age_scale
    }
}

/// Deterministic random for stable behavior
fn rand_deterministic(seed: u64, probability: f32) -> bool {
    let hash = seed.wrapping_mul(0x9E3779B97F4A7C15);
    let mixed = (hash ^ (hash >> 33)).wrapping_mul(0xFF51AFD7ED558CCD);
    let final_hash = (mixed ^ (mixed >> 33)).wrapping_mul(0xC4CEB9FE1A85EC53);
    (final_hash as f32 / u64::MAX as f32) < probability
}

/// Get deterministic angle from seed
fn rand_angle(seed: u64) -> f32 {
    let hash = seed.wrapping_mul(0x5851F42D4C957F2D);
    (hash as f32 / u64::MAX as f32) * std::f32::consts::TAU
}
