//! Dog Breeding System
//!
//! Manages dog breeding, puppies, and lineage tracking.

use super::taming::{Dog, DogCoat, DogId, DogState};
use super::types::AnimalSpecies;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Breeding cooldown in game-time seconds
const BREEDING_COOLDOWN: f32 = 3600.0; // 1 hour game time

/// Time for puppy to mature into adult
const MATURATION_TIME: f32 = 7200.0; // 2 hours game time

/// Maximum dogs a player can have
const MAX_DOGS: usize = 8;

/// Maximum puppies per litter
const MAX_LITTER_SIZE: u8 = 4;

/// Breeding compatibility result
#[derive(Debug, Clone)]
pub enum BreedingCompatibility {
    Compatible,
    SameDog,
    NotMature,
    OnCooldown { remaining: f32 },
    TooManyBreedings,
    LowHealth,
    TooHungry,
    MaxDogsReached,
}

/// Puppy state (immature dog)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Puppy {
    pub id: DogId,
    pub name: String,
    pub parent1_id: DogId,
    pub parent2_id: DogId,
    pub coat: DogCoat,
    pub age: f32,           // Time since birth
    pub maturity: f32,      // 0.0 - 1.0 progress to adulthood
    pub health: f32,
    pub max_health: f32,
    #[serde(skip)]
    pub position: Vec3,
    pub following_parent: Option<DogId>,
}

impl Puppy {
    /// Create a new puppy
    pub fn new(
        id: DogId,
        name: String,
        parent1: &Dog,
        parent2: &Dog,
        coat: DogCoat,
        position: Vec3,
    ) -> Self {
        // Puppies have lower health than adults
        let avg_health = (parent1.max_health + parent2.max_health) / 2.0;
        let puppy_health = avg_health * 0.4;

        Self {
            id,
            name,
            parent1_id: parent1.id,
            parent2_id: parent2.id,
            coat,
            age: 0.0,
            maturity: 0.0,
            health: puppy_health,
            max_health: puppy_health,
            position,
            following_parent: Some(parent1.id),
        }
    }

    /// Update puppy state
    pub fn update(&mut self, dt: f32) {
        self.age += dt;
        self.maturity = (self.age / MATURATION_TIME).min(1.0);

        // Health increases as puppy matures
        let health_growth = self.maturity * 0.6 + 0.4; // 40% at birth, 100% at maturity
        self.max_health = self.max_health * health_growth;
        self.health = self.health.min(self.max_health);
    }

    /// Check if puppy is ready to become an adult
    pub fn is_mature(&self) -> bool {
        self.maturity >= 1.0
    }

    /// Convert puppy to adult dog
    pub fn mature_into_dog(self, parent1: &Dog, parent2: &Dog, variation_roll: f32) -> Dog {
        Dog::from_breeding(self.id, self.name, parent1, parent2, variation_roll)
    }
}

/// Lineage tracking for a dog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DogLineage {
    pub dog_id: DogId,
    pub parent1: Option<DogId>,
    pub parent2: Option<DogId>,
    pub generation: u8,
    pub offspring: Vec<DogId>,
    pub original_species: AnimalSpecies,
}

/// Breeding event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreedingRecord {
    pub parent1_id: DogId,
    pub parent2_id: DogId,
    pub offspring_ids: Vec<DogId>,
    pub timestamp: f64, // Game time
}

/// Result of a breeding attempt
#[derive(Debug, Clone)]
pub enum BreedingResult {
    Success {
        puppies: Vec<Puppy>,
        litter_size: u8,
    },
    Failed {
        reason: BreedingCompatibility,
    },
}

/// Dog kennel/breeding management
#[derive(Debug, Default)]
pub struct DogKennel {
    /// All owned dogs
    pub dogs: HashMap<DogId, Dog>,
    /// All puppies (not yet mature)
    pub puppies: HashMap<DogId, Puppy>,
    /// Lineage tracking
    pub lineages: HashMap<DogId, DogLineage>,
    /// Breeding history
    pub breeding_history: Vec<BreedingRecord>,
    /// Next ID for new dogs/puppies
    next_id: u64,
}

