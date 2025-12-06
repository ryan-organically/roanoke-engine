# Hunting Skill Tree Specification

## Overview

The Hunting Skill Tree develops the player's ability to track, stalk, and harvest wildlife in the Roanoke wilderness. Beginning with basic prey identification, players progress through specialized hunting techniques, eventually mastering the pursuit of the most dangerous predators. Skills integrate with the Animal System's 10 species, providing counter-strategies for each behavior type.

## Design Philosophy

Hunting in colonial-era Roanoke was a matter of survival. Native hunters developed intimate knowledge of animal behavior passed down through generations. European colonists brought their own hunting traditions. This skill tree reflects both approaches—primal instinct and learned technique.

---

## Skill Tree Structure

```
                              [LEGENDARY HUNTER]
                                      |
                 +--------------------+--------------------+
                 |                                         |
          [Apex Predator]                           [Master Trapper]
                 |                                         |
       +---------+---------+                     +---------+---------+
       |                   |                     |                   |
[Beast Slayer]      [Shadow Hunter]       [Snare Master]      [Lure Crafter]
       |                   |                     |                   |
       +---------+---------+                     +---------+---------+
                 |                                         |
          [Big Game Hunter]                         [Trap Setter]
                 |                                         |
                 +--------------------+--------------------+
                                      |
                              [Wilderness Scout]
                                      |
                 +--------------------+--------------------+
                 |                                         |
          [Predator Sense]                          [Prey Instinct]
                 |                                         |
       +---------+---------+                     +---------+---------+
       |                   |                     |                   |
[Wolf Tracker]      [Serpent Eye]         [Boar Hunter]      [Deer Stalker]
       |                   |                     |                   |
       +--------------------+--------------------+--------------------+
                                      |
                              [BASIC TRACKER]
                               (Starting Point)
```

---

## Tier 1: Foundation

### Basic Tracker (Starting Skill)
**Unlock:** Automatic at game start
**Description:** Fundamental awareness of animal presence in the wild.

**Effects:**
- Animals within 30m show directional indicator on screen edge
- Can identify animal tracks on ground (visual effect)
- Crouching reduces detection range by 25%
- Basic skinning ability (50% loot yield)

**Tracks Visible:**
| Animal | Track Description |
|--------|-------------------|
| Black Bear | Large paw prints, 5 toes with claw marks |
| Wild Boar | Cloven hoof prints, deep in mud |
| Gray Wolf | Dog-like prints in groups |
| Deer | Delicate cloven prints |

---

## Tier 2: Prey Specialization

### Boar Hunter
**Unlock:** Kill your first Wild Boar
**Prerequisite:** Basic Tracker
**Description:** Knowledge of the aggressive wild boar's behavior and weaknesses.

**Effects:**
- Wild Boar tracks glow and show direction of travel
- +25% damage against Wild Boar
- Boar charge attack telegraph visible (ground indicator)
- Unlocks: Boar bait crafting

**Counter-Tactics:**
- Boars charge in straight lines—sidestep and strike
- Boars are less aggressive when not in groups
- Fire scares boars (torch equipped = reduced aggression)

**Loot Bonus:**
| Item | Base Drop | With Skill |
|------|-----------|------------|
| Boar Hide | 60% | 85% |
| Boar Tusk | 25% | 50% |
| Boar Meat | 2-3 | 3-5 |
| Boar Fat | 40% | 65% |

---

### Deer Stalker
**Unlock:** Successfully sneak within 10m of a deer without alerting it
**Prerequisite:** Basic Tracker
**Description:** The patience and stealth needed to hunt wary prey.

**Effects:**
- Deer and similar prey have reduced detection range (-30%)
- Movement while crouched is 20% quieter
- Can identify deer bedding areas (rest spots)
- Unlocks: Deer call (attracts deer to location)

**Prey Animals Affected:**
- Deer (when implemented)
- Rabbits (when implemented)
- Turkey (when implemented)

**Stealth Mechanics:**
- Wind direction indicator appears
- Downwind approach gives stealth bonus
- Sudden movements break stealth

