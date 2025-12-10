# Mining Skill Tree Specification

## Overview

The Mining Skill Tree develops the player's ability to locate, extract, and process mineral resources from the Roanoke wilderness. Beginning with surface collection and basic prospecting, players progress through specialized extraction techniques, eventually mastering the deep earth and its rarest treasures. Mining integrates with the crafting, economy, and faction systems.

## Design Philosophy

The colonial frontier sat atop untapped mineral wealth. Native peoples had long known where copper gleamed in stream beds and mica flaked from cliff faces. European colonists arrived hungry for gold and silver, but survival demanded they first master iron and coal. This skill tree reflects the progression from desperate scavenging to systematic extraction — the birth of American mining.

---

## The Ten Ores

### Ore Classification

| Tier | Ore | Rarity | Primary Use | Biome |
|------|-----|--------|-------------|-------|
| 1 | **Flint** | Common | Tools, fire-starting | Riverbeds, cliffs |
| 2 | **Coal** | Common | Fuel, smithing | Hills, exposed seams |
| 3 | **Bog Iron** | Uncommon | Basic metalwork | Swamps, bogs |
| 4 | **Rock Salt** | Uncommon | Preservation, trade | Caves, dry lakebeds |
| 5 | **Copper** | Uncommon | Advanced tools, decoration | Mountains, streams |
| 6 | **Mica** | Rare | Decoration, trade, windows | Granite outcrops |
| 7 | **Lead** | Rare | Ammunition, weights | Deep caves |
| 8 | **Silver** | Very Rare | Currency, jewelry | Mountain veins |
| 9 | **Gold** | Extremely Rare | Wealth, trade | Streams, deep rock |
| 10 | **Sulfur** | Legendary | Gunpowder, alchemy | Volcanic vents, caves |

---

## Ore Details

### Tier 1: Flint (Chert)
**Rarity:** Common
**Hardness:** 7 (Mohs scale)
**Tool Required:** None (hand collection) or Basic Pick

**Description:** Sharp-edged silica stone essential for fire-starting and primitive tools. Found abundantly in riverbeds and cliff faces.

**Properties:**
| Property | Value |
|----------|-------|
| Stack Size | 50 |
| Weight | 0.2 kg |
| Base Value | 1 coin |
| Spawn Rate | High |

**Extraction:**
- Surface collection: No tool required
- Cliff extraction: Basic pick, 2 swings
- Quality varies by source (river = smooth, cliff = sharp)

**Uses:**
| Recipe | Flint Required | Result |
|--------|----------------|--------|
| Flint Knife | 2 | Basic cutting tool |
| Fire Starter | 1 + Steel | Ignite fires |
| Arrowheads (10) | 3 | Ammunition |
| Flint Axe | 4 + Wood | Chopping tool |

**Yield by Skill:**
| Mining Level | Yield per Node |
|--------------|----------------|
| Novice | 1-2 |
| Apprentice | 2-3 |
| Journeyman | 3-4 |
| Expert | 4-5 |
| Master | 5-6 |

---

### Tier 2: Coal
**Rarity:** Common
**Hardness:** 2.5
**Tool Required:** Basic Pick

**Description:** Black combustite stone that burns hot and long. Essential for smithing and surviving harsh winters.

**Properties:**
| Property | Value |
|----------|-------|
| Stack Size | 100 |
| Weight | 0.3 kg |
| Base Value | 2 coins |
| Spawn Rate | Medium-High |
| Burn Time | 10 minutes |

**Extraction:**
- Exposed seams: 3-5 swings
- Underground: 5-8 swings
- Produces coal dust (debuff if not ventilated)

**Uses:**
| Recipe | Coal Required | Result |
|--------|---------------|--------|
| Forge Fuel | 5 | Smelt 1 metal ingot |
| Campfire (long) | 3 | 2 hour burn |
| Charcoal Filter | 10 | Water purification |
| Heating | 2/hour | Prevent hypothermia |

**Vein Sizes:**
| Vein Type | Coal Yield | Rarity |
|-----------|------------|--------|
| Surface Seam | 5-15 | Common |
| Hill Deposit | 20-40 | Uncommon |
| Underground Seam | 50-100 | Rare |

---

### Tier 3: Bog Iron
**Rarity:** Uncommon
**Hardness:** 4
**Tool Required:** Iron Pick (or patience with Basic)

**Description:** Iron oxide deposits found in swamps and bogs. The foundation of frontier metalworking — rusty, impure, but workable.

**Properties:**
| Property | Value |
|----------|-------|
| Stack Size | 30 |
| Weight | 1.5 kg |
| Base Value | 8 coins |
| Spawn Rate | Medium |
| Smelt Ratio | 3:1 (ore to ingot) |

**Extraction:**
- Found in shallow bog water (wade in)
- Dig from peat soil: 4-6 swings
- Must be dried before smelting

**Processing Pipeline:**
```
Bog Iron Ore → Dry (2 hours) → Dried Ore → Smelt (Coal x5) → Impure Iron → Refine → Iron Ingot
```

**Uses:**
| Recipe | Iron Ingots | Result |
|--------|-------------|--------|
| Iron Knife | 1 | Durable blade |
| Iron Axe Head | 2 | Chopping upgrade |
| Iron Pick Head | 3 | Mining upgrade |
| Nails (20) | 1 | Construction |
| Iron Pot | 2 | Cooking vessel |

