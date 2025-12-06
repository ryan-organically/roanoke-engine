//! Wolf Taming and Domesticated Dog System
//!
//! Handles the taming of lone wolves based on player's naturalist skills
//! and manages domesticated dogs including behavior and breeding.

use super::entity::{Animal, AnimalId};
use super::types::AnimalSpecies;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Required skill thresholds for wolf taming
/// Player needs good naturalist skills (hunting, fishing, agriculture, discovery)
#[derive(Debug, Clone, Copy)]
pub struct TamingRequirements {
    /// Minimum combined naturalist score (0-100)
    pub min_naturalist_score: u32,
    /// Minimum hunting skill tier
    pub min_hunting_tier: u8,
    /// Minimum encyclopedia discoveries
    pub min_discoveries: u32,
}

impl Default for TamingRequirements {
    fn default() -> Self {
        Self {
            min_naturalist_score: 25,
            min_hunting_tier: 3,   // Wolf Tracker or better
            min_discoveries: 10,   // At least 10 fauna/flora discovered
        }
    }
}

/// Player's naturalist proficiency for taming calculations
#[derive(Debug, Clone, Default)]
pub struct NaturalistProfile {
    pub hunting_tier: u8,
    pub hunting_points: u32,
    pub fishing_level: u8,
    pub agriculture_level: u8,
    pub discovery_count: u32,
    pub wolf_kills: u32,           // Killing wolves reduces taming ability
    pub animals_fed: u32,          // Feeding animals increases taming ability
}

impl NaturalistProfile {
    /// Calculate the overall naturalist score (0-100)
    pub fn naturalist_score(&self) -> u32 {
        let mut score: u32 = 0;

        // Hunting contributes up to 30 points
        score += (self.hunting_tier as u32 * 3).min(30);

        // Fishing contributes up to 20 points
        score += (self.fishing_level as u32 * 4).min(20);

        // Agriculture contributes up to 20 points
        score += (self.agriculture_level as u32 * 4).min(20);

        // Discovery contributes up to 30 points
        score += (self.discovery_count).min(30);

        // Penalty for wolf kills (each kill reduces by 5, max -25)
        let kill_penalty = (self.wolf_kills * 5).min(25);
        score = score.saturating_sub(kill_penalty);

        // Bonus for feeding animals (each feeding adds 1, max +10)
        score += self.animals_fed.min(10);

        score.min(100)
    }

    /// Check if player meets taming requirements
    pub fn can_tame(&self, requirements: &TamingRequirements) -> bool {
        self.naturalist_score() >= requirements.min_naturalist_score
            && self.hunting_tier >= requirements.min_hunting_tier
            && self.discovery_count >= requirements.min_discoveries
    }

    /// Calculate taming speed multiplier (0.5 - 2.0)
    pub fn taming_speed_multiplier(&self) -> f32 {
        let score = self.naturalist_score() as f32;
        // Score 0-25: 0.5x, 26-50: 1.0x, 51-75: 1.5x, 76-100: 2.0x
        0.5 + (score / 100.0) * 1.5
    }
}

/// Taming interaction types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TamingAction {
    /// Offer food to the wolf
    Feed,
    /// Crouch/stay still to appear non-threatening
    Crouch,
    /// Whistle or make calming sounds
    Whistle,
    /// Throw bait near the wolf
    ThrowBait,
    /// Wait patiently (passive)
    Wait,
}

impl TamingAction {
    /// Get the taming progress bonus for this action
    pub fn progress_bonus(&self) -> f32 {
        match self {
            Self::Feed => 0.15,      // Best action, requires meat
            Self::Crouch => 0.05,    // Good passive action
            Self::Whistle => 0.08,   // Medium action
            Self::ThrowBait => 0.12, // Good but uses resources
            Self::Wait => 0.02,      // Minimal progress
        }
    }

    /// Get the required distance for this action to work
    pub fn required_distance(&self) -> f32 {
        match self {
            Self::Feed => 5.0,       // Very close
            Self::Crouch => 15.0,    // Medium range
            Self::Whistle => 20.0,   // Longer range
            Self::ThrowBait => 12.0, // Medium range
            Self::Wait => 10.0,      // Close-ish
        }
    }
}

