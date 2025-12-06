//! Core type definitions for the animal system

use serde::{Deserialize, Serialize};

/// Unique identifier for animal species
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimalSpecies {
    BlackBear,
    EasternCougar,
    GrayWolf,
    TimberRattlesnake,
    AmericanAlligator,
    WildBoar,
    Copperhead,
    RedWolf,
    Bobcat,
    Cottonmouth,
}

impl AnimalSpecies {
    /// Get the display name for this species
    pub fn name(&self) -> &'static str {
        match self {
            Self::BlackBear => "Black Bear",
            Self::EasternCougar => "Eastern Cougar",
            Self::GrayWolf => "Gray Wolf",
            Self::TimberRattlesnake => "Timber Rattlesnake",
            Self::AmericanAlligator => "American Alligator",
            Self::WildBoar => "Wild Boar",
            Self::Copperhead => "Copperhead Snake",
            Self::RedWolf => "Red Wolf",
            Self::Bobcat => "Bobcat",
            Self::Cottonmouth => "Cottonmouth",
        }
    }

    /// Parse species from name string
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "black bear" | "blackbear" => Some(Self::BlackBear),
            "eastern cougar" | "easterncougar" | "cougar" => Some(Self::EasternCougar),
            "gray wolf" | "graywolf" | "wolf" => Some(Self::GrayWolf),
            "timber rattlesnake" | "timberrattlesnake" | "rattlesnake" => Some(Self::TimberRattlesnake),
            "american alligator" | "americanalligator" | "alligator" => Some(Self::AmericanAlligator),
            "wild boar" | "wildboar" | "boar" => Some(Self::WildBoar),
            "copperhead" | "copperhead snake" => Some(Self::Copperhead),
            "red wolf" | "redwolf" => Some(Self::RedWolf),
            "bobcat" => Some(Self::Bobcat),
            "cottonmouth" => Some(Self::Cottonmouth),
            _ => None,
        }
    }

    /// Get the base stats for this species
    pub fn base_stats(&self) -> AnimalStats {
        match self {
            Self::BlackBear => AnimalStats {
                health: 150.0,
                damage: 25.0,
                speed: 35.0,
                speed_in_water: None,
                detection_range: 20.0,
                attack_range: 2.0,
            },
            Self::EasternCougar => AnimalStats {
                health: 100.0,
                damage: 30.0,
                speed: 50.0,
                speed_in_water: None,
                detection_range: 30.0,
                attack_range: 3.0,
            },
            Self::GrayWolf => AnimalStats {
                health: 80.0,
                damage: 20.0,
                speed: 45.0,
                speed_in_water: None,
                detection_range: 25.0,
                attack_range: 2.0,
            },
            Self::TimberRattlesnake => AnimalStats {
                health: 30.0,
                damage: 15.0,
                speed: 15.0,
                speed_in_water: None,
                detection_range: 10.0,
                attack_range: 1.5,
            },
            Self::AmericanAlligator => AnimalStats {
                health: 200.0,
                damage: 40.0,
                speed: 20.0,
                speed_in_water: Some(35.0),
                detection_range: 15.0,
                attack_range: 2.0,
            },
            Self::WildBoar => AnimalStats {
                health: 90.0,
                damage: 18.0,
                speed: 30.0,
                speed_in_water: None,
                detection_range: 15.0,
                attack_range: 1.5,
            },
            Self::Copperhead => AnimalStats {
                health: 25.0,
                damage: 12.0,
                speed: 12.0,
                speed_in_water: None,
                detection_range: 8.0,
                attack_range: 1.0,
            },
            Self::RedWolf => AnimalStats {
                health: 70.0,
                damage: 18.0,
                speed: 42.0,
                speed_in_water: None,
                detection_range: 22.0,
                attack_range: 2.0,
            },
            Self::Bobcat => AnimalStats {
                health: 60.0,
                damage: 15.0,
                speed: 40.0,
                speed_in_water: None,
                detection_range: 20.0,
                attack_range: 2.0,
            },
            Self::Cottonmouth => AnimalStats {
                health: 35.0,
                damage: 16.0,
                speed: 10.0,
                speed_in_water: Some(18.0),
                detection_range: 12.0,
                attack_range: 1.5,
            },
        }
    }

    /// Get the behavior type for this species
    pub fn behavior_type(&self) -> BehaviorType {
        match self {
            Self::BlackBear => BehaviorType::Territorial,
            Self::EasternCougar => BehaviorType::Stalker,
            Self::GrayWolf | Self::RedWolf => BehaviorType::PackHunter,
            Self::TimberRattlesnake | Self::AmericanAlligator => BehaviorType::Ambush,
            Self::WildBoar | Self::Cottonmouth => BehaviorType::Aggressive,
            Self::Copperhead => BehaviorType::Hidden,
            Self::Bobcat => BehaviorType::Stalker,
        }
    }

    /// Get the aggression type for this species
    pub fn aggression_type(&self) -> AggressionType {
        match self {
            Self::BlackBear => AggressionType::Defensive,
            Self::EasternCougar => AggressionType::Predatory,
            Self::GrayWolf => AggressionType::Aggressive,
            Self::TimberRattlesnake | Self::Copperhead => AggressionType::Defensive,
            Self::AmericanAlligator | Self::WildBoar => AggressionType::Territorial,
            Self::RedWolf | Self::Bobcat => AggressionType::Cautious,
            Self::Cottonmouth => AggressionType::Aggressive,
        }
    }

    /// Get the danger level (1-10)
    pub fn danger_level(&self) -> u8 {
        match self {
            Self::BlackBear => 7,
            Self::EasternCougar => 8,
            Self::GrayWolf => 6,
            Self::TimberRattlesnake => 5,
            Self::AmericanAlligator => 9,
            Self::WildBoar => 4,
            Self::Copperhead => 3,
            Self::RedWolf => 5,
            Self::Bobcat => 3,
            Self::Cottonmouth => 4,
        }
    }

    /// Get spawn rate (0.0 - 1.0)
    pub fn spawn_rate(&self) -> f32 {
        match self {
            Self::BlackBear => 0.15,
            Self::EasternCougar => 0.08,
            Self::GrayWolf => 0.20,
            Self::TimberRattlesnake => 0.25,
            Self::AmericanAlligator => 0.12,
            Self::WildBoar => 0.30,
            Self::Copperhead => 0.35,
            Self::RedWolf => 0.18,
            Self::Bobcat => 0.22,
            Self::Cottonmouth => 0.20,
        }
    }

    /// Get health threshold at which animal flees
    pub fn flee_health(&self) -> f32 {
        match self {
            Self::BlackBear => 30.0,
            Self::EasternCougar => 20.0,
            Self::GrayWolf => 15.0,
            Self::TimberRattlesnake => 10.0,
            Self::AmericanAlligator => 40.0,
            Self::WildBoar => 20.0,
            Self::Copperhead => 8.0,
            Self::RedWolf => 15.0,
            Self::Bobcat => 12.0,
            Self::Cottonmouth => 10.0,
        }
    }

    /// Get pack size range if this is a pack animal
    /// Note: For wolves, use wolf_group_config() for more nuanced spawning
    pub fn pack_size(&self) -> Option<(u8, u8)> {
        match self {
            Self::GrayWolf => Some((1, 6)),  // Now includes lone wolves
            Self::RedWolf => Some((1, 4)),   // Now includes lone wolves
            _ => None,
        }
    }

    /// Get wolf-specific group configuration with probabilities
    /// Returns None for non-wolf species
    pub fn wolf_group_config(&self) -> Option<WolfGroupConfig> {
        match self {
            Self::GrayWolf => Some(WolfGroupConfig {
                lone_wolf_chance: 0.20,      // 20% chance of lone wolf
                pair_chance: 0.25,            // 25% chance of pair
                small_pack_chance: 0.35,      // 35% chance of 3-4 wolves
                large_pack_chance: 0.20,      // 20% chance of 5-6 wolves
            }),
            Self::RedWolf => Some(WolfGroupConfig {
                lone_wolf_chance: 0.25,      // Red wolves more often alone
                pair_chance: 0.30,            // More often in pairs
                small_pack_chance: 0.35,      // 3-4 wolves
                large_pack_chance: 0.10,      // Rarely large packs
            }),
            _ => None,
        }
    }

    /// Check if this species can be tamed
    pub fn is_tameable(&self) -> bool {
        matches!(self, Self::GrayWolf | Self::RedWolf)
    }

    /// Get habitats where this species can spawn
    pub fn habitats(&self) -> &'static [Habitat] {
        match self {
            Self::BlackBear => &[Habitat::Forests, Habitat::Mountains, Habitat::Swamps],
            Self::EasternCougar => &[Habitat::Forests, Habitat::Mountains, Habitat::RockyAreas],
            Self::GrayWolf => &[Habitat::Forests, Habitat::Plains, Habitat::Mountains],
            Self::TimberRattlesnake => &[Habitat::Forests, Habitat::RockyAreas, Habitat::Meadows],
            Self::AmericanAlligator => &[Habitat::Swamps, Habitat::Rivers, Habitat::Marshes],
            Self::WildBoar => &[Habitat::Forests, Habitat::Swamps, Habitat::Fields],
            Self::Copperhead => &[Habitat::Forests, Habitat::RockyAreas, Habitat::NearWater],
            Self::RedWolf => &[Habitat::Forests, Habitat::Swamps, Habitat::CoastalPlains],
            Self::Bobcat => &[Habitat::Forests, Habitat::Swamps, Habitat::RockyAreas],
            Self::Cottonmouth => &[Habitat::Swamps, Habitat::Rivers, Habitat::Marshes],
        }
    }

    /// Get times of day when this species is active
    pub fn active_times(&self) -> &'static [TimeOfDay] {
        match self {
            Self::BlackBear => &[TimeOfDay::Dawn, TimeOfDay::Dusk, TimeOfDay::Night],
            Self::EasternCougar => &[TimeOfDay::Night, TimeOfDay::Dawn],
            Self::GrayWolf => &[TimeOfDay::Night, TimeOfDay::Dusk],
            Self::TimberRattlesnake => &[TimeOfDay::Day, TimeOfDay::Dusk],
            Self::AmericanAlligator => &[TimeOfDay::Any],
            Self::WildBoar => &[TimeOfDay::Dawn, TimeOfDay::Dusk],
            Self::Copperhead => &[TimeOfDay::Night, TimeOfDay::Dusk],
            Self::RedWolf => &[TimeOfDay::Night, TimeOfDay::Dawn],
            Self::Bobcat => &[TimeOfDay::Night, TimeOfDay::Dawn, TimeOfDay::Dusk],
            Self::Cottonmouth => &[TimeOfDay::Any],
        }
    }

    /// Get the weakness for this species
    pub fn weakness(&self) -> Weakness {
        match self {
            Self::BlackBear | Self::GrayWolf | Self::RedWolf => Weakness::Fire,
            Self::EasternCougar => Weakness::LoudNoises,
            Self::TimberRattlesnake | Self::AmericanAlligator => Weakness::Cold,
            Self::WildBoar => Weakness::Spears,
            Self::Copperhead => Weakness::Boots,
            Self::Bobcat => Weakness::Dogs,
            Self::Cottonmouth => Weakness::LongWeapons,
        }
    }

    /// Get attacks for this species
    pub fn attacks(&self) -> &'static [AttackDef] {
        match self {
            Self::BlackBear => &[
                AttackDef {
                    name: "claw_swipe",
                    damage: 25.0,
                    cooldown: 2.0,
                    effect: Some(StatusEffectType::Bleeding),
                },
                AttackDef {
                    name: "bite",
                    damage: 30.0,
                    cooldown: 3.0,
                    effect: None,
                },
            ],
            Self::EasternCougar => &[
                AttackDef {
                    name: "pounce",
                    damage: 35.0,
                    cooldown: 4.0,
                    effect: Some(StatusEffectType::Knockdown),
                },
                AttackDef {
                    name: "throat_bite",
                    damage: 40.0,
                    cooldown: 5.0,
                    effect: Some(StatusEffectType::Bleeding),
                },
            ],
            Self::GrayWolf => &[
                AttackDef {
                    name: "bite",
                    damage: 20.0,
                    cooldown: 1.5,
                    effect: None,
                },
                AttackDef {
                    name: "pack_howl",
                    damage: 0.0,
                    cooldown: 30.0,
                    effect: Some(StatusEffectType::SummonPack),
                },
            ],
            Self::TimberRattlesnake => &[
                AttackDef {
                    name: "venomous_bite",
                    damage: 15.0,
                    cooldown: 3.0,
                    effect: Some(StatusEffectType::Poison),
                },
                AttackDef {
                    name: "rattle_warning",
                    damage: 0.0,
                    cooldown: 2.0,
                    effect: Some(StatusEffectType::Fear),
                },
            ],
            Self::AmericanAlligator => &[
                AttackDef {
                    name: "death_roll",
                    damage: 50.0,
                    cooldown: 6.0,
                    effect: Some(StatusEffectType::Stun),
                },
                AttackDef {
                    name: "tail_whip",
                    damage: 25.0,
                    cooldown: 3.0,
                    effect: Some(StatusEffectType::Knockback),
                },
                AttackDef {
                    name: "crushing_bite",
                    damage: 40.0,
                    cooldown: 4.0,
                    effect: None,
                },
            ],
            Self::WildBoar => &[
                AttackDef {
                    name: "charge",
                    damage: 25.0,
                    cooldown: 4.0,
                    effect: Some(StatusEffectType::Knockback),
                },
                AttackDef {
                    name: "tusk_gore",
                    damage: 20.0,
                    cooldown: 2.0,
                    effect: Some(StatusEffectType::Bleeding),
                },
            ],
            Self::Copperhead => &[AttackDef {
                name: "venomous_strike",
                damage: 12.0,
                cooldown: 2.5,
                effect: Some(StatusEffectType::Poison),
            }],
            Self::RedWolf => &[
                AttackDef {
                    name: "quick_bite",
                    damage: 18.0,
                    cooldown: 1.2,
                    effect: None,
                },
                AttackDef {
                    name: "hamstring",
                    damage: 15.0,
                    cooldown: 4.0,
                    effect: Some(StatusEffectType::Slow),
                },
            ],
            Self::Bobcat => &[
                AttackDef {
                    name: "claw_rake",
                    damage: 15.0,
                    cooldown: 1.5,
                    effect: None,
                },
                AttackDef {
                    name: "leap_attack",
                    damage: 20.0,
                    cooldown: 3.0,
                    effect: Some(StatusEffectType::Stun),
                },
            ],
            Self::Cottonmouth => &[
                AttackDef {
                    name: "venomous_bite",
                    damage: 16.0,
                    cooldown: 2.0,
                    effect: Some(StatusEffectType::Poison),
                },
                AttackDef {
                    name: "threat_display",
                    damage: 0.0,
                    cooldown: 3.0,
                    effect: Some(StatusEffectType::Intimidate),
                },
            ],
        }
    }

    /// Get loot items dropped by this species
    pub fn loot(&self) -> &'static [&'static str] {
        match self {
            Self::BlackBear => &["bear_pelt", "bear_meat", "bear_fat", "claws"],
            Self::EasternCougar => &["cougar_pelt", "cougar_meat", "fangs"],
            Self::GrayWolf => &["wolf_pelt", "wolf_meat", "teeth"],
            Self::TimberRattlesnake => &["snake_skin", "venom_gland", "rattles"],
            Self::AmericanAlligator => &["alligator_hide", "alligator_meat", "teeth"],
            Self::WildBoar => &["boar_hide", "boar_meat", "tusks"],
            Self::Copperhead => &["snake_skin", "venom_gland"],
            Self::RedWolf => &["red_wolf_pelt", "wolf_meat", "teeth"],
            Self::Bobcat => &["bobcat_pelt", "bobcat_meat", "claws"],
            Self::Cottonmouth => &["snake_skin", "venom_gland", "fangs"],
        }
    }

    /// Iterator over all species
    pub fn all() -> impl Iterator<Item = AnimalSpecies> {
        [
            Self::BlackBear,
            Self::EasternCougar,
            Self::GrayWolf,
            Self::TimberRattlesnake,
            Self::AmericanAlligator,
            Self::WildBoar,
            Self::Copperhead,
            Self::RedWolf,
            Self::Bobcat,
            Self::Cottonmouth,
        ]
        .into_iter()
    }

    /// Get the base orb color (RGB) for this species
    /// Colors are designed to be distinctive and reflect the animal's nature
    pub fn orb_color(&self) -> [f32; 3] {
        match self {
            // Dark brown - Black Bear (forest creature)
            Self::BlackBear => [0.25, 0.15, 0.08],
            // Tawny gold - Eastern Cougar (stealthy predator)
            Self::EasternCougar => [0.85, 0.65, 0.30],
            // Silver gray - Gray Wolf (pack hunter)
            Self::GrayWolf => [0.55, 0.55, 0.60],
            // Yellow-brown with warning pattern - Timber Rattlesnake
            Self::TimberRattlesnake => [0.75, 0.60, 0.20],
            // Dark olive green - American Alligator (swamp dweller)
            Self::AmericanAlligator => [0.25, 0.35, 0.15],
            // Brown-red - Wild Boar (aggressive charger)
            Self::WildBoar => [0.55, 0.30, 0.20],
            // Copper orange - Copperhead (hidden danger)
            Self::Copperhead => [0.80, 0.45, 0.25],
            // Rusty red-brown - Red Wolf
            Self::RedWolf => [0.70, 0.35, 0.25],
            // Spotted tan - Bobcat (agile stalker)
            Self::Bobcat => [0.70, 0.55, 0.40],
            // Dark water brown - Cottonmouth (aggressive water snake)
            Self::Cottonmouth => [0.40, 0.30, 0.20],
        }
    }

    /// Get the orb scale (radius) based on animal size
    pub fn orb_scale(&self) -> f32 {
        match self {
            Self::BlackBear => 1.2,
            Self::EasternCougar => 0.9,
            Self::GrayWolf => 0.8,
            Self::TimberRattlesnake => 0.4,
            Self::AmericanAlligator => 1.5,
            Self::WildBoar => 0.9,
            Self::Copperhead => 0.35,
            Self::RedWolf => 0.75,
            Self::Bobcat => 0.6,
            Self::Cottonmouth => 0.4,
        }
    }
}