**Biome Spawn:**
| Biome | Spawn Chance | Typical Deposit |
|-------|--------------|-----------------|
| Swamp | 15% per chunk | 3-8 ore |
| Bog | 25% per chunk | 5-12 ore |
| River Delta | 10% per chunk | 2-5 ore |

---
 
### Tier 4: Rock Salt
**Rarity:** Uncommon
**Hardness:** 2
**Tool Required:** Any Pick

**Description:** Crystalline sodium chloride — worth its weight in silver for food preservation. Found in caves and ancient seabeds.

**Properties:**
| Property | Value |
|----------|-------|
| Stack Size | 40 |
| Weight | 0.5 kg |
| Base Value | 12 coins |
| Spawn Rate | Medium-Low |

**Extraction:**
- Cave walls: 3-4 swings
- Salt flats: Surface collection
- Dissolves in water (protect from rain!)

**Uses:**
| Recipe | Salt Required | Result |
|--------|---------------|--------|
| Preserved Meat | 2 per lb | Stops spoilage |
| Cured Hide | 3 | Better leather |
| Salt Lick | 10 | Attract animals |
| Medicinal Salt | 1 | Wound cleaning |
| Trade Good | 5 | High value bundle |

**Special Mechanics:**
- Salt dissolves if inventory gets wet (swimming, rain)
- Store in waterproof container to protect
- Native factions value salt highly (+reputation when gifted)

**Deposit Types:**
| Type | Yield | Location |
|------|-------|----------|
| Cave Crystal | 5-10 | Cave walls |
| Salt Flat | 15-30 | Dry lake beds |
| Brine Pool | Unlimited* | Requires evaporation |

*Brine pools require evaporation process (craft salt pan)

---

### Tier 5: Copper
**Rarity:** Uncommon
**Hardness:** 3
**Tool Required:** Iron Pick

**Description:** Reddish-orange native metal, prized by natives for decoration and by colonists for tools harder than iron. Found in mountain streams and rock veins.

**Properties:**
| Property | Value |
|----------|-------|
| Stack Size | 25 |
| Weight | 2.0 kg |
| Base Value | 20 coins |
| Spawn Rate | Low-Medium |
| Smelt Ratio | 2:1 |

**Extraction:**
- Stream nuggets: Hand collection (rare)
- Rock veins: 6-8 swings with iron pick
- Native copper: Direct use, no smelting

**Processing:**
```
Copper Ore → Smelt (Coal x3) → Copper Ingot
Native Copper Nugget → Direct use (no processing)
```

**Uses:**
| Recipe | Copper | Result |
|--------|--------|--------|
| Copper Knife | 1 ingot | Sharp, decorative |
| Copper Pot | 2 ingots | Superior cooking |
| Copper Wire | 1 ingot | 10 wire lengths |
| Bronze Ingot | 1 copper + tin* | Stronger alloy |
| Wampum Beads | 1 nugget | Currency (native) |
| Decoration | varies | Faction reputation |

*Tin not native to region — must trade for it

**Faction Value:**
| Faction | Copper Value |
|---------|--------------|
| Native tribes | Very High (sacred) |
| Colonial | High (practical) |
| Spanish | Medium (have plenty) |

---

### Tier 6: Mica
**Rarity:** Rare
**Hardness:** 2.5
**Tool Required:** Any Pick (careful extraction)

**Description:** Glittering sheet silicate that flakes into thin, transparent layers. Valued for windows, decoration, and native ceremonial use.

**Properties:**
| Property | Value |
|----------|-------|
| Stack Size | 20 |
| Weight | 0.1 kg |
| Base Value | 35 coins |
| Spawn Rate | Low |

**Extraction:**
- Granite outcrops: 4-5 careful swings
- Quality degrades with rough handling
- Best extracted with specialized tools

**Quality Grades:**
| Grade | Transparency | Value Multiplier |
|-------|--------------|------------------|
| Shattered | Opaque | 0.25x |
| Flaked | Translucent | 0.5x |
| Sheet | Clear | 1.0x |
| Perfect Sheet | Crystal clear | 2.0x |

**Uses:**
| Recipe | Mica | Result |
|--------|------|--------|
| Window Pane | 4 sheets | Light without draft |
| Lantern Glass | 2 sheets | Safe lamp cover |
| Ceremonial Paint | 1 flaked | Glittering war paint |
| Decorative Inlay | varies | Furniture beauty |
| Trade Bundle | 10 sheets | High value goods |

**Extraction Skill Impact:**
| Mining Level | Perfect Sheet Chance |
|--------------|---------------------|
| Novice | 5% |
| Apprentice | 15% |
| Journeyman | 30% |
| Expert | 50% |
| Master | 75% |

---

### Tier 7: Lead (Galena)
**Rarity:** Rare
**Hardness:** 2.5
**Tool Required:** Iron Pick or better

**Description:** Dense, dark gray ore containing lead and traces of silver. Essential for ammunition in the age of muskets.

**Properties:**
| Property | Value |
|----------|-------|
| Stack Size | 20 |
| Weight | 3.0 kg |
| Base Value | 45 coins |
| Spawn Rate | Low |
| Smelt Ratio | 2:1 |

**Extraction:**
- Deep caves only
- Vein mining: 8-10 swings
- Toxic dust (hold breath or use cloth mask)

**Health Hazard:**
```
Lead Exposure: Mining without protection
- 0-5 ore: No effect
- 6-15 ore: Minor poisoning (-10% stamina)
- 16+ ore: Lead sickness (requires treatment)
```

**Processing:**
```
Galena Ore → Smelt (Coal x4) → Lead Ingot + Silver Trace (10% chance)
```

