# Flora & Plant System Specification

## Roanoke Engine - Botanical Framework

This document specifies the comprehensive system for plants, herbs, fungi, and cultivated crops in the Roanoke wilderness. Flora serves as resources for survival, medicine, crafting, and spiritual practice.

---

## Table of Contents

1. [Overview](#overview)
2. [Core Data Structures](#core-data-structures)
3. [Species Definitions](#species-definitions)
4. [Growth & Lifecycle](#growth--lifecycle)
5. [Harvesting System](#harvesting-system)
6. [Medicinal & Alchemical Uses](#medicinal--alchemical-uses)
7. [Poison & Danger System](#poison--danger-system)
8. [Cultivation & Farming](#cultivation--farming)
9. [Foraging Skill Tree](#foraging-skill-tree)
10. [Seasonal Behavior](#seasonal-behavior)
11. [Environmental Integration](#environmental-integration)
12. [Native Plant Knowledge](#native-plant-knowledge)
13. [Rendering & Visuals](#rendering--visuals)

---

## Overview

### Design Philosophy

The forests and fields of 16th-century Virginia held countless plants unknown to Europeans. Native peoples had developed sophisticated botanical knowledge over millennia. This system captures the rich plant life of the region while providing meaningful gameplay through foraging, medicine, and cultivation.

### Plant Categories

| Category | Count | Gameplay Role |
|----------|-------|---------------|
| Trees | 15 | Resources, landmarks, shelter |
| Shrubs & Bushes | 12 | Berries, materials, cover |
| Herbs & Medicinals | 25 | Healing, buffs, crafting |
| Flowers | 15 | Beauty, dyes, medicine |
| Fungi | 12 | Food, poison, special effects |
| Aquatic Plants | 8 | Swamp resources, fish habitat |
| Crops | 12 | Food production, trade |

### Key Features

- **Seasonal Growth**: Plants grow, bloom, and die with the seasons
- **Habitat-Specific**: Each species has preferred biomes
- **Procedural Placement**: Plants spawn based on terrain and conditions
- **Harvestable Resources**: Multiple parts (leaves, roots, bark, fruit)
- **Hidden Properties**: Medicinal/poisonous effects discovered through use
- **Cultivation System**: Grow crops and transplant wild plants

---

## Core Data Structures

### Flora Species Definition

```rust
// flora/species.rs

use serde::{Deserialize, Serialize};
use glam::Vec3;

/// All flora species in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FloraSpecies {
    // === TREES ===
    WhiteOak,
    RedOak,
    LoblollyPine,
    LongleafPine,
    BaldCypress,
    EasternRedCedar,
    AmericanBeech,
    TulipPoplar,
    SweetGum,
    SugarMaple,
    AmericanSycamore,
    BlackWalnut,
    SouthernMagnolia,
    FloweringDogwood,
    EasternHemlock,

    // === SHRUBS ===
    Blueberry,
    Blackberry,
    WildRaspberry,
    AmericanHolly,
    Rhododendron,
    MountainLaurel,
    Bayberry,
    Elderberry,
    SpiceBush,
    WinterBerry,
    ButtonBush,
    WaxMyrtle,

    // === HERBS & MEDICINALS ===
    AmericanGinseng,
    Goldenseal,
    Echinacea,
    BlackCohosh,
    BloodRoot,
    WildGinger,
    Yarrow,
    Plantain,
    JewelWeed,
    WildMint,
    Sassafras,
    WitchHazel,
    SkullCap,
    Lobelia,
    MayApple,
    Pokeweed,
    BoneSit,
    Comfrey,
    Mullein,
    StingingNettle,
    WildLettuce,
    Valerian,
    Catnip,
    Chamomile,
    Feverfew,

    // === FLOWERS ===
    Trillium,
    BlackEyedSusan,
    CardinalFlower,
    WildColumbine,
    BlueLobelia,
    WildRose,
    PassionFlower,
    MorningGlory,
    WildViolet,
    JackInThePulpit,
    DutchmansBreech,
    BloodRoot,
    FirePink,
    WildGeranium,
    ButterflWeed,

    // === FUNGI ===
    Chanterelle,
    Morel,
    ChickenOfTheWoods,
    HenOfTheWoods,
    LionsMain,
    Puffball,
    TurkeyTail,
    Reishi,
    DestroyingAngel,     // DEADLY
    DeathCap,            // DEADLY
    FalseMorel,          // TOXIC
    JackOLantern,        // TOXIC

    // === AQUATIC ===
    Cattail,
    WaterLily,
    Duckweed,
    Arrowhead,
    Pickerelweed,
    WildRice,
    Lotus,
    WaterHyacinth,

    // === CROPS ===
    Corn,
    Squash,
    Beans,
    Tobacco,
    Sunflower,
    Pumpkin,
    Watermelon,
    Cotton,
    Indigo,
    SweetPotato,
    Gourd,
    Jerusalem Artichoke,
}

/// Plant category for shared behaviors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloraCategory {
    Tree,
    Shrub,
    Herb,
    Flower,
    Fungus,
    Aquatic,
    Crop,
}

/// Complete species definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloraSpeciesDef {
    pub id: FloraSpecies,
    pub common_name: &'static str,
    pub scientific_name: &'static str,
    pub native_names: Vec<(&'static str, &'static str)>,  // (tribe, name)
    pub category: FloraCategory,

    // Physical properties
    pub size: FloraSize,
    pub growth_rate: f32,           // Days to mature
    pub lifespan: Option<f32>,      // Days until death (None = perennial)
    pub spread_rate: f32,           // How aggressively it spreads

    // Environmental needs
    pub habitats: Vec<Habitat>,
    pub soil_preference: SoilType,
    pub moisture_needs: MoistureLevel,
    pub light_needs: LightLevel,
    pub temperature_range: (f32, f32),

    // Seasonal behavior
    pub growth_seasons: Vec<Season>,
    pub bloom_season: Option<Season>,
    pub fruit_season: Option<Season>,
    pub dormant_seasons: Vec<Season>,

    // Harvesting
    pub harvestable_parts: Vec<PlantPart>,
    pub harvest_yields: HarvestYields,
    pub regrows_after_harvest: bool,
    pub regrowth_time: Option<f32>,

    // Properties
    pub edibility: Edibility,
    pub medicinal_properties: Vec<MedicinalProperty>,
    pub crafting_uses: Vec<CraftingUse>,
    pub dangers: Vec<PlantDanger>,

    // Spawning
    pub spawn_rate: f32,
    pub cluster_size: (u8, u8),
    pub min_spacing: f32,

    // Visuals
    pub mesh_variant: &'static str,
    pub seasonal_colors: SeasonalColors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloraSize {
    Tiny,       // < 0.1m (mosses, small flowers)
    Small,      // 0.1 - 0.5m (herbs, mushrooms)
    Medium,     // 0.5 - 2m (shrubs, large flowers)
    Large,      // 2 - 10m (small trees)
    Huge,       // 10 - 30m (mature trees)
    Massive,    // > 30m (ancient trees)
}

impl FloraSize {
    pub fn height_range(&self) -> (f32, f32) {
        match self {
            Self::Tiny => (0.02, 0.1),
            Self::Small => (0.1, 0.5),
            Self::Medium => (0.5, 2.0),
            Self::Large => (2.0, 10.0),
            Self::Huge => (10.0, 30.0),
            Self::Massive => (30.0, 50.0),
        }
    }
}
```

### Plant Instance

```rust
// flora/entity.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FloraId(pub u64);

/// Runtime instance of a plant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloraInstance {
    pub id: FloraId,
    pub species: FloraSpecies,

    // Position and transform
    pub position: Vec3,
    pub rotation: f32,
    pub scale: f32,

    // Growth state
    pub growth_stage: GrowthStage,
    pub growth_progress: f32,    // 0.0 - 1.0 within stage
    pub age_days: f32,
    pub health: f32,             // 0.0 - 1.0

    // Seasonal state
    pub current_appearance: SeasonalAppearance,
    pub is_dormant: bool,
    pub has_bloomed: bool,
    pub has_fruited: bool,

    // Harvesting state
    pub last_harvested: Option<f32>,   // Game time
    pub times_harvested: u32,
    pub available_parts: Vec<PlantPart>,

    // Chunk association
    pub chunk: ChunkCoord,
    pub persistent: bool,    // Player-planted = persistent
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrowthStage {
    Seed,
    Seedling,
    Juvenile,
    Mature,
    Flowering,
    Fruiting,
    Senescent,
    Dead,
}

impl GrowthStage {
    pub fn visual_scale(&self) -> f32 {
        match self {
            Self::Seed => 0.0,
            Self::Seedling => 0.1,
            Self::Juvenile => 0.4,
            Self::Mature => 1.0,
            Self::Flowering => 1.0,
            Self::Fruiting => 1.0,
            Self::Senescent => 0.9,
            Self::Dead => 0.8,
        }
    }

    pub fn is_harvestable(&self) -> bool {
        matches!(self, Self::Mature | Self::Flowering | Self::Fruiting)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeasonalAppearance {
    Dormant,         // No leaves/flowers
    Budding,         // New growth emerging
    FullFoliage,     // Full leaves
    Flowering,       // Flowers visible
    Fruiting,        // Fruit/seeds visible
    FallColors,      // Autumn coloration
    Bare,            // Deciduous winter state
    Evergreen,       // Maintains foliage
}
```

---

## Species Definitions

### Trees (15 Species)

| Species | Height | Habitat | Special Properties |
|---------|--------|---------|-------------------|
| White Oak | 20-30m | Forests | Acorns edible, bark medicinal |
| Red Oak | 18-25m | Forests | Acorns (leach tannins first) |
| Loblolly Pine | 25-35m | Coastal | Resin, turpentine, lumber |
| Longleaf Pine | 30-40m | Coastal | Fire-resistant, naval stores |
| Bald Cypress | 20-35m | Swamps | Rot-resistant wood, "knees" |
| Eastern Red Cedar | 8-15m | Edges | Aromatic, moth-repellent |
| American Beech | 15-25m | Forests | Nuts edible, smooth bark |
| Tulip Poplar | 25-40m | Forests | Straight lumber, bee tree |
| Sweet Gum | 20-30m | Wet areas | Medicinal resin |
| Sugar Maple | 20-30m | Forests | Syrup, hardwood |
| American Sycamore | 20-35m | Rivers | Massive trunk, distinctive bark |
| Black Walnut | 18-25m | Forests | Valuable nuts, dye, lumber |
| Southern Magnolia | 15-25m | Coastal | Ornamental, medicinal |
| Flowering Dogwood | 5-10m | Understory | Medicinal bark |
| Eastern Hemlock | 20-30m | Ravines | Tea, Native medicine |

### Detailed Tree Definition

```rust
pub fn get_tree_def(species: FloraSpecies) -> FloraSpeciesDef {
    match species {
        FloraSpecies::WhiteOak => FloraSpeciesDef {
            id: FloraSpecies::WhiteOak,
            common_name: "White Oak",
            scientific_name: "Quercus alba",
            native_names: vec![
                ("Algonquin", "Mishkwaakwat"),
                ("Cherokee", "Tsugwagi"),
            ],
            category: FloraCategory::Tree,

            size: FloraSize::Huge,
            growth_rate: 3650.0,    // 10 years to mature (game-accelerated)
            lifespan: None,          // Lives indefinitely
            spread_rate: 0.1,

            habitats: vec![
                Habitat::DeciduousForest,
                Habitat::MixedForest,
                Habitat::ForestEdge,
            ],
            soil_preference: SoilType::Loamy,
            moisture_needs: MoistureLevel::Moderate,
            light_needs: LightLevel::FullSun,
            temperature_range: (-20.0, 35.0),

            growth_seasons: vec![Season::Spring, Season::Summer],
            bloom_season: Some(Season::Spring),
            fruit_season: Some(Season::Fall),
            dormant_seasons: vec![Season::Winter],

            harvestable_parts: vec![
                PlantPart::Bark,
                PlantPart::Wood,
                PlantPart::Leaves,
                PlantPart::Acorns,
            ],
            harvest_yields: HarvestYields {
                bark: Some((1, 3)),
                wood: Some((5, 15)),
                leaves: Some((3, 8)),
                acorns: Some((10, 30)),
                ..Default::default()
            },
            regrows_after_harvest: true,
            regrowth_time: Some(365.0),

            edibility: Edibility::EdibleParts,  // Acorns edible
            medicinal_properties: vec![
                MedicinalProperty {
                    effect: MedicinalEffect::WoundSalve,
                    potency: 0.6,
                    preparation: PreparationType::Decoction,
                    part: PlantPart::Bark,
                },
                MedicinalProperty {
                    effect: MedicinalEffect::Astringent,
                    potency: 0.7,
                    preparation: PreparationType::Poultice,
                    part: PlantPart::Bark,
                },
            ],
            crafting_uses: vec![
                CraftingUse::Lumber,
                CraftingUse::Fuel,
                CraftingUse::TanningAgent,
                CraftingUse::Dye(DyeColor::Brown),
            ],
            dangers: vec![],

            spawn_rate: 0.3,
            cluster_size: (1, 5),
            min_spacing: 8.0,

            mesh_variant: "oak_large",
            seasonal_colors: SeasonalColors {
                spring: Vec3::new(0.4, 0.7, 0.3),
                summer: Vec3::new(0.2, 0.5, 0.2),
                fall: Vec3::new(0.8, 0.3, 0.1),
                winter: Vec3::new(0.4, 0.35, 0.3),  // Bare
            },
        },
        // ... other trees
    }
}
```

### Herbs & Medicinals (25 Species)

#### Medicinal Herb Table

| Species | Habitat | Primary Effect | Danger | Rarity |
|---------|---------|----------------|--------|--------|
| American Ginseng | Shaded forest | Stamina restore | None | Rare |
| Goldenseal | Rich forest | Infection cure | None | Uncommon |
| Echinacea | Prairies | Immunity boost | None | Common |
| Black Cohosh | Forest | Pain relief | Overdose | Uncommon |
| Bloodroot | Forest | Antiseptic | Caustic | Rare |
| Wild Ginger | Forest floor | Digestive | None | Common |
| Yarrow | Fields | Stop bleeding | None | Common |
| Plantain | Disturbed soil | Wound healing | None | Common |
| Jewelweed | Wet areas | Poison ivy cure | None | Common |
| Wild Mint | Streams | Nausea relief | None | Common |
| Sassafras | Forest edge | Blood purifier | Liver (high dose) | Common |
| Witch Hazel | Wet forest | Astringent | None | Common |
| Skullcap | Wet meadow | Calming | None | Uncommon |
| Lobelia | Wet areas | Respiratory | Toxic (high) | Uncommon |
| May Apple | Forest | Purgative | Toxic (most parts) | Common |
| Pokeweed | Disturbed | Young shoots edible | Root deadly | Common |
| Boneset | Wet areas | Fever break | None | Common |
| Comfrey | Moist soil | Bone/wound heal | Internal danger | Uncommon |
| Mullein | Dry fields | Respiratory | None | Common |
| Stinging Nettle | Rich soil | Nutritious, diuretic | Sting | Common |
| Wild Lettuce | Fields | Sedative | Overdose | Uncommon |
| Valerian | Wet meadow | Sleep aid | None | Uncommon |
| Catnip | Disturbed | Calming, fever | None | Common |
| Chamomile | Fields | Calming, digestive | Allergy | Common |
| Feverfew | Fields | Headache/fever | None | Uncommon |

### Fungi (12 Species)

| Species | Habitat | Edibility | Special |
|---------|---------|-----------|---------|
| Chanterelle | Forest floor | Choice edible | Golden, fruity |
| Morel | Spring forest | Choice edible | Must cook |
| Chicken of Woods | Dead trees | Good edible | Orange shelves |
| Hen of Woods | Oak bases | Good edible | Medicinal |
| Lion's Mane | Hardwood | Good edible | Brain health |
| Giant Puffball | Meadows | Edible (young) | Must be white inside |
| Turkey Tail | Dead wood | Medicinal tea | Immune boost |
| Reishi | Hemlock | Medicinal | Longevity tonic |
| **Destroying Angel** | Forest | **DEADLY** | White, beautiful |
| **Death Cap** | Under oaks | **DEADLY** | Delayed symptoms |
| False Morel | Spring | **TOXIC** | Wrinkled, not pitted |
| Jack O'Lantern | Tree bases | **TOXIC** | Glows faintly |

---

## Growth & Lifecycle

### Growth Simulation

```rust
// flora/growth.rs

pub struct GrowthSystem {
    pub species_defs: HashMap<FloraSpecies, FloraSpeciesDef>,
}

impl GrowthSystem {
    /// Update plant growth based on time passed
    pub fn update(&self, plants: &mut [FloraInstance], dt_days: f32, season: Season, weather: &WeatherState) {
        for plant in plants.iter_mut() {
            let def = self.species_defs.get(&plant.species).unwrap();

            // Check if growing season
            if def.dormant_seasons.contains(&season) {
                plant.is_dormant = true;
                plant.current_appearance = SeasonalAppearance::Dormant;
                continue;
            }

            plant.is_dormant = false;
            plant.age_days += dt_days;

            // Growth rate modifiers
            let mut growth_mod = 1.0;

            // Weather effects
            growth_mod *= match weather.current {
                WeatherType::Stormy => 1.3,  // Rain helps
                WeatherType::Clear if season == Season::Summer => 0.8,  // Drought stress
                _ => 1.0,
            };

            // Health affects growth
            growth_mod *= plant.health;

            // Progress through current stage
            let stage_duration = self.get_stage_duration(&plant.growth_stage, def);
            plant.growth_progress += (dt_days / stage_duration) * growth_mod;

            // Advance stage if ready
            if plant.growth_progress >= 1.0 {
                plant.growth_progress = 0.0;
                plant.growth_stage = self.next_stage(plant.growth_stage, def, season);
            }

            // Update appearance
            plant.current_appearance = self.calculate_appearance(plant, def, season);
        }
    }

    fn next_stage(&self, current: GrowthStage, def: &FloraSpeciesDef, season: Season) -> GrowthStage {
        match current {
            GrowthStage::Seed => GrowthStage::Seedling,
            GrowthStage::Seedling => GrowthStage::Juvenile,
            GrowthStage::Juvenile => GrowthStage::Mature,
            GrowthStage::Mature => {
                if def.bloom_season == Some(season) {
                    GrowthStage::Flowering
                } else {
                    GrowthStage::Mature
                }
            },
            GrowthStage::Flowering => {
                if def.fruit_season.is_some() {
                    GrowthStage::Fruiting
                } else {
                    GrowthStage::Mature
                }
            },
            GrowthStage::Fruiting => GrowthStage::Mature,
            GrowthStage::Senescent => GrowthStage::Dead,
            GrowthStage::Dead => GrowthStage::Dead,
        }
    }

    fn calculate_appearance(&self, plant: &FloraInstance, def: &FloraSpeciesDef, season: Season) -> SeasonalAppearance {
        if plant.is_dormant {
            return SeasonalAppearance::Dormant;
        }

        match plant.growth_stage {
            GrowthStage::Flowering => SeasonalAppearance::Flowering,
            GrowthStage::Fruiting => SeasonalAppearance::Fruiting,
            GrowthStage::Mature | GrowthStage::Juvenile => {
                match season {
                    Season::Spring => SeasonalAppearance::Budding,
                    Season::Summer => SeasonalAppearance::FullFoliage,
                    Season::Fall => {
                        if def.category == FloraCategory::Tree &&
                           !matches!(def.id, FloraSpecies::LoblollyPine | FloraSpecies::LongleafPine | FloraSpecies::EasternRedCedar) {
                            SeasonalAppearance::FallColors
                        } else {
                            SeasonalAppearance::FullFoliage
                        }
                    },
                    Season::Winter => {
                        if self.is_evergreen(def) {
                            SeasonalAppearance::Evergreen
                        } else {
                            SeasonalAppearance::Bare
                        }
                    },
                }
            },
            _ => SeasonalAppearance::FullFoliage,
        }
    }

    fn is_evergreen(&self, def: &FloraSpeciesDef) -> bool {
        matches!(def.id,
            FloraSpecies::LoblollyPine |
            FloraSpecies::LongleafPine |
            FloraSpecies::EasternRedCedar |
            FloraSpecies::EasternHemlock |
            FloraSpecies::AmericanHolly |
            FloraSpecies::Rhododendron |
            FloraSpecies::MountainLaurel
        )
    }
}
```

---

## Harvesting System

### Harvestable Parts

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlantPart {
    // Common
    Leaves,
    Stems,
    Roots,
    Bark,
    Wood,

    // Reproductive
    Flowers,
    Seeds,
    Fruit,
    Berries,
    Nuts,
    Acorns,

    // Special
    Resin,
    Sap,
    Pollen,
    Mushroom,
    Tuber,
    Bulb,
    Rhizome,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HarvestYields {
    pub leaves: Option<(u32, u32)>,      // (min, max)
    pub stems: Option<(u32, u32)>,
    pub roots: Option<(u32, u32)>,
    pub bark: Option<(u32, u32)>,
    pub wood: Option<(u32, u32)>,
    pub flowers: Option<(u32, u32)>,
    pub seeds: Option<(u32, u32)>,
    pub fruit: Option<(u32, u32)>,
    pub berries: Option<(u32, u32)>,
    pub nuts: Option<(u32, u32)>,
    pub acorns: Option<(u32, u32)>,
    pub resin: Option<(u32, u32)>,
    pub sap: Option<(u32, u32)>,
    pub mushroom: Option<(u32, u32)>,
    pub tuber: Option<(u32, u32)>,
}

/// Harvest action and results
pub struct HarvestSystem;

impl HarvestSystem {
    pub fn harvest(
        plant: &mut FloraInstance,
        part: PlantPart,
        tool: Option<&Tool>,
        skills: &ForagingSkills,
    ) -> HarvestResult {
        let def = get_flora_def(plant.species);

        // Check if part is available
        if !plant.available_parts.contains(&part) {
            return HarvestResult::PartNotAvailable;
        }

        // Check growth stage
        if !plant.growth_stage.is_harvestable() {
            return HarvestResult::NotReady;
        }

        // Calculate yield
        let base_yield = def.harvest_yields.get_yield(part).unwrap();
        let mut yield_amount = rand::thread_rng().gen_range(base_yield.0..=base_yield.1);

        // Tool bonus
        if let Some(tool) = tool {
            yield_amount = (yield_amount as f32 * tool.harvest_multiplier()) as u32;
        }

        // Skill bonus
        yield_amount = (yield_amount as f32 * skills.yield_multiplier()) as u32;

        // Quality determination
        let quality = Self::determine_quality(plant, skills);

        // Remove from available parts (or mark as harvested)
        if def.regrows_after_harvest {
            plant.last_harvested = Some(get_game_time());
            plant.available_parts.retain(|p| *p != part);
        } else if part == PlantPart::Roots || part == PlantPart::Wood {
            // Destructive harvest
            plant.growth_stage = GrowthStage::Dead;
        }

        plant.times_harvested += 1;

        HarvestResult::Success {
            item: HarvestedItem {
                species: plant.species,
                part,
                quantity: yield_amount,
                quality,
            },
            plant_destroyed: plant.growth_stage == GrowthStage::Dead,
        }
    }

    fn determine_quality(plant: &FloraInstance, skills: &ForagingSkills) -> ItemQuality {
        let base_quality = plant.health * 0.5 + plant.growth_progress * 0.3;
        let skill_bonus = skills.quality_bonus();

        let quality_score = base_quality + skill_bonus;

        if quality_score > 0.9 { ItemQuality::Exceptional }
        else if quality_score > 0.7 { ItemQuality::Good }
        else if quality_score > 0.4 { ItemQuality::Average }
        else { ItemQuality::Poor }
    }
}
```

### Harvest Tools

| Tool | Speed | Yield | Quality | Best For |
|------|-------|-------|---------|----------|
| Bare Hands | 1x | 0.8x | 0.9x | Berries, leaves |
| Knife | 1.5x | 1.0x | 1.0x | Herbs, bark |
| Sickle | 2x | 1.2x | 0.95x | Grasses, grains |
| Trowel | 1.5x | 1.1x | 1.0x | Roots, tubers |
| Axe | 1x | 1.0x | 0.8x | Wood, bark |
| Pruning Shears | 2x | 1.3x | 1.1x | Precise cuts |

---

## Medicinal & Alchemical Uses

### Medicinal Effects

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MedicinalEffect {
    // Healing
    WoundHealing,       // Speeds wound recovery
    StopBleeding,       // Stops bleeding status
    BoneHealing,        // Speeds fracture recovery
    BurnTreatment,      // Reduces burn damage

    // Illness
    FeverReduction,     // Reduces fever
    InfectionCure,      // Cures infection
    PoisonCure,         // Neutralizes poison
    SnakebiteRemedy,    // Specifically for venom
    ParasiteCure,       // Removes parasites

    // Buffs
    StaminaRestore,     // Restores stamina
    StaminaBoost,       // Increases max stamina
    HealthRegen,        // Slow health regeneration
    PainRelief,         // Reduces pain debuff
    ImmunityBoost,      // Resistance to disease

    // Mental
    Calming,            // Reduces fear/stress
    SleepAid,           // Helps with insomnia
    Stimulant,          // Increases alertness
    AntiDepressant,     // Mood improvement
    Hallucinogenic,     // Vision effects

    // Utility
    Astringent,         // Tightens tissue
    Antiseptic,         // Prevents infection
    AntiInflammatory,   // Reduces swelling
    Diuretic,           // Increases urination
    Laxative,           // Digestive aid
    Emetic,             // Induces vomiting (poison treatment)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicinalProperty {
    pub effect: MedicinalEffect,
    pub potency: f32,           // 0.0 - 1.0
    pub preparation: PreparationType,
    pub part: PlantPart,
    pub discovered: bool,       // Has player learned this?
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparationType {
    Raw,            // Eat/apply directly
    Dried,          // Sun-dried
    Crushed,        // Mortar and pestle
    Poultice,       // Crushed + water, applied
    Tea,            // Steeped in hot water
    Decoction,      // Boiled
    Tincture,       // Alcohol extract
    Salve,          // Mixed with fat/oil
    Smoke,          // Inhaled
    Compress,       // Wrapped on body
}

impl PreparationType {
    pub fn potency_modifier(&self) -> f32 {
        match self {
            Self::Raw => 0.5,
            Self::Dried => 0.7,
            Self::Crushed => 0.8,
            Self::Poultice => 0.9,
            Self::Tea => 0.8,
            Self::Decoction => 1.0,
            Self::Tincture => 1.3,
            Self::Salve => 1.0,
            Self::Smoke => 0.7,
            Self::Compress => 0.9,
        }
    }

    pub fn required_tools(&self) -> Vec<ToolType> {
        match self {
            Self::Raw => vec![],
            Self::Dried => vec![],  // Just time
            Self::Crushed => vec![ToolType::MortarPestle],
            Self::Poultice => vec![ToolType::MortarPestle],
            Self::Tea => vec![ToolType::Pot, ToolType::Fire],
            Self::Decoction => vec![ToolType::Pot, ToolType::Fire],
            Self::Tincture => vec![ToolType::Jar, ToolType::Alcohol],
            Self::Salve => vec![ToolType::Pot, ToolType::Fat],
            Self::Smoke => vec![ToolType::Fire, ToolType::Pipe],
            Self::Compress => vec![ToolType::Cloth],
        }
    }
}
```

### Recipe System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicineRecipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<RecipeIngredient>,
    pub preparation: PreparationType,
    pub effects: Vec<RecipeEffect>,
    pub discovery_method: RecipeDiscoveryMethod,
    pub discovered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub species: FloraSpecies,
    pub part: PlantPart,
    pub quantity: u32,
    pub quality_minimum: Option<ItemQuality>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeEffect {
    pub effect: MedicinalEffect,
    pub potency: f32,
    pub duration: f32,  // Seconds
}

pub fn get_medicine_recipes() -> Vec<MedicineRecipe> {
    vec![
        MedicineRecipe {
            id: "healing_poultice".into(),
            name: "Healing Poultice".into(),
            ingredients: vec![
                RecipeIngredient {
                    species: FloraSpecies::Yarrow,
                    part: PlantPart::Leaves,
                    quantity: 2,
                    quality_minimum: None,
                },
                RecipeIngredient {
                    species: FloraSpecies::Plantain,
                    part: PlantPart::Leaves,
                    quantity: 2,
                    quality_minimum: None,
                },
            ],
            preparation: PreparationType::Poultice,
            effects: vec![
                RecipeEffect {
                    effect: MedicinalEffect::WoundHealing,
                    potency: 0.8,
                    duration: 600.0,
                },
                RecipeEffect {
                    effect: MedicinalEffect::StopBleeding,
                    potency: 0.9,
                    duration: 60.0,
                },
            ],
            discovery_method: RecipeDiscoveryMethod::NativeTeaching,
            discovered: false,
        },

        MedicineRecipe {
            id: "ginseng_tonic".into(),
            name: "Ginseng Stamina Tonic".into(),
            ingredients: vec![
                RecipeIngredient {
                    species: FloraSpecies::AmericanGinseng,
                    part: PlantPart::Roots,
                    quantity: 1,
                    quality_minimum: Some(ItemQuality::Good),
                },
            ],
            preparation: PreparationType::Decoction,
            effects: vec![
                RecipeEffect {
                    effect: MedicinalEffect::StaminaRestore,
                    potency: 1.0,
                    duration: 0.0,  // Instant
                },
                RecipeEffect {
                    effect: MedicinalEffect::StaminaBoost,
                    potency: 0.5,
                    duration: 1800.0,  // 30 minutes
                },
            ],
            discovery_method: RecipeDiscoveryMethod::Experimentation,
            discovered: false,
        },

        MedicineRecipe {
            id: "snakebite_remedy".into(),
            name: "Snakebite Remedy".into(),
            ingredients: vec![
                RecipeIngredient {
                    species: FloraSpecies::Echinacea,
                    part: PlantPart::Roots,
                    quantity: 2,
                    quality_minimum: None,
                },
                RecipeIngredient {
                    species: FloraSpecies::BlackCohosh,
                    part: PlantPart::Roots,
                    quantity: 1,
                    quality_minimum: None,
                },
            ],
            preparation: PreparationType::Decoction,
            effects: vec![
                RecipeEffect {
                    effect: MedicinalEffect::SnakebiteRemedy,
                    potency: 0.9,
                    duration: 0.0,
                },
                RecipeEffect {
                    effect: MedicinalEffect::PoisonCure,
                    potency: 0.7,
                    duration: 0.0,
                },
            ],
            discovery_method: RecipeDiscoveryMethod::NativeTeaching,
            discovered: false,
        },

        MedicineRecipe {
            id: "poison_ivy_cure".into(),
            name: "Poison Ivy Salve".into(),
            ingredients: vec![
                RecipeIngredient {
                    species: FloraSpecies::JewelWeed,
                    part: PlantPart::Stems,
                    quantity: 3,
                    quality_minimum: None,
                },
            ],
            preparation: PreparationType::Crushed,
            effects: vec![
                RecipeEffect {
                    effect: MedicinalEffect::AntiInflammatory,
                    potency: 0.95,
                    duration: 300.0,
                },
            ],
            discovery_method: RecipeDiscoveryMethod::Experimentation,
            discovered: false,
        },

        // ... many more recipes
    ]
}
```

---

## Poison & Danger System

### Dangerous Plants

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantDanger {
    pub danger_type: DangerType,
    pub affected_parts: Vec<PlantPart>,
    pub onset_time: f32,           // Seconds until effect
    pub severity: DangerSeverity,
    pub treatment: Option<TreatmentInfo>,
    pub discovered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerType {
    ContactPoison,      // Touch causes reaction
    IngestedPoison,     // Must be eaten
    InhaledPoison,      // Spores or pollen
    Allergenic,         // May cause allergic reaction
    Thorns,             // Physical damage
    Irritant,           // Causes rash/itch
    Photosensitizer,    // Causes sun sensitivity
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerSeverity {
    Minor,          // Discomfort, minor debuff
    Moderate,       // Significant debuff, some damage
    Severe,         // Major damage, dangerous
    Lethal,         // Can kill without treatment
    Deadly,         // Almost always fatal
}

/// Poison effects applied to player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoisonEffect {
    pub source: FloraSpecies,
    pub poison_type: PoisonType,
    pub severity: f32,          // 0.0 - 1.0
    pub onset_progress: f32,    // 0.0 = just exposed, 1.0 = symptoms starting
    pub duration_remaining: f32,
    pub symptoms: Vec<PoisonSymptom>,
    pub treated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoisonType {
    // Contact poisons
    Urushiol,           // Poison ivy/oak
    Raphides,           // Jack-in-the-pulpit crystals

    // Ingested poisons
    Amatoxin,           // Destroying angel/death cap
    Gyromitrin,         // False morel
    Muscarine,          // Jack o'lantern
    Sanguinarine,       // Bloodroot
    Podophyllotoxin,    // May apple

    // Complex
    Cardiac,            // Affects heart
    Neurotoxic,         // Affects nervous system
    Hepatotoxic,        // Affects liver
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoisonSymptom {
    Nausea,
    Vomiting,
    Diarrhea,
    AbdominalPain,
    Sweating,
    Dizziness,
    Confusion,
    BlurredVision,
    HeartPalpitations,
    DifficultyBreathing,
    Seizures,
    LiverFailure,
    Death,
}
```

### Deadly Fungi Detail

```rust
/// Death Cap and Destroying Angel - most dangerous fungi
pub fn create_deadly_amanita_entry() -> FloraSpeciesDef {
    FloraSpeciesDef {
        id: FloraSpecies::DestroyingAngel,
        common_name: "Destroying Angel",
        scientific_name: "Amanita bisporigera",
        native_names: vec![],
        category: FloraCategory::Fungus,

        size: FloraSize::Small,
        growth_rate: 3.0,
        lifespan: Some(7.0),
        spread_rate: 0.1,

        habitats: vec![Habitat::DeciduousForest, Habitat::MixedForest],
        soil_preference: SoilType::Loamy,
        moisture_needs: MoistureLevel::Moderate,
        light_needs: LightLevel::Shade,
        temperature_range: (10.0, 25.0),

        growth_seasons: vec![Season::Summer, Season::Fall],
        bloom_season: None,
        fruit_season: None,
        dormant_seasons: vec![Season::Winter, Season::Spring],

        harvestable_parts: vec![PlantPart::Mushroom],
        harvest_yields: HarvestYields {
            mushroom: Some((1, 2)),
            ..Default::default()
        },
        regrows_after_harvest: false,
        regrowth_time: None,

        edibility: Edibility::DeadlyPoisonous,
        medicinal_properties: vec![],
        crafting_uses: vec![CraftingUse::Poison],

        dangers: vec![
            PlantDanger {
                danger_type: DangerType::IngestedPoison,
                affected_parts: vec![PlantPart::Mushroom],
                onset_time: 21600.0,  // 6 hours - terrifyingly delayed
                severity: DangerSeverity::Deadly,
                treatment: Some(TreatmentInfo {
                    treatment_type: TreatmentType::EmergencyMedical,
                    effectiveness: 0.3,  // Even treatment often fails
                    time_window: 7200.0,  // Must treat within 2 hours of ingestion
                }),
                discovered: false,
            }
        ],

        spawn_rate: 0.05,
        cluster_size: (1, 3),
        min_spacing: 2.0,

        mesh_variant: "mushroom_white_elegant",
        seasonal_colors: SeasonalColors::white(),
    }
}

/// Poison progression for deadly fungi
impl PoisonEffect {
    pub fn amatoxin_progression(&mut self, dt: f32) -> Vec<PoisonSymptom> {
        // Amatoxin has distinct phases
        let time_elapsed = self.total_time();
        let mut new_symptoms = vec![];

        // Phase 1: 6-12 hours - GI symptoms
        if time_elapsed > 21600.0 && time_elapsed < 43200.0 {
            if !self.symptoms.contains(&PoisonSymptom::Nausea) {
                new_symptoms.push(PoisonSymptom::Nausea);
                new_symptoms.push(PoisonSymptom::Vomiting);
                new_symptoms.push(PoisonSymptom::Diarrhea);
                new_symptoms.push(PoisonSymptom::AbdominalPain);
            }
        }

        // Phase 2: 24-72 hours - False recovery (symptoms ease)
        // Player may think they're getting better...

        // Phase 3: 72+ hours - Liver/kidney failure
        if time_elapsed > 259200.0 {
            if !self.symptoms.contains(&PoisonSymptom::LiverFailure) {
                new_symptoms.push(PoisonSymptom::LiverFailure);
                new_symptoms.push(PoisonSymptom::Confusion);
            }
        }

        // Phase 4: Death
        if time_elapsed > 345600.0 && !self.treated {
            new_symptoms.push(PoisonSymptom::Death);
        }

        self.symptoms.extend(new_symptoms.clone());
        new_symptoms
    }
}
```

### Look-alike Warnings

The game warns about dangerous look-alikes:

| Deadly Species | Safe Look-alike | Key Difference |
|----------------|-----------------|----------------|
| Destroying Angel | Button Mushroom | Volva at base, white gills |
| Death Cap | Paddy Straw | Volva, ring on stem |
| False Morel | True Morel | Wrinkled vs pitted cap |
| Jack O'Lantern | Chanterelle | True gills vs false ridges |
| Poison Hemlock | Wild Carrot | Purple blotches on stem |
| Water Hemlock | Angelica | Chambered root |

---

## Cultivation & Farming

### Garden System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Garden {
    pub id: GardenId,
    pub position: Vec3,
    pub size: (u32, u32),  // Grid cells
    pub plots: Vec<GardenPlot>,
    pub soil_quality: f32,
    pub irrigation: bool,
    pub fenced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GardenPlot {
    pub grid_pos: (u32, u32),
    pub plant: Option<CultivatedPlant>,
    pub soil_state: SoilState,
    pub watered: bool,
    pub fertilized: bool,
    pub weeds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultivatedPlant {
    pub species: FloraSpecies,
    pub planted_day: f32,
    pub growth_stage: GrowthStage,
    pub growth_progress: f32,
    pub health: f32,
    pub watered_today: bool,
    pub days_without_water: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoilState {
    Untilled,
    Tilled,
    Planted,
    Fallow,
    Depleted,
}

impl Garden {
    pub fn update(&mut self, dt_days: f32, weather: &WeatherState, season: Season) {
        for plot in &mut self.plots {
            // Natural watering from rain
            if weather.is_raining() {
                plot.watered = true;
            }

            // Weed growth
            if season != Season::Winter {
                plot.weeds += 0.05 * dt_days;
            }

            // Plant growth
            if let Some(plant) = &mut plot.plant {
                let mut growth_rate = 1.0;

                // Water requirement
                if !plant.watered_today {
                    plant.days_without_water += 1;
                    if plant.days_without_water > 2 {
                        plant.health -= 0.1 * dt_days;
                    }
                } else {
                    plant.days_without_water = 0;
                }

                // Soil quality
                growth_rate *= plot.soil_state.fertility_modifier();

                // Fertilizer bonus
                if plot.fertilized {
                    growth_rate *= 1.3;
                }

                // Weed competition
                if plot.weeds > 0.5 {
                    growth_rate *= 0.5;
                }

                // Health affects growth
                growth_rate *= plant.health;

                // Progress growth
                let def = get_flora_def(plant.species);
                let days_to_mature = def.growth_rate / 10.0;  // Crops grow faster
                plant.growth_progress += dt_days / days_to_mature * growth_rate;

                // Stage advancement
                if plant.growth_progress >= 1.0 {
                    plant.advance_stage();
                }

                plant.watered_today = false;
            }
        }
    }
}
```

### Three Sisters Planting

Native American companion planting provides bonuses:

```rust
/// Check for Three Sisters companion planting bonus
pub fn check_companion_bonus(garden: &Garden, plot: &GardenPlot) -> f32 {
    let mut bonus = 0.0;
    let (x, y) = plot.grid_pos;

    // Get adjacent plants
    let neighbors = garden.get_neighbors(x, y);

    if let Some(plant) = &plot.plant {
        match plant.species {
            FloraSpecies::Corn => {
                // Corn benefits from beans (nitrogen)
                if neighbors.iter().any(|p| matches!(p.species, FloraSpecies::Beans)) {
                    bonus += 0.2;
                }
            },
            FloraSpecies::Beans => {
                // Beans climb corn stalks
                if neighbors.iter().any(|p| matches!(p.species, FloraSpecies::Corn)) {
                    bonus += 0.25;
                }
            },
            FloraSpecies::Squash => {
                // Squash leaves shade soil, reduce weeds
                if neighbors.iter().any(|p| matches!(p.species, FloraSpecies::Corn | FloraSpecies::Beans)) {
                    bonus += 0.15;
                    // Also reduces weed growth in plot
                }
            },
            _ => {}
        }
    }

    bonus
}
```

### Crop Calendar

| Crop | Plant Season | Harvest Season | Days to Mature |
|------|--------------|----------------|----------------|
| Corn | Spring | Fall | 90 |
| Beans | Spring | Summer/Fall | 60 |
| Squash | Spring | Fall | 80 |
| Pumpkin | Spring | Fall | 100 |
| Tobacco | Spring | Summer | 70 |
| Sunflower | Spring | Fall | 80 |
| Sweet Potato | Spring | Fall | 120 |
| Cotton | Spring | Fall | 150 |

---

## Foraging Skill Tree

### Skill Structure

```
                           [MASTER BOTANIST]
                                   |
                  +----------------+----------------+
                  |                                 |
           [Herbal Mastery]                 [Fungal Mastery]
                  |                                 |
         +--------+--------+              +--------+--------+
         |                 |              |                 |
    [Herbalist]     [Poison Sage]    [Mushroomer]    [Cultivation]
         |                 |              |                 |
         +--------+--------+              +--------+--------+
                  |                                 |
           [Plant Identifier]              [Mushroom Hunter]
                  |                                 |
                  +----------------+----------------+
                                   |
                           [FORAGER'S EYE]
                            (Starting Point)
```

### Skills Detail

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForagingSkills {
    pub points: u32,

    // Tier 1
    pub foragers_eye: bool,         // Starting skill

    // Tier 2
    pub plant_identifier: bool,     // Identify plants faster
    pub mushroom_hunter: bool,      // Find mushrooms easier

    // Tier 3
    pub herbalist: bool,            // Medicinal knowledge
    pub poison_sage: bool,          // Identify poisons
    pub mushroomer: bool,           // All fungi knowledge
    pub cultivation: bool,          // Better farming

    // Tier 4
    pub herbal_mastery: bool,       // Max medicine potency
    pub fungal_mastery: bool,       // Perfect fungi ID

    // Tier 5
    pub master_botanist: bool,      // All bonuses

    // Tracking
    pub plants_harvested: u32,
    pub species_discovered: u32,
    pub poisons_identified: u32,
    pub medicines_crafted: u32,
    pub crops_harvested: u32,
}

impl ForagingSkills {
    pub fn yield_multiplier(&self) -> f32 {
        let mut mult = 1.0;
        if self.foragers_eye { mult += 0.1; }
        if self.plant_identifier { mult += 0.15; }
        if self.herbalist { mult += 0.2; }
        if self.herbal_mastery { mult += 0.25; }
        if self.master_botanist { mult += 0.3; }
        mult
    }

    pub fn quality_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if self.foragers_eye { bonus += 0.05; }
        if self.plant_identifier { bonus += 0.1; }
        if self.herbalist { bonus += 0.15; }
        if self.master_botanist { bonus += 0.2; }
        bonus
    }

    pub fn poison_detection_chance(&self) -> f32 {
        if self.master_botanist { return 1.0; }
        if self.poison_sage { return 0.9; }
        if self.mushroomer { return 0.7; }
        if self.mushroom_hunter { return 0.5; }
        0.2
    }

    pub fn medicine_potency_bonus(&self) -> f32 {
        if self.master_botanist { return 0.5; }
        if self.herbal_mastery { return 0.4; }
        if self.herbalist { return 0.25; }
        0.0
    }
}
```

---

## Seasonal Behavior

### Seasonal Plant States

```rust
impl FloraInstance {
    pub fn update_for_season(&mut self, season: Season, def: &FloraSpeciesDef) {
        // Check dormancy
        self.is_dormant = def.dormant_seasons.contains(&season);

        // Update appearance
        match season {
            Season::Spring => {
                if def.growth_seasons.contains(&Season::Spring) {
                    self.current_appearance = SeasonalAppearance::Budding;
                }
                if def.bloom_season == Some(Season::Spring) {
                    self.has_bloomed = true;
                    self.growth_stage = GrowthStage::Flowering;
                }
            },
            Season::Summer => {
                if !self.is_dormant {
                    self.current_appearance = SeasonalAppearance::FullFoliage;
                }
                if def.fruit_season == Some(Season::Summer) {
                    self.growth_stage = GrowthStage::Fruiting;
                }
            },
            Season::Fall => {
                // Deciduous trees change color
                if def.category == FloraCategory::Tree && !is_evergreen(def.id) {
                    self.current_appearance = SeasonalAppearance::FallColors;
                }
                if def.fruit_season == Some(Season::Fall) {
                    self.growth_stage = GrowthStage::Fruiting;
                }
            },
            Season::Winter => {
                if is_evergreen(def.id) {
                    self.current_appearance = SeasonalAppearance::Evergreen;
                } else if def.category == FloraCategory::Tree {
                    self.current_appearance = SeasonalAppearance::Bare;
                } else {
                    self.is_dormant = true;
                }
            },
        }

        // Update available parts based on season
        self.update_available_parts(season, def);
    }

    fn update_available_parts(&mut self, season: Season, def: &FloraSpeciesDef) {
        self.available_parts.clear();

        // Always available (if mature)
        if self.growth_stage >= GrowthStage::Mature {
            if def.harvestable_parts.contains(&PlantPart::Bark) {
                self.available_parts.push(PlantPart::Bark);
            }
            if def.harvestable_parts.contains(&PlantPart::Wood) {
                self.available_parts.push(PlantPart::Wood);
            }
            if def.harvestable_parts.contains(&PlantPart::Roots) {
                self.available_parts.push(PlantPart::Roots);
            }
        }

        // Seasonal parts
        if !self.is_dormant {
            if def.harvestable_parts.contains(&PlantPart::Leaves) {
                self.available_parts.push(PlantPart::Leaves);
            }
        }

        if self.growth_stage == GrowthStage::Flowering {
            if def.harvestable_parts.contains(&PlantPart::Flowers) {
                self.available_parts.push(PlantPart::Flowers);
            }
        }

        if self.growth_stage == GrowthStage::Fruiting {
            for part in [PlantPart::Fruit, PlantPart::Berries, PlantPart::Nuts, PlantPart::Acorns] {
                if def.harvestable_parts.contains(&part) {
                    self.available_parts.push(part);
                }
            }
        }
    }
}
```

---

## Environmental Integration

### Biome Plant Distribution

```rust
pub fn get_biome_plants(biome: Biome) -> Vec<(FloraSpecies, f32)> {
    match biome {
        Biome::DeciduousForest => vec![
            (FloraSpecies::WhiteOak, 0.3),
            (FloraSpecies::RedOak, 0.25),
            (FloraSpecies::AmericanBeech, 0.2),
            (FloraSpecies::TulipPoplar, 0.15),
            (FloraSpecies::SugarMaple, 0.15),
            (FloraSpecies::FloweringDogwood, 0.1),
            (FloraSpecies::AmericanGinseng, 0.02),
            (FloraSpecies::Goldenseal, 0.03),
            (FloraSpecies::BlackCohosh, 0.05),
            (FloraSpecies::Trillium, 0.08),
            (FloraSpecies::Chanterelle, 0.05),
            (FloraSpecies::Morel, 0.02),
        ],
        Biome::PineForest => vec![
            (FloraSpecies::LoblollyPine, 0.4),
            (FloraSpecies::LongleafPine, 0.3),
            (FloraSpecies::EasternRedCedar, 0.15),
            (FloraSpecies::Blueberry, 0.1),
            (FloraSpecies::Rhododendron, 0.08),
        ],
        Biome::Swamp => vec![
            (FloraSpecies::BaldCypress, 0.4),
            (FloraSpecies::Cattail, 0.3),
            (FloraSpecies::WaterLily, 0.2),
            (FloraSpecies::Pickerelweed, 0.15),
            (FloraSpecies::ButtonBush, 0.1),
            (FloraSpecies::SkullCap, 0.05),
        ],
        Biome::Meadow => vec![
            (FloraSpecies::BlackEyedSusan, 0.2),
            (FloraSpecies::Echinacea, 0.15),
            (FloraSpecies::Yarrow, 0.2),
            (FloraSpecies::WildRose, 0.1),
            (FloraSpecies::Mullein, 0.1),
            (FloraSpecies::Chamomile, 0.08),
        ],
        Biome::RiverBank => vec![
            (FloraSpecies::AmericanSycamore, 0.25),
            (FloraSpecies::WildMint, 0.15),
            (FloraSpecies::JewelWeed, 0.2),
            (FloraSpecies::WoodDuck, 0.1),
            (FloraSpecies::Cattail, 0.2),
        ],
        // ...
    }
}
```

### Weather Effects on Plants

```rust
pub fn apply_weather_effects(plants: &mut [FloraInstance], weather: &WeatherState, dt: f32) {
    for plant in plants {
        match weather.current_weather {
            WeatherType::Stormy => {
                // Heavy rain helps water-loving plants
                if plant_prefers_moisture(plant.species) {
                    plant.health = (plant.health + 0.01 * dt).min(1.0);
                }
                // Wind damage to tall plants
                if get_flora_def(plant.species).size >= FloraSize::Large {
                    let damage_chance = 0.001 * dt;
                    if rand::random::<f32>() < damage_chance {
                        plant.health -= 0.1;
                    }
                }
            },
            WeatherType::Clear => {
                // Drought stress in summer
                if get_current_season() == Season::Summer {
                    if !plant_is_drought_tolerant(plant.species) {
                        plant.health -= 0.001 * dt;
                    }
                }
            },
            WeatherType::Foggy => {
                // Moisture-loving plants thrive
                if plant_prefers_moisture(plant.species) {
                    plant.health = (plant.health + 0.005 * dt).min(1.0);
                }
            },
            _ => {}
        }
    }
}
```

---

## Native Plant Knowledge

### Traditional Uses

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeKnowledge {
    pub species: FloraSpecies,
    pub tribe: &'static str,
    pub native_name: &'static str,
    pub traditional_uses: Vec<TraditionalUse>,
    pub spiritual_significance: Option<String>,
    pub taboos: Vec<String>,
    pub discovery_requirement: NativeDiscoveryRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraditionalUse {
    pub use_type: UseType,
    pub description: String,
    pub preparation: Option<PreparationType>,
    pub effectiveness: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseType {
    Medicine,
    Food,
    Tool,
    Dye,
    Ceremony,
    Construction,
    Fiber,
    Poison,
}

pub fn get_native_knowledge(species: FloraSpecies) -> Option<NativeKnowledge> {
    match species {
        FloraSpecies::AmericanGinseng => Some(NativeKnowledge {
            species,
            tribe: "Cherokee",
            native_name: "Atali-guli",
            traditional_uses: vec![
                TraditionalUse {
                    use_type: UseType::Medicine,
                    description: "Root chewed for strength before battle or long journey".into(),
                    preparation: Some(PreparationType::Raw),
                    effectiveness: 0.9,
                },
                TraditionalUse {
                    use_type: UseType::Ceremony,
                    description: "Offered to water spirits for safe river crossing".into(),
                    preparation: None,
                    effectiveness: 1.0,
                },
            ],
            spiritual_significance: Some(
                "The man-root spirit aids warriors and hunters. \
                 It hides from the greedy but reveals itself to the respectful.".into()
            ),
            taboos: vec![
                "Never harvest all ginseng from an area - leave the eldest plant".into(),
                "Plant the red berries before taking the root".into(),
                "Do not dig ginseng when the moon is dark".into(),
            ],
            discovery_requirement: NativeDiscoveryRequirement::Relationship(Faction::Cherokee, 2),
        }),

        FloraSpecies::Sassafras => Some(NativeKnowledge {
            species,
            tribe: "Powhatan",
            native_name: "Pavnees",
            traditional_uses: vec![
                TraditionalUse {
                    use_type: UseType::Medicine,
                    description: "Root bark tea purifies the blood in spring".into(),
                    preparation: Some(PreparationType::Tea),
                    effectiveness: 0.8,
                },
                TraditionalUse {
                    use_type: UseType::Food,
                    description: "Dried leaves ground into filé powder for stews".into(),
                    preparation: Some(PreparationType::Dried),
                    effectiveness: 1.0,
                },
            ],
            spiritual_significance: Some(
                "The three-shaped leaves represent past, present, and future.".into()
            ),
            taboos: vec![],
            discovery_requirement: NativeDiscoveryRequirement::Relationship(Faction::Powhatan, 1),
        }),

        FloraSpecies::Tobacco => Some(NativeKnowledge {
            species,
            tribe: "Powhatan",
            native_name: "Uppowoc",
            traditional_uses: vec![
                TraditionalUse {
                    use_type: UseType::Ceremony,
                    description: "Smoke carries prayers to the spirits".into(),
                    preparation: Some(PreparationType::Smoke),
                    effectiveness: 1.0,
                },
                TraditionalUse {
                    use_type: UseType::Medicine,
                    description: "Poultice draws venom from snakebites".into(),
                    preparation: Some(PreparationType::Poultice),
                    effectiveness: 0.6,
                },
            ],
            spiritual_significance: Some(
                "Sacred plant of communication with ancestors. \
                 Burning tobacco opens the way between worlds.".into()
            ),
            taboos: vec![
                "Never use tobacco frivolously - it angers the spirits".into(),
                "Always offer tobacco before taking from the land".into(),
            ],
            discovery_requirement: NativeDiscoveryRequirement::Ceremony,
        }),

        _ => None,
    }
}
```

---

## Rendering & Visuals

### Flora Rendering Pipeline

```rust
// In crates/croatoan_render/src/flora_pipeline.rs

pub struct FloraPipeline {
    pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,

    // Meshes by size category
    tree_meshes: HashMap<String, TreeMesh>,
    shrub_meshes: HashMap<String, PlantMesh>,
    herb_meshes: HashMap<String, PlantMesh>,
    mushroom_meshes: HashMap<String, PlantMesh>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FloraInstance {
    pub model_matrix: [[f32; 4]; 4],
    pub seasonal_color: [f32; 4],
    pub growth_scale: f32,
    pub wind_phase: f32,
    pub _padding: [f32; 2],
}

impl FloraPipeline {
    pub fn render_flora(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        plants: &[FloraInstance],
        camera: &Camera,
    ) {
        // Frustum culling
        let visible: Vec<_> = plants.iter()
            .filter(|p| camera.frustum_contains(p.position))
            .collect();

        // LOD selection based on distance
        let (near, mid, far) = self.sort_by_lod(&visible, camera.position);

        // Render each LOD level
        // Near: Full detail
        // Mid: Reduced detail
        // Far: Billboard imposters
    }
}
```

### Seasonal Color System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalColors {
    pub spring: Vec3,    // Fresh green, some pink
    pub summer: Vec3,    // Deep green
    pub fall: Vec3,      // Orange, red, yellow
    pub winter: Vec3,    // Brown, gray, or green for evergreen
}

impl SeasonalColors {
    pub fn get_color(&self, season: Season, progress: f32) -> Vec3 {
        let (from, to) = match season {
            Season::Spring => (&self.winter, &self.spring),
            Season::Summer => (&self.spring, &self.summer),
            Season::Fall => (&self.summer, &self.fall),
            Season::Winter => (&self.fall, &self.winter),
        };

        from.lerp(*to, progress)
    }

    pub fn deciduous_default() -> Self {
        Self {
            spring: Vec3::new(0.5, 0.75, 0.3),
            summer: Vec3::new(0.2, 0.55, 0.2),
            fall: Vec3::new(0.9, 0.5, 0.2),
            winter: Vec3::new(0.4, 0.35, 0.3),
        }
    }

    pub fn evergreen_default() -> Self {
        Self {
            spring: Vec3::new(0.15, 0.4, 0.15),
            summer: Vec3::new(0.1, 0.35, 0.1),
            fall: Vec3::new(0.12, 0.38, 0.12),
            winter: Vec3::new(0.08, 0.3, 0.1),
        }
    }
}
```

---

## Implementation Priority

### Phase 1: Core Flora
- [ ] FloraSpecies enum and definitions
- [ ] FloraInstance entity
- [ ] Basic spawning in chunks
- [ ] Simple harvesting

### Phase 2: Growth System
- [ ] Growth stages
- [ ] Seasonal appearance changes
- [ ] Part availability by season

### Phase 3: Harvesting
- [ ] Harvest actions
- [ ] Tool bonuses
- [ ] Yield calculation
- [ ] Quality system

### Phase 4: Medicine
- [ ] Medicinal properties
- [ ] Recipe system
- [ ] Effect application
- [ ] Discovery mechanics

### Phase 5: Danger
- [ ] Poison system
- [ ] Symptom progression
- [ ] Treatment mechanics
- [ ] Look-alike warnings

### Phase 6: Cultivation
- [ ] Garden plots
- [ ] Planting mechanics
- [ ] Watering/care
- [ ] Companion planting

### Phase 7: Integration
- [ ] Encyclopedia integration
- [ ] Skill tree implementation
- [ ] Native knowledge system
- [ ] Visual rendering

---

*The plant life of Roanoke provides sustenance, healing, danger, and wonder to those who learn its secrets.*