impl DogKennel {
    pub fn new() -> Self {
        Self {
            dogs: HashMap::new(),
            puppies: HashMap::new(),
            lineages: HashMap::new(),
            breeding_history: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a newly tamed dog to the kennel
    pub fn add_dog(&mut self, dog: Dog) {
        // Create lineage entry for wild-caught dog
        let lineage = DogLineage {
            dog_id: dog.id,
            parent1: None,
            parent2: None,
            generation: 0,
            offspring: Vec::new(),
            original_species: dog.original_species,
        };

        if dog.id.0 >= self.next_id {
            self.next_id = dog.id.0 + 1;
        }

        self.lineages.insert(dog.id, lineage);
        self.dogs.insert(dog.id, dog);
    }

    /// Get total dog count (including puppies)
    pub fn total_count(&self) -> usize {
        self.dogs.len() + self.puppies.len()
    }

    /// Check if two dogs can breed
    pub fn check_breeding_compatibility(
        &self,
        dog1_id: DogId,
        dog2_id: DogId,
    ) -> BreedingCompatibility {
        if dog1_id == dog2_id {
            return BreedingCompatibility::SameDog;
        }

        if self.total_count() >= MAX_DOGS {
            return BreedingCompatibility::MaxDogsReached;
        }

        let dog1 = match self.dogs.get(&dog1_id) {
            Some(d) => d,
            None => return BreedingCompatibility::NotMature,
        };

        let dog2 = match self.dogs.get(&dog2_id) {
            Some(d) => d,
            None => return BreedingCompatibility::NotMature,
        };

        // Check cooldowns
        if dog1.breeding_cooldown > 0.0 {
            return BreedingCompatibility::OnCooldown {
                remaining: dog1.breeding_cooldown,
            };
        }
        if dog2.breeding_cooldown > 0.0 {
            return BreedingCompatibility::OnCooldown {
                remaining: dog2.breeding_cooldown,
            };
        }

        // Check breeding eligibility
        if !dog1.can_breed || !dog2.can_breed {
            return BreedingCompatibility::NotMature;
        }

        // Check breeding limits
        if dog1.times_bred >= 5 || dog2.times_bred >= 5 {
            return BreedingCompatibility::TooManyBreedings;
        }

        // Check health
        if dog1.health < dog1.max_health * 0.5 || dog2.health < dog2.max_health * 0.5 {
            return BreedingCompatibility::LowHealth;
        }

        // Check hunger
        if dog1.hunger > 0.7 || dog2.hunger > 0.7 {
            return BreedingCompatibility::TooHungry;
        }

        BreedingCompatibility::Compatible
    }

    /// Attempt to breed two dogs
    pub fn breed(
        &mut self,
        dog1_id: DogId,
        dog2_id: DogId,
        game_time: f64,
    ) -> BreedingResult {
        // Check compatibility
        let compat = self.check_breeding_compatibility(dog1_id, dog2_id);
        if !matches!(compat, BreedingCompatibility::Compatible) {
            return BreedingResult::Failed { reason: compat };
        }

        // Get breeding position (average of parents)
        let (breed_pos, coat1, coat2, species) = {
            let dog1 = self.dogs.get(&dog1_id).unwrap();
            let dog2 = self.dogs.get(&dog2_id).unwrap();
            (
                (dog1.position + dog2.position) / 2.0,
                dog1.coat,
                dog2.coat,
                dog1.original_species,
            )
        };

        // Determine litter size (1-4 puppies)
        let base_roll = (game_time * 0.618033988749895) % 1.0;
        let litter_size = match base_roll as f32 {
            r if r < 0.3 => 1,
            r if r < 0.6 => 2,
            r if r < 0.85 => 3,
            _ => 4,
        };

        let mut puppies = Vec::new();
        let mut offspring_ids = Vec::new();

        for i in 0..litter_size {
            let puppy_id = DogId(self.next_id);
            self.next_id += 1;

            let variation_roll = ((game_time + i as f64) * 1.618033988749895) % 1.0;
            let coat = DogCoat::breed(coat1, coat2, variation_roll as f32);

            // Offset position slightly for each puppy
            let offset = Vec3::new(
                (i as f32 - 1.5) * 1.0,
                0.0,
                ((i as f32 * 1.5) % 2.0 - 1.0) * 1.0,
            );

            let puppy = {
                let dog1 = self.dogs.get(&dog1_id).unwrap();
                let dog2 = self.dogs.get(&dog2_id).unwrap();
                Puppy::new(
                    puppy_id,
                    format!("Puppy #{}", puppy_id.0),
                    dog1,
                    dog2,
                    coat,
                    breed_pos + offset,
                )
            };

            offspring_ids.push(puppy_id);

            // Create lineage entry
            let lineage = DogLineage {
                dog_id: puppy_id,
                parent1: Some(dog1_id),
                parent2: Some(dog2_id),
                generation: {
                    let g1 = self.lineages.get(&dog1_id).map(|l| l.generation).unwrap_or(0);
                    let g2 = self.lineages.get(&dog2_id).map(|l| l.generation).unwrap_or(0);
                    g1.max(g2) + 1
                },
                offspring: Vec::new(),
                original_species: species,
            };
            self.lineages.insert(puppy_id, lineage);

            puppies.push(puppy);
        }

        // Update parent lineages
        if let Some(lineage) = self.lineages.get_mut(&dog1_id) {
            lineage.offspring.extend(offspring_ids.iter().cloned());
        }
        if let Some(lineage) = self.lineages.get_mut(&dog2_id) {
            lineage.offspring.extend(offspring_ids.iter().cloned());
        }

        // Apply breeding cooldown and increment breeding count
        if let Some(dog1) = self.dogs.get_mut(&dog1_id) {
            dog1.breeding_cooldown = BREEDING_COOLDOWN;
            dog1.times_bred += 1;
            dog1.state = DogState::Breeding;
        }
        if let Some(dog2) = self.dogs.get_mut(&dog2_id) {
            dog2.breeding_cooldown = BREEDING_COOLDOWN;
            dog2.times_bred += 1;
            dog2.state = DogState::Breeding;
        }

        // Record breeding event
        self.breeding_history.push(BreedingRecord {
            parent1_id: dog1_id,
            parent2_id: dog2_id,
            offspring_ids,
            timestamp: game_time,
        });

        // Add puppies to kennel
        for puppy in &puppies {
            self.puppies.insert(puppy.id, puppy.clone());
        }

        BreedingResult::Success {
            puppies,
            litter_size,
        }
    }

    /// Update all dogs and puppies
    pub fn update(&mut self, dt: f32, player_pos: Vec3) {
        // Update adult dogs
        for dog in self.dogs.values_mut() {
            dog.update(dt, player_pos);
        }

        // Update puppies and check for maturation
        let mut matured_puppies = Vec::new();

        for (id, puppy) in self.puppies.iter_mut() {
            puppy.update(dt);

            if puppy.is_mature() {
                matured_puppies.push(*id);
            } else {
                // Puppies follow their parent
                if let Some(parent_id) = puppy.following_parent {
                    if let Some(parent) = self.dogs.get(&parent_id) {
                        let to_parent = parent.position - puppy.position;
                        let dist = to_parent.length();
                        if dist > 3.0 {
                            let dir = to_parent.normalize_or_zero();
                            puppy.position += dir * 20.0 * dt; // Puppies are slower
                        }
                    }
                }
            }
        }

        // Mature puppies into adult dogs
        for puppy_id in matured_puppies {
            if let Some(puppy) = self.puppies.remove(&puppy_id) {
                // Get parents for inheritance calculation
                let (parent1, parent2) = {
                    let p1 = self.dogs.get(&puppy.parent1_id).cloned();
                    let p2 = self.dogs.get(&puppy.parent2_id).cloned();
                    match (p1, p2) {
                        (Some(p1), Some(p2)) => (p1, p2),
                        _ => continue, // Parents missing, skip maturation
                    }
                };

                let variation_roll = (puppy.age * 0.618033988749895) % 1.0;
                let mut adult_dog = puppy.mature_into_dog(&parent1, &parent2, variation_roll as f32);
                adult_dog.can_breed = true; // Now able to breed

                self.dogs.insert(adult_dog.id, adult_dog);
            }
        }
    }

    /// Get a dog by ID
    pub fn get_dog(&self, id: DogId) -> Option<&Dog> {
        self.dogs.get(&id)
    }

    /// Get a mutable dog by ID
    pub fn get_dog_mut(&mut self, id: DogId) -> Option<&mut Dog> {
        self.dogs.get_mut(&id)
    }

    /// Get all dogs near a position
    pub fn dogs_near(&self, pos: Vec3, radius: f32) -> Vec<&Dog> {
        self.dogs
            .values()
            .filter(|d| d.position.distance(pos) < radius)
            .collect()
    }

    /// Get lineage information
    pub fn get_lineage(&self, id: DogId) -> Option<&DogLineage> {
        self.lineages.get(&id)
    }

    /// Get highest generation in the kennel
    pub fn highest_generation(&self) -> u8 {
        self.lineages
            .values()
            .map(|l| l.generation)
            .max()
            .unwrap_or(0)
    }

    /// Get all dogs of a specific generation
    pub fn dogs_of_generation(&self, generation: u8) -> Vec<&Dog> {
        self.dogs
            .values()
            .filter(|d| {
                self.lineages
                    .get(&d.id)
                    .map(|l| l.generation == generation)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Rename a dog or puppy
    pub fn rename(&mut self, id: DogId, new_name: String) -> bool {
        if let Some(dog) = self.dogs.get_mut(&id) {
            dog.name = new_name;
            return true;
        }
        if let Some(puppy) = self.puppies.get_mut(&id) {
            puppy.name = new_name;
            return true;
        }
        false
    }

    /// Remove a dog (death, release, etc.)
    pub fn remove_dog(&mut self, id: DogId) -> Option<Dog> {
        self.dogs.remove(&id)
    }

    /// Get breeding statistics
    pub fn breeding_stats(&self) -> BreedingStats {
        BreedingStats {
            total_dogs: self.dogs.len(),
            total_puppies: self.puppies.len(),
            total_breedings: self.breeding_history.len(),
            highest_generation: self.highest_generation(),
            wild_caught: self.dogs.values().filter(|d| d.generation == 0).count(),
        }
    }
}

/// Summary statistics for breeding
#[derive(Debug, Clone)]
pub struct BreedingStats {
    pub total_dogs: usize,
    pub total_puppies: usize,
    pub total_breedings: usize,
    pub highest_generation: u8,
    pub wild_caught: usize,
}
