//! Plant growth and lifecycle system
//!
//! Models realistic plant growth stages, seasonal changes, and environmental responses.

use serde::{Deserialize, Serialize};
use super::{FloraSpecies, FloraHabitat};
use crate::encyclopedia::Season;

/// Growth stage of a plant instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GrowthStage {
    #[default]
    Seed,
    Seedling,
    Juvenile,
    Mature,
    Flowering,
    Fruiting,
    Dormant,
    Dead,
}

impl GrowthStage {
    pub fn can_harvest(&self) -> bool {
        matches!(self, Self::Mature | Self::Flowering | Self::Fruiting)
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Seed => "Seed",
            Self::Seedling => "Seedling",
            Self::Juvenile => "Young",
            Self::Mature => "Mature",
            Self::Flowering => "Flowering",
            Self::Fruiting => "Fruiting",
            Self::Dormant => "Dormant",
            Self::Dead => "Dead",
        }
    }
}

/// Individual plant instance in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantInstance {
    pub species: FloraSpecies,
    pub position: [f32; 3],
    pub stage: GrowthStage,
    /// Age in game days
    pub age_days: f32,
    /// Health 0-100
    pub health: f32,
    /// Growth progress to next stage (0-1)
    pub growth_progress: f32,
    /// Times harvested (affects regrowth)
    pub harvest_count: u32,
    /// Unique ID for tracking
    pub id: u64,
    /// Scale modifier (natural variation)
    pub scale: f32,
    /// Rotation for visual variety
    pub rotation: f32,
}

impl PlantInstance {
    pub fn new(species: FloraSpecies, position: [f32; 3], id: u64) -> Self {
        Self {
            species,
            position,
            stage: GrowthStage::Mature, // Most spawned plants are mature
            age_days: 100.0, // Assume established
            health: 100.0,
            growth_progress: 0.0,
            harvest_count: 0,
            id,
            scale: 0.9 + rand_float() * 0.2, // 0.9-1.1 variation
            rotation: rand_float() * std::f32::consts::TAU,
        }
    }

    pub fn new_seedling(species: FloraSpecies, position: [f32; 3], id: u64) -> Self {
        Self {
            species,
            position,
            stage: GrowthStage::Seedling,
            age_days: 0.0,
            health: 100.0,
            growth_progress: 0.0,
            harvest_count: 0,
            id,
            scale: 0.3,
            rotation: rand_float() * std::f32::consts::TAU,
        }
    }

    /// Update plant growth based on time and conditions
    pub fn update(&mut self, delta_days: f32, season: Season, weather_quality: f32) {
        if self.stage == GrowthStage::Dead {
            return;
        }

        self.age_days += delta_days;

        // Check if season is appropriate for growth
        let growing_seasons = self.species.growing_seasons();
        let is_growing_season = growing_seasons.contains(&season);

        if is_growing_season && self.health > 0.0 {
            // Growth rate affected by health and weather
            let growth_rate = self.calculate_growth_rate(weather_quality);
            self.growth_progress += growth_rate * delta_days;

            // Advance stage if progress complete
            if self.growth_progress >= 1.0 {
                self.advance_stage(season);
                self.growth_progress = 0.0;
            }
        } else if !is_growing_season {
            // Enter dormancy in off-season (for non-evergreens)
            self.handle_seasonal_change(season);
        }

        // Health decay if conditions poor
        if weather_quality < 0.3 {
            self.health -= delta_days * 2.0;
        }

        if self.health <= 0.0 {
            self.stage = GrowthStage::Dead;
        }
    }