---

## Tier 3: Predator Awareness

### Wolf Tracker
**Unlock:** Survive an encounter with a wolf pack (3+ wolves)
**Prerequisite:** Basic Tracker
**Description:** Understanding of pack hunter tactics and how to counter them.

**Effects:**
- Wolf pack members shown on mini-map when one is spotted
- Pack alpha highlighted with distinct marker
- Warning when being flanked by pack members
- +20% damage against wolves

**Pack Counter-Tactics:**
- Killing the alpha reduces pack morale by 50%
- Isolated wolves are cautious—pick them off
- Fire and loud noises scatter packs temporarily
- Back against wall prevents flanking

**Wolf Behavior Insights:**
| Behavior State | Player Warning |
|----------------|----------------|
| Stalking | "You're being watched..." |
| Circling | Flanking arrows on HUD |
| Pack Howl | "Wolves are calling reinforcements!" |
| Low Morale | "The pack is losing confidence" |

---

### Serpent Eye
**Unlock:** Spot a snake before it strikes you (awareness check)
**Prerequisite:** Basic Tracker
**Description:** Heightened awareness of camouflaged reptilian threats.

**Effects:**
- Snakes highlighted with subtle outline when within 15m
- Rattlesnake rattle audio range +50%
- Cottonmouth/Copperhead give off faint heat shimmer
- +30% damage against snakes
- Reduced poison duration from snake bites (-25%)

**Snake Species:**
| Species | Danger | Detection Bonus |
|---------|--------|-----------------|
| Timber Rattlesnake | 5 | Audio warning enhanced |
| Copperhead | 3 | Ground scan highlights |
| Cottonmouth | 4 | Water edge scan |

**Avoidance Tips:**
- Watch where you step in tall grass
- Snakes sun on rocks during morning
- Cottonmouths guard water sources
- Heavy boots reduce bite damage

---

## Tier 4: Intermediate Skills

### Predator Sense
**Unlock:** Kill 3 different predator species
**Prerequisites:** Wolf Tracker AND Serpent Eye
**Description:** Instinctive awareness of when you're being hunted.

**Effects:**
- Screen edge darkens when a predator is stalking you
- Stalking predators (cougars) periodically pinged on mini-map
- Ambush predators (alligators, snakes) revealed when you look at them
- "Sixth sense" audio cue when about to be attacked from behind

**Predator Detection:**
| Predator Type | Detection Method |
|---------------|------------------|
| Stalker (Cougar, Bobcat) | Periodic location ping |
| Ambush (Alligator, Snakes) | Highlight on direct look |
| Pack Hunter (Wolves) | Full pack revealed |
| Territorial (Bear) | Territory boundary shown |

---

### Prey Instinct
**Unlock:** Kill 10 prey animals (boar, deer, etc.)
**Prerequisites:** Boar Hunter AND Deer Stalker
**Description:** Deep understanding of prey animal patterns and behavior.

**Effects:**
- Prey animals show patrol paths as faint trails
- Feeding and drinking times known (UI indicator)
- Wounded prey leaves blood trail (easier tracking)
- +50% skinning yield from prey animals

**Prey Behavior Knowledge:**
- Dawn/dusk: Peak activity
- Midday: Resting in shade
- Night: Bedded down, easier to approach
- Storm: Sheltering, predictable locations

---

## Tier 5: Advanced Hunting

### Wilderness Scout
**Unlock:** Discover 5 animal dens/nesting sites
**Prerequisites:** Predator Sense AND Prey Instinct
**Description:** Expert knowledge of animal territories and home ranges.

**Effects:**
- Animal dens/nests visible on map when nearby
- Can identify high-traffic animal paths
- Territory boundaries of all animals visible
- Spawn areas for each species known

**Territory System:**
| Animal | Territory Size | Den Type |
|--------|----------------|----------|
| Black Bear | 100m radius | Cave, hollow log |
| Cougar | 150m radius | Rocky outcrop, cave |
| Wolf Pack | 200m radius | Pack den, open area |
| Alligator | 50m radius | Water's edge, bank |
| Boar | 75m radius | Thicket, mud wallow |