/// Result of a taming attempt
#[derive(Debug, Clone)]
pub enum TamingResult {
    /// Progress made toward taming
    Progress { new_progress: f32, message: &'static str },
    /// Wolf is now tamed
    Success { dog_id: DogId },
    /// Taming failed (wolf fled or became aggressive)
    Failed { reason: &'static str },
    /// Player doesn't meet requirements
    NotQualified { reason: &'static str },
    /// Wolf is not in a tameable state
    NotReady { reason: &'static str },
}

/// Unique identifier for domesticated dogs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DogId(pub u64);

/// Coat color/pattern for dogs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DogCoat {
    Gray,
    Black,
    White,
    Brown,
    Brindle,
    Spotted,
    Red,      // From RedWolf
    Mixed,    // Breeding result
}

impl DogCoat {
    /// Get coat from parent species
    pub fn from_species(species: AnimalSpecies) -> Self {
        match species {
            AnimalSpecies::GrayWolf => Self::Gray,
            AnimalSpecies::RedWolf => Self::Red,
            _ => Self::Gray,
        }
    }

    /// Breed two coats together
    pub fn breed(parent1: DogCoat, parent2: DogCoat, variation_roll: f32) -> DogCoat {
        if parent1 == parent2 {
            // Same coat - small chance of variation
            if variation_roll < 0.1 {
                Self::Mixed
            } else {
                parent1
            }
        } else {
            // Different coats - mix or inherit
            if variation_roll < 0.3 {
                parent1
            } else if variation_roll < 0.6 {
                parent2
            } else if variation_roll < 0.8 {
                Self::Mixed
            } else {
                // Random new coat
                match (variation_roll * 10.0) as u8 % 6 {
                    0 => Self::Black,
                    1 => Self::White,
                    2 => Self::Brown,
                    3 => Self::Brindle,
                    4 => Self::Spotted,
                    _ => Self::Mixed,
                }
            }
        }
    }
}

/// Dog behavioral state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DogState {
    #[default]
    Following,    // Following the player
    Sitting,      // Sitting/waiting
    Guarding,     // Guarding a position
    Hunting,      // Hunting nearby game
    Attacking,    // Engaged in combat
    Resting,      // Resting/recovering
    Playing,      // Playing (increases loyalty)
    Breeding,     // In breeding cooldown
}

/// Command the player can give to dogs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DogCommand {
    Follow,
    Sit,
    Guard,
    Hunt,
    Attack,
    Rest,
    Stay,
    Come,
}

/// A domesticated dog (tamed wolf or bred offspring)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dog {
    pub id: DogId,
    pub name: String,

    // Origin
    pub original_species: AnimalSpecies,
    pub coat: DogCoat,
    pub generation: u8,  // 0 = wild-caught, 1+ = bred

    // Stats
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub speed: f32,

    // Position (when active)
    #[serde(skip)]
    pub position: Vec3,
    #[serde(skip)]
    pub velocity: Vec3,

    // Behavior
    pub state: DogState,
    pub loyalty: f32,        // 0.0 - 1.0
    pub energy: f32,         // 0.0 - 1.0, affects performance
    pub hunger: f32,         // 0.0 - 1.0, needs feeding

    // Training
    pub obedience: f32,      // 0.0 - 1.0, command response rate
    pub aggression: f32,     // 0.0 - 1.0, combat eagerness
    pub hunting_skill: f32,  // 0.0 - 1.0, hunting effectiveness

    // Breeding
    pub can_breed: bool,
    pub breeding_cooldown: f32,
    pub times_bred: u8,

    // Timestamps (not serialized)
    #[serde(skip)]
    pub last_fed: Option<Instant>,
    #[serde(skip)]
    pub last_command: Option<(DogCommand, Instant)>,
}

impl Dog {
    /// Create a new dog from a tamed wolf
    pub fn from_tamed_wolf(id: DogId, name: String, wolf: &Animal) -> Self {
        let base_stats = wolf.species.base_stats();

        Self {
            id,
            name,
            original_species: wolf.species,
            coat: DogCoat::from_species(wolf.species),
            generation: 0,
            health: wolf.current_health,
            max_health: wolf.max_health,
            damage: base_stats.damage * 0.8, // Slightly less aggressive
            speed: base_stats.speed,
            position: wolf.position,
            velocity: Vec3::ZERO,
            state: DogState::Following,
            loyalty: 0.3,    // Starts at 30% loyalty
            energy: 0.8,
            hunger: 0.5,
            obedience: 0.4,  // Needs training
            aggression: 0.5,
            hunting_skill: 0.6, // Retains some wild instincts
            can_breed: true,
            breeding_cooldown: 0.0,
            times_bred: 0,
            last_fed: None,
            last_command: None,
        }
    }