    fn calculate_growth_rate(&self, weather_quality: f32) -> f32 {
        // Base growth rate (days to advance one stage)
        let base_days = match self.species.category() {
            crate::encyclopedia::PlantCategory::Tree => 365.0, // Trees are slow
            crate::encyclopedia::PlantCategory::Shrub => 90.0,
            crate::encyclopedia::PlantCategory::Herb => 30.0,
            crate::encyclopedia::PlantCategory::Fungus => 7.0,
            crate::encyclopedia::PlantCategory::Aquatic => 20.0,
            crate::encyclopedia::PlantCategory::Vine => 45.0,
            crate::encyclopedia::PlantCategory::Crop => 60.0,
            _ => 30.0,
        };

        let health_modifier = self.health / 100.0;
        let weather_modifier = 0.5 + weather_quality * 0.5;

        (1.0 / base_days) * health_modifier * weather_modifier
    }

    fn advance_stage(&mut self, season: Season) {
        self.stage = match self.stage {
            GrowthStage::Seed => GrowthStage::Seedling,
            GrowthStage::Seedling => GrowthStage::Juvenile,
            GrowthStage::Juvenile => GrowthStage::Mature,
            GrowthStage::Mature => {
                if self.can_flower(season) {
                    GrowthStage::Flowering
                } else {
                    GrowthStage::Mature
                }
            }
            GrowthStage::Flowering => GrowthStage::Fruiting,
            GrowthStage::Fruiting => GrowthStage::Mature, // Cycle back
            GrowthStage::Dormant => GrowthStage::Mature,
            GrowthStage::Dead => GrowthStage::Dead,
        };

        // Update scale for growth
        self.scale = match self.stage {
            GrowthStage::Seedling => 0.3,
            GrowthStage::Juvenile => 0.6,
            GrowthStage::Mature | GrowthStage::Flowering | GrowthStage::Fruiting => {
                0.9 + rand_float() * 0.2
            }
            _ => self.scale,
        };
    }

    fn can_flower(&self, season: Season) -> bool {
        // Most plants flower in spring/summer
        matches!(season, Season::Spring | Season::Summer)
    }

    fn handle_seasonal_change(&mut self, season: Season) {
        match self.species.category() {
            crate::encyclopedia::PlantCategory::Tree => {
                // Deciduous trees go dormant in winter
                if season == Season::Winter && self.stage != GrowthStage::Dormant {
                    self.stage = GrowthStage::Dormant;
                }
            }
            crate::encyclopedia::PlantCategory::Herb => {
                // Many herbs die back in winter
                if season == Season::Winter {
                    self.stage = GrowthStage::Dormant;
                }
            }
            _ => {}
        }
    }

    /// Check if this plant can be harvested now
    pub fn can_harvest(&self) -> bool {
        self.stage.can_harvest() && self.health > 20.0
    }

    /// Harvest the plant, returning success and damaging the plant
    pub fn harvest(&mut self) -> HarvestResult {
        if !self.can_harvest() {
            return HarvestResult::Failed;
        }

        self.harvest_count += 1;

        // Determine yield quality based on stage and health
        let quality = match self.stage {
            GrowthStage::Fruiting => HarvestQuality::Prime,
            GrowthStage::Flowering => HarvestQuality::Good,
            GrowthStage::Mature => HarvestQuality::Average,
            _ => HarvestQuality::Poor,
        };

        // Damage from harvesting
        let damage = match self.species.category() {
            crate::encyclopedia::PlantCategory::Tree => 5.0, // Trees regenerate
            crate::encyclopedia::PlantCategory::Shrub => 20.0,
            crate::encyclopedia::PlantCategory::Herb => 50.0, // Herbs heavily impacted
            crate::encyclopedia::PlantCategory::Fungus => 30.0,
            _ => 30.0,
        };

        self.health -= damage;

        // Herbs and fungi might be destroyed by harvest
        if self.species.category() == crate::encyclopedia::PlantCategory::Herb
            || self.species.category() == crate::encyclopedia::PlantCategory::Fungus
        {
            if self.harvest_count >= 2 || self.health <= 0.0 {
                self.stage = GrowthStage::Dead;
            } else {
                self.stage = GrowthStage::Juvenile; // Regrows
            }
        }

        HarvestResult::Success {
            quality,
            quantity: self.calculate_yield(),
        }
    }

