# Animal Database, Universal Taming & Entomology System Specification

## Roanoke Engine - Creature Management Framework

This document specifies the database schema for animal data persistence, a universal taming system covering all species, entomology with insect collection, beekeeping mechanics, and firefly jar light sources.

---

## Table of Contents

1. [Overview](#overview)
2. [Database Schema](#database-schema)
3. [Universal Taming System](#universal-taming-system)
4. [Companion System](#companion-system)
5. [Entomology System](#entomology-system)
6. [Beekeeping System](#beekeeping-system)
7. [Firefly Jar System](#firefly-jar-system)
8. [Skill Tree Extensions](#skill-tree-extensions)
9. [Implementation Phases](#implementation-phases)

---

## Overview

### Design Goals

- **Unified Animal Data:** Single database schema for all creature relationships, taming progress, and breeding lineages
- **Universal Taming:** Consistent framework with species-specific variations
- **Period Authenticity:** 1580s Carolina-appropriate insects and beekeeping methods
- **Emergent Gameplay:** Interconnected systems (entomology feeds apiary, fireflies provide light)

### Database Choice: SQLite

**Rationale:**
- Embedded (no external server)
- Single-file saves (portable)
- ACID compliant for save integrity
- Relational for Player ↔ Animal ↔ Species relationships
- Rust support via `rusqlite`

---

## Database Schema

### Entity Relationship Diagram

```
┌─────────────────┐       ┌──────────────────┐
│ animal_         │       │ tamed_           │
│ relationships   │──────>│ animals          │
├─────────────────┤       ├──────────────────┤
│ id              │       │ id               │
│ species         │       │ relationship_id  │
│ relationship_   │       │ species          │
│   type          │       │ unique_name      │
│ trust_level     │       │ health/stats     │
│ bond_level      │       │ loyalty          │
│ taming_phase    │       │ trained_skills   │
│ taming_progress │       │ generation       │
└─────────────────┘       │ parent1/2_id     │
         │                └──────────────────┘
         │                         │
         v                         v
┌─────────────────┐       ┌──────────────────┐
│ animal_         │       │ breeding_        │
│ memories        │       │ lineage          │
├─────────────────┤       ├──────────────────┤
│ relationship_id │       │ offspring_id     │
│ memory_type     │       │ parent1/2_id     │
│ impact          │       │ generation       │
│ game_time       │       │ inherited_traits │
└─────────────────┘       └──────────────────┘

┌─────────────────┐       ┌──────────────────┐
│ insect_         │       │ insect_jars      │
│ collection      │       ├──────────────────┤
├─────────────────┤       │ jar_slot         │
│ species         │       │ species          │
│ quantity        │       │ quantity         │
│ discovery_tier  │       │ light_intensity  │
│ total_caught    │       │ air_quality      │
└─────────────────┘       └──────────────────┘

┌─────────────────┐       ┌──────────────────┐
│ apiaries        │       │ bee_species      │
├─────────────────┤       ├──────────────────┤
│ position        │       │ species          │
│ hive_type       │       │ honey_production │
│ bee_count       │       │ aggression       │
│ honey_stored    │       │ disease_resist   │
│ colony_strength │       └──────────────────┘
│ aggression_lvl  │
└─────────────────┘
```

### Table Definitions

```sql
-- =============================================
-- ANIMAL RELATIONSHIPS
-- =============================================

CREATE TABLE animal_relationships (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    species TEXT NOT NULL,           -- AnimalSpecies enum name
    entity_id INTEGER,               -- Runtime entity ID (nullable)
    relationship_type TEXT NOT NULL, -- 'wild', 'taming', 'tamed', 'companion', 'bred'

    -- Metrics (0-100 scale)
    trust_level INTEGER DEFAULT 0,
    respect_level INTEGER DEFAULT 0,
    fear_level INTEGER DEFAULT 0,
    bond_level INTEGER DEFAULT 0,

    -- Taming progress
    taming_phase TEXT,               -- Current phase
    taming_progress REAL DEFAULT 0.0,-- 0.0-1.0 within phase
    total_taming_time REAL DEFAULT 0.0,

    -- Interaction history
    total_interactions INTEGER DEFAULT 0,
    positive_interactions INTEGER DEFAULT 0,
    negative_interactions INTEGER DEFAULT 0,
    last_interaction_time REAL,

    -- Timestamps
    first_encounter_time REAL,
    tamed_time REAL,
    unique_name TEXT,

    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- =============================================
-- TAMED ANIMAL INSTANCES
-- =============================================

CREATE TABLE tamed_animals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    relationship_id INTEGER NOT NULL REFERENCES animal_relationships(id),
    species TEXT NOT NULL,
    unique_name TEXT NOT NULL,

    -- Stats
    health REAL NOT NULL,
    max_health REAL NOT NULL,
    stamina REAL DEFAULT 100.0,
    hunger REAL DEFAULT 0.0,

    -- Combat/Utility
    damage REAL DEFAULT 10.0,
    speed REAL DEFAULT 30.0,
    defense REAL DEFAULT 0.0,

    -- Behavior
    loyalty REAL DEFAULT 0.5,
    obedience REAL DEFAULT 0.5,
    aggression REAL DEFAULT 0.5,

    -- Skills (JSON: {"tracking": 50, "hunting": 30})
    trained_skills TEXT DEFAULT '{}',

    -- Appearance
    coat_variant TEXT,
    size_modifier REAL DEFAULT 1.0,

    -- Breeding
    generation INTEGER DEFAULT 0,    -- 0 = wild-caught
    can_breed INTEGER DEFAULT 1,
    breeding_cooldown REAL DEFAULT 0.0,
    times_bred INTEGER DEFAULT 0,
    parent1_id INTEGER REFERENCES tamed_animals(id),
    parent2_id INTEGER REFERENCES tamed_animals(id),

    -- State
    current_state TEXT DEFAULT 'following',
    home_position TEXT,              -- JSON [x, y, z]

    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- =============================================
-- BREEDING LINEAGE
-- =============================================

CREATE TABLE breeding_lineage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    offspring_id INTEGER NOT NULL REFERENCES tamed_animals(id),
    parent1_id INTEGER NOT NULL REFERENCES tamed_animals(id),
    parent2_id INTEGER REFERENCES tamed_animals(id),
    generation INTEGER NOT NULL,
    breeding_time REAL NOT NULL,
    inherited_traits TEXT,           -- JSON
    mutation_occurred INTEGER DEFAULT 0,
    mutation_type TEXT
);

-- =============================================
-- ANIMAL MEMORIES
-- =============================================

CREATE TABLE animal_memories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    relationship_id INTEGER NOT NULL REFERENCES animal_relationships(id),
    memory_type TEXT NOT NULL,       -- 'positive', 'negative', 'feeding', 'combat'
    description TEXT,
    impact INTEGER DEFAULT 0,        -- -100 to +100
    game_time REAL NOT NULL,
    position TEXT                    -- JSON [x, y, z]
);

-- =============================================
-- DISCOVERED SPECIES
-- =============================================

CREATE TABLE discovered_species (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    species_type TEXT NOT NULL,      -- 'animal', 'insect', 'bird'
    species_name TEXT NOT NULL UNIQUE,
    discovery_tier TEXT DEFAULT 'Unknown',
    observation_time REAL DEFAULT 0.0,
    sightings INTEGER DEFAULT 0,
    behaviors_witnessed TEXT,        -- JSON array
    first_discovered_time REAL,
    mastery_achieved_time REAL
);

-- =============================================
-- ENTOMOLOGY
-- =============================================

CREATE TABLE insect_collection (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    species TEXT NOT NULL UNIQUE,
    quantity INTEGER DEFAULT 0,
    quality_average REAL DEFAULT 1.0,
    discovery_tier TEXT DEFAULT 'Unknown',
    total_caught INTEGER DEFAULT 0,
    total_released INTEGER DEFAULT 0,
    first_caught_time REAL,
    habitats_found TEXT,             -- JSON array
    seasons_found TEXT               -- JSON array
);

CREATE TABLE insect_jars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    jar_slot INTEGER NOT NULL,
    jar_type TEXT DEFAULT 'basic',
    species TEXT,
    quantity INTEGER DEFAULT 0,
    max_capacity INTEGER DEFAULT 5,
    lid_closed INTEGER DEFAULT 1,
    air_quality REAL DEFAULT 1.0,
    light_intensity REAL DEFAULT 0.0,
    bioluminescence_remaining REAL DEFAULT 0.0
);

-- =============================================
-- BEEKEEPING
-- =============================================

CREATE TABLE apiaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    position TEXT NOT NULL,          -- JSON [x, y, z]
    hive_type TEXT DEFAULT 'wild',

    -- Colony
    bee_species TEXT DEFAULT 'EuropeanHoneybee',
    queen_present INTEGER DEFAULT 1,
    colony_strength REAL DEFAULT 1.0,
    bee_count INTEGER DEFAULT 0,
    max_bees INTEGER DEFAULT 5000,

    -- Production
    honey_stored REAL DEFAULT 0.0,
    wax_stored REAL DEFAULT 0.0,
    propolis_stored REAL DEFAULT 0.0,

    -- Health
    disease_level REAL DEFAULT 0.0,
    mite_level REAL DEFAULT 0.0,

    -- Behavior
    aggression_level REAL DEFAULT 0.5,
    foraging_radius REAL DEFAULT 50.0,

    -- State
    last_harvest_time REAL,
    winterized INTEGER DEFAULT 0,
    established_time REAL
);

CREATE TABLE wild_hives (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    position TEXT NOT NULL,
    bee_species TEXT,
    colony_size INTEGER,
    honey_amount REAL,
    discovered INTEGER DEFAULT 0,
    captured INTEGER DEFAULT 0,
    discovery_time REAL
);
```

---

## Universal Taming System

### Taming Categories

| Category | Species | Primary Method | Phases |
|----------|---------|----------------|--------|
| **Canine** | GrayWolf, RedWolf, Husky, Fox | Feed + patience | 5 |
| **Equine** | Horse, Donkey | Multi-phase training | 10 |
| **Avian** | RingNeckedPheasant, future birds | Capture + cage | 4 |
| **Porcine** | WildBoar (future) | Pen + food | 3 |
| **Feline** | Bobcat (future) | Master hunter only | 6 |

### Taming Requirements by Species

| Species | Naturalist Score | Hunting Tier | Discoveries | Required Items |
|---------|-----------------|--------------|-------------|----------------|
| GrayWolf | 25 | 3 | 10 | meat |
| RedWolf | 35 | 4 | 15 | meat, wolf_bait |
| Husky | 20 | 2 | 8 | meat |
| Fox | 40 | 5 | 20 | small_game |
| Horse | 15 | 2 | 5 | halter, saddle |
| Donkey | 10 | 1 | 3 | halter |
| RingNeckedPheasant | 5 | 1 | 2 | bird_seed, bird_cage |

### Canine Taming Phases

```
1. AWARENESS (threshold: 0.3)
   - Animal notices player
   - Actions: Wait, Crouch

2. APPROACH (threshold: 0.5)
   - Reduce flight distance
   - Actions: Crouch, SpeakSoftly, Wait

3. CALMING (threshold: 0.6)
   - Reduce fear response
   - Actions: SpeakSoftly, ShowHands, Wait

4. FEEDING (threshold: 0.8)
   - First food acceptance
   - Actions: ThrowBait, Feed, OfferFood

5. BONDED (threshold: 1.0)
   - Full trust established
   - Actions: Touch, Lead, Name
```

### Equine Taming Phases (Existing System)

```
1. Awareness     → 0.5 threshold
2. Approach      → 0.6 threshold
3. Calming       → 0.7 threshold
4. Touch         → 0.6 threshold
5. Haltering     → 0.5 threshold
6. GroundWork    → 0.8 threshold
7. Saddling      → 0.6 threshold
8. Mounting      → 0.7 threshold
9. Riding        → 0.9 threshold
10. Bonded       → 1.0 complete
```

### Avian Taming Phases

```
1. CAPTURE (threshold: 0.3)
   - Net/trap the bird
   - Actions: ThrowNet, SetTrap

2. CAGING (threshold: 0.5)
   - Acclimate to cage
   - Actions: ProvideCage, CoverCage

3. HAND_FEEDING (threshold: 0.8)
   - Accept food from hand
   - Actions: OfferSeed, Wait, SpeakSoftly

4. PERCH_TRAINING (threshold: 1.0)
   - Perch on hand/shoulder
   - Actions: OfferPerch, PerchCommand
```

### Taming Actions

| Action | Base Progress | Distance | Category |
|--------|---------------|----------|----------|
| Feed | 0.15 | 5m | All |
| ThrowBait | 0.12 | 12m | Canine |
| Whistle | 0.08 | 20m | Canine/Equine |
| Crouch | 0.05 | 15m | All |
| Wait | 0.02 | 10m | All |
| SpeakSoftly | 0.06 | 8m | All |
| ShowHands | 0.04 | 6m | Canine |
| Touch | 0.10 | 2m | Post-calming |
| PlaceHalter | 0.15 | 1m | Equine |
| ThrowNet | 0.20 | 8m | Avian |
| OfferSeed | 0.12 | 3m | Avian |

---

## Companion System

### Companion States

```
Following    - Default, follows player
Guarding     - Stays at location, alerts to threats
Hunting      - Pursues small game (canine)
Attacking    - Combat engagement
Resting      - Recovers stamina
Playing      - Idle animation
Mounted      - Player riding (equine)
Working      - Carrying cargo (donkey)
Perched      - On player shoulder (avian)
Caged        - In bird cage (avian)
```

### Companion Abilities by Bond Level

| Species | 20% Bond | 40% Bond | 60% Bond | 80% Bond |
|---------|----------|----------|----------|----------|
| **Wolf** | Guard alert | Track | Hunt small game | Fight alongside |
| **Red Wolf** | Guard alert | Track | Hunt medium game | Fight alongside |
| **Husky** | Guard alert | Carry small load | Track | Pull sled |
| **Fox** | Scout | Distract enemy | Hunt vermin | Retrieve items |
| **Horse** | Mount (walk) | Mount (trot/canter) | Carry cargo | Jump obstacles |
| **Donkey** | Carry light load | Carry medium load | Carry heavy load | Pull cart |
| **Pheasant** | Perch | Scout (short) | Distract enemy | Retrieve small items |

### Companion Commands

```rust
enum CompanionCommand {
    // Universal
    Follow,
    Stay,
    Guard,
    Rest,
    Dismiss,

    // Canine
    Hunt,
    Attack,
    Fetch,
    Track { target: TrackTarget },
    Heel,

    // Equine
    Mount,
    Dismount,
    Carry { item: ItemId },
    Wait,

    // Avian
    Scout { direction: Vec3 },
    Perch,
    Cage,
    Release,
    Retrieve { item: ItemId },
}
```

---

## Entomology System

### Insect Species (50 Total)

#### Butterflies & Moths (10)

| Species | Scientific Name | Rarity | Active Time | Habitat |
|---------|-----------------|--------|-------------|---------|
| Eastern Tiger Swallowtail | *Papilio glaucus* | Common | Day | Forest edge, meadow |
| Monarch Butterfly | *Danaus plexippus* | Common | Day | Meadow, milkweed |
| Painted Lady | *Vanessa cardui* | Common | Day | Fields, gardens |
| Common Buckeye | *Junonia coenia* | Common | Day | Open areas |
| Black Swallowtail | *Papilio polyxenes* | Uncommon | Day | Meadow, garden |
| Cabbage White | *Pieris rapae* | Very Common | Day | Fields, gardens |
| Luna Moth | *Actias luna* | Rare | Night | Deciduous forest |
| Cecropia Moth | *Hyalophora cecropia* | Rare | Night | Forest |
| Io Moth | *Automeris io* | Uncommon | Night | Forest, meadow |
| Polyphemus Moth | *Antheraea polyphemus* | Uncommon | Night | Forest |

#### Beetles (10)

| Species | Scientific Name | Rarity | Active Time | Special |
|---------|-----------------|--------|-------------|---------|
| Eastern Hercules Beetle | *Dynastes tityus* | Rare | Night | Largest NA beetle |
| June Beetle | *Phyllophaga spp.* | Common | Night | Attracted to light |
| Firefly | *Photinus pyralis* | Common | Dusk/Night | **Bioluminescent** |
| Fire Beetle | *Pyrophorus noctilucus* | Rare | Night | **Bioluminescent** |
| Ladybug | *Coccinellidae spp.* | Very Common | Day | Beneficial |
| Ten-Lined June Beetle | *Polyphylla decemlineata* | Uncommon | Night | Large |
| Dung Beetle | *Scarabaeidae spp.* | Common | Day | Rolls dung |
| Long-Horned Beetle | *Cerambycidae spp.* | Uncommon | Day | In dead wood |
| Tiger Beetle | *Cicindela spp.* | Common | Day | Very fast |
| Strawberry Sap Beetle | *Stelidota geminata* | Common | Day | Near fruit |

#### Bees & Wasps (8)

| Species | Scientific Name | Aggression | Keepable | Notes |
|---------|-----------------|------------|----------|-------|
| European Honeybee | *Apis mellifera* | Medium | **Yes** | Best honey |
| Eastern Bumblebee | *Bombus impatiens* | Low | **Yes** | Good pollinator |
| Carpenter Bee | *Xylocopa virginica* | Very Low | No | Damages wood |
| Sweat Bee | *Halictidae spp.* | Very Low | No | Attracted to sweat |
| Mud Dauber Wasp | *Sceliphron caementarium* | Low | No | Builds mud nests |
| Paper Wasp | *Polistes spp.* | Medium | No | Paper nests |
| Bald-Faced Hornet | *Dolichovespula maculata* | High | No | Large paper nests |
| Yellow Jacket | *Vespula spp.* | High | No | Ground nests |

#### Dragonflies & Damselflies (6)

| Species | Scientific Name | Rarity | Catch Difficulty |
|---------|-----------------|--------|------------------|
| Common Green Darner | *Anax junius* | Common | Hard |
| Eastern Pondhawk | *Erythemis simplicicollis* | Common | Medium |
| Blue Dasher | *Pachydiplax longipennis* | Common | Medium |
| Ebony Jewelwing | *Calopteryx maculata* | Uncommon | Easy |
| American Rubyspot | *Hetaerina americana* | Uncommon | Easy |
| Common Whitetail | *Plathemis lydia* | Common | Medium |

#### Other Insects (10)

| Species | Scientific Name | Notes |
|---------|-----------------|-------|
| Periodical Cicada | *Magicicada septendecim* | 17-year cycle |
| Dog Day Cicada | *Neotibicen canicularis* | Annual, loud |
| Praying Mantis | *Mantis religiosa* | Predator |
| Grasshopper | *Melanoplus spp.* | Jumping |
| Katydid | *Pterophylla camellifolia* | Night singer |
| Field Cricket | *Gryllus pennsylvanicus* | Night singer |
| Water Strider | *Gerris remigis* | Walks on water |
| Walking Stick | *Diapheromera femorata* | Camouflage |
| Mayfly | *Ephemeroptera spp.* | Very short-lived |
| Earwig | *Forficula auricularia* | Pincers |

#### Harmful/Notable (6)

| Species | Scientific Name | Danger | Notes |
|---------|-----------------|--------|-------|
| Mosquito | *Culicidae spp.* | Disease vector | Night biter |
| Deer Tick | *Ixodes scapularis* | Disease vector | Check after woods |
| Chigger | *Trombiculidae spp.* | Intense itch | Tall grass |
| Horse Fly | *Tabanus spp.* | Painful bite | Day biter |
| Black Fly | *Simuliidae spp.* | Swarm biter | Near water |
| Velvet Ant | *Dasymutilla occidentalis* | Severe sting | "Cow killer" |

### Catching Mechanics

#### Tools

| Tool | Description | Best For | Modifier |
|------|-------------|----------|----------|
| Butterfly Net | Long-handled mesh net | Flying insects | 2.0x butterflies, 1.5x others |
| Insect Jar | Glass jar with lid | Slow/crawling | 1.8x fireflies, 1.5x beetles |
| Bare Hands | No tool | Easy catches | 0.5x all |
| Tweezers | Fine-tipped | Tiny/parasites | 2.0x ticks, 0.3x others |
| Sweep Net | Heavy-duty net | Grass insects | 1.5x grasshoppers |

#### Catch Difficulty

```
Base Success = 0.3 + (Skill × 0.4) + (Tool Modifier × 0.2)

Modifiers:
- Time of day match: +0.1
- Weather (clear): +0.05
- Crouching: +0.1
- Moving slowly: +0.1
- Running: -0.2
```

#### Specimen Quality

```
Quality (0.0-1.0) affects:
- Collection value
- Firefly glow duration
- Trade value

Quality Factors:
- Catch method (net > hands)
- Speed of catch (quick > struggle)
- Weather conditions
- Player skill level
```

### Insect Jar System

#### Jar Types

| Type | Capacity | Special | Craft Materials |
|------|----------|---------|-----------------|
| Clay Jar | 3 | Basic | clay, kiln |
| Glass Jar | 5 | See-through | sand, furnace |
| Ornate Jar | 5 | Display value | glass, decorations |
| Firefly Lantern | 5 | Extended glow | glass, wire frame |

#### Jar Mechanics

```rust
struct InsectJar {
    jar_type: JarType,
    contents: Vec<JarredInsect>,
    max_capacity: u8,
    lid_closed: bool,
    air_holes: bool,           // Without = insects suffocate
    air_quality: f32,          // 0.0-1.0, degrades if sealed
    light_intensity: f32,      // For bioluminescent insects
}

struct JarredInsect {
    species: InsectSpecies,
    quality: f32,              // 0.0-1.0
    health: f32,               // Degrades in captivity
    time_captured: f64,
}
```

#### Jar Updates (per game tick)

1. If `lid_closed && !air_holes`: `air_quality -= dt × 0.01`
2. If `air_quality < 0.3`: `insect.health -= dt × 0.05`
3. Remove dead insects (`health <= 0`)
4. Update `light_intensity` for fireflies (night only)

---

## Beekeeping System

### Wild Hive Discovery

#### Spawn Locations

- Tree hollows (oak, maple preferred)
- Rock crevices
- Old building cavities
- Underground (bumblebees)

#### Discovery Methods

1. **Visual:** See bees entering/exiting
2. **Audio:** Hear buzzing (range: 20m)
3. **Bee tracking:** Follow foraging bee back
4. **Skill unlock:** "Keen Observer" highlights hives

#### Wild Hive Properties

| Property | Range | Notes |
|----------|-------|-------|
| Colony size | 100-5000 | Larger = more honey, more danger |
| Honey amount | 0.0-10.0 kg | Accumulates over time |
| Aggression | 0.0-1.0 | Higher when disturbed |
| Species | Variable | Determines behavior |

### Hive Capture

#### Requirements

| Item | Purpose |
|------|---------|
| Smoker | Calms bees (reduces aggression 50%) |
| Bee veil | Reduces sting damage 80% |
| Gloves | Reduces sting damage 50% |
| Hive box | Transport container |

#### Capture Process

```
1. Approach hive (aggression rises)
2. Apply smoke (reduces aggression)
3. Open hive (aggression spike)
4. Locate queen
5. Transfer queen to box (colony follows)
6. Transport to apiary site
7. Install in prepared hive
```

#### Capture Risks

- Stings: 1-10 damage each, chance based on aggression
- Queen loss: 10% chance, colony dies
- Swarm escape: 20% chance if fumbled

### Hive Types

| Type | Capacity | Harvest | Period Accuracy | Cost |
|------|----------|---------|-----------------|------|
| Wild Tree | 3000 | Destructive | N/A | Free |
| Log Hive | 4000 | Difficult | 1580s | Low |
| Skep (straw dome) | 5000 | Destructive | 1580s | Low |
| Wooden Box | 6000 | Easy | Later period | Medium |

### Apiary Management

#### Placement Requirements

- Flat ground
- Near water source (within 100m)
- Near flowering plants
- Protected from wind
- Away from high traffic

#### Colony Health Factors

| Factor | Effect | Mitigation |
|--------|--------|------------|
| Disease | Colony strength -5%/day | Inspect, quarantine |
| Mites | Colony strength -2%/day | Treatment herbs |
| Starvation | Colony death | Leave honey reserves |
| Cold | Winter die-off | Winterization |
| Swarming | Lose 50% colony | Add space, split hive |

#### Production Rates

```
Daily Honey = base_rate × colony_strength × nectar_factor × season_modifier

Base rates by species:
- European Honeybee: 0.05 kg/day
- Eastern Bumblebee: 0.015 kg/day

Nectar factor = min(1.0, nearby_flowers / 10)

Season modifiers:
- Spring: 0.8
- Summer: 1.0
- Fall: 0.6
- Winter: 0.0 (no foraging)
```

#### Byproducts

| Product | Rate | Uses |
|---------|------|------|
| Honey | 0.05 kg/day | Food, medicine, trade |
| Beeswax | honey × 0.1 | Candles, waterproofing |
| Propolis | 0.01 kg/day | Medicine, sealant |
| Royal Jelly | Rare | Medicine, special food |

### Bee Aggression Mechanics

#### Aggression Triggers

| Action | Aggression Increase |
|--------|---------------------|
| Approach hive | +0.1 |
| Open hive | +0.3 |
| Harvest honey | +0.2 |
| Quick movements | +0.15 |
| Dark clothing | +0.1 |
| Perfume/strong scent | +0.2 |

#### Aggression Reduction

| Factor | Reduction |
|--------|-----------|
| Smoker | -0.5 |
| White clothing | -0.1 |
| Slow movements | -0.1 |
| High skill | -0.2 |
| Time (per minute) | -0.05 |

#### Sting Mechanics

```
Sting chance per second = aggression × 0.3

Stings received = sting_chance × bee_count / 1000

Damage per sting = 2 HP
Poison stack = 1 per sting (causes DoT)

Mitigation:
- Bee veil: -80% stings
- Gloves: -50% stings
- Full suit: -95% stings
```

---

## Firefly Jar System

### Firefly Behavior (World)

#### Spawn Conditions

- Time: 7 PM - 11 PM (19:00-23:00)
- Weather: Clear or partly cloudy
- Season: Late spring through early fall
- Moon phase: Any (brighter on dark nights)

#### Spawn Locations

| Biome | Density | Quality |
|-------|---------|---------|
| Meadow | High | Normal |
| Forest edge | High | Normal |
| Wetland/marsh | Very high | High |
| Dense forest | Low | Normal |
| Near water | High | High |
| Open field | Medium | Normal |

#### Firefly World Entity

```rust
struct WorldFirefly {
    position: Vec3,
    velocity: Vec3,
    glow_phase: f32,           // For flashing pattern
    quality: f32,              // 0.7-1.0
    species: FireflySpecies,   // Different flash patterns
}
```

### Catching Fireflies

#### Methods

| Method | Success Rate | Quality Preserved |
|--------|--------------|-------------------|
| Jar (slow approach) | 60% | 95% |
| Jar (quick grab) | 40% | 80% |
| Net | 80% | 70% |
| Bare hands | 30% | 60% |

#### Catch Window

```
Optimal catch time: 20:00-22:00
- Before 19:00: No fireflies
- 19:00-20:00: Few, scattered
- 20:00-22:00: Peak activity
- 22:00-23:00: Declining
- After 23:00: None
```

### Firefly Lantern

#### Properties

```rust
struct FireflyLantern {
    jar: InsectJar,
    glow_remaining: f32,       // Total seconds of light
    current_intensity: f32,    // 0.0-1.0
    flash_timer: f32,          // For realistic pattern
    lantern_style: LanternStyle,
}

const FIREFLY_GLOW_DURATION: f32 = 300.0;  // 5 min per firefly
const FIREFLY_LIGHT_RADIUS: f32 = 6.0;      // meters at full
const FIREFLY_LIGHT_COLOR: [f32; 3] = [0.8, 1.0, 0.4]; // Yellow-green
```

#### Light Behavior

```
Intensity = (firefly_count × 0.2).min(1.0)
Radius = FIREFLY_LIGHT_RADIUS × intensity
Flicker = 0.7 + sin(time × 0.5) × 0.3  // Slow pulse

Light only active:
- At night (hour >= 19 || hour <= 5)
- While glow_remaining > 0
- With lid closed
```

#### Glow Duration

```
Total glow = sum(firefly.quality × FIREFLY_GLOW_DURATION)

Example:
- 5 fireflies at 0.8 quality each
- Total = 5 × 0.8 × 300 = 1200 seconds = 20 minutes

Consumption:
- Glow decreases in real-time when active
- Firefly health also decreases (captivity stress)
```

#### Decorative Placement

Requirements:
- At least 3 fireflies
- Quality average > 0.7
- Suitable surface

When placed:
- Functions as light source in world
- Attracts more fireflies at night
- Can be retrieved

### Firefly Release

```
Release all: Opens jar, fireflies fly away
- Returns jar to empty state
- Small reputation bonus with nature spirits
- Fireflies may return to area at night
```

---

## Skill Tree Extensions

### Naturalist Skills

```
Tier 1 (0 pts):
├── Keen Observer
│   - Highlights insects within 10m
│   - +10% catch chance

Tier 2 (100 pts):
├── Insect Catcher
│   - +15% catch chance
│   - Unlocks sweep net
├── Plant Identifier
│   - Identify nectar sources
│   - Better apiary placement

Tier 3 (300 pts):
├── Butterfly Collector
│   - +20% butterfly catch
│   - Display cases available
├── Beetle Expert
│   - +20% beetle catch
│   - Identify rare species

Tier 4 (600 pts):
├── Bee Whisperer
│   - -50% bee aggression
│   - +20% honey yield
├── Moth Hunter
│   - Night catching +30%
│   - Luna moth tracking

Tier 5 (1000 pts):
├── Master Entomologist
│   - +25% all catch rates
│   - Rare species spawn boost
├── Apiarist
│   - Unlock apiaries
│   - Colony health bonus

Tier 6 (1500 pts):
└── Firefly Keeper
    - 2x firefly glow duration
    - Fireflies breed in captivity
```

### Skill Effects Summary

| Skill | Effect |
|-------|--------|
| Keen Observer | Highlight + 10% catch |
| Insect Catcher | +15% catch, sweep net |
| Butterfly Collector | +20% butterfly catch |
| Beetle Expert | +20% beetle catch |
| Bee Whisperer | -50% aggression, +20% honey |
| Moth Hunter | +30% night catch |
| Master Entomologist | +25% all, rare spawn |
| Apiarist | Apiaries, colony health |
| Firefly Keeper | 2x glow, breeding |

---

## Implementation Phases

### Phase 1: Database Foundation
- [ ] Add `rusqlite` dependency
- [ ] Create database module structure
- [ ] Implement schema creation
- [ ] Add migration system
- [ ] Integrate with save/load

### Phase 2: Universal Taming Framework
- [ ] Create TamingCategory enum
- [ ] Implement TamingRequirements per species
- [ ] Refactor wolf taming to universal system
- [ ] Integrate horse taming
- [ ] Add avian taming phases

### Phase 3: Companion System
- [ ] Unify Dog/Horse companion structs
- [ ] Implement CompanionAbilities
- [ ] Add companion commands
- [ ] Create companion AI states

### Phase 4: Entomology Core
- [ ] Implement InsectSpecies enum (50 species)
- [ ] Add insect spawning by biome/time
- [ ] Implement catching mechanics
- [ ] Create insect jar system

### Phase 5: Firefly System
- [ ] Firefly world spawning
- [ ] Catching at dusk/night
- [ ] Firefly lantern light source
- [ ] Glow duration mechanics
- [ ] Decorative placement

### Phase 6: Beekeeping Core
- [ ] Wild hive generation
- [ ] Hive discovery mechanics
- [ ] Capture system
- [ ] Basic apiary structure

### Phase 7: Apiary Management
- [ ] Production calculations
- [ ] Colony health system
- [ ] Harvest mechanics
- [ ] Winterization

### Phase 8: Integration
- [ ] Naturalist skill tree
- [ ] Encyclopedia integration
- [ ] UI for collections
- [ ] Trading integration

---

## Testing Checklist

### Database
- [ ] Tables created on first run
- [ ] Data persists across saves
- [ ] Migrations run correctly
- [ ] Foreign keys enforced

### Taming
- [ ] Wolf taming still works
- [ ] Horse taming still works
- [ ] New species can be tamed
- [ ] Requirements enforced
- [ ] Progress saved to DB

### Companions
- [ ] Commands work per species
- [ ] Abilities unlock at bond levels
- [ ] Companions persist across sessions
- [ ] Combat abilities function

### Entomology
- [ ] Insects spawn in correct biomes
- [ ] Time-of-day filtering works
- [ ] Catching mechanics balanced
- [ ] Jars hold correct capacity

### Fireflies
- [ ] Spawn at dusk only
- [ ] Light works at night
- [ ] Glow duration accurate
- [ ] Release functions

### Beekeeping
- [ ] Wild hives generate
- [ ] Capture succeeds/fails appropriately
- [ ] Production rates correct
- [ ] Aggression mechanics work
- [ ] Winterization prevents death

---

## File Locations

### New Files

```
roanoke_game/src/database/
├── mod.rs
├── schema.rs
├── queries.rs
└── migrations.rs

roanoke_game/src/taming/
├── mod.rs
├── universal.rs
├── companions.rs
├── canine.rs
├── equine.rs
└── avian.rs

roanoke_game/src/entomology/
├── mod.rs
├── species.rs
├── catching.rs
├── jars.rs
└── collection.rs

roanoke_game/src/beekeeping/
├── mod.rs
├── species.rs
├── wild_hives.rs
├── apiary.rs
├── production.rs
└── mechanics.rs

roanoke_game/src/fireflies/
├── mod.rs
├── spawner.rs
├── lantern.rs
└── light.rs
```

### Files to Modify

| File | Changes |
|------|---------|
| `Cargo.toml` | Add rusqlite |
| `src/lib.rs` | Module declarations |
| `src/game_state.rs` | Database integration |
| `src/animals/types.rs` | taming_category() |
| `src/animals/taming.rs` | Universal integration |
| `src/progression/skills.rs` | Naturalist tree |
| `src/encyclopedia/mod.rs` | Insect discovery |