/// Base stats for an animal species
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AnimalStats {
    pub health: f32,
    pub damage: f32,
    pub speed: f32,
    pub speed_in_water: Option<f32>,
    pub detection_range: f32,
    pub attack_range: f32,
}

/// Time periods when animal is active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeOfDay {
    Dawn,  // 5:00 - 8:00
    Day,   // 8:00 - 17:00
    Dusk,  // 17:00 - 20:00
    Night, // 20:00 - 5:00
    Any,   // Always active
}

impl TimeOfDay {
    /// Get current time of day from hour (0-23)
    pub fn from_hour(hour: u8) -> Self {
        match hour {
            5..=7 => Self::Dawn,
            8..=16 => Self::Day,
            17..=19 => Self::Dusk,
            _ => Self::Night,
        }
    }

    /// Check if this time matches (Any matches everything)
    pub fn matches(&self, current: TimeOfDay) -> bool {
        *self == TimeOfDay::Any || *self == current
    }
}

/// Primary behavior archetype
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorType {
    Territorial, // Defends area, attacks if approached
    Stalker,     // Follows prey, ambushes
    PackHunter,  // Coordinates with pack members
    Ambush,      // Waits hidden, strikes when close
    Aggressive,  // Attacks on sight
    Hidden,      // Camouflaged, defensive only
}

