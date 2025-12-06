//! Faction System
//!
//! Comprehensive faction system with cultural traits, skill trees, weapons,
//! abilities, and inter-faction relationships for the 1580s Roanoke setting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// FACTION DEFINITIONS
// ============================================================================

/// All factions in the game world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    // European Colonial Powers
    Spanish,
    French,
    English,

    // Mesoamerican
    Aztec,

    // Native American Nations
    Powhatan,
    Tuscarora,
    Cherokee,
    Catawba,
    Pamunkey,

    // Special factions
    Independent, // Unaffiliated
    Wildlife,    // Animals (affects hunting reputation)
}

impl Faction {
    /// Get display name for the faction
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Spanish => "Spanish Conquistadors",
            Self::French => "French Coureurs des Bois",
            Self::English => "English Colonists",
            Self::Aztec => "Aztec Remnants",
            Self::Powhatan => "Powhatan Confederacy",
            Self::Tuscarora => "Tuscarora Nation",
            Self::Cherokee => "Cherokee Nation",
            Self::Catawba => "Catawba Nation",
            Self::Pamunkey => "Pamunkey Tribe",
            Self::Independent => "Independent",
            Self::Wildlife => "Wildlife",
        }
    }

    /// Get faction motto/tagline
    pub fn motto(&self) -> &'static str {
        match self {
            Self::Spanish => "God, Gold, and Glory",
            Self::French => "The Forest is Our Cathedral",
            Self::English => "For Queen and Country",
            Self::Aztec => "The Sun Demands Blood",
            Self::Powhatan => "This Land Was Always Ours",
            Self::Tuscarora => "People of the Hemp",
            Self::Cherokee => "Ani-Yunwiya - The Principal People",
            Self::Catawba => "People of the River",
            Self::Pamunkey => "The Rising Corn People",
            Self::Independent => "Beholden to None",
            Self::Wildlife => "Nature's Balance",
        }
    }

    /// Get faction culture type for grouping
    pub fn culture(&self) -> FactionCulture {
        match self {
            Self::Spanish | Self::French | Self::English => FactionCulture::European,
            Self::Aztec => FactionCulture::Mesoamerican,
            Self::Powhatan | Self::Tuscarora | Self::Cherokee | Self::Catawba | Self::Pamunkey => {
                FactionCulture::NativeAmerican
            }
            Self::Independent | Self::Wildlife => FactionCulture::Neutral,
        }
    }

    /// Get all playable factions
    pub fn all_playable() -> &'static [Faction] {
        &[
            Self::Spanish,
            Self::French,
            Self::English,
            Self::Aztec,
            Self::Powhatan,
            Self::Tuscarora,
            Self::Cherokee,
            Self::Catawba,
            Self::Pamunkey,
        ]
    }

    /// Get faction's primary language
    pub fn language(&self) -> &'static str {
        match self {
            Self::Spanish => "Spanish",
            Self::French => "French",
            Self::English => "English",
            Self::Aztec => "Nahuatl",
            Self::Powhatan | Self::Pamunkey => "Algonquian",
            Self::Tuscarora => "Tuscarora (Iroquoian)",
            Self::Cherokee => "Cherokee (Iroquoian)",
            Self::Catawba => "Catawban (Siouan)",
            Self::Independent | Self::Wildlife => "Various",
        }
    }
}

/// Faction culture groupings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactionCulture {
    European,
    Mesoamerican,
    NativeAmerican,
    Neutral,
}

// ============================================================================
// FACTION STANDING SYSTEM
// ============================================================================

/// Standing level with a faction (-3 to +3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Standing {
    War = -3,       // Active warfare, attack on sight
    Hostile = -2,   // Aggressive, very high prices
    Suspicious = -1, // Watched, elevated prices
    Neutral = 0,     // Standard interactions
    Friendly = 1,    // Discounts, side quests
    Allied = 2,      // Deep discounts, training
    BloodBond = 3,   // Full faction benefits
}

impl Standing {
    /// Create standing from reputation value
    pub fn from_reputation(rep: i32) -> Self {
        match rep {
            r if r <= -1000 => Self::War,
            r if r <= -500 => Self::Hostile,
            r if r <= -100 => Self::Suspicious,
            r if r <= 99 => Self::Neutral,
            r if r <= 499 => Self::Friendly,
            r if r <= 999 => Self::Allied,
            _ => Self::BloodBond,
        }
    }

    /// Get numeric value for calculations
    pub fn value(&self) -> i8 {
        match self {
            Self::War => -3,
            Self::Hostile => -2,
            Self::Suspicious => -1,
            Self::Neutral => 0,
            Self::Friendly => 1,
            Self::Allied => 2,
            Self::BloodBond => 3,
        }
    }

    /// Get trade price multiplier
    pub fn trade_multiplier(&self) -> f32 {
        match self {
            Self::War => 0.0,       // No trade
            Self::Hostile => 3.0,    // 300% prices
            Self::Suspicious => 1.5, // 150% prices
            Self::Neutral => 1.0,    // Normal
            Self::Friendly => 0.85,  // 15% discount
            Self::Allied => 0.70,    // 30% discount
            Self::BloodBond => 0.50, // 50% discount
        }
    }

    /// Check if faction members will attack on sight
    pub fn attacks_on_sight(&self) -> bool {
        matches!(self, Self::War | Self::Hostile)
    }

    /// Check if can access faction skill training
    pub fn can_train(&self) -> bool {
        matches!(self, Self::Allied | Self::BloodBond)
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::War => "At war - attacks on sight, bounty placed",
            Self::Hostile => "Hostile - aggressive patrols, restricted areas",
            Self::Suspicious => "Suspicious - watched closely, limited access",
            Self::Neutral => "Neutral - standard interactions",
            Self::Friendly => "Friendly - discounts, side quests available",
            Self::Allied => "Allied - training available, deep trust",
            Self::BloodBond => "Blood Bond - full faction member benefits",
        }
    }
}

impl Default for Standing {
    fn default() -> Self {
        Self::Neutral
    }
}

// ============================================================================
// FACTION RELATIONSHIPS
// ============================================================================

/// Relationship between two factions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionRelationship {
    pub faction_a: Faction,
    pub faction_b: Faction,
    pub base_standing: Standing,
    pub current_standing: Standing,
    pub modifiable: bool,
    pub context: String,
}

impl FactionRelationship {
    pub fn new(
        a: Faction,
        b: Faction,
        standing: Standing,
        modifiable: bool,
        context: &str,
    ) -> Self {
        Self {
            faction_a: a,
            faction_b: b,
            base_standing: standing,
            current_standing: standing,
            modifiable,
            context: context.to_string(),
        }
    }
}

