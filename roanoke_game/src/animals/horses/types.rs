//! Horse species and type definitions
//!
//! Defines the various horse breeds native to colonial Virginia/Roanoke area,
//! each with unique characteristics suited to different environments and tasks.

use serde::{Deserialize, Serialize};

/// Horse species/breeds available in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorseSpecies {
    /// Banker Horse - Feral coastal breed, excellent beach/marsh navigation
    /// Hardy, good stamina, smaller stature, salt-tolerant
    BankerHorse,

    /// Carolina Marsh Tacky - Wetland specialist from Carolina lowcountry
    /// Calm temperament, swamp navigation, disease resistant
    CarolinaMarshTacky,

    /// Colonial Spanish - Versatile heritage breed
    /// Intelligent, trainable, good all-around utility
    ColonialSpanish,

    /// Chincoteague Pony - Wild island breed from barrier islands
    /// Spirited, independent, excellent swimming ability
    ChincoteaguePony,

    /// Virginia Draught - Heavy work horse bred for farming
    /// Strong, patient, excellent for plowing and hauling
    VirginiaDraught,

    /// Chickasaw Horse - Swift breed favored by native peoples
    /// Fast, agile, excellent for racing and quick travel
    Chickasaw,
}

impl HorseSpecies {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::BankerHorse => "Banker Horse",
            Self::CarolinaMarshTacky => "Carolina Marsh Tacky",
            Self::ColonialSpanish => "Colonial Spanish Horse",
            Self::ChincoteaguePony => "Chincoteague Pony",
            Self::VirginiaDraught => "Virginia Draught",
            Self::Chickasaw => "Chickasaw Horse",
        }
    }

    /// Get base stats for this species
    pub fn base_stats(&self) -> HorseStats {
        match self {
            Self::BankerHorse => HorseStats {
                health: 120.0,
                stamina: 140.0,
                speed: 35.0,
                acceleration: 12.0,
                strength: 45.0,
                agility: 40.0,
                swim_speed: 25.0,
                carry_capacity: 180.0,
                courage: 55.0,
            },
            Self::CarolinaMarshTacky => HorseStats {
                health: 110.0,
                stamina: 130.0,
                speed: 32.0,
                acceleration: 14.0,
                strength: 40.0,
                agility: 50.0,
                swim_speed: 28.0,
                carry_capacity: 160.0,
                courage: 65.0,
            },
            Self::ColonialSpanish => HorseStats {
                health: 130.0,
                stamina: 120.0,
                speed: 38.0,
                acceleration: 15.0,
                strength: 50.0,
                agility: 45.0,
                swim_speed: 20.0,
                carry_capacity: 200.0,
                courage: 60.0,
            },
            Self::ChincoteaguePony => HorseStats {
                health: 90.0,
                stamina: 110.0,
                speed: 30.0,
                acceleration: 18.0,
                strength: 30.0,
                agility: 60.0,
                swim_speed: 35.0,
                carry_capacity: 120.0,
                courage: 45.0,
            },
            Self::VirginiaDraught => HorseStats {
                health: 180.0,
                stamina: 100.0,
                speed: 25.0,
                acceleration: 8.0,
                strength: 90.0,
                agility: 25.0,
                swim_speed: 12.0,
                carry_capacity: 350.0,
                courage: 70.0,
            },
            Self::Chickasaw => HorseStats {
                health: 100.0,
                stamina: 150.0,
                speed: 55.0,
                acceleration: 22.0,
                strength: 35.0,
                agility: 55.0,
                swim_speed: 18.0,
                carry_capacity: 140.0,
                courage: 50.0,
            },
        }
    }

    /// Get temperament profile for this species
    pub fn temperament(&self) -> TemperamentProfile {
        match self {
            Self::BankerHorse => TemperamentProfile {
                base_nervousness: 0.3,
                curiosity: 0.5,
                stubbornness: 0.4,
                sociability: 0.6,
                trainability: 0.6,
                aggression: 0.2,
                flight_threshold: 0.6,
            },
            Self::CarolinaMarshTacky => TemperamentProfile {
                base_nervousness: 0.2,
                curiosity: 0.4,
                stubbornness: 0.3,
                sociability: 0.7,
                trainability: 0.7,
                aggression: 0.1,
                flight_threshold: 0.7,
            },
            Self::ColonialSpanish => TemperamentProfile {
                base_nervousness: 0.35,
                curiosity: 0.6,
                stubbornness: 0.45,
                sociability: 0.5,
                trainability: 0.8,
                aggression: 0.25,
                flight_threshold: 0.55,
            },
            Self::ChincoteaguePony => TemperamentProfile {
                base_nervousness: 0.5,
                curiosity: 0.7,
                stubbornness: 0.6,
                sociability: 0.4,
                trainability: 0.5,
                aggression: 0.35,
                flight_threshold: 0.4,
            },
            Self::VirginiaDraught => TemperamentProfile {
                base_nervousness: 0.15,
                curiosity: 0.3,
                stubbornness: 0.5,
                sociability: 0.6,
                trainability: 0.65,
                aggression: 0.15,
                flight_threshold: 0.8,
            },
            Self::Chickasaw => TemperamentProfile {
                base_nervousness: 0.4,
                curiosity: 0.55,
                stubbornness: 0.35,
                sociability: 0.45,
                trainability: 0.7,
                aggression: 0.2,
                flight_threshold: 0.5,
            },
        }
    }

    /// Get preferred habitats for spawning
    pub fn preferred_habitats(&self) -> &'static [HorseHabitat] {
        match self {
            Self::BankerHorse => &[HorseHabitat::Beach, HorseHabitat::CoastalMarsh, HorseHabitat::Dunes],
            Self::CarolinaMarshTacky => &[HorseHabitat::Swamp, HorseHabitat::CoastalMarsh, HorseHabitat::Wetlands],
            Self::ColonialSpanish => &[HorseHabitat::Grassland, HorseHabitat::OpenPlains, HorseHabitat::Meadows],
            Self::ChincoteaguePony => &[HorseHabitat::BarrierIsland, HorseHabitat::Beach, HorseHabitat::CoastalMarsh],
            Self::VirginiaDraught => &[HorseHabitat::Farmland, HorseHabitat::Grassland, HorseHabitat::Meadows],
            Self::Chickasaw => &[HorseHabitat::OpenPlains, HorseHabitat::Grassland, HorseHabitat::Prairie],
        }
    }

    /// Get primary use/specialization
    pub fn primary_use(&self) -> HorseUse {
        match self {
            Self::BankerHorse => HorseUse::CoastalTravel,
            Self::CarolinaMarshTacky => HorseUse::SwampNavigation,
            Self::ColonialSpanish => HorseUse::GeneralUtility,
            Self::ChincoteaguePony => HorseUse::IslandExploration,
            Self::VirginiaDraught => HorseUse::HeavyLabor,
            Self::Chickasaw => HorseUse::Racing,
        }
    }

    /// Get coat colors available for this species
    pub fn available_coats(&self) -> &'static [HorseCoat] {
        match self {
            Self::BankerHorse => &[
                HorseCoat::Bay, HorseCoat::Chestnut, HorseCoat::Dun,
                HorseCoat::Buckskin, HorseCoat::Brown,
            ],
            Self::CarolinaMarshTacky => &[
                HorseCoat::Bay, HorseCoat::Dun, HorseCoat::Roan,
                HorseCoat::Grullo, HorseCoat::Blue,
            ],
            Self::ColonialSpanish => &[
                HorseCoat::Bay, HorseCoat::Black, HorseCoat::Grey,
                HorseCoat::Chestnut, HorseCoat::Palomino, HorseCoat::Pinto,
            ],
            Self::ChincoteaguePony => &[
                HorseCoat::Pinto, HorseCoat::Bay, HorseCoat::Chestnut,
                HorseCoat::Palomino, HorseCoat::Buckskin,
            ],
            Self::VirginiaDraught => &[
                HorseCoat::Bay, HorseCoat::Black, HorseCoat::Chestnut,
                HorseCoat::Roan, HorseCoat::Grey,
            ],
            Self::Chickasaw => &[
                HorseCoat::Bay, HorseCoat::Chestnut, HorseCoat::Sorrel,
                HorseCoat::Dun, HorseCoat::Buckskin,
            ],
        }
    }

    /// Get taming difficulty (0.0 = easy, 1.0 = very difficult)
    pub fn taming_difficulty(&self) -> f32 {
        match self {
            Self::BankerHorse => 0.5,
            Self::CarolinaMarshTacky => 0.35,
            Self::ColonialSpanish => 0.45,
            Self::ChincoteaguePony => 0.7,
            Self::VirginiaDraught => 0.3,
            Self::Chickasaw => 0.6,
        }
    }

    /// Get spawn rarity (0.0 = common, 1.0 = very rare)
    pub fn rarity(&self) -> f32 {
        match self {
            Self::BankerHorse => 0.4,
            Self::CarolinaMarshTacky => 0.5,
            Self::ColonialSpanish => 0.3,
            Self::ChincoteaguePony => 0.6,
            Self::VirginiaDraught => 0.35,
            Self::Chickasaw => 0.55,
        }
    }

    /// Get orb color for rendering (RGB)
    pub fn orb_color(&self) -> [f32; 3] {
        match self {
            Self::BankerHorse => [0.65, 0.50, 0.35],      // Sandy brown
            Self::CarolinaMarshTacky => [0.45, 0.55, 0.40], // Muddy green-brown
            Self::ColonialSpanish => [0.55, 0.40, 0.30],  // Rich chestnut
            Self::ChincoteaguePony => [0.70, 0.60, 0.45], // Pinto cream
            Self::VirginiaDraught => [0.35, 0.30, 0.25],  // Dark bay
            Self::Chickasaw => [0.75, 0.55, 0.35],        // Golden sorrel
        }
    }

    /// Get orb scale based on horse size
    pub fn orb_scale(&self) -> f32 {
        match self {
            Self::BankerHorse => 1.4,
            Self::CarolinaMarshTacky => 1.3,
            Self::ColonialSpanish => 1.5,
            Self::ChincoteaguePony => 1.1,
            Self::VirginiaDraught => 1.8,
            Self::Chickasaw => 1.4,
        }
    }

    /// Iterator over all species
    pub fn all() -> impl Iterator<Item = HorseSpecies> {
        [
            Self::BankerHorse,
            Self::CarolinaMarshTacky,
            Self::ColonialSpanish,
            Self::ChincoteaguePony,
            Self::VirginiaDraught,
            Self::Chickasaw,
        ].into_iter()
    }
}

