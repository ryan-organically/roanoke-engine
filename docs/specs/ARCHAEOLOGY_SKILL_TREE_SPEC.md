# Archaeological Skill Tree Specification

## Overview

The Archaeological Skill Tree allows players to discover, identify, and utilize ancient fossils and artifacts found throughout the Roanoke landscape. Starting with basic fossil collection (megalodon teeth, mastodon bones), players progress through increasingly sophisticated knowledge of the ancient world, unlocking crafting recipes, trade value, and mystical properties attributed to these relics.

## Historical Context

In the 1580s Roanoke era, colonists and Native Americans encountered fossils without understanding their true origins. Megalodon teeth were called "tongue stones" (glossopetrae) and believed to be serpent tongues with protective properties. Large bones were attributed to giants or mythical beasts. This skill tree captures that blend of discovery and mysticism.

---

## Skill Tree Structure

```
                        [MASTER ANTIQUARIAN]
                               |
              +----------------+----------------+
              |                                 |
       [Ancient Lore]                    [Relic Artisan]
              |                                 |
    +---------+---------+              +--------+--------+
    |                   |              |                 |
[Bone Reader]    [Stone Sage]    [Fossil Smith]   [Curio Trader]
    |                   |              |                 |
    +-------------------+              +-----------------+
              |                                 |
       [Field Scholar]                  [Keen Collector]
              |                                 |
              +----------------+----------------+
                               |
                      [Curious Eye]
                               |
              +----------------+----------------+
              |                                 |
    [Mastodon Seeker]              [Megalodon Hunter]
              |                                 |
              +----------------+----------------+
                               |
                        [NOVICE DIGGER]
                         (Starting Point)
```

---

## Tier 1: Foundation Skills

### Novice Digger (Starting Skill)
**Unlock:** Automatic at game start
**Description:** Basic awareness of unusual objects in the ground.

**Effects:**
- Can spot fossil dig sites (faint shimmer on ground)
- 25% chance to successfully extract a fossil without damage
- Unlocks basic digging action at marked sites

**Dig Sites Visible:**
- Surface fossils (partially exposed)
- Shallow deposits (1-2 ft depth)

---

## Tier 2: Specialization Branch

### Megalodon Hunter
**Unlock:** Find your first megalodon tooth
**Prerequisite:** Novice Digger
**Description:** Specialized knowledge of ancient sea creature remains.

**Effects:**
- Coastal and riverbed dig sites now visible
- 40% extraction success rate for marine fossils
- Can identify tooth size/quality (Small, Medium, Large, Exceptional)
- Unlocks: Megalodon tooth locations on beaches, riverbeds, cliff faces

**Findable Items:**
| Item | Rarity | Location |
|------|--------|----------|
| Small Megalodon Tooth | Common | Beaches, riverbeds |
| Medium Megalodon Tooth | Uncommon | Cliff erosion, deep riverbed |
| Large Megalodon Tooth | Rare | Underwater, cave deposits |
| Exceptional Megalodon Tooth | Very Rare | Ancient seabed formations |
| Fossilized Shark Vertebra | Uncommon | Coastal cliffs |
| Ancient Whale Bone Fragment | Rare | Deep coastal digs |

---

### Mastodon Seeker
**Unlock:** Find your first mastodon bone
**Prerequisite:** Novice Digger
**Description:** Knowledge of the great beasts that once roamed these lands.

**Effects:**
- Inland and bog dig sites now visible
- 40% extraction success rate for terrestrial fossils
- Can identify bone type (Rib, Tusk, Skull Fragment, Limb)
- Unlocks: Mastodon bone locations in bogs, riverbanks, caves

**Findable Items:**
| Item | Rarity | Location |
|------|--------|----------|
| Mastodon Rib Fragment | Common | Bogs, riverbanks |
| Mastodon Tusk Shard | Uncommon | Deep bog, cave floor |
| Mastodon Molar | Rare | Riverside erosion |
| Mastodon Skull Fragment | Very Rare | Cave deposits |
| Complete Mastodon Tusk | Legendary | Ancient bog preservation |
| Giant Sloth Claw | Rare | Cave systems |
| Dire Wolf Fang | Uncommon | Forest cave entrances |

---

## Tier 3: Knowledge Branch

### Curious Eye
**Unlock:** Collect 5 different fossil types
**Prerequisites:** Megalodon Hunter OR Mastodon Seeker
**Description:** Trained perception for spotting buried relics.

