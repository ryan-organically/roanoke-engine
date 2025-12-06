# Roanoke Financial Marketplace & Loot Drop System

**Related Documents:**
- `DECADE_FINANCIAL_ROADMAP.md` - 10-year economic strategy, institutional investment thesis
- `TRILLION_DOLLAR_VISION.md` - 20-year trillion-dollar infrastructure thesis
- `ROADMAP.md` - Technical implementation roadmap

## Executive Summary

A skill, luck, and time-based economy where every item has provenance, rarity derives from mathematical scarcity, and value emerges from player-driven markets. The system creates "digital artifact" value without blockchain—through deterministic rarity, deflationary sinks, and permanent item history.

**Investment Thesis:** This loot system is the value-generation engine that powers the dual-currency economy (Wampum/Tobacco) detailed in `DECADE_FINANCIAL_ROADMAP.md`. Items are the assets; currencies are the medium of exchange; the marketplace is the liquidity layer.

---

## Part I: Core Rarity Architecture

### 1.1 Rarity Tiers

```
TIER          BASE DROP RATE    COLOR       PARTICLE EFFECT
─────────────────────────────────────────────────────────────
Crude         45.0%             Gray        None
Common        30.0%             White       None
Uncommon      15.0%             Green       Faint shimmer
Rare          6.5%              Blue        Soft glow
Epic          2.5%              Purple      Aura pulse
Legendary     0.8%              Orange      Fire wisps
Mythic        0.18%             Red/Gold    Reality distortion
Primordial    0.02%             Black/Star  Void particles
```

**Primordial Tier**: Items that exist in single-digit quantities server-wide. Once dropped, that specific variant can NEVER drop again. The game tracks all Primordial drops globally.

### 1.2 The Drop Roll System

Every loot event executes a **7-layer roll cascade**:

```rust
struct DropRoll {
    // Layer 1: Base RNG (0.0 - 1.0)
    base_roll: f64,

    // Layer 2: Luck Modifier (from perks, gear, consumables)
    luck_bonus: f64,           // Additive: +0.05 = 5% better rolls

    // Layer 3: Skill Mastery (relevant skill level)
    skill_modifier: f64,       // 0.0 at level 1, up to 0.15 at level 100

    // Layer 4: Time Investment (consecutive play session)
    session_pity: f64,         // Increases 0.001 per minute, caps at 0.10

    // Layer 5: Moon Phase / Weather
    celestial_modifier: f64,   // -0.02 to +0.05 based on in-game conditions

    // Layer 6: Event Multiplier (seasonal events)
    event_multiplier: f64,     // 1.0 normal, up to 2.0 during events

    // Layer 7: Karma System (anti-streak protection)
    karma_correction: f64,     // Builds when unlucky, consumed on good drop
}

fn calculate_final_roll(roll: &DropRoll) -> f64 {
    let modified = roll.base_roll
        - roll.luck_bonus
        - roll.skill_modifier
        - roll.session_pity
        - roll.celestial_modifier
        - roll.karma_correction;

    (modified * roll.event_multiplier).clamp(0.0, 1.0)
}

fn determine_rarity(final_roll: f64) -> Rarity {
    match final_roll {
        r if r <= 0.0002 => Rarity::Primordial,
        r if r <= 0.002  => Rarity::Mythic,
        r if r <= 0.01   => Rarity::Legendary,
        r if r <= 0.035  => Rarity::Epic,
        r if r <= 0.10   => Rarity::Rare,
        r if r <= 0.25   => Rarity::Uncommon,
        r if r <= 0.55   => Rarity::Common,
        _ => Rarity::Crude,
    }
}
```

### 1.3 Pity System (Karma Accumulation)

Players accumulate "karma" when receiving below-expected drops:

```rust
struct KarmaTracker {
    accumulated_karma: f64,
    drops_since_rare: u32,
    drops_since_epic: u32,
    drops_since_legendary: u32,
    lifetime_karma_spent: f64,  // Tracked for achievements
}

impl KarmaTracker {
    fn on_drop(&mut self, rarity: Rarity, expected_rarity: Rarity) {
        let deficit = expected_rarity.value() - rarity.value();
        if deficit > 0 {
            // Unlucky: accumulate karma
            self.accumulated_karma += deficit as f64 * 0.002;
        } else {
            // Lucky: spend karma
            self.accumulated_karma = (self.accumulated_karma - 0.01).max(0.0);
        }

        // Hard pity counters
        match rarity {
            Rarity::Rare | higher => self.drops_since_rare = 0,
            _ => self.drops_since_rare += 1,
        }
        // ... similar for epic, legendary

        // Guaranteed drop thresholds
        if self.drops_since_rare >= 50 { /* Force rare next drop */ }
        if self.drops_since_epic >= 200 { /* Force epic next drop */ }
        if self.drops_since_legendary >= 1000 { /* Force legendary next drop */ }
    }
}
```

---

## Part II: Item Variant Architecture

### 2.1 Item DNA System

Every item has a unique "genetic" composition determining its properties:

```rust
struct ItemDNA {
    // 64-bit seed generating all item properties
    genome: u64,

    // Decoded traits
    base_type: ItemBaseType,
    rarity: Rarity,
    quality: Quality,           // 0-100, affects base stats
    prefix: Option<Prefix>,     // Magical prefix modifier
    suffix: Option<Suffix>,     // Magical suffix modifier
    implicit: Option<Implicit>, // Innate bonus based on base type

    // Variant markers
    variant_class: VariantClass,
    seasonal_tag: Option<SeasonalTag>,
    event_tag: Option<EventTag>,

    // Provenance (immutable history)
    provenance: ItemProvenance,
}

struct ItemProvenance {
    first_owner_id: PlayerId,
    first_owner_name: String,       // Permanently recorded
    discovery_timestamp: u64,       // Unix timestamp
    discovery_location: WorldCoord, // Where it dropped
    discovery_method: DropSource,   // How it was obtained

    // Kill provenance (for hunting drops)
    creature_killed: Option<CreatureType>,
    kill_method: Option<KillMethod>,
    kill_quality: Option<KillQuality>,

    // Ownership chain (last 10 owners)
    ownership_history: Vec<OwnershipRecord>,

    // Usage statistics
    times_traded: u32,
    total_trade_value: u64,         // Sum of all sale prices
    kills_with_item: u32,           // For weapons
    resources_gathered: u32,        // For tools
}
```

### 2.2 Prefix System (324 Prefixes)

Prefixes modify item stats and appearance:

```
CATEGORY: ELEMENTAL (36 prefixes)
──────────────────────────────────────────────────────────────────
Tier 1 (Common+):   Chilled, Heated, Sparked, Dampened...
Tier 2 (Rare+):     Freezing, Blazing, Crackling, Drowning...
Tier 3 (Epic+):     Glacial, Infernal, Voltaic, Abyssal...
Tier 4 (Legendary+): Permafrost, Hellforged, Thundergod's, Leviathan's...

CATEGORY: PHYSICAL (48 prefixes)
──────────────────────────────────────────────────────────────────
Tier 1: Sharp, Heavy, Light, Balanced...
Tier 2: Honed, Brutal, Swift, Precise...
Tier 3: Razor, Crushing, Lightning, Surgical...
Tier 4: Vorpal, Annihilating, Phantom, Inevitable...

CATEGORY: MYSTICAL (60 prefixes)
──────────────────────────────────────────────────────────────────
Tier 1: Lucky, Blessed, Cursed, Haunted...
Tier 2: Fortunate, Sacred, Hexed, Spectral...
Tier 3: Fated, Divine, Malefic, Ethereal...
Tier 4: Destiny's, Godtouched, Doombound, Voidwalker's...

CATEGORY: CREATURE (72 prefixes - from hunting)
──────────────────────────────────────────────────────────────────
Tier 1: Wolf's, Bear's, Deer's, Fox's...
Tier 2: Alpha's, Grizzly's, Stag's, Vixen's...
Tier 3: Dire Wolf's, Kodiak's, Great Elk's, Spirit Fox's...
Tier 4: Fenrir's, Ursa Major's, Cernunnos's, Kitsune's...

CATEGORY: TEMPORAL (24 prefixes - time/season locked)
──────────────────────────────────────────────────────────────────
Dawn, Morning, Noon, Dusk, Midnight, Witching...
Spring's, Summer's, Autumn's, Winter's...
New Moon's, Full Moon's, Eclipse's, Solstice's...

CATEGORY: PRIMORDIAL (12 prefixes - Mythic+ only)
──────────────────────────────────────────────────────────────────
Firstborn, Lastborn, Eternal, Infinite...
Omega, Alpha, Genesis, Terminus...
Worldbreaker's, Godslayer's, Starforged, Voidborn...
```

### 2.3 Suffix System (216 Suffixes)

Suffixes grant special abilities and bonuses:

