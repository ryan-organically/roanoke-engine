//! Ecology and Nature Balance System
//!
//! A scientifically-grounded ecological simulation modeling the relationship between
//! player actions and the natural environment. Based on real ecological principles:
//! - Carrying capacity and population dynamics
//! - Trophic cascades (predator-prey relationships)
//! - Habitat degradation and recovery
//! - Biodiversity indices
//!
//! No supernatural elements - all consequences flow from ecological mechanisms.

pub mod population;
pub mod habitat;
pub mod consequences;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::animals::types::AnimalSpecies;
use crate::flora::FloraSpecies;

/// Regional ecosystem health tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemRegion {
    pub id: u32,
    pub name: String,
    pub center: [f32; 3],
    pub radius: f32,
    pub biome: BiomeType,

    // Population tracking
    pub fauna_populations: HashMap<AnimalSpecies, PopulationData>,
    pub flora_density: HashMap<FloraSpecies, f32>,

    // Ecosystem metrics
    pub biodiversity_index: f32,    // 0-1, higher is healthier
    pub habitat_quality: f32,       // 0-1, degradation level
    pub prey_availability: f32,     // Affects predator populations
    pub vegetation_cover: f32,      // Affects herbivore carrying capacity

    // Human impact tracking
    pub hunting_pressure: f32,      // Recent hunting activity
    pub harvesting_pressure: f32,   // Recent plant harvesting
    pub disturbance_level: f32,     // Noise, fire, construction
}

impl Default for EcosystemRegion {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Unnamed Region".to_string(),
            center: [0.0, 0.0, 0.0],
            radius: 500.0,
            biome: BiomeType::DeciduousForest,
            fauna_populations: HashMap::new(),
            flora_density: HashMap::new(),
            biodiversity_index: 0.7,
            habitat_quality: 1.0,
            prey_availability: 1.0,
            vegetation_cover: 1.0,
            hunting_pressure: 0.0,
            harvesting_pressure: 0.0,
            disturbance_level: 0.0,
        }
    }
}

impl EcosystemRegion {
    pub fn new(id: u32, name: &str, center: [f32; 3], radius: f32, biome: BiomeType) -> Self {
        let mut region = Self {
            id,
            name: name.to_string(),
            center,
            radius,
            biome,
            ..Default::default()
        };

        // Initialize populations based on biome
        region.initialize_populations();
        region
    }

    fn initialize_populations(&mut self) {
        // Set baseline populations based on biome type
        let species = self.biome.native_species();

        for species in species {
            let carrying_capacity = species.base_carrying_capacity(self.biome);
            self.fauna_populations.insert(species, PopulationData {
                current: (carrying_capacity as f32 * 0.7) as u32,
                carrying_capacity,
                birth_rate: species.birth_rate(),
                death_rate: species.natural_death_rate(),
                migration_pressure: 0.0,
                last_hunted: 0.0,
                total_harvested: 0,
            });
        }
    }

    /// Update ecosystem state based on time passage
    pub fn update(&mut self, delta_days: f32, game_time: f64) {
        // Decay pressure metrics over time
        let decay_rate = 0.05 * delta_days;
        self.hunting_pressure = (self.hunting_pressure - decay_rate).max(0.0);
        self.harvesting_pressure = (self.harvesting_pressure - decay_rate).max(0.0);
        self.disturbance_level = (self.disturbance_level - decay_rate * 2.0).max(0.0);

        // Update populations
        self.update_populations(delta_days, game_time);

        // Update habitat quality
        self.update_habitat_quality(delta_days);

        // Recalculate biodiversity
        self.calculate_biodiversity();
    }

