//! Habitat quality and environmental conditions
//!
//! Models the physical environment and how human activities affect it.

use serde::{Deserialize, Serialize};
use super::BiomeType;

/// Detailed habitat quality metrics for a region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitatQuality {
    // Vegetation metrics
    pub canopy_cover: f32,        // 0-1, tree coverage
    pub understory_density: f32,  // 0-1, shrub/herb layer
    pub dead_wood: f32,           // 0-1, important for many species
    pub ground_cover: f32,        // 0-1, leaf litter, vegetation

    // Water
    pub water_quality: f32,       // 0-1, pollution level
    pub water_availability: f32,  // 0-1, drought conditions

    // Disturbance
    pub fragmentation: f32,       // 0-1, how broken up habitat is
    pub edge_effect: f32,         // 0-1, proportion that's edge habitat
    pub human_presence: f32,      // 0-1, regular human activity

    // Soil/substrate
    pub soil_quality: f32,        // 0-1, affects plant growth
    pub erosion_level: f32,       // 0-1, soil loss

    // Seasonal
    pub current_season: HabitatSeason,
}

impl Default for HabitatQuality {
    fn default() -> Self {
        Self {
            canopy_cover: 0.7,
            understory_density: 0.6,
            dead_wood: 0.3,
            ground_cover: 0.8,
            water_quality: 0.9,
            water_availability: 0.8,
            fragmentation: 0.1,
            edge_effect: 0.2,
            human_presence: 0.1,
            soil_quality: 0.8,
            erosion_level: 0.1,
            current_season: HabitatSeason::Summer,
        }
    }
}

impl HabitatQuality {
    /// Create habitat quality for a specific biome
    pub fn for_biome(biome: BiomeType) -> Self {
        match biome {
            BiomeType::DeciduousForest => Self {
                canopy_cover: 0.8,
                understory_density: 0.6,
                dead_wood: 0.4,
                ground_cover: 0.9,
                water_availability: 0.7,
                ..Default::default()
            },
            BiomeType::PineForest => Self {
                canopy_cover: 0.7,
                understory_density: 0.4,
                dead_wood: 0.3,
                ground_cover: 0.6,
                soil_quality: 0.6, // More acidic
                ..Default::default()
            },
            BiomeType::Swamp => Self {
                canopy_cover: 0.5,
                understory_density: 0.7,
                ground_cover: 0.4,
                water_availability: 1.0,
                water_quality: 0.7,
                ..Default::default()
            },
            BiomeType::CoastalMarsh => Self {
                canopy_cover: 0.1,
                understory_density: 0.8,
                ground_cover: 0.9,
                water_availability: 1.0,
                water_quality: 0.8,
                soil_quality: 0.5,
                ..Default::default()
            },
            BiomeType::Meadow => Self {
                canopy_cover: 0.1,
                understory_density: 0.9,
                ground_cover: 1.0,
                dead_wood: 0.1,
                ..Default::default()
            },
            BiomeType::River => Self {
                canopy_cover: 0.4,
                water_availability: 1.0,
                water_quality: 0.9,
                ..Default::default()
            },
            BiomeType::Mountain => Self {
                canopy_cover: 0.5,
                understory_density: 0.4,
                ground_cover: 0.5,
                water_availability: 0.6,
                soil_quality: 0.5,
                erosion_level: 0.3,
                ..Default::default()
            },
        }
    }

    /// Calculate overall habitat suitability score
    pub fn overall_quality(&self) -> f32 {
        let vegetation = (self.canopy_cover + self.understory_density + self.ground_cover) / 3.0;
        let water = (self.water_quality + self.water_availability) / 2.0;
        let disturbance = 1.0 - (self.fragmentation + self.edge_effect + self.human_presence) / 3.0;
        let soil = (self.soil_quality + (1.0 - self.erosion_level)) / 2.0;

        (vegetation * 0.3 + water * 0.25 + disturbance * 0.25 + soil * 0.2).clamp(0.0, 1.0)
    }

    /// Apply damage from human activities
    pub fn apply_damage(&mut self, damage_type: HabitatDamage, intensity: f32) {
        match damage_type {
            HabitatDamage::TreeFelling => {
                self.canopy_cover = (self.canopy_cover - intensity * 0.1).max(0.0);
                self.dead_wood += intensity * 0.05;
                self.fragmentation += intensity * 0.02;
            }
            HabitatDamage::Burning => {
                self.understory_density = (self.understory_density - intensity * 0.3).max(0.0);
                self.ground_cover = (self.ground_cover - intensity * 0.2).max(0.0);
                self.dead_wood = (self.dead_wood - intensity * 0.5).max(0.0);
                // Fire can actually improve some habitats after recovery
            }
            HabitatDamage::Overharvesting => {
                self.understory_density = (self.understory_density - intensity * 0.15).max(0.0);
                self.ground_cover = (self.ground_cover - intensity * 0.1).max(0.0);
            }
            HabitatDamage::Pollution => {
                self.water_quality = (self.water_quality - intensity * 0.2).max(0.0);
                self.soil_quality = (self.soil_quality - intensity * 0.1).max(0.0);
            }
            HabitatDamage::Construction => {
                self.canopy_cover = (self.canopy_cover - intensity * 0.3).max(0.0);
                self.understory_density = (self.understory_density - intensity * 0.4).max(0.0);
                self.ground_cover = (self.ground_cover - intensity * 0.5).max(0.0);
                self.fragmentation += intensity * 0.1;
                self.human_presence += intensity * 0.15;
            }
            HabitatDamage::Trampling => {
                self.ground_cover = (self.ground_cover - intensity * 0.1).max(0.0);
                self.erosion_level += intensity * 0.05;
                self.human_presence += intensity * 0.05;
            }
        }
    }

