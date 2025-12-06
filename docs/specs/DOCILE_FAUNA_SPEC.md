# Docile Fauna System Specification

## Roanoke Engine Wildlife Framework - Prey & Ambient Creatures

This document specifies the architecture for docile wildlife in Roanoke Engine, designed as a companion system to the dangerous wildlife (ANIMAL_SYSTEM_SPEC.md). These creatures provide hunting opportunities, ambient atmosphere, and ecosystem interactions.

---

## Table of Contents

1. [Overview](#overview)
2. [Core Data Structures](#core-data-structures)
3. [Species Definitions](#species-definitions)
4. [Behavior System](#behavior-system)
5. [Flight Response System](#flight-response-system)
6. [Grouping & Social Behavior](#grouping--social-behavior)
7. [Seasonal Behavior](#seasonal-behavior)
8. [Spawning System](#spawning-system)
9. [Harvesting System](#harvesting-system)
10. [Interaction System](#interaction-system)
11. [Environmental Impact](#environmental-impact)
12. [Rendering Integration](#rendering-integration)
13. [Audio Integration](#audio-integration)
14. [Implementation Phases](#implementation-phases)

---

## Overview

### Design Goals

- **Living World**: Fauna creates a believable, breathing ecosystem
- **Hunting Gameplay**: Provides prey for the Hunting Skill Tree
- **Ambient Beauty**: Fireflies, butterflies, songbirds enhance atmosphere
- **Ecosystem Dynamics**: Predator-prey relationships with hostile fauna
- **Seasonal Realism**: Behavior changes with seasons and time of day

### Relationship to Hostile Fauna

| Aspect | Hostile Fauna | Docile Fauna |
|--------|---------------|--------------|
| Danger Level | 1-10 | 0 |
| Primary Behavior | Hunt/Attack | Flee/Hide |
| Player Interaction | Combat | Hunting/Observation |
| Grouping | Packs (predatory) | Herds/Flocks (defensive) |
| Spawning | Territorial | Habitat-based |

### Species Count: 20 Docile Creatures

**Categories:**
- Large Mammals (1): White-tailed Deer
- Small Mammals (7): Rabbit, Raccoon, Opossum, Gray Squirrel, Flying Squirrel, Beaver, River Otter
- Large Birds (1): Wild Turkey
- Small Birds (4): Cardinal, Blue Jay, Wood Duck, Hummingbird, Black Skimmer
- Reptiles (2): Box Turtle, Painted Turtle
- Amphibians (2): Bullfrog, Wood Frog
- Insects (2): Monarch Butterfly, Firefly

---

## Core Data Structures

### Location: `roanoke_game/src/fauna/mod.rs`

```rust
//! Docile fauna system module
//!
//! Submodules:
//!   - species.rs     - Species definitions and stats
//!   - entity.rs      - Fauna entity struct
//!   - manager.rs     - Central fauna management
//!   - behavior.rs    - AI state machines (flee-focused)
//!   - spawner.rs     - Habitat-based spawning
//!   - harvest.rs     - Skinning and loot
//!   - interact.rs    - Feeding, taming, observation
//!   - groups.rs      - Herds, flocks, swarms
//!   - seasonal.rs    - Season-dependent behavior
```

### Fauna Species Enum

```rust
// species.rs

use serde::{Deserialize, Serialize};

/// All docile fauna species
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocileSpecies {
    // Large Mammals
    WhiteTailedDeer,

    // Small Mammals
    EasternCottontail,
    CommonRaccoon,
    VirginiaOpossum,
    GraySquirrel,
    FlyingSquirrel,
    AmericanBeaver,
    RiverOtter,

    // Large Birds
    WildTurkey,

    // Small Birds
    NorthernCardinal,
    BlueJay,
    WoodDuck,
    RubyThroatedHummingbird,
    BlackSkimmer,

    // Reptiles
    EasternBoxTurtle,
    PaintedTurtle,

    // Amphibians
    AmericanBullfrog,
    WoodFrog,

    // Insects
    MonarchButterfly,
    Firefly,
}

/// Category for shared behaviors and rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FaunaCategory {
    LargeMammal,
    SmallMammal,
    AquaticMammal,
    LargeBird,
    SmallBird,
    TinyBird,
    Waterfowl,
    ShoreBird,
    Reptile,
    Amphibian,
    Insect,
}

impl DocileSpecies {
    pub fn category(&self) -> FaunaCategory {
        match self {
            Self::WhiteTailedDeer => FaunaCategory::LargeMammal,
            Self::EasternCottontail | Self::CommonRaccoon |
            Self::VirginiaOpossum | Self::GraySquirrel |
            Self::FlyingSquirrel => FaunaCategory::SmallMammal,
            Self::AmericanBeaver | Self::RiverOtter => FaunaCategory::AquaticMammal,
            Self::WildTurkey => FaunaCategory::LargeBird,
            Self::NorthernCardinal | Self::BlueJay => FaunaCategory::SmallBird,
            Self::RubyThroatedHummingbird => FaunaCategory::TinyBird,
            Self::WoodDuck => FaunaCategory::Waterfowl,
            Self::BlackSkimmer => FaunaCategory::ShoreBird,
            Self::EasternBoxTurtle | Self::PaintedTurtle => FaunaCategory::Reptile,
            Self::AmericanBullfrog | Self::WoodFrog => FaunaCategory::Amphibian,
            Self::MonarchButterfly | Self::Firefly => FaunaCategory::Insect,
        }
    }
}
```

### Fauna Stats

```rust
// species.rs (continued)

/// Base stats for docile fauna
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaunaStats {
    pub health: f32,
    pub speed: f32,
    pub swim_speed: Option<f32>,      // For aquatic species
    pub glide_speed: Option<f32>,     // For flying squirrel
    pub detection_range: f32,          // How far they detect threats
    pub flee_range: f32,               // Distance at which they flee
    pub stamina_time: f32,             // How long they can run before tiring
}

/// Behavior archetype for docile fauna
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocileBehavior {
    Skittish,      // Easily frightened, flees quickly (deer)
    Timid,         // Shy, avoids contact (rabbit)
    Cautious,      // Wary but may investigate (turkey)
    Curious,       // May approach to investigate (raccoon)
    Docile,        // Calm and unbothered (box turtle)
    Playful,       // Engages in play behaviors (otter)
    Ambient,       // Background wildlife, minimal reaction (firefly)
    Bold,          // Less afraid, may mob threats (blue jay)
    Energetic,     // High activity, quick movements (squirrel)
    Passive,       // Minimal response, plays dead (opossum)
    Stationary,    // Mostly sits still (bullfrog)
    Basking,       // Sun-bathing behavior (painted turtle)
    Hyperactive,   // Constant rapid movement (hummingbird)
    Secretive,     // Hides, rarely seen (wood frog)
    Nocturnal,     // Active at night (flying squirrel)
    Industrious,   // Constantly working (beaver)
    Territorial,   // Defends feeding area (cardinal)
    Active,        // High mobility (black skimmer)
    Shy,           // Very wary of approach (wood duck)
    Peaceful,      // Gentle, no threat response (butterfly)
}

impl DocileBehavior {
    /// Multiplier for how quickly awareness increases
    pub fn alert_multiplier(&self) -> f32 {
        match self {
            Self::Skittish => 1.5,
            Self::Timid => 1.2,
            Self::Cautious => 1.0,
            Self::Curious => 0.8,
            Self::Docile => 0.5,
            Self::Playful => 0.7,
            Self::Ambient => 0.3,
            Self::Bold => 0.6,
            Self::Energetic => 1.1,
            Self::Passive => 0.4,
            Self::Stationary => 0.6,
            Self::Basking => 0.7,
            Self::Hyperactive => 1.3,
            Self::Secretive => 1.4,
            Self::Nocturnal => 0.9,
            Self::Industrious => 0.8,
            Self::Territorial => 0.7,
            Self::Active => 1.0,
            Self::Shy => 1.3,
            Self::Peaceful => 0.2,
        }
    }

    /// Bonus to flee speed
    pub fn flee_speed_bonus(&self) -> f32 {
        match self {
            Self::Skittish => 10.0,
            Self::Timid => 5.0,
            Self::Docile => -5.0,
            Self::Passive => -10.0,
            Self::Hyperactive => 15.0,
            _ => 0.0,
        }
    }

    /// Chance to hide instead of flee
    pub fn hide_chance(&self) -> f32 {
        match self {
            Self::Timid => 0.7,
            Self::Secretive => 0.8,
            Self::Passive => 0.9,  // Plays dead
            Self::Nocturnal => 0.5,
            _ => 0.0,
        }
    }

    /// Chance to investigate player instead of flee
    pub fn investigate_chance(&self) -> f32 {
        match self {
            Self::Curious => 0.6,
            Self::Cautious => 0.3,
            Self::Bold => 0.4,
            Self::Playful => 0.3,
            _ => 0.0,
        }
    }
}
```

---

## Species Definitions

### Complete Species Reference Table

| ID | Name | Category | HP | Speed | Swim | Detection | Flee | Behavior | Spawn |
|----|------|----------|----|----|------|-----------|------|----------|-------|
| white_tailed_deer | White-tailed Deer | Large Mammal | 60 | 45 | - | 30 | 20 | Skittish | 0.40 |
| eastern_cottontail | Eastern Cottontail | Small Mammal | 15 | 35 | - | 20 | 15 | Timid | 0.50 |
| wild_turkey | Wild Turkey | Large Bird | 40 | 25 | - | 35 | 20 | Cautious | 0.35 |
| beaver | American Beaver | Aquatic Mammal | 50 | 15 | 30 | 25 | 15 | Industrious | 0.25 |
| raccoon | Common Raccoon | Small Mammal | 30 | 20 | - | 30 | 10 | Curious | 0.30 |
| opossum | Virginia Opossum | Small Mammal | 25 | 15 | - | 20 | 12 | Passive | 0.28 |
| gray_squirrel | Eastern Gray Squirrel | Small Mammal | 12 | 25 | - | 25 | 15 | Energetic | 0.60 |
| box_turtle | Eastern Box Turtle | Reptile | 20 | 3 | - | 10 | 2 | Docile | 0.20 |
| monarch_butterfly | Monarch Butterfly | Insect | 1 | 15 | - | 5 | 10 | Peaceful | 0.40 |
| cardinal | Northern Cardinal | Small Bird | 8 | 30 | - | 20 | 15 | Territorial | 0.45 |
| blue_jay | Blue Jay | Small Bird | 10 | 32 | - | 30 | 20 | Bold | 0.40 |
| wood_duck | Wood Duck | Waterfowl | 25 | 20 | 25 | 25 | 20 | Shy | 0.25 |
| bullfrog | American Bullfrog | Amphibian | 15 | 10 | 20 | 15 | 8 | Stationary | 0.35 |
| firefly | Firefly | Insect | 1 | 8 | - | 3 | 5 | Ambient | 0.70 |
| black_skimmer | Black Skimmer | Shore Bird | 20 | 35 | - | 30 | 25 | Active | 0.20 |
| river_otter | River Otter | Aquatic Mammal | 45 | 18 | 40 | 25 | 15 | Playful | 0.15 |
| painted_turtle | Painted Turtle | Reptile | 18 | 4 | 15 | 12 | 8 | Basking | 0.30 |
| hummingbird | Ruby-throated Hummingbird | Tiny Bird | 3 | 50 | - | 15 | 10 | Hyperactive | 0.25 |
| wood_frog | Wood Frog | Amphibian | 8 | 8 | - | 10 | 6 | Secretive | 0.20 |
| flying_squirrel | Southern Flying Squirrel | Small Mammal | 10 | 20 | - | 20 | 15 | Nocturnal | 0.15 |

### Detailed Species Definitions

```rust
// species.rs (continued)

/// Complete species definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocileSpeciesDef {
    pub id: DocileSpecies,
    pub name: &'static str,
    pub scientific_name: &'static str,
    pub category: FaunaCategory,
    pub behavior: DocileBehavior,
    pub stats: FaunaStats,
    pub habitats: Vec<Habitat>,
    pub grouping: GroupingDef,
    pub harvest: HarvestDef,
    pub interactions: InteractionDef,
    pub sounds: Vec<&'static str>,
    pub animations: Vec<&'static str>,
    pub spawn_rate: f32,
    pub active_times: Vec<TimeOfDay>,
    pub seasonal_behavior: SeasonalBehavior,
    pub flight_response: FlightResponse,
    pub unique_behavior: Option<UniqueBehavior>,
}

/// Get the complete definition for a species
pub fn get_species_def(species: DocileSpecies) -> DocileSpeciesDef {
    match species {
        DocileSpecies::WhiteTailedDeer => DocileSpeciesDef {
            id: DocileSpecies::WhiteTailedDeer,
            name: "White-tailed Deer",
            scientific_name: "Odocoileus virginianus",
            category: FaunaCategory::LargeMammal,
            behavior: DocileBehavior::Skittish,
            stats: FaunaStats {
                health: 60.0,
                speed: 45.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 30.0,
                flee_range: 20.0,
                stamina_time: 15.0,
            },
            habitats: vec![Habitat::Forests, Habitat::Meadows, Habitat::ForestEdges],
            grouping: GroupingDef {
                group_type: GroupType::Herd,
                size_min: 3,
                size_max: 8,
                flees_together: true,
            },
            harvest: HarvestDef {
                meat: 8,
                hide: 1,
                antlers: Some(1),
                bones: 4,
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["corn", "apples", "berries"],
            },
            sounds: vec!["snort", "bleat", "stomp"],
            animations: vec!["grazing", "alert", "jumping", "running"],
            spawn_rate: 0.40,
            active_times: vec![TimeOfDay::Dawn, TimeOfDay::Dusk],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::NewbornsPresent,
                summer: SeasonState::Normal,
                fall: SeasonState::MatingSeason,
                winter: SeasonState::GroupedForaging,
            },
            flight_response: FlightResponse {
                trigger_distance: 15.0,
                zigzag_pattern: true,
                jump_obstacles: true,
                warns_others: true,
                ..Default::default()
            },
            unique_behavior: None,
        },

        DocileSpecies::EasternCottontail => DocileSpeciesDef {
            id: DocileSpecies::EasternCottontail,
            name: "Eastern Cottontail Rabbit",
            scientific_name: "Sylvilagus floridanus",
            category: FaunaCategory::SmallMammal,
            behavior: DocileBehavior::Timid,
            stats: FaunaStats {
                health: 15.0,
                speed: 35.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 20.0,
                flee_range: 15.0,
                stamina_time: 8.0,
            },
            habitats: vec![Habitat::Meadows, Habitat::ForestEdges, Habitat::Brushlands],
            grouping: GroupingDef {
                group_type: GroupType::Solitary,
                size_min: 1,
                size_max: 2,
                flees_together: false,
            },
            harvest: HarvestDef {
                meat: 1,
                hide: 1,
                bones: 1,
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: true,
                petable: true,
                rideable: false,
                food_preference: vec!["carrots", "lettuce", "clover"],
            },
            sounds: vec!["thump", "squeal"],
            animations: vec!["hopping", "grooming", "eating", "hiding"],
            spawn_rate: 0.50,
            active_times: vec![TimeOfDay::Dawn, TimeOfDay::Dusk, TimeOfDay::Night],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::BreedingActive,
                summer: SeasonState::Normal,
                fall: SeasonState::Fattening,
                winter: SeasonState::LessActive,
            },
            flight_response: FlightResponse {
                trigger_distance: 10.0,
                zigzag_pattern: true,
                hides_in_burrows: true,
                freezes_first: true,
                ..Default::default()
            },
            unique_behavior: None,
        },

        DocileSpecies::WildTurkey => DocileSpeciesDef {
            id: DocileSpecies::WildTurkey,
            name: "Wild Turkey",
            scientific_name: "Meleagris gallopavo",
            category: FaunaCategory::LargeBird,
            behavior: DocileBehavior::Cautious,
            stats: FaunaStats {
                health: 40.0,
                speed: 25.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 35.0,
                flee_range: 20.0,
                stamina_time: 10.0,
            },
            habitats: vec![Habitat::Forests, Habitat::ForestEdges, Habitat::OakGroves],
            grouping: GroupingDef {
                group_type: GroupType::Flock,
                size_min: 5,
                size_max: 15,
                flees_together: true,
            },
            harvest: HarvestDef {
                meat: 4,
                feathers: Some(20),
                bones: 2,
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["seeds", "berries", "insects"],
            },
            sounds: vec!["gobble", "purr", "cluck"],
            animations: vec!["pecking", "strutting", "roosting", "flying_short"],
            spawn_rate: 0.35,
            active_times: vec![TimeOfDay::Dawn, TimeOfDay::Day, TimeOfDay::Dusk],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::MatingDisplays,
                summer: SeasonState::Normal,
                fall: SeasonState::Flocking,
                winter: SeasonState::LargeFlocks,
            },
            flight_response: FlightResponse {
                trigger_distance: 18.0,
                can_fly_short: true,
                roosts_in_trees: true,
                alerts_flock: true,
                ..Default::default()
            },
            unique_behavior: None,
        },

        DocileSpecies::AmericanBeaver => DocileSpeciesDef {
            id: DocileSpecies::AmericanBeaver,
            name: "American Beaver",
            scientific_name: "Castor canadensis",
            category: FaunaCategory::AquaticMammal,
            behavior: DocileBehavior::Industrious,
            stats: FaunaStats {
                health: 50.0,
                speed: 15.0,
                swim_speed: Some(30.0),
                glide_speed: None,
                detection_range: 25.0,
                flee_range: 15.0,
                stamina_time: 12.0,
            },
            habitats: vec![Habitat::Rivers, Habitat::Ponds, Habitat::Streams],
            grouping: GroupingDef {
                group_type: GroupType::Family,
                size_min: 2,
                size_max: 6,
                flees_together: true,
            },
            harvest: HarvestDef {
                meat: 3,
                pelt: Some(1),
                castoreum: Some(1),
                teeth: Some(2),
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: false,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["bark", "water_plants"],
            },
            sounds: vec!["splash", "whine", "tail_slap"],
            animations: vec!["swimming", "building", "gnawing", "diving"],
            spawn_rate: 0.25,
            active_times: vec![TimeOfDay::Dusk, TimeOfDay::Night, TimeOfDay::Dawn],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::DamRepair,
                summer: SeasonState::Normal,
                fall: SeasonState::FoodCaching,
                winter: SeasonState::LodgeBound,
            },
            flight_response: FlightResponse {
                trigger_distance: 12.0,
                dives_underwater: true,
                warns_with_tail_slap: true,
                hide_in_lodge: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::EnvironmentalEngineer {
                creates_dams: true,
                modifies_waterways: true,
                creates_wetlands: true,
            }),
        },

        DocileSpecies::CommonRaccoon => DocileSpeciesDef {
            id: DocileSpecies::CommonRaccoon,
            name: "Common Raccoon",
            scientific_name: "Procyon lotor",
            category: FaunaCategory::SmallMammal,
            behavior: DocileBehavior::Curious,
            stats: FaunaStats {
                health: 30.0,
                speed: 20.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 30.0,
                flee_range: 10.0,
                stamina_time: 10.0,
            },
            habitats: vec![Habitat::Forests, Habitat::NearWater, Habitat::HollowTrees],
            grouping: GroupingDef {
                group_type: GroupType::Solitary,
                size_min: 1,
                size_max: 3,
                flees_together: false,
            },
            harvest: HarvestDef {
                meat: 2,
                pelt: Some(1),
                fat: Some(1),
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["corn", "eggs", "berries", "fish"],
            },
            sounds: vec!["chitter", "purr", "growl"],
            animations: vec!["washing", "climbing", "foraging", "standing"],
            spawn_rate: 0.30,
            active_times: vec![TimeOfDay::Night, TimeOfDay::Dusk],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::Active,
                summer: SeasonState::Normal,
                fall: SeasonState::Fattening,
                winter: SeasonState::Denning,
            },
            flight_response: FlightResponse {
                trigger_distance: 8.0,
                climbs_trees: true,
                investigates_first: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::Mischievous {
                washes_food: true,
                steals_shiny_objects: true,
                raids_camps: true,
            }),
        },

        DocileSpecies::VirginiaOpossum => DocileSpeciesDef {
            id: DocileSpecies::VirginiaOpossum,
            name: "Virginia Opossum",
            scientific_name: "Didelphis virginiana",
            category: FaunaCategory::SmallMammal,
            behavior: DocileBehavior::Passive,
            stats: FaunaStats {
                health: 25.0,
                speed: 15.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 20.0,
                flee_range: 12.0,
                stamina_time: 6.0,
            },
            habitats: vec![Habitat::Forests, Habitat::Brushlands, Habitat::NearWater],
            grouping: GroupingDef {
                group_type: GroupType::Solitary,
                size_min: 1,
                size_max: 1,
                flees_together: false,
            },
            harvest: HarvestDef {
                meat: 2,
                hide: 1,
                fat: Some(1),
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["fruits", "insects", "eggs"],
            },
            sounds: vec!["hiss", "click", "growl"],
            animations: vec!["waddling", "playing_dead", "climbing", "grooming"],
            spawn_rate: 0.28,
            active_times: vec![TimeOfDay::Night],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::Active,
                summer: SeasonState::Normal,
                fall: SeasonState::Foraging,
                winter: SeasonState::Sluggish,
            },
            flight_response: FlightResponse {
                trigger_distance: 6.0,
                plays_dead: true,
                shows_teeth: true,
                slow_moving: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::Opossum {
                immune_to_rabies: true,
                eats_venomous_snakes: true,
                carriable_babies: true,
            }),
        },

        DocileSpecies::GraySquirrel => DocileSpeciesDef {
            id: DocileSpecies::GraySquirrel,
            name: "Eastern Gray Squirrel",
            scientific_name: "Sciurus carolinensis",
            category: FaunaCategory::SmallMammal,
            behavior: DocileBehavior::Energetic,
            stats: FaunaStats {
                health: 12.0,
                speed: 25.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 25.0,
                flee_range: 15.0,
                stamina_time: 10.0,
            },
            habitats: vec![Habitat::Forests, Habitat::OakGroves, Habitat::PineForests],
            grouping: GroupingDef {
                group_type: GroupType::LooseGroup,
                size_min: 2,
                size_max: 5,
                flees_together: false,
            },
            harvest: HarvestDef {
                meat: 1,
                hide: 1,
                tail: Some(1),
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["nuts", "acorns", "seeds"],
            },
            sounds: vec!["chatter", "bark", "squeak"],
            animations: vec!["jumping", "burying_nuts", "tail_flicking", "climbing"],
            spawn_rate: 0.60,
            active_times: vec![TimeOfDay::Day],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::NestBuilding,
                summer: SeasonState::Normal,
                fall: SeasonState::NutGathering,
                winter: SeasonState::LessActive,
            },
            flight_response: FlightResponse {
                trigger_distance: 10.0,
                spirals_up_trees: true,
                jumps_between_branches: true,
                freezes_on_tree_trunk: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::Squirrel {
                caches_food: true,
                deceptive_burying: true,
                acrobatic_jumps: true,
            }),
        },

        DocileSpecies::FlyingSquirrel => DocileSpeciesDef {
            id: DocileSpecies::FlyingSquirrel,
            name: "Southern Flying Squirrel",
            scientific_name: "Glaucomys volans",
            category: FaunaCategory::SmallMammal,
            behavior: DocileBehavior::Nocturnal,
            stats: FaunaStats {
                health: 10.0,
                speed: 20.0,
                swim_speed: None,
                glide_speed: Some(35.0),
                detection_range: 20.0,
                flee_range: 15.0,
                stamina_time: 12.0,
            },
            habitats: vec![Habitat::DeciduousForests, Habitat::MixedForests, Habitat::TreeCavities],
            grouping: GroupingDef {
                group_type: GroupType::Communal,
                size_min: 2,
                size_max: 8,
                flees_together: false,
            },
            harvest: HarvestDef {
                meat: 1,
                pelt: Some(1),
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["nuts", "berries", "fungi"],
            },
            sounds: vec!["chirp", "twitter", "soft_chatter"],
            animations: vec!["gliding", "climbing", "grooming", "landing"],
            spawn_rate: 0.15,
            active_times: vec![TimeOfDay::Night],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::Breeding,
                summer: SeasonState::RaisingYoung,
                fall: SeasonState::NutGathering,
                winter: SeasonState::CommunalNesting,
            },
            flight_response: FlightResponse {
                trigger_distance: 8.0,
                glides_to_safety: true,
                spirals_up_tree: true,
                silent_escape: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::FlyingSquirrel {
                glides: true,
                communal_nesting: true,
                uv_fluorescent: true,
            }),
        },

        DocileSpecies::RiverOtter => DocileSpeciesDef {
            id: DocileSpecies::RiverOtter,
            name: "North American River Otter",
            scientific_name: "Lontra canadensis",
            category: FaunaCategory::AquaticMammal,
            behavior: DocileBehavior::Playful,
            stats: FaunaStats {
                health: 45.0,
                speed: 18.0,
                swim_speed: Some(40.0),
                glide_speed: None,
                detection_range: 25.0,
                flee_range: 15.0,
                stamina_time: 20.0,
            },
            habitats: vec![Habitat::Rivers, Habitat::Lakes, Habitat::Marshes],
            grouping: GroupingDef {
                group_type: GroupType::Family,
                size_min: 2,
                size_max: 4,
                flees_together: true,
            },
            harvest: HarvestDef {
                meat: 3,
                pelt: Some(1),
                fat: Some(2),
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: false,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["fish", "crayfish", "frogs"],
            },
            sounds: vec!["chirp", "growl", "whistle"],
            animations: vec!["swimming", "sliding", "playing", "diving"],
            spawn_rate: 0.15,
            active_times: vec![TimeOfDay::Dawn, TimeOfDay::Dusk],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::Active,
                summer: SeasonState::TeachingYoung,
                fall: SeasonState::Normal,
                winter: SeasonState::ActiveUnderIce,
            },
            flight_response: FlightResponse {
                trigger_distance: 10.0,
                dives_underwater: true,
                slides_to_water: true,
                uses_multiple_exits: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::Otter {
                makes_slides: true,
                plays_with_objects: true,
                social_grooming: true,
            }),
        },

        DocileSpecies::EasternBoxTurtle => DocileSpeciesDef {
            id: DocileSpecies::EasternBoxTurtle,
            name: "Eastern Box Turtle",
            scientific_name: "Terrapene carolina",
            category: FaunaCategory::Reptile,
            behavior: DocileBehavior::Docile,
            stats: FaunaStats {
                health: 20.0,
                speed: 3.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 10.0,
                flee_range: 2.0,
                stamina_time: 5.0,
            },
            habitats: vec![Habitat::Forests, Habitat::Meadows, Habitat::WetlandEdges],
            grouping: GroupingDef {
                group_type: GroupType::Solitary,
                size_min: 1,
                size_max: 1,
                flees_together: false,
            },
            harvest: HarvestDef {
                shell: Some(1),
                meat: 1,
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: true,
                petable: true,
                rideable: false,
                food_preference: vec!["berries", "mushrooms", "worms"],
            },
            sounds: vec!["hiss"],
            animations: vec!["walking_slow", "hiding_in_shell", "eating", "basking"],
            spawn_rate: 0.20,
            active_times: vec![TimeOfDay::Day],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::Active,
                summer: SeasonState::Normal,
                fall: SeasonState::Foraging,
                winter: SeasonState::Hibernating,
            },
            flight_response: FlightResponse {
                trigger_distance: 2.0,
                retreats_into_shell: true,
                very_slow_retreat: true,
                camouflages: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::BoxTurtle {
                longevity_years: 50,
                homing_instinct: true,
                closes_shell_completely: true,
            }),
        },

        DocileSpecies::PaintedTurtle => DocileSpeciesDef {
            id: DocileSpecies::PaintedTurtle,
            name: "Painted Turtle",
            scientific_name: "Chrysemys picta",
            category: FaunaCategory::Reptile,
            behavior: DocileBehavior::Basking,
            stats: FaunaStats {
                health: 18.0,
                speed: 4.0,
                swim_speed: Some(15.0),
                glide_speed: None,
                detection_range: 12.0,
                flee_range: 8.0,
                stamina_time: 15.0,
            },
            habitats: vec![Habitat::Ponds, Habitat::SlowRivers, Habitat::Marshes],
            grouping: GroupingDef {
                group_type: GroupType::BaskingGroup,
                size_min: 3,
                size_max: 12,
                flees_together: true,
            },
            harvest: HarvestDef {
                shell: Some(1),
                meat: 1,
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["aquatic_plants", "insects", "small_fish"],
            },
            sounds: vec!["splash"],
            animations: vec!["basking", "swimming", "diving", "crawling"],
            spawn_rate: 0.30,
            active_times: vec![TimeOfDay::Day],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::Emerging,
                summer: SeasonState::ActiveBasking,
                fall: SeasonState::PreparingHibernation,
                winter: SeasonState::HibernatingUnderwater,
            },
            flight_response: FlightResponse {
                trigger_distance: 5.0,
                slides_into_water: true,
                group_diving: true,
                hides_underwater: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::PaintedTurtle {
                stacks_on_logs: true,
                sun_bathes: true,
                hibernates_in_mud: true,
            }),
        },

        DocileSpecies::NorthernCardinal => DocileSpeciesDef {
            id: DocileSpecies::NorthernCardinal,
            name: "Northern Cardinal",
            scientific_name: "Cardinalis cardinalis",
            category: FaunaCategory::SmallBird,
            behavior: DocileBehavior::Territorial,
            stats: FaunaStats {
                health: 8.0,
                speed: 30.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 20.0,
                flee_range: 15.0,
                stamina_time: 12.0,
            },
            habitats: vec![Habitat::ForestEdges, Habitat::Brushlands, Habitat::Gardens],
            grouping: GroupingDef {
                group_type: GroupType::Pair,
                size_min: 2,
                size_max: 2,
                flees_together: true,
            },
            harvest: HarvestDef {
                feathers: Some(5),
                bones: 1,
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["seeds", "berries", "insects"],
            },
            sounds: vec!["chirp", "song", "chip"],
            animations: vec!["perching", "flying", "ground_hopping", "singing"],
            spawn_rate: 0.45,
            active_times: vec![TimeOfDay::Dawn, TimeOfDay::Day, TimeOfDay::Dusk],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::SingingMating,
                summer: SeasonState::Nesting,
                fall: SeasonState::Normal,
                winter: SeasonState::Flocking,
            },
            flight_response: FlightResponse {
                trigger_distance: 8.0,
                quick_takeoff: true,
                hides_in_brush: true,
                warns_others: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::Cardinal {
                year_round_resident: true,
                males_bright_red: true,
                morning_chorus: true,
            }),
        },

        DocileSpecies::BlueJay => DocileSpeciesDef {
            id: DocileSpecies::BlueJay,
            name: "Blue Jay",
            scientific_name: "Cyanocitta cristata",
            category: FaunaCategory::SmallBird,
            behavior: DocileBehavior::Bold,
            stats: FaunaStats {
                health: 10.0,
                speed: 32.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 30.0,
                flee_range: 20.0,
                stamina_time: 15.0,
            },
            habitats: vec![Habitat::Forests, Habitat::OakGroves, Habitat::ForestEdges],
            grouping: GroupingDef {
                group_type: GroupType::FamilyGroup,
                size_min: 3,
                size_max: 7,
                flees_together: true,
            },
            harvest: HarvestDef {
                feathers: Some(6),
                bones: 1,
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["nuts", "seeds", "insects"],
            },
            sounds: vec!["jay_call", "mimic", "alarm"],
            animations: vec!["flying", "caching_nuts", "mobbing", "perching"],
            spawn_rate: 0.40,
            active_times: vec![TimeOfDay::Day],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::Nesting,
                summer: SeasonState::Normal,
                fall: SeasonState::NutGathering,
                winter: SeasonState::Flocking,
            },
            flight_response: FlightResponse {
                trigger_distance: 12.0,
                loud_alarm_call: true,
                mobs_predators: true,
                intelligent_escape: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::BlueJay {
                mimics_hawks: true,
                caches_acorns: true,
                mobs_birds_of_prey: true,
            }),
        },

        DocileSpecies::WoodDuck => DocileSpeciesDef {
            id: DocileSpecies::WoodDuck,
            name: "Wood Duck",
            scientific_name: "Aix sponsa",
            category: FaunaCategory::Waterfowl,
            behavior: DocileBehavior::Shy,
            stats: FaunaStats {
                health: 25.0,
                speed: 20.0,
                swim_speed: Some(25.0),
                glide_speed: None,
                detection_range: 25.0,
                flee_range: 20.0,
                stamina_time: 18.0,
            },
            habitats: vec![Habitat::Ponds, Habitat::Swamps, Habitat::WoodedStreams],
            grouping: GroupingDef {
                group_type: GroupType::PairOrFlock,
                size_min: 2,
                size_max: 12,
                flees_together: true,
            },
            harvest: HarvestDef {
                meat: 2,
                feathers: Some(15),
                bones: 2,
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["aquatic_plants", "acorns", "seeds"],
            },
            sounds: vec!["whistle", "squeal", "call"],
            animations: vec!["swimming", "diving", "flying", "preening"],
            spawn_rate: 0.25,
            active_times: vec![TimeOfDay::Dawn, TimeOfDay::Dusk],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::NestingInTrees,
                summer: SeasonState::RaisingYoung,
                fall: SeasonState::Flocking,
                winter: SeasonState::SouthernMovement,
            },
            flight_response: FlightResponse {
                trigger_distance: 15.0,
                takes_off_from_water: true,
                whistles_alarm: true,
                agile_flight: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::WoodDuck {
                nests_in_tree_cavities: true,
                ducklings_jump_from_nest: true,
                colorful_plumage: true,
            }),
        },

        DocileSpecies::RubyThroatedHummingbird => DocileSpeciesDef {
            id: DocileSpecies::RubyThroatedHummingbird,
            name: "Ruby-throated Hummingbird",
            scientific_name: "Archilochus colubris",
            category: FaunaCategory::TinyBird,
            behavior: DocileBehavior::Hyperactive,
            stats: FaunaStats {
                health: 3.0,
                speed: 50.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 15.0,
                flee_range: 10.0,
                stamina_time: 5.0,
            },
            habitats: vec![Habitat::Gardens, Habitat::ForestEdges, Habitat::Meadows],
            grouping: GroupingDef {
                group_type: GroupType::Solitary,
                size_min: 1,
                size_max: 1,
                flees_together: false,
            },
            harvest: HarvestDef {
                feathers: Some(2),
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: true,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["nectar", "sugar_water"],
            },
            sounds: vec!["chirp", "wing_buzz", "twitter"],
            animations: vec!["hovering", "darting", "feeding", "territorial_chase"],
            spawn_rate: 0.25,
            active_times: vec![TimeOfDay::Day],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::ArrivingMigration,
                summer: SeasonState::Territorial,
                fall: SeasonState::FeedingFrenzy,
                winter: SeasonState::Absent,
            },
            flight_response: FlightResponse {
                trigger_distance: 5.0,
                instant_acceleration: true,
                hovers: true,
                flies_backward: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::Hummingbird {
                pollinates: true,
                torpor: true,
                aggressive_territorial: true,
            }),
        },

        DocileSpecies::BlackSkimmer => DocileSpeciesDef {
            id: DocileSpecies::BlackSkimmer,
            name: "Black Skimmer",
            scientific_name: "Rynchops niger",
            category: FaunaCategory::ShoreBird,
            behavior: DocileBehavior::Active,
            stats: FaunaStats {
                health: 20.0,
                speed: 35.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 30.0,
                flee_range: 25.0,
                stamina_time: 25.0,
            },
            habitats: vec![Habitat::Beaches, Habitat::CoastalWaters, Habitat::Estuaries],
            grouping: GroupingDef {
                group_type: GroupType::Colony,
                size_min: 5,
                size_max: 20,
                flees_together: true,
            },
            harvest: HarvestDef {
                feathers: Some(8),
                bones: 2,
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: false,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["small_fish"],
            },
            sounds: vec!["bark", "yip", "call"],
            animations: vec!["skimming", "flying", "resting", "preening"],
            spawn_rate: 0.20,
            active_times: vec![TimeOfDay::Dawn, TimeOfDay::Dusk, TimeOfDay::Night],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::NestingColonies,
                summer: SeasonState::FishingActive,
                fall: SeasonState::Grouping,
                winter: SeasonState::CoastalMovement,
            },
            flight_response: FlightResponse {
                trigger_distance: 20.0,
                graceful_takeoff: true,
                low_flight: true,
                colonial_defense: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::Skimmer {
                skims_fishing: true,
                unique_bill_shape: true,
                night_fishing: true,
            }),
        },

        DocileSpecies::AmericanBullfrog => DocileSpeciesDef {
            id: DocileSpecies::AmericanBullfrog,
            name: "American Bullfrog",
            scientific_name: "Lithobates catesbeianus",
            category: FaunaCategory::Amphibian,
            behavior: DocileBehavior::Stationary,
            stats: FaunaStats {
                health: 15.0,
                speed: 10.0,
                swim_speed: Some(20.0),
                glide_speed: None,
                detection_range: 15.0,
                flee_range: 8.0,
                stamina_time: 8.0,
            },
            habitats: vec![Habitat::Ponds, Habitat::Marshes, Habitat::SlowRivers],
            grouping: GroupingDef {
                group_type: GroupType::LooseGroup,
                size_min: 3,
                size_max: 10,
                flees_together: false,
            },
            harvest: HarvestDef {
                frog_legs: Some(2),
                skin: Some(1),
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: false,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["insects", "small_fish"],
            },
            sounds: vec!["deep_croak", "splash", "grunt"],
            animations: vec!["sitting", "jumping", "swimming", "catching_prey"],
            spawn_rate: 0.35,
            active_times: vec![TimeOfDay::Night, TimeOfDay::Dusk],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::LoudChorusing,
                summer: SeasonState::Active,
                fall: SeasonState::PreparingHibernation,
                winter: SeasonState::Hibernating,
            },
            flight_response: FlightResponse {
                trigger_distance: 5.0,
                jumps_to_water: true,
                dive_deep: true,
                camouflages_in_mud: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::Bullfrog {
                territorial_calls: true,
                eats_anything_smaller: true,
                tadpole_stage: true,
            }),
        },

        DocileSpecies::WoodFrog => DocileSpeciesDef {
            id: DocileSpecies::WoodFrog,
            name: "Wood Frog",
            scientific_name: "Lithobates sylvaticus",
            category: FaunaCategory::Amphibian,
            behavior: DocileBehavior::Secretive,
            stats: FaunaStats {
                health: 8.0,
                speed: 8.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 10.0,
                flee_range: 6.0,
                stamina_time: 5.0,
            },
            habitats: vec![Habitat::Woodlands, Habitat::ForestPonds, Habitat::VernalPools],
            grouping: GroupingDef {
                group_type: GroupType::BreedingAggregation,
                size_min: 5,
                size_max: 30,
                flees_together: false,
            },
            harvest: HarvestDef {
                frog_legs: Some(1),
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: false,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["insects", "worms"],
            },
            sounds: vec!["quack_croak", "splash"],
            animations: vec!["hopping", "sitting", "swimming", "calling"],
            spawn_rate: 0.20,
            active_times: vec![TimeOfDay::Night],  // Also rain
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::ExplosiveBreeding,
                summer: SeasonState::ForestDwelling,
                fall: SeasonState::PreparingFreeze,
                winter: SeasonState::FrozenHibernation,
            },
            flight_response: FlightResponse {
                trigger_distance: 4.0,
                erratic_jumping: true,
                hides_under_leaves: true,
                freezes_in_place: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::WoodFrog {
                freeze_tolerant: true,
                explosive_breeder: true,
                terrestrial_adult: true,
            }),
        },

        DocileSpecies::MonarchButterfly => DocileSpeciesDef {
            id: DocileSpecies::MonarchButterfly,
            name: "Monarch Butterfly",
            scientific_name: "Danaus plexippus",
            category: FaunaCategory::Insect,
            behavior: DocileBehavior::Peaceful,
            stats: FaunaStats {
                health: 1.0,
                speed: 15.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 5.0,
                flee_range: 10.0,
                stamina_time: 20.0,
            },
            habitats: vec![Habitat::Meadows, Habitat::Gardens, Habitat::MilkweedPatches],
            grouping: GroupingDef {
                group_type: GroupType::Swarm,
                size_min: 5,
                size_max: 50,
                flees_together: false,
            },
            harvest: HarvestDef {
                butterfly_wing: Some(2),
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: false,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["nectar"],
            },
            sounds: vec![],
            animations: vec!["flying_erratic", "feeding", "resting", "mating_dance"],
            spawn_rate: 0.40,
            active_times: vec![TimeOfDay::Day],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::Arriving,
                summer: SeasonState::Breeding,
                fall: SeasonState::Migrating,
                winter: SeasonState::Absent,
            },
            flight_response: FlightResponse {
                trigger_distance: 3.0,
                erratic_flight: true,
                height_variation: true,
                wind_affected: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::Butterfly {
                migration: true,
                pollinates: true,
                toxic_to_eat: true,
            }),
        },

        DocileSpecies::Firefly => DocileSpeciesDef {
            id: DocileSpecies::Firefly,
            name: "Firefly",
            scientific_name: "Photinus pyralis",
            category: FaunaCategory::Insect,
            behavior: DocileBehavior::Ambient,
            stats: FaunaStats {
                health: 1.0,
                speed: 8.0,
                swim_speed: None,
                glide_speed: None,
                detection_range: 3.0,
                flee_range: 5.0,
                stamina_time: 30.0,
            },
            habitats: vec![Habitat::Meadows, Habitat::ForestEdges, Habitat::Wetlands],
            grouping: GroupingDef {
                group_type: GroupType::Swarm,
                size_min: 10,
                size_max: 100,
                flees_together: false,
            },
            harvest: HarvestDef {
                glowworm: Some(1),
                ..Default::default()
            },
            interactions: InteractionDef {
                feedable: false,
                tameable: false,
                petable: false,
                rideable: false,
                food_preference: vec!["nectar", "pollen"],
            },
            sounds: vec![],
            animations: vec!["floating", "blinking", "landing", "mating_flash"],
            spawn_rate: 0.70,
            active_times: vec![TimeOfDay::Night, TimeOfDay::Dusk],
            seasonal_behavior: SeasonalBehavior {
                spring: SeasonState::Emerging,
                summer: SeasonState::PeakActivity,
                fall: SeasonState::Declining,
                winter: SeasonState::Absent,
            },
            flight_response: FlightResponse {
                trigger_distance: 2.0,
                random_pattern: true,
                turns_off_light: true,
                slow_floating: true,
                ..Default::default()
            },
            unique_behavior: Some(UniqueBehavior::Firefly {
                bioluminescence: true,
                synchronous_flashing: true,
                mating_signals: true,
                light_color: "yellow-green",
                light_pattern: LightPattern::Intermittent,
            }),
        },
    }
}
```

---

## Behavior System

### Docile Behavior State Machine

```rust
// behavior.rs

/// High-level behavior states for docile fauna
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaunaBehaviorState {
    Idle(IdleSubstate),
    Foraging(ForagingSubstate),
    Alert(AlertSubstate),
    Fleeing(FleeSubstate),
    Hiding(HideSubstate),
    Social(SocialSubstate),
    Resting(RestSubstate),
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleSubstate {
    Standing,
    Sitting,
    Grooming,
    LookingAround,
    Basking,      // Turtles
    Floating,     // Fireflies, butterflies
    Perching,     // Birds
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForagingSubstate {
    Searching,
    Eating,
    Drinking,
    Caching,      // Squirrels burying nuts
    Hunting,      // Frogs catching insects
    Grazing,      // Deer
    Pecking,      // Birds, turkeys
    Fishing,      // Otters, skimmers
    Building,     // Beavers
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSubstate {
    Listening,
    LookingAtThreat,
    FreezingInPlace,
    WarningOthers,
    Investigating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FleeSubstate {
    Running,
    Flying,
    Swimming,
    Diving,
    Gliding,      // Flying squirrel
    Jumping,      // Rabbits, frogs
    ZigZagging,   // Deer, rabbits
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HideSubstate {
    InBurrow,
    InShell,      // Turtles
    InWater,
    InTree,
    InBrush,
    UnderLeaves,
    PlayingDead,  // Opossum
    InLodge,      // Beaver
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocialSubstate {
    Following,     // Herd/flock member
    Leading,       // Herd/flock leader
    Playing,       // Otters
    Mating,
    Calling,       // Frogs, birds
    Mobbing,       // Blue jays
    Sliding,       // Otters
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestSubstate {
    Sleeping,
    Roosting,     // Birds in trees
    Denning,      // Winter rest
    Hibernating,
    Torpor,       // Hummingbirds
}
```

### Behavior Update Logic

```rust
// behavior.rs (continued)

pub struct FaunaBehaviorContext<'a> {
    pub fauna: &'a DocileFauna,
    pub species_def: &'a DocileSpeciesDef,
    pub player: &'a Player,
    pub predators_nearby: &'a [AnimalId],  // From hostile fauna system
    pub group_members: &'a [FaunaId],
    pub time_of_day: TimeOfDay,
    pub season: Season,
    pub weather: Weather,
    pub dt: f32,
}

pub fn update_fauna_behavior(ctx: &FaunaBehaviorContext, fauna: &mut DocileFauna) {
    let player_dist = fauna.position.distance(ctx.player.position);
    let threat_detected = player_dist < ctx.species_def.stats.detection_range
        || !ctx.predators_nearby.is_empty();

    // Update awareness level
    update_awareness(fauna, ctx, threat_detected);

    // State machine transitions
    let new_state = match fauna.behavior_state {
        FaunaBehaviorState::Idle(_) | FaunaBehaviorState::Foraging(_) => {
            if should_flee(fauna, ctx) {
                initiate_flee(fauna, ctx)
            } else if should_hide(fauna, ctx) {
                initiate_hide(fauna, ctx)
            } else if fauna.awareness > 0.5 {
                FaunaBehaviorState::Alert(AlertSubstate::LookingAtThreat)
            } else if should_forage(fauna, ctx) {
                initiate_foraging(fauna, ctx)
            } else {
                fauna.behavior_state
            }
        },

        FaunaBehaviorState::Alert(sub) => {
            if should_flee(fauna, ctx) {
                initiate_flee(fauna, ctx)
            } else if fauna.awareness < 0.3 {
                FaunaBehaviorState::Idle(IdleSubstate::Standing)
            } else if ctx.species_def.behavior.investigate_chance() > 0.0
                && rand::random::<f32>() < ctx.species_def.behavior.investigate_chance() * ctx.dt
            {
                FaunaBehaviorState::Alert(AlertSubstate::Investigating)
            } else {
                fauna.behavior_state
            }
        },

        FaunaBehaviorState::Fleeing(_) => {
            if is_safe(fauna, ctx) {
                FaunaBehaviorState::Alert(AlertSubstate::LookingAtThreat)
            } else if fauna.stamina <= 0.0 {
                // Exhausted, must hide
                initiate_hide(fauna, ctx)
            } else {
                fauna.behavior_state
            }
        },

        FaunaBehaviorState::Hiding(_) => {
            if is_safe(fauna, ctx) && fauna.hide_timer <= 0.0 {
                FaunaBehaviorState::Alert(AlertSubstate::Listening)
            } else {
                fauna.behavior_state
            }
        },

        FaunaBehaviorState::Social(_) => {
            if should_flee(fauna, ctx) {
                initiate_flee(fauna, ctx)
            } else {
                fauna.behavior_state
            }
        },

        FaunaBehaviorState::Resting(_) => {
            if should_flee(fauna, ctx) {
                initiate_flee(fauna, ctx)
            } else if is_active_time(ctx.time_of_day, &ctx.species_def.active_times) {
                FaunaBehaviorState::Idle(IdleSubstate::Standing)
            } else {
                fauna.behavior_state
            }
        },

        FaunaBehaviorState::Dead => FaunaBehaviorState::Dead,
    };

    fauna.behavior_state = new_state;
    execute_fauna_state(fauna, ctx);
}

fn should_flee(fauna: &DocileFauna, ctx: &FaunaBehaviorContext) -> bool {
    let player_dist = fauna.position.distance(ctx.player.position);
    let flee_threshold = ctx.species_def.flight_response.trigger_distance;

    // Predators always trigger flee
    if !ctx.predators_nearby.is_empty() {
        return true;
    }

    // Player within flee range and awareness high enough
    player_dist < flee_threshold && fauna.awareness > 0.7
}

fn should_hide(fauna: &DocileFauna, ctx: &FaunaBehaviorContext) -> bool {
    let hide_chance = ctx.species_def.behavior.hide_chance();
    hide_chance > 0.0 && fauna.awareness > 0.8 && rand::random::<f32>() < hide_chance
}
```

---

## Flight Response System

### Flight Response Definition

```rust
// behavior.rs (continued)

/// Defines how a species flees from threats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlightResponse {
    pub trigger_distance: f32,

    // Movement patterns
    pub zigzag_pattern: bool,
    pub jump_obstacles: bool,
    pub erratic_flight: bool,
    pub erratic_jumping: bool,

    // Escape methods
    pub climbs_trees: bool,
    pub spirals_up_trees: bool,
    pub jumps_between_branches: bool,
    pub freezes_on_tree_trunk: bool,
    pub glides_to_safety: bool,
    pub spirals_up_tree: bool,
    pub silent_escape: bool,

    // Water escape
    pub dives_underwater: bool,
    pub jumps_to_water: bool,
    pub slides_into_water: bool,
    pub takes_off_from_water: bool,
    pub dive_deep: bool,
    pub slides_to_water: bool,
    pub uses_multiple_exits: bool,
    pub group_diving: bool,
    pub hides_underwater: bool,

    // Hiding
    pub hides_in_burrows: bool,
    pub freezes_first: bool,
    pub hides_in_brush: bool,
    pub hides_under_leaves: bool,
    pub freezes_in_place: bool,
    pub retreats_into_shell: bool,
    pub very_slow_retreat: bool,
    pub camouflages: bool,
    pub camouflages_in_mud: bool,
    pub hide_in_lodge: bool,

    // Flying
    pub can_fly_short: bool,
    pub roosts_in_trees: bool,
    pub quick_takeoff: bool,
    pub agile_flight: bool,
    pub graceful_takeoff: bool,
    pub low_flight: bool,
    pub instant_acceleration: bool,
    pub hovers: bool,
    pub flies_backward: bool,
    pub height_variation: bool,
    pub wind_affected: bool,

    // Social responses
    pub warns_others: bool,
    pub alerts_flock: bool,
    pub whistles_alarm: bool,
    pub warns_with_tail_slap: bool,
    pub loud_alarm_call: bool,
    pub mobs_predators: bool,
    pub intelligent_escape: bool,
    pub colonial_defense: bool,

    // Passive defense
    pub plays_dead: bool,
    pub shows_teeth: bool,
    pub slow_moving: bool,
    pub investigates_first: bool,
    pub random_pattern: bool,
    pub turns_off_light: bool,
    pub slow_floating: bool,
}

/// Execute flight behavior based on species response
pub fn execute_flee(fauna: &mut DocileFauna, ctx: &FaunaBehaviorContext) {
    let response = &ctx.species_def.flight_response;
    let threat_pos = get_primary_threat_position(fauna, ctx);

    // Calculate flee direction (away from threat)
    let flee_dir = (fauna.position - threat_pos).normalize();

    // Apply species-specific flee modifiers
    let flee_speed = ctx.species_def.stats.speed
        + ctx.species_def.behavior.flee_speed_bonus();

    // Zigzag pattern
    let final_dir = if response.zigzag_pattern {
        let zigzag_offset = (fauna.flee_time * 3.0).sin() * 0.5;
        let perp = Vec3::new(-flee_dir.z, 0.0, flee_dir.x);
        (flee_dir + perp * zigzag_offset).normalize()
    } else {
        flee_dir
    };

    // Update velocity
    fauna.velocity = final_dir * flee_speed;

    // Handle special escape methods
    if response.climbs_trees && is_near_tree(fauna.position) {
        fauna.behavior_state = FaunaBehaviorState::Hiding(HideSubstate::InTree);
    } else if response.dives_underwater && is_near_water(fauna.position) {
        fauna.behavior_state = FaunaBehaviorState::Fleeing(FleeSubstate::Diving);
    } else if response.glides_to_safety && fauna.position.y > 5.0 {
        fauna.behavior_state = FaunaBehaviorState::Fleeing(FleeSubstate::Gliding);
    }

    // Drain stamina
    fauna.stamina -= ctx.dt / ctx.species_def.stats.stamina_time;
    fauna.flee_time += ctx.dt;
}
```

---

## Grouping & Social Behavior

### Group Types

```rust
// groups.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupType {
    Solitary,           // Lives alone (raccoon, opossum)
    Pair,               // Mated pair (cardinal)
    Family,             // Family unit (beaver, otter)
    FamilyGroup,        // Extended family (blue jay)
    Herd,               // Large prey group (deer)
    Flock,              // Bird group (turkey)
    LooseGroup,         // Casual association (squirrel, bullfrog)
    Swarm,              // Insects (firefly, butterfly)
    Colony,             // Nesting colony (black skimmer)
    Communal,           // Share sleeping quarters (flying squirrel)
    BaskingGroup,       // Gather to sun (painted turtle)
    PairOrFlock,        // Seasonal variation (wood duck)
    BreedingAggregation, // Breeding season only (wood frog)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupingDef {
    pub group_type: GroupType,
    pub size_min: u8,
    pub size_max: u8,
    pub flees_together: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaunaGroupId(pub u64);

#[derive(Debug)]
pub struct FaunaGroup {
    pub id: FaunaGroupId,
    pub species: DocileSpecies,
    pub group_type: GroupType,
    pub members: Vec<FaunaId>,
    pub leader: Option<FaunaId>,
    pub center_position: Vec3,
    pub flee_direction: Option<Vec3>,
    pub group_awareness: f32,
}

impl FaunaGroup {
    /// Update group coordination
    pub fn update(&mut self, manager: &DocileFaunaManager, dt: f32) {
        // Calculate group center
        let positions: Vec<Vec3> = self.members.iter()
            .filter_map(|id| manager.get(*id))
            .map(|f| f.position)
            .collect();

        if !positions.is_empty() {
            self.center_position = positions.iter().sum::<Vec3>() / positions.len() as f32;
        }

        // Aggregate awareness (highest member awareness)
        self.group_awareness = self.members.iter()
            .filter_map(|id| manager.get(*id))
            .map(|f| f.awareness)
            .fold(0.0, f32::max);

        // Elect leader (for herds/flocks - typically oldest or most aware)
        if matches!(self.group_type, GroupType::Herd | GroupType::Flock) {
            self.leader = self.members.iter()
                .filter_map(|id| manager.get(*id).map(|f| (*id, f.awareness)))
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(id, _)| id);
        }
    }

    /// Propagate warning to all members
    pub fn warn_group(&mut self, threat_position: Vec3, manager: &mut DocileFaunaManager) {
        self.flee_direction = Some(
            (self.center_position - threat_position).normalize()
        );

        for member_id in &self.members {
            if let Some(fauna) = manager.get_mut(*member_id) {
                fauna.awareness = 1.0;
                fauna.last_threat_position = Some(threat_position);
            }
        }
    }

    /// Get formation offset for group member
    pub fn get_formation_position(&self, member_idx: usize) -> Vec3 {
        match self.group_type {
            GroupType::Herd | GroupType::Flock => {
                // Loose cluster around center
                let angle = (member_idx as f32 / self.members.len() as f32) * std::f32::consts::TAU;
                let radius = 3.0 + (member_idx % 3) as f32 * 2.0;
                self.center_position + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius)
            },
            GroupType::BaskingGroup => {
                // Linear on log
                self.center_position + Vec3::new(member_idx as f32 * 0.5, 0.0, 0.0)
            },
            GroupType::Swarm => {
                // 3D cloud
                let offset = Vec3::new(
                    (member_idx as f32 * 1.618).sin() * 5.0,
                    (member_idx as f32 * 2.718).cos() * 3.0,
                    (member_idx as f32 * 3.14).sin() * 5.0,
                );
                self.center_position + offset
            },
            _ => self.center_position,
        }
    }
}
```

---

## Seasonal Behavior

### Season System

```rust
// seasonal.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeasonState {
    // General
    Normal,
    Active,
    LessActive,
    Sluggish,

    // Breeding
    BreedingActive,
    MatingSeason,
    MatingDisplays,
    SingingMating,
    ExplosiveBreeding,
    Breeding,

    // Nesting
    Nesting,
    NestBuilding,
    NestingInTrees,
    NestingColonies,
    RaisingYoung,
    TeachingYoung,

    // Young
    NewbornsPresent,

    // Food
    Foraging,
    Fattening,
    NutGathering,
    FoodCaching,
    FeedingFrenzy,

    // Social
    Flocking,
    LargeFlocks,
    GroupedForaging,
    Grouping,
    CommunalNesting,

    // Hibernation/Dormancy
    Hibernating,
    HibernatingUnderwater,
    FrozenHibernation,
    PreparingHibernation,
    PreparingFreeze,
    Denning,
    LodgeBound,
    Torpor,

    // Migration
    Arriving,
    ArrivingMigration,
    Migrating,
    SouthernMovement,
    CoastalMovement,
    Absent,

    // Activity changes
    PeakActivity,
    FishingActive,
    ActiveBasking,
    ActiveUnderIce,
    Emerging,
    Declining,
    ForestDwelling,

    // Building
    DamRepair,

    // Chorus/Display
    LoudChorusing,
    Territorial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalBehavior {
    pub spring: SeasonState,
    pub summer: SeasonState,
    pub fall: SeasonState,
    pub winter: SeasonState,
}

impl SeasonalBehavior {
    pub fn get_state(&self, season: Season) -> SeasonState {
        match season {
            Season::Spring => self.spring,
            Season::Summer => self.summer,
            Season::Fall => self.fall,
            Season::Winter => self.winter,
        }
    }
}

/// Apply seasonal modifiers to fauna behavior
pub fn apply_seasonal_effects(
    fauna: &mut DocileFauna,
    species: &DocileSpeciesDef,
    season: Season,
) {
    let state = species.seasonal_behavior.get_state(season);

    match state {
        SeasonState::Absent => {
            // Don't spawn this species
            fauna.should_despawn = true;
        },
        SeasonState::Hibernating | SeasonState::HibernatingUnderwater |
        SeasonState::FrozenHibernation | SeasonState::Denning |
        SeasonState::LodgeBound => {
            fauna.behavior_state = FaunaBehaviorState::Resting(RestSubstate::Hibernating);
            fauna.hidden = true;
        },
        SeasonState::LessActive | SeasonState::Sluggish => {
            fauna.activity_modifier = 0.5;
        },
        SeasonState::PeakActivity | SeasonState::FeedingFrenzy => {
            fauna.activity_modifier = 1.5;
            fauna.spawn_rate_modifier = 1.3;
        },
        SeasonState::Flocking | SeasonState::LargeFlocks |
        SeasonState::GroupedForaging | SeasonState::CommunalNesting => {
            fauna.group_size_modifier = 2.0;
        },
        SeasonState::MatingSeason | SeasonState::MatingDisplays |
        SeasonState::SingingMating | SeasonState::BreedingActive => {
            fauna.vocalization_rate = 2.0;
        },
        _ => {}
    }
}
```

---

## Spawning System

### Habitat-Based Spawning

```rust
// spawner.rs

/// Habitats for docile fauna spawning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Habitat {
    // Forests
    Forests,
    ForestEdges,
    DeciduousForests,
    MixedForests,
    OakGroves,
    PineForests,
    Woodlands,

    // Open areas
    Meadows,
    Brushlands,
    Gardens,
    MilkweedPatches,

    // Water
    Rivers,
    Ponds,
    Streams,
    Lakes,
    Marshes,
    Swamps,
    WoodedStreams,
    SlowRivers,
    NearWater,
    WetlandEdges,
    Wetlands,
    ForestPonds,
    VernalPools,

    // Coastal
    Beaches,
    CoastalWaters,
    Estuaries,

    // Shelter
    HollowTrees,
    TreeCavities,
}

/// Spawn conditions from environment
#[derive(Debug, Clone)]
pub struct SpawnConditions {
    pub weather: Weather,
    pub temperature: Temperature,
    pub moon_phase: MoonPhase,
    pub time_of_day: TimeOfDay,
    pub season: Season,
}

impl SpawnConditions {
    /// Get spawn rate modifier based on conditions
    pub fn get_modifier(&self) -> f32 {
        let weather_mod = match self.weather {
            Weather::Clear => 1.0,
            Weather::Rain => 0.7,
            Weather::Storm => 0.3,
            Weather::Snow => 0.2,
            _ => 1.0,
        };

        let temp_mod = match self.temperature {
            Temperature::Hot => 0.8,
            Temperature::Warm => 1.0,
            Temperature::Cool => 0.9,
            Temperature::Cold => 0.4,
        };

        let moon_mod = match self.moon_phase {
            MoonPhase::Full => 1.2,
            MoonPhase::Waning => 1.0,
            MoonPhase::New => 0.8,
            MoonPhase::Waxing => 1.0,
        };

        weather_mod * temp_mod * moon_mod
    }
}

pub struct DocileFaunaSpawner {
    spawn_density_base: f32,
    max_fauna_per_chunk: u8,
    min_player_distance: f32,
    max_active_fauna: usize,
}

impl DocileFaunaSpawner {
    pub fn on_chunk_loaded(
        &self,
        chunk: ChunkCoord,
        chunk_data: &LoadedChunk,
        manager: &mut DocileFaunaManager,
        player_pos: Vec3,
        conditions: &SpawnConditions,
        seed: u32,
    ) {
        if manager.fauna_count() >= self.max_active_fauna {
            return;
        }

        let habitats = determine_chunk_habitats(chunk, chunk_data, seed);
        let spawn_modifier = conditions.get_modifier();

        // Get eligible species
        let eligible: Vec<&DocileSpeciesDef> = ALL_SPECIES.iter()
            .filter(|s| {
                // Habitat match
                s.habitats.iter().any(|h| habitats.contains(h)) &&
                // Time of day match
                s.active_times.contains(&conditions.time_of_day) &&
                // Not absent this season
                s.seasonal_behavior.get_state(conditions.season) != SeasonState::Absent
            })
            .collect();

        // Spawn fauna
        let spawn_positions = generate_spawn_positions(chunk, seed, 12);

        for pos in spawn_positions {
            if pos.distance(player_pos) < self.min_player_distance {
                continue;
            }

            // Weighted species selection
            if let Some(species_def) = select_species(&eligible, pos, seed, spawn_modifier) {
                let group_size = rand_range(
                    species_def.grouping.size_min,
                    species_def.grouping.size_max,
                    seed,
                );

                let group_id = if group_size > 1 {
                    Some(manager.create_group(species_def.id, species_def.grouping.group_type))
                } else {
                    None
                };

                for i in 0..group_size {
                    let offset = Vec3::new(
                        seeded_float(seed + i as u32) * 5.0 - 2.5,
                        0.0,
                        seeded_float(seed + i as u32 + 100) * 5.0 - 2.5,
                    );
                    manager.spawn(species_def.id, pos + offset, group_id);
                }
            }
        }
    }
}
```

---

## Harvesting System

### Harvest Definitions

```rust
// harvest.rs

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HarvestDef {
    // Common
    pub meat: u8,
    pub hide: u8,
    pub bones: u8,

    // Mammals
    pub pelt: Option<u8>,
    pub fat: Option<u8>,
    pub tail: Option<u8>,
    pub antlers: Option<u8>,
    pub castoreum: Option<u8>,
    pub teeth: Option<u8>,

    // Birds
    pub feathers: Option<u8>,

    // Reptiles
    pub shell: Option<u8>,
    pub skin: Option<u8>,

    // Amphibians
    pub frog_legs: Option<u8>,

    // Insects
    pub butterfly_wing: Option<u8>,
    pub glowworm: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarvestTool {
    None,
    Knife,
    SkinningKnife,
}

/// Tool requirements for harvest types
pub fn get_tool_requirement(item_type: &str) -> HarvestTool {
    match item_type {
        "meat" | "hide" | "pelt" => HarvestTool::Knife,
        "feathers" | "shell" | "butterfly_wing" | "glowworm" => HarvestTool::None,
        _ => HarvestTool::Knife,
    }
}

/// Skill modifiers for harvesting
#[derive(Debug, Clone)]
pub struct HarvestSkillModifiers {
    pub yield_bonus: f32,
    pub quality_bonus: f32,
}

impl HarvestSkillModifiers {
    pub fn from_hunting_skill(level: u32) -> Self {
        Self {
            yield_bonus: 0.25 * level as f32,
            quality_bonus: 0.5 * level as f32,
        }
    }

    pub fn from_trapping_skill(level: u32) -> Self {
        Self {
            yield_bonus: 0.15 * level as f32,
            quality_bonus: 0.3 * level as f32,
        }
    }
}

/// Freshness decay for harvested materials
#[derive(Debug, Clone, Copy)]
pub enum Freshness {
    Immediate,  // 1.0x quality
    OneHour,    // 0.8x quality
    ThreeHours, // 0.6x quality
    OneDay,     // 0.3x quality
    Spoiled,    // Unusable
}

impl Freshness {
    pub fn quality_multiplier(&self) -> f32 {
        match self {
            Self::Immediate => 1.0,
            Self::OneHour => 0.8,
            Self::ThreeHours => 0.6,
            Self::OneDay => 0.3,
            Self::Spoiled => 0.0,
        }
    }
}

/// Process fauna harvest
pub fn harvest_fauna(
    fauna: &DocileFauna,
    species_def: &DocileSpeciesDef,
    player_tool: HarvestTool,
    skill_modifiers: &HarvestSkillModifiers,
    time_since_death: f32,
) -> Vec<HarvestItem> {
    let mut items = Vec::new();
    let harvest = &species_def.harvest;

    let freshness = match time_since_death {
        t if t < 60.0 => Freshness::Immediate,
        t if t < 3600.0 => Freshness::OneHour,
        t if t < 10800.0 => Freshness::ThreeHours,
        t if t < 86400.0 => Freshness::OneDay,
        _ => Freshness::Spoiled,
    };

    let quality = freshness.quality_multiplier();
    let yield_mult = 1.0 + skill_modifiers.yield_bonus;

    // Meat
    if harvest.meat > 0 && player_tool >= HarvestTool::Knife {
        let amount = (harvest.meat as f32 * yield_mult).round() as u8;
        items.push(HarvestItem::Meat { amount, quality });
    }

    // Hide/Pelt
    if harvest.hide > 0 && player_tool >= HarvestTool::Knife {
        items.push(HarvestItem::Hide { quality });
    }
    if let Some(pelt) = harvest.pelt {
        if player_tool >= HarvestTool::Knife {
            items.push(HarvestItem::Pelt { quality: quality + skill_modifiers.quality_bonus });
        }
    }

    // Feathers (no tool needed)
    if let Some(feathers) = harvest.feathers {
        let amount = (feathers as f32 * yield_mult).round() as u8;
        items.push(HarvestItem::Feathers { amount });
    }

    // Bones
    if harvest.bones > 0 {
        items.push(HarvestItem::Bones { amount: harvest.bones });
    }

    // Special items
    if let Some(antlers) = harvest.antlers {
        items.push(HarvestItem::Antlers);
    }
    if let Some(shell) = harvest.shell {
        items.push(HarvestItem::Shell);
    }
    if let Some(frog_legs) = harvest.frog_legs {
        items.push(HarvestItem::FrogLegs { amount: frog_legs });
    }
    if let Some(_) = harvest.glowworm {
        items.push(HarvestItem::Glowworm);
    }

    items
}

#[derive(Debug, Clone)]
pub enum HarvestItem {
    Meat { amount: u8, quality: f32 },
    Hide { quality: f32 },
    Pelt { quality: f32 },
    Feathers { amount: u8 },
    Bones { amount: u8 },
    Antlers,
    Shell,
    FrogLegs { amount: u8 },
    Glowworm,
    ButterflyWing { amount: u8 },
    Castoreum,
    Teeth { amount: u8 },
    Fat { amount: u8 },
    Tail,
}
```

---

## Interaction System

### Player Interactions

```rust
// interact.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionDef {
    pub feedable: bool,
    pub tameable: bool,
    pub petable: bool,
    pub rideable: bool,
    pub food_preference: Vec<&'static str>,
}

/// Effects of player interactions on fauna and area
#[derive(Debug, Clone)]
pub struct InteractionEffects {
    // Feeding effects
    pub trust_increase: f32,
    pub spawn_rate_increase: f32,
    pub taming_progress: f32,

    // Hunting effects
    pub area_fear_increase: f32,
    pub spawn_rate_decrease: f32,
    pub flee_distance_increase: f32,

    // Observation (no effect)
    pub knowledge_gain: bool,
    pub map_marking: bool,
}

impl Default for InteractionEffects {
    fn default() -> Self {
        Self {
            trust_increase: 5.0,
            spawn_rate_increase: 0.1,
            taming_progress: 10.0,
            area_fear_increase: 20.0,
            spawn_rate_decrease: 0.3,
            flee_distance_increase: 10.0,
            knowledge_gain: true,
            map_marking: true,
        }
    }
}

/// Area memory for fauna reactions
pub struct AreaMemory {
    pub center: Vec3,
    pub radius: f32,
    pub fear_level: f32,       // 0-100, increases from hunting
    pub trust_level: f32,      // 0-100, increases from feeding
    pub last_hunting: Option<f64>,
    pub last_feeding: Option<f64>,
}

impl AreaMemory {
    pub fn apply_hunt(&mut self, game_time: f64) {
        self.fear_level = (self.fear_level + 20.0).min(100.0);
        self.last_hunting = Some(game_time);
    }

    pub fn apply_feeding(&mut self, game_time: f64) {
        self.trust_level = (self.trust_level + 5.0).min(100.0);
        self.last_feeding = Some(game_time);
    }

    pub fn decay(&mut self, dt: f32) {
        // Fear decays over time
        self.fear_level = (self.fear_level - dt * 0.01).max(0.0);
        // Trust decays slower
        self.trust_level = (self.trust_level - dt * 0.005).max(0.0);
    }

    pub fn get_spawn_modifier(&self) -> f32 {
        1.0 - (self.fear_level * 0.003) + (self.trust_level * 0.001)
    }

    pub fn get_flee_distance_modifier(&self) -> f32 {
        1.0 + (self.fear_level * 0.01)
    }
}

/// Taming system for tameable species
#[derive(Debug, Clone)]
pub struct TamingProgress {
    pub fauna_id: FaunaId,
    pub species: DocileSpecies,
    pub progress: f32,          // 0-100
    pub trust: f32,             // 0-100
    pub feeding_count: u32,
    pub time_spent_near: f32,
}

impl TamingProgress {
    pub fn feed(&mut self, food_item: &str, species_def: &DocileSpeciesDef) -> bool {
        if species_def.interactions.food_preference.contains(&food_item) {
            self.progress += 10.0;
            self.trust += 5.0;
            self.feeding_count += 1;
            true
        } else {
            false
        }
    }

    pub fn is_tamed(&self) -> bool {
        self.progress >= 100.0 && self.trust >= 50.0
    }
}

/// Tameable species
pub fn can_tame(species: DocileSpecies) -> bool {
    matches!(species,
        DocileSpecies::EasternCottontail |
        DocileSpecies::EasternBoxTurtle
    )
}
```

---

## Environmental Impact

### Unique Behaviors

```rust
// unique.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UniqueBehavior {
    EnvironmentalEngineer {
        creates_dams: bool,
        modifies_waterways: bool,
        creates_wetlands: bool,
    },
    Mischievous {
        washes_food: bool,
        steals_shiny_objects: bool,
        raids_camps: bool,
    },
    Opossum {
        immune_to_rabies: bool,
        eats_venomous_snakes: bool,
        carriable_babies: bool,
    },
    Squirrel {
        caches_food: bool,
        deceptive_burying: bool,
        acrobatic_jumps: bool,
    },
    FlyingSquirrel {
        glides: bool,
        communal_nesting: bool,
        uv_fluorescent: bool,
    },
    Otter {
        makes_slides: bool,
        plays_with_objects: bool,
        social_grooming: bool,
    },
    BoxTurtle {
        longevity_years: u32,
        homing_instinct: bool,
        closes_shell_completely: bool,
    },
    PaintedTurtle {
        stacks_on_logs: bool,
        sun_bathes: bool,
        hibernates_in_mud: bool,
    },
    Cardinal {
        year_round_resident: bool,
        males_bright_red: bool,
        morning_chorus: bool,
    },
    BlueJay {
        mimics_hawks: bool,
        caches_acorns: bool,
        mobs_birds_of_prey: bool,
    },
    WoodDuck {
        nests_in_tree_cavities: bool,
        ducklings_jump_from_nest: bool,
        colorful_plumage: bool,
    },
    Hummingbird {
        pollinates: bool,
        torpor: bool,
        aggressive_territorial: bool,
    },
    Skimmer {
        skims_fishing: bool,
        unique_bill_shape: bool,
        night_fishing: bool,
    },
    Bullfrog {
        territorial_calls: bool,
        eats_anything_smaller: bool,
        tadpole_stage: bool,
    },
    WoodFrog {
        freeze_tolerant: bool,
        explosive_breeder: bool,
        terrestrial_adult: bool,
    },
    Butterfly {
        migration: bool,
        pollinates: bool,
        toxic_to_eat: bool,
    },
    Firefly {
        bioluminescence: bool,
        synchronous_flashing: bool,
        mating_signals: bool,
        light_color: &'static str,
        light_pattern: LightPattern,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LightPattern {
    Intermittent,
    Continuous,
    Synchronized,
}

/// Beaver dam creation
pub struct BeaverDam {
    pub position: Vec3,
    pub health: f32,
    pub water_level_raise: f32,
    pub builder_id: FaunaId,
}

impl BeaverDam {
    /// Affects water flow in chunk
    pub fn apply_to_chunk(&self, chunk: &mut ChunkData) {
        // Raise water level behind dam
        // Create pond area
        // Modify terrain wetness
    }
}

/// Raccoon camp raiding
pub fn check_camp_raid(raccoon: &DocileFauna, camp_position: Vec3, inventory: &mut Inventory) -> bool {
    let dist = raccoon.position.distance(camp_position);
    if dist < 15.0 && raccoon.behavior_state == FaunaBehaviorState::Foraging(ForagingSubstate::Searching) {
        // Chance to steal shiny items or food
        if let Some(item) = inventory.get_stealable_item() {
            // Raccoon takes item
            return true;
        }
    }
    false
}
```

---

## Rendering Integration

### Fauna Pipeline

```rust
// In crates/croatoan_render/src/fauna_pipeline.rs

pub struct FaunaPipeline {
    pipeline: wgpu::RenderPipeline,
    mesh_cache: HashMap<DocileSpecies, FaunaMesh>,
    instance_buffer: wgpu::Buffer,
    firefly_light_buffer: wgpu::Buffer,  // For bioluminescence
}

pub struct FaunaMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    skeleton: Option<Skeleton>,
    scale: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FaunaInstance {
    pub model_matrix: [[f32; 4]; 4],
    pub animation_data: [f32; 4],  // time, state, blend, _padding
    pub color_tint: [f32; 4],      // For seasonal variations, gender dimorphism
}

/// Firefly light for point light rendering
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FireflyLight {
    pub position: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
    pub blink_phase: f32,
}

impl FaunaPipeline {
    pub fn update_firefly_lights(
        &mut self,
        fireflies: &[&DocileFauna],
        game_time: f64,
        queue: &wgpu::Queue,
    ) {
        let lights: Vec<FireflyLight> = fireflies.iter()
            .map(|f| {
                // Blink pattern
                let phase = (game_time + f.id.0 as f64 * 0.1) % 2.0;
                let intensity = if phase < 0.3 { 1.0 } else { 0.0 };

                FireflyLight {
                    position: f.position.to_array(),
                    intensity,
                    color: [0.9, 1.0, 0.4],  // Yellow-green
                    blink_phase: phase as f32,
                }
            })
            .collect();

        queue.write_buffer(&self.firefly_light_buffer, 0, bytemuck::cast_slice(&lights));
    }
}
```

---

## Audio Integration

### Fauna Sounds

```rust
// audio.rs additions

pub enum FaunaSound {
    // Deer
    DeerSnort,
    DeerBleat,
    DeerStomp,

    // Rabbit
    RabbitThump,
    RabbitSqueal,

    // Turkey
    TurkeyGobble,
    TurkeyPurr,
    TurkeyCluck,

    // Beaver
    BeaverSplash,
    BeaverWhine,
    BeaverTailSlap,

    // Raccoon
    RaccoonChitter,
    RaccoonPurr,
    RaccoonGrowl,

    // Opossum
    OpossumHiss,
    OpossumClick,

    // Squirrel
    SquirrelChatter,
    SquirrelBark,

    // Birds
    CardinalSong,
    CardinalChip,
    BlueJayCall,
    BlueJayAlarm,
    WoodDuckWhistle,
    HummingbirdBuzz,
    SkimmerBark,

    // Frogs
    BullfrogCroak,
    WoodFrogQuack,

    // Generic
    Splash,
    WingFlutter,
    Footsteps,
}

/// Get ambient sounds based on fauna in area
pub fn get_fauna_ambient_sounds(
    fauna_list: &[&DocileFauna],
    time_of_day: TimeOfDay,
    season: Season,
) -> Vec<(Vec3, FaunaSound, f32)> {
    let mut sounds = Vec::new();

    for fauna in fauna_list {
        // Vocalization based on season and time
        match fauna.species {
            DocileSpecies::NorthernCardinal if time_of_day == TimeOfDay::Dawn => {
                if rand::random::<f32>() < 0.1 {
                    sounds.push((fauna.position, FaunaSound::CardinalSong, 0.6));
                }
            },
            DocileSpecies::AmericanBullfrog if time_of_day == TimeOfDay::Night
                && season == Season::Spring => {
                sounds.push((fauna.position, FaunaSound::BullfrogCroak, 0.8));
            },
            DocileSpecies::WildTurkey if season == Season::Spring => {
                if rand::random::<f32>() < 0.05 {
                    sounds.push((fauna.position, FaunaSound::TurkeyGobble, 0.7));
                }
            },
            _ => {}
        }
    }

    sounds
}
```

---

## Implementation Phases

### Phase 1: Foundation ⏳
- [ ] Create `roanoke_game/src/fauna/` module structure
- [ ] Implement `DocileSpecies`, `FaunaStats`, core type definitions
- [ ] Implement `DocileFaunaManager` with basic CRUD
- [ ] Add `FaunaId` generation and tracking
- [ ] Integrate with spatial hash from animal system

### Phase 2: Spawning ⏳
- [ ] Implement `DocileFaunaSpawner` with habitat detection
- [ ] Add time-of-day and seasonal filtering
- [ ] Implement group spawning (herds, flocks, swarms)
- [ ] Integrate with chunk loading

### Phase 3: Behavior ⏳
- [ ] Implement `FaunaBehaviorState` enum and transitions
- [ ] Add idle, foraging, alert states
- [ ] Implement flee behavior with species-specific responses
- [ ] Add hiding behavior

### Phase 4: Groups & Social ⏳
- [ ] Implement `FaunaGroup` system
- [ ] Add herd/flock coordination
- [ ] Implement warning propagation
- [ ] Add leader following behavior

### Phase 5: Harvesting ⏳
- [ ] Implement harvest system with tool requirements
- [ ] Add skill modifiers from hunting tree
- [ ] Implement freshness decay
- [ ] Create loot tables per species

### Phase 6: Interactions ⏳
- [ ] Implement feeding system
- [ ] Add taming for eligible species
- [ ] Implement area memory (fear/trust)
- [ ] Add observation/knowledge system

### Phase 7: Seasonal ⏳
- [ ] Implement season state system
- [ ] Add migration (butterflies, hummingbirds absent in winter)
- [ ] Implement hibernation states
- [ ] Add breeding behaviors

### Phase 8: Rendering ⏳
- [ ] Create `FaunaPipeline` in `croatoan_render`
- [ ] Implement basic mesh rendering
- [ ] Add firefly bioluminescence
- [ ] Implement animation state machine

### Phase 9: Polish ⏳
- [ ] Audio integration (calls, ambient sounds)
- [ ] Persistence (save/load)
- [ ] Environmental impact (beaver dams)
- [ ] Unique behaviors (raccoon raids, opossum playing dead)

---

## Species Quick Reference

| Species | Category | HP | Speed | Behavior | Group | Active | Harvest |
|---------|----------|----|----|----------|-------|--------|---------|
| White-tailed Deer | Large Mammal | 60 | 45 | Skittish | Herd 3-8 | Dawn/Dusk | Meat 8, Hide, Antlers |
| Eastern Cottontail | Small Mammal | 15 | 35 | Timid | Solo 1-2 | Dawn/Dusk/Night | Meat 1, Hide |
| Wild Turkey | Large Bird | 40 | 25 | Cautious | Flock 5-15 | Day | Meat 4, Feathers 20 |
| American Beaver | Aquatic | 50 | 15/30 | Industrious | Family 2-6 | Night | Meat 3, Pelt, Castoreum |
| Common Raccoon | Small Mammal | 30 | 20 | Curious | Solo 1-3 | Night | Meat 2, Pelt |
| Virginia Opossum | Small Mammal | 25 | 15 | Passive | Solo 1 | Night | Meat 2, Hide |
| Gray Squirrel | Small Mammal | 12 | 25 | Energetic | Loose 2-5 | Day | Meat 1, Hide, Tail |
| Flying Squirrel | Small Mammal | 10 | 20/35g | Nocturnal | Communal 2-8 | Night | Meat 1, Pelt |
| River Otter | Aquatic | 45 | 18/40 | Playful | Family 2-4 | Dawn/Dusk | Meat 3, Pelt |
| Box Turtle | Reptile | 20 | 3 | Docile | Solo 1 | Day | Shell, Meat 1 |
| Painted Turtle | Reptile | 18 | 4/15 | Basking | Basking 3-12 | Day | Shell, Meat 1 |
| Cardinal | Small Bird | 8 | 30 | Territorial | Pair 2 | Day | Feathers 5 |
| Blue Jay | Small Bird | 10 | 32 | Bold | Family 3-7 | Day | Feathers 6 |
| Wood Duck | Waterfowl | 25 | 20/25 | Shy | Flock 2-12 | Dawn/Dusk | Meat 2, Feathers 15 |
| Hummingbird | Tiny Bird | 3 | 50 | Hyperactive | Solo 1 | Day | Feathers 2 |
| Black Skimmer | Shore Bird | 20 | 35 | Active | Colony 5-20 | Dawn/Dusk/Night | Feathers 8 |
| Bullfrog | Amphibian | 15 | 10/20 | Stationary | Loose 3-10 | Night | Frog Legs 2 |
| Wood Frog | Amphibian | 8 | 8 | Secretive | Breed 5-30 | Night/Rain | Frog Legs 1 |
| Monarch Butterfly | Insect | 1 | 15 | Peaceful | Swarm 5-50 | Day | Wings 2 |
| Firefly | Insect | 1 | 8 | Ambient | Swarm 10-100 | Night | Glowworm 1 |

---

## Integration with Other Systems

### Predator-Prey Dynamics

Docile fauna interacts with hostile fauna:
- Wolves, cougars, bobcats hunt deer, rabbits, squirrels
- Snakes hunt frogs, small birds
- Alligators hunt beavers, otters, ducks
- Blue jays mob and warn about predators

### Hunting Skill Tree Integration

See `HUNTING_SKILL_TREE_SPEC.md` for:
- Deer Stalker skill (reduced detection)
- Prey Instinct skill (patrol paths visible)
- Trap system for capturing prey
- Skinning yield bonuses

### Archaeology Integration

See `ARCHAEOLOGY_SKILL_TREE_SPEC.md` for:
- Ancestral Horn attracts prey
- Bone-based items affect fauna behavior

---

## Save/Load Data

```rust
#[derive(Serialize, Deserialize)]
pub struct DocileFaunaSaveData {
    pub active_fauna: Vec<SerializedFauna>,
    pub groups: Vec<SerializedGroup>,
    pub area_memories: Vec<AreaMemory>,
    pub taming_progress: Vec<TamingProgress>,
    pub beaver_dams: Vec<BeaverDam>,
    pub statistics: FaunaStatistics,
}

#[derive(Serialize, Deserialize)]
pub struct FaunaStatistics {
    pub total_observed: HashMap<DocileSpecies, u32>,
    pub total_harvested: HashMap<DocileSpecies, u32>,
    pub total_fed: HashMap<DocileSpecies, u32>,
    pub tamed_creatures: Vec<(DocileSpecies, String)>,  // (species, name)
}
```