```
CATEGORY: OF ATTRIBUTE (36 suffixes)
──────────────────────────────────────────────────────────────────
...of Strength (+1 to +50 Strength based on tier)
...of Agility, ...of Vitality, ...of Wisdom...
...of the Titan (T4: +40-50 to all physical stats)

CATEGORY: OF SKILL (48 suffixes)
──────────────────────────────────────────────────────────────────
...of the Hunter (+5% to +25% hunting XP)
...of the Archaeologist, ...of the Trapper...
...of Mastery (T4: +20% XP to all skills)

CATEGORY: OF PROTECTION (36 suffixes)
──────────────────────────────────────────────────────────────────
...of Warding (+5 to +50 armor)
...of the Fortress, ...of Evasion...
...of Immortality (T4: +5% chance to survive lethal damage)

CATEGORY: OF FORTUNE (24 suffixes)
──────────────────────────────────────────────────────────────────
...of Luck (+1% to +15% luck)
...of Prosperity (+5% to +30% gold find)
...of the Jackpot (T4: Double loot 5% of the time)

CATEGORY: OF SLAYING (36 suffixes)
──────────────────────────────────────────────────────────────────
...of Wolf Slaying (+10% to +50% damage vs wolves)
...of Bear Slaying, ...of Beast Slaying...
...of Extinction (T4: +30% damage vs all creatures)

CATEGORY: OF GATHERING (24 suffixes)
──────────────────────────────────────────────────────────────────
...of Harvesting (+10% to +40% resource yield)
...of Preservation (+10% to +30% resource quality)
...of Abundance (T4: 10% chance for double resources)

CATEGORY: OF LEGEND (12 suffixes - Mythic+ only)
──────────────────────────────────────────────────────────────────
...of Myth, ...of Legend, ...of Eternity...
...of the Forgotten Gods, ...of the First People...
...of Roanoke (T4: Unique effect based on item type)
```

### 2.4 Quality Gradients

Every item rolls a quality score (0-100) affecting base stats:

```rust
struct QualityDistribution {
    // Quality follows a weighted bell curve
    // Mean shifts based on skill level and luck
}

fn roll_quality(skill_level: u32, luck: f64) -> u32 {
    let base_mean = 35.0 + (skill_level as f64 * 0.3); // 35-65 mean
    let luck_bonus = luck * 20.0; // Up to +20 mean
    let std_dev = 15.0 - (skill_level as f64 * 0.05); // Tighter at high skill

    let quality = normal_distribution(base_mean + luck_bonus, std_dev);
    quality.clamp(0, 100) as u32
}

// Quality affects stats multiplicatively
fn apply_quality(base_stat: f64, quality: u32) -> f64 {
    let multiplier = 0.5 + (quality as f64 / 100.0); // 0.5x to 1.5x
    base_stat * multiplier
}
```

**Quality Descriptors:**
```
0-10:   Ruined      (red text, visibly damaged model)
11-25:  Poor        (gray text, worn appearance)
26-40:  Adequate    (white text, normal appearance)
41-60:  Fine        (light blue text, polished appearance)
61-80:  Superior    (blue text, pristine appearance)
81-95:  Exceptional (purple text, subtle glow)
96-100: Perfect     (gold text, radiant effect)
```

### 2.5 Variant Classes

Items spawn in variant classes affecting their market niche:

```rust
enum VariantClass {
    // Standard variants (85% of drops)
    Standard,

    // Visual variants (10% of drops)
    Weathered,      // Aged appearance, same stats
    Pristine,       // New appearance, same stats
    Ornate,         // Decorative flourishes
    Minimalist,     // Stripped-down aesthetic

    // Functional variants (4% of drops)
    Lightweight,    // -20% weight, -5% damage
    Reinforced,     // +20% durability, +10% weight
    Balanced,       // +10% accuracy, -5% damage
    Aggressive,     // +15% damage, -10% durability

    // Rare variants (1% of drops)
    Masterwork,     // +25% all stats, unique maker's mark
    Ancient,        // Pre-colonial origin, unique appearance
    Corrupted,      // Cursed stats, can be purified
    Blessed,        // Holy origin, bonus vs undead

    // Ultra-rare variants (0.1% of drops - Legendary+ only)
    Prototype,      // First of its kind, experimental stats
    Perfected,      // Mathematically optimal rolls
    Anomalous,      // Breaks normal stat rules

    // Unique variants (0.01% - one per server)
    Singular,       // Literally one exists, named item
}
```

---

## Part III: Drop Source Integration

### 3.1 Hunting Drop Matrix

Animal kills produce loot based on multiple factors:

```rust
struct HuntingDropCalculation {
    // Base factors
    animal_type: AnimalType,
    animal_rarity: AnimalRarity,     // Common, Uncommon, Rare, Legendary beast
    animal_size: f32,                // Percentile vs species average
    animal_age: AnimalAge,           // Young, Adult, Elder, Ancient
    animal_health: f32,              // % health when killed

    // Kill quality factors
    kill_method: KillMethod,
    kill_speed: f32,                 // Time from first hit to death
    kill_distance: f32,              // For ranged kills
    vital_hit: bool,                 // Heart/brain shot
    clean_kill: bool,                // No suffering (quick death)

    // Environmental factors
    moon_phase: MoonPhase,
    weather: Weather,
    time_of_day: TimeOfDay,
    biome: Biome,

    // Player factors
    hunting_skill: u32,
    tracking_skill: u32,
    relevant_perks: Vec<Perk>,
    equipped_luck: f64,
}

enum KillMethod {
    Bow { draw_strength: f32, arrow_type: ArrowType },
    Spear { thrown: bool, thrust_power: f32 },
    Trap { trap_type: TrapType, bait_used: Option<BaitType> },
    Knife { stealth_kill: bool },
    Club { stunning_blow: bool },
    Natural { cause: NaturalCause }, // Found dead, lowest quality
}

enum KillQuality {
    Botched,      // Multiple hits, suffering, damaged pelt: 0.5x drops
    Poor,         // Slow kill, some damage: 0.75x drops
    Standard,     // Normal kill: 1.0x drops
    Clean,        // Quick, minimal suffering: 1.25x drops
    Perfect,      // One-shot vital hit: 1.5x drops
    Legendary,    // Perfect conditions + legendary animal: 2.0x drops
}
```

**Drop Tables by Animal Type:**

```
DEER (Common)
──────────────────────────────────────────────────────────────────
Guaranteed:  Venison (1-4 based on size)
Common:      Deer Hide, Antler Fragment, Sinew
Uncommon:    Quality Deer Hide, Intact Antler
Rare:        Premium Pelt, Trophy Antler, Deer Heart
Epic:        Albino Pelt, Crown Antler, Spirit Essence
Legendary:   White Stag's Blessing (accessory, +luck)

BEAR (Uncommon)
──────────────────────────────────────────────────────────────────
Guaranteed:  Bear Meat (2-8 based on size)
Common:      Bear Hide, Bear Fat, Claws
Uncommon:    Thick Bear Pelt, Bear Teeth, Quality Fat
Rare:        Grizzly Pelt, Bear Heart, Claws (Trophy)
Epic:        Spirit Bear Pelt, Ursa Minor (constellation map)
Legendary:   Great Bear's Fury (weapon mod, +damage vs beasts)

LEGENDARY BEASTS (Region-specific, one spawn per real-time week)
──────────────────────────────────────────────────────────────────
Guaranteed:  3-5 Epic materials
High Chance: 1-2 Legendary materials
Possible:    Mythic crafting component
Rare:        Beast's Named Weapon (unique stats)
Ultra-Rare:  Beast Soul (bound accessory, major bonuses)
```

### 3.2 Fossil Drop System

Archaeological finds with geological rarity:

```rust
struct FossilDropSystem {
    // Dig site factors
    site_type: DigSiteType,
    site_rarity: SiteRarity,
    depth: f32,                      // Deeper = rarer
    soil_type: SoilType,
    geological_layer: GeologicalLayer,

    // Extraction factors
    extraction_skill: u32,
    tool_quality: u32,
    extraction_time: f32,            // Patience rewarded
    extraction_precision: f32,       // Mini-game score

    // Discovery factors
    survey_skill: u32,
    knowledge_level: u32,            // Unlocks better sites
}

enum GeologicalLayer {
    Surface,           // Recent: shells, bones, artifacts
    Pleistocene,       // Ice age: megafauna, early human
    Cretaceous,        // Dinosaurs, ammonites
    Jurassic,          // Marine reptiles, early dinosaurs
    Triassic,          // Primitive reptiles, early mammals
    Paleozoic,         // Ancient marine life, trilobites
    Precambrian,       // Primordial: stromatolites, impossible fossils
}
```

**Fossil Rarity by Layer:**

```
SURFACE LAYER (0-1m depth)
──────────────────────────────────────────────────────────────────
Common:      Shell Fragments, Bone Shards, Pottery Shards
Uncommon:    Intact Shells, Animal Bones, Arrowheads
Rare:        Colonial Artifacts, Native Tools, Jewelry
Epic:        Historical Documents, Ceremonial Items
Legendary:   Lost Colony Artifacts (tied to Roanoke lore)

PLEISTOCENE (1-5m depth)
──────────────────────────────────────────────────────────────────
Common:      Mammoth Bone Fragments, Ice Age Shells
Uncommon:    Saber-tooth Fangs, Dire Wolf Bones
Rare:        Mammoth Ivory, Giant Sloth Claws
Epic:        Preserved Mammoth Hair, Frozen Seeds
Legendary:   Intact Mammoth Tusk (furniture/trophy)

ANCIENT LAYERS (5m+ depth, Rare dig sites only)
──────────────────────────────────────────────────────────────────
Rare:        Dinosaur Bone Fragments
Epic:        Intact Dinosaur Bones, Ammonites
Legendary:   Dinosaur Teeth, Trilobites
Mythic:      Amber with Inclusions, Petrified Wood
Primordial:  "Impossible" Fossils (anachronistic, mysterious)
```