**Uses:**
| Recipe | Lead | Result |
|--------|------|--------|
| Musket Balls (20) | 1 ingot | Ammunition |
| Fishing Weights | 0.5 ingot | Better fishing |
| Window Caming | 1 ingot | Hold glass panes |
| Roof Flashing | 2 ingots | Waterproof roof |
| Shot Tower* | 50 ingots | Mass produce ammo |

*Requires advanced crafting station

**Military Value:**
- Colonial factions pay premium for lead
- Can be cast into ammunition at camp
- Strategic resource in conflicts

---

### Tier 8: Silver
**Rarity:** Very Rare
**Hardness:** 2.5
**Tool Required:** Steel Pick

**Description:** Precious white metal found in mountain veins, often alongside lead. Currency of empires, desire of conquistadors.

**Properties:**
| Property | Value |
|----------|-------|
| Stack Size | 10 |
| Weight | 2.5 kg |
| Base Value | 150 coins |
| Spawn Rate | Very Low |
| Smelt Ratio | 3:1 |

**Extraction:**
- Mountain veins: 10-12 swings
- Found alongside galena (lead)
- Requires steel tools
- Deep mining only

**Processing:**
```
Silver Ore → Smelt (Coal x6) → Raw Silver → Refine (Crucible) → Pure Silver
```

**Purity Grades:**
| Grade | Purity | Value |
|-------|--------|-------|
| Raw Silver | 60% | 100 coins |
| Refined Silver | 85% | 150 coins |
| Pure Silver | 99% | 200 coins |
| Sterling | 92.5% | 175 coins (best for crafting) |

**Uses:**
| Recipe | Silver | Result |
|--------|--------|--------|
| Silver Coins (10) | 1 pure | Currency |
| Silver Ring | 0.5 pure | Jewelry (+charm) |
| Silver Knife | 2 sterling | Decorative weapon |
| Silverware Set | 3 sterling | Luxury trade good |
| Silver Cross | 1 pure | Religious item |
| Faction Tribute | 5 pure | Major reputation boost |

**Faction Reactions:**
| Faction | Silver Response |
|---------|-----------------|
| Spanish | "Where did you find this?" (suspicious) |
| English | High interest, trade priority |
| French | Fair trade value |
| Native | Moderate interest (copper preferred) |

---

### Tier 9: Gold
**Rarity:** Extremely Rare
**Hardness:** 2.5
**Tool Required:** Steel Pick + Gold Pan (for placer)

**Description:** The metal that launched expeditions and doomed colonies. Found in mountain streams and deep quartz veins. Handle with discretion — gold attracts trouble.

**Properties:**
| Property | Value |
|----------|-------|
| Stack Size | 5 |
| Weight | 5.0 kg |
| Base Value | 500 coins |
| Spawn Rate | Extremely Low |
| Smelt Ratio | 4:1 |

**Extraction Methods:**

**Placer Mining (Streams):**
- Use gold pan in streams
- Mini-game: Swirl to separate gold from sediment
- Yields gold dust and occasional nuggets
- Skill affects success rate

| Skill Level | Dust/Hour | Nugget Chance |
|-------------|-----------|---------------|
| Novice | 0.1 oz | 1% |
| Apprentice | 0.3 oz | 3% |
| Journeyman | 0.5 oz | 5% |
| Expert | 0.8 oz | 8% |
| Master | 1.2 oz | 12% |

**Vein Mining (Mountains):**
- Quartz veins with gold traces
- 12-15 swings with steel pick
- Yields gold ore (must process)

**Processing:**
```
Gold Dust (10 oz) → Melt → Gold Nugget
Gold Nugget (5) → Smelt → Raw Gold Bar
Raw Gold Bar → Refine → Pure Gold Bar
```

**Uses:**
| Recipe | Gold | Result |
|--------|------|--------|
| Gold Coins (10) | 1 bar | Universal currency |
| Gold Ring | 0.25 bar | Jewelry (+major charm) |
| Gold Tooth | 0.1 bar | Permanent intimidation |
| Trade for Ship | 10 bars | Buy a vessel |
| Faction Bribe | 5 bars | Instant Allied status |

**Danger Mechanics:**
- Carrying gold increases bandit encounter rate
- Factions may demand "taxes" on gold
- Spanish particularly aggressive about gold sources
- Hide gold in stash, don't carry openly

---

### Tier 10: Sulfur
**Rarity:** Legendary
**Hardness:** 2
**Tool Required:** Steel Pick + Heat Protection

**Description:** Brimstone — the devil's mineral. Found only near volcanic vents and deep in cursed caves. Essential for gunpowder, dangerous to extract.

**Properties:**
| Property | Value |
|----------|-------|
| Stack Size | 15 |
| Weight | 0.8 kg |
| Base Value | 200 coins |
| Spawn Rate | Legendary (fixed spawns) |

**Extraction Hazards:**
```
Sulfur Mining Dangers:
- Toxic fumes: Damage over time without protection
- Heat damage: Near volcanic vents
- Explosive: Can ignite if struck with sparks
- Demonic reputation: Some NPCs fear carriers
```

**Required Equipment:**
| Protection | Effect |
|------------|--------|
| Cloth Mask | -50% fume damage |
| Alchemist's Mask | -90% fume damage |
| Wet Cloth | -30% heat damage |
| Fire-resistant Gloves | Prevent ignition |

**Processing:**
```
Raw Sulfur → Purify (Careful heating) → Pure Sulfur
Pure Sulfur + Charcoal + Saltite → Gunpowder
```