**Effects:**
- Dig site visibility range increased by 50%
- Dig sites glow more prominently
- 55% extraction success rate
- Can now spot artifact sites (human-made ancient objects)
- Mini-map shows nearby dig sites

**New Findable Items:**
| Item | Rarity | Location |
|------|--------|----------|
| Trilobite Fossil | Uncommon | Rocky outcrops |
| Petrified Wood | Common | Forest floor, riverbed |
| Ancient Shell Cluster | Common | Coastal areas |
| Crinoid Stem Fossils | Uncommon | Limestone areas |

---

## Tier 4: Dual Specialization

### Field Scholar
**Unlock:** Collect 10 different fossil types
**Prerequisites:** Curious Eye + (Megalodon Hunter AND Mastodon Seeker)
**Description:** Academic understanding of the ancient world.

**Effects:**
- 70% extraction success rate
- Can estimate fossil age (Older/Younger classification)
- Fossils found have +1 quality tier chance
- Deep dig sites now visible (3-4 ft depth)
- Unlocks fossil examination (detailed item descriptions)

**Examination Reveals:**
- Estimated age period
- Potential trade value
- Mystical properties (per period beliefs)
- Related specimens to seek

---

### Keen Collector
**Unlock:** Sell or trade 500 gold worth of fossils
**Prerequisites:** Curious Eye
**Description:** Eye for valuable specimens and market knowledge.

**Effects:**
- Fossil quality visible before extraction
- +25% trade value for all fossils
- Collectors and traders marked on map
- Unlocks fossil appraisal (exact value shown)
- Rare fossil spawn rate +10%

---

## Tier 5: Advanced Skills

### Bone Reader
**Unlock:** Examine 20 terrestrial fossils
**Prerequisite:** Field Scholar
**Description:** Deep understanding of ancient beasts and their mystical significance.

**Effects:**
- 85% extraction success for bone fossils
- Can craft bone-based items
- Unlocks bone divination (gameplay hints from examining bones)
- Mastodon dig sites always visible regardless of distance

**Unlocked Crafting:**
| Recipe | Materials | Effect |
|--------|-----------|--------|
| Bone Talisman | 2 Mastodon Rib + Leather Cord | +10% dig site visibility |
| Giant's Tooth Necklace | 1 Mastodon Molar + Sinew | Intimidation: Animals less likely to attack |
| Ancestral Horn | 1 Mastodon Tusk Shard + Wood | Summons nearby animals (hunting aid) |
| Dire Fang Knife | 2 Dire Wolf Fang + Wood Handle | Skinning efficiency +25% |

---

### Stone Sage
**Unlock:** Examine 20 marine fossils
**Prerequisite:** Field Scholar
**Description:** Wisdom of the ancient seas and tongue stones.

**Effects:**
- 85% extraction success for marine fossils
- Can craft tooth-based items
- Unlocks tide reading (weather/tide prediction)
- Megalodon dig sites always visible regardless of distance

**Unlocked Crafting:**
| Recipe | Materials | Effect |
|--------|-----------|--------|
| Tongue Stone Amulet | 1 Large Megalodon Tooth + Silver Wire | Poison resistance +50% |
| Serpent's Guard | 3 Small Megalodon Teeth + Leather | Snake bite immunity |
| Sea Hunter's Charm | 1 Whale Bone + 2 Shark Vertebra | Fish spawn +25% when fishing |
| Leviathan Blade | 1 Exceptional Megalodon Tooth + Iron | Powerful melee weapon |

---

### Fossil Smith
**Unlock:** Craft 10 fossil-based items
**Prerequisite:** Keen Collector
**Description:** Master craftsman of prehistoric materials.

**Effects:**
- All fossil crafting recipes available
- Crafted items +1 quality tier
- Can repair damaged fossils (restore value)
- Unlocks advanced fossil crafting

**Advanced Crafting:**
| Recipe | Materials | Effect |
|--------|-----------|--------|
| Primordial Armor (Chest) | 5 Mastodon Rib + 2 Tusk Shard + Leather | Defense +15, Intimidation |
| Giant's Pauldrons | 2 Mastodon Skull Fragment + Iron | Defense +8, Knockback resistance |
| Abyssal Shield | 1 Complete Tusk + Whale Bone | Block +20, Water breathing extended |
| Crown of Ages | Skull Fragment + 4 Exceptional Teeth | All fossil find rates +25% |

---

### Curio Trader
**Unlock:** Complete 10 fossil trades with NPCs
**Prerequisite:** Keen Collector
**Description:** Renowned dealer in curiosities and antiquities.