/// Get all default faction relationships
/// Ensures a good mix of friendly, neutral, and hostile relationships
pub fn get_default_relationships() -> Vec<FactionRelationship> {
    use Faction::*;
    use Standing::*;

    vec![
        // ========== SPANISH RELATIONSHIPS ==========
        // Spanish-French: Rivals for New World (Hostile)
        FactionRelationship::new(
            Spanish, French, Hostile, true,
            "European rivals competing for New World dominance"
        ),
        // Spanish-English: Religious/Political enemies (War)
        FactionRelationship::new(
            Spanish, English, War, false,
            "Protestant-Catholic warfare, privateering, the Armada"
        ),
        // Spanish-Aztec: Blood feud from conquest (War - cannot change)
        FactionRelationship::new(
            Spanish, Aztec, War, false,
            "Conquest of Mexico, destruction of Tenochtitlan"
        ),
        // Spanish-Powhatan: Suspicious (wary of Europeans)
        FactionRelationship::new(
            Spanish, Powhatan, Suspicious, true,
            "Unknown Europeans with strange weapons"
        ),
        // Spanish-Tuscarora: Suspicious
        FactionRelationship::new(
            Spanish, Tuscarora, Suspicious, true,
            "Distant but wary contact"
        ),
        // Spanish-Cherokee: Neutral (limited contact)
        FactionRelationship::new(
            Spanish, Cherokee, Neutral, true,
            "Mountain peoples, limited interaction"
        ),
        // Spanish-Catawba: Suspicious
        FactionRelationship::new(
            Spanish, Catawba, Suspicious, true,
            "Trade potential but mutual wariness"
        ),
        // Spanish-Pamunkey: Suspicious
        FactionRelationship::new(
            Spanish, Pamunkey, Suspicious, true,
            "Royal tribe wary of foreign powers"
        ),

        // ========== FRENCH RELATIONSHIPS ==========
        // French-English: Colonial rivals (Suspicious)
        FactionRelationship::new(
            French, English, Suspicious, true,
            "Competing colonial ambitions, but not open war"
        ),
        // French-Aztec: Friendly (enemy of my enemy)
        FactionRelationship::new(
            French, Aztec, Friendly, true,
            "Share hatred of Spanish, trade partners"
        ),
        // French-Powhatan: Allied (excellent trade relations)
        FactionRelationship::new(
            French, Powhatan, Allied, true,
            "Strong fur trade, cultural respect"
        ),
        // French-Tuscarora: Friendly (trade partners)
        FactionRelationship::new(
            French, Tuscarora, Friendly, true,
            "Hemp and fur trade networks"
        ),
        // French-Cherokee: Friendly (mountain trade routes)
        FactionRelationship::new(
            French, Cherokee, Friendly, true,
            "Deerskin trade, peaceful coexistence"
        ),
        // French-Catawba: Allied (strongest Native ally)
        FactionRelationship::new(
            French, Catawba, Allied, true,
            "Close trade partnership, guides"
        ),
        // French-Pamunkey: Friendly
        FactionRelationship::new(
            French, Pamunkey, Friendly, true,
            "Respectful diplomacy"
        ),

        // ========== ENGLISH RELATIONSHIPS ==========
        // English-Aztec: Suspicious (strange newcomers)
        FactionRelationship::new(
            English, Aztec, Suspicious, true,
            "Mutual unfamiliarity, potential alliance against Spanish"
        ),
        // English-Powhatan: Hostile (Roanoke tensions)
        FactionRelationship::new(
            English, Powhatan, Hostile, true,
            "Land disputes, cultural clashes, the Lost Colony"
        ),
        // English-Tuscarora: Suspicious
        FactionRelationship::new(
            English, Tuscarora, Suspicious, true,
            "Limited contact, mutual wariness"
        ),
        // English-Cherokee: Neutral (distant)
        FactionRelationship::new(
            English, Cherokee, Neutral, true,
            "Too far inland for significant contact"
        ),
        // English-Catawba: Neutral (potential trade)
        FactionRelationship::new(
            English, Catawba, Neutral, true,
            "Emerging trade possibilities"
        ),
        // English-Pamunkey: Hostile (primary conflict)
        FactionRelationship::new(
            English, Pamunkey, Hostile, true,
            "Core of the Powhatan resistance"
        ),

        // ========== AZTEC RELATIONSHIPS ==========
        // Aztec-Powhatan: Friendly (shared opposition to Europeans)
        FactionRelationship::new(
            Aztec, Powhatan, Friendly, true,
            "Indigenous solidarity, trade of medicine and weapons"
        ),
        // Aztec-Tuscarora: Allied (strong bonds)
        FactionRelationship::new(
            Aztec, Tuscarora, Allied, true,
            "Deep respect, exchange of martial traditions"
        ),
        // Aztec-Cherokee: Friendly
        FactionRelationship::new(
            Aztec, Cherokee, Friendly, true,
            "Warrior cultures appreciate each other"
        ),
        // Aztec-Catawba: Neutral
        FactionRelationship::new(
            Aztec, Catawba, Neutral, true,
            "Distant but respectful"
        ),
        // Aztec-Pamunkey: Friendly
        FactionRelationship::new(
            Aztec, Pamunkey, Friendly, true,
            "Share knowledge of resistance"
        ),

        // ========== NATIVE AMERICAN INTER-RELATIONSHIPS ==========
        // Powhatan-Tuscarora: Suspicious (competing confederacies)
        FactionRelationship::new(
            Powhatan, Tuscarora, Suspicious, true,
            "Border tensions, different language families"
        ),
        // Powhatan-Cherokee: Hostile (historical enemies)
        FactionRelationship::new(
            Powhatan, Cherokee, Hostile, true,
            "Generational warfare, raiding history"
        ),
        // Powhatan-Catawba: Suspicious (competition)
        FactionRelationship::new(
            Powhatan, Catawba, Suspicious, true,
            "Trade competition, occasional raids"
        ),
        // Powhatan-Pamunkey: BloodBond (same confederacy)
        FactionRelationship::new(
            Powhatan, Pamunkey, BloodBond, false,
            "Pamunkey is the paramount tribe of the Confederacy"
        ),

        // Tuscarora-Cherokee: Allied (Iroquoian kinship)
        FactionRelationship::new(
            Tuscarora, Cherokee, Allied, true,
            "Shared Iroquoian language family, cultural ties"
        ),
        // Tuscarora-Catawba: Friendly
        FactionRelationship::new(
            Tuscarora, Catawba, Friendly, true,
            "Piedmont neighbors, trade partners"
        ),
        // Tuscarora-Pamunkey: Neutral
        FactionRelationship::new(
            Tuscarora, Pamunkey, Neutral, true,
            "Respectful distance"
        ),

        // Cherokee-Catawba: Friendly (trade)
        FactionRelationship::new(
            Cherokee, Catawba, Friendly, true,
            "Mountain-river trade routes"
        ),
        // Cherokee-Pamunkey: Suspicious (Powhatan connection)
        FactionRelationship::new(
            Cherokee, Pamunkey, Suspicious, true,
            "Tension due to Powhatan alliance"
        ),

        // Catawba-Pamunkey: Neutral
        FactionRelationship::new(
            Catawba, Pamunkey, Neutral, true,
            "Little direct interaction"
        ),
    ]
}

/// Relationship matrix for quick lookup
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionRelationshipMatrix {
    /// Standing between factions (key: (a, b) where a < b alphabetically)
    relationships: HashMap<(Faction, Faction), FactionRelationship>,
}

impl FactionRelationshipMatrix {
    pub fn new() -> Self {
        let mut matrix = Self::default();
        for rel in get_default_relationships() {
            matrix.set_relationship(rel);
        }
        matrix
    }

    /// Normalize faction pair ordering for consistent key lookup
    fn normalize_pair(a: Faction, b: Faction) -> (Faction, Faction) {
        if (a as u8) <= (b as u8) {
            (a, b)
        } else {
            (b, a)
        }
    }

    /// Get relationship between two factions
    pub fn get_standing(&self, a: Faction, b: Faction) -> Standing {
        if a == b {
            return Standing::BloodBond; // Same faction
        }

        let key = Self::normalize_pair(a, b);
        self.relationships
            .get(&key)
            .map(|r| r.current_standing)
            .unwrap_or(Standing::Neutral)
    }

    /// Set or update a relationship
    pub fn set_relationship(&mut self, rel: FactionRelationship) {
        let key = Self::normalize_pair(rel.faction_a, rel.faction_b);
        self.relationships.insert(key, rel);
    }

    /// Modify standing between factions (if allowed)
    pub fn modify_standing(&mut self, a: Faction, b: Faction, delta: i8) -> bool {
        let key = Self::normalize_pair(a, b);

        if let Some(rel) = self.relationships.get_mut(&key) {
            if !rel.modifiable {
                return false;
            }

            let current_value = rel.current_standing.value();
            let new_value = (current_value + delta).clamp(-3, 3);

            rel.current_standing = match new_value {
                -3 => Standing::War,
                -2 => Standing::Hostile,
                -1 => Standing::Suspicious,
                0 => Standing::Neutral,
                1 => Standing::Friendly,
                2 => Standing::Allied,
                3 => Standing::BloodBond,
                _ => Standing::Neutral,
            };
            true
        } else {
            false
        }
    }

    /// Get all factions hostile to a given faction
    pub fn get_hostile_factions(&self, faction: Faction) -> Vec<Faction> {
        Faction::all_playable()
            .iter()
            .filter(|&&f| f != faction && self.get_standing(faction, f).attacks_on_sight())
            .copied()
            .collect()
    }

    /// Get all factions allied with a given faction
    pub fn get_allied_factions(&self, faction: Faction) -> Vec<Faction> {
        Faction::all_playable()
            .iter()
            .filter(|&&f| {
                f != faction
                    && matches!(
                        self.get_standing(faction, f),
                        Standing::Allied | Standing::BloodBond
                    )
            })
            .copied()
            .collect()
    }

    /// Get all factions with friendly or better relations
    pub fn get_friendly_factions(&self, faction: Faction) -> Vec<Faction> {
        Faction::all_playable()
            .iter()
            .filter(|&&f| f != faction && self.get_standing(faction, f).value() >= 1)
            .copied()
            .collect()
    }
}

// ============================================================================
// FACTION TRAITS
// ============================================================================

/// Cultural trait providing passive bonuses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionTrait {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub effect: TraitEffect,
}