**Extraction Quality System:**

```rust
enum ExtractionResult {
    Shattered,    // Failed mini-game: fragments only, 0.25x value
    Damaged,      // Poor extraction: visible cracks, 0.5x value
    Intact,       // Standard extraction: full value
    Pristine,     // Excellent extraction: 1.5x value
    Museum,       // Perfect extraction: 2x value, special display
    Scientific,   // Perfect + rare: 3x value, unlocks research
}
```

### 3.3 Weapon Drop System

Weapons found in the world or crafted:

```rust
struct WeaponDrop {
    // Source determines base quality
    source: WeaponSource,

    // Material affects tier ceiling
    material: WeaponMaterial,

    // Construction quality
    craftsmanship: u32,          // 0-100

    // Combat modifiers
    base_damage: f32,
    attack_speed: f32,
    durability: u32,
    weight: f32,
}

enum WeaponSource {
    // Found weapons (random stats)
    WorldDrop { location: Biome },
    CreatureDrop { creature: CreatureType },
    ChestLoot { chest_tier: u32 },

    // Crafted weapons (controlled stats)
    PlayerCrafted { crafter_skill: u32 },
    NpcCrafted { npc_tier: NpcTier },

    // Special sources
    QuestReward { quest_difficulty: u32 },
    EventDrop { event: Event },
    TradePost { trader_reputation: u32 },

    // Ultra-rare sources
    LegendaryBeastDrop,
    AncientCache,
    BurialSite,
}

enum WeaponMaterial {
    // Tier 1: Common materials
    Wood,
    Bone,
    Stone,

    // Tier 2: Worked materials (Uncommon+)
    FlintKnapped,
    HardenedBone,
    Antler,
    Copper,

    // Tier 3: Advanced materials (Rare+)
    Bronze,
    Iron,
    SteelTrade,      // European trade goods

    // Tier 4: Exotic materials (Epic+)
    MeteorIron,
    ObsidianVolcanic,
    PetrifiedWood,
    FossilBone,

    // Tier 5: Mythical materials (Legendary+)
    StarMetal,
    DragonBone,      // Ancient beast fossils
    CrystalizedSap,  // Ancient tree hearts

    // Tier 6: Primordial materials (Mythic+)
    VoidStone,
    FirstFlame,
    EternalIce,
    WorldTree,
}
```

---

## Part IV: Deflationary Mechanisms

### 4.1 Item Degradation System

All items have durability that cannot be fully restored:

```rust
struct ItemDurability {
    current: u32,
    maximum: u32,
    original_maximum: u32,    // Never increases
    repair_count: u32,
}

impl ItemDurability {
    fn repair(&mut self, repair_skill: u32, materials: &Materials) -> RepairResult {
        let repair_amount = calculate_repair(repair_skill, materials);
        self.current = (self.current + repair_amount).min(self.maximum);

        // Each repair permanently reduces max durability
        let degradation = 1 + (self.repair_count / 5); // Accelerating decay
        self.maximum = self.maximum.saturating_sub(degradation);
        self.repair_count += 1;

        // Item breaks permanently when max durability hits threshold
        if self.maximum < self.original_maximum / 4 {
            RepairResult::Broken // Item destroyed
        } else {
            RepairResult::Repaired
        }
    }
}
```

**Durability Loss Events:**
- Combat use: 1-3 durability per hit
- Harvesting: 1 durability per harvest
- Environmental: 0.1 durability per in-game hour in rain/snow
- Death: 5-15% of max durability lost

### 4.2 Crafting Consumption

Rare items consumed to create rarer items:

```rust
struct CraftingRecipe {
    inputs: Vec<(ItemRequirement, u32)>,
    output: ItemTemplate,
    success_rate: f64,           // Failure consumes inputs
    critical_rate: f64,          // Critical success = bonus stats
    skill_requirement: u32,
}

// Example: Legendary weapon requires consuming Epic materials
let legendary_bow_recipe = CraftingRecipe {
    inputs: vec![
        (ItemRequirement::Rarity(Rarity::Epic), 3),      // 3 Epic items destroyed
        (ItemRequirement::Material(Material::StarMetal), 5),
        (ItemRequirement::Type(ItemType::BowString), 1),
        (ItemRequirement::Specific(ItemId::AncientBlueprint), 1),
    ],
    output: ItemTemplate::LegendaryBow,
    success_rate: 0.65,          // 35% chance to lose everything
    critical_rate: 0.05,         // 5% chance for Mythic upgrade
    skill_requirement: 80,
};
```

### 4.3 Sacrifice System

Destroy items for permanent character bonuses:

```rust
struct SacrificeAltar {
    // Accumulated sacrifice value unlocks tiers
    total_sacrificed_value: u64,

    // Permanent unlocks
    unlocked_bonuses: Vec<SacrificeBonus>,
}

enum SacrificeBonus {
    // Tier 1: Sacrifice 10,000 value
    LuckBonus(f64),              // +0.5% luck permanently

    // Tier 2: Sacrifice 100,000 value
    DropRateBonus(f64),          // +1% drop rate

    // Tier 3: Sacrifice 1,000,000 value
    RarityUpgradeChance(f64),    // 0.1% chance drops upgrade rarity

    // Tier 4: Sacrifice 10,000,000 value
    PrimordialSight,             // Can see Primordial drop locations

    // Tier 5: Sacrifice a Primordial item
    PrimordialBlessing,          // Unique per-item effect
}
```

### 4.4 Market Tax System

Every transaction has fees that remove currency:

```rust
struct MarketFees {
    // Flat listing fee (removed from economy)
    listing_fee: u64,            // 1% of listed price

    // Transaction tax (removed from economy)
    sale_tax: f64,               // 5% of sale price

    // Rarity surcharge (higher for rare items)
    rarity_tax: f64,             // 0% common, up to 10% Primordial

    // Cross-region tax (trading between servers)
    transfer_tax: f64,           // 15% for cross-server trades
}

fn calculate_seller_receives(sale_price: u64, item: &Item, cross_server: bool) -> u64 {
    let rarity_rate = match item.rarity {
        Rarity::Common => 0.00,
        Rarity::Uncommon => 0.01,
        Rarity::Rare => 0.02,
        Rarity::Epic => 0.03,
        Rarity::Legendary => 0.05,
        Rarity::Mythic => 0.08,
        Rarity::Primordial => 0.10,
        _ => 0.00,
    };

    let transfer_rate = if cross_server { 0.15 } else { 0.0 };
    let total_tax = 0.05 + rarity_rate + transfer_rate;

    (sale_price as f64 * (1.0 - total_tax)) as u64
}
```

### 4.5 Item Binding Mechanics

Some items become untradeable:

```rust
enum BindingType {
    // Freely tradeable
    Unbound,

    // Becomes bound when equipped
    BindOnEquip,

    // Bound immediately when obtained
    BindOnPickup,

    // Bound after first use
    BindOnUse,

    // Can be traded X times total
    LimitedTrades(u32),

    // Can only be traded to party members present at drop
    PartyBound,

    // Bound to account, tradeable between own characters
    AccountBound,

    // Cannot ever be traded or dropped
    Soulbound,
}
```

---

## Part V: Marketplace Architecture

### 5.1 Auction House System

```rust
struct AuctionListing {
    item: Item,
    seller: PlayerId,

    // Pricing
    starting_bid: u64,
    buyout_price: Option<u64>,
    current_bid: u64,
    current_bidder: Option<PlayerId>,

    // Timing
    duration: AuctionDuration,
    start_time: u64,
    end_time: u64,

    // Anti-snipe protection
    extension_count: u32,        // Max 3 extensions
}

enum AuctionDuration {
    Short,      // 12 hours
    Medium,     // 24 hours
    Long,       // 48 hours
    Extended,   // 7 days (higher listing fee)
}

impl AuctionListing {
    fn place_bid(&mut self, bidder: PlayerId, amount: u64) -> BidResult {
        if amount <= self.current_bid {
            return BidResult::TooLow;
        }

        // Anti-snipe: extend auction if bid in last 5 minutes
        let time_remaining = self.end_time - current_time();
        if time_remaining < 300 && self.extension_count < 3 {
            self.end_time += 300; // Add 5 minutes
            self.extension_count += 1;
        }

        // Refund previous bidder
        if let Some(prev_bidder) = self.current_bidder {
            refund_bid(prev_bidder, self.current_bid);
        }

        self.current_bid = amount;
        self.current_bidder = Some(bidder);
        BidResult::Success
    }
}
```

### 5.2 Instant Trade System

Direct player-to-player trading:

```rust
struct TradeSession {
    initiator: PlayerId,
    recipient: PlayerId,

    initiator_offer: TradeOffer,
    recipient_offer: TradeOffer,

    initiator_confirmed: bool,
    recipient_confirmed: bool,

    // Anti-scam delay
    confirmation_delay: u32,     // 3 second delay after last change
    last_modified: u64,
}

struct TradeOffer {
    items: Vec<Item>,
    gold: u64,

    // Item verification
    item_snapshots: Vec<ItemSnapshot>, // Captured at offer time
}

impl TradeSession {
    fn finalize(&self) -> TradeResult {
        // Verify items haven't changed since snapshot
        for (item, snapshot) in self.initiator_offer.items.iter()
            .zip(self.initiator_offer.item_snapshots.iter())
        {
            if item.dna != snapshot.dna {
                return TradeResult::ItemModified;
            }
        }

        // Execute trade atomically
        // ...

        // Update item provenance
        for item in traded_items {
            item.provenance.ownership_history.push(OwnershipRecord {
                owner: new_owner,
                acquired: current_time(),
                method: AcquisitionMethod::Trade,
                price_paid: trade_value,
            });
            item.provenance.times_traded += 1;
        }

        TradeResult::Success
    }
}
```

