//! Encyclopedia and Discovery System
//!
//! A naturalistic field journal system for cataloging wildlife and flora observations.
//! Based on real-world naturalist documentation practices from the colonial era.

pub mod entries;
pub mod observer;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::animals::types::AnimalSpecies;
use crate::flora::FloraSpecies;

/// Discovery tier representing depth of knowledge about a species.
/// Based on naturalist observation methodology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum DiscoveryTier {
    /// Never encountered - exists as rumor or in books only
    #[default]
    Unknown,
    /// Briefly glimpsed - basic silhouette, rough size
    Sighted,
    /// Watched from distance - behavior patterns, habitat
    Observed,
    /// Extensive study - biology, weaknesses, uses
    Studied,
    /// Complete mastery - all knowledge unlocked
    Mastered,
}

impl DiscoveryTier {
    /// Get the observation time required to reach this tier (cumulative seconds)
    pub fn observation_time_required(&self) -> f32 {
        match self {
            Self::Unknown => 0.0,
            Self::Sighted => 3.0,      // Quick glimpse
            Self::Observed => 30.0,    // Half minute of watching
            Self::Studied => 180.0,    // 3 minutes total
            Self::Mastered => 600.0,   // 10 minutes of careful study
        }
    }

    /// Get the next tier, if any
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Unknown => Some(Self::Sighted),
            Self::Sighted => Some(Self::Observed),
            Self::Observed => Some(Self::Studied),
            Self::Studied => Some(Self::Mastered),
            Self::Mastered => None,
        }
    }

    /// Get display name for this tier
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Sighted => "Sighted",
            Self::Observed => "Observed",
            Self::Studied => "Studied",
            Self::Mastered => "Mastered",
        }
    }

    /// Get color for UI display (RGB)
    pub fn color(&self) -> [f32; 3] {
        match self {
            Self::Unknown => [0.3, 0.3, 0.3],    // Gray
            Self::Sighted => [0.6, 0.4, 0.2],    // Bronze
            Self::Observed => [0.7, 0.7, 0.7],   // Silver
            Self::Studied => [0.85, 0.65, 0.1],  // Gold
            Self::Mastered => [0.4, 0.8, 0.4],   // Green (naturalist mastery)
        }
    }
}

/// Information revealed at each discovery tier for fauna
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaunaKnowledge {
    /// Common name (revealed at Sighted)
    pub common_name: Option<String>,
    /// Size category (revealed at Sighted)
    pub size_category: Option<SizeCategory>,
    /// Diet type (revealed at Observed)
    pub diet: Option<DietType>,
    /// Active times (revealed at Observed)
    pub activity_pattern: Option<ActivityPattern>,
    /// Preferred habitats (revealed at Observed)
    pub habitats: Option<Vec<String>>,
    /// Danger level 1-10 (revealed at Studied)
    pub danger_level: Option<u8>,
    /// Attack patterns (revealed at Studied)
    pub attack_descriptions: Option<Vec<String>>,
    /// Weaknesses (revealed at Studied)
    pub weaknesses: Option<Vec<String>>,
    /// Loot drops (revealed at Studied)
    pub loot_items: Option<Vec<String>>,
    /// Scientific/taxonomic name (revealed at Mastered)
    pub scientific_name: Option<String>,
    /// Detailed behavioral notes (revealed at Mastered)
    pub behavioral_notes: Option<String>,
    /// Tracking bonus unlocked (at Mastered)
    pub tracking_bonus: Option<f32>,
}