    fn update_populations(&mut self, delta_days: f32, _game_time: f64) {
        // Collect population changes to apply after iteration
        let mut population_changes: HashMap<AnimalSpecies, i32> = HashMap::new();

        for (species, pop) in &self.fauna_populations {
            let mut change = 0i32;

            // Natural reproduction (logistic growth)
            let growth_factor = 1.0 - (pop.current as f32 / pop.carrying_capacity as f32);
            let births = (pop.current as f32 * pop.birth_rate * growth_factor * delta_days / 30.0) as i32;
            change += births;

            // Natural deaths
            let deaths = (pop.current as f32 * pop.death_rate * delta_days / 365.0) as i32;
            change -= deaths;

            // Hunting pressure reduces population
            let hunting_deaths = (self.hunting_pressure * pop.current as f32 * 0.1 * delta_days) as i32;
            change -= hunting_deaths;

            // Habitat quality affects carrying capacity realization
            if self.habitat_quality < 0.5 {
                let stress_deaths = (pop.current as f32 * 0.01 * (1.0 - self.habitat_quality) * delta_days) as i32;
                change -= stress_deaths;
            }

            // Predator-prey dynamics
            if species.is_predator() {
                // Predators decline if prey scarce
                if self.prey_availability < 0.5 {
                    let starvation = (pop.current as f32 * 0.02 * (1.0 - self.prey_availability) * delta_days) as i32;
                    change -= starvation;
                }
            } else {
                // Prey species contribute to prey availability
                // (handled in prey_availability calculation)
            }

            population_changes.insert(*species, change);
        }

        // Apply changes
        for (species, change) in population_changes {
            if let Some(pop) = self.fauna_populations.get_mut(&species) {
                pop.current = ((pop.current as i32 + change).max(0) as u32).min(pop.carrying_capacity * 2);
            }
        }

        // Update prey availability based on herbivore populations
        self.update_prey_availability();
    }

    fn update_prey_availability(&mut self) {
        let mut total_prey = 0u32;
        let mut prey_capacity = 0u32;

        for (species, pop) in &self.fauna_populations {
            if species.is_prey() {
                total_prey += pop.current;
                prey_capacity += pop.carrying_capacity;
            }
        }

        if prey_capacity > 0 {
            self.prey_availability = (total_prey as f32 / prey_capacity as f32).min(1.5);
        }
    }

    fn update_habitat_quality(&mut self, delta_days: f32) {
        // Habitat degrades from disturbance
        let degradation = self.disturbance_level * 0.01 * delta_days;
        self.habitat_quality = (self.habitat_quality - degradation).max(0.1);

        // Habitat slowly recovers if undisturbed
        if self.disturbance_level < 0.1 {
            let recovery = 0.005 * delta_days * self.vegetation_cover;
            self.habitat_quality = (self.habitat_quality + recovery).min(1.0);
        }

        // Vegetation cover affected by harvesting
        self.vegetation_cover = (self.vegetation_cover - self.harvesting_pressure * 0.05 * delta_days).max(0.2);

        // Vegetation recovers over time
        if self.harvesting_pressure < 0.1 {
            self.vegetation_cover = (self.vegetation_cover + 0.01 * delta_days).min(1.0);
        }
    }

    fn calculate_biodiversity(&mut self) {
        // Simple Shannon diversity index approximation
        let total_pop: u32 = self.fauna_populations.values().map(|p| p.current).sum();

        if total_pop == 0 {
            self.biodiversity_index = 0.0;
            return;
        }

        let mut diversity = 0.0f32;
        for pop in self.fauna_populations.values() {
            if pop.current > 0 {
                let proportion = pop.current as f32 / total_pop as f32;
                diversity -= proportion * proportion.ln();
            }
        }

        // Normalize to 0-1 range
        let max_diversity = (self.fauna_populations.len() as f32).ln();
        if max_diversity > 0.0 {
            self.biodiversity_index = (diversity / max_diversity).min(1.0);
        }
    }