### 5.3 Price Discovery & History

```rust
struct MarketData {
    // Per-item-template price tracking
    price_history: HashMap<ItemTemplateId, PriceHistory>,

    // Global market metrics
    total_volume_24h: u64,
    total_listings: u32,
    total_gold_supply: u64,
}

struct PriceHistory {
    // Rolling windows
    average_price_1h: f64,
    average_price_24h: f64,
    average_price_7d: f64,
    average_price_30d: f64,

    // Extremes
    all_time_high: u64,
    all_time_high_date: u64,
    all_time_low: u64,
    all_time_low_date: u64,

    // Volume
    sales_count_24h: u32,
    sales_volume_24h: u64,

    // Recent sales (for price reference)
    recent_sales: VecDeque<Sale>, // Last 100 sales
}

struct Sale {
    price: u64,
    timestamp: u64,
    item_quality: u32,
    item_variant: VariantClass,
}
```

### 5.4 Order Book System

Advanced trading with buy/sell orders:

```rust
struct OrderBook {
    item_template: ItemTemplateId,

    // Buy orders (sorted by price descending)
    buy_orders: BTreeMap<u64, Vec<BuyOrder>>,

    // Sell orders (sorted by price ascending)
    sell_orders: BTreeMap<u64, Vec<SellOrder>>,
}

struct BuyOrder {
    buyer: PlayerId,
    price: u64,                  // Max price willing to pay
    quantity: u32,
    min_quality: u32,            // Minimum quality accepted
    created_at: u64,
    expires_at: u64,
}

struct SellOrder {
    seller: PlayerId,
    item: Item,
    price: u64,                  // Asking price
    created_at: u64,
    expires_at: u64,
}

impl OrderBook {
    fn match_orders(&mut self) -> Vec<Trade> {
        let mut trades = Vec::new();

        // Match highest buy with lowest sell
        while let (Some((&buy_price, _)), Some((&sell_price, _))) =
            (self.buy_orders.last_key_value(), self.sell_orders.first_key_value())
        {
            if buy_price >= sell_price {
                // Execute trade at midpoint price
                let trade_price = (buy_price + sell_price) / 2;
                trades.push(self.execute_match(trade_price));
            } else {
                break; // No more matches possible
            }
        }

        trades
    }
}
```

---

## Part VI: Seasonal & Event Systems

### 6.1 Seasonal Calendar

```rust
struct SeasonalCalendar {
    // Four major seasons (3 months each)
    seasons: [Season; 4],

    // Current season state
    current_season: Season,
    season_progress: f32,        // 0.0 to 1.0

    // Seasonal drop modifiers
    active_modifiers: Vec<SeasonalModifier>,
}

enum Season {
    Spring {
        start_date: (u8, u8),    // March 20
        drops: SpringDrops,
    },
    Summer {
        start_date: (u8, u8),    // June 21
        drops: SummerDrops,
    },
    Autumn {
        start_date: (u8, u8),    // September 22
        drops: AutumnDrops,
    },
    Winter {
        start_date: (u8, u8),    // December 21
        drops: WinterDrops,
    },
}

struct SeasonalDrops {
    // Season-exclusive items (cannot drop outside season)
    exclusive_items: Vec<ItemTemplate>,

    // Boosted drop rates during season
    boosted_items: Vec<(ItemTemplate, f64)>,

    // Season-specific creatures
    seasonal_creatures: Vec<CreatureTemplate>,

    // Limited cosmetics (never return)
    yearly_cosmetics: Vec<CosmeticTemplate>,
}
```

**Seasonal Exclusives:**

```
SPRING (March 20 - June 20)
──────────────────────────────────────────────────────────────────
Exclusive Materials:  First Bloom Petals, Spring Water, New Growth Sap
Exclusive Creatures:  Newborn animals (higher quality pelts)
Exclusive Events:     Spring Awakening (double fossil spawns)
Limited Cosmetics:    Cherry Blossom weapon skins (yearly unique)

SUMMER (June 21 - September 21)
──────────────────────────────────────────────────────────────────
Exclusive Materials:  Solstice Amber, Sunfire Ore, Lightning-Struck Wood
Exclusive Creatures:  Legendary Summer Stag
Exclusive Events:     Midsummer Hunt (legendary beast spawn rate +50%)
Limited Cosmetics:    Sunburst armor effects (yearly unique)

AUTUMN (September 22 - December 20)
──────────────────────────────────────────────────────────────────
Exclusive Materials:  Harvest Moon Essence, Fallen Leaves, Frost-Touched Bark
Exclusive Creatures:  Ghost animals (spectral variants)
Exclusive Events:     Harvest Festival (NPC traders offer rare items)
Limited Cosmetics:    Ember Trail effects (yearly unique)

WINTER (December 21 - March 19)
──────────────────────────────────────────────────────────────────
Exclusive Materials:  Eternal Ice, Starfrost Crystal, Hibernation Essence
Exclusive Creatures:  Snow-variant animals (white pelts)
Exclusive Events:     Winter Solstice (Mythic drop rate +25%)
Limited Cosmetics:    Aurora Borealis effects (yearly unique)
```

### 6.2 Live Event System

```rust
struct LiveEvent {
    event_id: EventId,
    name: String,
    description: String,

    // Timing
    start_time: u64,
    end_time: u64,
    is_announced: bool,          // Some events are surprises

    // Event mechanics
    event_type: EventType,
    participation_requirements: Vec<Requirement>,

    // Rewards
    participation_rewards: Vec<Reward>,
    milestone_rewards: Vec<(u32, Reward)>,
    completion_rewards: Vec<Reward>,

    // Limited drops
    event_exclusive_drops: Vec<EventDrop>,

    // Global progress
    server_progress: f64,
    server_milestone: u32,
}

enum EventType {
    // World Boss: Server-wide hunt
    WorldBoss {
        boss: CreatureTemplate,
        spawn_locations: Vec<WorldCoord>,
        health_pool: u64,        // Shared across server
    },

    // Competition: Player vs player scoring
    Competition {
        scoring_method: ScoringMethod,
        leaderboard_rewards: Vec<(u32, Reward)>, // Top X get rewards
    },

    // Collection: Gather specific items
    Collection {
        required_items: Vec<(ItemTemplate, u32)>,
        trade_in_npc: NpcId,
    },

    // Exploration: Discover hidden locations
    Exploration {
        discovery_points: Vec<DiscoveryPoint>,
        rewards_per_discovery: u32,
    },

    // Crafting: Server-wide crafting goal
    Crafting {
        target_item: ItemTemplate,
        server_goal: u32,
        personal_contribution_cap: u32,
    },

    // Mystery: Unknown rewards, hidden mechanics
    Mystery {
        clues: Vec<Clue>,
        solution_reward: Reward,
    },
}
```

**Event Drop Mechanics:**

```rust
struct EventDrop {
    item: ItemTemplate,

    // Drop method
    drop_method: EventDropMethod,

    // Scarcity controls
    max_drops_server: Option<u32>,     // Hard cap for entire event
    max_drops_per_player: Option<u32>, // Per-player cap
    drops_remaining: u32,

    // Time-based availability
    available_hours: Option<Vec<u32>>, // Only drops during specific hours

    // Participation requirement
    min_event_contribution: u32,
}

enum EventDropMethod {
    // Drops from event activities
    ActivityDrop { base_rate: f64 },

    // Drops from event boss
    BossDrop { damage_threshold: u32 },

    // Drops from event collection turn-in
    TurnInReward { items_per_drop: u32 },

    // Random airdrop to active participants
    RandomAirdrop { interval_minutes: u32 },

    // First-come-first-served from event location
    LocationSpawn { respawn_minutes: u32 },

    // Awarded based on leaderboard position
    LeaderboardReward { top_percent: f32 },
}
```

### 6.3 Limited Edition Drops

Items that can NEVER be obtained again:

```rust
struct LimitedEdition {
    // Identification
    edition_id: EditionId,
    edition_name: String,
    edition_number: u32,         // "Edition 1 of 100"
    total_editions: u32,

    // Tracking
    drop_date: u64,
    drop_event: Option<EventId>,
    drop_season: Option<Season>,

    // Scarcity proof
    editions_dropped: u32,
    editions_existing: u32,      // Accounts for destroyed items
    editions_held: HashMap<PlayerId, u32>,

    // Value tracking
    last_sale_price: u64,
    average_sale_price: f64,
    floor_price: u64,            // Lowest current listing
}

// Registry of all limited editions
struct LimitedEditionRegistry {
    editions: HashMap<EditionId, LimitedEdition>,

    // Verification that no new editions can be created
    edition_sealed: HashMap<EditionId, bool>,
    seal_timestamp: HashMap<EditionId, u64>,
    seal_block_hash: HashMap<EditionId, String>, // Game state hash at seal
}
```

**Limited Edition Categories:**

