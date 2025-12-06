# Nature Discovery & Encyclopedia System Specification

## Roanoke Engine - Living World Knowledge System

This document specifies the comprehensive system for discovering, cataloging, and understanding the natural world of Roanoke. The Encyclopedia serves as both a gameplay progression system and an immersive window into colonial-era naturalism.

---

## Table of Contents

1. [Overview](#overview)
2. [Discovery Mechanics](#discovery-mechanics)
3. [Encyclopedia Structure](#encyclopedia-structure)
4. [Animal Entries](#animal-entries)
5. [Plant & Flora Entries](#plant--flora-entries)
6. [Environmental Features](#environmental-features)
7. [Knowledge Progression](#knowledge-progression)
8. [Field Notes System](#field-notes-system)
9. [Naturalist Skill Tree](#naturalist-skill-tree)
10. [Integration with Other Systems](#integration-with-other-systems)
11. [Data Structures](#data-structures)
12. [UI/UX Design](#uiux-design)

---

## Overview

### Design Philosophy

In the 16th century, natural philosophy was emerging as a formal discipline. Colonists arriving in the New World encountered countless species unknown to Europeans. This system captures that wonder of discovery while providing meaningful gameplay progression.

### Core Pillars

- **Discovery Through Observation**: Simply seeing an animal isn't enough. True knowledge comes from patient study.
- **Knowledge Rewards Gameplay**: Higher understanding yields practical benefits.
- **Colonial Naturalist Aesthetic**: Entries feel like pages from a naturalist's journal.
- **Interconnected Ecosystem**: Entries reference each other, revealing ecological relationships.

### Key Features

| Feature | Description |
|---------|-------------|
| Sighting System | First observations unlock basic entries |
| Study Progress | Extended observation reveals behaviors |
| Interaction Knowledge | Different interactions unlock different facts |
| Seasonal Variations | Some knowledge only available in certain seasons |
| Native Wisdom | NPCs can share knowledge, accelerating discovery |
| Sketches & Notes | Visual journal with hand-drawn aesthetic |

---

## Discovery Mechanics

### Discovery Tiers

Each species/plant has five discovery tiers:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryTier {
    Unknown,      // Never seen - silhouette in encyclopedia
    Sighted,      // Seen once - basic info unlocked
    Observed,     // Watched for cumulative 60 seconds
    Studied,      // Multiple observations, behaviors witnessed
    Mastered,     // All knowledge unlocked, special abilities
}
```

### Discovery Actions

| Action | Progress Gained | Requirements |
|--------|-----------------|--------------|
| First Sighting | Tier 0 → 1 | See species within 50m |
| Extended Watch | +5% per 10s | Stay within 30m, undetected |
| Behavior Witnessed | +10% | See specific behavior (hunting, feeding, etc.) |
| Interaction | +15% | Feed, trap, or examine |
| Kill & Examine | +20% | Harvest the animal |
| Native Knowledge | +25% | Learn from NPC |
| Find Nest/Den | +15% | Discover home location |
| Seasonal Observation | +10% | Observe in different season |

### Detection Mechanics

Animals become aware of observation, affecting study:

```rust
pub struct ObservationState {
    pub target_id: Option<EntityId>,
    pub observation_time: f32,
    pub detected: bool,
    pub behaviors_witnessed: Vec<BehaviorType>,
    pub distance_maintained: f32,
}

impl ObservationState {
    /// Quality multiplier based on observation conditions
    pub fn quality_multiplier(&self) -> f32 {
        let mut mult = 1.0;

        // Closer observation = better quality
        if self.distance_maintained < 15.0 { mult *= 1.5; }
        else if self.distance_maintained < 25.0 { mult *= 1.2; }

        // Undetected observation = better quality
        if !self.detected { mult *= 1.5; }

        // Multiple behaviors = bonus
        mult *= 1.0 + (self.behaviors_witnessed.len() as f32 * 0.1);

        mult
    }
}
```

### Discovery Events

Special discoveries that unlock unique knowledge:

| Event | Trigger | Unlock |
|-------|---------|--------|
| Predator Hunt | Witness predator kill prey | Hunting behavior entry |
| Mating Display | Observe during breeding season | Mating rituals entry |
| Pack Dynamics | Watch wolf pack for 5+ minutes | Social hierarchy entry |
| Nocturnal Activity | Observe at night | Night behavior entry |
| Weather Response | Observe during storm | Weather patterns entry |
| Territorial Dispute | Witness two animals fight | Territory entry |
| Migration | Find animal far from usual habitat | Migration entry |
| Rare Variant | Spot albino/melanistic specimen | Rare variants entry |

---

## Encyclopedia Structure

### Main Categories

```
ENCYCLOPEDIA OF THE NEW WORLD
├── FAUNA (Animals)
│   ├── Dangerous Beasts (10 species)
│   ├── Game Animals (8 species)
│   ├── Small Creatures (12 species)
│   ├── Birds (8 species)
│   ├── Aquatic Life (6 species)
│   └── Insects & Arachnids (8 species)
│
├── FLORA (Plants)
│   ├── Trees (15 species)
│   ├── Shrubs & Bushes (12 species)
│   ├── Herbs & Medicinals (20 species)
│   ├── Flowers (15 species)
│   ├── Fungi (10 species)
│   ├── Aquatic Plants (8 species)
│   └── Crops & Cultivars (10 species)
│
├── NATURAL FEATURES
│   ├── Terrain Types
│   ├── Water Features
│   ├── Geological Formations
│   └── Weather Phenomena
│
└── NATIVE KNOWLEDGE
    ├── Tribal Lore
    ├── Traditional Uses
    └── Sacred Sites
```

### Entry Structure

Each entry contains multiple sections unlocked progressively:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncyclopediaEntry {
    pub id: String,
    pub category: EntryCategory,
    pub common_name: String,
    pub scientific_name: Option<String>,  // Unlocked at Studied tier
    pub native_name: Option<String>,       // Unlocked via NPC

    pub discovery_tier: DiscoveryTier,
    pub discovery_progress: f32,  // 0.0 - 1.0 within current tier

    pub sections: Vec<EntrySection>,
    pub illustrations: Vec<Illustration>,
    pub field_notes: Vec<FieldNote>,

    pub first_sighted: Option<GameDateTime>,
    pub sighting_locations: Vec<Vec3>,
    pub times_encountered: u32,

    // Gameplay integration
    pub practical_knowledge: PracticalKnowledge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrySection {
    pub title: String,
    pub content: String,
    pub unlock_tier: DiscoveryTier,
    pub unlock_requirement: Option<UnlockRequirement>,
    pub unlocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlockRequirement {
    ObserveBehavior(BehaviorType),
    Season(Season),
    TimeOfDay(TimeOfDay),
    WeatherCondition(WeatherType),
    NativeKnowledge,
    KillCount(u32),
    InteractionType(InteractionType),
    FindLocation(LocationType),
}
```

---

## Animal Entries

### Entry Template: Black Bear

```
╔══════════════════════════════════════════════════════════════╗
║  BLACK BEAR                                          [★★★★☆] ║
║  Ursus americanus (Unlocked at Studied)                      ║
║  "Makwa" - Algonquin (Learned from Native)                   ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  [ILLUSTRATION]          FIRST SIGHTED: Day 12, Spring      ║
║  ┌────────────────┐      ENCOUNTERS: 7                       ║
║  │    ╱▔▔▔▔╲      │      KNOWLEDGE: ████████░░ 80%           ║
║  │   ( •  • )     │                                          ║
║  │    ╲ ▽▽ ╱      │      STATUS: Territorial                 ║
║  │     ╲──╱       │      DANGER: ████████░░ 8/10             ║
║  │   ╱▔▔▔▔▔▔╲     │                                          ║
║  └────────────────┘                                          ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  DESCRIPTION (Sighted)                                       ║
║  ──────────────────                                          ║
║  A large, powerful beast covered in thick black fur. Adults  ║
║  stand as tall as a man when upright. Despite their bulk,    ║
║  they move with surprising speed and agility.                ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  HABITAT (Observed)                                          ║
║  ──────────────────                                          ║
║  Prefers dense forests with abundant berry bushes and        ║
║  hollow trees for denning. Often found near streams where    ║
║  fish are plentiful. Territory spans roughly 100 paces.      ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  BEHAVIOR (Studied)                                          ║
║  ──────────────────                                          ║
║  Primarily active at dawn and dusk. Will defend territory    ║
║  aggressively if approached. Mothers with cubs are           ║
║  especially dangerous. In autumn, feeds voraciously to       ║
║  prepare for winter dormancy.                                ║
║                                                              ║
║  ☑ Territorial warning observed (standing, roaring)          ║
║  ☑ Fishing behavior witnessed                                ║
║  ☑ Berry foraging observed                                   ║
║  ☐ Cubs with mother (Spring only)                            ║
║  ☐ Winter den located                                        ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  HUNTING NOTES (Studied)                                     ║
║  ──────────────────                                          ║
║  Weakness: Fire frightens them. Spears provide safe          ║
║  distance. Avoid direct confrontation.                       ║
║                                                              ║
║  Best hunted: Early morning when drowsy                      ║
║  Avoid: Females with cubs, cornered bears                    ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  PRACTICAL USES (Kill & Harvest)                             ║
║  ──────────────────                                          ║
║  • Pelt: Exceptional warmth, crafting material               ║
║  • Meat: 3-5 portions, rich and fatty                        ║
║  • Claws: Trophy, tool crafting                              ║
║  • Fat: Lamp oil, cooking, waterproofing                     ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  NATIVE WISDOM (Learned from Weroance)                       ║
║  ──────────────────                                          ║
║  "The bear spirit grants strength to those who respect       ║
║  the forest. Killing without need angers the spirits."       ║
║                                                              ║
║  Traditional Uses:                                           ║
║  • Bear grease mixed with herbs heals wounds                 ║
║  • Claws worn as protection talisman                         ║
║  • Never kill a bear in its den - bad fortune follows        ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  FIELD NOTES                                                 ║
║  ──────────────────                                          ║
║  [Day 12] First encounter near the river. Fled at my         ║
║  approach. Magnificent creature.                             ║
║                                                              ║
║  [Day 15] Observed fishing. They wait motionless, then       ║
║  strike with incredible speed.                               ║
║                                                              ║
║  [Day 23] Nearly mauled when I stumbled into its territory.  ║
║  Note the scratches on trees marking boundaries.             ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

### Animal Entry Data

```rust
pub fn get_animal_entry(species: AnimalSpecies) -> EncyclopediaEntry {
    match species {
        AnimalSpecies::BlackBear => EncyclopediaEntry {
            id: "black_bear".into(),
            category: EntryCategory::DangerousBeast,
            common_name: "Black Bear".into(),
            scientific_name: Some("Ursus americanus".into()),
            native_name: Some("Makwa".into()),

            sections: vec![
                EntrySection {
                    title: "Description".into(),
                    content: "A large, powerful beast covered in thick black fur...".into(),
                    unlock_tier: DiscoveryTier::Sighted,
                    unlock_requirement: None,
                    unlocked: false,
                },
                EntrySection {
                    title: "Habitat".into(),
                    content: "Prefers dense forests with abundant berry bushes...".into(),
                    unlock_tier: DiscoveryTier::Observed,
                    unlock_requirement: None,
                    unlocked: false,
                },
                EntrySection {
                    title: "Behavior".into(),
                    content: "Primarily active at dawn and dusk...".into(),
                    unlock_tier: DiscoveryTier::Studied,
                    unlock_requirement: None,
                    unlocked: false,
                },
                EntrySection {
                    title: "Hunting Notes".into(),
                    content: "Weakness: Fire frightens them...".into(),
                    unlock_tier: DiscoveryTier::Studied,
                    unlock_requirement: Some(UnlockRequirement::KillCount(1)),
                    unlocked: false,
                },
                EntrySection {
                    title: "Native Wisdom".into(),
                    content: "The bear spirit grants strength...".into(),
                    unlock_tier: DiscoveryTier::Observed,
                    unlock_requirement: Some(UnlockRequirement::NativeKnowledge),
                    unlocked: false,
                },
                EntrySection {
                    title: "Seasonal Behavior - Winter Den".into(),
                    content: "In late autumn, bears seek out caves or hollow logs...".into(),
                    unlock_tier: DiscoveryTier::Mastered,
                    unlock_requirement: Some(UnlockRequirement::FindLocation(LocationType::Den)),
                    unlocked: false,
                },
                EntrySection {
                    title: "Seasonal Behavior - Cubs".into(),
                    content: "Females emerge in spring with 1-3 cubs...".into(),
                    unlock_tier: DiscoveryTier::Mastered,
                    unlock_requirement: Some(UnlockRequirement::Season(Season::Spring)),
                    unlocked: false,
                },
            ],

            practical_knowledge: PracticalKnowledge {
                weakness_revealed: false,
                hunting_bonus: 0.0,
                detection_range_bonus: 0.0,
                loot_bonus: 0.0,
                tracking_ability: false,
            },

            ..Default::default()
        },
        // ... other species
    }
}
```

---

## Plant & Flora Entries

### Flora Categories

| Category | Count | Examples |
|----------|-------|----------|
| Trees | 15 | Oak, Pine, Birch, Cypress, Magnolia |
| Shrubs | 12 | Blueberry, Holly, Rhododendron |
| Herbs | 20 | Ginseng, Echinacea, Yarrow, Mint |
| Flowers | 15 | Trillium, Black-Eyed Susan, Cardinal Flower |
| Fungi | 10 | Chanterelle, Morel, Turkey Tail, Destroying Angel |
| Aquatic | 8 | Cattail, Water Lily, Duckweed |
| Crops | 10 | Corn, Squash, Tobacco, Beans |

### Flora Discovery

Plants have unique discovery mechanics:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloraDiscoveryMethod {
    Proximity,      // Just get close
    Harvest,        // Pick/cut the plant
    Consume,        // Eat or use
    Cultivation,    // Successfully grow
    AlchemyUse,     // Use in crafting
    NativeWisdom,   // Learn from NPC
    SeasonalFind,   // Find in specific season
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloraEntry {
    pub id: String,
    pub common_name: String,
    pub scientific_name: Option<String>,
    pub native_name: Option<String>,
    pub category: FloraCategory,

    pub discovery_tier: DiscoveryTier,
    pub discovery_progress: f32,

    // Flora-specific
    pub edibility: Edibility,
    pub medicinal_properties: Vec<MedicinalProperty>,
    pub crafting_uses: Vec<CraftingUse>,
    pub growth_conditions: GrowthConditions,
    pub harvest_seasons: Vec<Season>,
    pub habitat_types: Vec<Habitat>,

    // Poisonous plants have hidden dangers
    pub hidden_dangers: Vec<HiddenDanger>,
    pub danger_discovered: bool,

    // Visuals
    pub appearance_notes: String,
    pub illustrations: Vec<Illustration>,

    pub sections: Vec<EntrySection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edibility {
    Safe,               // Fully edible
    EdibleParts,        // Some parts edible
    EdibleProcessed,    // Edible after cooking/processing
    MedicinalOnly,      // Not food, but useful
    Inedible,           // No food value
    Poisonous,          // Harmful if consumed
    DeadlyPoisonous,    // Fatal if consumed
}

#[derive(Debug, Clone)]
pub struct MedicinalProperty {
    pub effect: MedicinalEffect,
    pub potency: f32,
    pub preparation: PreparationType,
    pub discovered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MedicinalEffect {
    Healing,
    PoisonCure,
    StaminaRestore,
    PainRelief,
    FeverReduction,
    WoundSalve,
    SnakebiteRemedy,
    SleepAid,
    Stimulant,
    Antibiotic,
}
```

### Example Flora Entry: Ginseng

```
╔══════════════════════════════════════════════════════════════╗
║  AMERICAN GINSENG                                    [★★★★★] ║
║  Panax quinquefolius                                         ║
║  "Garent oquen" - Iroquois (Man Root)                        ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  [ILLUSTRATION]          FIRST FOUND: Day 34, Summer         ║
║  ┌────────────────┐      HARVESTED: 3 times                  ║
║  │     🌿🌿🌿      │      KNOWLEDGE: ██████████ 100%          ║
║  │    🌿🌿🌿🌿🌿    │                                          ║
║  │      │         │      RARITY: ████████░░ Rare             ║
║  │      │         │      VALUE: ████████████ Precious        ║
║  │   ╰──┴──╯      │                                          ║
║  └────────────────┘                                          ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  IDENTIFICATION (Sighted)                                    ║
║  ──────────────────                                          ║
║  A low-growing plant with a single stem bearing 3-5          ║
║  compound leaves, each with 5 leaflets. In summer, produces  ║
║  small greenish-white flowers followed by bright red         ║
║  berries. The root resembles a human figure.                 ║
║                                                              ║
║  HEIGHT: 8-15 inches                                         ║
║  LEAVES: Compound, 5 leaflets each                           ║
║  FLOWERS: Greenish-white (Summer)                            ║
║  FRUIT: Red berries (Fall)                                   ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  HABITAT (Observed)                                          ║
║  ──────────────────                                          ║
║  Found in rich, shaded deciduous forests. Prefers north-     ║
║  facing slopes with deep leaf litter. Often grows near       ║
║  sugar maple, yellow poplar, and black cohosh.               ║
║                                                              ║
║  COMPANION PLANTS: Sugar Maple, Maidenhair Fern              ║
║  SOIL: Rich, loamy, well-drained                             ║
║  LIGHT: Deep shade to partial shade                          ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  MEDICINAL PROPERTIES (Harvested)                            ║
║  ──────────────────                                          ║
║  The root is the source of all medicinal value:              ║
║                                                              ║
║  ☑ Stamina Restoration (+50% stamina recovery)               ║
║  ☑ Health Regeneration (Slow heal over time)                 ║
║  ☑ Stress Relief (Reduces fear effects)                      ║
║  ☑ Longevity Tonic (Reduces hunger rate)                     ║
║                                                              ║
║  PREPARATION:                                                ║
║  • Raw root: Minor effect, bitter taste                      ║
║  • Dried root: Standard potency, long shelf life             ║
║  • Ginseng tea: Quick stamina boost                          ║
║  • Ginseng tincture: Maximum potency                         ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  NATIVE WISDOM (Learned)                                     ║
║  ──────────────────                                          ║
║  "The man-root is sacred. It hides from those who seek       ║
║  it with greed. Approach with respect, and it will           ║
║  reveal itself."                                             ║
║                                                              ║
║  Traditional Uses:                                           ║
║  • Root chewed before long journeys                          ║
║  • Given to elders to extend life                            ║
║  • Used in fertility ceremonies                              ║
║  • Never harvest all plants - leave some to reseed           ║
║                                                              ║
║  ⚠ TABOO: Harvesting ginseng carelessly brings bad luck     ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  CULTIVATION (Mastered)                                      ║
║  ──────────────────                                          ║
║  Seeds require 18 months stratification before germinating.  ║
║  Plant in fall, expect sprouts in second spring.             ║
║  Roots reach harvest maturity after 5-7 years.               ║
║                                                              ║
║  ☑ Seeds obtained from wild plant                            ║
║  ☑ Successfully germinated                                   ║
║  ☐ Harvested cultivated root (5 years growth)                ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

### Dangerous Flora

Some plants have hidden dangers only revealed through experience:

```rust
#[derive(Debug, Clone)]
pub struct HiddenDanger {
    pub danger_type: DangerType,
    pub discovery_method: DangerDiscoveryMethod,
    pub discovered: bool,
    pub warning_text: String,
    pub full_description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerType {
    ContactPoison,      // Poison ivy, poison oak
    IngestedPoison,     // Deadly nightshade, hemlock
    DelayedPoison,      // Effects appear hours later
    Hallucinogen,       // Causes confusion
    Skin irritant,      // Causes rash
    Thorns,             // Physical damage
    AllergyRisk,        // May cause severe reaction
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerDiscoveryMethod {
    PersonalExperience,    // You got hurt
    WitnessedEffect,       // Saw someone else get hurt
    NativeWarning,         // NPC told you
    BookKnowledge,         // Read about it
    AnimalBehavior,        // Animals avoid it
}
```

---

## Environmental Features

### Terrain & Geological Entries

```rust
pub enum TerrainEntry {
    // Water Features
    River,
    Stream,
    Waterfall,
    Lake,
    Pond,
    Swamp,
    Marsh,
    Spring,

    // Geological
    Cave,
    Cliff,
    RockyOutcrop,
    Boulder,
    Ravine,
    Valley,

    // Soil Types
    Loam,
    Clay,
    Sandy,
    Rocky,
    Peat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainFeatureEntry {
    pub id: TerrainEntry,
    pub name: String,
    pub description: String,

    // Practical information
    pub associated_resources: Vec<ResourceType>,
    pub associated_flora: Vec<String>,
    pub associated_fauna: Vec<String>,
    pub dangers: Vec<String>,
    pub shelter_value: f32,
    pub water_source: bool,

    // Discovery
    pub discovered_locations: Vec<Vec3>,
    pub discovery_tier: DiscoveryTier,
}
```

### Weather Phenomena Entries

```rust
pub enum WeatherPhenomenonEntry {
    // Common
    Rain,
    Thunderstorm,
    Fog,
    Wind,
    Snow,
    Frost,

    // Severe
    Hurricane,
    Tornado,
    Flood,
    Drought,
    Blizzard,

    // Rare
    AuroraBorealis,
    Earthquake,
    Eclipse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherEntry {
    pub id: WeatherPhenomenonEntry,
    pub name: String,
    pub description: String,

    pub survival_tips: Vec<String>,
    pub danger_level: u8,
    pub seasonal_likelihood: HashMap<Season, f32>,

    pub witnessed_count: u32,
    pub first_witnessed: Option<GameDateTime>,

    // Hidden knowledge
    pub native_interpretation: Option<String>,
    pub omen_meaning: Option<String>,
    pub discovered_meanings: bool,
}
```

---

## Knowledge Progression

### Tier Bonuses

Each discovery tier provides gameplay benefits:

```rust
impl DiscoveryTier {
    pub fn get_bonuses(&self) -> TierBonuses {
        match self {
            DiscoveryTier::Unknown => TierBonuses {
                tracking_range: 0.0,
                damage_bonus: 0.0,
                detection_warning: false,
                weakness_known: false,
                loot_bonus: 0.0,
                behavior_prediction: false,
            },
            DiscoveryTier::Sighted => TierBonuses {
                tracking_range: 10.0,
                damage_bonus: 0.0,
                detection_warning: false,
                weakness_known: false,
                loot_bonus: 0.0,
                behavior_prediction: false,
            },
            DiscoveryTier::Observed => TierBonuses {
                tracking_range: 20.0,
                damage_bonus: 0.05,
                detection_warning: true,  // Know when it spots you
                weakness_known: false,
                loot_bonus: 0.1,
                behavior_prediction: false,
            },
            DiscoveryTier::Studied => TierBonuses {
                tracking_range: 35.0,
                damage_bonus: 0.15,
                detection_warning: true,
                weakness_known: true,     // Weakness revealed
                loot_bonus: 0.25,
                behavior_prediction: true, // Know attack patterns
            },
            DiscoveryTier::Mastered => TierBonuses {
                tracking_range: 50.0,
                damage_bonus: 0.25,
                detection_warning: true,
                weakness_known: true,
                loot_bonus: 0.5,
                behavior_prediction: true,
                // Plus unique mastery ability
            },
        }
    }
}
```

### Mastery Abilities

Each species has a unique mastery ability:

| Species | Mastery Ability |
|---------|-----------------|
| Black Bear | "Bear Caller" - Can lure bears with special bait |
| Gray Wolf | "Pack Sense" - Always know pack size and alpha location |
| Eastern Cougar | "Stalker's Instinct" - Cannot be ambushed by cougars |
| Alligator | "Swamp Walker" - Know safe paths through gator territory |
| White-tailed Deer | "Silent Stalker" - 50% less detection when hunting deer |
| Wild Turkey | "Turkey Whisperer" - Can call turkeys to your location |
| Ginseng | "Root Finder" - Ginseng glows faintly in your vision |
| Chanterelle | "Mushroom Eye" - All edible fungi highlighted |

---

## Field Notes System

### Automatic Field Notes

The game automatically generates field notes based on observations:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldNote {
    pub entry_id: String,
    pub timestamp: GameDateTime,
    pub location: Vec3,
    pub weather: WeatherType,
    pub time_of_day: TimeOfDay,
    pub note_type: FieldNoteType,
    pub content: String,
    pub associated_sketch: Option<SketchId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldNoteType {
    FirstSighting,
    BehaviorObserved,
    InteractionRecord,
    KillRecord,
    SeasonalNote,
    WeatherNote,
    NativeKnowledge,
    PersonalObservation,
    DangerWarning,
}

impl FieldNote {
    pub fn generate_sighting(species: &str, distance: f32, time: TimeOfDay) -> Self {
        let content = format!(
            "First sighted {} at approximately {} paces distance during {}. {}",
            species,
            distance as i32,
            time.description(),
            Self::random_observation(species)
        );

        FieldNote {
            note_type: FieldNoteType::FirstSighting,
            content,
            ..Default::default()
        }
    }

    fn random_observation(species: &str) -> &'static str {
        // Pool of naturalist-style observations
        match rand::thread_rng().gen_range(0..5) {
            0 => "A magnificent specimen.",
            1 => "Appeared healthy and alert.",
            2 => "Seemed undisturbed by my presence.",
            3 => "Fled upon noticing my approach.",
            4 => "Worthy of further study.",
            _ => "",
        }
    }
}
```

### Custom Notes

Players can add their own notes:

```rust
pub struct CustomNote {
    pub entry_id: String,
    pub content: String,  // Player-written
    pub timestamp: GameDateTime,
    pub location: Option<Vec3>,
    pub tags: Vec<String>,
}
```

---

## Naturalist Skill Tree

### Skill Tree Structure

```
                           [MASTER NATURALIST]
                                   |
                  +----------------+----------------+
                  |                                 |
           [Botanical Expert]              [Zoological Expert]
                  |                                 |
         +--------+--------+              +--------+--------+
         |                 |              |                 |
    [Herbalist]     [Mycologist]    [Ornithologist]  [Mammalogist]
         |                 |              |                 |
         +--------+--------+              +--------+--------+
                  |                                 |
           [Plant Identifier]              [Animal Observer]
                  |                                 |
                  +----------------+----------------+
                                   |
                           [CURIOUS MIND]
                            (Starting Point)
```

### Skill Definitions

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalistSkills {
    // Tier 1
    pub curious_mind: bool,           // Starting skill

    // Tier 2
    pub plant_identifier: bool,       // +25% flora discovery speed
    pub animal_observer: bool,        // +25% fauna discovery speed

    // Tier 3
    pub herbalist: bool,              // Unlock medicinal properties faster
    pub mycologist: bool,             // Identify fungi, avoid poison
    pub ornithologist: bool,          // Bird behaviors fully revealed
    pub mammalogist: bool,            // Mammal behaviors fully revealed

    // Tier 4
    pub botanical_expert: bool,       // All plant uses known immediately
    pub zoological_expert: bool,      // All animal weaknesses known

    // Tier 5
    pub master_naturalist: bool,      // Unique abilities

    // Tracking
    pub discovery_count: u32,
    pub mastered_species: u32,
    pub field_notes_written: u32,
}

impl NaturalistSkills {
    pub fn discovery_speed_modifier(&self) -> f32 {
        let mut modifier = 1.0;
        if self.curious_mind { modifier += 0.1; }
        if self.plant_identifier { modifier += 0.25; }  // For plants
        if self.animal_observer { modifier += 0.25; }   // For animals
        if self.botanical_expert { modifier += 0.5; }   // For plants
        if self.zoological_expert { modifier += 0.5; }  // For animals
        if self.master_naturalist { modifier += 1.0; }
        modifier
    }
}
```

---

## Integration with Other Systems

### Hunting Skill Tree Integration

```rust
/// Knowledge bonuses applied during hunting
pub fn apply_knowledge_to_hunting(
    species: AnimalSpecies,
    knowledge: &EncyclopediaEntry,
    hunting_skills: &HuntingSkills,
) -> HuntingModifiers {
    let tier = knowledge.discovery_tier;
    let tier_bonuses = tier.get_bonuses();

    HuntingModifiers {
        damage_bonus: tier_bonuses.damage_bonus
            + if hunting_skills.beast_slayer { 0.5 } else { 0.0 },

        tracking_range: tier_bonuses.tracking_range
            + if hunting_skills.wilderness_scout { 20.0 } else { 0.0 },

        stealth_bonus: if tier >= DiscoveryTier::Studied { 0.2 } else { 0.0 },

        loot_multiplier: 1.0 + tier_bonuses.loot_bonus
            + if hunting_skills.prey_instinct { 0.5 } else { 0.0 },

        weakness_damage: if tier_bonuses.weakness_known { 1.5 } else { 1.0 },
    }
}
```

### Nature Morality Integration

Encyclopedia knowledge affects morality system:

```rust
/// Actions that affect nature morality based on knowledge
pub fn calculate_morality_impact(
    action: NatureAction,
    knowledge: &EncyclopediaEntry,
) -> f32 {
    let base_impact = action.base_morality_impact();

    // Ignorance is partially excused
    let knowledge_modifier = match knowledge.discovery_tier {
        DiscoveryTier::Unknown => 0.3,   // Didn't know better
        DiscoveryTier::Sighted => 0.5,   // Should have known
        DiscoveryTier::Observed => 0.75,  // Knew it was wrong
        DiscoveryTier::Studied => 1.0,   // Full responsibility
        DiscoveryTier::Mastered => 1.25, // Expert should know better
    };

    base_impact * knowledge_modifier
}
```

### Weather System Integration

Weather affects discovery opportunities:

```rust
/// Weather-based discovery modifiers
pub fn get_weather_discovery_mods(weather: WeatherType) -> DiscoveryMods {
    match weather {
        WeatherType::Clear => DiscoveryMods {
            observation_quality: 1.2,
            animal_activity: 1.0,
            plant_visibility: 1.0,
        },
        WeatherType::Stormy => DiscoveryMods {
            observation_quality: 0.5,
            animal_activity: 0.3,  // Animals hide
            plant_visibility: 0.8,
            // But storm behaviors observable
            storm_behaviors_visible: true,
        },
        WeatherType::Foggy => DiscoveryMods {
            observation_quality: 0.7,
            animal_activity: 0.8,
            plant_visibility: 0.6,
            // Nocturnal animals more active in fog
            nocturnal_bonus: 1.5,
        },
        // ...
    }
}
```

---

## Data Structures

### Core Types

```rust
// encyclopedia/mod.rs

pub mod entries;
pub mod discovery;
pub mod field_notes;
pub mod illustrations;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Main encyclopedia manager
#[derive(Debug, Serialize, Deserialize)]
pub struct Encyclopedia {
    pub entries: HashMap<String, EncyclopediaEntry>,
    pub flora_entries: HashMap<String, FloraEntry>,
    pub terrain_entries: HashMap<String, TerrainFeatureEntry>,
    pub weather_entries: HashMap<String, WeatherEntry>,

    pub field_notes: Vec<FieldNote>,
    pub custom_notes: Vec<CustomNote>,
    pub illustrations: Vec<Illustration>,

    // Statistics
    pub total_discoveries: u32,
    pub species_mastered: u32,
    pub plants_mastered: u32,
    pub unique_behaviors_witnessed: u32,
    pub rarest_find: Option<String>,
    pub first_discovery: Option<GameDateTime>,

    // Active observation
    pub active_observation: Option<ObservationState>,
}

impl Encyclopedia {
    pub fn new() -> Self {
        let mut enc = Self::default();
        enc.initialize_entries();
        enc
    }

    /// Initialize all entries as Unknown
    fn initialize_entries(&mut self) {
        // Add all animal entries
        for species in AnimalSpecies::iter() {
            let entry = get_animal_entry(species);
            self.entries.insert(entry.id.clone(), entry);
        }

        // Add all docile fauna
        for species in DocileSpecies::iter() {
            let entry = get_docile_entry(species);
            self.entries.insert(entry.id.clone(), entry);
        }

        // Add all flora
        for plant in FloraSpecies::iter() {
            let entry = get_flora_entry(plant);
            self.flora_entries.insert(entry.id.clone(), entry);
        }
    }

    /// Process a sighting event
    pub fn on_sighting(&mut self, entity_id: &str, distance: f32, time: TimeOfDay) {
        if let Some(entry) = self.entries.get_mut(entity_id) {
            if entry.discovery_tier == DiscoveryTier::Unknown {
                entry.discovery_tier = DiscoveryTier::Sighted;
                entry.first_sighted = Some(GameDateTime::now());
                entry.unlock_tier_sections(DiscoveryTier::Sighted);

                // Generate field note
                let note = FieldNote::generate_sighting(&entry.common_name, distance, time);
                self.field_notes.push(note);

                self.total_discoveries += 1;
            }
            entry.times_encountered += 1;
        }
    }

    /// Process observation time
    pub fn on_observation(&mut self, entity_id: &str, duration: f32, quality: f32) {
        if let Some(entry) = self.entries.get_mut(entity_id) {
            let progress_gain = (duration / 60.0) * quality;
            entry.discovery_progress += progress_gain;

            // Check tier advancement
            if entry.discovery_progress >= 1.0 {
                entry.advance_tier();
            }
        }
    }

    /// Get discovery percentage for category
    pub fn category_completion(&self, category: EntryCategory) -> f32 {
        let entries: Vec<_> = self.entries.values()
            .filter(|e| e.category == category)
            .collect();

        if entries.is_empty() { return 0.0; }

        let discovered = entries.iter()
            .filter(|e| e.discovery_tier > DiscoveryTier::Unknown)
            .count();

        discovered as f32 / entries.len() as f32
    }
}
```

### Save/Load

```rust
#[derive(Serialize, Deserialize)]
pub struct EncyclopediaSaveData {
    pub entries: HashMap<String, EntrySaveData>,
    pub flora_entries: HashMap<String, FloraSaveData>,
    pub field_notes: Vec<FieldNote>,
    pub custom_notes: Vec<CustomNote>,
    pub stats: EncyclopediaStats,
}

#[derive(Serialize, Deserialize)]
pub struct EntrySaveData {
    pub id: String,
    pub discovery_tier: DiscoveryTier,
    pub discovery_progress: f32,
    pub first_sighted: Option<GameDateTime>,
    pub times_encountered: u32,
    pub sighting_locations: Vec<[f32; 3]>,
    pub unlocked_sections: Vec<String>,
    pub behaviors_witnessed: Vec<BehaviorType>,
}
```

---

## UI/UX Design

### Encyclopedia Menu

```
┌─────────────────────────────────────────────────────────────────┐
│  ENCYCLOPEDIA OF THE NEW WORLD                        [X] Close │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌─────────────────────────────────────────┐ │
│  │  CATEGORIES  │  │                                         │ │
│  ├──────────────┤  │  BLACK BEAR                             │ │
│  │ ▶ FAUNA      │  │  ═══════════                            │ │
│  │   Dangerous  │  │                                         │ │
│  │   Game       │  │  [Illustration]     First Seen: Day 12  │ │
│  │   Small      │  │                     Encounters: 7       │ │
│  │   Birds      │  │                     Knowledge: 80%      │ │
│  │   Aquatic    │  │                                         │ │
│  │   Insects    │  │  Description                            │ │
│  │              │  │  ───────────                            │ │
│  │ ▶ FLORA      │  │  A large, powerful beast covered in     │ │
│  │   Trees      │  │  thick black fur...                     │ │
│  │   Shrubs     │  │                                         │ │
│  │   Herbs      │  │  [More sections...]                     │ │
│  │   Flowers    │  │                                         │ │
│  │   Fungi      │  │                                         │ │
│  │              │  │                                         │ │
│  │ ▶ FEATURES   │  │  ┌───────────────────────────────────┐ │ │
│  │              │  │  │ FIELD NOTES                       │ │ │
│  │ ▶ WEATHER    │  │  ├───────────────────────────────────┤ │ │
│  │              │  │  │ Day 12: First encounter near...   │ │ │
│  │ ▶ NATIVE     │  │  │ Day 15: Observed fishing...       │ │ │
│  │   KNOWLEDGE  │  │  │ Day 23: Nearly mauled when...     │ │ │
│  └──────────────┘  │  └───────────────────────────────────┘ │ │
│                    └─────────────────────────────────────────┘ │
│                                                                 │
│  [◀ Previous]  [1] [2] [3] [4] [5]  [Next ▶]                   │
└─────────────────────────────────────────────────────────────────┘
```

### Discovery Notification

```
┌─────────────────────────────────────────┐
│ 🔍 NEW DISCOVERY                        │
│                                         │
│ [Illustration]  WHITE-TAILED DEER       │
│                 Discovered!             │
│                                         │
│ "A graceful creature with a distinctive │
│  white tail raised when alarmed..."     │
│                                         │
│ Press [J] to view Encyclopedia          │
└─────────────────────────────────────────┘
```

### Observation HUD

```
┌─────────────────────────────────┐
│ 👁 OBSERVING: Gray Wolf         │
│ ══════════════════════════════  │
│ Distance: 25m                   │
│ Status: Unaware                 │
│                                 │
│ Knowledge: ████████░░ 73%       │
│ Quality: ★★★☆☆ (Good)           │
│                                 │
│ [Keep still for better quality] │
└─────────────────────────────────┘
```

---

## Implementation Priority

### Phase 1: Core System
- [ ] Encyclopedia data structures
- [ ] Entry initialization for all species
- [ ] Basic discovery (sighting only)
- [ ] Encyclopedia UI menu

### Phase 2: Observation System
- [ ] Observation state tracking
- [ ] Tier progression logic
- [ ] Quality calculation
- [ ] Behavior witnessing

### Phase 3: Knowledge Benefits
- [ ] Tier bonus application
- [ ] Hunting integration
- [ ] Weakness revelation
- [ ] Tracking improvements

### Phase 4: Field Notes
- [ ] Automatic note generation
- [ ] Custom note writing
- [ ] Note display in entries

### Phase 5: Flora System
- [ ] Flora entries
- [ ] Medicinal properties
- [ ] Danger discovery
- [ ] Cultivation tracking

### Phase 6: Polish
- [ ] Illustration system
- [ ] Native knowledge integration
- [ ] Weather entries
- [ ] Achievement system

---

*This encyclopedia serves as both gameplay mechanic and worldbuilding device, encouraging players to truly understand the New World they inhabit.*