**Effects:**
- +50% trade value for all fossils
- Special collector NPCs seek you out
- Can commission specific fossil hunts from NPCs
- Unlocks curio shop (sell fossils from camp)

**Special Trades:**
- European collectors pay premium for "giant bones"
- Native shamans trade rare items for "spirit stones"
- Alchemists seek specific fossils for experiments

---

## Tier 6: Mastery

### Ancient Lore
**Unlock:** Max both Bone Reader and Stone Sage
**Description:** Keeper of knowledge from before human memory.

**Effects:**
- 95% extraction success rate
- All fossils found are +2 quality tiers
- Can read fossil "memories" (lore snippets about ancient creatures)
- Unlocks legendary dig sites
- Ancient creature spirits may appear as guides

**Legendary Sites:**
| Site | Location Hint | Contents |
|------|---------------|----------|
| The Graveyard of Giants | Deep forest bog | Multiple complete mastodon skeletons |
| Leviathan's Rest | Underwater cave | Massive megalodon jaw |
| The Bone Cathedral | Mountain cave | Mixed prehistoric creature remains |
| Frozen Moment | Glacier cave | Perfectly preserved specimens |

---

### Relic Artisan
**Unlock:** Max both Fossil Smith and Curio Trader
**Description:** Creator of legendary items from primordial materials.

**Effects:**
- All crafted fossil items are legendary quality
- Unique crafting recipes unlocked
- Items crafted gain mystical properties
- Can restore complete skeletons (massive trade value)

**Legendary Crafting:**
| Recipe | Materials | Effect |
|--------|-----------|--------|
| Megalodon Jaw Throne | Complete Jaw + Iron Frame | Placeable: +50% intimidation aura |
| Mastodon Bone Totem | Complete Skeleton Set | Placeable: Wards area from predators |
| Primordial Blade | Legendary tooth + Legendary bone + Meteoric Iron | Best melee weapon in game |
| Cloak of the Ancients | Giant Sloth Claw + Dire Wolf Fang + Rare Pelts | Cold immunity, stealth +30% |

---

## Tier 7: Ultimate Skill

### Master Antiquarian
**Unlock:** Complete Ancient Lore AND Relic Artisan
**Description:** The foremost authority on the ancient world. Legends say you can commune with creatures long dead.

**Effects:**
- 100% extraction success rate
- All fossils found at maximum quality
- Can sense any dig site on the entire map
- Spirit of the Mastodon companion (summonable mount)
- Spirit of the Megalodon blessing (underwater breathing, swim speed)
- Title: "Master Antiquarian" visible to other players
- Unlocks final questline: "Echoes of the Primordial World"

---

## Dig Site Mechanics

### Site Types
| Type | Depth | Tool Required | Time | Skill Visibility |
|------|-------|---------------|------|------------------|
| Surface | 0 ft | Hands | 5 sec | Novice Digger |
| Shallow | 1-2 ft | Shovel | 15 sec | Novice Digger |
| Standard | 2-3 ft | Shovel | 30 sec | Curious Eye |
| Deep | 3-4 ft | Pick + Shovel | 60 sec | Field Scholar |
| Legendary | 5+ ft | Special Tools | 120 sec | Ancient Lore |

### Extraction Mechanics
- Each dig has success chance based on skill level
- Failed extraction = fossil damaged (lower quality/value)
- Critical failure = fossil destroyed
- Weather affects dig speed (rain slows, clear speeds)
- Time of day affects visibility (dawn/dusk best)

### Site Respawn
- Common sites: 3-5 in-game days
- Uncommon sites: 7-10 in-game days
- Rare sites: 15-20 in-game days
- Legendary sites: One-time only (per save)

---

## Fossil Quality Tiers

| Tier | Name | Value Multiplier | Visual |
|------|------|------------------|--------|
| 1 | Damaged | 0.25x | Cracked, discolored |
| 2 | Poor | 0.5x | Chipped, faded |
| 3 | Common | 1.0x | Standard appearance |
| 4 | Fine | 1.5x | Clean, well-preserved |
| 5 | Exceptional | 2.5x | Perfect preservation |
| 6 | Legendary | 5.0x | Glowing, pristine |

---

## Integration with Existing Systems

### NPC Interactions
- **Village Shamans:** Trade fossils for blessings/spiritual items
- **European Traders:** Pay gold for "curiosities"
- **Craftspeople:** Use fossils in special recipes
- **Children:** Will follow player showing fossils (flavor)

### Animal System
- Predators avoid players wearing bone intimidation gear
- Spirit companions from max skill tree