```
NEVER-RETURNING DROPS
──────────────────────────────────────────────────────────────────

1. LAUNCH EDITIONS
   - "Founder's" prefix items (first 30 days only)
   - First kill of each legendary beast (server-first)
   - Day-one login rewards
   - Beta tester exclusives

2. ANNUAL EDITIONS
   - "2024 Harvest" seasonal items
   - "Midsummer 2024" event rewards
   - Yearly anniversary items
   - Real-world holiday tie-ins

3. MILESTONE EDITIONS
   - Server population milestones (1000 players, 10000 players)
   - Content update celebrations
   - Community achievement rewards
   - Developer appreciation items

4. SINGULAR EDITIONS (One per server EVER)
   - First Primordial drop
   - Server-first achievements
   - Competition grand prizes
   - Hidden discovery rewards

5. RETIRED EDITIONS
   - Items removed from drop tables
   - Nerfed/buffed items (original version preserved)
   - Replaced content memorabilia
   - "Legacy" versions of updated items
```

---

## Part VII: Progression Integration

### 7.1 Perk Tree: Fortune Branch

Luck and drop-focused perks:

```
FORTUNE PERK TREE
══════════════════════════════════════════════════════════════════

Tier 1 (5 skill points each)
├─ Lucky Find: +2% base luck
├─ Sharp Eyes: +5% chance to spot rare resource nodes
└─ Fortunate Soul: +10% gold from all sources

Tier 2 (10 skill points each, requires 1 Tier 1)
├─ Treasure Sense: Nearby chests glow faintly
├─ Quality Affinity: +5% item quality on all drops
├─ Lucky Strike: Critical hits have +1% drop rate bonus
└─ Collector's Eye: See item rarity before pickup

Tier 3 (15 skill points each, requires 2 Tier 2)
├─ Fortune's Favor: Pity system charges 25% faster
├─ Lucky Streak: Each drop without rare increases luck by 0.5%
├─ Golden Touch: +15% value from selling items
└─ Serendipity: 5% chance for bonus drop on any loot

Tier 4 (25 skill points each, requires 2 Tier 3)
├─ Mythic Magnetism: +50% Legendary+ drop rate
├─ Perfect Timing: Seasonal exclusive drop windows extended 1 hour
├─ Jackpot: 1% chance for double quantity on all drops
└─ Chosen by Fate: Guaranteed minimum Rare quality from bosses

Tier 5 (50 skill points, requires all Tier 4)
└─ Avatar of Fortune: Once per real-time week, upgrade any drop
   by one rarity tier. Also: +25% all luck bonuses.

LUCK CALCULATION WITH PERKS
──────────────────────────────────────────────────────────────────
base_luck = 0.0
perk_luck = sum of all luck perks
gear_luck = sum of equipped item luck bonuses
consumable_luck = active potion/food luck bonus
event_luck = current event luck modifier

final_luck = (base_luck + perk_luck + gear_luck + consumable_luck)
             * (1.0 + event_luck)
             * avatar_of_fortune_multiplier
```

### 7.2 Skill Mastery Bonuses

Each skill provides drop bonuses at milestones:

```rust
struct SkillMasteryBonus {
    skill: Skill,
    level: u32,
    bonus: MasteryBonus,
}

enum MasteryBonus {
    LuckBonus(f64),
    QualityBonus(f64),
    DropRateBonus(f64),
    ExclusiveUnlock(ItemTemplate),
    VariantUnlock(VariantClass),
}

// Example: Hunting skill milestones
const HUNTING_MASTERY: &[SkillMasteryBonus] = &[
    SkillMasteryBonus { skill: Skill::Hunting, level: 10,
        bonus: MasteryBonus::QualityBonus(0.05) },  // +5% pelt quality

    SkillMasteryBonus { skill: Skill::Hunting, level: 25,
        bonus: MasteryBonus::ExclusiveUnlock(ItemTemplate::HuntersTalisman) },

    SkillMasteryBonus { skill: Skill::Hunting, level: 50,
        bonus: MasteryBonus::DropRateBonus(0.10) },  // +10% hunting drops

    SkillMasteryBonus { skill: Skill::Hunting, level: 75,
        bonus: MasteryBonus::VariantUnlock(VariantClass::Masterwork) },

    SkillMasteryBonus { skill: Skill::Hunting, level: 99,
        bonus: MasteryBonus::ExclusiveUnlock(ItemTemplate::MasterHuntersBow) },

    SkillMasteryBonus { skill: Skill::Hunting, level: 100,
        bonus: MasteryBonus::LuckBonus(0.15) },  // +15% luck for hunting only
];
```

### 7.3 Achievement Unlocks

Achievements that permanently affect drops:

```rust
struct DropAchievement {
    id: AchievementId,
    name: String,
    requirement: AchievementRequirement,
    reward: DropReward,
}

enum DropReward {
    PermanentLuckBonus(f64),
    UnlockDropSource(DropSource),
    UnlockItemVariant(VariantClass),
    UnlockPrefix(Prefix),
    UnlockSuffix(Suffix),
    GuaranteedDrop(ItemTemplate),
    TitleWithBonus(Title, f64),  // Title that grants luck when displayed
}

const DROP_ACHIEVEMENTS: &[DropAchievement] = &[
    // Collection achievements
    DropAchievement {
        id: AchievementId::FullSetCollector,
        name: "Full Set Collector",
        requirement: AchievementRequirement::CollectFullSet(ItemSet::AnyRare),
        reward: DropReward::PermanentLuckBonus(0.01),
    },

    // Rarity achievements
    DropAchievement {
        id: AchievementId::FirstLegendary,
        name: "Legendary Finder",
        requirement: AchievementRequirement::ObtainRarity(Rarity::Legendary),
        reward: DropReward::UnlockItemVariant(VariantClass::Blessed),
    },

    DropAchievement {
        id: AchievementId::MythicHunter,
        name: "Mythic Hunter",
        requirement: AchievementRequirement::ObtainCount(Rarity::Mythic, 10),
        reward: DropReward::UnlockPrefix(Prefix::Godtouched),
    },

    // Trading achievements
    DropAchievement {
        id: AchievementId::MerchantPrince,
        name: "Merchant Prince",
        requirement: AchievementRequirement::TotalTradeValue(10_000_000),
        reward: DropReward::TitleWithBonus(Title::MerchantPrince, 0.03),
    },

    // Sacrifice achievements
    DropAchievement {
        id: AchievementId::Offerings,
        name: "Generous Offerings",
        requirement: AchievementRequirement::SacrificeValue(1_000_000),
        reward: DropReward::UnlockDropSource(DropSource::SacrificeBlessing),
    },
];
```

### 7.4 Online Prowess System

Competitive multiplayer affects drops:

```rust
struct OnlineRanking {
    player_id: PlayerId,

    // PvP rating (if applicable)
    pvp_rating: u32,
    pvp_rank: Rank,

    // Leaderboard positions
    hunting_rank: u32,
    trading_rank: u32,
    exploration_rank: u32,
    collection_rank: u32,

    // Seasonal performance
    season_points: u32,
    season_tier: SeasonTier,

    // Lifetime statistics
    lifetime_legendary_drops: u32,
    lifetime_trades: u32,
    lifetime_gold_earned: u64,
}

enum SeasonTier {
    Bronze,      // No bonus
    Silver,      // +2% luck
    Gold,        // +5% luck, exclusive drops
    Platinum,    // +8% luck, exclusive cosmetics
    Diamond,     // +12% luck, title, exclusive mount
    Champion,    // +15% luck, all above, legacy statue
}

struct RankRewards {
    tier: SeasonTier,
    luck_bonus: f64,
    exclusive_drops: Vec<ItemTemplate>,
    cosmetics: Vec<CosmeticTemplate>,
    title: Option<Title>,
    special_rewards: Vec<SpecialReward>,
}

// Season end rewards
fn calculate_season_rewards(player: &OnlineRanking) -> Vec<Reward> {
    let tier_rewards = get_tier_rewards(player.season_tier);
    let mut rewards = tier_rewards.clone();

    // Top 100 in any category gets extra rewards
    if player.hunting_rank <= 100 {
        rewards.push(Reward::ExclusiveItem(ItemTemplate::MasterHunterTrophy));
    }
    if player.trading_rank <= 100 {
        rewards.push(Reward::ExclusiveItem(ItemTemplate::MerchantGuildBadge));
    }

    // Top 10 get named items
    if player.hunting_rank <= 10 {
        rewards.push(Reward::NamedItem(
            ItemTemplate::LegendaryBow,
            format!("{}s Legendary Bow", player.name)
        ));
    }

    // #1 gets server-unique item
    if player.hunting_rank == 1 {
        rewards.push(Reward::SingularItem(
            ItemTemplate::ChampionsBow,
            format!("Season {} Champion", current_season())
        ));
    }

    rewards
}
```

---

## Part VIII: Infinite Scalability Architecture

### 8.1 Procedural Item Generation

Items generated from seeds, not stored:

```rust
struct ItemSeed {
    // 256-bit seed determines everything
    seed: [u64; 4],
}

impl ItemSeed {
    fn generate(&self) -> Item {
        let mut rng = SeededRng::new(self.seed);

        Item {
            dna: ItemDNA {
                genome: rng.next_u64(),
                base_type: ItemBaseType::from_seed(rng.next_u64()),
                rarity: Rarity::from_roll(rng.next_f64()),
                quality: rng.next_u32() % 101,
                prefix: self.roll_prefix(&mut rng),
                suffix: self.roll_suffix(&mut rng),
                implicit: self.roll_implicit(&mut rng),
                variant_class: VariantClass::from_seed(rng.next_u64()),
                // ...
            },
            // ...
        }
    }

    // Deterministic: same seed always produces same item
    fn verify(&self, item: &Item) -> bool {
        self.generate().dna == item.dna
    }
}

// Storage is just seeds + provenance
struct ItemStorage {
    seed: ItemSeed,
    provenance: ItemProvenance,
    current_durability: u32,
    modifications: Vec<Modification>,
}
```