/// Aggression response pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggressionType {
    Defensive,   // Only attacks if threatened/approached
    Predatory,   // Hunts player as prey
    Aggressive,  // Attacks readily
    Territorial, // Attacks in territory
    Cautious,    // Evaluates threat before engaging
}

/// Habitat types for spawn filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Habitat {
    Forests,
    Mountains,
    Swamps,
    Rivers,
    Marshes,
    Plains,
    RockyAreas,
    Meadows,
    Fields,
    CoastalPlains,
    NearWater,
}

/// Attack definition
#[derive(Debug, Clone, Copy)]
pub struct AttackDef {
    pub name: &'static str,
    pub damage: f32,
    pub cooldown: f32,
    pub effect: Option<StatusEffectType>,
}

/// Weakness types (affects damage taken)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weakness {
    Fire,
    LoudNoises,
    Cold,
    Spears,
    Boots,
    Dogs,
    LongWeapons,
}

/// Status effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusEffectType {
    Bleeding,
    Poison,
    Knockdown,
    Knockback,
    Stun,
    Fear,
    Slow,
    SummonPack,
    Intimidate,
}

impl StatusEffectType {
    /// Get the duration of this effect in seconds
    pub fn duration(&self) -> f32 {
        match self {
            Self::Bleeding => 10.0,
            Self::Poison => 15.0,
            Self::Knockdown => 2.0,
            Self::Knockback => 0.5,
            Self::Stun => 3.0,
            Self::Fear => 2.0,
            Self::Slow => 5.0,
            Self::SummonPack => 0.0,
            Self::Intimidate => 1.0,
        }
    }