/// Types of trait effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraitEffect {
    /// Damage modifier for weapon type
    DamageModifier {
        weapon_type: Option<WeaponCategory>,
        multiplier: f32,
    },
    /// Stat bonus (e.g., +15% health)
    StatBonus { stat: StatType, bonus: f32 },
    /// Detection range modifier
    DetectionModifier { multiplier: f32 },
    /// Resource gathering yield
    ResourceYield {
        resource: ResourceType,
        multiplier: f32,
    },
    /// Starting reputation with factions
    ReputationBonus {
        factions: Vec<Faction>,
        amount: i32,
    },
    /// Movement speed in terrain
    MovementModifier {
        terrain: Option<TerrainType>,
        multiplier: f32,
    },
    /// Crafting quality bonus
    CraftingBonus {
        category: CraftingCategory,
        quality_bonus: f32,
    },
    /// Resistance to damage type
    Resistance { damage_type: DamageType, reduction: f32 },
    /// Passive health regeneration
    HealthRegen { rate: f32, condition: RegenCondition },
    /// Kill effect (e.g., health on kill)
    OnKillEffect { effect: KillEffect, amount: f32 },
    /// Trade price modifier
    TradeModifier { multiplier: f32 },
    /// Reload speed bonus
    ReloadModifier { multiplier: f32 },
    /// Carry capacity bonus
    CarryCapacity { bonus: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatType {
    Health,
    Stamina,
    Damage,
    Speed,
    Accuracy,
    Stealth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Pelts,
    Food,
    Treasure,
    Plants,
    Fish,
    Crops,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainType {
    Forest,
    Mountain,
    River,
    Snow,
    Swamp,
    Grassland,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CraftingCategory {
    Weapons,
    Armor,
    Traps,
    Medicine,
    Pottery,
    Textiles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
    Fire,
    Poison,
    Cold,
    Bleed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegenCondition {
    Always,
    InForest,
    NearWater,
    InSacredSite,
    DuringDay,
    DuringNight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KillEffect {
    RestoreHealth,
    RestoreStamina,
    GainDamageBoost,
}

/// Get all traits for a faction
pub fn get_faction_traits(faction: Faction) -> Vec<FactionTrait> {
    match faction {
        Faction::Spanish => vec![
            FactionTrait {
                id: "steel_supremacy",
                name: "Steel Supremacy",
                description: "+15% melee damage with metal weapons",
                effect: TraitEffect::DamageModifier {
                    weapon_type: Some(WeaponCategory::MetalMelee),
                    multiplier: 1.15,
                },
            },
            FactionTrait {
                id: "gunpowder_mastery",
                name: "Gunpowder Mastery",
                description: "-20% reload time for firearms",
                effect: TraitEffect::ReloadModifier { multiplier: 0.80 },
            },
            FactionTrait {
                id: "inquisitors_eye",
                name: "Inquisitor's Eye",
                description: "+25% detection of hidden enemies",
                effect: TraitEffect::DetectionModifier { multiplier: 1.25 },
            },
            FactionTrait {
                id: "gold_fever",
                name: "Gold Fever",
                description: "+30% treasure detection range",
                effect: TraitEffect::ResourceYield {
                    resource: ResourceType::Treasure,
                    multiplier: 1.30,
                },
            },
            FactionTrait {
                id: "conquistador_constitution",
                name: "Conquistador's Constitution",
                description: "+10% disease resistance",
                effect: TraitEffect::Resistance {
                    damage_type: DamageType::Poison,
                    reduction: 0.10,
                },
            },
        ],
        Faction::French => vec![
            FactionTrait {
                id: "voyageur_endurance",
                name: "Voyageur's Endurance",
                description: "+20% stamina, +15% carrying capacity",
                effect: TraitEffect::StatBonus {
                    stat: StatType::Stamina,
                    bonus: 0.20,
                },
            },
            FactionTrait {
                id: "trade_tongue",
                name: "Trade Tongue",
                description: "+1 starting reputation with all Native factions",
                effect: TraitEffect::ReputationBonus {
                    factions: vec![
                        Faction::Powhatan,
                        Faction::Tuscarora,
                        Faction::Cherokee,
                        Faction::Catawba,
                        Faction::Pamunkey,
                    ],
                    amount: 100,
                },
            },
            FactionTrait {
                id: "master_trapper",
                name: "Master Trapper",
                description: "+40% pelt quality from trapped animals",
                effect: TraitEffect::ResourceYield {
                    resource: ResourceType::Pelts,
                    multiplier: 1.40,
                },
            },
            FactionTrait {
                id: "river_runner",
                name: "River Runner",
                description: "+30% canoe speed, no stamina cost for paddling",
                effect: TraitEffect::MovementModifier {
                    terrain: Some(TerrainType::River),
                    multiplier: 1.30,
                },
            },
            FactionTrait {
                id: "winter_hardened",
                name: "Winter Hardened",
                description: "No movement penalty in snow, -50% cold damage",
                effect: TraitEffect::Resistance {
                    damage_type: DamageType::Cold,
                    reduction: 0.50,
                },
            },
        ],
        Faction::English => vec![
            FactionTrait {
                id: "colonial_grit",
                name: "Colonial Grit",
                description: "+15% health, slower starvation",
                effect: TraitEffect::StatBonus {
                    stat: StatType::Health,
                    bonus: 0.15,
                },
            },
            FactionTrait {
                id: "naval_connections",
                name: "Naval Connections",
                description: "Access to monthly ship-delivered supplies",
                effect: TraitEffect::TradeModifier { multiplier: 0.90 },
            },
            FactionTrait {
                id: "protestant_work_ethic",
                name: "Protestant Work Ethic",
                description: "+25% construction speed",
                effect: TraitEffect::CraftingBonus {
                    category: CraftingCategory::Weapons,
                    quality_bonus: 0.25,
                },
            },
            FactionTrait {
                id: "common_law",
                name: "Common Law",
                description: "Reduced reputation loss from crimes",
                effect: TraitEffect::ReputationBonus {
                    factions: vec![Faction::English],
                    amount: 50,
                },
            },
            FactionTrait {
                id: "island_mentality",
                name: "Island Mentality",
                description: "+20% defense in fortified structures",
                effect: TraitEffect::Resistance {
                    damage_type: DamageType::Physical,
                    reduction: 0.20,
                },
            },
        ],
        Faction::Aztec => vec![
            FactionTrait {
                id: "jaguars_heart",
                name: "Jaguar's Heart",
                description: "+20% melee damage, +10% attack speed",
                effect: TraitEffect::DamageModifier {
                    weapon_type: Some(WeaponCategory::Melee),
                    multiplier: 1.20,
                },
            },
            FactionTrait {
                id: "eagles_vision",
                name: "Eagle's Vision",
                description: "See enemy health bars, +30% tracking range",
                effect: TraitEffect::DetectionModifier { multiplier: 1.30 },
            },
            FactionTrait {
                id: "obsidian_edge",
                name: "Obsidian Edge",
                description: "Obsidian weapons cause +50% bleed damage",
                effect: TraitEffect::DamageModifier {
                    weapon_type: Some(WeaponCategory::Obsidian),
                    multiplier: 1.50,
                },
            },
            FactionTrait {
                id: "sacred_calendar",
                name: "Sacred Calendar",
                description: "Bonuses on specific days (+25% random stat)",
                effect: TraitEffect::StatBonus {
                    stat: StatType::Damage,
                    bonus: 0.25,
                },
            },
            FactionTrait {
                id: "blood_sacrifice",
                name: "Blood Sacrifice",
                description: "Killing enemies restores 5% health",
                effect: TraitEffect::OnKillEffect {
                    effect: KillEffect::RestoreHealth,
                    amount: 0.05,
                },
            },
        ],
        Faction::Powhatan => vec![
            FactionTrait {
                id: "children_of_ahone",
                name: "Children of Ahone",
                description: "+15% all stats near sacred sites",
                effect: TraitEffect::HealthRegen {
                    rate: 1.0,
                    condition: RegenCondition::InSacredSite,
                },
            },
            FactionTrait {
                id: "tidewater_mastery",
                name: "Tidewater Mastery",
                description: "+30% fishing yield, water navigation bonus",
                effect: TraitEffect::ResourceYield {
                    resource: ResourceType::Fish,
                    multiplier: 1.30,
                },
            },
            FactionTrait {
                id: "confederacy_networks",
                name: "Confederacy Networks",
                description: "+2 starting reputation with allied tribes",
                effect: TraitEffect::ReputationBonus {
                    factions: vec![Faction::Pamunkey],
                    amount: 200,
                },
            },
            FactionTrait {
                id: "corn_mothers_blessing",
                name: "Corn Mother's Blessing",
                description: "+25% crop growth, Three Sisters farming",
                effect: TraitEffect::ResourceYield {
                    resource: ResourceType::Crops,
                    multiplier: 1.25,
                },
            },
            FactionTrait {
                id: "werowance_authority",
                name: "Werowance Authority",
                description: "Can command allied tribe members",
                effect: TraitEffect::StatBonus {
                    stat: StatType::Damage,
                    bonus: 0.10,
                },
            },
        ],
        Faction::Tuscarora => vec![
            FactionTrait {
                id: "hemp_weavers",
                name: "Hemp Weavers",
                description: "+30% rope/textile crafting quality",
                effect: TraitEffect::CraftingBonus {
                    category: CraftingCategory::Textiles,
                    quality_bonus: 0.30,
                },
            },
            FactionTrait {
                id: "longhouse_unity",
                name: "Longhouse Unity",
                description: "+10% all stats when near clan members",
                effect: TraitEffect::StatBonus {
                    stat: StatType::Health,
                    bonus: 0.10,
                },
            },
            FactionTrait {
                id: "three_sisters_masters",
                name: "Three Sisters Masters",
                description: "+40% crop yield for corn/beans/squash",
                effect: TraitEffect::ResourceYield {
                    resource: ResourceType::Crops,
                    multiplier: 1.40,
                },
            },
            FactionTrait {
                id: "piedmont_pathfinders",
                name: "Piedmont Pathfinders",
                description: "+20% movement speed on hills",
                effect: TraitEffect::MovementModifier {
                    terrain: Some(TerrainType::Mountain),
                    multiplier: 1.20,
                },
            },
            FactionTrait {
                id: "revenge_tradition",
                name: "Revenge Tradition",
                description: "+25% damage vs faction that last killed you",
                effect: TraitEffect::DamageModifier {
                    weapon_type: None,
                    multiplier: 1.25,
                },
            },
        ],
        Faction::Cherokee => vec![
            FactionTrait {
                id: "mountain_born",
                name: "Mountain Born",
                description: "No terrain penalties in mountains/hills",
                effect: TraitEffect::MovementModifier {
                    terrain: Some(TerrainType::Mountain),
                    multiplier: 1.0,
                },
            },
            FactionTrait {
                id: "seven_clan_system",
                name: "Seven Clan System",
                description: "Always have refuge in any Cherokee town",
                effect: TraitEffect::ReputationBonus {
                    factions: vec![Faction::Cherokee],
                    amount: 100,
                },
            },
            FactionTrait {
                id: "ballplay_champions",
                name: "Ballplay Champions",
                description: "+15% all physical stats",
                effect: TraitEffect::StatBonus {
                    stat: StatType::Stamina,
                    bonus: 0.15,
                },
            },
            FactionTrait {
                id: "fire_keepers",
                name: "Fire Keepers",
                description: "Fire-based abilities +30% effective",
                effect: TraitEffect::DamageModifier {
                    weapon_type: Some(WeaponCategory::Fire),
                    multiplier: 1.30,
                },
            },
            FactionTrait {
                id: "didanawisgi_blessing",
                name: "Didanawisgi Blessing",
                description: "Herbal remedies +50% effectiveness",
                effect: TraitEffect::CraftingBonus {
                    category: CraftingCategory::Medicine,
                    quality_bonus: 0.50,
                },
            },
        ],
        Faction::Catawba => vec![
            FactionTrait {
                id: "river_warriors",
                name: "River Warriors",
                description: "+20% combat effectiveness near water",
                effect: TraitEffect::DamageModifier {
                    weapon_type: None,
                    multiplier: 1.20,
                },
            },
            FactionTrait {
                id: "master_potters",
                name: "Master Potters",
                description: "Pottery crafting +50%, trade value +30%",
                effect: TraitEffect::CraftingBonus {
                    category: CraftingCategory::Pottery,
                    quality_bonus: 0.50,
                },
            },
            FactionTrait {
                id: "slave_trade_knowledge",
                name: "Slave Trade Knowledge",
                description: "Can capture and sell enemies",
                effect: TraitEffect::TradeModifier { multiplier: 1.30 },
            },
            FactionTrait {
                id: "flathead_identity",
                name: "Flathead Identity",
                description: "Immune to intimidation effects",
                effect: TraitEffect::Resistance {
                    damage_type: DamageType::Physical,
                    reduction: 0.0,
                },
            },
            FactionTrait {
                id: "trading_post_savvy",
                name: "Trading Post Savvy",
                description: "Best prices from all European factions",
                effect: TraitEffect::TradeModifier { multiplier: 0.80 },
            },
        ],
        Faction::Pamunkey => vec![
            FactionTrait {
                id: "royal_blood",
                name: "Royal Blood",
                description: "+2 reputation with all Powhatan tribes",
                effect: TraitEffect::ReputationBonus {
                    factions: vec![Faction::Powhatan],
                    amount: 200,
                },
            },
            FactionTrait {
                id: "corn_lords",
                name: "Corn Lords",
                description: "+50% corn yield, never starve",
                effect: TraitEffect::ResourceYield {
                    resource: ResourceType::Crops,
                    multiplier: 1.50,
                },
            },
            FactionTrait {
                id: "keeper_of_secrets",
                name: "Keeper of Secrets",
                description: "Access to confederacy lore and locations",
                effect: TraitEffect::DetectionModifier { multiplier: 1.20 },
            },
            FactionTrait {
                id: "diplomatic_immunity",
                name: "Diplomatic Immunity",
                description: "Cannot be attacked in neutral villages",
                effect: TraitEffect::Resistance {
                    damage_type: DamageType::Physical,
                    reduction: 1.0,
                },
            },
            FactionTrait {
                id: "chosen_people",
                name: "Chosen People",
                description: "+15% all stats in Powhatan territory",
                effect: TraitEffect::StatBonus {
                    stat: StatType::Health,
                    bonus: 0.15,
                },
            },
        ],
        Faction::Independent | Faction::Wildlife => vec![],
    }
}

// ============================================================================
// FACTION WEAPONS
// ============================================================================

/// Weapon categories for trait effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponCategory {
    Melee,
    MetalMelee,
    Obsidian,
    Bow,
    Firearm,
    Throwing,
    Spear,
    Club,
    Dagger,
    Fire,
}

/// A faction-specific weapon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionWeapon {
    pub id: &'static str,
    pub name: &'static str,
    pub faction: Faction,
    pub category: WeaponCategory,
    pub base_damage: u32,
    pub range: f32,
    pub attack_speed: f32,
    pub properties: Vec<WeaponProperty>,
    pub required_standing: Standing,
    pub description: &'static str,
}

/// Special weapon properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeaponProperty {
    /// Chance to cause bleeding
    BleedChance(f32),
    /// Armor penetration percentage
    ArmorPenetration(f32),
    /// Stun duration on hit
    StunDuration(f32),
    /// Bonus damage vs target type
    BonusVsType { target: TargetType, multiplier: f32 },
    /// Thrown weapon returns on miss
    ReturnOnMiss,
    /// Silent attack (doesn't alert others)
    SilentAttack,
    /// Draw speed bonus for bows
    DrawSpeedBonus(f32),
    /// Parry window extension
    ParryWindowBonus(f32),
    /// Reload speed bonus
    ReloadSpeedBonus(f32),
    /// Can be used while mounted
    MountedUse,
    /// Fire damage over time
    FireDamage(f32),
    /// Poison damage over time
    PoisonDamage(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetType {
    Human,
    Animal,
    Armored,
    Fleeing,
    Unaware,
    Prone,
}

/// Get all weapons for a faction
pub fn get_faction_weapons(faction: Faction) -> Vec<FactionWeapon> {
    match faction {
        Faction::Spanish => vec![
            FactionWeapon {
                id: "toledo_espada_ropera",
                name: "Toledo Espada Ropera",
                faction: Faction::Spanish,
                category: WeaponCategory::MetalMelee,
                base_damage: 45,
                range: 2.0,
                attack_speed: 1.2,
                properties: vec![
                    WeaponProperty::ParryWindowBonus(0.20),
                    WeaponProperty::BleedChance(0.15),
                ],
                required_standing: Standing::Friendly,
                description: "A masterwork rapier from the Toledo forges",
            },
            FactionWeapon {
                id: "conquistador_rodela",
                name: "Conquistador's Rodela",
                faction: Faction::Spanish,
                category: WeaponCategory::MetalMelee,
                base_damage: 5,
                range: 1.0,
                attack_speed: 1.5,
                properties: vec![
                    WeaponProperty::StunDuration(1.0),
                    WeaponProperty::ParryWindowBonus(0.25),
                ],
                required_standing: Standing::Neutral,
                description: "A steel buckler for close-quarters defense",
            },
            FactionWeapon {
                id: "spanish_arquebus",
                name: "Spanish Arquebus",
                faction: Faction::Spanish,
                category: WeaponCategory::Firearm,
                base_damage: 120,
                range: 80.0,
                attack_speed: 0.2,
                properties: vec![
                    WeaponProperty::ArmorPenetration(0.40),
                ],
                required_standing: Standing::Allied,
                description: "A heavy matchlock firearm with superior accuracy",
            },
            FactionWeapon {
                id: "alabarda",
                name: "Alabarda",
                faction: Faction::Spanish,
                category: WeaponCategory::Spear,
                base_damage: 55,
                range: 3.5,
                attack_speed: 0.8,
                properties: vec![
                    WeaponProperty::ArmorPenetration(0.30),
                    WeaponProperty::BonusVsType { target: TargetType::Armored, multiplier: 1.50 },
                ],
                required_standing: Standing::Allied,
                description: "A polearm combining spear and axe",
            },
            FactionWeapon {
                id: "daga_de_misericordia",
                name: "Daga de Misericordia",
                faction: Faction::Spanish,
                category: WeaponCategory::Dagger,
                base_damage: 25,
                range: 1.0,
                attack_speed: 2.0,
                properties: vec![
                    WeaponProperty::BonusVsType { target: TargetType::Prone, multiplier: 2.0 },
                    WeaponProperty::ArmorPenetration(0.50),
                ],
                required_standing: Standing::Friendly,
                description: "A mercy dagger for finishing wounded foes",
            },
        ],
        Faction::French => vec![
            FactionWeapon {
                id: "tomahawk_francais",
                name: "Tomahawk Fran\u{e7}ais",
                faction: Faction::French,
                category: WeaponCategory::Throwing,
                base_damage: 35,
                range: 15.0,
                attack_speed: 1.0,
                properties: vec![
                    WeaponProperty::ReturnOnMiss,
                    WeaponProperty::BonusVsType { target: TargetType::Fleeing, multiplier: 1.50 },
                ],
                required_standing: Standing::Neutral,
                description: "A French trade tomahawk, balanced for throwing",
            },
            FactionWeapon {
                id: "fusil_de_chasse",
                name: "Fusil de Chasse",
                faction: Faction::French,
                category: WeaponCategory::Firearm,
                base_damage: 90,
                range: 60.0,
                attack_speed: 0.25,
                properties: vec![
                    WeaponProperty::BonusVsType { target: TargetType::Animal, multiplier: 1.30 },
                ],
                required_standing: Standing::Friendly,
                description: "A French hunting rifle prized for accuracy",
            },
            FactionWeapon {
                id: "couteau_de_traite",
                name: "Couteau de Traite",
                faction: Faction::French,
                category: WeaponCategory::Dagger,
                base_damage: 20,
                range: 1.0,
                attack_speed: 2.2,
                properties: vec![
                    WeaponProperty::SilentAttack,
                ],
                required_standing: Standing::Neutral,
                description: "A trade knife, perfect for skinning and silent work",
            },
            FactionWeapon {
                id: "voyageurs_paddle",
                name: "Voyageur's Paddle",
                faction: Faction::French,
                category: WeaponCategory::Club,
                base_damage: 30,
                range: 2.0,
                attack_speed: 1.0,
                properties: vec![
                    WeaponProperty::StunDuration(0.5),
                ],
                required_standing: Standing::Neutral,
                description: "A sturdy canoe paddle that doubles as a weapon",
            },
            FactionWeapon {
                id: "musket_hatchet_combo",
                name: "Musket-Hatchet Combo",
                faction: Faction::French,
                category: WeaponCategory::Firearm,
                base_damage: 70,
                range: 50.0,
                attack_speed: 0.3,
                properties: vec![
                    WeaponProperty::MountedUse,
                ],
                required_standing: Standing::Allied,
                description: "A compact musket with integrated hatchet head",
            },
        ],
        Faction::English => vec![
            FactionWeapon {
                id: "english_longbow",
                name: "English Longbow",
                faction: Faction::English,
                category: WeaponCategory::Bow,
                base_damage: 50,
                range: 120.0,
                attack_speed: 0.6,
                properties: vec![
                    WeaponProperty::ArmorPenetration(0.25),
                    WeaponProperty::DrawSpeedBonus(0.15),
                ],
                required_standing: Standing::Friendly,
                description: "The legendary weapon of English yeomen",
            },
            FactionWeapon {
                id: "brown_bess_musket",
                name: "Brown Bess Musket",
                faction: Faction::English,
                category: WeaponCategory::Firearm,
                base_damage: 100,
                range: 70.0,
                attack_speed: 0.22,
                properties: vec![],
                required_standing: Standing::Allied,
                description: "A reliable military musket with bayonet mount",
            },
            FactionWeapon {
                id: "hanger_sword",
                name: "Hanger Sword",
                faction: Faction::English,
                category: WeaponCategory::MetalMelee,
                base_damage: 40,
                range: 1.8,
                attack_speed: 1.4,
                properties: vec![],
                required_standing: Standing::Neutral,
                description: "A short naval sword for close combat",
            },
            FactionWeapon {
                id: "billhook",
                name: "Billhook",
                faction: Faction::English,
                category: WeaponCategory::Spear,
                base_damage: 45,
                range: 3.0,
                attack_speed: 0.9,
                properties: vec![
                    WeaponProperty::BonusVsType { target: TargetType::Armored, multiplier: 1.30 },
                ],
                required_standing: Standing::Friendly,
                description: "A hooked polearm for pulling riders down",
            },
            FactionWeapon {
                id: "buckler_and_cudgel",
                name: "Buckler & Cudgel",
                faction: Faction::English,
                category: WeaponCategory::Club,
                base_damage: 25,
                range: 1.5,
                attack_speed: 1.3,
                properties: vec![
                    WeaponProperty::StunDuration(1.5),
                    WeaponProperty::ParryWindowBonus(0.15),
                ],
                required_standing: Standing::Neutral,
                description: "A non-lethal combination for subduing foes",
            },
        ],
        Faction::Aztec => vec![
            FactionWeapon {
                id: "macuahuitl",
                name: "Macuahuitl",
                faction: Faction::Aztec,
                category: WeaponCategory::Obsidian,
                base_damage: 55,
                range: 2.2,
                attack_speed: 1.0,
                properties: vec![
                    WeaponProperty::BleedChance(0.35),
                ],
                required_standing: Standing::Friendly,
                description: "The obsidian-edged sword-club of the Mexica",
            },
            FactionWeapon {
                id: "tepoztopilli",
                name: "Tepoztopilli",
                faction: Faction::Aztec,
                category: WeaponCategory::Obsidian,
                base_damage: 50,
                range: 3.5,
                attack_speed: 0.85,
                properties: vec![
                    WeaponProperty::ArmorPenetration(0.30),
                ],
                required_standing: Standing::Friendly,
                description: "An obsidian-tipped spear for thrusting",
            },
            FactionWeapon {
                id: "atlatl",
                name: "Atlatl & Tlacochtli",
                faction: Faction::Aztec,
                category: WeaponCategory::Throwing,
                base_damage: 40,
                range: 40.0,
                attack_speed: 0.7,
                properties: vec![],
                required_standing: Standing::Neutral,
                description: "Dart thrower with double the range of throwing",
            },
            FactionWeapon {
                id: "cuauhololli",
                name: "Cuauhololli",
                faction: Faction::Aztec,
                category: WeaponCategory::Club,
                base_damage: 35,
                range: 1.8,
                attack_speed: 1.2,
                properties: vec![
                    WeaponProperty::StunDuration(2.0),
                    WeaponProperty::BonusVsType { target: TargetType::Armored, multiplier: 1.50 },
                ],
                required_standing: Standing::Neutral,
                description: "A ball-headed mace for stunning foes",
            },
            FactionWeapon {
                id: "tecpatl",
                name: "Tecpatl",
                faction: Faction::Aztec,
                category: WeaponCategory::Obsidian,
                base_damage: 30,
                range: 1.0,
                attack_speed: 2.5,
                properties: vec![
                    WeaponProperty::BonusVsType { target: TargetType::Unaware, multiplier: 2.0 },
                ],
                required_standing: Standing::Friendly,
                description: "A ritual flint knife for sacrificial strikes",
            },
            FactionWeapon {
                id: "chimalli",
                name: "Chimalli",
                faction: Faction::Aztec,
                category: WeaponCategory::Melee,
                base_damage: 10,
                range: 1.2,
                attack_speed: 1.5,
                properties: vec![
                    WeaponProperty::ParryWindowBonus(0.20),
                ],
                required_standing: Standing::Neutral,
                description: "A feathered shield blocking projectiles",
            },
        ],
        Faction::Powhatan => vec![
            FactionWeapon {
                id: "powhatan_longbow",
                name: "Powhatan Longbow",
                faction: Faction::Powhatan,
                category: WeaponCategory::Bow,
                base_damage: 45,
                range: 80.0,
                attack_speed: 0.7,
                properties: vec![
                    WeaponProperty::SilentAttack,
                    WeaponProperty::BonusVsType { target: TargetType::Animal, multiplier: 1.20 },
                ],
                required_standing: Standing::Neutral,
                description: "A silent bow crafted for hunting deer",
            },
            FactionWeapon {
                id: "powhatan_tomahawk",
                name: "Tomahawk",
                faction: Faction::Powhatan,
                category: WeaponCategory::Throwing,
                base_damage: 35,
                range: 12.0,
                attack_speed: 1.1,
                properties: vec![
                    WeaponProperty::ReturnOnMiss,
                ],
                required_standing: Standing::Neutral,
                description: "A versatile throwing and melee weapon",
            },
            FactionWeapon {
                id: "pogamoggan",
                name: "War Club (Pogamoggan)",
                faction: Faction::Powhatan,
                category: WeaponCategory::Club,
                base_damage: 40,
                range: 1.8,
                attack_speed: 1.0,
                properties: vec![
                    WeaponProperty::StunDuration(1.0),
                    WeaponProperty::BonusVsType { target: TargetType::Armored, multiplier: 1.20 },
                ],
                required_standing: Standing::Friendly,
                description: "A stone-headed war club for stunning enemies",
            },
            FactionWeapon {
                id: "hunting_spear",
                name: "Hunting Spear",
                faction: Faction::Powhatan,
                category: WeaponCategory::Spear,
                base_damage: 35,
                range: 3.0,
                attack_speed: 0.9,
                properties: vec![
                    WeaponProperty::BonusVsType { target: TargetType::Animal, multiplier: 1.40 },
                ],
                required_standing: Standing::Neutral,
                description: "A spear designed for hunting large game",
            },
            FactionWeapon {
                id: "flint_knife",
                name: "Flint Knife",
                faction: Faction::Powhatan,
                category: WeaponCategory::Dagger,
                base_damage: 20,
                range: 1.0,
                attack_speed: 2.3,
                properties: vec![
                    WeaponProperty::BleedChance(0.20),
                    WeaponProperty::SilentAttack,
                ],
                required_standing: Standing::Neutral,
                description: "A sharp flint blade for cutting and skinning",
            },
        ],
        Faction::Tuscarora => vec![
            FactionWeapon {
                id: "tuscarora_war_club",
                name: "Tuscarora War Club",
                faction: Faction::Tuscarora,
                category: WeaponCategory::Club,
                base_damage: 45,
                range: 2.0,
                attack_speed: 0.95,
                properties: vec![
                    WeaponProperty::StunDuration(1.5),
                ],
                required_standing: Standing::Friendly,
                description: "A ball-headed club with high stun chance",
            },
            FactionWeapon {
                id: "hemp_backed_bow",
                name: "Hemp-Backed Bow",
                faction: Faction::Tuscarora,
                category: WeaponCategory::Bow,
                base_damage: 40,
                range: 70.0,
                attack_speed: 0.8,
                properties: vec![
                    WeaponProperty::DrawSpeedBonus(0.15),
                ],
                required_standing: Standing::Neutral,
                description: "A durable bow reinforced with hemp fiber",
            },
            FactionWeapon {
                id: "deer_bone_knife",
                name: "Deer-Bone Knife",
                faction: Faction::Tuscarora,
                category: WeaponCategory::Dagger,
                base_damage: 22,
                range: 1.0,
                attack_speed: 2.4,
                properties: vec![],
                required_standing: Standing::Neutral,
                description: "A lightweight bone knife for swift attacks",
            },
            FactionWeapon {
                id: "piedmont_tomahawk",
                name: "Piedmont Tomahawk",
                faction: Faction::Tuscarora,
                category: WeaponCategory::Throwing,
                base_damage: 38,
                range: 14.0,
                attack_speed: 1.0,
                properties: vec![],
                required_standing: Standing::Friendly,
                description: "A balanced tomahawk from the Piedmont hills",
            },
            FactionWeapon {
                id: "turtle_shell_shield",
                name: "Turtle Shell Shield",
                faction: Faction::Tuscarora,
                category: WeaponCategory::Melee,
                base_damage: 12,
                range: 1.2,
                attack_speed: 1.4,
                properties: vec![
                    WeaponProperty::ParryWindowBonus(0.25),
                ],
                required_standing: Standing::Friendly,
                description: "A durable shield made from turtle carapace",
            },
        ],
        Faction::Cherokee => vec![
            FactionWeapon {
                id: "cherokee_warbow",
                name: "Cherokee Warbow",
                faction: Faction::Cherokee,
                category: WeaponCategory::Bow,
                base_damage: 48,
                range: 85.0,
                attack_speed: 0.65,
                properties: vec![
                    WeaponProperty::DrawSpeedBonus(0.25),
                ],
                required_standing: Standing::Friendly,
                description: "A powerful double-curved bow",
            },
            FactionWeapon {
                id: "stickball_racket",
                name: "Stickball Racket",
                faction: Faction::Cherokee,
                category: WeaponCategory::Club,
                base_damage: 30,
                range: 2.5,
                attack_speed: 1.6,
                properties: vec![],
                required_standing: Standing::Neutral,
                description: "Swift attacks, can catch thrown projectiles",
            },
            FactionWeapon {
                id: "war_hawk_tomahawk",
                name: "War Hawk Tomahawk",
                faction: Faction::Cherokee,
                category: WeaponCategory::Throwing,
                base_damage: 42,
                range: 16.0,
                attack_speed: 0.9,
                properties: vec![
                    WeaponProperty::BleedChance(0.30),
                ],
                required_standing: Standing::Allied,
                description: "A pipe tomahawk with razor edge",
            },
            FactionWeapon {
                id: "river_cane_blowgun",
                name: "River Cane Blowgun",
                faction: Faction::Cherokee,
                category: WeaponCategory::Throwing,
                base_damage: 15,
                range: 25.0,
                attack_speed: 1.5,
                properties: vec![
                    WeaponProperty::SilentAttack,
                    WeaponProperty::PoisonDamage(3.0),
                ],
                required_standing: Standing::Friendly,
                description: "Silent poison dart delivery",
            },
            FactionWeapon {
                id: "flint_war_knife",
                name: "Flint War Knife",
                faction: Faction::Cherokee,
                category: WeaponCategory::Dagger,
                base_damage: 28,
                range: 1.0,
                attack_speed: 2.2,
                properties: vec![
                    WeaponProperty::BleedChance(0.40),
                ],
                required_standing: Standing::Friendly,
                description: "A sharp flint blade causing deep wounds",
            },
        ],
        Faction::Catawba => vec![
            FactionWeapon {
                id: "catawba_river_club",
                name: "Catawba River Club",
                faction: Faction::Catawba,
                category: WeaponCategory::Club,
                base_damage: 38,
                range: 1.9,
                attack_speed: 1.1,
                properties: vec![],
                required_standing: Standing::Neutral,
                description: "A water-hardened club, +20% damage near rivers",
            },
            FactionWeapon {
                id: "piedmont_war_bow",
                name: "Piedmont War Bow",
                faction: Faction::Catawba,
                category: WeaponCategory::Bow,
                base_damage: 42,
                range: 65.0,
                attack_speed: 0.75,
                properties: vec![
                    WeaponProperty::MountedUse,
                ],
                required_standing: Standing::Friendly,
                description: "A compact bow excellent for canoe combat",
            },
            FactionWeapon {
                id: "trading_hatchet",
                name: "Trading Hatchet",
                faction: Faction::Catawba,
                category: WeaponCategory::Throwing,
                base_damage: 32,
                range: 12.0,
                attack_speed: 1.2,
                properties: vec![],
                required_standing: Standing::Neutral,
                description: "A European-style trade hatchet, well-balanced",
            },
            FactionWeapon {
                id: "potters_blade",
                name: "Potter's Blade",
                faction: Faction::Catawba,
                category: WeaponCategory::Dagger,
                base_damage: 18,
                range: 1.0,
                attack_speed: 2.5,
                properties: vec![
                    WeaponProperty::BleedChance(0.30),
                ],
                required_standing: Standing::Neutral,
                description: "A sharp ceramic edge causing deep cuts",
            },
            FactionWeapon {
                id: "flathead_shield",
                name: "Flathead Shield",
                faction: Faction::Catawba,
                category: WeaponCategory::Melee,
                base_damage: 10,
                range: 1.2,
                attack_speed: 1.4,
                properties: vec![
                    WeaponProperty::ParryWindowBonus(0.15),
                ],
                required_standing: Standing::Friendly,
                description: "A decorated shield inspiring fear",
            },
        ],
        Faction::Pamunkey => vec![
            FactionWeapon {
                id: "paramounts_mace",
                name: "Paramount's Mace",
                faction: Faction::Pamunkey,
                category: WeaponCategory::Club,
                base_damage: 50,
                range: 2.0,
                attack_speed: 0.9,
                properties: vec![
                    WeaponProperty::StunDuration(2.0),
                ],
                required_standing: Standing::BloodBond,
                description: "Symbol of paramount authority",
            },
            FactionWeapon {
                id: "sacred_bow",
                name: "Sacred Bow",
                faction: Faction::Pamunkey,
                category: WeaponCategory::Bow,
                base_damage: 48,
                range: 85.0,
                attack_speed: 0.65,
                properties: vec![
                    WeaponProperty::BleedChance(0.20),
                ],
                required_standing: Standing::Allied,
                description: "A blessed bow with enhanced critical chance",
            },
            FactionWeapon {
                id: "corn_knife",
                name: "Corn Knife",
                faction: Faction::Pamunkey,
                category: WeaponCategory::Dagger,
                base_damage: 25,
                range: 1.0,
                attack_speed: 2.0,
                properties: vec![],
                required_standing: Standing::Friendly,
                description: "A harvesting tool with combat utility",
            },
            FactionWeapon {
                id: "royal_tomahawk",
                name: "Royal Tomahawk",
                faction: Faction::Pamunkey,
                category: WeaponCategory::Throwing,
                base_damage: 40,
                range: 15.0,
                attack_speed: 0.95,
                properties: vec![],
                required_standing: Standing::Allied,
                description: "A decorated tomahawk, cannot be disarmed",
            },
            FactionWeapon {
                id: "pamunkey_great_shield",
                name: "Pamunkey Great Shield",
                faction: Faction::Pamunkey,
                category: WeaponCategory::Melee,
                base_damage: 15,
                range: 1.5,
                attack_speed: 1.2,
                properties: vec![
                    WeaponProperty::ParryWindowBonus(0.30),
                ],
                required_standing: Standing::Allied,
                description: "Royal emblems provide morale to allies",
            },
        ],
        Faction::Independent | Faction::Wildlife => vec![],
    }
}

// ============================================================================
// FACTION ABILITIES
// ============================================================================

/// A faction-specific ability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionAbility {
    pub id: &'static str,
    pub name: &'static str,
    pub faction: Faction,
    pub cooldown_secs: f32,
    pub duration_secs: Option<f32>,
    pub effect: AbilityEffect,
    pub required_standing: Standing,
    pub skill_tier: u8,
    pub description: &'static str,
}

/// Ability effect types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbilityEffect {
    /// Buff self or allies
    StatBuff {
        stat: StatType,
        multiplier: f32,
        target: AbilityTarget,
    },
    /// Increase damage
    DamageBuff { multiplier: f32 },
    /// Reduce damage taken
    DefenseBuff { reduction: f32 },
    /// Area damage
    AreaDamage {
        radius: f32,
        damage: u32,
        damage_type: DamageType,
    },
    /// Become invisible
    Stealth { duration: f32 },
    /// Summon allies
    Summon {
        entity_type: SummonType,
        count: u8,
        duration: f32,
    },
    /// Reveal hidden enemies
    Reveal { radius: f32 },
    /// Heal self or allies
    Heal { amount: u32, target: AbilityTarget },
    /// Fear enemies
    Fear { radius: f32, duration: f32 },
    /// Slow time (perception effect)
    SlowTime { duration: f32, factor: f32 },
    /// Mark target for bonus damage
    Mark { duration: f32, bonus_damage: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbilityTarget {
    SelfOnly,
    Allies,
    Enemies,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummonType {
    Wolf,
    Jaguar,
    Eagle,
    SpiritWarrior,
    Militia,
}

/// Get all abilities for a faction
pub fn get_faction_abilities(faction: Faction) -> Vec<FactionAbility> {
    match faction {
        Faction::Spanish => vec![
            FactionAbility {
                id: "santiago",
                name: "Santiago!",
                faction: Faction::Spanish,
                cooldown_secs: 120.0,
                duration_secs: Some(10.0),
                effect: AbilityEffect::StatBuff {
                    stat: StatType::Damage,
                    multiplier: 1.30,
                    target: AbilityTarget::SelfOnly,
                },
                required_standing: Standing::Friendly,
                skill_tier: 2,
                description: "War cry granting +30% damage and +20% speed for 10s",
            },
            FactionAbility {
                id: "steel_wall",
                name: "Steel Wall",
                faction: Faction::Spanish,
                cooldown_secs: 90.0,
                duration_secs: Some(5.0),
                effect: AbilityEffect::DefenseBuff { reduction: 1.0 },
                required_standing: Standing::Allied,
                skill_tier: 3,
                description: "Block all frontal damage for 5s",
            },
            FactionAbility {
                id: "estocada",
                name: "Estocada",
                faction: Faction::Spanish,
                cooldown_secs: 8.0,
                duration_secs: None,
                effect: AbilityEffect::DamageBuff { multiplier: 2.0 },
                required_standing: Standing::Friendly,
                skill_tier: 3,
                description: "Lunging thrust dealing 2x damage",
            },
            FactionAbility {
                id: "divine_judgment",
                name: "Divine Judgment",
                faction: Faction::Spanish,
                cooldown_secs: 180.0,
                duration_secs: None,
                effect: AbilityEffect::Mark {
                    duration: 10.0,
                    bonus_damage: 1.0,
                },
                required_standing: Standing::Allied,
                skill_tier: 4,
                description: "Next firearm shot is a guaranteed critical",
            },
            FactionAbility {
                id: "conquerors_presence",
                name: "Conqueror's Presence",
                faction: Faction::Spanish,
                cooldown_secs: 0.0, // Passive
                duration_secs: None,
                effect: AbilityEffect::Fear {
                    radius: 20.0,
                    duration: 0.0,
                },
                required_standing: Standing::BloodBond,
                skill_tier: 6,
                description: "Enemies within 20m have -25% morale (passive)",
            },
        ],
        Faction::French => vec![
            FactionAbility {
                id: "portage",
                name: "Portage",
                faction: Faction::French,
                cooldown_secs: 0.0, // Passive
                duration_secs: None,
                effect: AbilityEffect::StatBuff {
                    stat: StatType::Speed,
                    multiplier: 0.80,
                    target: AbilityTarget::SelfOnly,
                },
                required_standing: Standing::Neutral,
                skill_tier: 1,
                description: "Carry canoe overland at 80% speed",
            },
            FactionAbility {
                id: "trade_parley",
                name: "Trade Parley",
                faction: Faction::French,
                cooldown_secs: 300.0,
                duration_secs: None,
                effect: AbilityEffect::Fear {
                    radius: 30.0,
                    duration: 10.0,
                },
                required_standing: Standing::Allied,
                skill_tier: 3,
                description: "Initiate peaceful dialogue with any hostile group",
            },
            FactionAbility {
                id: "ghost_walk",
                name: "Ghost Walk",
                faction: Faction::French,
                cooldown_secs: 60.0,
                duration_secs: Some(10.0),
                effect: AbilityEffect::Stealth { duration: 10.0 },
                required_standing: Standing::Friendly,
                skill_tier: 3,
                description: "Enhanced stealth with silent footsteps for 10s",
            },
            FactionAbility {
                id: "natures_bounty",
                name: "Nature's Bounty",
                faction: Faction::French,
                cooldown_secs: 0.0, // Passive
                duration_secs: None,
                effect: AbilityEffect::StatBuff {
                    stat: StatType::Stamina,
                    multiplier: 2.0,
                    target: AbilityTarget::SelfOnly,
                },
                required_standing: Standing::Allied,
                skill_tier: 4,
                description: "Double gathering yield in wilderness",
            },
            FactionAbility {
                id: "spirit_walk",
                name: "Spirit Walk",
                faction: Faction::French,
                cooldown_secs: 300.0,
                duration_secs: Some(20.0),
                effect: AbilityEffect::Stealth { duration: 20.0 },
                required_standing: Standing::BloodBond,
                skill_tier: 6,
                description: "Full invisibility for 20s",
            },
        ],
        Faction::English => vec![
            FactionAbility {
                id: "for_the_queen",
                name: "For the Queen!",
                faction: Faction::English,
                cooldown_secs: 180.0,
                duration_secs: Some(15.0),
                effect: AbilityEffect::StatBuff {
                    stat: StatType::Health,
                    multiplier: 1.20,
                    target: AbilityTarget::SelfOnly,
                },
                required_standing: Standing::Friendly,
                skill_tier: 2,
                description: "+20% all stats for 15s",
            },
            FactionAbility {
                id: "defensive_formation",
                name: "Defensive Formation",
                faction: Faction::English,
                cooldown_secs: 120.0,
                duration_secs: Some(20.0),
                effect: AbilityEffect::DefenseBuff { reduction: 0.30 },
                required_standing: Standing::Allied,
                skill_tier: 3,
                description: "All allies gain +30% defense for 20s",
            },
            FactionAbility {
                id: "signal_fire",
                name: "Signal Fire",
                faction: Faction::English,
                cooldown_secs: 600.0,
                duration_secs: None,
                effect: AbilityEffect::Summon {
                    entity_type: SummonType::Militia,
                    count: 4,
                    duration: 300.0,
                },
                required_standing: Standing::Allied,
                skill_tier: 3,
                description: "Call reinforcements from nearest settlement",
            },
            FactionAbility {
                id: "colonial_resolve",
                name: "Colonial Resolve",
                faction: Faction::English,
                cooldown_secs: 300.0,
                duration_secs: None,
                effect: AbilityEffect::Heal {
                    amount: 1,
                    target: AbilityTarget::SelfOnly,
                },
                required_standing: Standing::Allied,
                skill_tier: 4,
                description: "Ignore next lethal hit, survive with 1 HP",
            },
            FactionAbility {
                id: "crowns_authority",
                name: "Crown's Authority",
                faction: Faction::English,
                cooldown_secs: 0.0, // Passive
                duration_secs: None,
                effect: AbilityEffect::Fear {
                    radius: 0.0,
                    duration: 0.0,
                },
                required_standing: Standing::BloodBond,
                skill_tier: 6,
                description: "Diplomatic immunity in neutral territories",
            },
        ],
        Faction::Aztec => vec![
            FactionAbility {
                id: "blood_offering",
                name: "Blood Offering",
                faction: Faction::Aztec,
                cooldown_secs: 60.0,
                duration_secs: Some(30.0),
                effect: AbilityEffect::DamageBuff { multiplier: 1.30 },
                required_standing: Standing::Neutral,
                skill_tier: 1,
                description: "Sacrifice 20% HP for +30% damage (30s)",
            },
            FactionAbility {
                id: "jaguar_pounce",
                name: "Jaguar Pounce",
                faction: Faction::Aztec,
                cooldown_secs: 15.0,
                duration_secs: None,
                effect: AbilityEffect::DamageBuff { multiplier: 1.50 },
                required_standing: Standing::Friendly,
                skill_tier: 2,
                description: "Leap to target within 5m with bonus damage",
            },
            FactionAbility {
                id: "eagle_dive",
                name: "Eagle Dive",
                faction: Faction::Aztec,
                cooldown_secs: 20.0,
                duration_secs: None,
                effect: AbilityEffect::DamageBuff { multiplier: 2.0 },
                required_standing: Standing::Friendly,
                skill_tier: 2,
                description: "Aerial attack from elevation deals 2x damage",
            },
            FactionAbility {
                id: "jaguar_rage",
                name: "Jaguar Rage",
                faction: Faction::Aztec,
                cooldown_secs: 180.0,
                duration_secs: Some(30.0),
                effect: AbilityEffect::StatBuff {
                    stat: StatType::Damage,
                    multiplier: 1.50,
                    target: AbilityTarget::SelfOnly,
                },
                required_standing: Standing::Allied,
                skill_tier: 3,
                description: "+50% damage, -30% damage taken for 30s",
            },
            FactionAbility {
                id: "call_of_the_hunt",
                name: "Call of the Hunt",
                faction: Faction::Aztec,
                cooldown_secs: 120.0,
                duration_secs: Some(60.0),
                effect: AbilityEffect::Reveal { radius: 100.0 },
                required_standing: Standing::Allied,
                skill_tier: 4,
                description: "Mark all enemies within 100m",
            },
            FactionAbility {
                id: "wrath_of_huitzilopochtli",
                name: "Wrath of Huitzilopochtli",
                faction: Faction::Aztec,
                cooldown_secs: 300.0,
                duration_secs: None,
                effect: AbilityEffect::AreaDamage {
                    radius: 10.0,
                    damage: 100,
                    damage_type: DamageType::Fire,
                },
                required_standing: Standing::BloodBond,
                skill_tier: 6,
                description: "AoE fire damage around self",
            },
        ],
        Faction::Powhatan => vec![
            FactionAbility {
                id: "hunters_focus",
                name: "Hunter's Focus",
                faction: Faction::Powhatan,
                cooldown_secs: 45.0,
                duration_secs: Some(5.0),
                effect: AbilityEffect::SlowTime {
                    duration: 5.0,
                    factor: 0.5,
                },
                required_standing: Standing::Friendly,
                skill_tier: 2,
                description: "Slow time perception while aiming (5s)",
            },
            FactionAbility {
                id: "rivers_gift",
                name: "River's Gift",
                faction: Faction::Powhatan,
                cooldown_secs: 0.0, // Passive
                duration_secs: None,
                effect: AbilityEffect::Heal {
                    amount: 5,
                    target: AbilityTarget::SelfOnly,
                },
                required_standing: Standing::Friendly,
                skill_tier: 3,
                description: "Regenerate stamina while in water",
            },
            FactionAbility {
                id: "war_paint",
                name: "War Paint",
                faction: Faction::Powhatan,
                cooldown_secs: 600.0,
                duration_secs: Some(600.0),
                effect: AbilityEffect::StatBuff {
                    stat: StatType::Damage,
                    multiplier: 1.15,
                    target: AbilityTarget::SelfOnly,
                },
                required_standing: Standing::Allied,
                skill_tier: 3,
                description: "+15% damage, +10% intimidation (10min)",
            },
            FactionAbility {
                id: "spirit_guide",
                name: "Spirit Guide",
                faction: Faction::Powhatan,
                cooldown_secs: 120.0,
                duration_secs: Some(60.0),
                effect: AbilityEffect::Reveal { radius: 50.0 },
                required_standing: Standing::Allied,
                skill_tier: 4,
                description: "Animal spirit scouts ahead, revealing enemies",
            },
            FactionAbility {
                id: "voice_of_the_land",
                name: "Voice of the Land",
                faction: Faction::Powhatan,
                cooldown_secs: 86400.0, // Once per day
                duration_secs: None,
                effect: AbilityEffect::Fear {
                    radius: 0.0,
                    duration: 0.0,
                },
                required_standing: Standing::BloodBond,
                skill_tier: 6,
                description: "All Native factions +1 reputation",
            },
        ],
        Faction::Tuscarora => vec![
            FactionAbility {
                id: "bear_roar",
                name: "Bear Roar",
                faction: Faction::Tuscarora,
                cooldown_secs: 90.0,
                duration_secs: None,
                effect: AbilityEffect::Fear {
                    radius: 15.0,
                    duration: 3.0,
                },
                required_standing: Standing::Friendly,
                skill_tier: 3,
                description: "Terrify enemies causing them to flee (3s)",
            },
            FactionAbility {
                id: "wolf_pack",
                name: "Wolf Pack",
                faction: Faction::Tuscarora,
                cooldown_secs: 180.0,
                duration_secs: Some(120.0),
                effect: AbilityEffect::Summon {
                    entity_type: SummonType::Wolf,
                    count: 3,
                    duration: 120.0,
                },
                required_standing: Standing::Allied,
                skill_tier: 3,
                description: "Call 3 wolves to fight alongside you",
            },
            FactionAbility {
                id: "turtle_defense",
                name: "Turtle Defense",
                faction: Faction::Tuscarora,
                cooldown_secs: 120.0,
                duration_secs: Some(10.0),
                effect: AbilityEffect::DefenseBuff { reduction: 0.50 },
                required_standing: Standing::Friendly,
                skill_tier: 3,
                description: "+50% damage reduction for 10s",
            },
            FactionAbility {
                id: "deer_speed",
                name: "Deer Speed",
                faction: Faction::Tuscarora,
                cooldown_secs: 60.0,
                duration_secs: Some(15.0),
                effect: AbilityEffect::StatBuff {
                    stat: StatType::Speed,
                    multiplier: 1.25,
                    target: AbilityTarget::SelfOnly,
                },
                required_standing: Standing::Friendly,
                skill_tier: 3,
                description: "+25% movement speed for 15s",
            },
            FactionAbility {
                id: "great_law",
                name: "Great Law",
                faction: Faction::Tuscarora,
                cooldown_secs: 3600.0, // Once per hour
                duration_secs: None,
                effect: AbilityEffect::Fear {
                    radius: 100.0,
                    duration: 0.0,
                },
                required_standing: Standing::BloodBond,
                skill_tier: 6,
                description: "End any conflict with rival factions in area",
            },
        ],
        Faction::Cherokee => vec![
            FactionAbility {
                id: "raven_curse",
                name: "Raven Curse",
                faction: Faction::Cherokee,
                cooldown_secs: 120.0,
                duration_secs: Some(30.0),
                effect: AbilityEffect::Mark {
                    duration: 30.0,
                    bonus_damage: 0.25,
                },
                required_standing: Standing::Allied,
                skill_tier: 3,
                description: "Curse enemy: take +25% damage from all sources",
            },
            FactionAbility {
                id: "battle_ritual",
                name: "Battle Ritual",
                faction: Faction::Cherokee,
                cooldown_secs: 300.0,
                duration_secs: Some(120.0),
                effect: AbilityEffect::StatBuff {
                    stat: StatType::Damage,
                    multiplier: 1.20,
                    target: AbilityTarget::Allies,
                },
                required_standing: Standing::Allied,
                skill_tier: 3,
                description: "Grant all allies +20% damage for 2min",
            },
            FactionAbility {
                id: "medicine_heal",
                name: "Medicine Heal",
                faction: Faction::Cherokee,
                cooldown_secs: 60.0,
                duration_secs: None,
                effect: AbilityEffect::Heal {
                    amount: 50,
                    target: AbilityTarget::SelfOnly,
                },
                required_standing: Standing::Friendly,
                skill_tier: 3,
                description: "Heal 50 HP using herbal medicine",
            },
            FactionAbility {
                id: "eternal_flame",
                name: "Eternal Flame",
                faction: Faction::Cherokee,
                cooldown_secs: 180.0,
                duration_secs: Some(30.0),
                effect: AbilityEffect::AreaDamage {
                    radius: 8.0,
                    damage: 10,
                    damage_type: DamageType::Fire,
                },
                required_standing: Standing::Allied,
                skill_tier: 4,
                description: "Create ring of fire damaging enemies",
            },
            FactionAbility {
                id: "voice_of_ancestors",
                name: "Voice of Ancestors",
                faction: Faction::Cherokee,
                cooldown_secs: 300.0,
                duration_secs: Some(60.0),
                effect: AbilityEffect::Summon {
                    entity_type: SummonType::SpiritWarrior,
                    count: 4,
                    duration: 60.0,
                },
                required_standing: Standing::BloodBond,
                skill_tier: 6,
                description: "Summon 4 spirit warriors to fight",
            },
        ],
        Faction::Catawba => vec![
            FactionAbility {
                id: "river_ambush",
                name: "River Ambush",
                faction: Faction::Catawba,
                cooldown_secs: 45.0,
                duration_secs: None,
                effect: AbilityEffect::DamageBuff { multiplier: 1.75 },
                required_standing: Standing::Friendly,
                skill_tier: 3,
                description: "Attack from water deals +75% damage",
            },
            FactionAbility {
                id: "raid_call",
                name: "Raid Call",
                faction: Faction::Catawba,
                cooldown_secs: 300.0,
                duration_secs: Some(180.0),
                effect: AbilityEffect::Summon {
                    entity_type: SummonType::SpiritWarrior,
                    count: 8,
                    duration: 180.0,
                },
                required_standing: Standing::Allied,
                skill_tier: 4,
                description: "Call 8 raid warriors for 3 minutes",
            },
            FactionAbility {
                id: "water_hide",
                name: "Water Hide",
                faction: Faction::Catawba,
                cooldown_secs: 30.0,
                duration_secs: Some(60.0),
                effect: AbilityEffect::Stealth { duration: 60.0 },
                required_standing: Standing::Friendly,
                skill_tier: 3,
                description: "Hide underwater indefinitely with reed",
            },
            FactionAbility {
                id: "trade_network",
                name: "Trade Network",
                faction: Faction::Catawba,
                cooldown_secs: 0.0, // Passive
                duration_secs: None,
                effect: AbilityEffect::StatBuff {
                    stat: StatType::Stamina,
                    multiplier: 1.0,
                    target: AbilityTarget::SelfOnly,
                },
                required_standing: Standing::Allied,
                skill_tier: 4,
                description: "Passive income from all trade routes",
            },
            FactionAbility {
                id: "flood_the_land",
                name: "Flood the Land",
                faction: Faction::Catawba,
                cooldown_secs: 600.0,
                duration_secs: None,
                effect: AbilityEffect::Summon {
                    entity_type: SummonType::SpiritWarrior,
                    count: 20,
                    duration: 300.0,
                },
                required_standing: Standing::BloodBond,
                skill_tier: 6,
                description: "Call Catawba warriors from all settlements",
            },
        ],
        Faction::Pamunkey => vec![
            FactionAbility {
                id: "royal_command",
                name: "Royal Command",
                faction: Faction::Pamunkey,
                cooldown_secs: 120.0,
                duration_secs: Some(60.0),
                effect: AbilityEffect::StatBuff {
                    stat: StatType::Damage,
                    multiplier: 1.20,
                    target: AbilityTarget::Allies,
                },
                required_standing: Standing::Allied,
                skill_tier: 3,
                description: "Command allies to fight with +20% damage",
            },
            FactionAbility {
                id: "sacred_protection",
                name: "Sacred Protection",
                faction: Faction::Pamunkey,
                cooldown_secs: 180.0,
                duration_secs: Some(30.0),
                effect: AbilityEffect::DefenseBuff { reduction: 0.40 },
                required_standing: Standing::Allied,
                skill_tier: 3,
                description: "Temple spirits grant +40% defense",
            },
            FactionAbility {
                id: "corn_blessing",
                name: "Corn Blessing",
                faction: Faction::Pamunkey,
                cooldown_secs: 600.0,
                duration_secs: None,
                effect: AbilityEffect::Heal {
                    amount: 100,
                    target: AbilityTarget::Allies,
                },
                required_standing: Standing::Friendly,
                skill_tier: 3,
                description: "Heal all allies for 100 HP",
            },
            FactionAbility {
                id: "paramount_authority",
                name: "Paramount Authority",
                faction: Faction::Pamunkey,
                cooldown_secs: 300.0,
                duration_secs: Some(120.0),
                effect: AbilityEffect::Fear {
                    radius: 30.0,
                    duration: 120.0,
                },
                required_standing: Standing::Allied,
                skill_tier: 4,
                description: "Speak with paramount authority, enemies hesitate",
            },
            FactionAbility {
                id: "unite_the_people",
                name: "Unite the People",
                faction: Faction::Pamunkey,
                cooldown_secs: 3600.0, // Once per hour
                duration_secs: Some(300.0),
                effect: AbilityEffect::Summon {
                    entity_type: SummonType::SpiritWarrior,
                    count: 30,
                    duration: 300.0,
                },
                required_standing: Standing::BloodBond,
                skill_tier: 6,
                description: "All confederacy tribes rally to your call",
            },
        ],
        Faction::Independent | Faction::Wildlife => vec![],
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standing_from_reputation() {
        assert_eq!(Standing::from_reputation(-1500), Standing::War);
        assert_eq!(Standing::from_reputation(-750), Standing::Hostile);
        assert_eq!(Standing::from_reputation(-150), Standing::Suspicious);
        assert_eq!(Standing::from_reputation(0), Standing::Neutral);
        assert_eq!(Standing::from_reputation(250), Standing::Friendly);
        assert_eq!(Standing::from_reputation(750), Standing::Allied);
        assert_eq!(Standing::from_reputation(1500), Standing::BloodBond);
    }

    #[test]
    fn test_relationship_matrix() {
        let matrix = FactionRelationshipMatrix::new();

        // Spanish-English should be at War
        assert_eq!(
            matrix.get_standing(Faction::Spanish, Faction::English),
            Standing::War
        );

        // French-Powhatan should be Allied
        assert_eq!(
            matrix.get_standing(Faction::French, Faction::Powhatan),
            Standing::Allied
        );

        // Same faction should be BloodBond
        assert_eq!(
            matrix.get_standing(Faction::Cherokee, Faction::Cherokee),
            Standing::BloodBond
        );
    }

    #[test]
    fn test_faction_traits() {
        let spanish_traits = get_faction_traits(Faction::Spanish);
        assert_eq!(spanish_traits.len(), 5);

        let french_traits = get_faction_traits(Faction::French);
        assert_eq!(french_traits.len(), 5);
    }

    #[test]
    fn test_faction_weapons() {
        let aztec_weapons = get_faction_weapons(Faction::Aztec);
        assert!(!aztec_weapons.is_empty());

        // Macuahuitl should exist
        assert!(aztec_weapons.iter().any(|w| w.id == "macuahuitl"));
    }

    #[test]
    fn test_faction_abilities() {
        let powhatan_abilities = get_faction_abilities(Faction::Powhatan);
        assert!(!powhatan_abilities.is_empty());

        // Hunter's Focus should exist
        assert!(powhatan_abilities.iter().any(|a| a.id == "hunters_focus"));
    }

    #[test]
    fn test_relationship_balance() {
        let relationships = get_default_relationships();

        let mut hostile_count = 0;
        let mut neutral_count = 0;
        let mut friendly_count = 0;

        for rel in &relationships {
            match rel.base_standing {
                Standing::War | Standing::Hostile | Standing::Suspicious => hostile_count += 1,
                Standing::Neutral => neutral_count += 1,
                Standing::Friendly | Standing::Allied | Standing::BloodBond => friendly_count += 1,
            }
        }

        // Ensure good mix
        assert!(hostile_count > 5, "Should have some hostile relationships");
        assert!(neutral_count >= 2, "Should have some neutral relationships");
        assert!(friendly_count > 10, "Should have many friendly relationships");
    }
}