### 8.2 Variant Expansion System

New variants can be added without breaking existing items:

```rust
struct VariantRegistry {
    // Core variants (never change)
    core_variants: Vec<VariantDefinition>,

    // Extension variants (added over time)
    extension_variants: Vec<VariantDefinition>,

    // Version tracking
    registry_version: u32,
}

impl VariantRegistry {
    fn add_variant(&mut self, variant: VariantDefinition) {
        // New variants get new IDs, never reuse
        variant.id = self.next_variant_id();
        self.extension_variants.push(variant);
        self.registry_version += 1;

        // Old items unaffected - they reference old IDs
        // New drops can roll new variants
    }
}

// Forward compatibility: unknown variants render as "Unknown Variant"
fn render_variant(variant_id: VariantId, registry: &VariantRegistry) -> String {
    registry.get(variant_id)
        .map(|v| v.name.clone())
        .unwrap_or("Unknown Variant".to_string())
}
```

### 8.3 Prefix/Suffix Scaling

Modifiers scale with content updates:

```rust
struct ModifierTier {
    tier: u32,
    min_item_level: u32,
    stat_multiplier: f64,
}

struct PrefixDefinition {
    id: PrefixId,
    name: String,
    base_effect: Effect,
    tiers: Vec<ModifierTier>,

    // Can add new tiers without changing existing items
    current_max_tier: u32,
}

impl PrefixDefinition {
    fn add_tier(&mut self, tier: ModifierTier) {
        // Existing items keep their tier
        // New content can drop higher tiers
        self.tiers.push(tier);
        self.current_max_tier = tier.tier;
    }
}
```

### 8.4 Economy Monitoring & Balancing

Real-time economy health tracking:

```rust
struct EconomyMonitor {
    // Gold metrics
    total_gold_supply: u64,
    gold_created_24h: u64,
    gold_destroyed_24h: u64,      // Taxes, fees, sinks
    gold_velocity: f64,           // Transactions per gold per day

    // Item metrics
    items_created_24h: u32,
    items_destroyed_24h: u32,
    rarity_distribution: HashMap<Rarity, u32>,

    // Market metrics
    average_listing_time: f64,
    bid_ask_spread: f64,
    liquidity_score: f64,

    // Inflation indicators
    price_index: f64,             // Basket of common items
    price_change_7d: f64,

    // Alerts
    alerts: Vec<EconomyAlert>,
}

enum EconomyAlert {
    InflationHigh { rate: f64 },
    DeflationHigh { rate: f64 },
    GoldPoolLow { amount: u64 },
    RarityImbalance { rarity: Rarity, deviation: f64 },
    ExploitDetected { pattern: String },
}

// Auto-balancing levers
struct EconomyLevers {
    // Drop rate multipliers (1.0 = normal)
    global_drop_multiplier: f64,
    rarity_multipliers: HashMap<Rarity, f64>,

    // Gold generation/sink rates
    gold_source_multiplier: f64,
    tax_rate_multiplier: f64,

    // Event spawn rates
    event_frequency: f64,
    boss_spawn_rate: f64,
}

impl EconomyMonitor {
    fn recommend_adjustments(&self) -> Vec<LeverAdjustment> {
        let mut adjustments = Vec::new();

        // Too much gold = increase sinks
        if self.gold_created_24h > self.gold_destroyed_24h * 1.5 {
            adjustments.push(LeverAdjustment {
                lever: Lever::TaxRate,
                change: 0.05, // +5% tax
                reason: "Gold inflation detected".to_string(),
            });
        }

        // Too many legendaries = reduce drop rate
        let legendary_ratio = self.rarity_distribution[&Rarity::Legendary] as f64
            / self.items_created_24h as f64;
        if legendary_ratio > 0.01 { // More than 1%
            adjustments.push(LeverAdjustment {
                lever: Lever::RarityMultiplier(Rarity::Legendary),
                change: -0.1, // -10% legendary drops
                reason: "Legendary oversupply".to_string(),
            });
        }

        adjustments
    }
}
```

---

## Part IX: Technical Implementation

### 9.1 Database Schema

```sql
-- Item storage (minimal, seed-based)
CREATE TABLE items (
    item_id UUID PRIMARY KEY,
    seed BYTEA NOT NULL,           -- 256-bit seed
    owner_id UUID NOT NULL,

    -- Mutable state
    current_durability INT,
    location JSONB,                -- Inventory slot, equipped, etc.

    -- Provenance (immutable)
    provenance JSONB NOT NULL,

    -- Modifications (enchants, etc.)
    modifications JSONB DEFAULT '[]',

    -- Indexes for market
    rarity SMALLINT GENERATED ALWAYS AS (calculate_rarity(seed)),
    base_type INT GENERATED ALWAYS AS (calculate_base_type(seed)),

    created_at TIMESTAMP DEFAULT NOW()
);

-- Auction listings
CREATE TABLE auction_listings (
    listing_id UUID PRIMARY KEY,
    item_id UUID REFERENCES items(item_id),
    seller_id UUID NOT NULL,

    starting_bid BIGINT NOT NULL,
    buyout_price BIGINT,
    current_bid BIGINT DEFAULT 0,
    current_bidder_id UUID,

    duration_hours INT NOT NULL,
    started_at TIMESTAMP DEFAULT NOW(),
    ends_at TIMESTAMP NOT NULL,
    extension_count INT DEFAULT 0,

    status VARCHAR(20) DEFAULT 'active'
);

-- Price history (aggregated)
CREATE TABLE price_history (
    item_template_id INT NOT NULL,
    time_bucket TIMESTAMP NOT NULL,  -- Hourly buckets

    avg_price BIGINT,
    min_price BIGINT,
    max_price BIGINT,
    volume INT,

    PRIMARY KEY (item_template_id, time_bucket)
);

-- Limited editions registry
CREATE TABLE limited_editions (
    edition_id UUID PRIMARY KEY,
    edition_name VARCHAR(255) NOT NULL,
    edition_number INT NOT NULL,
    total_editions INT NOT NULL,

    drop_timestamp TIMESTAMP,
    drop_event_id UUID,

    editions_dropped INT DEFAULT 0,
    editions_existing INT DEFAULT 0,
    is_sealed BOOLEAN DEFAULT FALSE,
    sealed_at TIMESTAMP,

    UNIQUE(edition_name, edition_number)
);

-- Player drop statistics (for pity system)
CREATE TABLE player_drop_stats (
    player_id UUID PRIMARY KEY,

    karma_accumulated FLOAT DEFAULT 0,
    drops_since_rare INT DEFAULT 0,
    drops_since_epic INT DEFAULT 0,
    drops_since_legendary INT DEFAULT 0,

    lifetime_drops BIGINT DEFAULT 0,
    lifetime_legendary_drops INT DEFAULT 0,
    lifetime_mythic_drops INT DEFAULT 0,
    lifetime_primordial_drops INT DEFAULT 0,

    updated_at TIMESTAMP DEFAULT NOW()
);

-- Economy snapshots
CREATE TABLE economy_snapshots (
    snapshot_time TIMESTAMP PRIMARY KEY,

    total_gold_supply BIGINT,
    gold_created_24h BIGINT,
    gold_destroyed_24h BIGINT,

    items_created_24h INT,
    items_destroyed_24h INT,

    rarity_distribution JSONB,
    price_index FLOAT,

    lever_settings JSONB
);
```

### 9.2 Drop Calculation Flow

```rust
// Main drop function
pub fn calculate_drop(
    source: DropSource,
    player: &Player,
    world_state: &WorldState,
    rng_seed: u64,
) -> Vec<Item> {
    let mut rng = SeededRng::new(rng_seed);
    let mut drops = Vec::new();

    // Build drop context
    let context = DropContext {
        source: source.clone(),
        player_luck: calculate_player_luck(player),
        player_skills: player.skills.clone(),
        player_perks: player.active_perks.clone(),
        karma: player.drop_stats.karma_accumulated,

        time_of_day: world_state.time_of_day,
        moon_phase: world_state.moon_phase,
        weather: world_state.weather,
        season: world_state.current_season,
        active_events: world_state.active_events.clone(),

        session_time: player.current_session_duration(),
    };

    // Get drop table for source
    let drop_table = get_drop_table(&source);

    // Roll for each potential drop
    for entry in drop_table.entries {
        if should_drop(&entry, &context, &mut rng) {
            let item = generate_item(&entry, &context, &mut rng);
            drops.push(item);
        }
    }

    // Apply quantity modifiers
    apply_quantity_bonuses(&mut drops, &context);

    // Update player karma
    update_karma(player, &drops);

    // Record for economy monitoring
    record_drops(&drops);

    drops
}

fn should_drop(
    entry: &DropTableEntry,
    context: &DropContext,
    rng: &mut SeededRng,
) -> bool {
    let base_rate = entry.base_drop_rate;

    // Apply all modifiers
    let modified_rate = base_rate
        * (1.0 + context.player_luck)
        * get_skill_modifier(entry, context)
        * get_perk_modifier(entry, context)
        * get_celestial_modifier(context)
        * get_event_modifier(entry, context)
        * get_karma_modifier(context);

    rng.next_f64() < modified_rate
}

fn generate_item(
    entry: &DropTableEntry,
    context: &DropContext,
    rng: &mut SeededRng,
) -> Item {
    // Generate seed
    let seed = ItemSeed::new(rng.next_u64_array());

    // Calculate rarity with all modifiers
    let rarity = calculate_rarity_with_context(&seed, context, rng);

    // Build provenance
    let provenance = ItemProvenance {
        first_owner_id: context.player.id,
        first_owner_name: context.player.name.clone(),
        discovery_timestamp: current_timestamp(),
        discovery_location: context.player.position,
        discovery_method: context.source.clone(),
        // ... fill from context
    };

    // Generate item from seed with rarity override
    let mut item = seed.generate();
    item.dna.rarity = rarity;
    item.provenance = provenance;

    item
}
```

