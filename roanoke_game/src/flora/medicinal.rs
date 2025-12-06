//! Medicinal and toxicology system
//!
//! Handles the effects of consuming plants, creating remedies, and poison/antidote mechanics.
//! Based on historical colonial-era herbal medicine practices.

use serde::{Deserialize, Serialize};
use super::FloraSpecies;

/// Status effects that can be applied by plants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlantEffect {
    // Beneficial effects
    Healing,           // Restores health over time
    StaminaRestore,    // Restores stamina
    PainRelief,        // Reduces damage taken perception
    FeverReduction,    // Cures fever status
    PoisonResist,      // Reduces poison damage
    ImmuneBoost,       // Reduces disease chance
    WoundCleanse,      // Prevents infection
    BoneHealing,       // Speeds fracture recovery
    DigestionAid,      // Improves food value
    SleepAid,          // Improves rest quality

    // Negative effects
    Nausea,            // Reduces hunger recovery
    Vomiting,          // Empties stomach, damages
    Hallucination,     // Visual distortion
    Paralysis,         // Can't move
    HeartIrregular,    // Stamina problems
    Blindness,         // Temporary vision loss
    Tremors,           // Reduced accuracy
    OrganFailure,      // Delayed lethal damage

    // Neutral/utility
    Stimulant,         // Prevents sleep, boosts awareness
    Sedative,          // Induces drowsiness
    Diuretic,          // Increases thirst
}

impl PlantEffect {
    pub fn is_beneficial(&self) -> bool {
        matches!(
            self,
            Self::Healing
                | Self::StaminaRestore
                | Self::PainRelief
                | Self::FeverReduction
                | Self::PoisonResist
                | Self::ImmuneBoost
                | Self::WoundCleanse
                | Self::BoneHealing
                | Self::DigestionAid
                | Self::SleepAid
        )
    }

    pub fn is_harmful(&self) -> bool {
        matches!(
            self,
            Self::Nausea
                | Self::Vomiting
                | Self::Hallucination
                | Self::Paralysis
                | Self::HeartIrregular
                | Self::Blindness
                | Self::Tremors
                | Self::OrganFailure
        )
    }

    pub fn base_duration(&self) -> f32 {
        match self {
            Self::Healing => 60.0,
            Self::StaminaRestore => 30.0,
            Self::PainRelief => 300.0,
            Self::FeverReduction => 120.0,
            Self::PoisonResist => 600.0,
            Self::ImmuneBoost => 1800.0,
            Self::WoundCleanse => 0.0, // Instant
            Self::BoneHealing => 3600.0,
            Self::DigestionAid => 900.0,
            Self::SleepAid => 600.0,
            Self::Nausea => 300.0,
            Self::Vomiting => 30.0,
            Self::Hallucination => 1800.0,
            Self::Paralysis => 120.0,
            Self::HeartIrregular => 600.0,
            Self::Blindness => 300.0,
            Self::Tremors => 600.0,
            Self::OrganFailure => 86400.0, // 24 hours to death
            Self::Stimulant => 1200.0,
            Self::Sedative => 600.0,
            Self::Diuretic => 300.0,
        }
    }
}

/// A medicinal preparation made from plants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remedy {
    pub name: String,
    pub ingredients: Vec<String>,
    pub effects: Vec<(PlantEffect, f32)>, // Effect and potency
    pub preparation: PreparationType,
    pub shelf_life: f32, // Days before spoiling
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreparationType {
    Raw,       // Eaten/applied directly
    Tea,       // Steeped in hot water
    Poultice,  // Mashed and applied to wounds
    Tincture,  // Extracted in alcohol
    Salve,     // Mixed with fat/oil
    Smoke,     // Burned and inhaled
    Powder,    // Dried and ground
}

impl PreparationType {
    pub fn potency_modifier(&self) -> f32 {
        match self {
            Self::Raw => 0.5,
            Self::Tea => 0.8,
            Self::Poultice => 1.0,
            Self::Tincture => 1.5,
            Self::Salve => 1.2,
            Self::Smoke => 0.6,
            Self::Powder => 1.0,
        }
    }