**Uses:**
| Recipe | Sulfur | Result |
|--------|--------|--------|
| Gunpowder (10 charges) | 1 pure + charcoal + saltpeter | Ammunition base |
| Fire Bomb | 2 pure | Incendiary weapon |
| Medicinal Sulfur | 0.5 pure | Skin treatments |
| Alchemical Base | 1 pure | Crafting component |
| Demon Ward | 3 pure | Superstitious protection |

**Fixed Spawn Locations:**
- The Devil's Vent (volcanic area)
- Brimstone Cavern (deep cave system)
- Hell's Gate (underground lake with fumes)

Each location has environmental puzzles and hazards.

---

## Skill Tree Structure

```
                              [EARTH SOVEREIGN]
                                      |
                 +--------------------+--------------------+
                 |                                         |
          [Deep Delver]                             [Master Refiner]
                 |                                         |
       +---------+---------+                     +---------+---------+
       |                   |                     |                   |
[Vein Finder]       [Stone Breaker]       [Pure Smelter]      [Efficient Processor]
       |                   |                     |                   |
       +---------+---------+                     +---------+---------+
                 |                                         |
          [Underground Expert]                      [Metallurgist]
                 |                                         |
                 +--------------------+--------------------+
                                      |
                              [Journeyman Miner]
                                      |
                 +--------------------+--------------------+
                 |                                         |
          [Ore Recognition]                         [Tool Mastery]
                 |                                         |
       +---------+---------+                     +---------+---------+
       |                   |                     |                   |
[Iron Seeker]       [Precious Eye]         [Pick Efficiency]   [Endurance]
       |                   |                     |                   |
       +---------+---------+                     +---------+---------+
                 |                                         |
          [Prospector]                              [Steady Miner]
                 |                                         |
                 +--------------------+--------------------+
                                      |
                               [SURFACE COLLECTOR]
                                (Starting Point)
```

---

## Tier 1: Foundation

### Surface Collector (Starting Skill)
**Unlock:** Collect your first flint
**Description:** Basic ability to recognize and gather surface minerals.

**Effects:**
- Can identify flint, coal, and exposed ores
- Hand collection of surface materials
- Basic understanding of mineral appearance
- Can craft crude stone tools

**Collection Mechanics:**
| Material | Method | Time |
|----------|--------|------|
| Flint | Hand pick | 2 seconds |
| Coal (exposed) | Hand gather | 3 seconds |
| Stream pebbles | Examine | 1 second |

---

## Tier 2: Basic Skills

### Prospector
**Unlock:** Find 5 different mineral types
**Prerequisite:** Surface Collector
**Description:** Understanding of where minerals form and how to find them.

**Effects:**
- Mineral deposits highlighted within 20m
- Can identify ore quality before mining
- Stream panning unlocked
- +15% chance to find bonus ore

**Prospecting Mechanics:**
- "Prospect" action on terrain
- Success reveals nearby deposits
- Higher skill = larger detection radius

---

### Steady Miner
**Unlock:** Mine 50 ore of any type
**Prerequisite:** Surface Collector
**Description:** Consistent, efficient extraction technique.

**Effects:**
- -20% stamina cost per swing
- Mining speed +15%
- Reduced tool wear (-10%)
- Steady rhythm bonus (combo swings)

**Combo System:**
| Consecutive Hits | Speed Bonus |
|------------------|-------------|
| 3 | +5% |
| 5 | +10% |
| 8 | +15% |
| 10+ | +20% |

---

## Tier 3: Ore Specialization

### Iron Seeker
**Unlock:** Smelt your first iron ingot
**Prerequisites:** Prospector
**Description:** Expertise in locating and extracting iron ores.

**Effects:**
- Bog iron deposits visible at 40m
- +25% iron ore yield
- Can identify iron quality (pure vs impure)
- Faster iron smelting

**Iron Grades:**
| Grade | Ingot Yield | Quality |
|-------|-------------|---------|
| Poor Bog Iron | 4:1 | Brittle |
| Standard Bog Iron | 3:1 | Normal |
| Rich Bog Iron | 2:1 | Strong |
| Hematite | 2:1 | Superior |

---

### Precious Eye
**Unlock:** Find your first copper, silver, or gold
**Prerequisites:** Prospector
**Description:** Trained eye for valuable ores that others miss.

**Effects:**
- Precious metal deposits visible at 30m
- +20% precious ore yield
- Can spot gold flakes in streams
- Increased chance for silver in galena

**Precious Metal Detection:**
| Metal | Base Find Chance | With Skill |
|-------|------------------|------------|
| Copper | 5% per chunk | 12% |
| Silver | 1% per chunk | 4% |
| Gold | 0.2% per chunk | 1% |

---

### Pick Efficiency
**Unlock:** Break 100 ore nodes
**Prerequisites:** Steady Miner
**Description:** Maximum force with minimum effort.

**Effects:**
- -30% swings required per node
- Critical hit chance (instant break) +10%
- Tools last 25% longer
- Can mine one tier above tool rating

**Tool Override:**
| Normally Required | Can Use Instead |
|-------------------|-----------------|
| Iron Pick | Stone Pick |
| Steel Pick | Iron Pick |
| (With -25% efficiency penalty) |

---

### Endurance
**Unlock:** Mine for 30 minutes continuous
**Prerequisites:** Steady Miner
**Description:** The stamina of a career miner.

