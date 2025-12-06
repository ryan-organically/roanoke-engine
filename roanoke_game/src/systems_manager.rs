//! Central Systems Manager
//!
//! Coordinates all gameplay systems and manages data pipelines between them.
//! Handles the flow of data from:
//! - Animals → Encyclopedia (observations)
//! - Weather → Ecology (environmental effects)
//! - Ecology → Animals (population dynamics)
//! - Time → Flora (growth cycles)
//! - Player actions → All systems

use serde::{Deserialize, Serialize};
use glam::Vec3;

use crate::encyclopedia::{Encyclopedia, Season};
use crate::encyclopedia::observer::{ObservationManager, BehaviorWitnessType};
use crate::flora::{FloraSpecies, FloraManager};
use crate::flora::medicinal::MedicinalSystem;
use crate::ecology::{EcologyManager, EcosystemRegion};
use crate::ecology::consequences::ConsequenceModifiers;
use crate::weather::{WeatherManager, WeatherState};
use crate::animals::types::AnimalSpecies;
use crate::animals::manager::AnimalManager;

/// Serializable state for save/load
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemsSaveData {
    pub encyclopedia: EncyclopediaSaveData,
    pub flora: FloraSaveData,
    pub ecology: EcologySaveData,
    pub weather: WeatherSaveData,
    pub game_day: u32,
    pub game_hour: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncyclopediaSaveData {
    pub discovered_fauna: Vec<(String, u32)>,  // (species_name, discovery_tier)
    pub discovered_flora: Vec<(String, u32)>,
    pub total_observations: u32,
    pub behaviors_witnessed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloraSaveData {
    pub known_plants: Vec<String>,
    pub harvested_count: u32,
    pub remedies_crafted: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcologySaveData {
    pub hunting_pressure_by_region: Vec<(u32, f32)>,
    pub total_animals_hunted: u32,
    pub ecosystem_health_scores: Vec<(u32, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSaveData {
    pub current_weather_type: u8,
    pub temperature: f32,
    pub wind_speed: f32,
    pub day_of_year: u32,
    pub cumulative_rainfall: f32,
}

/// Central manager for all gameplay systems
pub struct SystemsManager {
    // Core systems
    pub encyclopedia: Encyclopedia,
    pub observation_manager: ObservationManager,
    pub flora_manager: FloraManager,
    pub medicinal_system: MedicinalSystem,
    pub ecology_manager: EcologyManager,
    pub weather_manager: WeatherManager,

    // Cached modifiers for performance
    cached_consequence_modifiers: ConsequenceModifiers,
    modifier_cache_time: f32,

    // Time tracking
    pub game_day: u32,
    pub game_hour: f32,
    last_ecology_update: f32,
    last_flora_update: f32,

    // Pipeline state
    pending_observations: Vec<PendingObservation>,
    pending_harvests: Vec<PendingHarvest>,

    // Statistics
    pub stats: SystemsStats,
}

#[derive(Debug, Clone, Default)]
pub struct SystemsStats {
    pub animals_observed: u32,
    pub plants_harvested: u32,
    pub remedies_crafted: u32,
    pub animals_hunted: u32,
    pub discoveries_made: u32,
}

#[derive(Debug, Clone)]
struct PendingObservation {
    species: AnimalSpecies,
    entity_id: u64,
    position: Vec3,
    behavior: Option<BehaviorWitnessType>,
    duration: f32,
}

#[derive(Debug, Clone)]
struct PendingHarvest {
    species: FloraSpecies,
    position: Vec3,
    quality: f32,
}

impl SystemsManager {
    pub fn new(seed: u32) -> Self {
        Self {
            encyclopedia: Encyclopedia::new(),
            observation_manager: ObservationManager::new(),
            flora_manager: FloraManager::new(seed),
            medicinal_system: MedicinalSystem::new(),
            ecology_manager: EcologyManager::with_initial_regions(),
            weather_manager: WeatherManager::new(), // Uses Default, seed not needed
            cached_consequence_modifiers: ConsequenceModifiers::new(),
            modifier_cache_time: 0.0,
            game_day: 1,
            game_hour: 8.0, // Start at 8 AM
            last_ecology_update: 0.0,
            last_flora_update: 0.0,
            pending_observations: Vec::new(),
            pending_harvests: Vec::new(),
            stats: SystemsStats::default(),
        }
    }

    /// Main update loop - call every frame
    pub fn update(
        &mut self,
        delta_time: f32,
        player_pos: Vec3,
        player_look_dir: Vec3,
        animal_manager: &AnimalManager,
    ) {
        // Update time
        self.update_time(delta_time);

        // Update weather (every frame for smooth transitions)
        self.weather_manager.update(delta_time);

        // Update observation manager with player state
        self.observation_manager.update_player(
            [player_pos.x, player_pos.y, player_pos.z],
            [player_look_dir.x, player_look_dir.y, player_look_dir.z],
        );

        // Process animal observations pipeline
        self.process_observation_pipeline(player_pos, animal_manager, delta_time);

        // Update ecology periodically (every game hour)
        if self.game_hour - self.last_ecology_update >= 1.0 || self.last_ecology_update > self.game_hour {
            self.update_ecology_pipeline();
            self.last_ecology_update = self.game_hour;
        }

        // Update flora periodically (every 6 game hours)
        let hours_since_flora = if self.game_hour >= self.last_flora_update {
            self.game_hour - self.last_flora_update
        } else {
            24.0 - self.last_flora_update + self.game_hour
        };
        if hours_since_flora >= 6.0 {
            self.update_flora_pipeline(delta_time);
            self.last_flora_update = self.game_hour;
        }

        // Refresh cached modifiers periodically
        self.modifier_cache_time += delta_time;
        if self.modifier_cache_time > 60.0 { // Every minute
            self.refresh_consequence_modifiers(player_pos);
            self.modifier_cache_time = 0.0;
        }
    }

    fn update_time(&mut self, delta_time: f32) {
        // Game time runs at 1 real second = 1 game minute
        let game_minutes = delta_time;
        self.game_hour += game_minutes / 60.0;

        if self.game_hour >= 24.0 {
            self.game_hour -= 24.0;
            self.game_day += 1;

            // Daily updates
            self.on_new_day();
        }
    }

    fn on_new_day(&mut self) {
        // Update season in encyclopedia
        let day_of_year = (self.game_day % 365) as u32;
        self.encyclopedia.current_season = Season::from_day_of_year(day_of_year);

        // Flora daily growth
        self.flora_manager.advance_day();

        // Ecology daily population changes
        self.ecology_manager.daily_update();
    }

    /// Process observations of nearby animals
    fn process_observation_pipeline(
        &mut self,
        player_pos: Vec3,
        animal_manager: &AnimalManager,
        delta_time: f32,
    ) {
        // Get animals within observation range
        let observation_range = 100.0;
        let nearby_animals = animal_manager.animals_near(player_pos, observation_range);

        for animal in nearby_animals {
            let distance = (animal.position - player_pos).length();

            // Check if we should start or update an observation
            let existing_sessions = self.observation_manager.get_fauna_sessions(animal.species);

            if existing_sessions.is_empty() && distance < 50.0 {
                // Start new observation session
                let _session_id = self.observation_manager.start_fauna_observation(
                    animal.species,
                    animal.id.0,  // Extract u64 from AnimalId
                    [animal.position.x, animal.position.y, animal.position.z],
                );

                // Record sighting in encyclopedia
                self.encyclopedia.record_sighting(animal.species);
                self.stats.animals_observed += 1;
            } else {
                // Update existing sessions
                for session_id in existing_sessions {
                    let has_los = self.check_line_of_sight(player_pos, animal.position);

                    if let Some(effective_time) = self.observation_manager.update_session(
                        session_id,
                        [animal.position.x, animal.position.y, animal.position.z],
                        has_los,
                        delta_time,
                    ) {
                        // Record observation time in encyclopedia
                        self.encyclopedia.add_observation_time(animal.species, effective_time);
                    }

                    // Check for behavior observations based on animal state
                    if let Some(behavior) = self.detect_animal_behavior(animal) {
                        self.observation_manager.record_behavior(
                            session_id,
                            behavior,
                            self.game_hour,
                        );
                        self.encyclopedia.record_behavior(animal.species, behavior);
                    }

                    // End session if target lost
                    if self.observation_manager.should_end_session(session_id) {
                        if let Some(session) = self.observation_manager.end_session(session_id) {
                            // Finalize observation data
                            self.finalize_observation(animal.species, &session);
                        }
                    }
                }
            }
        }
    }

    fn check_line_of_sight(&self, from: Vec3, to: Vec3) -> bool {
        // Simplified LoS check - in full implementation would raycast
        let distance = (to - from).length();
        distance < 100.0 // Basic range check
    }

    fn detect_animal_behavior(&self, animal: &crate::animals::entity::Animal) -> Option<BehaviorWitnessType> {
        // Map animal behavior state to witness type
        use crate::animals::behavior::BehaviorState;

        match &animal.behavior_state {
            BehaviorState::Idle => None,
            BehaviorState::Patrol => None,
            BehaviorState::Alert(_) => Some(BehaviorWitnessType::TerritorialDisplay),
            BehaviorState::Pursue(_) => Some(BehaviorWitnessType::Hunting),
            BehaviorState::Attack(_) => Some(BehaviorWitnessType::AttackingPrey),
            BehaviorState::Flee(_) => Some(BehaviorWitnessType::FleeingPredator),
            BehaviorState::Dead => None,
        }
    }

    fn finalize_observation(
        &mut self,
        species: AnimalSpecies,
        session: &crate::encyclopedia::observer::ObservationSession,
    ) {
        // Calculate XP from observation
        let base_xp = session.session_time * session.quality_multiplier;
        let behavior_bonus: f32 = session.behaviors_witnessed
            .iter()
            .map(|b| b.behavior_type.xp_bonus())
            .sum();

        let total_xp = base_xp * (1.0 + behavior_bonus * 0.1);
        self.encyclopedia.add_study_xp(species, total_xp);

        // Check for discovery tier advancement
        if self.encyclopedia.check_tier_advancement(species) {
            self.stats.discoveries_made += 1;
        }
    }

    /// Update ecology based on weather and hunting
    fn update_ecology_pipeline(&mut self) {
        // Get weather effects
        let weather_state = &self.weather_manager.current_state;
        let weather_modifier = weather_state.weather_type.hunting_modifier();

        // Apply weather to ecology
        self.ecology_manager.apply_weather_effects(
            weather_modifier,
            weather_state.precipitation_rate,
        );

        // Update populations
        self.ecology_manager.update_populations(1.0); // 1 hour delta
    }

    /// Update flora growth based on time and weather
    fn update_flora_pipeline(&mut self, _delta_time: f32) {
        let weather = &self.weather_manager.current_state;
        let growth_modifier = self.calculate_growth_modifier(weather);

        self.flora_manager.update_growth(growth_modifier, self.game_hour);
    }

    fn calculate_growth_modifier(&self, weather: &crate::weather::WeatherState) -> f32 {
        let mut modifier = 1.0;

        // Rain helps growth
        modifier += weather.precipitation_rate * 0.2;

        // Temperature affects growth
        let temp = weather.temperature;
        if temp < 40.0 {
            modifier *= 0.5; // Cold slows growth
        } else if temp > 90.0 {
            modifier *= 0.7; // Heat stress
        } else if temp > 60.0 && temp < 80.0 {
            modifier *= 1.2; // Optimal temperature
        }

        modifier
    }

    fn refresh_consequence_modifiers(&mut self, player_pos: Vec3) {
        // Get the region the player is in
        if let Some(region) = self.ecology_manager.get_region_at_vec3(player_pos) {
            // Use default modifiers based on region - actual consequence tracking handled elsewhere
            self.cached_consequence_modifiers = ConsequenceModifiers::default();
        }
    }

    // === PUBLIC API FOR GAME INTEGRATION ===

    /// Record a hunting kill - affects ecology and encyclopedia
    pub fn record_hunt(&mut self, species: AnimalSpecies, position: Vec3) {
        self.stats.animals_hunted += 1;

        // Update ecology hunting pressure
        self.ecology_manager.record_hunt(species, position);

        // Record in encyclopedia (studying harvested animals)
        self.encyclopedia.record_harvest_study(species);
    }

    /// Attempt to harvest a plant
    pub fn harvest_plant(&mut self, species: FloraSpecies, position: Vec3) -> Option<HarvestResult> {
        let weather = &self.weather_manager.current_state;
        let quality_modifier = 1.0 - weather.precipitation_rate * 0.3; // Rain reduces quality

        if let Some(harvest) = self.flora_manager.harvest(species, position, quality_modifier) {
            self.stats.plants_harvested += 1;

            // Record in encyclopedia
            self.encyclopedia.record_flora_harvest(species);

            // Check for medicinal properties
            let medicinal_info = self.medicinal_system.get_plant_info(species);

            return Some(HarvestResult {
                species,
                quantity: harvest.quantity,
                quality: harvest.quality,
                medicinal_info,
            });
        }

        None
    }

    /// Craft a remedy from ingredients
    pub fn craft_remedy(
        &mut self,
        ingredients: &[FloraSpecies],
    ) -> Option<crate::flora::medicinal::Remedy> {
        if let Some(remedy) = self.medicinal_system.try_craft(ingredients) {
            self.stats.remedies_crafted += 1;
            Some(remedy)
        } else {
            None
        }
    }

    /// Get current gameplay modifiers from ecology
    pub fn get_gameplay_modifiers(&self) -> &ConsequenceModifiers {
        &self.cached_consequence_modifiers
    }

    /// Get weather effects on gameplay
    pub fn get_weather_effects(&self) -> crate::weather::effects::WeatherEffects {
        crate::weather::effects::WeatherEffects::from_weather(&self.weather_manager.current_state)
    }

    /// Get animal spawn rate modifier
    pub fn get_spawn_rate_modifier(&self) -> f32 {
        let eco_mod = self.cached_consequence_modifiers.animal_spawn_rate;
        let weather_mod = self.weather_manager.current_state.weather_type.hunting_modifier();
        eco_mod * weather_mod
    }

    /// Get current season
    pub fn current_season(&self) -> Season {
        self.encyclopedia.current_season
    }

    // === SAVE/LOAD ===

    pub fn to_save_data(&self) -> SystemsSaveData {
        SystemsSaveData {
            encyclopedia: EncyclopediaSaveData {
                discovered_fauna: self.encyclopedia.get_discovered_fauna(),
                discovered_flora: self.encyclopedia.get_discovered_flora(),
                total_observations: self.stats.animals_observed,
                behaviors_witnessed: self.encyclopedia.get_witnessed_behaviors(),
            },
            flora: FloraSaveData {
                known_plants: self.flora_manager.get_known_plants(),
                harvested_count: self.stats.plants_harvested,
                remedies_crafted: self.stats.remedies_crafted,
            },
            ecology: EcologySaveData {
                hunting_pressure_by_region: self.ecology_manager.get_hunting_pressures(),
                total_animals_hunted: self.stats.animals_hunted,
                ecosystem_health_scores: self.ecology_manager.get_health_scores(),
            },
            weather: WeatherSaveData {
                current_weather_type: self.weather_manager.current_weather_type_id(),
                temperature: self.weather_manager.current_state.temperature,
                wind_speed: self.weather_manager.current_state.wind_speed,
                day_of_year: self.weather_manager.day_of_year,
                cumulative_rainfall: self.weather_manager.cumulative_rainfall,
            },
            game_day: self.game_day,
            game_hour: self.game_hour,
        }
    }

    pub fn from_save_data(&mut self, data: SystemsSaveData) {
        self.game_day = data.game_day;
        self.game_hour = data.game_hour;

        // Restore encyclopedia
        self.encyclopedia.restore_discoveries(
            &data.encyclopedia.discovered_fauna,
            &data.encyclopedia.discovered_flora,
        );

        // Restore flora
        self.flora_manager.restore_known_plants(&data.flora.known_plants);

        // Restore ecology
        self.ecology_manager.restore_hunting_pressures(&data.ecology.hunting_pressure_by_region);
        self.ecology_manager.restore_health_scores(&data.ecology.ecosystem_health_scores);

        // Restore weather
        self.weather_manager.restore_state(
            data.weather.day_of_year,
            data.weather.cumulative_rainfall,
        );

        // Restore stats
        self.stats.animals_observed = data.encyclopedia.total_observations;
        self.stats.plants_harvested = data.flora.harvested_count;
        self.stats.remedies_crafted = data.flora.remedies_crafted;
        self.stats.animals_hunted = data.ecology.total_animals_hunted;
    }
}

/// Result of harvesting a plant
#[derive(Debug, Clone)]
pub struct HarvestResult {
    pub species: FloraSpecies,
    pub quantity: u32,
    pub quality: f32,
    pub medicinal_info: Option<crate::flora::medicinal::PlantMedicinalInfo>,
}

/// Events that can be emitted by the systems manager
#[derive(Debug, Clone)]
pub enum SystemEvent {
    DiscoveryMade { species_name: String, tier: u32 },
    BehaviorWitnessed { species_name: String, behavior: String },
    PlantHarvested { species_name: String, quantity: u32 },
    RemedyCrafted { remedy_name: String },
    EcosystemWarning { region_id: u32, warning: String },
    WeatherAlert { message: String },
}