/// Core stats for horses
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HorseStats {
    /// Maximum health points
    pub health: f32,
    /// Maximum stamina for running/work
    pub stamina: f32,
    /// Base movement speed
    pub speed: f32,
    /// How quickly horse reaches top speed
    pub acceleration: f32,
    /// Pulling/carrying power
    pub strength: f32,
    /// Turning ability and obstacle navigation
    pub agility: f32,
    /// Swimming speed
    pub swim_speed: f32,
    /// Weight horse can carry/pull
    pub carry_capacity: f32,
    /// Bravery in combat/dangerous situations
    pub courage: f32,
}

/// Temperament profile affecting behavior
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemperamentProfile {
    /// How easily spooked (0.0 = calm, 1.0 = very nervous)
    pub base_nervousness: f32,
    /// Interest in investigating things
    pub curiosity: f32,
    /// Resistance to training/commands
    pub stubbornness: f32,
    /// Preference for herd company
    pub sociability: f32,
    /// How quickly horse learns
    pub trainability: f32,
    /// Tendency toward aggressive behaviors
    pub aggression: f32,
    /// Threshold before fleeing from threats
    pub flight_threshold: f32,
}

/// Habitat types for horse spawning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorseHabitat {
    Beach,
    CoastalMarsh,
    Dunes,
    Swamp,
    Wetlands,
    Grassland,
    OpenPlains,
    Meadows,
    BarrierIsland,
    Farmland,
    Prairie,
    Forest,
    Mountains,
}

