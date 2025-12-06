//! Ecological consequences that affect gameplay
//!
//! Translates ecosystem state into tangible game effects.
//! All consequences are grounded in real ecological principles.

use serde::{Deserialize, Serialize};
use super::{EcosystemHealth, EcologicalConsequence, EcosystemRegion};
use crate::animals::types::AnimalSpecies;

/// Active consequence affecting gameplay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveConsequence {
    pub consequence_type: ConsequenceType,
    pub severity: f32,           // 0-1 severity scale
    pub region_id: u32,
    pub started_at: f64,         // Game time when started
    pub duration: Option<f32>,   // None = permanent until conditions change
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsequenceType {
    // Resource availability
    AbundantForaging,      // Bonus to foraging yields
    ReducedForaging,       // Penalty to foraging
    ResourceScarcity,      // Severe lack of resources

    // Wildlife behavior
    WaryWildlife,          // Animals flee more easily
    AggravatedWildlife,    // Animals more aggressive
    AbundantWildlife,      // More animal spawns
    ScarceWildlife,        // Fewer animal spawns

    // Hunting
    EasyHunting,           // Better tracking, more targets
    DifficultHunting,      // Fewer targets, wary animals

    // Health and safety
    DiseaseRisk,           // Chance of catching illness
    PredatorPressure,      // More predator encounters
    SafeTravel,            // Reduced encounter rate

    // Environmental
    FloraRecovery,         // Plants regenerating quickly
    FloraDepletion,        // Plants not regenerating

    // Special events
    MigrationEvent,        // Animals moving through region
    Bloom,                 // Unusual plant abundance
    DieOff,               // Population crash
}

impl ConsequenceType {
    pub fn from_ecological(eco: EcologicalConsequence) -> Self {
        match eco {
            EcologicalConsequence::Overpopulation(_) => Self::AggravatedWildlife,
            EcologicalConsequence::LocalExtinction(_) => Self::ScarceWildlife,
            EcologicalConsequence::ReducedForaging => Self::ReducedForaging,
            EcologicalConsequence::ResourceScarcity => Self::ResourceScarcity,
            EcologicalConsequence::WaryWildlife => Self::WaryWildlife,
            EcologicalConsequence::IncreasedPredation => Self::PredatorPressure,
            EcologicalConsequence::DiseaseOutbreak => Self::DiseaseRisk,
        }
    }

    pub fn base_severity(&self) -> f32 {
        match self {
            Self::AbundantForaging | Self::AbundantWildlife | Self::EasyHunting |
            Self::SafeTravel | Self::FloraRecovery | Self::Bloom => 0.5, // Positive, moderate

            Self::ReducedForaging | Self::WaryWildlife | Self::DifficultHunting |
            Self::ScarceWildlife | Self::FloraDepletion => 0.5, // Negative, moderate

            Self::ResourceScarcity | Self::AggravatedWildlife | Self::DiseaseRisk |
            Self::PredatorPressure | Self::DieOff => 0.75, // Negative, severe

            Self::MigrationEvent => 0.3, // Neutral event
        }
    }

    pub fn is_positive(&self) -> bool {
        matches!(
            self,
            Self::AbundantForaging | Self::AbundantWildlife | Self::EasyHunting |
            Self::SafeTravel | Self::FloraRecovery | Self::Bloom
        )
    }
}

/// Gameplay modifiers from active consequences
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsequenceModifiers {
    // Resource gathering
    pub foraging_yield: f32,      // Multiplier (1.0 = normal)
    pub harvest_quality: f32,     // Multiplier

    // Wildlife interaction
    pub animal_spawn_rate: f32,   // Multiplier
    pub animal_flee_distance: f32, // Multiplier (higher = more wary)
    pub animal_aggression: f32,    // Multiplier
    pub tracking_difficulty: f32,  // Multiplier

    // Player health
    pub disease_chance: f32,      // Added chance per day
    pub predator_encounter_rate: f32, // Multiplier

    // Movement/travel
    pub travel_safety: f32,       // Multiplier for encounter avoidance

    // Regeneration
    pub flora_regrowth_rate: f32, // Multiplier
}

impl ConsequenceModifiers {
    pub fn new() -> Self {
        Self {
            foraging_yield: 1.0,
            harvest_quality: 1.0,
            animal_spawn_rate: 1.0,
            animal_flee_distance: 1.0,
            animal_aggression: 1.0,
            tracking_difficulty: 1.0,
            disease_chance: 0.0,
            predator_encounter_rate: 1.0,
            travel_safety: 1.0,
            flora_regrowth_rate: 1.0,
        }
    }