    pub fn preparation_time(&self) -> f32 {
        match self {
            Self::Raw => 0.0,
            Self::Tea => 5.0,      // 5 minutes
            Self::Poultice => 2.0,
            Self::Tincture => 1440.0, // 1 day
            Self::Salve => 30.0,
            Self::Smoke => 1.0,
            Self::Powder => 10.0,
        }
    }
}

/// Get medicinal effects of a plant when consumed/applied
pub fn get_plant_effects(species: FloraSpecies) -> Vec<(PlantEffect, f32)> {
    match species {
        // Healing herbs
        FloraSpecies::Yarrow => vec![
            (PlantEffect::WoundCleanse, 1.0),
            (PlantEffect::Healing, 0.5),
            (PlantEffect::FeverReduction, 0.3),
        ],
        FloraSpecies::Plantain => vec![
            (PlantEffect::WoundCleanse, 0.8),
            (PlantEffect::Healing, 0.3),
        ],
        FloraSpecies::Comfrey => vec![
            (PlantEffect::BoneHealing, 1.0),
            (PlantEffect::Healing, 0.4),
        ],
        FloraSpecies::Goldenseal => vec![
            (PlantEffect::WoundCleanse, 1.2),
            (PlantEffect::ImmuneBoost, 0.5),
        ],

        // Tonics and stimulants
        FloraSpecies::Ginseng => vec![
            (PlantEffect::StaminaRestore, 1.0),
            (PlantEffect::ImmuneBoost, 0.8),
            (PlantEffect::Stimulant, 0.3),
        ],
        FloraSpecies::Echinacea => vec![
            (PlantEffect::ImmuneBoost, 1.0),
            (PlantEffect::FeverReduction, 0.4),
        ],

        // Pain and sedation
        FloraSpecies::WildLettuce => vec![
            (PlantEffect::PainRelief, 0.8),
            (PlantEffect::Sedative, 0.6),
            (PlantEffect::SleepAid, 0.5),
        ],
        FloraSpecies::BlackCohosh => vec![
            (PlantEffect::PainRelief, 0.5),
            (PlantEffect::Sedative, 0.3),
        ],

        // Respiratory
        FloraSpecies::Mullein => vec![
            (PlantEffect::Healing, 0.3), // Respiratory healing
        ],
        FloraSpecies::Lobelia => vec![
            (PlantEffect::Healing, 0.4),
            (PlantEffect::Nausea, 0.5), // Side effect
        ],

        // Digestive
        FloraSpecies::WildGinger => vec![
            (PlantEffect::DigestionAid, 1.0),
            (PlantEffect::Nausea, -0.5), // Reduces nausea
        ],
        FloraSpecies::Boneset => vec![
            (PlantEffect::FeverReduction, 1.0),
            (PlantEffect::DigestionAid, 0.3),
        ],

        // Topical
        FloraSpecies::JewelWeed => vec![
            (PlantEffect::PoisonResist, 0.8), // Counters poison ivy
            (PlantEffect::Healing, 0.2),
        ],
        FloraSpecies::WitchHazel => vec![
            (PlantEffect::WoundCleanse, 0.7),
            (PlantEffect::Healing, 0.3),
        ],

        // Immune support
        FloraSpecies::Elderberry => vec![
            (PlantEffect::ImmuneBoost, 1.0),
            (PlantEffect::FeverReduction, 0.5),
            (PlantEffect::Nausea, 0.2), // Raw berries
        ],

        // TOXIC PLANTS
        FloraSpecies::JimsonWeed => vec![
            (PlantEffect::Hallucination, 2.0),
            (PlantEffect::HeartIrregular, 1.0),
            (PlantEffect::Paralysis, 0.5),
            (PlantEffect::Blindness, 0.8),
        ],
        FloraSpecies::Foxglove => vec![
            (PlantEffect::HeartIrregular, 2.0),
            (PlantEffect::Nausea, 1.0),
            (PlantEffect::Blindness, 0.5), // Yellow vision
        ],
        FloraSpecies::DestroyingAngel | FloraSpecies::DeathCap => vec![
            (PlantEffect::Nausea, 1.0),
            (PlantEffect::Vomiting, 1.5),
            (PlantEffect::OrganFailure, 2.0),
        ],
        FloraSpecies::Bloodroot => vec![
            (PlantEffect::Nausea, 0.8),
            (PlantEffect::Vomiting, 0.5),
        ],
        FloraSpecies::Pokeweed => vec![
            (PlantEffect::Nausea, 1.0),
            (PlantEffect::Vomiting, 1.0),
            (PlantEffect::Tremors, 0.5),
        ],
        FloraSpecies::Snakeroot => vec![
            (PlantEffect::Tremors, 1.0),
            (PlantEffect::Nausea, 0.8),
        ],

        _ => Vec::new(),
    }
}