/// Primary use/specialization of horse breed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorseUse {
    CoastalTravel,
    SwampNavigation,
    GeneralUtility,
    IslandExploration,
    HeavyLabor,
    Racing,
    Combat,
    PackAnimal,
    Herding,
}

/// Horse coat colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorseCoat {
    Bay,        // Brown body, black mane/tail
    Black,
    Brown,
    Chestnut,   // Reddish-brown
    Dun,        // Yellowish with dark stripe
    Buckskin,   // Golden with black points
    Palomino,   // Golden with white mane/tail
    Grey,
    White,
    Roan,       // Mixed white and colored hairs
    Pinto,      // Large colored patches
    Grullo,     // Mouse gray/dun
    Sorrel,     // Copper red
    Blue,       // Blue-grey roan
}

impl HorseCoat {
    /// Get RGB color for coat
    pub fn color(&self) -> [f32; 3] {
        match self {
            Self::Bay => [0.55, 0.35, 0.20],
            Self::Black => [0.15, 0.12, 0.10],
            Self::Brown => [0.45, 0.30, 0.18],
            Self::Chestnut => [0.65, 0.35, 0.18],
            Self::Dun => [0.75, 0.60, 0.35],
            Self::Buckskin => [0.80, 0.65, 0.35],
            Self::Palomino => [0.90, 0.75, 0.40],
            Self::Grey => [0.60, 0.60, 0.62],
            Self::White => [0.95, 0.93, 0.90],
            Self::Roan => [0.55, 0.45, 0.45],
            Self::Pinto => [0.70, 0.55, 0.40],
            Self::Grullo => [0.50, 0.48, 0.45],
            Self::Sorrel => [0.75, 0.45, 0.25],
            Self::Blue => [0.45, 0.48, 0.55],
        }
    }