**Effects:**
- +50% stamina pool while mining
- Stamina regeneration +30% underground
- Can mine while encumbered (slowly)
- Resistance to cave-in stun

---

## Tier 4: Intermediate Skills

### Ore Recognition
**Unlock:** Max Iron Seeker AND Precious Eye
**Description:** Instant identification of all ore types and quality.

**Effects:**
- All ore types visible at 50m
- Quality assessment before first swing
- Vein size estimation
- Hidden deposits revealed

**Vein Analysis:**
```
[Looking at Iron Deposit]
Type: Bog Iron
Quality: Rich
Estimated Yield: 8-12 ore
Depth: Surface
Recommendation: Extract immediately
```

---

### Tool Mastery
**Unlock:** Max Pick Efficiency AND Endurance
**Description:** Complete control of mining implements.

**Effects:**
- Can use any pick at any tier (with penalties)
- Tool durability +50%
- Swing speed +25%
- Precision strikes (choose exact hit point)

**Precision Mining:**
- Target specific parts of ore vein
- Avoid hitting waste rock
- Extract without damaging adjacent veins

---

## Tier 5: Core Competency

### Journeyman Miner
**Unlock:** Complete both Ore Recognition AND Tool Mastery
**Description:** A true professional miner — respected in any camp.

**Effects:**
- All previous bonuses stack
- Can teach mining to NPCs
- Mining reputation unlocked
- Access to Advanced branches

**Title Earned:** "Journeyman Miner"

**Passive Bonuses:**
| Stat | Bonus |
|------|-------|
| Ore Detection Range | 50m |
| Yield Bonus | +30% |
| Stamina Efficiency | +40% |
| Tool Durability | +50% |
| Mining Speed | +35% |

---

## Tier 6: Deep Mining Branch

### Underground Expert
**Unlock:** Mine 100 ore from caves
**Prerequisite:** Journeyman Miner
**Description:** Comfortable in the depths where surface rules don't apply.

**Effects:**
- Night vision in caves (dim light sufficient)
- Cave navigation intuition
- Detect unstable ceilings (cave-in warning)
- +30% ore yield underground

**Underground Mechanics:**
| Hazard | Detection Chance |
|--------|------------------|
| Cave-in | 80% |
| Gas pocket | 70% |
| Flooded tunnel | 90% |
| Creature lair | 60% |

---

### Metallurgist
**Unlock:** Smelt 50 ingots of any type
**Prerequisite:** Journeyman Miner
**Description:** Understanding of metal properties and processing.

**Effects:**
- Smelting efficiency +40%
- Can assess metal purity
- Alloy crafting unlocked
- Reduced fuel consumption

**Alloy Recipes:**
| Alloy | Components | Properties |
|-------|------------|------------|
| Bronze | Copper + Tin | Harder than copper |
| Steel | Iron + Carbon (coal) | Superior strength |
| Electrum | Gold + Silver | Beautiful, valuable |

---

## Tier 7: Specialization

### Vein Finder
**Unlock:** Discover 10 hidden ore veins
**Prerequisite:** Underground Expert
**Description:** Supernatural sense for finding rich deposits.

**Effects:**
- Ore detection range: 100m
- Can sense ore type before seeing it
- Hidden veins revealed automatically
- +50% chance for rare ore in any vein

**Vein Sensing:**
- Different ore "signatures"
- Can track veins through rock
- Predict vein continuation direction

---

### Stone Breaker
**Unlock:** Mine 500 ore total
**Prerequisite:** Underground Expert
**Description:** Raw power through stone and ore.

**Effects:**
- -50% swings required
- Can break nodes other tools can't
- Immune to cave-in damage
- AOE mining (hit multiple nodes)

**AOE Mining:**
- Powerful overhead swing
- Hits 3 adjacent nodes
- Higher stamina cost
- Risk of quality damage

---

### Pure Smelter
**Unlock:** Achieve 95%+ purity on 20 refines
**Prerequisite:** Metallurgist
**Description:** The art of extracting perfect metal.

**Effects:**
- Smelting always produces top quality
- Rare material extraction from byproducts
- Silver from galena: 25% (up from 10%)
- Gold traces visible in quartz

**Byproduct Extraction:**
| Primary Ore | Byproduct | Chance |
|-------------|-----------|--------|
| Galena | Silver | 25% |
| Quartz | Gold dust | 5% |
| Copper | Native silver | 10% |
| Iron | Chromium | 3% |

---

### Efficient Processor
**Unlock:** Reduce processing waste by 50%
**Prerequisite:** Metallurgist
**Description:** Maximizing yield from every piece of ore.

**Effects:**
- Processing ratio improved by 1 tier
- No ore wasted (minimum 1 ingot per ore)
- Slag can be reprocessed
- Fuel efficiency doubled

**Improved Ratios:**
| Ore | Normal Ratio | With Skill |
|-----|--------------|------------|
| Bog Iron | 3:1 | 2:1 |
| Copper | 2:1 | 1.5:1 |
| Silver | 3:1 | 2:1 |
| Gold | 4:1 | 3:1 |

---

## Tier 8: Mastery

### Deep Delver
**Unlock:** Max Vein Finder AND Stone Breaker
**Description:** Master of the underground realm.

**Effects:**
- Can mine Legendary deposits
- Immune to all underground hazards
- Detect sulfur deposits
- Underground movement speed +50%

**Deep Delver Abilities:**
| Ability | Effect | Cooldown |
|---------|--------|----------|
| Tremor Sense | Reveal all ore in 200m | 10 min |
| Stone Communion | Pass through unstable areas | 5 min |
| Deep Breath | Ignore toxic fumes for 60s | 3 min |