**Scout Actions:**
- Mark den on map (persists)
- Estimate animal population in area
- Predict respawn timing
- Identify safe travel routes

---

## Tier 6: Specialization Branches

### Big Game Hunter
**Unlock:** Kill a Black Bear or Alligator solo
**Prerequisite:** Wilderness Scout
**Description:** Expertise in hunting the largest and most dangerous game.

**Effects:**
- +35% damage against bears, alligators, cougars
- Critical hit chance +15% on large animals
- Large animal attack patterns fully telegraphed
- Can craft large animal lures

**Big Game Tactics:**
| Animal | Weakness | Optimal Strategy |
|--------|----------|------------------|
| Black Bear | Fire, spears | Keep distance, use reach weapons |
| Alligator | Land speed, cold | Lure to land, attack from sides |
| Cougar | Noise, groups | Never turn your back, face it |

**Trophy Hunting:**
- Perfect kills (no damage taken) yield trophy items
- Trophies can be mounted at camp
- Trophy collection unlocks cosmetics

---

### Trap Setter
**Unlock:** Successfully trap 5 animals
**Prerequisite:** Wilderness Scout
**Description:** The art of capturing prey without direct confrontation.

**Effects:**
- Trap crafting recipes unlocked
- Traps deal +50% damage
- Trapped animals cannot flee
- Can set traps while crouched without alerting nearby animals

**Trap Types:**
| Trap | Materials | Effective Against | Damage |
|------|-----------|-------------------|--------|
| Snare | Rope, Stake | Small prey | Capture |
| Deadfall | Logs, Rope, Bait | Medium prey | 50 |
| Pit Trap | Digging + Stakes | Large animals | 75 |
| Jaw Trap | Iron, Spring | Predators | 40 + Bleed |
| Net Trap | Rope, Frame | Birds, small prey | Capture |

**Trap Placement:**
- Place on animal paths for best results
- Bait increases trigger chance
- Weather affects trap effectiveness
- Must check traps regularly (animals can escape)

---

## Tier 7: Master Branches

### Beast Slayer
**Unlock:** Kill one of each predator species
**Prerequisite:** Big Game Hunter
**Description:** Feared hunter of the most dangerous beasts.

**Effects:**
- +50% damage against all predators
- Predators have 10% chance to flee on sight
- Killing blow triggers "Intimidation" (nearby animals flee)
- Unlocks legendary weapon crafting

**Predator Kill Count Required:**
- [ ] Black Bear
- [ ] Eastern Cougar
- [ ] Gray Wolf (pack alpha counts)
- [ ] Timber Rattlesnake
- [ ] American Alligator
- [ ] Cottonmouth
- [ ] Copperhead
- [ ] Red Wolf (pack alpha counts)
- [ ] Bobcat
- [ ] Wild Boar

**Beast Slayer Crafting:**
| Recipe | Materials | Effect |
|--------|-----------|--------|
| Predator's Cloak | 3 Predator Pelts + Sinew | -50% predator aggro range |
| Fang Necklace | 5 Different Fangs | +25% damage vs predators |
| Apex Hunter's Bow | Bear Sinew + Hardwood + Cougar Gut | Best ranged weapon |

---

### Shadow Hunter
**Unlock:** Kill 5 animals without being detected
**Prerequisite:** Big Game Hunter
**Description:** Master of stealth hunting—the unseen death.

**Effects:**
- Movement completely silent while crouched
- Can move at 75% speed while crouched (normally 50%)
- Stealth kills deal 3x damage
- Animals don't alert others when killed silently

**Stealth Mechanics:**
- Approach from downwind
- Move during ambient noise (wind, rain)
- Avoid line of sight
- Use cover and shadows

**Shadow Kill Conditions:**
- Animal unaware of player
- Attack from behind or side
- Single killing blow
- No other animals witness

---

### Snare Master
**Unlock:** Capture 20 animals in traps
**Prerequisite:** Trap Setter
**Description:** Legendary trapper whose snares never fail.