    /// Apply a consequence to the modifiers
    pub fn apply(&mut self, consequence_type: ConsequenceType, severity: f32) {
        match consequence_type {
            ConsequenceType::AbundantForaging => {
                self.foraging_yield += severity * 0.5;
                self.harvest_quality += severity * 0.3;
            }
            ConsequenceType::ReducedForaging => {
                self.foraging_yield -= severity * 0.4;
                self.harvest_quality -= severity * 0.2;
            }
            ConsequenceType::ResourceScarcity => {
                self.foraging_yield -= severity * 0.7;
                self.harvest_quality -= severity * 0.5;
            }

            ConsequenceType::WaryWildlife => {
                self.animal_flee_distance += severity * 0.5;
                self.tracking_difficulty += severity * 0.4;
            }
            ConsequenceType::AggravatedWildlife => {
                self.animal_aggression += severity * 0.6;
                self.animal_flee_distance -= severity * 0.3;
            }
            ConsequenceType::AbundantWildlife => {
                self.animal_spawn_rate += severity * 0.4;
                self.tracking_difficulty -= severity * 0.2;
            }
            ConsequenceType::ScarceWildlife => {
                self.animal_spawn_rate -= severity * 0.5;
                self.tracking_difficulty += severity * 0.3;
            }

            ConsequenceType::EasyHunting => {
                self.tracking_difficulty -= severity * 0.4;
                self.animal_spawn_rate += severity * 0.2;
            }
            ConsequenceType::DifficultHunting => {
                self.tracking_difficulty += severity * 0.5;
                self.animal_flee_distance += severity * 0.3;
            }

            ConsequenceType::DiseaseRisk => {
                self.disease_chance += severity * 0.05; // Up to 5% per day
            }
            ConsequenceType::PredatorPressure => {
                self.predator_encounter_rate += severity * 0.5;
                self.travel_safety -= severity * 0.3;
            }
            ConsequenceType::SafeTravel => {
                self.travel_safety += severity * 0.3;
                self.predator_encounter_rate -= severity * 0.2;
            }

            ConsequenceType::FloraRecovery => {
                self.flora_regrowth_rate += severity * 0.5;
            }
            ConsequenceType::FloraDepletion => {
                self.flora_regrowth_rate -= severity * 0.4;
            }

            ConsequenceType::MigrationEvent => {
                self.animal_spawn_rate += severity * 0.3;
            }
            ConsequenceType::Bloom => {
                self.foraging_yield += severity * 0.8;
            }
            ConsequenceType::DieOff => {
                self.animal_spawn_rate -= severity * 0.6;
            }
        }

        // Clamp values to reasonable ranges
        self.foraging_yield = self.foraging_yield.clamp(0.1, 2.0);
        self.harvest_quality = self.harvest_quality.clamp(0.3, 1.5);
        self.animal_spawn_rate = self.animal_spawn_rate.clamp(0.1, 2.0);
        self.animal_flee_distance = self.animal_flee_distance.clamp(0.5, 2.0);
        self.animal_aggression = self.animal_aggression.clamp(0.5, 2.0);
        self.tracking_difficulty = self.tracking_difficulty.clamp(0.5, 2.0);
        self.disease_chance = self.disease_chance.clamp(0.0, 0.2);
        self.predator_encounter_rate = self.predator_encounter_rate.clamp(0.2, 3.0);
        self.travel_safety = self.travel_safety.clamp(0.3, 1.5);
        self.flora_regrowth_rate = self.flora_regrowth_rate.clamp(0.1, 2.0);
    }

    /// Combine modifiers from multiple regions (for overlapping effects)
    pub fn combine(&mut self, other: &ConsequenceModifiers) {
        // Average the modifiers
        self.foraging_yield = (self.foraging_yield + other.foraging_yield) / 2.0;
        self.harvest_quality = (self.harvest_quality + other.harvest_quality) / 2.0;
        self.animal_spawn_rate = (self.animal_spawn_rate + other.animal_spawn_rate) / 2.0;
        self.animal_flee_distance = (self.animal_flee_distance + other.animal_flee_distance) / 2.0;
        self.animal_aggression = (self.animal_aggression + other.animal_aggression) / 2.0;
        self.tracking_difficulty = (self.tracking_difficulty + other.tracking_difficulty) / 2.0;
        self.disease_chance = (self.disease_chance + other.disease_chance) / 2.0;
        self.predator_encounter_rate = (self.predator_encounter_rate + other.predator_encounter_rate) / 2.0;
        self.travel_safety = (self.travel_safety + other.travel_safety) / 2.0;
        self.flora_regrowth_rate = (self.flora_regrowth_rate + other.flora_regrowth_rate) / 2.0;
    }
}