    /// Get damage per second (for DoT effects)
    pub fn damage_per_second(&self) -> f32 {
        match self {
            Self::Bleeding => 2.0,
            Self::Poison => 3.0,
            _ => 0.0,
        }
    }

    /// Get movement speed modifier (1.0 = normal)
    pub fn speed_modifier(&self) -> f32 {
        match self {
            Self::Poison => 0.8,
            Self::Knockdown | Self::Knockback | Self::Stun => 0.0,
            Self::Slow => 0.5,
            Self::Fear => 1.2, // Faster but erratic
            _ => 1.0,
        }
    }

    /// Whether the player can act during this effect
    pub fn can_act(&self) -> bool {
        match self {
            Self::Knockdown | Self::Knockback | Self::Stun => false,
            _ => true,
        }
    }
}

/// Wolf group type - determines behavior patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WolfGroupType {
    /// Single wolf - curious, potentially tameable
    Lone,
    /// Two wolves - usually flee, sometimes aggressive
    Pair,
    /// 3-4 wolves - standard pack hunting
    SmallPack,
    /// 5-6 wolves - aggressive coordinated hunting
    LargePack,
}

impl WolfGroupType {
    /// Determine group type from count
    pub fn from_count(count: u8) -> Self {
        match count {
            1 => Self::Lone,
            2 => Self::Pair,
            3..=4 => Self::SmallPack,
            _ => Self::LargePack,
        }
    }