    /// Create a puppy from breeding
    pub fn from_breeding(
        id: DogId,
        name: String,
        parent1: &Dog,
        parent2: &Dog,
        variation_roll: f32,
    ) -> Self {
        // Inherit traits from parents with variation
        let avg_health = (parent1.max_health + parent2.max_health) / 2.0;
        let avg_damage = (parent1.damage + parent2.damage) / 2.0;
        let avg_speed = (parent1.speed + parent2.speed) / 2.0;

        // Slight variation (+/- 10%)
        let health_var = 1.0 + (variation_roll - 0.5) * 0.2;
        let damage_var = 1.0 + ((variation_roll * 1.618) % 1.0 - 0.5) * 0.2;
        let speed_var = 1.0 + ((variation_roll * 2.718) % 1.0 - 0.5) * 0.2;

        // Bred dogs have better base obedience
        let base_obedience = ((parent1.obedience + parent2.obedience) / 2.0 + 0.1).min(0.8);

        Self {
            id,
            name,
            original_species: parent1.original_species, // Inherit from first parent
            coat: DogCoat::breed(parent1.coat, parent2.coat, variation_roll),
            generation: parent1.generation.max(parent2.generation) + 1,
            health: avg_health * health_var,
            max_health: avg_health * health_var,
            damage: avg_damage * damage_var,
            speed: avg_speed * speed_var,
            position: parent1.position,
            velocity: Vec3::ZERO,
            state: DogState::Resting,
            loyalty: 0.5, // Puppies are more loyal initially
            energy: 1.0,
            hunger: 0.3,
            obedience: base_obedience,
            aggression: (parent1.aggression + parent2.aggression) / 2.0 * 0.9,
            hunting_skill: (parent1.hunting_skill + parent2.hunting_skill) / 2.0,
            can_breed: false, // Must mature first
            breeding_cooldown: 0.0,
            times_bred: 0,
            last_fed: None,
            last_command: None,
        }
    }

    /// Feed the dog
    pub fn feed(&mut self, food_quality: f32) {
        self.hunger = (self.hunger - food_quality * 0.4).max(0.0);
        self.loyalty = (self.loyalty + 0.05).min(1.0);
        self.energy = (self.energy + food_quality * 0.2).min(1.0);
        self.last_fed = Some(Instant::now());
    }

    /// Issue a command to the dog
    pub fn command(&mut self, cmd: DogCommand) -> bool {
        // Check if dog will obey based on loyalty and obedience
        let obey_chance = self.loyalty * 0.5 + self.obedience * 0.5;

        // Simple deterministic check based on command hash
        let cmd_factor = match cmd {
            DogCommand::Follow => 0.9,  // Easy command
            DogCommand::Sit => 0.8,
            DogCommand::Guard => 0.7,
            DogCommand::Hunt => 0.6,
            DogCommand::Attack => 0.5,
            DogCommand::Rest => 0.9,
            DogCommand::Stay => 0.7,
            DogCommand::Come => 0.85,
        };

        let will_obey = obey_chance >= cmd_factor * 0.5;

        if will_obey {
            self.state = match cmd {
                DogCommand::Follow | DogCommand::Come => DogState::Following,
                DogCommand::Sit | DogCommand::Stay => DogState::Sitting,
                DogCommand::Guard => DogState::Guarding,
                DogCommand::Hunt => DogState::Hunting,
                DogCommand::Attack => DogState::Attacking,
                DogCommand::Rest => DogState::Resting,
            };
            self.last_command = Some((cmd, Instant::now()));

            // Training improves obedience slightly
            self.obedience = (self.obedience + 0.01).min(1.0);
            true
        } else {
            false
        }
    }

    /// Check if dog can breed
    pub fn can_breed_now(&self) -> bool {
        self.can_breed
            && self.breeding_cooldown <= 0.0
            && self.health > self.max_health * 0.5
            && self.hunger < 0.7
            && self.times_bred < 5  // Max 5 breeding cycles
    }