**Title Earned:** "Deep Delver"

---

### Master Refiner
**Unlock:** Max Pure Smelter AND Efficient Processor
**Description:** Alchemist of metals — nothing escapes your crucible.

**Effects:**
- Can purify any ore to 100%
- Create masterwork ingots (+50% item quality)
- Extract gold from any ore (tiny amounts)
- Legendary crafting materials unlocked

**Masterwork Materials:**
| Material | Source | Bonus to Items |
|----------|--------|----------------|
| Masterwork Iron | Perfect smelting | +30% durability |
| Masterwork Copper | Perfect smelting | +20% effectiveness |
| Masterwork Silver | Perfect smelting | +50% value |
| Masterwork Gold | Perfect smelting | +100% value |

**Title Earned:** "Master Refiner"

---

## Tier 9: Legendary

### Earth Sovereign
**Unlock:** Complete BOTH Deep Delver AND Master Refiner
**Description:** The earth yields its secrets to you alone. Legends speak of miners who could smell gold through mountains.

**Effects:**
- All mining skills at maximum
- Can find any ore anywhere (even "impossible" locations)
- Ore respawns faster in your presence
- Command over underground creatures

**Legendary Abilities:**
| Ability | Effect | Cooldown |
|---------|--------|----------|
| Earth's Bounty | Double all yields for 10 min | 1 hour |
| Gold Nose | Track nearest gold deposit | 15 min |
| Stone Whisper | Ore veins extend toward you | 30 min |
| Mole's Path | Phase through solid rock (short distance) | 20 min |

**Title Earned:** "Earth Sovereign"

**Companion Unlock: Mine Canary**
- Loyal bird that detects dangers
- Warns of gas, cave-ins, creatures
- Can scout ahead in tunnels
- Passive: +5% rare ore chance

---

## Inventory Integration