    fn calculate_yield(&self) -> u32 {
        let base = match self.species.category() {
            crate::encyclopedia::PlantCategory::Tree => 3,
            crate::encyclopedia::PlantCategory::Shrub => 5,
            crate::encyclopedia::PlantCategory::Herb => 2,
            crate::encyclopedia::PlantCategory::Fungus => 1,
            crate::encyclopedia::PlantCategory::Crop => 8,
            _ => 2,
        };

        let health_mod = (self.health / 100.0 * base as f32) as u32;
        let stage_mod = match self.stage {
            GrowthStage::Fruiting => 2,
            GrowthStage::Flowering => 1,
            _ => 0,
        };

        (health_mod + stage_mod).max(1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarvestResult {
    Failed,
    Success { quality: HarvestQuality, quantity: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarvestQuality {
    Poor,
    Average,
    Good,
    Prime,
}

impl HarvestQuality {
    pub fn value_multiplier(&self) -> f32 {
        match self {
            Self::Poor => 0.5,
            Self::Average => 1.0,
            Self::Good => 1.5,
            Self::Prime => 2.0,
        }
    }
}

/// Flora manager for tracking all plants in the world
#[derive(Debug, Default)]
pub struct FloraManager {
    /// All plant instances by chunk
    pub plants_by_chunk: std::collections::HashMap<(i32, i32), Vec<PlantInstance>>,
    /// Next plant ID
    next_id: u64,
    /// Current season
    pub current_season: Season,
    /// Plants pending removal
    pending_removal: Vec<u64>,
}

impl FloraManager {
    pub fn new() -> Self {
        Self {
            plants_by_chunk: std::collections::HashMap::new(),
            next_id: 0,
            current_season: Season::Spring,
            pending_removal: Vec::new(),
        }
    }

    /// Spawn a new plant
    pub fn spawn_plant(&mut self, species: FloraSpecies, position: [f32; 3]) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let chunk = position_to_chunk(position);
        let plant = PlantInstance::new(species, position, id);

        self.plants_by_chunk
            .entry(chunk)
            .or_insert_with(Vec::new)
            .push(plant);

        id
    }

    /// Update all plants in active chunks
    pub fn update(&mut self, active_chunks: &[(i32, i32)], delta_days: f32, weather_quality: f32) {
        for chunk in active_chunks {
            if let Some(plants) = self.plants_by_chunk.get_mut(chunk) {
                for plant in plants.iter_mut() {
                    plant.update(delta_days, self.current_season, weather_quality);

                    if plant.stage == GrowthStage::Dead && plant.age_days > 30.0 {
                        self.pending_removal.push(plant.id);
                    }
                }
            }
        }

        // Remove dead plants
        self.cleanup_dead_plants();
    }

    fn cleanup_dead_plants(&mut self) {
        let to_remove: std::collections::HashSet<u64> =
            self.pending_removal.drain(..).collect();

        if to_remove.is_empty() {
            return;
        }

        for plants in self.plants_by_chunk.values_mut() {
            plants.retain(|p| !to_remove.contains(&p.id));
        }
    }

    /// Get plants near a position
    pub fn get_plants_near(&self, position: [f32; 3], radius: f32) -> Vec<&PlantInstance> {
        let chunk = position_to_chunk(position);
        let mut result = Vec::new();

        // Check surrounding chunks
        for dx in -1..=1 {
            for dz in -1..=1 {
                let check_chunk = (chunk.0 + dx, chunk.1 + dz);
                if let Some(plants) = self.plants_by_chunk.get(&check_chunk) {
                    for plant in plants {
                        let dist = distance(position, plant.position);
                        if dist <= radius {
                            result.push(plant);
                        }
                    }
                }
            }
        }

        result
    }

    /// Find plant by ID
    pub fn get_plant_mut(&mut self, id: u64) -> Option<&mut PlantInstance> {
        for plants in self.plants_by_chunk.values_mut() {
            if let Some(plant) = plants.iter_mut().find(|p| p.id == id) {
                return Some(plant);
            }
        }
        None
    }

    /// Get the closest harvestable plant to a position
    /// Returns (plant_id, species, distance, can_harvest) for HUD display
    pub fn get_closest_harvestable(
        &self,
        position: [f32; 3],
        max_distance: f32,
    ) -> Option<(u64, super::FloraSpecies, f32, bool)> {
        let chunk = position_to_chunk(position);
        let mut closest: Option<(u64, super::FloraSpecies, f32, bool)> = None;
        let mut min_dist = max_distance;

        for dx in -1..=1 {
            for dz in -1..=1 {
                let check_chunk = (chunk.0 + dx, chunk.1 + dz);
                if let Some(plants) = self.plants_by_chunk.get(&check_chunk) {
                    for plant in plants {
                        // Only consider small plants (herbs, shrubs, fungi, crops)
                        // Trees are too large for foraging
                        let cat = plant.species.category();
                        if cat == crate::encyclopedia::PlantCategory::Tree {
                            continue;
                        }

                        let dist = distance(position, plant.position);
                        if dist < min_dist {
                            min_dist = dist;
                            closest = Some((plant.id, plant.species, dist, plant.can_harvest()));
                        }
                    }
                }
            }
        }

        closest
    }

    /// Harvest a plant by ID, returning the harvest result
    pub fn harvest_plant(&mut self, id: u64) -> Option<(super::FloraSpecies, HarvestResult)> {
        for plants in self.plants_by_chunk.values_mut() {
            if let Some(plant) = plants.iter_mut().find(|p| p.id == id) {
                let species = plant.species;
                let result = plant.harvest();
                return Some((species, result));
            }
        }
        None
    }

    /// Set current season
    pub fn set_season(&mut self, season: Season) {
        self.current_season = season;
    }

    /// Generate flora for a new chunk based on habitat
    pub fn generate_chunk_flora(
        &mut self,
        chunk: (i32, i32),
        habitat: FloraHabitat,
        density: f32,
    ) {
        if self.plants_by_chunk.contains_key(&chunk) {
            return; // Already generated
        }

        let mut plants = Vec::new();
        let chunk_size = 64.0;
        let base_x = chunk.0 as f32 * chunk_size;
        let base_z = chunk.1 as f32 * chunk_size;

        // Get species appropriate for this habitat
        let candidates: Vec<FloraSpecies> = FloraSpecies::all()
            .filter(|s| s.habitats().contains(&habitat))
            .collect();

        if candidates.is_empty() {
            self.plants_by_chunk.insert(chunk, plants);
            return;
        }

        // Spawn plants based on density and rarity
        let target_count = (density * 20.0) as usize;

        for _ in 0..target_count {
            // Pick a random species weighted by rarity
            let species = pick_weighted_species(&candidates);

            // Random position in chunk
            let x = base_x + rand_float() * chunk_size;
            let z = base_z + rand_float() * chunk_size;
            let y = 0.0; // Would be determined by terrain

            let id = self.next_id;
            self.next_id += 1;

            plants.push(PlantInstance::new(species, [x, y, z], id));
        }

        self.plants_by_chunk.insert(chunk, plants);
    }
}

fn position_to_chunk(position: [f32; 3]) -> (i32, i32) {
    let chunk_size = 64.0;
    (
        (position[0] / chunk_size).floor() as i32,
        (position[2] / chunk_size).floor() as i32,
    )
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn pick_weighted_species(candidates: &[FloraSpecies]) -> FloraSpecies {
    let total_weight: f32 = candidates.iter().map(|s| s.rarity().spawn_weight()).sum();
    let mut r = rand_float() * total_weight;

    for species in candidates {
        r -= species.rarity().spawn_weight();
        if r <= 0.0 {
            return *species;
        }
    }

    candidates[0]
}

// Simple random function (would use proper RNG in real implementation)
fn rand_float() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}