### 9.3 Market Matching Engine

```rust
pub struct MatchingEngine {
    order_books: HashMap<ItemTemplateId, OrderBook>,
    pending_trades: Vec<PendingTrade>,
}

impl MatchingEngine {
    pub fn process_tick(&mut self) -> Vec<ExecutedTrade> {
        let mut executed = Vec::new();

        for (_, book) in &mut self.order_books {
            // Match orders
            let trades = book.match_orders();

            for trade in trades {
                // Verify both parties still have items/gold
                if self.verify_trade(&trade) {
                    // Execute atomically
                    let result = self.execute_trade(trade);
                    executed.push(result);
                }
            }
        }

        // Process pending trades (P2P trades awaiting confirmation)
        self.process_pending_trades(&mut executed);

        // Update price history
        self.update_price_history(&executed);

        executed
    }

    fn execute_trade(&mut self, trade: Trade) -> ExecutedTrade {
        // Atomic transaction
        let transaction = Transaction::new();

        // Transfer item
        transaction.transfer_item(trade.item_id, trade.seller_id, trade.buyer_id);

        // Calculate fees
        let fees = calculate_fees(&trade);
        let seller_receives = trade.price - fees.total;

        // Transfer gold
        transaction.transfer_gold(trade.buyer_id, trade.seller_id, seller_receives);
        transaction.destroy_gold(fees.total); // Deflationary sink

        // Update provenance
        transaction.update_provenance(trade.item_id, OwnershipRecord {
            owner: trade.buyer_id,
            acquired: current_timestamp(),
            method: AcquisitionMethod::Purchase,
            price_paid: trade.price,
        });

        // Commit
        transaction.commit();

        ExecutedTrade {
            trade,
            fees,
            executed_at: current_timestamp(),
        }
    }
}
```

---

## Part X: UI/UX Specifications

### 10.1 Item Tooltip

```
┌─────────────────────────────────────────────────────┐
│ ★ GODTOUCHED RECURVE BOW OF LEGEND ★               │  <- Name (colored by rarity)
│ Legendary Masterwork Bow                            │  <- Type line
├─────────────────────────────────────────────────────┤
│ Quality: Exceptional (87/100) ████████░░            │  <- Quality bar
│ Durability: 234/250                                 │
├─────────────────────────────────────────────────────┤
│ ⚔ Base Damage: 45-62                               │  <- Base stats
│ ⚡ Attack Speed: 1.4/sec                            │
│ 🎯 Accuracy: +15%                                   │
├─────────────────────────────────────────────────────┤
│ ✧ Godtouched: +25% damage to legendary beasts      │  <- Prefix effect
│ ✧ Of Legend: +10% chance for legendary drops       │  <- Suffix effect
│ ◈ Innate: Arrows travel 20% faster                 │  <- Implicit
├─────────────────────────────────────────────────────┤
│ 🌙 Drops only during Full Moon                     │  <- Special conditions
│ 🏆 Season 3 Champion Exclusive                     │  <- Limited edition tag
├─────────────────────────────────────────────────────┤
│ PROVENANCE                                          │
│ Found by: WolfHunter_42                            │
│ Location: Misty Valley (234, 891)                  │
│ Method: Perfect Kill - Legendary White Stag        │
│ Date: March 15, 2024 3:42 AM                       │
│ Times Traded: 7                                     │
│ Total Trade Value: 2,847,000 gold                  │
├─────────────────────────────────────────────────────┤
│ 💰 Market Value: ~350,000 - 420,000 gold           │  <- Price estimate
│ 📈 +15% from last week                             │  <- Price trend
└─────────────────────────────────────────────────────┘
```

### 10.2 Market Interface

```
╔════════════════════════════════════════════════════════════════╗
║ 🏛 ROANOKE MARKETPLACE                              [Gold: 45,230] ║
╠════════════════════════════════════════════════════════════════╣
║ [Search: ___________] [Filters ▼] [My Listings] [My Bids]      ║
╠════════════════════════════════════════════════════════════════╣
║                                                                  ║
║ CATEGORY          RARITY         PRICE RANGE      SORT BY       ║
║ ☐ Weapons         ☐ Common       Min: [____]      [Price ▼    ] ║
║ ☐ Armor           ☐ Uncommon     Max: [____]                    ║
║ ☐ Tools           ☐ Rare                                        ║
║ ☐ Materials       ☑ Epic         SPECIAL                        ║
║ ☐ Fossils         ☑ Legendary    ☐ Limited Edition             ║
║ ☐ Consumables     ☐ Mythic       ☐ Event Exclusive             ║
║ ☐ Cosmetics       ☐ Primordial   ☐ Perfect Quality             ║
║                                                                  ║
╠════════════════════════════════════════════════════════════════╣
║ LISTINGS (47 results)                                           ║
╠════════════════════════════════════════════════════════════════╣
║ ┌──────────────────────────────────────────────────────────┐   ║
║ │ [Icon] Blazing Longbow of the Hunt        LEGENDARY      │   ║
║ │        Quality: 92  |  Durability: 98%                   │   ║
║ │        Current Bid: 125,000  |  Buyout: 180,000         │   ║
║ │        Time Left: 2h 34m  |  Bids: 7                    │   ║
║ │        [View] [Bid] [Buyout]                            │   ║
║ └──────────────────────────────────────────────────────────┘   ║
║ ┌──────────────────────────────────────────────────────────┐   ║
║ │ [Icon] Permafrost Hunting Knife           EPIC           │   ║
║ │        Quality: 78  |  Durability: 100%                  │   ║
║ │        Current Bid: 45,000  |  Buyout: 62,000           │   ║
║ │        Time Left: 14h 22m  |  Bids: 3                   │   ║
║ │        [View] [Bid] [Buyout]                            │   ║
║ └──────────────────────────────────────────────────────────┘   ║
║                                                                  ║
║ [< Prev]  Page 1 of 5  [Next >]                                ║
╚════════════════════════════════════════════════════════════════╝
```

### 10.3 Drop Notification

```
┌─────────────────────────────────────────┐
│     ✦ LEGENDARY DROP ✦                 │
│                                         │
│   [Animated Item Icon with Glow]        │
│                                         │
│   VORPAL HUNTING SPEAR                  │
│   of the Godslayer                      │
│                                         │
│   Quality: PERFECT (100)                │
│   Variant: Masterwork                   │
│                                         │
│   "First perfect Vorpal Spear on        │
│    this server!"                        │
│                                         │
│   [Equip] [Stash] [Inspect]            │
└─────────────────────────────────────────┘

// Server-wide announcement for Mythic+
╔═══════════════════════════════════════════════════════════════╗
║ 🌟 MYTHIC DROP 🌟                                              ║
║ Player "WolfHunter_42" has obtained:                          ║
║ PRIMORDIAL BOW OF THE FIRST PEOPLE                           ║
║ This is drop #3 of this item type on the server!             ║
╚═══════════════════════════════════════════════════════════════╝
```

---

## Part XI: Anti-Exploit Measures

### 11.1 Drop Rate Verification

```rust
struct DropVerification {
    // Server-side seed generation
    fn generate_drop_seed(player: &Player, action: &Action) -> DropSeed {
        // Combine unpredictable server state with action
        let seed_material = [
            server_secret_key,
            player.id.as_bytes(),
            action.timestamp.to_bytes(),
            action.action_id.as_bytes(),
            previous_block_hash, // Chain of actions
        ].concat();

        DropSeed::from_hash(sha256(&seed_material))
    }

    // Verify drop was legitimate
    fn verify_drop(drop: &Drop, player: &Player, action: &Action) -> bool {
        let expected_seed = generate_drop_seed(player, action);
        drop.seed == expected_seed
    }
}
```

### 11.2 Market Manipulation Detection

```rust
struct ManipulationDetector {
    fn detect_wash_trading(&self, trades: &[Trade]) -> Vec<Alert> {
        // Detect trades between related accounts
        let mut alerts = Vec::new();

        for trade in trades {
            // Check for circular trading
            if self.is_circular_trade(trade) {
                alerts.push(Alert::WashTrading(trade.id));
            }

            // Check for price manipulation
            if self.is_price_anomaly(trade) {
                alerts.push(Alert::PriceManipulation(trade.id));
            }
        }

        alerts
    }

    fn detect_botting(&self, player: &Player) -> Option<Alert> {
        // Inhuman patterns
        let patterns = analyze_action_patterns(player);

        if patterns.actions_per_minute > 200 {
            return Some(Alert::PossibleBot(player.id));
        }

        if patterns.timing_variance < 0.01 { // Too consistent
            return Some(Alert::PossibleBot(player.id));
        }

        None
    }
}
```