    /// Get name for UI display
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bay => "Bay",
            Self::Black => "Black",
            Self::Brown => "Brown",
            Self::Chestnut => "Chestnut",
            Self::Dun => "Dun",
            Self::Buckskin => "Buckskin",
            Self::Palomino => "Palomino",
            Self::Grey => "Grey",
            Self::White => "White",
            Self::Roan => "Roan",
            Self::Pinto => "Pinto",
            Self::Grullo => "Grullo",
            Self::Sorrel => "Sorrel",
            Self::Blue => "Blue Roan",
        }
    }
}

/// Horse gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorseGender {
    Stallion,
    Mare,
    Gelding,
}

impl HorseGender {
    /// Get stat modifiers for gender
    pub fn stat_modifiers(&self) -> (f32, f32, f32) {
        // Returns (strength_mod, speed_mod, temperament_mod)
        match self {
            Self::Stallion => (1.1, 1.05, 1.2),  // Stronger, faster, more temperamental
            Self::Mare => (1.0, 1.0, 0.9),       // Balanced, calmer
            Self::Gelding => (1.05, 1.0, 0.7),   // Slightly stronger, much calmer
        }
    }
}

/// Horse age categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorseAge {
    Foal,       // 0-1 years, cannot be ridden
    Yearling,   // 1-2 years, light training only
    Young,      // 2-4 years, can be trained and ridden
    Prime,      // 4-12 years, peak performance
    Mature,     // 12-18 years, experienced but slowing
    Elder,      // 18+ years, reduced stats but wise
}

impl HorseAge {
    /// Get stat multipliers for age
    pub fn stat_multipliers(&self) -> HorseAgeModifiers {
        match self {
            Self::Foal => HorseAgeModifiers {
                health: 0.4,
                stamina: 0.3,
                speed: 0.5,
                strength: 0.2,
                trainability: 0.0,
                experience_gain: 0.0,
            },
            Self::Yearling => HorseAgeModifiers {
                health: 0.6,
                stamina: 0.5,
                speed: 0.7,
                strength: 0.4,
                trainability: 1.3,
                experience_gain: 1.5,
            },
            Self::Young => HorseAgeModifiers {
                health: 0.85,
                stamina: 0.9,
                speed: 0.95,
                strength: 0.8,
                trainability: 1.2,
                experience_gain: 1.3,
            },
            Self::Prime => HorseAgeModifiers {
                health: 1.0,
                stamina: 1.0,
                speed: 1.0,
                strength: 1.0,
                trainability: 1.0,
                experience_gain: 1.0,
            },
            Self::Mature => HorseAgeModifiers {
                health: 0.95,
                stamina: 0.85,
                speed: 0.9,
                strength: 0.95,
                trainability: 0.7,
                experience_gain: 0.7,
            },
            Self::Elder => HorseAgeModifiers {
                health: 0.8,
                stamina: 0.6,
                speed: 0.7,
                strength: 0.75,
                trainability: 0.4,
                experience_gain: 0.4,
            },
        }
    }