    /// Update dog state
    pub fn update(&mut self, dt: f32, player_pos: Vec3) {
        // Update hunger over time
        self.hunger = (self.hunger + dt * 0.001).min(1.0);

        // Update energy based on activity
        let energy_drain = match self.state {
            DogState::Resting => -0.1,    // Recover energy
            DogState::Sitting => -0.02,   // Slight recovery
            DogState::Following => 0.01,
            DogState::Guarding => 0.005,
            DogState::Hunting => 0.03,
            DogState::Attacking => 0.05,
            DogState::Playing => 0.02,
            DogState::Breeding => 0.0,
        };
        self.energy = (self.energy - energy_drain * dt).clamp(0.0, 1.0);

        // Update breeding cooldown
        if self.breeding_cooldown > 0.0 {
            self.breeding_cooldown -= dt;
        }

        // Auto-rest if exhausted
        if self.energy < 0.1 {
            self.state = DogState::Resting;
        }

        // Loyalty decay if hungry
        if self.hunger > 0.8 {
            self.loyalty = (self.loyalty - dt * 0.001).max(0.0);
        }

        // Movement based on state
        match self.state {
            DogState::Following => {
                let to_player = player_pos - self.position;
                let dist = to_player.length();
                if dist > 5.0 {
                    // Move toward player
                    let dir = to_player.normalize_or_zero();
                    self.velocity = dir * self.speed * 0.8;
                } else if dist < 2.0 {
                    // Too close, slow down
                    self.velocity = Vec3::ZERO;
                } else {
                    // Maintain distance
                    self.velocity = self.velocity * 0.9;
                }
            }
            DogState::Sitting | DogState::Guarding | DogState::Resting => {
                self.velocity = Vec3::ZERO;
            }
            _ => {
                // Other states handled elsewhere
            }
        }

        // Apply velocity
        self.position += self.velocity * dt;
    }

    /// Get damage output considering state and stats
    pub fn combat_damage(&self) -> f32 {
        let mut damage = self.damage;

        // Loyalty bonus
        damage *= 0.8 + self.loyalty * 0.4;

        // Energy penalty
        if self.energy < 0.3 {
            damage *= 0.5;
        }

        // Hunger penalty
        if self.hunger > 0.7 {
            damage *= 0.7;
        }

        damage
    }
}

/// Taming system manager
pub struct TamingSystem {
    pub requirements: TamingRequirements,
    next_dog_id: u64,
}

impl TamingSystem {
    pub fn new() -> Self {
        Self {
            requirements: TamingRequirements::default(),
            next_dog_id: 1,
        }
    }

    /// Attempt to perform a taming action on a wolf
    pub fn attempt_taming(
        &mut self,
        wolf: &mut Animal,
        action: TamingAction,
        player_pos: Vec3,
        profile: &NaturalistProfile,
    ) -> TamingResult {
        // Check if player meets requirements
        if !profile.can_tame(&self.requirements) {
            return TamingResult::NotQualified {
                reason: "Insufficient naturalist skills",
            };
        }

        // Check if wolf is tameable (lone wolf)
        if !wolf.can_be_tamed() {
            return TamingResult::NotReady {
                reason: "This wolf cannot be tamed",
            };
        }

        // Check distance
        let dist = wolf.position.distance(player_pos);
        if dist > action.required_distance() {
            return TamingResult::NotReady {
                reason: "Too far away",
            };
        }

        // Check if wolf is in appropriate state (curious or sniffing)
        let is_receptive = matches!(
            wolf.behavior_state,
            super::behavior::BehaviorState::Curious(_) | super::behavior::BehaviorState::Approaching
        );

        if !is_receptive && wolf.curiosity_level < 0.5 {
            return TamingResult::NotReady {
                reason: "Wolf is not curious enough",
            };
        }

        // Calculate progress
        let base_progress = action.progress_bonus();
        let speed_mult = profile.taming_speed_multiplier();
        let curiosity_mult = 0.5 + wolf.curiosity_level * 0.5;
        let progress = base_progress * speed_mult * curiosity_mult;

        wolf.advance_taming(progress);
        wolf.record_positive_interaction();

        // Check if tamed
        if wolf.is_tamed() {
            let dog_id = DogId(self.next_dog_id);
            self.next_dog_id += 1;
            return TamingResult::Success { dog_id };
        }

        let message = match action {
            TamingAction::Feed => "The wolf accepts the food cautiously",
            TamingAction::Crouch => "The wolf seems less threatened",
            TamingAction::Whistle => "The wolf's ears perk up",
            TamingAction::ThrowBait => "The wolf investigates the bait",
            TamingAction::Wait => "The wolf watches you curiously",
        };

        TamingResult::Progress {
            new_progress: wolf.taming_progress,
            message,
        }
    }

    /// Convert a fully tamed wolf to a dog
    pub fn create_dog_from_wolf(&mut self, wolf: &Animal, name: String) -> Option<Dog> {
        if !wolf.is_tamed() {
            return None;
        }

        let dog_id = DogId(self.next_dog_id);
        self.next_dog_id += 1;

        Some(Dog::from_tamed_wolf(dog_id, name, wolf))
    }
}

impl Default for TamingSystem {
    fn default() -> Self {
        Self::new()
    }
}