**Effects:**
- Traps have 100% trigger rate when animal walks over
- Trapped animals cannot break free
- Can craft "humane traps" (capture alive for trade)
- Trap range extended (larger trigger area)

**Advanced Traps:**
| Trap | Materials | Special Effect |
|------|-----------|----------------|
| Bear Trap | Iron, Chain | Immobilize large predators |
| Venom Snare | Trap + Snake Venom | Poison on trigger |
| Alarm Trap | Bells, Wire | Alerts player to animal presence |
| Combo Trap | Any trap + Net | Capture + Damage |

**Trap Mastery Bonuses:**
- Traps last twice as long before degrading
- Can pick up and reuse triggered traps
- Trapped animal loot +25%

---

### Lure Crafter
**Unlock:** Craft 10 different bait types
**Prerequisite:** Trap Setter
**Description:** Expert in attracting specific animals to your location.

**Effects:**
- All lure/bait effectiveness +100%
- Can craft species-specific lures
- Lures work at double range
- Unlocks "Call" abilities for each animal type

**Lure Recipes:**
| Lure | Materials | Attracts |
|------|-----------|----------|
| Meat Scraps | Raw Meat | General predators |
| Blood Bait | Blood + Meat | Wolves, Cougars |
| Fish Bait | Fish + Entrails | Alligators, Bears |
| Musk Lure | Glands + Fat | Boars, Deer |
| Rodent Lure | Small Bones + Seeds | Snakes, Bobcats |
| Honey Bait | Honey + Berries | Bears specifically |

**Animal Calls:**
| Call | Attracts | Risk |
|------|----------|------|
| Deer Bleat | Deer | May attract predators |
| Boar Grunt | Boars | Aggressive response |
| Wolf Howl | Wolves | Entire pack responds |
| Bear Roar | Bears | Very dangerous |
| Distress Call | Predators | They come to investigate |

---

## Tier 8: Ultimate Skills

### Apex Predator
**Unlock:** Max both Beast Slayer and Shadow Hunter
**Description:** You have become the most dangerous hunter in the wilderness.

**Effects:**
- All animals have reduced aggression toward you (-50%)
- Predators may submit rather than fight (20% chance)
- Can "claim" a territory (animals avoid your camp)
- Unlocks taming mechanic (befriend wolves)

**Apex Abilities:**
| Ability | Effect | Cooldown |
|---------|--------|----------|
| Predator's Roar | All animals in 30m flee | 5 min |
| Alpha Presence | Wolves won't attack | Passive |
| Intimidating Gaze | Single animal freezes | 1 min |
| Territory Claim | 50m radius safe zone | Permanent |

**Wolf Companion:**
- Can tame injured wolf by feeding it
- Wolf follows and assists in combat
- Wolf warns of nearby predators
- Only one companion at a time

---

### Master Trapper
**Unlock:** Max both Snare Master and Lure Crafter
**Description:** Your traps are legendary—none escape your snares.