    /// Check if horse can be ridden
    pub fn can_ride(&self) -> bool {
        !matches!(self, Self::Foal)
    }

    /// Check if horse can be trained
    pub fn can_train(&self) -> bool {
        !matches!(self, Self::Foal | Self::Elder)
    }

    /// Check if horse can breed
    pub fn can_breed(&self) -> bool {
        matches!(self, Self::Young | Self::Prime | Self::Mature)
    }
}

/// Age-based stat modifiers
#[derive(Debug, Clone, Copy)]
pub struct HorseAgeModifiers {
    pub health: f32,
    pub stamina: f32,
    pub speed: f32,
    pub strength: f32,
    pub trainability: f32,
    pub experience_gain: f32,
}

/// Herd behavior type for wild horses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HerdType {
    /// Single wild horse
    Solitary,
    /// Pair (usually mare and foal, or bonded pair)
    Pair,
    /// Small band (3-5 horses)
    SmallBand,
    /// Large herd (6-12 horses)
    LargeHerd,
    /// Bachelor band (young stallions)
    BachelorBand,
}

impl HerdType {
    /// Get size range for this herd type
    pub fn size_range(&self) -> (u8, u8) {
        match self {
            Self::Solitary => (1, 1),
            Self::Pair => (2, 2),
            Self::SmallBand => (3, 5),
            Self::LargeHerd => (6, 12),
            Self::BachelorBand => (2, 4),
        }
    }

    /// Get behavior modifiers for herd type
    pub fn behavior_modifiers(&self) -> HerdBehavior {
        match self {
            Self::Solitary => HerdBehavior {
                alertness: 0.8,
                flight_distance: 40.0,
                aggression: 0.2,
                curiosity: 0.6,
            },
            Self::Pair => HerdBehavior {
                alertness: 0.7,
                flight_distance: 35.0,
                aggression: 0.3,
                curiosity: 0.5,
            },
            Self::SmallBand => HerdBehavior {
                alertness: 0.6,
                flight_distance: 30.0,
                aggression: 0.4,
                curiosity: 0.4,
            },
            Self::LargeHerd => HerdBehavior {
                alertness: 0.5,
                flight_distance: 25.0,
                aggression: 0.3,
                curiosity: 0.3,
            },
            Self::BachelorBand => HerdBehavior {
                alertness: 0.65,
                flight_distance: 35.0,
                aggression: 0.5,
                curiosity: 0.55,
            },
        }
    }
}

/// Herd behavior modifiers
#[derive(Debug, Clone, Copy)]
pub struct HerdBehavior {
    pub alertness: f32,
    pub flight_distance: f32,
    pub aggression: f32,
    pub curiosity: f32,
}

/// Equipment slot types for horses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorseEquipmentSlot {
    Saddle,
    Bridle,
    SaddleBags,
    Blanket,
    Horseshoes,
    Armor,
    Decoration,
}

/// Quality tier for horse equipment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EquipmentQuality {
    Makeshift,
    Basic,
    Quality,
    Fine,
    Masterwork,
    Legendary,
}

impl EquipmentQuality {
    /// Get stat bonus multiplier
    pub fn bonus_multiplier(&self) -> f32 {
        match self {
            Self::Makeshift => 0.8,
            Self::Basic => 1.0,
            Self::Quality => 1.15,
            Self::Fine => 1.3,
            Self::Masterwork => 1.5,
            Self::Legendary => 2.0,
        }
    }
}