/// Information revealed at each discovery tier for flora
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloraKnowledge {
    /// Common name (revealed at Sighted)
    pub common_name: Option<String>,
    /// Plant category (revealed at Sighted)
    pub category: Option<PlantCategory>,
    /// Growing seasons (revealed at Observed)
    pub seasons: Option<Vec<Season>>,
    /// Habitat preferences (revealed at Observed)
    pub habitats: Option<Vec<String>>,
    /// Edibility status (revealed at Studied)
    pub edibility: Option<Edibility>,
    /// Medicinal uses (revealed at Studied)
    pub medicinal_uses: Option<Vec<String>>,
    /// Toxicity information (revealed at Studied)
    pub toxicity: Option<ToxicityInfo>,
    /// Harvest yield info (revealed at Studied)
    pub harvest_info: Option<String>,
    /// Scientific name (revealed at Mastered)
    pub scientific_name: Option<String>,
    /// Detailed botanical notes (revealed at Mastered)
    pub botanical_notes: Option<String>,
    /// Cultivation tips (revealed at Mastered)
    pub cultivation_tips: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SizeCategory {
    Tiny,      // Insects, small rodents
    Small,     // Rabbits, snakes
    Medium,    // Foxes, bobcats
    Large,     // Deer, wolves, boar
    VeryLarge, // Bears, cougars
    Massive,   // Alligators, moose
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DietType {
    Herbivore,
    Carnivore,
    Omnivore,
    Insectivore,
    Piscivore, // Fish-eater
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityPattern {
    Diurnal,    // Active during day
    Nocturnal,  // Active at night
    Crepuscular,// Active at dawn/dusk
    Cathemeral, // Active any time
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlantCategory {
    Tree,
    Shrub,
    Herb,
    Fern,
    Moss,
    Fungus,
    Vine,
    Aquatic,
    Grass,
    Crop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Season {
    Spring,
    #[default]
    Summer,
    Fall,
    Winter,
}

impl Season {
    /// Convert day of year (1-365) to season
    pub fn from_day_of_year(day: u32) -> Self {
        match day {
            80..=171 => Self::Spring,   // Mar 21 - Jun 20
            172..=263 => Self::Summer,  // Jun 21 - Sep 20
            264..=354 => Self::Fall,    // Sep 21 - Dec 20
            _ => Self::Winter,           // Dec 21 - Mar 20
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Edibility {
    Safe,           // Safe to eat raw
    CookRequired,   // Must be cooked
    Medicinal,      // Edible but primarily medicinal
    Toxic,          // Poisonous
    Deadly,         // Lethal
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToxicityInfo {
    pub level: ToxicityLevel,
    pub symptoms: Vec<String>,
    pub onset_time: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToxicityLevel {
    None,
    Mild,     // Discomfort, nausea
    Moderate, // Sickness, weakness
    Severe,   // Serious harm
    Lethal,   // Can kill
}

/// Individual observation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationRecord {
    /// When this observation occurred (game time)
    pub timestamp: f64,
    /// Where the observation took place
    pub location: [f32; 3],
    /// Duration of this observation session (seconds)
    pub duration: f32,
    /// Weather conditions during observation
    pub weather: String,
    /// Time of day
    pub time_of_day: String,
    /// Special behaviors witnessed
    pub behaviors_witnessed: Vec<String>,
    /// Player notes (optional)
    pub notes: Option<String>,
}

/// Encyclopedia entry for a single species
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncyclopediaEntry<K> {
    /// Current discovery tier
    pub tier: DiscoveryTier,
    /// Total observation time accumulated
    pub observation_time: f32,
    /// Number of times encountered
    pub encounter_count: u32,
    /// Number killed/harvested
    pub harvest_count: u32,
    /// First discovery timestamp
    pub first_seen: Option<f64>,
    /// Location of first sighting
    pub first_seen_location: Option<[f32; 3]>,
    /// Observation log
    pub observations: Vec<ObservationRecord>,
    /// Knowledge unlocked at current tier
    pub knowledge: K,
    /// Whether a sketch has been made (at Studied tier)
    pub has_sketch: bool,
}

impl<K: Default> Default for EncyclopediaEntry<K> {
    fn default() -> Self {
        Self {
            tier: DiscoveryTier::Unknown,
            observation_time: 0.0,
            encounter_count: 0,
            harvest_count: 0,
            first_seen: None,
            first_seen_location: None,
            observations: Vec::new(),
            knowledge: K::default(),
            has_sketch: false,
        }
    }
}

/// The player's complete field journal / encyclopedia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encyclopedia {
    /// Animal entries
    pub fauna: HashMap<AnimalSpecies, EncyclopediaEntry<FaunaKnowledge>>,
    /// Plant entries
    pub flora: HashMap<FloraSpecies, EncyclopediaEntry<FloraKnowledge>>,
    /// Total species discovered (any tier above Unknown)
    pub total_discovered: u32,
    /// Species at Mastered tier
    pub total_mastered: u32,
    /// Naturalist skill level (affects observation speed)
    pub naturalist_level: u8,
    /// Sketch quality skill (affects sketch bonuses)
    pub sketch_skill: u8,
    /// Current season (affects availability and behaviors)
    pub current_season: Season,
}

impl Default for Encyclopedia {
    fn default() -> Self {
        Self::new()
    }
}

impl Encyclopedia {
    pub fn new() -> Self {
        Self {
            fauna: HashMap::new(),
            flora: HashMap::new(),
            total_discovered: 0,
            total_mastered: 0,
            naturalist_level: 1,
            sketch_skill: 1,
            current_season: Season::Summer,
        }
    }

    /// Get or create fauna entry
    pub fn get_fauna_entry(&mut self, species: AnimalSpecies) -> &mut EncyclopediaEntry<FaunaKnowledge> {
        self.fauna.entry(species).or_insert_with(|| EncyclopediaEntry {
            knowledge: FaunaKnowledge::default(),
            ..Default::default()
        })
    }

    /// Get or create flora entry
    pub fn get_flora_entry(&mut self, species: FloraSpecies) -> &mut EncyclopediaEntry<FloraKnowledge> {
        self.flora.entry(species).or_insert_with(|| EncyclopediaEntry {
            knowledge: FloraKnowledge::default(),
            ..Default::default()
        })
    }

    /// Record an animal sighting
    pub fn record_fauna_sighting(
        &mut self,
        species: AnimalSpecies,
        position: [f32; 3],
        game_time: f64,
    ) {
        let entry = self.get_fauna_entry(species);
        entry.encounter_count += 1;

        if entry.first_seen.is_none() {
            entry.first_seen = Some(game_time);
            entry.first_seen_location = Some(position);
        }

        // Automatic progression to Sighted on first encounter
        if entry.tier == DiscoveryTier::Unknown {
            self.advance_fauna_tier(species);
        }
    }

    /// Record a plant discovery
    pub fn record_flora_sighting(
        &mut self,
        species: FloraSpecies,
        position: [f32; 3],
        game_time: f64,
    ) {
        let entry = self.get_flora_entry(species);
        entry.encounter_count += 1;

        if entry.first_seen.is_none() {
            entry.first_seen = Some(game_time);
            entry.first_seen_location = Some(position);
        }

        if entry.tier == DiscoveryTier::Unknown {
            self.advance_flora_tier(species);
        }
    }

    /// Add observation time to a fauna entry
    pub fn add_fauna_observation_time(
        &mut self,
        species: AnimalSpecies,
        duration: f32,
        position: [f32; 3],
        game_time: f64,
        weather: &str,
        time_of_day: &str,
        behaviors: Vec<String>,
    ) -> bool {
        // Apply naturalist skill bonus (10% per level)
        let skill_bonus = 1.0 + (self.naturalist_level as f32 - 1.0) * 0.1;
        let adjusted_duration = duration * skill_bonus;

        let entry = self.get_fauna_entry(species);
        entry.observation_time += adjusted_duration;

        // Log observation
        entry.observations.push(ObservationRecord {
            timestamp: game_time,
            location: position,
            duration,
            weather: weather.to_string(),
            time_of_day: time_of_day.to_string(),
            behaviors_witnessed: behaviors,
            notes: None,
        });

        // Check for tier advancement
        if let Some(next_tier) = entry.tier.next() {
            let required = next_tier.observation_time_required();
            if entry.observation_time >= required {
                self.advance_fauna_tier(species);
                return true;
            }
        }
        false
    }

    /// Add observation time to a flora entry
    pub fn add_flora_observation_time(
        &mut self,
        species: FloraSpecies,
        duration: f32,
        position: [f32; 3],
        game_time: f64,
        weather: &str,
        time_of_day: &str,
    ) -> bool {
        let skill_bonus = 1.0 + (self.naturalist_level as f32 - 1.0) * 0.1;
        let adjusted_duration = duration * skill_bonus;

        let entry = self.get_flora_entry(species);
        entry.observation_time += adjusted_duration;

        entry.observations.push(ObservationRecord {
            timestamp: game_time,
            location: position,
            duration,
            weather: weather.to_string(),
            time_of_day: time_of_day.to_string(),
            behaviors_witnessed: Vec::new(),
            notes: None,
        });

        if let Some(next_tier) = entry.tier.next() {
            let required = next_tier.observation_time_required();
            if entry.observation_time >= required {
                self.advance_flora_tier(species);
                return true;
            }
        }
        false
    }

    /// Advance fauna to next discovery tier and reveal knowledge
    fn advance_fauna_tier(&mut self, species: AnimalSpecies) {
        let entry = self.get_fauna_entry(species);

        if let Some(next_tier) = entry.tier.next() {
            let was_unknown = entry.tier == DiscoveryTier::Unknown;
            entry.tier = next_tier;

            if was_unknown {
                self.total_discovered += 1;
            }

            if next_tier == DiscoveryTier::Mastered {
                self.total_mastered += 1;
            }

            // Reveal knowledge based on new tier
            self.reveal_fauna_knowledge(species, next_tier);
        }
    }

    /// Advance flora to next discovery tier
    fn advance_flora_tier(&mut self, species: FloraSpecies) {
        let entry = self.get_flora_entry(species);

        if let Some(next_tier) = entry.tier.next() {
            let was_unknown = entry.tier == DiscoveryTier::Unknown;
            entry.tier = next_tier;

            if was_unknown {
                self.total_discovered += 1;
            }

            if next_tier == DiscoveryTier::Mastered {
                self.total_mastered += 1;
            }

            self.reveal_flora_knowledge(species, next_tier);
        }
    }

    /// Populate knowledge fields based on tier for fauna
    fn reveal_fauna_knowledge(&mut self, species: AnimalSpecies, tier: DiscoveryTier) {
        let entry = self.fauna.get_mut(&species).unwrap();
        let knowledge = &mut entry.knowledge;

        match tier {
            DiscoveryTier::Sighted => {
                knowledge.common_name = Some(species.name().to_string());
                knowledge.size_category = Some(species.size_category());
            }
            DiscoveryTier::Observed => {
                knowledge.diet = Some(species.diet_type());
                knowledge.activity_pattern = Some(species.activity_pattern());
                knowledge.habitats = Some(
                    species.habitats()
                        .iter()
                        .map(|h| format!("{:?}", h))
                        .collect()
                );
            }
            DiscoveryTier::Studied => {
                knowledge.danger_level = Some(species.danger_level());
                knowledge.attack_descriptions = Some(
                    species.attacks()
                        .iter()
                        .map(|a| a.name.replace('_', " "))
                        .collect()
                );
                knowledge.weaknesses = Some(vec![format!("{:?}", species.weakness())]);
                knowledge.loot_items = Some(
                    species.loot().iter().map(|s| s.to_string()).collect()
                );
            }
            DiscoveryTier::Mastered => {
                knowledge.scientific_name = Some(species.scientific_name().to_string());
                knowledge.behavioral_notes = Some(species.behavioral_notes().to_string());
                knowledge.tracking_bonus = Some(0.25); // 25% tracking bonus
            }
            _ => {}
        }
    }

    /// Populate knowledge fields based on tier for flora
    fn reveal_flora_knowledge(&mut self, species: FloraSpecies, tier: DiscoveryTier) {
        let entry = self.flora.get_mut(&species).unwrap();
        let knowledge = &mut entry.knowledge;

        match tier {
            DiscoveryTier::Sighted => {
                knowledge.common_name = Some(species.name().to_string());
                knowledge.category = Some(species.category());
            }
            DiscoveryTier::Observed => {
                knowledge.seasons = Some(species.growing_seasons().to_vec());
                knowledge.habitats = Some(
                    species.habitats()
                        .iter()
                        .map(|h| format!("{:?}", h))
                        .collect()
                );
            }
            DiscoveryTier::Studied => {
                knowledge.edibility = Some(species.edibility());
                knowledge.medicinal_uses = Some(species.medicinal_uses());
                knowledge.toxicity = species.toxicity_info();
                knowledge.harvest_info = Some(species.harvest_description().to_string());
            }
            DiscoveryTier::Mastered => {
                knowledge.scientific_name = Some(species.scientific_name().to_string());
                knowledge.botanical_notes = Some(species.botanical_notes().to_string());
                knowledge.cultivation_tips = Some(species.cultivation_tips().to_string());
            }
            _ => {}
        }
    }

    /// Record a fauna kill/harvest
    pub fn record_fauna_harvest(&mut self, species: AnimalSpecies) {
        let entry = self.get_fauna_entry(species);
        entry.harvest_count += 1;
    }

    /// Record a flora harvest
    pub fn record_flora_harvest(&mut self, species: FloraSpecies) {
        let entry = self.get_flora_entry(species);
        entry.harvest_count += 1;
    }

    /// Create a sketch of a species (requires Studied tier, materials)
    pub fn create_fauna_sketch(&mut self, species: AnimalSpecies) -> bool {
        let entry = self.get_fauna_entry(species);
        if entry.tier >= DiscoveryTier::Studied && !entry.has_sketch {
            entry.has_sketch = true;
            true
        } else {
            false
        }
    }

    /// Create a botanical illustration
    pub fn create_flora_sketch(&mut self, species: FloraSpecies) -> bool {
        let entry = self.get_flora_entry(species);
        if entry.tier >= DiscoveryTier::Studied && !entry.has_sketch {
            entry.has_sketch = true;
            true
        } else {
            false
        }
    }

    /// Get discovery progress percentage
    pub fn discovery_progress(&self) -> f32 {
        let total_species = AnimalSpecies::all().count() + FloraSpecies::all().count();
        if total_species == 0 {
            return 0.0;
        }
        (self.total_discovered as f32 / total_species as f32) * 100.0
    }

    /// Get mastery progress percentage
    pub fn mastery_progress(&self) -> f32 {
        let total_species = AnimalSpecies::all().count() + FloraSpecies::all().count();
        if total_species == 0 {
            return 0.0;
        }
        (self.total_mastered as f32 / total_species as f32) * 100.0
    }

    /// Level up naturalist skill
    pub fn gain_naturalist_experience(&mut self, amount: u32) {
        // Simple leveling: 100 XP per level, max level 10
        let xp_per_level = 100;
        let total_xp = (self.naturalist_level as u32 - 1) * xp_per_level + amount;
        self.naturalist_level = ((total_xp / xp_per_level) + 1).min(10) as u8;
    }

    // === PIPELINE HELPER METHODS ===

    /// Simplified sighting record for pipeline
    pub fn record_sighting(&mut self, species: AnimalSpecies) {
        let entry = self.get_fauna_entry(species);
        entry.encounter_count += 1;

        if entry.tier == DiscoveryTier::Unknown {
            self.advance_fauna_tier(species);
        }
    }

    /// Add observation time (simplified for pipeline)
    pub fn add_observation_time(&mut self, species: AnimalSpecies, duration: f32) {
        let skill_bonus = 1.0 + (self.naturalist_level as f32 - 1.0) * 0.1;
        let adjusted_duration = duration * skill_bonus;

        let entry = self.get_fauna_entry(species);
        entry.observation_time += adjusted_duration;
    }

    /// Record a witnessed behavior
    pub fn record_behavior(&mut self, species: AnimalSpecies, behavior: crate::encyclopedia::observer::BehaviorWitnessType) {
        // Behaviors give bonus observation time
        let bonus = behavior.xp_bonus() * 5.0;
        self.add_observation_time(species, bonus);
    }

    /// Add study XP to a species
    pub fn add_study_xp(&mut self, species: AnimalSpecies, xp: f32) {
        let entry = self.get_fauna_entry(species);
        entry.observation_time += xp;
    }

    /// Check if species should advance tier
    pub fn check_tier_advancement(&mut self, species: AnimalSpecies) -> bool {
        let entry = self.get_fauna_entry(species);
        if let Some(next_tier) = entry.tier.next() {
            let required = next_tier.observation_time_required();
            if entry.observation_time >= required {
                self.advance_fauna_tier(species);
                return true;
            }
        }
        false
    }

    /// Record study of a harvested animal
    pub fn record_harvest_study(&mut self, species: AnimalSpecies) {
        self.record_fauna_harvest(species);
        // Harvesting gives significant study bonus
        self.add_observation_time(species, 30.0);
    }

    /// Record flora harvest (for pipeline)
    pub fn record_flora_harvest_simple(&mut self, species: FloraSpecies) {
        let entry = self.get_flora_entry(species);
        entry.harvest_count += 1;
        entry.observation_time += 10.0; // Harvesting teaches about the plant
    }

    /// Get discovered fauna for save data
    pub fn get_discovered_fauna(&self) -> Vec<(String, u32)> {
        self.fauna
            .iter()
            .filter(|(_, entry)| entry.tier > DiscoveryTier::Unknown)
            .map(|(species, entry)| (format!("{:?}", species), entry.tier as u32))
            .collect()
    }

    /// Get discovered flora for save data
    pub fn get_discovered_flora(&self) -> Vec<(String, u32)> {
        self.flora
            .iter()
            .filter(|(_, entry)| entry.tier > DiscoveryTier::Unknown)
            .map(|(species, entry)| (format!("{:?}", species), entry.tier as u32))
            .collect()
    }

    /// Get list of all witnessed behaviors
    pub fn get_witnessed_behaviors(&self) -> Vec<String> {
        let mut behaviors = Vec::new();
        for (species, entry) in &self.fauna {
            for obs in &entry.observations {
                for b in &obs.behaviors_witnessed {
                    let key = format!("{:?}:{}", species, b);
                    if !behaviors.contains(&key) {
                        behaviors.push(key);
                    }
                }
            }
        }
        behaviors
    }

    /// Restore discoveries from save data
    pub fn restore_discoveries(
        &mut self,
        fauna: &[(String, u32)],
        flora: &[(String, u32)],
    ) {
        // Restore fauna discoveries
        for (species_name, tier_val) in fauna {
            if let Some(species) = AnimalSpecies::from_name(species_name) {
                let entry = self.get_fauna_entry(species);
                entry.tier = match tier_val {
                    0 => DiscoveryTier::Unknown,
                    1 => DiscoveryTier::Sighted,
                    2 => DiscoveryTier::Observed,
                    3 => DiscoveryTier::Studied,
                    _ => DiscoveryTier::Mastered,
                };
            }
        }

        // Restore flora discoveries
        for (species_name, tier_val) in flora {
            if let Some(species) = FloraSpecies::from_name(species_name) {
                let entry = self.get_flora_entry(species);
                entry.tier = match tier_val {
                    0 => DiscoveryTier::Unknown,
                    1 => DiscoveryTier::Sighted,
                    2 => DiscoveryTier::Observed,
                    3 => DiscoveryTier::Studied,
                    _ => DiscoveryTier::Mastered,
                };
            }
        }

        // Recalculate totals
        self.total_discovered = self.fauna.values()
            .filter(|e| e.tier > DiscoveryTier::Unknown)
            .count() as u32
            + self.flora.values()
            .filter(|e| e.tier > DiscoveryTier::Unknown)
            .count() as u32;

        self.total_mastered = self.fauna.values()
            .filter(|e| e.tier == DiscoveryTier::Mastered)
            .count() as u32
            + self.flora.values()
            .filter(|e| e.tier == DiscoveryTier::Mastered)
            .count() as u32;
    }
}

impl Default for FaunaKnowledge {
    fn default() -> Self {
        Self {
            common_name: None,
            size_category: None,
            diet: None,
            activity_pattern: None,
            habitats: None,
            danger_level: None,
            attack_descriptions: None,
            weaknesses: None,
            loot_items: None,
            scientific_name: None,
            behavioral_notes: None,
            tracking_bonus: None,
        }
    }
}

impl Default for FloraKnowledge {
    fn default() -> Self {
        Self {
            common_name: None,
            category: None,
            seasons: None,
            habitats: None,
            edibility: None,
            medicinal_uses: None,
            toxicity: None,
            harvest_info: None,
            scientific_name: None,
            botanical_notes: None,
            cultivation_tips: None,
        }
    }
}

// Extension trait for AnimalSpecies to provide encyclopedia data
impl AnimalSpecies {
    pub fn size_category(&self) -> SizeCategory {
        match self {
            Self::Copperhead | Self::Cottonmouth | Self::TimberRattlesnake => SizeCategory::Small,
            Self::Bobcat | Self::Fox => SizeCategory::Medium,
            Self::GrayWolf | Self::RedWolf | Self::WildBoar | Self::Husky | Self::WhitetailDeer => SizeCategory::Large,
            Self::BlackBear | Self::EasternCougar | Self::Stag | Self::Donkey => SizeCategory::VeryLarge,
            Self::AmericanAlligator | Self::Horse => SizeCategory::Massive,
        }
    }

    pub fn diet_type(&self) -> DietType {
        match self {
            Self::WildBoar => DietType::Omnivore,
            Self::BlackBear => DietType::Omnivore,
            _ => DietType::Carnivore,
        }
    }

    pub fn activity_pattern(&self) -> ActivityPattern {
        match self {
            Self::TimberRattlesnake => ActivityPattern::Diurnal,
            Self::AmericanAlligator | Self::Cottonmouth => ActivityPattern::Cathemeral,
            Self::WildBoar | Self::BlackBear => ActivityPattern::Crepuscular,
            _ => ActivityPattern::Nocturnal,
        }
    }

    pub fn scientific_name(&self) -> &'static str {
        match self {
            Self::BlackBear => "Ursus americanus",
            Self::EasternCougar => "Puma concolor couguar",
            Self::GrayWolf => "Canis lupus",
            Self::TimberRattlesnake => "Crotalus horridus",
            Self::AmericanAlligator => "Alligator mississippiensis",
            Self::WildBoar => "Sus scrofa",
            Self::Copperhead => "Agkistrodon contortrix",
            Self::RedWolf => "Canis rufus",
            Self::Bobcat => "Lynx rufus",
            Self::Cottonmouth => "Agkistrodon piscivorus",
            // Docile animals
            Self::WhitetailDeer => "Odocoileus virginianus",
            Self::Stag => "Cervus elaphus",
            Self::Horse => "Equus caballus",
            Self::Donkey => "Equus asinus",
            Self::Fox => "Vulpes vulpes",
            Self::Husky => "Canis lupus familiaris",
        }
    }

    pub fn behavioral_notes(&self) -> &'static str {
        match self {
            Self::BlackBear => "Primarily forages for berries, nuts, and insects. Will become aggressive if cubs are threatened or food sources are approached. Excellent climbers and swimmers.",
            Self::EasternCougar => "Solitary ambush predator. Stalks prey silently before launching a powerful pounce. Most active at dawn and dusk. Rarely attacks humans unless cornered.",
            Self::GrayWolf => "Highly social pack hunters with complex communication. Alpha pairs lead hunting strategies. Can travel 30 miles in a day pursuing prey.",
            Self::TimberRattlesnake => "Generally docile if undisturbed. Rattles tail as warning before striking. Venom is hemotoxic. Hibernates communally in rocky dens during winter.",
            Self::AmericanAlligator => "Apex predator of wetland ecosystems. Uses death roll to tear prey. Females are protective of nests. Can survive brief cold snaps by entering brumation.",
            Self::WildBoar => "Highly adaptable and intelligent. Roots through soil for tubers and grubs. Males have sharp tusks used in dominance fights. Extremely dangerous when wounded.",
            Self::Copperhead => "Relies on camouflage rather than fleeing. Venom is mild compared to other pit vipers. Responsible for most snakebites due to proximity to human habitation.",
            Self::RedWolf => "Smaller and more wary than gray wolves. Hunts in mated pairs or small family groups. Historically persecuted and now extremely rare.",
            Self::Bobcat => "Adaptable predator of rabbits and rodents. Excellent stalker with keen eyesight. Typically avoids humans. Can leap 10 feet to catch birds.",
            Self::Cottonmouth => "Semi-aquatic pit viper. Named for white mouth displayed as threat. Stands ground rather than fleeing. Feeds on fish, frogs, and small mammals.",
            // Docile animals
            Self::WhitetailDeer => "Highly alert and cautious. Communicates danger through tail flagging. Most active at dawn and dusk. Forms loose herds in winter.",
            Self::Stag => "Male deer with impressive antlers used in mating displays and dominance fights. Leads and protects small herds. Can become aggressive during rutting season.",
            Self::Horse => "Wild horses live in family groups led by a dominant stallion. Excellent stamina and speed. Communicates through body language and vocalizations.",
            Self::Donkey => "Hardy and sure-footed pack animal. Intelligent and stubborn. Forms strong bonds with herd mates. Can defend against predators with powerful kicks.",
            Self::Fox => "Opportunistic omnivore and clever hunter. Caches food for later consumption. Solitary except during mating season. Known for intelligence and adaptability.",
            Self::Husky => "Pack-oriented working dog. Loyal and energetic. Excellent endurance for long journeys. Thick coat provides protection against harsh weather.",
        }
    }
}