/// Common remedy recipes
pub fn get_remedy_recipes() -> Vec<RemedyRecipe> {
    vec![
        RemedyRecipe {
            name: "Wound Poultice".to_string(),
            description: "A mashed herb mixture applied to wounds to prevent infection".to_string(),
            ingredients: vec!["yarrow_leaves".to_string(), "plantain_leaves".to_string()],
            preparation: PreparationType::Poultice,
            effects: vec![
                (PlantEffect::WoundCleanse, 1.5),
                (PlantEffect::Healing, 0.8),
            ],
            skill_required: 1,
        },
        RemedyRecipe {
            name: "Fever Tea".to_string(),
            description: "A bitter tea that reduces fever and aches".to_string(),
            ingredients: vec!["boneset".to_string(), "elderflowers".to_string()],
            preparation: PreparationType::Tea,
            effects: vec![
                (PlantEffect::FeverReduction, 1.2),
                (PlantEffect::ImmuneBoost, 0.5),
            ],
            skill_required: 2,
        },
        RemedyRecipe {
            name: "Pain Tincture".to_string(),
            description: "A potent extract for relieving severe pain".to_string(),
            ingredients: vec!["wild_lettuce".to_string()],
            preparation: PreparationType::Tincture,
            effects: vec![
                (PlantEffect::PainRelief, 1.5),
                (PlantEffect::Sedative, 0.8),
            ],
            skill_required: 4,
        },
        RemedyRecipe {
            name: "Stamina Tonic".to_string(),
            description: "A restorative drink made from ginseng root".to_string(),
            ingredients: vec!["ginseng_root".to_string()],
            preparation: PreparationType::Tea,
            effects: vec![
                (PlantEffect::StaminaRestore, 1.5),
                (PlantEffect::ImmuneBoost, 0.6),
            ],
            skill_required: 3,
        },
        RemedyRecipe {
            name: "Bone-Mend Salve".to_string(),
            description: "Applied to fractures to speed healing".to_string(),
            ingredients: vec!["comfrey".to_string()],
            preparation: PreparationType::Salve,
            effects: vec![
                (PlantEffect::BoneHealing, 1.5),
            ],
            skill_required: 3,
        },
        RemedyRecipe {
            name: "Poison Ivy Remedy".to_string(),
            description: "Crushed jewelweed applied to rashes".to_string(),
            ingredients: vec!["jewelweed".to_string()],
            preparation: PreparationType::Poultice,
            effects: vec![
                (PlantEffect::PoisonResist, 1.0),
                (PlantEffect::Healing, 0.3),
            ],
            skill_required: 1,
        },
        RemedyRecipe {
            name: "Sleeping Draught".to_string(),
            description: "A tea to induce restful sleep".to_string(),
            ingredients: vec!["wild_lettuce".to_string(), "elderflowers".to_string()],
            preparation: PreparationType::Tea,
            effects: vec![
                (PlantEffect::SleepAid, 1.5),
                (PlantEffect::Sedative, 1.0),
            ],
            skill_required: 3,
        },
        RemedyRecipe {
            name: "Antiseptic Wash".to_string(),
            description: "A solution for cleaning wounds".to_string(),
            ingredients: vec!["goldenseal_root".to_string()],
            preparation: PreparationType::Tea,
            effects: vec![
                (PlantEffect::WoundCleanse, 1.8),
            ],
            skill_required: 2,
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemedyRecipe {
    pub name: String,
    pub description: String,
    pub ingredients: Vec<String>,
    pub preparation: PreparationType,
    pub effects: Vec<(PlantEffect, f32)>,
    pub skill_required: u8,
}

/// Active effect on a character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub effect: PlantEffect,
    pub potency: f32,
    pub remaining_duration: f32,
    pub source: String,
}

impl ActiveEffect {
    pub fn new(effect: PlantEffect, potency: f32, source: &str) -> Self {
        Self {
            effect,
            potency,
            remaining_duration: effect.base_duration() * potency,
            source: source.to_string(),
        }
    }

    pub fn update(&mut self, delta_time: f32) -> bool {
        self.remaining_duration -= delta_time;
        self.remaining_duration <= 0.0
    }

    /// Get the current effect magnitude
    pub fn magnitude(&self) -> f32 {
        self.potency
    }
}

/// Tracks all active medicinal/toxic effects on a character
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EffectTracker {
    pub active_effects: Vec<ActiveEffect>,
}

impl EffectTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_effect(&mut self, effect: PlantEffect, potency: f32, source: &str) {
        // Check for existing effect of same type
        if let Some(existing) = self.active_effects.iter_mut().find(|e| e.effect == effect) {
            // Stack: increase potency and refresh duration
            existing.potency = (existing.potency + potency * 0.5).min(3.0);
            existing.remaining_duration = effect.base_duration() * existing.potency;
        } else {
            self.active_effects.push(ActiveEffect::new(effect, potency, source));
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.active_effects.retain_mut(|e| !e.update(delta_time));
    }

    pub fn has_effect(&self, effect: PlantEffect) -> bool {
        self.active_effects.iter().any(|e| e.effect == effect)
    }

    pub fn get_effect_magnitude(&self, effect: PlantEffect) -> f32 {
        self.active_effects
            .iter()
            .filter(|e| e.effect == effect)
            .map(|e| e.magnitude())
            .sum()
    }

    /// Calculate total healing rate modifier
    pub fn healing_modifier(&self) -> f32 {
        let mut modifier = 1.0;

        if self.has_effect(PlantEffect::Healing) {
            modifier += self.get_effect_magnitude(PlantEffect::Healing) * 0.5;
        }
        if self.has_effect(PlantEffect::OrganFailure) {
            modifier *= 0.1; // Severely reduced healing
        }

        modifier
    }

    /// Calculate stamina recovery modifier
    pub fn stamina_modifier(&self) -> f32 {
        let mut modifier = 1.0;

        if self.has_effect(PlantEffect::StaminaRestore) {
            modifier += self.get_effect_magnitude(PlantEffect::StaminaRestore) * 0.3;
        }
        if self.has_effect(PlantEffect::HeartIrregular) {
            modifier *= 0.5;
        }
        if self.has_effect(PlantEffect::Paralysis) {
            modifier = 0.0;
        }

        modifier
    }

    /// Check if character is incapacitated
    pub fn is_incapacitated(&self) -> bool {
        self.has_effect(PlantEffect::Paralysis) || self.has_effect(PlantEffect::OrganFailure)
    }

    /// Get damage per second from toxic effects
    pub fn poison_damage(&self) -> f32 {
        let mut damage = 0.0;

        if self.has_effect(PlantEffect::OrganFailure) {
            damage += 0.1 * self.get_effect_magnitude(PlantEffect::OrganFailure);
        }
        if self.has_effect(PlantEffect::Vomiting) {
            damage += 0.05 * self.get_effect_magnitude(PlantEffect::Vomiting);
        }

        damage
    }

    pub fn clear_all(&mut self) {
        self.active_effects.clear();
    }
}

/// Antidote information for poisons
pub fn get_antidote(poison_source: FloraSpecies) -> Option<Vec<FloraSpecies>> {
    match poison_source {
        // Amanita mushroom poisoning - no effective antidote in colonial era
        FloraSpecies::DestroyingAngel | FloraSpecies::DeathCap => None,

        // Jimsonweed - supportive care only
        FloraSpecies::JimsonWeed => None,

        // Foxglove - emetics might help if caught early
        FloraSpecies::Foxglove => Some(vec![FloraSpecies::Lobelia]), // Induces vomiting

        // Snakeroot - no known antidote
        FloraSpecies::Snakeroot => None,

        // Pokeweed - supportive care
        FloraSpecies::Pokeweed => Some(vec![FloraSpecies::WildGinger]), // Settles stomach

        _ => None,
    }
}

// === PIPELINE INTEGRATION TYPES ===

/// Medicinal information about a plant
#[derive(Debug, Clone)]
pub struct PlantMedicinalInfo {
    pub species: FloraSpecies,
    pub primary_effect: Option<PlantEffect>,
    pub is_toxic: bool,
    pub preparation_required: bool,
}

// Remedy struct is already defined above

/// Medicinal system manager for pipeline integration
#[derive(Debug)]
pub struct MedicinalSystem {
    known_remedies: Vec<Remedy>,
}

impl MedicinalSystem {
    pub fn new() -> Self {
        Self {
            known_remedies: Vec::new(),
        }
    }

    /// Get medicinal info for a plant species
    pub fn get_plant_info(&self, species: FloraSpecies) -> Option<PlantMedicinalInfo> {
        // Get primary medicinal effect
        let primary_effect = self.get_primary_effect(species);

        // Check if toxic
        let is_toxic = self.is_species_toxic(species);

        // Check if preparation needed
        let preparation_required = self.requires_preparation(species);

        if primary_effect.is_some() || is_toxic {
            Some(PlantMedicinalInfo {
                species,
                primary_effect,
                is_toxic,
                preparation_required,
            })
        } else {
            None
        }
    }

    /// Try to craft a remedy from ingredients
    pub fn try_craft(&mut self, ingredients: &[FloraSpecies]) -> Option<Remedy> {
        // Simple crafting logic - combine effects
        let mut effects = Vec::new();

        for &species in ingredients {
            if let Some(effect) = self.get_primary_effect(species) {
                effects.push((effect, 1.0));
            }
        }

        if effects.is_empty() {
            return None;
        }

        let remedy = Remedy {
            name: format!("Herbal Remedy ({})", ingredients.len()),
            ingredients: ingredients.iter().map(|s| s.name().to_string()).collect(),
            effects,
            preparation: PreparationType::Tea,
            shelf_life: 7.0, // 7 days
        };

        self.known_remedies.push(remedy.clone());
        Some(remedy)
    }

    /// Get primary medicinal effect for a species (stub implementation)
    fn get_primary_effect(&self, species: FloraSpecies) -> Option<PlantEffect> {
        // Map species to their primary medicinal effects
        match species {
            FloraSpecies::Ginseng => Some(PlantEffect::Stimulant),
            FloraSpecies::Goldenseal => Some(PlantEffect::WoundCleanse),
            FloraSpecies::Bloodroot => Some(PlantEffect::WoundCleanse),
            FloraSpecies::WildGinger => Some(PlantEffect::DigestionAid),
            FloraSpecies::JewelWeed => Some(PlantEffect::PoisonResist),
            FloraSpecies::Boneset => Some(PlantEffect::FeverReduction),
            FloraSpecies::Yarrow => Some(PlantEffect::WoundCleanse),
            FloraSpecies::Plantain => Some(PlantEffect::WoundCleanse),
            FloraSpecies::Mullein => Some(PlantEffect::PoisonResist),
            FloraSpecies::Comfrey => Some(PlantEffect::BoneHealing),
            FloraSpecies::Echinacea => Some(PlantEffect::ImmuneBoost),
            FloraSpecies::BlackCohosh => Some(PlantEffect::PainRelief),
            FloraSpecies::Lobelia => Some(PlantEffect::Stimulant),
            FloraSpecies::JimsonWeed => Some(PlantEffect::Hallucination),
            FloraSpecies::Foxglove => Some(PlantEffect::HeartIrregular),
            FloraSpecies::Pokeweed => Some(PlantEffect::Vomiting),
            FloraSpecies::Mayapple => Some(PlantEffect::DigestionAid),
            FloraSpecies::WildLettuce => Some(PlantEffect::Sedative),
            FloraSpecies::Snakeroot => Some(PlantEffect::HeartIrregular),
            _ => None,
        }
    }

    /// Check if a species is toxic
    fn is_species_toxic(&self, species: FloraSpecies) -> bool {
        matches!(species,
            FloraSpecies::JimsonWeed |
            FloraSpecies::Foxglove |
            FloraSpecies::Pokeweed |
            FloraSpecies::Mayapple |
            FloraSpecies::JackInThePulpit |
            FloraSpecies::Snakeroot
        )
    }

    /// Check if species requires preparation before use
    fn requires_preparation(&self, species: FloraSpecies) -> bool {
        matches!(species,
            FloraSpecies::Pokeweed |
            FloraSpecies::Mayapple |
            FloraSpecies::JackInThePulpit |
            FloraSpecies::Bloodroot
        )
    }
}