### Weather System
- Rain reveals more dig sites (erosion)
- Storms can uncover legendary sites
- Fog reduces dig site visibility

### Save System
- Skill tree progress saved
- Discovered sites saved
- Fossil collection saved

---

## Skill Point Acquisition

Players earn archaeology skill points through:
| Action | Points |
|--------|--------|
| First fossil discovery | 50 |
| New fossil type discovered | 25 |
| Successful extraction | 5 |
| Perfect extraction (no damage) | 10 |
| Complete a fossil set | 100 |
| Craft fossil item | 15 |
| Complete fossil trade | 10 |
| Find legendary site | 200 |

**Points required per tier:**
- Tier 1 → Tier 2: 100 points
- Tier 2 → Tier 3: 250 points
- Tier 3 → Tier 4: 500 points
- Tier 4 → Tier 5: 1000 points
- Tier 5 → Tier 6: 2000 points
- Tier 6 → Tier 7: 5000 points

---

## Implementation Priority

### Phase 1 (Core)
- [ ] Dig site spawning system
- [ ] Basic fossil items (megalodon teeth, mastodon bones)
- [ ] Novice Digger skill and extraction mechanic
- [ ] Fossil inventory integration

### Phase 2 (Progression)
- [ ] Tier 2-3 skills
- [ ] Quality system
- [ ] Fossil examination UI
- [ ] Skill point tracking

### Phase 3 (Crafting)
- [ ] Tier 4-5 skills
- [ ] Fossil crafting recipes
- [ ] Crafted item effects
- [ ] NPC fossil trades

### Phase 4 (Mastery)
- [ ] Tier 6-7 skills
- [ ] Legendary sites
- [ ] Spirit companions
- [ ] Master Antiquarian questline

---

## Data Structures (Rust)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FossilType {
    // Marine
    MegalodonTooth { size: ToothSize },
    SharkVertebra,
    WhaleBone,
    Trilobite,
    AncientShell,
    CrinoidStem,

    // Terrestrial
    MastodonRib,
    MastodonTusk { complete: bool },
    MastodonMolar,
    MastodonSkull,
    GiantSlothClaw,
    DireWolfFang,
    PetrifiedWood,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ToothSize {
    Small,
    Medium,
    Large,
    Exceptional,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FossilQuality {
    Damaged,
    Poor,
    Common,
    Fine,
    Exceptional,
    Legendary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fossil {
    pub fossil_type: FossilType,
    pub quality: FossilQuality,
    pub examined: bool,
    pub value: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchaeologySkills {
    pub points: u32,
    pub novice_digger: bool,
    pub megalodon_hunter: bool,
    pub mastodon_seeker: bool,
    pub curious_eye: bool,
    pub field_scholar: bool,
    pub keen_collector: bool,
    pub bone_reader: bool,
    pub stone_sage: bool,
    pub fossil_smith: bool,
    pub curio_trader: bool,
    pub ancient_lore: bool,
    pub relic_artisan: bool,
    pub master_antiquarian: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DigSite {
    pub position: Vec3,
    pub site_type: DigSiteType,
    pub contents: Vec<FossilType>,
    pub discovered: bool,
    pub excavated: bool,
    pub respawn_day: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DigSiteType {
    Surface,
    Shallow,
    Standard,
    Deep,
    Legendary,
}
```

---

## Audio/Visual Feedback

### Discovery
- Glinting particle effect on dig sites
- "Discovery" musical sting when finding new fossil type
- Unique ambient sounds near legendary sites

### Extraction
- Digging sound effects (shovel, pick)
- Success: Triumphant sound + golden particles
- Failure: Cracking sound + dust particles
- Critical failure: Shattering sound

### Skill Unlock
- Skill tree "unlock" fanfare
- Screen flash effect
- Notification popup

---

## Lore Snippets (Examination Text)

**Megalodon Tooth:**
> "The natives call these 'tongue stones' - teeth of a great serpent turned to rock by the gods. European scholars believe they fall from the sky during lunar eclipses. Whatever their origin, holding one fills you with a strange sense of ancient power."

**Mastodon Bone:**
> "A bone of impossible size. The natives speak of great hairy beasts that once shook the earth, hunted by their ancestors in times beyond memory. Some colonists whisper these are the bones of Biblical giants who perished in the Flood."

**Complete Mastodon Tusk:**
> "A spiral of ivory longer than a man is tall. The weight of ages rests in your hands. Those who gaze upon it feel the echo of thundering herds crossing a frozen land that no longer exists."