**Effects:**
- Can trap ANY animal including legendary beasts
- Passive trap income (traps work while you're away)
- Can craft "trap networks" (multiple linked traps)
- Unlocks trading post (sell furs to NPCs)

**Trap Network:**
- Connect up to 5 traps in a line
- First trap triggers, others activate if animal flees
- Guaranteed capture on networked traps
- Covers large area efficiently

**Trading Post:**
- Automated fur collection
- NPCs visit to purchase goods
- Reputation increases prices
- Can order specific supplies

---

## Tier 9: Legendary Mastery

### Legendary Hunter
**Unlock:** Complete both Apex Predator AND Master Trapper
**Description:** Songs are sung of your hunts. You are one with the wilderness.

**Effects:**
- All hunting skills at maximum effectiveness
- Can sense any animal on the entire map
- Legendary beasts can spawn (unique hunts)
- Spirit animal companion (choose one)
- Title: "Legendary Hunter" visible to all

**Spirit Animal Companions:**
| Spirit | Passive Bonus | Active Ability |
|--------|---------------|----------------|
| Spirit Bear | +50% health | Roar: Stun all enemies 3s |
| Spirit Cougar | +50% speed | Pounce: Teleport to target |
| Spirit Wolf | +50% damage | Pack Call: Summon wolf spirits |
| Spirit Serpent | Poison immunity | Venomous Strike: DoT attack |
| Spirit Alligator | Water breathing | Death Roll: Massive damage |

**Legendary Hunts:**
| Beast | Location | Reward |
|-------|----------|--------|
| The Ghost Cougar | Mountain peaks | Invisibility cloak |
| Fenrir (Giant Wolf) | Deep forest | Fenrir's Fang blade |
| The Swamp King (Mega Gator) | Deepest swamp | Impenetrable armor |
| Old Silverback (Huge Bear) | Ancient cave | Bear spirit token |
| The Serpent Mother | Hidden grotto | Poison mastery |

---

## Integration with Existing Systems

### Animal System Integration

**Behavior Counters:**
| Animal Behavior | Hunting Counter |
|-----------------|-----------------|
| Territorial (Bear) | Know boundaries, don't trigger |
| Stalker (Cougar) | Detect stalking, face threat |
| Pack Hunter (Wolves) | Target alpha, scatter pack |
| Ambush (Snakes, Gator) | Scan before stepping |
| Aggressive (Boar) | Sidestep charges |
| Hidden (Copperhead) | Ground awareness |

**Damage Bonus Stacking:**
| Source | Bonus |
|--------|-------|
| Species skill (Boar Hunter, etc.) | +25% |
| Category skill (Beast Slayer) | +50% |
| Weapon weakness match | +50% |
| Stealth attack | +200% |
| Critical hit | +100% |

### Archaeology Integration

**Fossil-Enhanced Hunting:**
| Archaeology Item | Hunting Bonus |
|------------------|---------------|
| Bone Talisman | +10% tracking range |
| Dire Fang Knife | +25% skinning yield |
| Giant's Tooth Necklace | Predators less aggressive |
| Ancestral Horn | Can call prey animals |

### NPC Village Integration

**Native Hunter NPCs:**
- Can teach hunting skills (alternate unlock path)
- Trade pelts for village goods
- Request specific animal kills (quests)
- Share hunting ground locations

**Hunter Reputation:**
| Level | Benefit |
|-------|---------|
| Stranger | Basic trading |
| Known Hunter | Skill training available |
| Respected | Quest access |
| Master Hunter | Secret hunting grounds revealed |
| Legend | Village protection, best prices |

---

## Loot Tables

### Skinning Yields by Animal

| Animal | Hide | Meat | Special | Rare |
|--------|------|------|---------|------|
| Black Bear | Bear Pelt (80%) | Bear Meat x3 (100%) | Bear Claws (40%) | Bear Heart (10%) |
| Eastern Cougar | Cougar Pelt (80%) | Cougar Meat x2 (100%) | Cougar Fangs (50%) | Cougar Eye (5%) |
| Gray Wolf | Wolf Pelt (85%) | Wolf Meat x2 (100%) | Wolf Fangs (45%) | Alpha Mane (5%*) |
| Timber Rattlesnake | Snakeskin (90%) | Snake Meat (100%) | Rattles (60%) | Venom Sac (25%) |
| American Alligator | Gator Hide (75%) | Gator Meat x4 (100%) | Gator Teeth (50%) | Gator Skull (8%) |
| Wild Boar | Boar Hide (85%) | Boar Meat x3 (100%) | Boar Tusks (40%) | Boar Heart (10%) |
| Copperhead | Snakeskin (90%) | — | Fangs (50%) | Venom Sac (20%) |
| Red Wolf | Wolf Pelt (85%) | Wolf Meat x2 (100%) | Wolf Fangs (45%) | — |
| Bobcat | Bobcat Pelt (85%) | Bobcat Meat (100%) | Claws (40%) | — |
| Cottonmouth | Snakeskin (90%) | — | Fangs (50%) | Venom Sac (30%) |

*Alpha Mane only drops from pack alphas

### Skinning Skill Modifiers

| Skill Level | Yield Modifier | Rare Drop Modifier |
|-------------|----------------|-------------------|
| Basic (50%) | 1.0x | 1.0x |
| Prey Instinct | 1.5x (prey) | 1.25x |
| Beast Slayer | 1.5x (predators) | 1.5x |
| Legendary Hunter | 2.0x (all) | 2.0x |

---

## Crafting Recipes

### Basic Hunting Gear

| Item | Materials | Effect |
|------|-----------|--------|
| Hunting Bow | Wood + Sinew + Feathers | Basic ranged weapon |
| Hunting Spear | Wood + Flint + Leather | Melee reach weapon |
| Skinning Knife | Iron + Wood | +10% skinning yield |
| Quiver | Leather + Wood | Carry +20 arrows |

### Advanced Hunting Gear

| Item | Materials | Required Skill | Effect |
|------|-----------|----------------|--------|
| Stalker's Bow | Hardwood + Cougar Gut + Feathers | Shadow Hunter | Silent shots |
| Bear Spear | Ironwood + Bear Claws + Leather | Big Game Hunter | +50% vs large |
| Tracker's Boots | Boar Hide + Wolf Pelt + Rubber | Wilderness Scout | Silent movement |
| Hunter's Cloak | 3 Mixed Pelts + Feathers | Predator Sense | Reduced detection |

### Trophy Crafting

| Trophy | Materials | Display Effect |
|--------|-----------|----------------|
| Bear Head Mount | Bear Skull + Wood Frame | Intimidation aura at camp |
| Wolf Pelt Rug | 2 Wolf Pelts + Frame | Comfort bonus at camp |
| Gator Tooth Necklace | 5 Gator Teeth + Sinew | +15% vs reptiles |
| Cougar Claw Bracers | 4 Cougar Claws + Leather | +10% attack speed |
| Serpent Fang Dagger | 3 Snake Fangs + Wood | Poison damage on hit |

---

## Skill Point Acquisition

| Action | Points |
|--------|--------|
| First kill of species | 50 |
| Kill any animal | 10 |
| Perfect kill (no damage taken) | 25 |
| Stealth kill | 20 |
| Trap capture | 15 |
| Skin animal | 5 |
| Discover den/nest | 30 |
| Survive predator encounter | 40 |
| Complete hunting quest | 100 |
| Legendary beast kill | 500 |

**Points Required Per Tier:**
- Tier 1 → Tier 2: 100 points
- Tier 2 → Tier 3: 200 points
- Tier 3 → Tier 4: 400 points
- Tier 4 → Tier 5: 750 points
- Tier 5 → Tier 6: 1,250 points
- Tier 6 → Tier 7: 2,000 points
- Tier 7 → Tier 8: 3,500 points
- Tier 8 → Tier 9: 6,000 points

---

## Data Structures (Rust)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HuntingSkills {
    pub points: u32,

    // Tier 1
    pub basic_tracker: bool,

    // Tier 2
    pub boar_hunter: bool,
    pub deer_stalker: bool,

    // Tier 3
    pub wolf_tracker: bool,
    pub serpent_eye: bool,

    // Tier 4
    pub predator_sense: bool,
    pub prey_instinct: bool,

    // Tier 5
    pub wilderness_scout: bool,

    // Tier 6
    pub big_game_hunter: bool,
    pub trap_setter: bool,

    // Tier 7
    pub beast_slayer: bool,
    pub shadow_hunter: bool,
    pub snare_master: bool,
    pub lure_crafter: bool,

    // Tier 8
    pub apex_predator: bool,
    pub master_trapper: bool,

    // Tier 9
    pub legendary_hunter: bool,

    // Tracking data
    pub kills_by_species: HashMap<AnimalSpecies, u32>,
    pub stealth_kills: u32,
    pub trap_captures: u32,
    pub dens_discovered: u32,
    pub perfect_kills: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TrapType {
    Snare,
    Deadfall,
    PitTrap,
    JawTrap,
    NetTrap,
    BearTrap,
    VenomSnare,
    AlarmTrap,
    ComboTrap,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trap {
    pub trap_type: TrapType,
    pub position: Vec3,
    pub rotation: f32,
    pub durability: f32,
    pub baited: bool,
    pub bait_type: Option<LureType>,
    pub triggered: bool,
    pub captured_animal: Option<AnimalId>,
    pub placed_time: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LureType {
    MeatScraps,
    BloodBait,
    FishBait,
    MuskLure,
    RodentLure,
    HoneyBait,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnimalCall {
    DeerBleat,
    BoarGrunt,
    WolfHowl,
    BearRoar,
    DistressCall,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimalDen {
    pub position: Vec3,
    pub species: AnimalSpecies,
    pub discovered: bool,
    pub marked_on_map: bool,
    pub spawn_count: u8,
    pub last_spawn_day: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HuntingStats {
    pub total_kills: u32,
    pub largest_kill: Option<(AnimalSpecies, f32)>,  // species, size
    pub fastest_kill: Option<(AnimalSpecies, f32)>,  // species, seconds
    pub longest_stalk: Option<(AnimalSpecies, f32)>, // species, meters
    pub legendary_kills: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpiritAnimal {
    Bear,
    Cougar,
    Wolf,
    Serpent,
    Alligator,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WolfCompanion {
    pub name: String,
    pub health: f32,
    pub max_health: f32,
    pub loyalty: f32,  // 0.0 - 1.0
    pub position: Vec3,
    pub state: CompanionState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CompanionState {
    Following,
    Attacking(AnimalId),
    Guarding(Vec3),
    Resting,
    Hunting,
}
```

---

## Audio/Visual Feedback

### Tracking
- Animal tracks pulse with faint glow
- Directional audio for nearby animals
- Blood trails shimmer in tracking mode
- Territory boundaries shown as faint mist

### Combat
- Hit sounds vary by weapon and animal
- Death cries unique to each species
- Slow-motion on perfect/critical kills
- Blood splatter effects

### Skill Progression
- Unlock fanfare with hunting horn sound
- New ability visual demonstration
- Title card for major milestones
- Trophy room visualization

---

## Implementation Priority

### Phase 1 (Core)
- [ ] Basic tracking system (HUD indicators)
- [ ] Track visual spawning on ground
- [ ] Basic skinning mechanic
- [ ] Hunting skill point tracking

### Phase 2 (Progression)
- [ ] Tier 1-3 skills
- [ ] Species-specific bonuses
- [ ] Stealth damage system
- [ ] Kill tracking per species

### Phase 3 (Trapping)
- [ ] Tier 4-5 skills
- [ ] Trap crafting and placement
- [ ] Bait/lure system
- [ ] Trap triggering mechanics

### Phase 4 (Mastery)
- [ ] Tier 6-7 skills
- [ ] Territory/den discovery
- [ ] Advanced crafting recipes
- [ ] Trophy system

### Phase 5 (Legendary)
- [ ] Tier 8-9 skills
- [ ] Wolf companion system
- [ ] Spirit animal selection
- [ ] Legendary beast spawning
- [ ] Legendary hunts questline

---

## Balance Considerations

### Damage Scaling
To prevent hunting from becoming trivial:
- Skill bonuses are multiplicative, not additive
- Max theoretical damage boost: ~8x (stacking everything)
- Legendary beasts have damage resistance
- Perfect kills require genuine skill (timing windows)

### Economy
- Pelts have weight (inventory management)
- Traders have limited gold
- Legendary materials are trade-restricted
- Some items are "soulbound" (can't be traded)

### Difficulty Scaling
| Difficulty | Animal HP | Detection Range | Loot Modifier |
|------------|-----------|-----------------|---------------|
| Easy | 0.75x | 0.75x | 1.25x |
| Normal | 1.0x | 1.0x | 1.0x |
| Hard | 1.5x | 1.25x | 0.85x |
| Survival | 2.0x | 1.5x | 0.7x |