    /// Get the pack size for this group type
    pub fn size_range(&self) -> (u8, u8) {
        match self {
            Self::Lone => (1, 1),
            Self::Pair => (2, 2),
            Self::SmallPack => (3, 4),
            Self::LargePack => (5, 6),
        }
    }
}

/// Configuration for wolf group spawning probabilities
#[derive(Debug, Clone, Copy)]
pub struct WolfGroupConfig {
    pub lone_wolf_chance: f32,
    pub pair_chance: f32,
    pub small_pack_chance: f32,
    pub large_pack_chance: f32,
}

impl WolfGroupConfig {
    /// Select a group type based on a random value (0.0 - 1.0)
    pub fn select_group_type(&self, roll: f32) -> WolfGroupType {
        let mut cumulative = 0.0;

        cumulative += self.lone_wolf_chance;
        if roll < cumulative {
            return WolfGroupType::Lone;
        }

        cumulative += self.pair_chance;
        if roll < cumulative {
            return WolfGroupType::Pair;
        }

        cumulative += self.small_pack_chance;
        if roll < cumulative {
            return WolfGroupType::SmallPack;
        }

        WolfGroupType::LargePack
    }
}

/// Game difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Survival,
}

impl Difficulty {
    pub fn health_multiplier(&self) -> f32 {
        match self {
            Self::Easy => 0.75,
            Self::Normal => 1.0,
            Self::Hard => 1.5,
            Self::Survival => 2.0,
        }
    }

    pub fn damage_multiplier(&self) -> f32 {
        match self {
            Self::Easy => 0.75,
            Self::Normal => 1.0,
            Self::Hard => 1.25,
            Self::Survival => 1.5,
        }
    }

    pub fn spawn_rate_multiplier(&self) -> f32 {
        match self {
            Self::Easy => 0.8,
            Self::Normal => 1.0,
            Self::Hard => 1.3,
            Self::Survival => 1.5,
        }
    }
}

// Pipeline helper methods are defined in the main impl block above