### Ore Item Structure

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OreItem {
    pub ore_type: OreType,
    pub quantity: u32,
    pub quality: OreQuality,
    pub weight: f32,
    pub provenance: OreProvenance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OreType {
    Flint,
    Coal,
    BogIron,
    RockSalt,
    Copper,
    Mica,
    Lead,
    Silver,
    Gold,
    Sulfur,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OreQuality {
    Poor,      // 0.5x value
    Standard,  // 1.0x value
    Rich,      // 1.5x value
    Pure,      // 2.0x value
    Legendary, // 3.0x value
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OreProvenance {
    pub source_location: Vec3,
    pub extraction_time: f64,
    pub extractor_skill: u8,
    pub processing_history: Vec<ProcessingStep>,
}
```

### Inventory Slot Types

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InventorySlotType {
    General,        // Any item
    Ore,            // Raw ores only
    Ingot,          // Processed metals
    Fuel,           // Coal, wood
    Tool,           // Picks, pans
    Valuable,       // Gold, silver, jewelry
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MiningInventory {
    pub ore_pouch: Vec<OreItem>,          // 20 slots, ore only
    pub ingot_case: Vec<IngotItem>,        // 10 slots, ingots only
    pub valuables_lockbox: Vec<ValuableItem>, // 5 slots, precious only
    pub tool_belt: Vec<MiningTool>,        // 3 slots
    pub general_capacity: u32,             // Weight limit
}
```

### Ore Extraction to Inventory Pipeline

```rust
pub struct MiningEvent {
    pub ore_node: OreNodeId,
    pub player: PlayerId,
    pub tool_used: ToolType,
    pub swings_taken: u32,
    pub skill_level: u8,
}

pub fn extract_ore(event: MiningEvent) -> MiningResult {
    // 1. Calculate base yield
    let base_yield = calculate_base_yield(&event.ore_node);

    // 2. Apply skill modifiers
    let skill_modifier = get_skill_modifier(event.skill_level);
    let modified_yield = base_yield * skill_modifier;

    // 3. Determine quality
    let quality = roll_quality(event.skill_level, event.ore_node.richness);

    // 4. Create ore items
    let ore_items: Vec<OreItem> = (0..modified_yield)
        .map(|_| OreItem {
            ore_type: event.ore_node.ore_type,
            quantity: 1,
            quality,
            weight: get_ore_weight(event.ore_node.ore_type),
            provenance: OreProvenance {
                source_location: event.ore_node.position,
                extraction_time: current_time(),
                extractor_skill: event.skill_level,
                processing_history: vec![],
            },
        })
        .collect();

    // 5. Stack identical ores
    let stacked_items = stack_ores(ore_items);

    // 6. Add to inventory
    for item in stacked_items {
        if !add_to_ore_pouch(&item) {
            if !add_to_general_inventory(&item) {
                drop_on_ground(&item);
            }
        }
    }

    // 7. Award skill points
    award_mining_xp(event.player, modified_yield, quality);

    MiningResult::Success(modified_yield)
}
```

### Inventory Weight System

```rust
pub struct InventoryWeight {
    pub current_weight: f32,
    pub max_weight: f32,
    pub ore_weight: f32,      // Subtotal
    pub ingot_weight: f32,    // Subtotal
    pub tool_weight: f32,     // Subtotal
}

impl InventoryWeight {
    pub fn encumbrance_level(&self) -> EncumbranceLevel {
        let ratio = self.current_weight / self.max_weight;
        match ratio {
            r if r < 0.5 => EncumbranceLevel::Light,
            r if r < 0.75 => EncumbranceLevel::Medium,
            r if r < 1.0 => EncumbranceLevel::Heavy,
            _ => EncumbranceLevel::Overloaded,
        }
    }

    pub fn movement_modifier(&self) -> f32 {
        match self.encumbrance_level() {
            EncumbranceLevel::Light => 1.0,
            EncumbranceLevel::Medium => 0.85,
            EncumbranceLevel::Heavy => 0.6,
            EncumbranceLevel::Overloaded => 0.3,
        }
    }
}
```

### Ore Display in Inventory UI

```rust
pub struct OreInventoryDisplay {
    pub icon: TextureId,           // Ore type icon
    pub quantity_text: String,     // "x15"
    pub quality_border: Color,     // Gold for pure, gray for poor
    pub weight_indicator: String,  // "4.5 kg"
    pub tooltip: OreTooltip,
}

pub struct OreTooltip {
    pub name: String,              // "Rich Copper Ore"
    pub description: String,       // "High quality copper..."
    pub stats: Vec<(String, String)>, // ("Smelt Ratio", "2:1")
    pub uses: Vec<String>,         // ["Copper Ingot", "Wampum"]
    pub value: String,             // "20 coins"
    pub weight: String,            // "2.0 kg each"
}
```

---

## Mining Node System

### Node Spawning

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OreNode {
    pub id: OreNodeId,
    pub position: Vec3,
    pub ore_type: OreType,
    pub richness: Richness,
    pub remaining_ore: u32,
    pub max_ore: u32,
    pub respawn_time: f64,
    pub required_tool: ToolTier,
    pub discovery_state: DiscoveryState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DiscoveryState {
    Hidden,              // Not visible until prospected
    Revealed,            // Visible on map
    PartiallyMined,      // Being extracted
    Depleted,            // Waiting for respawn
}

pub fn spawn_ore_nodes(chunk: &Chunk, biome: &Biome) -> Vec<OreNode> {
    let mut nodes = vec![];

    for ore_type in OreType::all() {
        let spawn_chance = get_spawn_chance(ore_type, biome);

        if random() < spawn_chance {
            let count = get_node_count(ore_type, biome);

            for _ in 0..count {
                let position = find_valid_position(chunk, ore_type);
                let richness = roll_richness(ore_type);

                nodes.push(OreNode {
                    id: generate_id(),
                    position,
                    ore_type,
                    richness,
                    remaining_ore: calculate_ore_amount(ore_type, richness),
                    max_ore: calculate_ore_amount(ore_type, richness),
                    respawn_time: 0.0, // Active
                    required_tool: get_required_tool(ore_type),
                    discovery_state: initial_discovery_state(ore_type),
                });
            }
        }
    }

    nodes
}
```

### Node Respawn

```rust
pub const RESPAWN_TIMES: [(OreType, f64); 10] = [
    (OreType::Flint, 1.0),       // 1 game day
    (OreType::Coal, 3.0),        // 3 game days
    (OreType::BogIron, 7.0),     // 1 week
    (OreType::RockSalt, 14.0),   // 2 weeks
    (OreType::Copper, 14.0),     // 2 weeks
    (OreType::Mica, 21.0),       // 3 weeks
    (OreType::Lead, 30.0),       // 1 month
    (OreType::Silver, 60.0),     // 2 months
    (OreType::Gold, 90.0),       // 3 months
    (OreType::Sulfur, 180.0),    // 6 months (legendary)
];
```

---

## Processing Stations

### Smelting Furnace

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmeltingFurnace {
    pub position: Vec3,
    pub fuel_remaining: f32,     // In burn-minutes
    pub temperature: f32,        // In degrees
    pub ore_input: Option<OreItem>,
    pub ingot_output: Option<IngotItem>,
    pub smelting_progress: f32,  // 0.0 - 1.0
    pub efficiency: f32,         // Based on builder skill
}

impl SmeltingFurnace {
    pub fn can_smelt(&self, ore: &OreItem) -> bool {
        let required_temp = get_smelt_temperature(ore.ore_type);
        self.temperature >= required_temp && self.fuel_remaining > 0.0
    }

    pub fn smelt(&mut self, ore: OreItem, player_skill: u8) -> Option<IngotItem> {
        let base_time = get_smelt_time(ore.ore_type);
        let skill_modifier = 1.0 - (player_skill as f32 * 0.02); // -2% per skill level
        let actual_time = base_time * skill_modifier;

        // Consume fuel
        self.fuel_remaining -= get_fuel_cost(ore.ore_type);

        // Calculate quality
        let quality = calculate_ingot_quality(ore.quality, player_skill);

        Some(IngotItem {
            metal_type: ore.ore_type.to_metal(),
            quality,
            weight: get_ingot_weight(ore.ore_type),
            provenance: IngotProvenance {
                source_ore: ore.provenance,
                smelted_time: current_time(),
                smelter_skill: player_skill,
            },
        })
    }
}
```

### Gold Panning Station

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoldPan {
    pub durability: f32,
    pub efficiency: f32, // Based on skill
}

pub fn pan_for_gold(
    player: &Player,
    pan: &GoldPan,
    stream: &WaterSource,
) -> PanningResult {
    // Mini-game: Swirl to separate
    let skill = player.mining_skills.prospector_level;
    let base_success = 0.05 + (skill as f32 * 0.02);

    // Stream richness affects chance
    let stream_modifier = stream.gold_richness;
    let final_chance = base_success * stream_modifier * pan.efficiency;

    if random() < final_chance {
        let amount = roll_gold_amount(skill);

        PanningResult::Success(GoldDust {
            ounces: amount,
            purity: roll_purity(skill),
        })
    } else {
        PanningResult::Nothing
    }
}
```

---

## Point Acquisition

| Action | Points |
|--------|--------|
| First ore collected | 10 |
| Mine any ore | 2 |
| Mine rare ore (silver+) | 15 |
| Complete ore vein | 25 |
| Smelt ingot | 5 |
| Achieve high purity | 10 |
| Discover hidden vein | 30 |
| Find legendary deposit | 100 |
| Survive mining hazard | 20 |
| Mine during storm | 10 |
| Pan gold successfully | 8 |
| Craft masterwork item | 50 |

**Points Required Per Tier:**
- Tier 1 → Tier 2: 50 points
- Tier 2 → Tier 3: 125 points
- Tier 3 → Tier 4: 250 points
- Tier 4 → Tier 5: 450 points
- Tier 5 → Tier 6: 700 points
- Tier 6 → Tier 7: 1000 points
- Tier 7 → Tier 8: 1400 points
- Tier 8 → Tier 9: 2000 points

---

## Cross-System Integration

### Hunting Synergies
| Hunting Skill | Mining Bonus |
|---------------|--------------|
| Wilderness Scout | Find cave entrances easier |
| Serpent Eye | Detect dangerous cave creatures |
| Trap Setter | Set alarms at mine entrances |

### Archaeology Synergies
| Archaeology Skill | Mining Bonus |
|-------------------|--------------|
| Dig Site knowledge | Faster ore extraction |
| Fossil recognition | Identify valuable rock types |
| Careful excavation | Higher quality ore extraction |

### Faction Integration
| Faction | Mining Relationship |
|---------|---------------------|
| Spanish | Very interested in gold/silver locations |
| English | Want iron and lead for military |
| Native | Trade copper, value salt |
| French | Fair traders for all metals |

### Economy Integration
- Ore and ingots have market values
- Processing adds value (ore < ingot < item)
- Rare metals drive faction quests
- Gold causes faction aggression

---

## Mining Tools

### Tool Tiers

| Tool | Material | Mineable Ores | Durability | Speed |
|------|----------|---------------|------------|-------|
| Hands | — | Flint only | — | Very Slow |
| Stone Pick | Flint + Wood | Flint, Coal | 50 uses | Slow |
| Copper Pick | Copper + Wood | + Bog Iron, Salt | 100 uses | Medium |
| Iron Pick | Iron + Wood | + Copper, Mica, Lead | 200 uses | Fast |
| Steel Pick | Steel + Wood | All ores | 400 uses | Very Fast |
| Master's Pick | Masterwork Steel | All + Legendary | 1000 uses | Maximum |

### Specialized Tools

| Tool | Use | Bonus |
|------|-----|-------|
| Gold Pan | Stream panning | Required for placer gold |
| Crucible | Refining metals | Higher purity output |
| Bellows | Furnace | Higher temperatures |
| Ore Cart | Transport | 5x carry capacity |
| Miner's Lamp | Cave lighting | See in dark, detect gas |
| Dowsing Rod | Prospecting | +20% ore detection |

---

## Hazards and Safety

### Cave Hazards

| Hazard | Effect | Prevention |
|--------|--------|------------|
| Cave-in | Damage, trapped | Timber supports |
| Gas Pocket | Poison damage | Canary, ventilation |
| Flooding | Drowning | Check for water sounds |
| Creatures | Combat | Clear before mining |
| Unstable ore | Explosion (sulfur) | Careful extraction |

### Environmental Effects

| Condition | Mining Effect |
|-----------|---------------|
| Rain | Flooded caves, slippery |
| Cold | Slower, frostbite risk |
| Heat | Faster stamina drain |
| Night | Reduced visibility |
| Storm | Dangerous cave conditions |

---

## Implementation Priority

### Phase 1 (Core)
- [ ] 10 ore types with basic properties
- [ ] Surface collection for flint/coal
- [ ] Basic pick tool and mining action
- [ ] Ore nodes spawn in appropriate biomes
- [ ] Simple inventory integration

### Phase 2 (Processing)
- [ ] Smelting furnace station
- [ ] Ore → Ingot conversion
- [ ] Fuel consumption system
- [ ] Quality grades for ore/ingots

### Phase 3 (Progression)
- [ ] Tier 1-5 skills
- [ ] Prospecting mechanic
- [ ] Tool tier requirements
- [ ] Skill point tracking

### Phase 4 (Advanced)
- [ ] Tier 6-8 skills
- [ ] Cave hazard system
- [ ] Gold panning mini-game
- [ ] Rare ore spawning

### Phase 5 (Mastery)
- [ ] Tier 9 legendary skills
- [ ] Sulfur and dangerous extraction
- [ ] Mine Canary companion
- [ ] Masterwork crafting
- [ ] Full economy integration

---

## Historical Notes

Mining in colonial America was primitive but essential:

- **Bog iron** was the primary iron source — literally pulled from swamps
- **Salt** was often more valuable than gold for food preservation
- **Copper** was sacred to many native tribes, traded across vast distances
- **Gold fever** drove the Spanish but largely eluded them in the Carolinas
- **Lead** became crucial for ammunition as firearms spread
- **Sulfur** (for gunpowder) was imported until local sources were found

This skill tree reflects the gradual mastery of mineral extraction on the frontier — from desperate survival prospecting to systematic mining operations that would eventually fuel a nation's industry.