    /// Natural recovery over time
    pub fn recover(&mut self, delta_days: f32) {
        let recovery_rate = 0.001 * delta_days;

        // Vegetation slowly recovers
        self.canopy_cover = (self.canopy_cover + recovery_rate * 0.1).min(1.0);
        self.understory_density = (self.understory_density + recovery_rate).min(1.0);
        self.ground_cover = (self.ground_cover + recovery_rate).min(1.0);

        // Water quality recovers faster
        self.water_quality = (self.water_quality + recovery_rate * 2.0).min(1.0);

        // Soil recovers slowly
        self.erosion_level = (self.erosion_level - recovery_rate * 0.5).max(0.0);
        self.soil_quality = (self.soil_quality + recovery_rate * 0.3).min(1.0);

        // Human presence decays
        self.human_presence = (self.human_presence - recovery_rate * 3.0).max(0.0);

        // Dead wood decomposes
        self.dead_wood = (self.dead_wood - recovery_rate * 0.1).max(0.0);
    }

    /// Apply seasonal changes
    pub fn apply_season(&mut self, season: HabitatSeason) {
        self.current_season = season;

        match season {
            HabitatSeason::Spring => {
                self.understory_density = (self.understory_density + 0.1).min(1.0);
                self.ground_cover = (self.ground_cover + 0.15).min(1.0);
                self.water_availability = (self.water_availability + 0.1).min(1.0);
            }
            HabitatSeason::Summer => {
                self.canopy_cover = (self.canopy_cover + 0.05).min(1.0);
                self.water_availability = (self.water_availability - 0.1).max(0.3);
            }
            HabitatSeason::Fall => {
                self.ground_cover = (self.ground_cover + 0.1).min(1.0); // Leaf fall
                self.dead_wood = (self.dead_wood + 0.05).min(0.5);
            }
            HabitatSeason::Winter => {
                self.understory_density = (self.understory_density - 0.2).max(0.1);
                self.ground_cover = (self.ground_cover - 0.1).max(0.3);
            }
        }
    }

    /// Get carrying capacity modifier for wildlife
    pub fn carrying_capacity_modifier(&self) -> f32 {
        let quality = self.overall_quality();

        // Non-linear relationship: small decreases in quality
        // have larger effects on carrying capacity
        quality.powf(1.5)
    }

    /// Check if habitat is suitable for a specific activity
    pub fn suitability_for(&self, activity: HabitatActivity) -> f32 {
        match activity {
            HabitatActivity::Foraging => {
                (self.understory_density + self.ground_cover) / 2.0
            }
            HabitatActivity::Hunting => {
                let visibility = 1.0 - self.understory_density * 0.5;
                let prey_habitat = self.overall_quality();
                (visibility + prey_habitat) / 2.0
            }
            HabitatActivity::Shelter => {
                (self.canopy_cover + self.dead_wood) / 2.0
            }
            HabitatActivity::WaterAccess => {
                self.water_availability * self.water_quality
            }
            HabitatActivity::Nesting => {
                let cover = (self.canopy_cover + self.understory_density) / 2.0;
                let disturbance = 1.0 - self.human_presence;
                cover * disturbance
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HabitatSeason {
    Spring,
    Summer,
    Fall,
    Winter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HabitatDamage {
    TreeFelling,
    Burning,
    Overharvesting,
    Pollution,
    Construction,
    Trampling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HabitatActivity {
    Foraging,
    Hunting,
    Shelter,
    WaterAccess,
    Nesting,
}

/// Track habitat change over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitatHistory {
    pub snapshots: Vec<HabitatSnapshot>,
    pub max_snapshots: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitatSnapshot {
    pub game_time: f64,
    pub overall_quality: f32,
    pub canopy: f32,
    pub understory: f32,
    pub water: f32,
}

impl HabitatHistory {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots,
        }
    }

    pub fn record(&mut self, game_time: f64, habitat: &HabitatQuality) {
        let snapshot = HabitatSnapshot {
            game_time,
            overall_quality: habitat.overall_quality(),
            canopy: habitat.canopy_cover,
            understory: habitat.understory_density,
            water: habitat.water_availability,
        };

        self.snapshots.push(snapshot);

        while self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
    }

    pub fn trend(&self) -> HabitatTrend {
        if self.snapshots.len() < 2 {
            return HabitatTrend::Stable;
        }

        let recent = self.snapshots.last().unwrap().overall_quality;
        let old = self.snapshots.first().unwrap().overall_quality;
        let change = recent - old;

        if change > 0.1 {
            HabitatTrend::Improving
        } else if change < -0.1 {
            HabitatTrend::Degrading
        } else {
            HabitatTrend::Stable
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HabitatTrend {
    Improving,
    Stable,
    Degrading,
}