    /// Record a hunting event
    pub fn record_hunt(&mut self, species: AnimalSpecies, quantity: u32, game_time: f64) {
        // Increase hunting pressure
        self.hunting_pressure += quantity as f32 * 0.1;
        self.disturbance_level += 0.05;

        // Update population
        if let Some(pop) = self.fauna_populations.get_mut(&species) {
            pop.current = pop.current.saturating_sub(quantity);
            pop.last_hunted = game_time;
            pop.total_harvested += quantity;
        }

        // Trophic cascade: removing predators increases prey
        if species.is_predator() {
            for (prey_species, pop) in &mut self.fauna_populations {
                if prey_species.is_prey() {
                    // Prey becomes more abundant without predation pressure
                    pop.carrying_capacity = (pop.carrying_capacity as f32 * 1.05) as u32;
                }
            }
        }
    }

    /// Record a plant harvesting event
    pub fn record_harvest(&mut self, species: FloraSpecies, quantity: u32) {
        self.harvesting_pressure += quantity as f32 * 0.02;

        // Update flora density
        let current = self.flora_density.entry(species).or_insert(1.0);
        *current = (*current - quantity as f32 * 0.01).max(0.1);

        // Reduced vegetation affects herbivores
        if quantity > 5 {
            self.vegetation_cover = (self.vegetation_cover - 0.01).max(0.2);
        }
    }

    /// Record disturbance (fire, construction, loud activity)
    pub fn record_disturbance(&mut self, intensity: f32) {
        self.disturbance_level = (self.disturbance_level + intensity).min(1.0);

        // High disturbance causes wildlife to flee
        if self.disturbance_level > 0.5 {
            for pop in self.fauna_populations.values_mut() {
                pop.migration_pressure += intensity * 0.5;
            }
        }
    }

    /// Get the ecosystem health rating
    pub fn health_rating(&self) -> EcosystemHealth {
        let score = (self.biodiversity_index + self.habitat_quality + self.vegetation_cover) / 3.0;

        if score >= 0.8 {
            EcosystemHealth::Thriving
        } else if score >= 0.6 {
            EcosystemHealth::Healthy
        } else if score >= 0.4 {
            EcosystemHealth::Stressed
        } else if score >= 0.2 {
            EcosystemHealth::Degraded
        } else {
            EcosystemHealth::Collapsed
        }
    }