/// Calculate consequences from ecosystem state
pub fn calculate_consequences(region: &EcosystemRegion) -> Vec<ActiveConsequence> {
    let mut consequences = Vec::new();
    let health = region.health_rating();

    // Base consequences from ecosystem health
    match health {
        EcosystemHealth::Thriving => {
            consequences.push(ActiveConsequence {
                consequence_type: ConsequenceType::AbundantForaging,
                severity: 0.4,
                region_id: region.id,
                started_at: 0.0,
                duration: None,
                description: "The land is abundant with resources".to_string(),
            });
            consequences.push(ActiveConsequence {
                consequence_type: ConsequenceType::AbundantWildlife,
                severity: 0.3,
                region_id: region.id,
                started_at: 0.0,
                duration: None,
                description: "Wildlife is plentiful in this region".to_string(),
            });
        }
        EcosystemHealth::Stressed => {
            consequences.push(ActiveConsequence {
                consequence_type: ConsequenceType::ReducedForaging,
                severity: 0.3,
                region_id: region.id,
                started_at: 0.0,
                duration: None,
                description: "Resources are becoming harder to find".to_string(),
            });
        }
        EcosystemHealth::Degraded => {
            consequences.push(ActiveConsequence {
                consequence_type: ConsequenceType::ResourceScarcity,
                severity: 0.5,
                region_id: region.id,
                started_at: 0.0,
                duration: None,
                description: "The land is depleted of resources".to_string(),
            });
            consequences.push(ActiveConsequence {
                consequence_type: ConsequenceType::PredatorPressure,
                severity: 0.4,
                region_id: region.id,
                started_at: 0.0,
                duration: None,
                description: "Hungry predators are becoming aggressive".to_string(),
            });
        }
        EcosystemHealth::Collapsed => {
            consequences.push(ActiveConsequence {
                consequence_type: ConsequenceType::ResourceScarcity,
                severity: 0.9,
                region_id: region.id,
                started_at: 0.0,
                duration: None,
                description: "The ecosystem has collapsed - survival is difficult".to_string(),
            });
            consequences.push(ActiveConsequence {
                consequence_type: ConsequenceType::DiseaseRisk,
                severity: 0.6,
                region_id: region.id,
                started_at: 0.0,
                duration: None,
                description: "Poor conditions have led to disease".to_string(),
            });
        }
        _ => {}
    }

    // Hunting pressure consequences
    if region.hunting_pressure > 0.6 {
        consequences.push(ActiveConsequence {
            consequence_type: ConsequenceType::WaryWildlife,
            severity: region.hunting_pressure,
            region_id: region.id,
            started_at: 0.0,
            duration: None,
            description: "Animals in this area have learned to fear hunters".to_string(),
        });
    }

    // Check for population-specific issues
    for (species, pop) in &region.fauna_populations {
        let ratio = pop.current as f32 / pop.carrying_capacity.max(1) as f32;

        if ratio < 0.2 {
            consequences.push(ActiveConsequence {
                consequence_type: ConsequenceType::ScarceWildlife,
                severity: 1.0 - ratio,
                region_id: region.id,
                started_at: 0.0,
                duration: None,
                description: format!("{} have become very rare here", species.name()),
            });
        } else if ratio > 1.5 {
            // Overpopulation can lead to aggression
            if species.is_predator() {
                consequences.push(ActiveConsequence {
                    consequence_type: ConsequenceType::PredatorPressure,
                    severity: (ratio - 1.0).min(1.0),
                    region_id: region.id,
                    started_at: 0.0,
                    duration: None,
                    description: format!("{} population is unusually high", species.name()),
                });
            }
        }
    }

    consequences
}

/// Weather-ecology interaction effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherEcologyEffect {
    pub weather_type: WeatherType,
    pub ecology_modifier: f32,
    pub duration_hours: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Drought,
    Flood,
}

impl WeatherType {
    /// Get effect on ecosystem
    pub fn ecology_effect(&self) -> f32 {
        match self {
            Self::Clear => 1.0,    // Normal
            Self::Cloudy => 1.0,
            Self::Rain => 1.1,     // Good for plants
            Self::Storm => 0.9,    // Minor damage
            Self::Drought => 0.7,  // Stress
            Self::Flood => 0.8,    // Disruption
        }
    }

    /// Get effect on wildlife activity
    pub fn wildlife_activity_modifier(&self) -> f32 {
        match self {
            Self::Clear => 1.0,
            Self::Cloudy => 1.1,   // Animals more active
            Self::Rain => 0.6,     // Less active
            Self::Storm => 0.2,    // Sheltering
            Self::Drought => 0.8,
            Self::Flood => 0.4,
        }
    }
}