### 11.3 RMT Prevention

```rust
struct RMTDetection {
    fn detect_rmt_patterns(&self, trade: &Trade) -> RiskScore {
        let mut score = 0.0;

        // Severely unbalanced trade
        if trade.gold_ratio() > 100.0 { // 100:1 gold to item value
            score += 0.5;
        }

        // New account receiving high value
        if trade.recipient.account_age_days < 7
            && trade.total_value > 100_000 {
            score += 0.3;
        }

        // Pattern of one-way trades
        if self.one_way_trade_count(trade.sender) > 10 {
            score += 0.2;
        }

        RiskScore(score)
    }
}
```

---

## Appendix A: Complete Prefix List (324 total)

[Abbreviated - full list would be 50+ pages]

```
ELEMENTAL PREFIXES (36)
Tier 1: Chilled, Heated, Sparked, Dampened, Frozen, Burning,
        Shocking, Soaked, Frosty, Smoldering
Tier 2: Freezing, Blazing, Crackling, Drowning, Glacial,
        Infernal, Voltaic, Torrential, Arctic, Volcanic
Tier 3: Permafrost, Hellfire, Thunderstruck, Abyssal,
        Avalanche, Magmatic, Tempest, Tsunami
Tier 4: Absolute Zero, Supernova, Godthunder, Leviathan's,
        Eternal Winter's, Phoenix's, Zeus's, Poseidon's

[Continue for all 324 prefixes across 6 categories...]
```

## Appendix B: Complete Suffix List (216 total)

[Abbreviated - full list would be 30+ pages]

```
ATTRIBUTE SUFFIXES (36)
Tier 1: of Strength (+5-10), of Agility (+5-10), of Vitality (+5-10)...
Tier 2: of Power (+15-25), of Grace (+15-25), of Endurance (+15-25)...
Tier 3: of the Giant (+30-40), of the Wind (+30-40), of the Mountain (+30-40)...
Tier 4: of the Titan (+45-50 all physical), of the Colossus (+50 single stat)...

[Continue for all 216 suffixes across 7 categories...]
```

## Appendix C: Drop Tables by Source

[Comprehensive drop tables for every creature, dig site, chest type, and event]

## Appendix D: Economy Balance Targets

```
TARGET METRICS (Per 1000 Active Players Per Day)
──────────────────────────────────────────────────────────────────
Gold Created:       500,000 - 700,000
Gold Destroyed:     400,000 - 600,000 (target: net neutral)

Items Created:      5,000 - 8,000
Items Destroyed:    3,000 - 5,000 (durability, crafting, sacrifice)

Rarity Distribution of Created Items:
- Crude:      40-50%
- Common:     25-35%
- Uncommon:   12-18%
- Rare:       5-8%
- Epic:       1.5-3%
- Legendary:  0.3-0.8%
- Mythic:     0.05-0.15%
- Primordial: <0.01%

Market Metrics:
- Average listing time: 12-24 hours
- Bid/ask spread: <15%
- Daily trading volume: 30-50% of gold supply
```

---

## Part XII: Currency Integration

This section bridges the loot system with the dual-currency economy defined in `DECADE_FINANCIAL_ROADMAP.md`.

### 12.1 Wampum (WPM) Generation from Drops

Every drop generates Wampum based on item properties:

```rust
struct DropToWampumConversion {
    // Base WPM by rarity
    fn calculate(&self, item: &Item, context: &DropContext) -> u64 {
        let base = match item.rarity {
            Rarity::Crude => 5,
            Rarity::Common => 20,
            Rarity::Uncommon => 100,
            Rarity::Rare => 500,
            Rarity::Epic => 2_500,
            Rarity::Legendary => 15_000,
            Rarity::Mythic => 100_000,
            Rarity::Primordial => 1_000_000,
        };

        // Quality multiplier (0.5x - 1.5x)
        let quality_mult = 0.5 + (item.quality as f64 / 100.0);

        // Kill quality bonus
        let kill_mult = match context.kill_quality {
            Some(KillQuality::Legendary) => 2.0,
            Some(KillQuality::Perfect) => 1.5,
            Some(KillQuality::Clean) => 1.25,
            _ => 1.0,
        };

        // First-time discovery bonus
        let discovery_mult = if context.is_first_of_type { 3.0 }
            else if context.is_server_first { 5.0 }
            else { 1.0 };

        (base as f64 * quality_mult * kill_mult * discovery_mult) as u64
    }
}
```

### 12.2 Tobacco (TBC) Drop Integration

TBC drops are extremely rare and tied to exceptional achievements:

```
TBC DROP TRIGGERS
════════════════════════════════════════════════════════════════════════════

TRIGGER                          TBC REWARD      PROBABILITY
───────────────────────────────────────────────────────────────────────────
Legendary item drop              10 TBC          10% chance on drop
Mythic item drop                 100 TBC         25% chance on drop
Primordial item drop             1,000 TBC       100% guaranteed
Server-first legendary beast     500 TBC         Guaranteed
Perfect quality + Legendary      50 TBC          Guaranteed
Season leaderboard top 10        100-1000 TBC    End of season
Event participation              1-10 TBC        Per milestone
```

### 12.3 Marketplace Fee Distribution

Transaction fees fund the economy:

```rust
struct FeeDistribution {
    // Where fees go
    fn distribute(&self, fee: u64) {
        let burned = (fee as f64 * 0.40) as u64;      // 40% burned (deflation)
        let treasury = (fee as f64 * 0.30) as u64;    // 30% to treasury
        let rewards = (fee as f64 * 0.20) as u64;     // 20% to player rewards
        let ecosystem = (fee as f64 * 0.10) as u64;   // 10% to creators

        self.burn(burned);
        self.treasury.deposit(treasury);
        self.reward_pool.deposit(rewards);
        self.creator_fund.deposit(ecosystem);
    }
}
```

### 12.4 Item-to-Currency Sinks

Items destroyed to generate deflationary pressure:

```
DESTRUCTION REWARDS
════════════════════════════════════════════════════════════════════════════

Sacrifice Altar:     Destroy items for permanent stat bonuses
                     Value threshold unlocks tiers (see Part IV)
                     TBC bonus for sacrificing Legendary+

Crafting Consumption: Epic+ items consumed to craft Legendary+
                      Failure chance creates additional sinks
                      Success grants bonus WPM

Durability Death:     Items at 0 durability destroyed permanently
                      High-value items generate "memorial" WPM
                      Primordials never fully destroy (become "ruined")
```

---

## Part XIII: Institutional Data Feeds

For institutional investors (see `DECADE_FINANCIAL_ROADMAP.md` Part II):

### 13.1 Real-Time API Endpoints

```
PUBLIC ENDPOINTS (Free)
════════════════════════════════════════════════════════════════════════════
GET /api/v1/economy/live           Live economy metrics (5s refresh)
GET /api/v1/prices/{template_id}   Current bid/ask/last for item
GET /api/v1/rarity/distribution    Current rarity distribution
GET /api/v1/supply/wpm             WPM total supply and velocity
GET /api/v1/supply/tbc             TBC circulating and burned

LICENSED ENDPOINTS ($5K/month)
════════════════════════════════════════════════════════════════════════════
GET /api/v1/history/trades         Full trade history (paginated)
GET /api/v1/history/prices         OHLCV candles by item
GET /api/v1/analytics/whales       Large holder activity
GET /api/v1/analytics/flow         Currency flow analysis
WS  /api/v1/stream/trades          Real-time trade stream
WS  /api/v1/stream/drops           Real-time drop notifications

INSTITUTIONAL ENDPOINTS ($25K/month)
════════════════════════════════════════════════════════════════════════════
GET /api/v1/audit/supply           Cryptographic supply proof
GET /api/v1/audit/provenance       Full provenance chain verification
GET /api/v1/custody/accounts       Multi-sig account management
POST /api/v1/custody/transfer      Institutional transfers
GET /api/v1/reports/quarterly      Audit-ready quarterly reports
```

### 13.2 Price Oracle Specification

```rust
struct PriceOracle {
    // Manipulation-resistant price feeds
    fn get_price(&self, template_id: ItemTemplateId) -> OraclePrice {
        OraclePrice {
            // Time-weighted average (resistant to flash crashes)
            twap_1h: self.calculate_twap(template_id, Duration::hours(1)),
            twap_24h: self.calculate_twap(template_id, Duration::hours(24)),

            // Volume-weighted average (resistant to wash trading)
            vwap_24h: self.calculate_vwap(template_id, Duration::hours(24)),

            // Last trade (for reference)
            last_price: self.get_last_trade(template_id).price,
            last_trade_time: self.get_last_trade(template_id).timestamp,

            // Liquidity metrics
            bid_depth: self.get_bid_depth(template_id),
            ask_depth: self.get_ask_depth(template_id),
            spread_bps: self.calculate_spread_bps(template_id),

            // Confidence
            confidence: self.calculate_confidence(template_id),
            sample_size: self.get_trade_count_24h(template_id),
        }
    }
}
```

---

*Document Version: 1.1*
*Last Updated: December 2024*
*Author: Game Design / Economy Team*

*See also: `DECADE_FINANCIAL_ROADMAP.md` for investment thesis and 10-year economic strategy.*