    /// Get consequences of current ecosystem state
    pub fn get_active_consequences(&self) -> Vec<EcologicalConsequence> {
        let mut consequences = Vec::new();
        let health = self.health_rating();

        // Population-based consequences
        for (species, pop) in &self.fauna_populations {
            let pop_ratio = pop.current as f32 / pop.carrying_capacity as f32;

            if pop_ratio > 1.5 {
                // Overpopulation
                consequences.push(EcologicalConsequence::Overpopulation(*species));
            } else if pop_ratio < 0.2 && pop.carrying_capacity > 0 {
                // Near extinction
                consequences.push(EcologicalConsequence::LocalExtinction(*species));
            }
        }

        // Habitat consequences
        match health {
            EcosystemHealth::Stressed => {
                consequences.push(EcologicalConsequence::ReducedForaging);
            }
            EcosystemHealth::Degraded => {
                consequences.push(EcologicalConsequence::ReducedForaging);
                consequences.push(EcologicalConsequence::IncreasedPredation);
            }
            EcosystemHealth::Collapsed => {
                consequences.push(EcologicalConsequence::ResourceScarcity);
                consequences.push(EcologicalConsequence::DiseaseOutbreak);
            }
            _ => {}
        }

        // Hunting pressure consequences
        if self.hunting_pressure > 0.7 {
            consequences.push(EcologicalConsequence::WaryWildlife);
        }

        consequences
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiomeType {
    DeciduousForest,
    PineForest,
    Swamp,
    CoastalMarsh,
    Meadow,
    River,
    Mountain,
}

impl BiomeType {
    pub fn native_species(&self) -> Vec<AnimalSpecies> {
        match self {
            Self::DeciduousForest => vec![
                AnimalSpecies::BlackBear,
                AnimalSpecies::EasternCougar,
                AnimalSpecies::GrayWolf,
                AnimalSpecies::WildBoar,
                AnimalSpecies::Bobcat,
                AnimalSpecies::TimberRattlesnake,
                AnimalSpecies::Copperhead,
            ],
            Self::Swamp => vec![
                AnimalSpecies::AmericanAlligator,
                AnimalSpecies::Cottonmouth,
                AnimalSpecies::WildBoar,
                AnimalSpecies::BlackBear,
            ],
            Self::PineForest => vec![
                AnimalSpecies::Bobcat,
                AnimalSpecies::WildBoar,
                AnimalSpecies::Copperhead,
            ],
            Self::CoastalMarsh => vec![
                AnimalSpecies::RedWolf,
                AnimalSpecies::Cottonmouth,
            ],
            Self::Meadow => vec![
                AnimalSpecies::TimberRattlesnake,
                AnimalSpecies::Copperhead,
            ],
            Self::River => vec![
                AnimalSpecies::AmericanAlligator,
                AnimalSpecies::Cottonmouth,
            ],
            Self::Mountain => vec![
                AnimalSpecies::EasternCougar,
                AnimalSpecies::BlackBear,
                AnimalSpecies::TimberRattlesnake,
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationData {
    pub current: u32,
    pub carrying_capacity: u32,
    pub birth_rate: f32,    // Per month
    pub death_rate: f32,    // Per year
    pub migration_pressure: f32,
    pub last_hunted: f64,
    pub total_harvested: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EcosystemHealth {
    Thriving,   // Excellent biodiversity, populations stable
    Healthy,    // Normal ecosystem function
    Stressed,   // Some pressure, reduced productivity
    Degraded,   // Significant damage, cascading effects
    Collapsed,  // Ecosystem failure, minimal wildlife
}

impl EcosystemHealth {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Thriving => "The ecosystem is thriving with abundant wildlife and healthy vegetation",
            Self::Healthy => "The ecosystem is in good health with balanced populations",
            Self::Stressed => "The ecosystem shows signs of stress from human activity",
            Self::Degraded => "The ecosystem is significantly degraded with declining wildlife",
            Self::Collapsed => "The ecosystem has collapsed - wildlife is scarce and vegetation sparse",
        }
    }

    pub fn spawn_modifier(&self) -> f32 {
        match self {
            Self::Thriving => 1.3,
            Self::Healthy => 1.0,
            Self::Stressed => 0.7,
            Self::Degraded => 0.4,
            Self::Collapsed => 0.1,
        }
    }

    pub fn resource_modifier(&self) -> f32 {
        match self {
            Self::Thriving => 1.2,
            Self::Healthy => 1.0,
            Self::Stressed => 0.8,
            Self::Degraded => 0.5,
            Self::Collapsed => 0.2,
        }
    }
}

/// Ecological consequences that affect gameplay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EcologicalConsequence {
    // Population-based
    Overpopulation(AnimalSpecies),  // Too many of a species
    LocalExtinction(AnimalSpecies), // Species very rare

    // Resource-based
    ReducedForaging,    // Less food from foraging
    ResourceScarcity,   // Severe lack of resources

    // Behavioral
    WaryWildlife,       // Animals harder to approach
    IncreasedPredation, // More predator encounters

    // Health
    DiseaseOutbreak,    // Increased disease risk
}

impl EcologicalConsequence {
    pub fn description(&self) -> String {
        match self {
            Self::Overpopulation(species) => {
                format!("{} population has grown beyond sustainable levels", species.name())
            }
            Self::LocalExtinction(species) => {
                format!("{} have become extremely rare in this area", species.name())
            }
            Self::ReducedForaging => {
                "Plant resources are less abundant than normal".to_string()
            }
            Self::ResourceScarcity => {
                "The land is nearly barren - finding food is very difficult".to_string()
            }
            Self::WaryWildlife => {
                "Animals in this area are extremely cautious of humans".to_string()
            }
            Self::IncreasedPredation => {
                "Desperate predators are more likely to attack".to_string()
            }
            Self::DiseaseOutbreak => {
                "Poor conditions have led to increased disease".to_string()
            }
        }
    }
}

// Extensions for AnimalSpecies for ecological calculations
impl AnimalSpecies {
    pub fn is_predator(&self) -> bool {
        matches!(
            self,
            Self::EasternCougar | Self::GrayWolf | Self::RedWolf | Self::Bobcat | Self::AmericanAlligator
        )
    }

    pub fn is_prey(&self) -> bool {
        // In this simplified model, boar is primary prey
        // In reality, there would be more prey species (deer, rabbits, etc.)
        matches!(self, Self::WildBoar)
    }

    pub fn base_carrying_capacity(&self, biome: BiomeType) -> u32 {
        let base = match self {
            Self::BlackBear => 5,
            Self::EasternCougar => 3,
            Self::GrayWolf => 8,
            Self::TimberRattlesnake => 20,
            Self::AmericanAlligator => 10,
            Self::WildBoar => 25,
            Self::Copperhead => 30,
            Self::RedWolf => 6,
            Self::Bobcat => 8,
            Self::Cottonmouth => 15,
        };

        // Adjust by biome suitability
        let multiplier = if self.is_native_to(biome) { 1.0 } else { 0.3 };

        (base as f32 * multiplier) as u32
    }

    pub fn is_native_to(&self, biome: BiomeType) -> bool {
        biome.native_species().contains(self)
    }

    pub fn birth_rate(&self) -> f32 {
        match self {
            Self::BlackBear => 0.05,      // Slow reproduction
            Self::EasternCougar => 0.04,
            Self::GrayWolf => 0.08,
            Self::TimberRattlesnake => 0.1,
            Self::AmericanAlligator => 0.06,
            Self::WildBoar => 0.15,       // Fast reproduction
            Self::Copperhead => 0.12,
            Self::RedWolf => 0.07,
            Self::Bobcat => 0.06,
            Self::Cottonmouth => 0.1,
        }
    }

    pub fn natural_death_rate(&self) -> f32 {
        match self {
            Self::BlackBear => 0.05,
            Self::EasternCougar => 0.08,
            Self::GrayWolf => 0.1,
            Self::TimberRattlesnake => 0.15,
            Self::AmericanAlligator => 0.03, // Long-lived
            Self::WildBoar => 0.2,
            Self::Copperhead => 0.2,
            Self::RedWolf => 0.12,
            Self::Bobcat => 0.1,
            Self::Cottonmouth => 0.15,
        }
    }
}

/// The main ecology manager tracking all regions
#[derive(Debug, Default)]
pub struct EcologyManager {
    pub regions: HashMap<u32, EcosystemRegion>,
    next_region_id: u32,
}

impl EcologyManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new ecosystem region
    pub fn create_region(&mut self, name: &str, center: [f32; 3], radius: f32, biome: BiomeType) -> u32 {
        let id = self.next_region_id;
        self.next_region_id += 1;

        let region = EcosystemRegion::new(id, name, center, radius, biome);
        self.regions.insert(id, region);

        id
    }

    /// Find region containing a position
    pub fn get_region_at(&self, position: [f32; 3]) -> Option<&EcosystemRegion> {
        for region in self.regions.values() {
            let dx = position[0] - region.center[0];
            let dz = position[2] - region.center[2];
            let dist = (dx * dx + dz * dz).sqrt();

            if dist <= region.radius {
                return Some(region);
            }
        }
        None
    }

    /// Find mutable region containing a position
    pub fn get_region_at_mut(&mut self, position: [f32; 3]) -> Option<&mut EcosystemRegion> {
        for region in self.regions.values_mut() {
            let dx = position[0] - region.center[0];
            let dz = position[2] - region.center[2];
            let dist = (dx * dx + dz * dz).sqrt();

            if dist <= region.radius {
                return Some(region);
            }
        }
        None
    }

    /// Update all regions
    pub fn update(&mut self, delta_days: f32, game_time: f64) {
        for region in self.regions.values_mut() {
            region.update(delta_days, game_time);
        }
    }

    /// Get overall world health average
    pub fn world_health(&self) -> f32 {
        if self.regions.is_empty() {
            return 0.7;
        }

        let total: f32 = self.regions.values()
            .map(|r| (r.biodiversity_index + r.habitat_quality) / 2.0)
            .sum();

        total / self.regions.len() as f32
    }
}

// === ADDITIONAL ECOLOGY MANAGER METHODS ===

use glam::Vec3;

impl EcologyManager {
    /// Create with initial regions
    pub fn with_initial_regions() -> Self {
        let mut manager = Self::new();
        manager.create_region("Central Forest", [0.0, 0.0, 0.0], 1000.0, BiomeType::DeciduousForest);
        manager.create_region("Northern Woods", [1500.0, 0.0, 0.0], 800.0, BiomeType::PineForest);
        manager.create_region("Coastal Plain", [0.0, 0.0, 1500.0], 1200.0, BiomeType::CoastalMarsh);
        manager.create_region("Southern Swamp", [-1000.0, 0.0, 500.0], 600.0, BiomeType::Swamp);
        manager
    }

    /// Apply weather effects to ecology
    pub fn apply_weather_effects(&mut self, _weather_modifier: f32, precipitation: f32) {
        for region in self.regions.values_mut() {
            region.vegetation_cover *= 1.0 + (precipitation * 0.01);
            region.vegetation_cover = region.vegetation_cover.clamp(0.0, 1.5);
        }
    }

    /// Update population dynamics
    pub fn update_populations(&mut self, delta_hours: f32) {
        let delta_days = delta_hours / 24.0;
        self.update(delta_days, 0.0);
    }

    /// Daily ecology update
    pub fn daily_update(&mut self) {
        for region in self.regions.values_mut() {
            region.hunting_pressure *= 0.95;
            region.harvesting_pressure *= 0.95;
            if region.disturbance_level < 0.5 {
                region.habitat_quality = (region.habitat_quality + 0.001).min(1.0);
            }
        }
    }

    /// Record a hunting kill
    pub fn record_hunt(&mut self, species: AnimalSpecies, position: Vec3) {
        if let Some(region) = self.get_region_at_mut([position.x, position.y, position.z]) {
            region.record_hunt(species, 1, 0.0);
        }
    }

    /// Get region at Vec3 position
    pub fn get_region_at_vec3(&self, position: Vec3) -> Option<&EcosystemRegion> {
        self.get_region_at([position.x, position.y, position.z])
    }

    /// Get hunting pressures for save data
    pub fn get_hunting_pressures(&self) -> Vec<(u32, f32)> {
        self.regions.iter().map(|(id, r)| (*id, r.hunting_pressure)).collect()
    }

    /// Get ecosystem health scores for save data
    pub fn get_health_scores(&self) -> Vec<(u32, f32)> {
        self.regions.iter().map(|(id, r)| (*id, (r.biodiversity_index + r.habitat_quality) / 2.0)).collect()
    }

    /// Restore hunting pressures from save data
    pub fn restore_hunting_pressures(&mut self, pressures: &[(u32, f32)]) {
        for (id, pressure) in pressures {
            if let Some(region) = self.regions.get_mut(id) {
                region.hunting_pressure = *pressure;
            }
        }
    }

    /// Restore health scores from save data
    pub fn restore_health_scores(&mut self, scores: &[(u32, f32)]) {
        for (id, score) in scores {
            if let Some(region) = self.regions.get_mut(id) {
                region.biodiversity_index = *score;
                region.habitat_quality = *score;
            }
        }
    }
}
