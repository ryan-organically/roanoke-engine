# Faction System Specification

**Document Version**: 1.0
**Last Updated**: December 2024
**Status**: Design Phase
**Game Era**: 1580s Virginia Roanoke Wilderness

---

## Table of Contents

1. [Overview & Design Philosophy](#overview--design-philosophy)
2. [Faction Relationship Matrix](#faction-relationship-matrix)
3. [European Colonial Powers](#european-colonial-powers)
   - [Spanish Conquistadors](#spanish-conquistadors)
   - [French Coureurs des Bois](#french-coureurs-des-bois)
   - [English Colonists](#english-colonists)
4. [Mesoamerican Powers](#mesoamerican-powers)
   - [Aztec Remnants (Mexica Exiles)](#aztec-remnants-mexica-exiles)
5. [Native American Nations](#native-american-nations)
   - [Powhatan Confederacy](#powhatan-confederacy)
   - [Tuscarora Nation](#tuscarora-nation)
   - [Cherokee Nation](#cherokee-nation)
   - [Catawba Nation](#catawba-nation)
   - [Pamunkey Tribe](#pamunkey-tribe)
6. [Reputation & Standing System](#reputation--standing-system)
7. [Trade Networks & Economics](#trade-networks--economics)
8. [Rust Data Structures](#rust-data-structures)
9. [Implementation Priority](#implementation-priority)

---

## Overview & Design Philosophy

The faction system represents the complex web of alliances, rivalries, and survival strategies that defined the 1580s colonial frontier. Each faction possesses:

- **Cultural Traits**: Passive bonuses reflecting their heritage and expertise
- **Signature Skill Tree**: Unique progression path unavailable to outsiders
- **Class Weapons**: Culturally significant armaments with special mechanics
- **Special Abilities**: Active powers tied to faction standing
- **Interrelationships**: Dynamic reputation affecting trade, combat, and quests

### Core Principles

1. **Historical Authenticity**: Mechanics grounded in documented 16th-century practices
2. **No "Evil" Factions**: Every group has understandable motivations and values
3. **Fluid Allegiances**: Player actions determine faction standing over time
4. **Cultural Exchange**: High reputation unlocks cross-faction learning opportunities
5. **Survival Priority**: All factions ultimately prioritize survival in the wilderness

---

## Faction Relationship Matrix

```
                    SPA   FRA   ENG   AZT   POW   TUS   CHE   CAT   PAM
Spanish (SPA)        -    -2    -3    -3    -1    -1    -1    -1    -1
French (FRA)        -2     -    -1    +1    +2    +1    +1    +2    +1
English (ENG)       -3    -1     -    -1    -2    -1     0     0    -2
Aztec (AZT)         -3    +1    -1     -    +1    +2    +1     0    +1
Powhatan (POW)      -1    +2    -2    +1     -    -1    -2    -1    +3
Tuscarora (TUS)     -1    +1    -1    +2    -1     -    +2    +1     0
Cherokee (CHE)      -1    +1     0    +1    -2    +2     -    +1    -1
Catawba (CAT)       -1    +2     0     0    -1    +1    +1     -     0
Pamunkey (PAM)      -1    +1    -2    +1    +3     0    -1     0     -

Legend: -3 (War) | -2 (Hostile) | -1 (Suspicious) | 0 (Neutral) | +1 (Friendly) | +2 (Allied) | +3 (Blood Bond)
```

### Relationship Dynamics

| Relationship | Effect on Player |
|--------------|------------------|
| War (-3) | Attack on sight, no trade, bounty placed |
| Hostile (-2) | Aggressive patrols, 300% trade prices, restricted areas |
| Suspicious (-1) | Watched closely, 150% trade prices, limited dialogue |
| Neutral (0) | Standard interactions, normal prices |
| Friendly (+1) | Discount trades (85%), side quests available |
| Allied (+2) | Deep discounts (70%), skill training available |
| Blood Bond (+3) | Full faction benefits, marriage options, leadership roles |

---

## European Colonial Powers

---

### Spanish Conquistadors

*"God, Gold, and Glory"*

The remnants of failed expeditions and deserters from Spanish Florida, these hardened soldiers seek riches and redemption in the Virginia wilderness. They bring superior metallurgy, gunpowder expertise, and a ruthless efficiency forged in decades of New World conquest.

#### Cultural Traits

| Trait | Effect | Description |
|-------|--------|-------------|
| **Steel Supremacy** | +15% melee damage with metal weapons | Generations of Toledo steel smithing |
| **Gunpowder Mastery** | -20% reload time for firearms | Extensive arquebus training |
| **Inquisitor's Eye** | +25% detection of hidden enemies | Paranoid vigilance honed fighting guerrillas |
| **Gold Fever** | +30% valuable item detection range | Obsessive treasure hunting instincts |
| **Conquistador's Constitution** | +10% disease resistance | Exposure to Old World plagues |

#### Signature Skill Tree: Way of the Conquistador

```
                        [CONQUISTADOR INITIATE]
                               |
              +----------------+----------------+
              |                                 |
     [SWORD & BUCKLER]                  [ARQUEBUS MASTERY]
              |                                 |
    +---------+---------+              +--------+--------+
    |                   |              |                 |
[TERCIO          [DUELIST'S      [VOLLEY         [MARKSMAN'S
 FORMATION]       GRACE]          FIRE]           PATIENCE]
    |                   |              |                 |
    +--------+----------+              +--------+--------+
             |                                  |
      [STEEL TEMPEST]                   [THUNDER OF GOD]
             |                                  |
             +----------------+-----------------+
                              |
                    [GOLD & GLORY]
                              |
                   [EL CONQUISTADOR]
```

##### Tier 1: Conquistador Initiate
- **Unlock**: Join Spanish faction or reach Friendly (+1) standing
- **Effects**:
  - Access to Spanish weapons and armor
  - Can purchase gunpowder from Spanish traders
  - Learn basic Spanish (enables faction dialogue)

##### Tier 2A: Sword & Buckler
- **Prerequisite**: Conquistador Initiate
- **Unlock**: Kill 25 enemies with melee weapons
- **Effects**:
  - +10% block effectiveness with small shields
  - New combo: Buckler bash (stuns for 1.5s)
  - Parry window increased by 0.2s

##### Tier 2B: Arquebus Mastery
- **Prerequisite**: Conquistador Initiate
- **Unlock**: Kill 15 enemies with firearms
- **Effects**:
  - -15% arquebus sway while aiming
  - Faster powder priming (reload phase 1 reduced 0.5s)
  - Can craft paper cartridges for faster reloads

##### Tier 3A: Tercio Formation
- **Prerequisite**: Sword & Buckler
- **Unlock**: Complete 3 battles alongside Spanish allies
- **Effects**:
  - When near 2+ allies: +20% damage resistance
  - Allied Spanish NPCs gain +10% accuracy
  - Unlocks "Hold the Line" command (allies form defensive square)

##### Tier 3B: Duelist's Grace
- **Prerequisite**: Sword & Buckler
- **Unlock**: Win 10 one-on-one melee fights
- **Effects**:
  - +25% movement speed while sword is drawn
  - Riposte attacks deal +40% damage
  - New move: Estocada (lunging thrust, 2x damage, 8s cooldown)

##### Tier 3C: Volley Fire
- **Prerequisite**: Arquebus Mastery
- **Unlock**: Kill 3 enemies with a single shot (penetration)
- **Effects**:
  - When firing with allies: synchronized volley deals +50% damage
  - Smoke screen after volley (-30% enemy accuracy for 5s)
  - Unlocks "Fire by Rank" command

##### Tier 3D: Marksman's Patience
- **Prerequisite**: Arquebus Mastery
- **Unlock**: Land 20 headshots with firearms
- **Effects**:
  - Holding aim for 3s grants "Perfect Shot" (+100% crit chance)
  - Crouch stability bonus increased to +40%
  - Can hold breath for 8s instead of 4s

##### Tier 4A: Steel Tempest
- **Prerequisite**: Tercio Formation OR Duelist's Grace
- **Unlock**: Kill 50 enemies with Spanish steel weapons
- **Effects**:
  - Sword attacks have 15% chance to cause bleed
  - New ability: Whirlwind (360 attack hitting all nearby, 15s cooldown)
  - Toledo steel weapons never degrade below 50% condition

##### Tier 4B: Thunder of God
- **Prerequisite**: Volley Fire OR Marksman's Patience
- **Unlock**: Kill a legendary animal with a firearm
- **Effects**:
  - Firearms cause "Terrified" status on nearby enemies (flee for 3s)
  - +20% damage vs animals
  - Can craft incendiary rounds (fire damage over time)

##### Tier 5: Gold & Glory
- **Prerequisite**: Steel Tempest OR Thunder of God
- **Unlock**: Accumulate 5000 gold worth of treasure
- **Effects**:
  - Sixth sense for buried treasure (UI indicator within 50m)
  - +50% sell price for all valuable items
  - Spanish merchants offer exclusive legendary items

##### Tier 6: El Conquistador (Ultimate)
- **Prerequisite**: Gold & Glory + Allied standing with Spanish
- **Unlock**: Complete the "Seven Cities" questline
- **Effects**:
  - Title: "El Conquistador" (feared by all hostile factions)
  - Unique armor: Gilded Morion (best helmet in game)
  - Ability: "Conqueror's Presence" - enemies within 20m have -25% morale
  - Can establish Spanish outposts anywhere on map
  - Command up to 8 Spanish soldier followers

#### Class Weapons

| Weapon | Type | Base Damage | Special Property |
|--------|------|-------------|------------------|
| Toledo Espada Ropera | Sword | 45 | +20% parry window, causes bleed |
| Conquistador's Rodela | Buckler | 5 (bash) | +25% block arc, fast recovery |
| Spanish Arquebus | Firearm | 120 | Superior accuracy, slow reload (5s) |
| Alabarda | Polearm | 55 | Armor penetration, dismounts riders |
| Daga de Misericordia | Dagger | 25 | +100% damage to prone enemies |

#### Special Abilities

| Ability | Unlock | Cooldown | Effect |
|---------|--------|----------|--------|
| **Santiago!** | Tier 2 | 120s | War cry: +30% damage, +20% speed for 10s |
| **Steel Wall** | Tier 3A | 90s | Block all frontal damage for 5s |
| **Estocada** | Tier 3B | 8s | Lunging thrust, 2x damage |
| **Divine Judgment** | Tier 4B | 180s | Next firearm shot guaranteed critical |
| **Conqueror's Presence** | Tier 6 | Passive | Enemies within 20m have -25% morale |

---

### French Coureurs des Bois

*"The Forest is Our Cathedral"*

French-Canadian woodsmen, fur traders, and adventurers who have "gone native" more than any other Europeans. They maintain the best relationships with Native peoples, serving as translators, traders, and cultural bridges. Their survival skills rival indigenous hunters.

#### Cultural Traits

| Trait | Effect | Description |
|-------|--------|-------------|
| **Voyageur's Endurance** | +20% stamina, +15% carrying capacity | Years of portaging canoes |
| **Trade Tongue** | +1 starting reputation with all Native factions | Cultural sensitivity |
| **Master Trapper** | +40% pelt quality from trapped animals | Superior trapping techniques |
| **River Runner** | +30% canoe speed, no stamina cost for paddling | Born on the waterways |
| **Winter Hardened** | No movement penalty in snow, -50% cold damage | Canadian winters |

#### Signature Skill Tree: Path of the Coureur

```
                        [APPRENTI VOYAGEUR]
                               |
              +----------------+----------------+
              |                                 |
       [FUR TRADE]                      [FOREST WISDOM]
              |                                 |
    +---------+---------+              +--------+--------+
    |                   |              |                 |
[MASTER           [NEGOTIATOR'S   [SILENT         [HERBALIST'S
 TRAPPER]          TONGUE]         SHADOW]          CRAFT]
    |                   |              |                 |
    +--------+----------+              +--------+--------+
             |                                  |
      [TRADE EMPIRE]                    [ONE WITH LAND]
             |                                  |
             +----------------+-----------------+
                              |
                     [SPIRIT BRIDGE]
                              |
                    [GRAND VOYAGEUR]
```

##### Tier 1: Apprenti Voyageur
- **Unlock**: Join French faction or complete fur trade tutorial
- **Effects**:
  - Can craft basic traps (snare, deadfall)
  - Canoe handling unlocked
  - Basic trade dialogue with Native villages

##### Tier 2A: Fur Trade
- **Prerequisite**: Apprenti Voyageur
- **Unlock**: Sell 50 pelts to any trader
- **Effects**:
  - Pelts stack to 50 instead of 20
  - Can identify animal quality before kill
  - +20% pelt sale prices

##### Tier 2B: Forest Wisdom
- **Prerequisite**: Apprenti Voyageur
- **Unlock**: Survive 10 days in wilderness without entering settlements
- **Effects**:
  - Natural shelter construction (no materials needed)
  - Edible plant highlighting in vision
  - Weather prediction 24 hours ahead

##### Tier 3A: Master Trapper
- **Prerequisite**: Fur Trade
- **Unlock**: Trap 100 animals
- **Effects**:
  - Advanced traps: Steel jaw, underwater, elevated
  - Traps reset automatically once
  - Animals cannot detect your trap scent

##### Tier 3B: Negotiator's Tongue
- **Prerequisite**: Fur Trade
- **Unlock**: Complete 20 trades with 5+ different factions
- **Effects**:
  - All trade prices improved by 15%
  - Can defuse hostile encounters with dialogue (50% chance)
  - Reputation gains doubled

##### Tier 3C: Silent Shadow
- **Prerequisite**: Forest Wisdom
- **Unlock**: Remain undetected for 30 cumulative minutes near enemies
- **Effects**:
  - Movement sound reduced by 60%
  - Can move at full speed while crouched in vegetation
  - Animal awareness range reduced by 40% against you

##### Tier 3D: Herbalist's Craft
- **Prerequisite**: Forest Wisdom
- **Unlock**: Craft 50 herbal remedies
- **Effects**:
  - Double yield from gathered plants
  - Can identify poisonous vs edible variants
  - Craft "Coureur's Tonic" (+50% stamina regen for 10min)

##### Tier 4A: Trade Empire
- **Prerequisite**: Master Trapper OR Negotiator's Tongue
- **Unlock**: Accumulate 3000 gold through fur trading
- **Effects**:
  - Establish trading posts (passive income)
  - Hire Native guides (2 followers)
  - Access to French black market (rare European goods)

##### Tier 4B: One With the Land
- **Prerequisite**: Silent Shadow OR Herbalist's Craft
- **Unlock**: Survive all four seasons in the wilderness
- **Effects**:
  - Passive health regeneration in forests (+1 HP/s)
  - Can befriend one wild animal permanently
  - Immune to poison from plants and insects

##### Tier 5: Spirit Bridge
- **Prerequisite**: Trade Empire OR One With the Land
- **Unlock**: Reach Allied (+2) standing with any 3 Native factions
- **Effects**:
  - Can participate in Native ceremonies (stat bonuses)
  - Native villages share resources freely
  - Learn one skill from any Native skill tree

##### Tier 6: Grand Voyageur (Ultimate)
- **Prerequisite**: Spirit Bridge + complete "Northwest Passage" questline
- **Effects**:
  - Title: "Grand Voyageur" (legendary status among traders)
  - Unique outfit: Coureur's Capote (best cold/stealth gear)
  - Ability: "Spirit Walk" - become invisible for 20s (300s cooldown)
  - All waterways revealed on map
  - Can fast travel between any water-adjacent locations

#### Class Weapons

| Weapon | Type | Base Damage | Special Property |
|--------|------|-------------|------------------|
| Tomahawk Français | Throwing Axe | 35 | Returns on miss, +50% vs fleeing enemies |
| Fusil de Chasse | Hunting Rifle | 90 | Superior accuracy, +30% vs animals |
| Couteau de Traite | Trade Knife | 20 | +100% skinning speed, silent kills |
| Voyageur's Paddle | Club | 30 | Doubles as canoe paddle, stuns fish |
| Musket-Hatchet Combo | Hybrid | 70/35 | Switch between ranged and melee instantly |

#### Special Abilities

| Ability | Unlock | Cooldown | Effect |
|---------|--------|----------|--------|
| **Portage** | Tier 1 | Passive | Carry canoe overland at 80% speed |
| **Trade Parley** | Tier 3B | 300s | Initiate peaceful dialogue with any hostile group |
| **Ghost Walk** | Tier 3C | 60s | 10s of enhanced stealth (footsteps silent) |
| **Nature's Bounty** | Tier 4B | Passive | Double gathering yield in wilderness |
| **Spirit Walk** | Tier 6 | 300s | Full invisibility for 20s |

---

### English Colonists

*"For Queen and Country"*

The desperate survivors and new arrivals of the Roanoke colony. Poorly adapted to the wilderness but possessing superior organizational skills, naval connections, and a stubborn determination to establish a permanent foothold. They represent the player's default starting faction.

#### Cultural Traits

| Trait | Effect | Description |
|-------|--------|-------------|
| **Colonial Grit** | +15% health, slower starvation | Stubborn survival instinct |
| **Naval Connections** | Access to ship-delivered supplies monthly | Crown supply chains |
| **Protestant Work Ethic** | +25% construction speed | Industrious building |
| **Common Law** | Reduced reputation loss from crimes | Presumption of innocence |
| **Island Mentality** | +20% defense bonus in fortified structures | Defensively minded |

#### Signature Skill Tree: Colonist's Path

```
                        [ROANOKE SETTLER]
                               |
              +----------------+----------------+
              |                                 |
      [FORTIFICATION]                   [FRONTIER SURVIVAL]
              |                                 |
    +---------+---------+              +--------+--------+
    |                   |              |                 |
[MASTER            [MILITIA      [WILDERNESS     [COLONIAL
 BUILDER]          CAPTAIN]       SCOUT]          FARMER]
    |                   |              |                 |
    +--------+----------+              +--------+--------+
             |                                  |
      [COLONIAL LEADER]                [FRONTIER MASTER]
             |                                  |
             +----------------+-----------------+
                              |
                    [NEW WORLD GOVERNOR]
                              |
                     [LORD OF ROANOKE]
```

##### Tier 1: Roanoke Settler
- **Unlock**: Default starting tree for new players
- **Effects**:
  - Basic English dialogue and faction interactions
  - Access to colonial settlement storage
  - Can request basic supplies from ships

##### Tier 2A: Fortification
- **Prerequisite**: Roanoke Settler
- **Unlock**: Build 10 structures
- **Effects**:
  - Palisade construction unlocked
  - Structures have +25% durability
  - Can repair structures 50% faster

##### Tier 2B: Frontier Survival
- **Prerequisite**: Roanoke Settler
- **Unlock**: Survive 5 days away from colonial settlement
- **Effects**:
  - Basic hunting unlocked (no skill penalty)
  - Can purify water with fire
  - Recognize dangerous wildlife

##### Tier 3A: Master Builder
- **Prerequisite**: Fortification
- **Unlock**: Build a complete home (4+ rooms)
- **Effects**:
  - Advanced structures: Stone walls, watchtowers, wells
  - Structures provide comfort bonus (+stamina regen)
  - Can build defensive cannon emplacements

##### Tier 3B: Militia Captain
- **Prerequisite**: Fortification
- **Unlock**: Successfully defend settlement from 3 attacks
- **Effects**:
  - Command up to 6 militia NPCs
  - Militia accuracy +20% when you're present
  - Unlock "Rally" command (militia regroups at your position)

##### Tier 3C: Wilderness Scout
- **Prerequisite**: Frontier Survival
- **Unlock**: Discover 20 points of interest
- **Effects**:
  - Extended minimap range (+50%)
  - Can mark locations for other players/NPCs
  - Fast travel between discovered English outposts

##### Tier 3D: Colonial Farmer
- **Prerequisite**: Frontier Survival
- **Unlock**: Harvest 100 crops
- **Effects**:
  - Crop yield +50%
  - Can grow European crops (wheat, turnips)
  - Livestock breeding unlocked

##### Tier 4A: Colonial Leader
- **Prerequisite**: Master Builder OR Militia Captain
- **Unlock**: Settlement reaches 20 population
- **Effects**:
  - Assign NPC jobs and schedules
  - Settlement generates passive resources
  - Can establish trade agreements with other factions

##### Tier 4B: Frontier Master
- **Prerequisite**: Wilderness Scout OR Colonial Farmer
- **Unlock**: Complete expedition to 3 distant regions
- **Effects**:
  - All terrain movement penalties halved
  - Can establish remote outposts
  - Wilderness survival bonuses apply everywhere

##### Tier 5: New World Governor
- **Prerequisite**: Colonial Leader OR Frontier Master
- **Unlock**: Control 3+ settlements
- **Effects**:
  - Title: "Governor" (diplomatic authority)
  - Can negotiate treaties with Native factions
  - Monthly supply shipments include rare items

##### Tier 6: Lord of Roanoke (Ultimate)
- **Prerequisite**: New World Governor + complete "Lost Colony" questline
- **Effects**:
  - Title: "Lord of Roanoke"
  - Unique structure: Governor's Mansion (best comfort/defense)
  - Can declare war or peace with any faction
  - Settlers from England arrive monthly (population growth)
  - Exclusive access to Crown armory (best English equipment)

#### Class Weapons

| Weapon | Type | Base Damage | Special Property |
|--------|------|-------------|------------------|
| English Longbow | Bow | 50 | Extreme range (120m), armor pierce |
| Brown Bess Musket | Firearm | 100 | Reliable, bayonet attachment |
| Hanger Sword | Sword | 40 | Fast attacks, naval boarding bonus |
| Billhook | Polearm | 45 | Can pull enemies, +50% vs cavalry |
| Buckler & Cudgel | Shield/Club | 25 | High block chance, non-lethal option |

#### Special Abilities

| Ability | Unlock | Cooldown | Effect |
|---------|--------|----------|--------|
| **For the Queen!** | Tier 2 | 180s | +20% all stats for 15s |
| **Defensive Formation** | Tier 3B | 120s | All allies gain +30% defense for 20s |
| **Signal Fire** | Tier 3C | 600s | Call reinforcements from nearest settlement |
| **Colonial Resolve** | Tier 4A | 300s | Ignore next lethal hit, survive with 1 HP |
| **Crown's Authority** | Tier 6 | Passive | Diplomatic immunity in neutral territories |

---

## Mesoamerican Powers

---

### Aztec Remnants (Mexica Exiles)

*"The Sun Demands Blood"*

Refugees and warriors from the fallen Aztec Empire, fled north to escape Spanish domination. They bring ancient martial traditions, sophisticated medicine, and a burning desire for vengeance against the conquistadors. Their presence in Virginia is anachronistic but creates dramatic tension.

> *"We were gods once. The pale ones broke our temples, but they cannot break our spirits. In this new land, we will rise again."*

#### Cultural Traits

| Trait | Effect | Description |
|-------|--------|-------------|
| **Jaguar's Heart** | +20% melee damage, +10% attack speed | Warrior caste training |
| **Eagle's Vision** | Can see enemy health bars, +30% tracking range | Predator instincts |
| **Obsidian Edge** | Obsidian weapons cause +50% bleed damage | Master volcanic glass smiths |
| **Sacred Calendar** | Bonuses on specific days (+25% random stat) | Tonalpohualli prophecies |
| **Blood Sacrifice** | Killing enemies restores 5% health | Ritual combat tradition |

#### Signature Skill Tree: Path of the Warrior Sun

```
                        [MACEHUALTIN INITIATE]
                               |
              +----------------+----------------+
              |                                 |
      [JAGUAR WARRIOR]                   [EAGLE WARRIOR]
              |                                 |
    +---------+---------+              +--------+--------+
    |                   |              |                 |
[OCELOTL          [SHADOW        [CUAUHTLI       [SOLAR
 FURY]             STALKER]       STRIKE]         ASCENSION]
    |                   |              |                 |
    +--------+----------+              +--------+--------+
             |                                  |
      [JAGUAR KNIGHT]                   [EAGLE KNIGHT]
             |                                  |
             +----------------+-----------------+
                              |
                       [CUACHICQUEH]
                              |
                     [CHAMPION OF THE SUN]
```

##### Tier 1: Macehualtin Initiate
- **Unlock**: Join Aztec faction or prove yourself in combat (kill 20 Spanish)
- **Effects**:
  - Access to obsidian weapons and Aztec armor
  - Learn Nahuatl (faction dialogue)
  - Can participate in ritual combat training

##### Tier 2A: Jaguar Warrior (Ocelomeh)
- **Prerequisite**: Macehualtin Initiate
- **Unlock**: Capture (not kill) 4 enemies in combat
- **Effects**:
  - Jaguar Armor unlocked (+20% armor, intimidation bonus)
  - Pounce attack: leap 5m to target, bonus damage
  - +30% damage at night

##### Tier 2B: Eagle Warrior (Cuauhtli)
- **Prerequisite**: Macehualtin Initiate
- **Unlock**: Kill 4 enemies in a single combat without taking damage
- **Effects**:
  - Eagle Armor unlocked (+15% speed, +10% armor)
  - Diving attack from elevation deals 2x damage
  - +30% damage during day

##### Tier 3A: Ocelotl Fury
- **Prerequisite**: Jaguar Warrior
- **Unlock**: Kill 30 enemies in stealth
- **Effects**:
  - Can enter "Jaguar Rage" - +50% damage, -30% damage taken (30s, 180s CD)
  - Stealth kills restore 20% health
  - Intimidation aura: weak enemies flee

##### Tier 3B: Shadow Stalker
- **Prerequisite**: Jaguar Warrior
- **Unlock**: Track and kill 10 fleeing enemies
- **Effects**:
  - Can track blood trails for 10 minutes after wounding
  - Movement speed +20% when pursuing wounded target
  - Targets you wound cannot sprint

##### Tier 3C: Cuauhtli Strike
- **Prerequisite**: Eagle Warrior
- **Unlock**: Kill 10 enemies with aerial/diving attacks
- **Effects**:
  - Atlatl range +40%, can throw two darts rapidly
  - Diving attacks stun for 2s
  - Can glide short distances with eagle wings (costume feature)

##### Tier 3D: Solar Ascension
- **Prerequisite**: Eagle Warrior
- **Unlock**: Perform 20 kills during sunrise/sunset
- **Effects**:
  - During golden hour: all stats +15%
  - Solar blessing: immune to fire damage
  - Attacks create light, blinding enemies in dark areas

##### Tier 4A: Jaguar Knight
- **Prerequisite**: Ocelotl Fury OR Shadow Stalker
- **Unlock**: Capture a Spanish officer alive
- **Effects**:
  - Title: Jaguar Knight
  - Command pack of 3 trained jaguars
  - Ability: "Call of the Hunt" - marks all enemies in 100m

##### Tier 4B: Eagle Knight
- **Prerequisite**: Cuauhtli Strike OR Solar Ascension
- **Unlock**: Kill an enemy with an atlatl from 80m+
- **Effects**:
  - Title: Eagle Knight
  - Trained eagle companion (scouts, attacks eyes)
  - Ability: "Eagle's Cry" - reveals all hidden enemies

##### Tier 5: Cuachicqueh (Shorn One)
- **Prerequisite**: Jaguar Knight OR Eagle Knight
- **Unlock**: Win 50 ritual combat duels
- **Effects**:
  - Shaved head with warrior's lock (visual change)
  - Cannot retreat from combat (locked in until victory)
  - +50% damage when below 30% health
  - All Aztec warriors follow your commands

##### Tier 6: Champion of the Sun (Ultimate)
- **Prerequisite**: Cuachicqueh + complete "Vengeance of the Fifth Sun" questline
- **Effects**:
  - Title: "Champion of the Sun" (godlike status)
  - Unique weapon: Macuahuitl of Quetzalcoatl (best melee in game)
  - Ability: "Wrath of Huitzilopochtli" - AoE fire damage (300s CD)
  - Can sacrifice captured enemies for powerful temporary buffs
  - Aztec refugees establish hidden city under your leadership

#### Class Weapons

| Weapon | Type | Base Damage | Special Property |
|--------|------|-------------|------------------|
| Macuahuitl | Sword-Club | 55 | Obsidian blades cause severe bleeding |
| Tepoztopilli | Spear | 50 | Obsidian-tipped, +30% armor pierce |
| Atlatl & Tlacochtli | Ranged | 40 | Dart thrower, 2x range of throwing |
| Cuauhololli | Club | 35 | Stuns on hit, +50% vs armored |
| Tecpatl | Dagger | 30 | Ritual blade, +100% crit to unaware targets |
| Chimalli | Shield | 10 (bash) | Feathered shield, +20% vs projectiles |

#### Special Abilities

| Ability | Unlock | Cooldown | Effect |
|---------|--------|----------|--------|
| **Blood Offering** | Tier 1 | 60s | Sacrifice 20% HP for +30% damage (30s) |
| **Jaguar Pounce** | Tier 2A | 15s | Leap to target within 5m, bonus damage |
| **Eagle Dive** | Tier 2B | 20s | Aerial attack from elevation, 2x damage |
| **Jaguar Rage** | Tier 3A | 180s | +50% damage, -30% damage taken (30s) |
| **Wrath of Huitzilopochtli** | Tier 6 | 300s | AoE fire damage around self |

---

## Native American Nations

---

### Powhatan Confederacy

*"This Land Was Always Ours"*

The dominant power of the coastal Virginia region, led by the paramount chief Wahunsenacah (Powhatan). A confederacy of 30+ Algonquian-speaking tribes controlling territory from the Potomac to the James River. They are the primary Native faction players encounter.

#### Cultural Traits

| Trait | Effect | Description |
|-------|--------|-------------|
| **Children of Ahone** | +15% all stats near sacred sites | Spiritual connection to the land |
| **Tidewater Mastery** | +30% fishing yield, water navigation bonus | Coastal adaptation |
| **Confederacy Networks** | +2 starting reputation with allied tribes | Political connections |
| **Corn Mother's Blessing** | +25% crop growth, Three Sisters farming | Agricultural tradition |
| **Werowance Authority** | Can command allied tribe members | Political hierarchy |

#### Signature Skill Tree: Way of the Powhatan

```
                          [NEWCOMER]
                               |
              +----------------+----------------+
              |                                 |
       [HUNTER'S PATH]                  [DIPLOMAT'S PATH]
              |                                 |
    +---------+---------+              +--------+--------+
    |                   |              |                 |
[DEER             [RIVER          [PEACE           [WAR
 STALKER]          KEEPER]         WEAVER]          CHIEF]
    |                   |              |                 |
    +--------+----------+              +--------+--------+
             |                                  |
       [SPIRIT WALKER]                  [CONFEDERATE LORD]
             |                                  |
             +----------------+-----------------+
                              |
                       [WEROWANCE]
                              |
                     [MAMANATOWICK]
```

##### Tier 1: Newcomer
- **Unlock**: Reach Neutral (0) standing with Powhatan
- **Effects**:
  - Can enter Powhatan villages peacefully
  - Basic trade enabled
  - Learn greeting phrases in Algonquian

##### Tier 2A: Hunter's Path
- **Prerequisite**: Newcomer
- **Unlock**: Gift 20 deer pelts to Powhatan villages
- **Effects**:
  - Powhatan hunting grounds accessible
  - Can craft Powhatan hunting arrows
  - Deer spawn locations revealed in confederacy territory

##### Tier 2B: Diplomat's Path
- **Prerequisite**: Newcomer
- **Unlock**: Complete 5 diplomatic quests for Powhatan
- **Effects**:
  - Can propose trade agreements
  - Access to chief's longhouse
  - Political dialogue options unlocked

##### Tier 3A: Deer Stalker
- **Prerequisite**: Hunter's Path
- **Unlock**: Hunt 50 deer in Powhatan territory
- **Effects**:
  - "Deer Sense" - deer visible through vegetation
  - Movement makes no sound in forests
  - Can lure deer with calls

##### Tier 3B: River Keeper
- **Prerequisite**: Hunter's Path
- **Unlock**: Provide 100 fish to Powhatan villages
- **Effects**:
  - Fish trap construction
  - Can navigate rivers at night safely
  - Water purification innate (no tools needed)

##### Tier 3C: Peace Weaver
- **Prerequisite**: Diplomat's Path
- **Unlock**: Prevent 3 conflicts through negotiation
- **Effects**:
  - Can arrange marriages for alliance bonuses
  - Hostage exchanges restore reputation
  - -50% reputation loss from factional conflicts

##### Tier 3D: War Chief
- **Prerequisite**: Diplomat's Path
- **Unlock**: Lead Powhatan warriors in 5 successful battles
- **Effects**:
  - Command up to 10 Powhatan warriors
  - War paint bonuses (+15% damage, +10% intimidation)
  - Can declare war on behalf of Confederacy

##### Tier 4A: Spirit Walker
- **Prerequisite**: Deer Stalker OR River Keeper
- **Unlock**: Complete vision quest with shaman
- **Effects**:
  - Animal spirits provide warnings of danger
  - Can communicate with animal companions
  - Ancestral visions reveal quest objectives

##### Tier 4B: Confederate Lord
- **Prerequisite**: Peace Weaver OR War Chief
- **Unlock**: Unite 5 tribes under your influence
- **Effects**:
  - +30 tribute goods weekly from allied tribes
  - Can relocate tribe members between villages
  - Veto power over confederacy decisions

##### Tier 5: Werowance (Chief)
- **Prerequisite**: Spirit Walker OR Confederate Lord
- **Unlock**: Earn Blood Bond (+3) with Powhatan faction
- **Effects**:
  - Title: Werowance (tribal chief)
  - Personal village with 30 inhabitants
  - Can perform sacred ceremonies
  - Marriage to Powhatan nobility possible

##### Tier 6: Mamanatowick (Ultimate)
- **Prerequisite**: Werowance + complete "Unification" questline
- **Effects**:
  - Title: Mamanatowick (Paramount Chief)
  - All confederacy tribes answer your call
  - Unique weapon: Powhatan's War Club (chief's mace)
  - Ability: "Voice of the Land" - all Native factions +1 reputation
  - Can negotiate with English as equal sovereign power
  - Found new tribes in unclaimed territory

#### Class Weapons

| Weapon | Type | Base Damage | Special Property |
|--------|------|-------------|------------------|
| Powhatan Longbow | Bow | 45 | Silent, +20% vs deer |
| Tomahawk | Throwing/Melee | 35/30 | Versatile, returns on miss |
| War Club (Pogamoggan) | Club | 40 | Stuns armored enemies |
| Hunting Spear | Spear | 35 | +40% vs animals, can be thrown |
| Flint Knife | Dagger | 20 | +100% skinning speed, bleed |
| Tidewater Shield | Shield | 8 (bash) | Lightweight, +15% parry |

#### Special Abilities

| Ability | Unlock | Cooldown | Effect |
|---------|--------|----------|--------|
| **Hunter's Focus** | Tier 2A | 45s | Slow time while aiming (5s) |
| **River's Gift** | Tier 3B | Passive | Regenerate stamina in water |
| **War Paint** | Tier 3D | 600s | +15% damage, +10% intimidation (10min) |
| **Spirit Guide** | Tier 4A | 120s | Animal spirit scouts ahead |
| **Voice of the Land** | Tier 6 | 86400s | All Native factions +1 reputation |

---

### Tuscarora Nation

*"People of the Hemp"*

An Iroquoian-speaking nation of the Carolina Piedmont, known for sophisticated agriculture, hemp cultivation, and formidable warriors. They maintain an uneasy independence between the coastal confederacies and the interior Cherokee.

#### Cultural Traits

| Trait | Effect | Description |
|-------|--------|-------------|
| **Hemp Weavers** | +30% rope/textile crafting quality | Legendary fiber crafts |
| **Longhouse Unity** | +10% all stats when near clan members | Strong family bonds |
| **Three Sisters Masters** | +40% crop yield for corn/beans/squash | Agricultural expertise |
| **Piedmont Pathfinders** | +20% movement speed on hills | Highland navigation |
| **Revenge Tradition** | +25% damage vs faction that last killed you | Blood feud customs |

#### Signature Skill Tree: Tuscarora Tradition

```
                        [FRIEND OF TUSCARORA]
                               |
              +----------------+----------------+
              |                                 |
       [CLAN WARRIOR]                   [CLAN PROVIDER]
              |                                 |
    +---------+---------+              +--------+--------+
    |                   |              |                 |
[BEAR CLAN        [WOLF CLAN     [TURTLE CLAN    [DEER CLAN
 FURY]             PACK]          WISDOM]         GRACE]
    |                   |              |                 |
    +--------+----------+              +--------+--------+
             |                                  |
       [WAR CAPTAIN]                    [CLAN MOTHER]
             |                                  |
             +----------------+-----------------+
                              |
                       [PEACE CHIEF]
                              |
                     [KEEPER OF THE FIRE]
```

##### Tier 1: Friend of Tuscarora
- **Unlock**: Complete hemp trade quest or assist village
- **Effects**:
  - Trade access to Tuscarora villages
  - Can purchase hemp rope and textiles
  - Basic Tuscarora dialogue

##### Tier 2A: Clan Warrior
- **Prerequisite**: Friend of Tuscarora
- **Unlock**: Prove martial prowess in wrestling match
- **Effects**:
  - Access to clan markings
  - Tuscarora warrior weapons available
  - Can join war parties

##### Tier 2B: Clan Provider
- **Prerequisite**: Friend of Tuscarora
- **Unlock**: Donate 50 food items to village stores
- **Effects**:
  - Access to Three Sisters seeds
  - Can use village storage
  - Agricultural tool crafting unlocked

##### Tier 3A: Bear Clan Fury
- **Prerequisite**: Clan Warrior
- **Unlock**: Kill a bear in single combat
- **Effects**:
  - Bear Clan initiation (cosmetic tattoo)
  - +25% damage when below 50% health
  - Cannot be knocked down

##### Tier 3B: Wolf Clan Pack
- **Prerequisite**: Clan Warrior
- **Unlock**: Hunt with wolf clan warriors 10 times
- **Effects**:
  - Wolf Clan initiation
  - Allied wolves fight alongside you (up to 3)
  - Pack tactics: +15% damage when allies nearby

##### Tier 3C: Turtle Clan Wisdom
- **Prerequisite**: Clan Provider
- **Unlock**: Learn 20 recipes from Turtle Clan elders
- **Effects**:
  - Turtle Clan initiation
  - +30% crafting quality
  - Elders share ancient knowledge (lore unlocks)

##### Tier 3D: Deer Clan Grace
- **Prerequisite**: Clan Provider
- **Unlock**: Supply village through winter without hunting deer
- **Effects**:
  - Deer Clan initiation
  - +25% movement speed
  - Deer will not flee from you

##### Tier 4A: War Captain
- **Prerequisite**: Bear Clan OR Wolf Clan
- **Unlock**: Lead 3 successful raids
- **Effects**:
  - Command Tuscarora war parties (up to 15)
  - War drum buffs (+10% damage to allies)
  - Scalp trophies provide reputation

##### Tier 4B: Clan Mother
- **Prerequisite**: Turtle Clan OR Deer Clan
- **Unlock**: Ensure village prosperity for one year
- **Effects**:
  - Title: Clan Mother (highest female honor)
  - Can select and depose war chiefs
  - Village production +30%

##### Tier 5: Peace Chief
- **Prerequisite**: War Captain OR Clan Mother
- **Unlock**: Negotiate lasting peace with 2 hostile nations
- **Effects**:
  - Title: Peace Chief
  - Diplomatic immunity in all villages
  - Can adopt outsiders into Tuscarora

##### Tier 6: Keeper of the Fire (Ultimate)
- **Prerequisite**: Peace Chief + complete "Joining of Nations" questline
- **Effects**:
  - Title: Keeper of the Fire (supreme civil authority)
  - Tuscarora join the Iroquois Confederacy (major political shift)
  - All Iroquoian nations respect your authority
  - Unique ability: "Great Law" - end any conflict with rival factions
  - Establish new longhouse villages anywhere

#### Class Weapons

| Weapon | Type | Base Damage | Special Property |
|--------|------|-------------|------------------|
| Tuscarora War Club | Club | 45 | Ball-headed, high stun chance |
| Hemp-Backed Bow | Bow | 40 | Durable, +15% draw speed |
| Deer-Bone Knife | Dagger | 22 | Lightweight, +20% attack speed |
| Piedmont Tomahawk | Axe | 38 | Balanced for throwing |
| Turtle Shell Shield | Shield | 12 (bash) | High durability, blocks arrows |

---

### Cherokee Nation

*"Ani-Yunwiya - The Principal People"*

The powerful Cherokee control the interior mountains and valleys, maintaining a civilization of towns and agriculture rivaling European settlements. Their warriors are legendary, and their shamans command respect even among enemies.

#### Cultural Traits

| Trait | Effect | Description |
|-------|--------|-------------|
| **Mountain Born** | No terrain penalties in mountains/hills | Highland adaptation |
| **Seven Clan System** | Always have refuge in any Cherokee town | Kinship networks |
| **Ballplay Champions** | +15% all physical stats | Intense athletic tradition |
| **Fire Keepers** | Fire-based abilities +30% effective | Sacred flame mastery |
| **Didanawisgi Blessing** | Herbal remedies +50% effectiveness | Medicine tradition |

#### Signature Skill Tree: Cherokee Path

```
                        [CHEROKEE FRIEND]
                               |
              +----------------+----------------+
              |                                 |
      [RED WAR PATH]                    [WHITE PEACE PATH]
              |                                 |
    +---------+---------+              +--------+--------+
    |                   |              |                 |
[RAVEN            [WAR          [MEDICINE       [BELOVED
 MOCKER]           PRIEST]       WALKER]         ELDER]
    |                   |              |                 |
    +--------+----------+              +--------+--------+
             |                                  |
       [RED WAR CHIEF]                  [WHITE PEACE CHIEF]
             |                                  |
             +----------------+-----------------+
                              |
                    [FIRST BELOVED MAN]
                              |
                     [UKU OF THE CHEROKEE]
```

##### Tier 1: Cherokee Friend
- **Unlock**: Aid a Cherokee town or defeat their enemies
- **Effects**:
  - Welcome in Cherokee territory
  - Can participate in stickball games
  - Basic trade access

##### Tier 2A: Red War Path
- **Prerequisite**: Cherokee Friend
- **Unlock**: Join Cherokee war party
- **Effects**:
  - Red (war) paint unlocked
  - Cherokee war weapons available
  - Can initiate raids with approval

##### Tier 2B: White Peace Path
- **Prerequisite**: Cherokee Friend
- **Unlock**: Complete diplomatic mission for Cherokee
- **Effects**:
  - White (peace) garments unlocked
  - Access to council meetings
  - Healing knowledge shared

##### Tier 3A: Raven Mocker
- **Prerequisite**: Red War Path
- **Unlock**: Take 20 enemy scalps
- **Effects**:
  - Feared supernatural status
  - Killing enemies extends your lifespan (max health +5 per kill, caps at +50)
  - Enemies have reduced morale against you

##### Tier 3B: War Priest (Didanawisgi)
- **Prerequisite**: Red War Path
- **Unlock**: Fast for 7 days before battle, then win
- **Effects**:
  - Battle rituals grant allies +20% damage
  - Can curse enemies (-15% accuracy)
  - War divination reveals enemy positions

##### Tier 3C: Medicine Walker
- **Prerequisite**: White Peace Path
- **Unlock**: Cure 30 ailments with herbal remedies
- **Effects**:
  - All healing items +100% effective
  - Can cure diseases
  - Poison immunity

##### Tier 3D: Beloved Elder
- **Prerequisite**: White Peace Path
- **Unlock**: Reach age 50+ OR complete wisdom questline
- **Effects**:
  - Title: Beloved Elder (exempted from conflict)
  - Words carry weight in all councils
  - Can grant sanctuary to fugitives

##### Tier 4A: Red War Chief
- **Prerequisite**: Raven Mocker OR War Priest
- **Unlock**: Lead 5 victorious Cherokee campaigns
- **Effects**:
  - Title: Red War Chief
  - Command all warriors in your town
  - War decisions are yours alone during conflict

##### Tier 4B: White Peace Chief
- **Prerequisite**: Medicine Walker OR Beloved Elder
- **Unlock**: Maintain peace for one year
- **Effects**:
  - Title: White Peace Chief
  - Civil authority over town
  - Can veto war declarations

##### Tier 5: First Beloved Man
- **Prerequisite**: Red War Chief OR White Peace Chief
- **Unlock**: Unite Red and White factions under your leadership
- **Effects**:
  - Title: First Beloved Man
  - Authority recognized in all Cherokee towns
  - Can balance war and peace

##### Tier 6: Uku of the Cherokee (Ultimate)
- **Prerequisite**: First Beloved Man + complete "Eternal Flame" questline
- **Effects**:
  - Title: Uku (High Priest-Chief)
  - Keeper of the Eternal Flame (sacred site bonuses nation-wide)
  - All Cherokee answer your call
  - Ability: "Voice of Ancestors" - summon spirit warriors
  - Can establish new towns under Cherokee law
  - Enemies of Cherokee are enemies of the land itself (environmental penalties)

#### Class Weapons

| Weapon | Type | Base Damage | Special Property |
|--------|------|-------------|------------------|
| Cherokee Warbow | Bow | 48 | Double-curved, +25% power |
| Stickball Racket | Club | 30 | Swift attacks, can catch projectiles |
| War Hawk Tomahawk | Axe | 42 | Pipe tomahawk, +30% crit |
| River Cane Blowgun | Ranged | 15 | Silent, poison delivery |
| Flint War Knife | Dagger | 28 | +40% bleed damage |

---

### Catawba Nation

*"People of the River"*

Siouan-speaking warriors of the Carolina Piedmont, renowned as fierce fighters and skilled potters. They maintain strong trade relationships with colonists while fiercely defending their territory from all threats.

#### Cultural Traits

| Trait | Effect | Description |
|-------|--------|-------------|
| **River Warriors** | +20% combat effectiveness near water | Riverine adaptation |
| **Master Potters** | Pottery crafting +50%, trade value +30% | Legendary ceramics |
| **Slave Trade Knowledge** | Can capture and sell enemies | Economic pragmatism |
| **Flathead Identity** | Immune to intimidation effects | Distinctive pride |
| **Trading Post Savvy** | Best prices from all European factions | Trade experience |

#### Signature Skill Tree: Catawba Warrior's Way

```
                        [CATAWBA ACQUAINTANCE]
                               |
              +----------------+----------------+
              |                                 |
       [RIVER FIGHTER]                   [TRADE MASTER]
              |                                 |
    +---------+---------+              +--------+--------+
    |                   |              |                 |
[RAID            [WATER        [POTTERY        [MARKET
 LEADER]          AMBUSH]       ARTISAN]        MANIPULATOR]
    |                   |              |                 |
    +--------+----------+              +--------+--------+
             |                                  |
       [RIVER HAWK]                     [TRADE LORD]
             |                                  |
             +----------------+-----------------+
                              |
                    [ESAW CHIEF]
                              |
                [KING OF THE CATAWBA]
```

##### Tier 1: Catawba Acquaintance
- **Unlock**: Trade pottery with Catawba or fight common enemy
- **Effects**:
  - Safe passage through Catawba territory
  - Can purchase Catawba pottery (trade goods)
  - Basic warrior training available

##### Tier 2A: River Fighter
- **Prerequisite**: Catawba Acquaintance
- **Unlock**: Win combat in/near water
- **Effects**:
  - No combat penalties in water
  - Can hold breath 3x longer
  - Canoe combat unlocked

##### Tier 2B: Trade Master
- **Prerequisite**: Catawba Acquaintance
- **Unlock**: Complete 10 successful trades for Catawba
- **Effects**:
  - Access to Catawba trade networks
  - Pottery crafting basics
  - European goods available at discount

##### Tier 3A: Raid Leader
- **Prerequisite**: River Fighter
- **Unlock**: Lead successful raid on enemy camp
- **Effects**:
  - Raid party command (up to 8 warriors)
  - Captive taking mechanics unlocked
  - Bonus loot from raids

##### Tier 3B: Water Ambush
- **Prerequisite**: River Fighter
- **Unlock**: Kill 15 enemies using water ambush tactics
- **Effects**:
  - Can hide underwater indefinitely with reed
  - Surprise attacks from water +75% damage
  - Canoe silent movement

##### Tier 3C: Pottery Artisan
- **Prerequisite**: Trade Master
- **Unlock**: Craft 50 pottery items
- **Effects**:
  - Master pottery recipes
  - Pottery sells for 3x base value
  - Special containers extend food preservation

##### Tier 3D: Market Manipulator
- **Prerequisite**: Trade Master
- **Unlock**: Control trade in region for one season
- **Effects**:
  - Can set prices at Catawba markets
  - Trade route information (cargo movements)
  - Merchant contacts across factions

##### Tier 4A: River Hawk
- **Prerequisite**: Raid Leader OR Water Ambush
- **Unlock**: Dominate river territory for one year
- **Effects**:
  - Title: River Hawk (feared raider)
  - River travel speed +50%
  - All river-adjacent areas under surveillance

##### Tier 4B: Trade Lord
- **Prerequisite**: Pottery Artisan OR Market Manipulator
- **Unlock**: Establish trade posts in 3 foreign territories
- **Effects**:
  - Title: Trade Lord
  - Passive income from all trade routes
  - Can embargo enemy factions

##### Tier 5: Esaw Chief
- **Prerequisite**: River Hawk OR Trade Lord
- **Unlock**: Earn Blood Bond with Catawba
- **Effects**:
  - Title: Esaw Chief (town leader)
  - Control over Catawba town
  - Combined warrior and trade authority

##### Tier 6: King of the Catawba (Ultimate)
- **Prerequisite**: Esaw Chief + complete "River Empire" questline
- **Effects**:
  - Title: King of the Catawba
  - All Catawba towns recognize sovereignty
  - Unique weapon: River King's Mace
  - Ability: "Flood the Land" - call Catawba warriors from all settlements
  - Control all river trade in region
  - Europeans must pay tribute for river access

#### Class Weapons

| Weapon | Type | Base Damage | Special Property |
|--------|------|-------------|------------------|
| Catawba River Club | Club | 38 | Water-hardened, +20% near rivers |
| Piedmont War Bow | Bow | 42 | Compact, excellent for canoe use |
| Trading Hatchet | Axe | 32 | European-style, well-balanced |
| Potter's Blade | Knife | 18 | Sharp ceramic edge, +30% bleed |
| Flathead Shield | Shield | 10 (bash) | Decorated, intimidation bonus |

---

### Pamunkey Tribe

*"The Rising Corn People"*

The most prestigious tribe within the Powhatan Confederacy, from whose ranks the paramount chiefs traditionally come. They control the richest agricultural lands and hold the sacred knowledge of the confederacy's founding.

#### Cultural Traits

| Trait | Effect | Description |
|-------|--------|-------------|
| **Royal Blood** | +2 reputation with all Powhatan tribes | Paramount lineage |
| **Corn Lords** | +50% corn yield, never starve | Agricultural mastery |
| **Keeper of Secrets** | Access to confederacy lore and locations | Sacred knowledge |
| **Diplomatic Immunity** | Cannot be attacked in neutral villages | Royal protection |
| **Chosen People** | +15% all stats in Powhatan territory | Homeland blessing |

#### Signature Skill Tree: Pamunkey Heritage

```
                      [PAMUNKEY ACCEPTED]
                               |
              +----------------+----------------+
              |                                 |
     [ROYAL TRADITION]                  [SACRED KEEPER]
              |                                 |
    +---------+---------+              +--------+--------+
    |                   |              |                 |
[LINEAGE         [CORN          [TEMPLE        [HISTORY
 HEIR]            LORD]          GUARDIAN]      KEEPER]
    |                   |              |                 |
    +--------+----------+              +--------+--------+
             |                                  |
       [ROYAL BLOOD]                    [SACRED WISDOM]
             |                                  |
             +----------------+-----------------+
                              |
                  [PARAMOUNT HEIR]
                              |
               [BLOOD OF POWHATAN]
```

##### Tier 1: Pamunkey Accepted
- **Unlock**: High standing with Powhatan (+2) or marriage into tribe
- **Effects**:
  - Residence rights in Pamunkey villages
  - Royal court access
  - Sacred sites revealed

##### Tier 2A: Royal Tradition
- **Prerequisite**: Pamunkey Accepted
- **Unlock**: Participate in royal ceremonies
- **Effects**:
  - Formal diplomatic training
  - Royal regalia (stat bonuses when worn)
  - Speak with authority of Pamunkey

##### Tier 2B: Sacred Keeper
- **Prerequisite**: Pamunkey Accepted
- **Unlock**: Receive vision from tribal spirits
- **Effects**:
  - Temple entry permitted
  - Learn sacred calendar
  - Prophecy hints for quests

##### Tier 3A: Lineage Heir
- **Prerequisite**: Royal Tradition
- **Unlock**: Complete succession trials
- **Effects**:
  - Recognized as potential paramount
  - Royal guards assigned (2 elite warriors)
  - Tribute collection rights

##### Tier 3B: Corn Lord
- **Prerequisite**: Royal Tradition
- **Unlock**: Ensure food surplus for 3 seasons
- **Effects**:
  - Control food distribution
  - Loyalty from fed tribes
  - Famine immunity for your settlements

##### Tier 3C: Temple Guardian
- **Prerequisite**: Sacred Keeper
- **Unlock**: Defend temple from desecration
- **Effects**:
  - Sacred weapons access
  - Spirit allies in combat (near temples)
  - Temple regeneration (+5 HP/s)

##### Tier 3D: History Keeper
- **Prerequisite**: Sacred Keeper
- **Unlock**: Learn complete confederacy history from elders
- **Effects**:
  - All confederacy locations known
  - Ancient artifact locations revealed
  - Oral history provides quest hints

##### Tier 4A: Royal Blood
- **Prerequisite**: Lineage Heir OR Corn Lord
- **Unlock**: Recognized by dying paramount as heir
- **Effects**:
  - Title: Royal Blood
  - All Pamunkey follow your commands
  - Royal tribute (significant passive income)

##### Tier 4B: Sacred Wisdom
- **Prerequisite**: Temple Guardian OR History Keeper
- **Unlock**: Complete all sacred rituals
- **Effects**:
  - Title: Sacred Wisdom
  - Can perform ceremonies
  - Spirit communication at will

##### Tier 5: Paramount Heir
- **Prerequisite**: Royal Blood OR Sacred Wisdom
- **Unlock**: Current paramount endorses succession
- **Effects**:
  - Title: Paramount Heir
  - Speak with paramount authority when he's absent
  - Command any confederacy warrior

##### Tier 6: Blood of Powhatan (Ultimate)
- **Prerequisite**: Paramount Heir + complete "Succession" questline (or current paramount dies)
- **Effects**:
  - Title: Blood of Powhatan (new Mamanatowick)
  - Supreme authority over entire confederacy
  - Unique crown: Feathered Crown of Powhatan
  - Ability: "Unite the People" - all confederacy tribes rally
  - Can wage war against Europeans as unified nation
  - Found new paramount dynasty

#### Class Weapons

| Weapon | Type | Base Damage | Special Property |
|--------|------|-------------|------------------|
| Paramount's Mace | Club | 50 | Symbol of authority, +30% vs enemies of confederacy |
| Sacred Bow | Bow | 48 | Blessed, +20% crit |
| Corn Knife | Dagger | 25 | Harvesting tool, +50% against corn spirits |
| Royal Tomahawk | Axe | 40 | Decorated, cannot be disarmed |
| Pamunkey Great Shield | Shield | 15 (bash) | Royal emblems, morale to allies |

---

## Reputation & Standing System

### Earning Reputation

| Action | Reputation Change |
|--------|------------------|
| Complete faction quest | +50 to +200 |
| Gift valuable items | +5 to +50 |
| Aid in combat | +20 per enemy killed |
| Trade fairly | +5 per transaction |
| Rescue faction member | +100 |
| Defend settlement | +150 |
| Betray faction secrets | -500 |
| Kill faction member | -200 |
| Steal from faction | -100 |
| Aid faction enemy | -50 |

### Standing Thresholds

| Standing | Reputation Range | Title |
|----------|-----------------|-------|
| War (-3) | -1000 and below | Enemy of the People |
| Hostile (-2) | -999 to -500 | Unwelcome |
| Suspicious (-1) | -499 to -100 | Outsider |
| Neutral (0) | -99 to +99 | Stranger |
| Friendly (+1) | +100 to +499 | Friend |
| Allied (+2) | +500 to +999 | Brother/Sister |
| Blood Bond (+3) | +1000 and above | Family |

### Reputation Decay

- Reputation decays toward neutral at rate of -1/day if no interactions
- Active relationships (trade, quests) prevent decay
- Blood Bond relationships never decay

---

## Trade Networks & Economics

### Faction Trade Goods

| Faction | Exports | Imports | Currency Preference |
|---------|---------|---------|---------------------|
| Spanish | Steel weapons, armor, gunpowder | Gold, silver, furs | Gold coins |
| French | European goods, firearms, alcohol | Furs, pelts, guides | Furs as currency |
| English | Tools, textiles, manufactured goods | Food, furs, labor | Barter/coins |
| Aztec | Obsidian weapons, medicine, feathers | Metal tools, allies | Cacao beans |
| Powhatan | Corn, fish, deerskins, tobacco | Metal goods, beads | Wampum |
| Tuscarora | Hemp rope, textiles, corn | Metal tools, beads | Wampum |
| Cherokee | Herbs, deerskins, minerals | Trade goods, salt | Wampum/barter |
| Catawba | Pottery, slaves, furs | Metal goods, cloth | Mixed |
| Pamunkey | Corn surplus, sacred items | Rare goods, tribute | Wampum |

### Trade Route Values

| Route | Goods | Base Value | Risk Level |
|-------|-------|------------|------------|
| French-Powhatan | Furs for goods | High | Low |
| Spanish-Cherokee | Weapons for guides | Medium | Medium |
| English-Catawba | Tools for pottery | Medium | Low |
| Aztec-Tuscarora | Medicine for hemp | High | High |
| Inter-tribal | Food distribution | Variable | Low |

---

## Rust Data Structures

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    Spanish,
    French,
    English,
    Aztec,
    Powhatan,
    Tuscarora,
    Cherokee,
    Catawba,
    Pamunkey,
    Independent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Standing {
    War,        // -3
    Hostile,    // -2
    Suspicious, // -1
    Neutral,    // 0
    Friendly,   // +1
    Allied,     // +2
    BloodBond,  // +3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionReputation {
    pub faction: Faction,
    pub reputation: i32,
    pub standing: Standing,
    pub last_interaction: f64, // game time
    pub completed_quests: Vec<String>,
    pub known_members: Vec<EntityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerFactionData {
    pub primary_faction: Option<Faction>,
    pub reputations: HashMap<Faction, FactionReputation>,
    pub unlocked_skills: HashMap<Faction, Vec<FactionSkillId>>,
    pub active_titles: Vec<FactionTitle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactionSkillId {
    // Spanish
    ConquistadorInitiate,
    SwordAndBuckler,
    ArquebusMastery,
    TercioFormation,
    DuelistsGrace,
    VolleyFire,
    MarkmansPatience,
    SteelTempest,
    ThunderOfGod,
    GoldAndGlory,
    ElConquistador,

    // French
    ApprentiVoyageur,
    FurTrade,
    ForestWisdom,
    MasterTrapper,
    NegotiatorsTongue,
    SilentShadow,
    HerbalistsCraft,
    TradeEmpire,
    OneWithLand,
    SpiritBridge,
    GrandVoyageur,

    // English
    RoanokeSettler,
    Fortification,
    FrontierSurvival,
    MasterBuilder,
    MilitiaCaptain,
    WildernessScout,
    ColonialFarmer,
    ColonialLeader,
    FrontierMaster,
    NewWorldGovernor,
    LordOfRoanoke,

    // Aztec
    MacehuallinInitiate,
    JaguarWarrior,
    EagleWarrior,
    OcelotlFury,
    ShadowStalker,
    CuauhtliStrike,
    SolarAscension,
    JaguarKnight,
    EagleKnight,
    Cuachicqueh,
    ChampionOfTheSun,

    // Powhatan
    PowhatanNewcomer,
    HuntersPath,
    DiplomatsPath,
    DeerStalker,
    RiverKeeper,
    PeaceWeaver,
    WarChief,
    SpiritWalker,
    ConfederateLord,
    Werowance,
    Mamanatowick,

    // Tuscarora
    FriendOfTuscarora,
    ClanWarrior,
    ClanProvider,
    BearClanFury,
    WolfClanPack,
    TurtleClanWisdom,
    DeerClanGrace,
    TuscaroraWarCaptain,
    ClanMother,
    PeaceChief,
    KeeperOfTheFire,

    // Cherokee
    CherokeeFriend,
    RedWarPath,
    WhitePeacePath,
    RavenMocker,
    CherokeeWarPriest,
    MedicineWalker,
    BelovedElder,
    RedWarChief,
    WhitePeaceChief,
    FirstBelovedMan,
    UkuOfCherokee,

    // Catawba
    CatawbaAcquaintance,
    RiverFighter,
    CatawbaTradeMaster,
    RaidLeader,
    WaterAmbush,
    PotteryArtisan,
    MarketManipulator,
    RiverHawk,
    TradeLord,
    EsawChief,
    KingOfCatawba,

    // Pamunkey
    PamunkeyAccepted,
    RoyalTradition,
    SacredKeeper,
    LineageHeir,
    CornLord,
    TempleGuardian,
    HistoryKeeper,
    RoyalBlood,
    SacredWisdom,
    ParamountHeir,
    BloodOfPowhatan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionTrait {
    pub name: String,
    pub description: String,
    pub effect: TraitEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraitEffect {
    DamageModifier { weapon_type: Option<WeaponType>, multiplier: f32 },
    StatBonus { stat: Stat, amount: f32 },
    DetectionModifier { multiplier: f32 },
    ResourceYield { resource: ResourceType, multiplier: f32 },
    ReputationModifier { factions: Vec<Faction>, amount: i32 },
    MovementModifier { terrain: Option<TerrainType>, multiplier: f32 },
    CraftingBonus { category: CraftingCategory, quality_bonus: f32 },
    HealthRegen { amount: f32, condition: Option<RegenCondition> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionWeapon {
    pub id: String,
    pub name: String,
    pub faction: Faction,
    pub weapon_type: WeaponType,
    pub base_damage: u32,
    pub special_properties: Vec<WeaponProperty>,
    pub unlock_requirement: FactionSkillId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeaponProperty {
    BleedChance(f32),
    ArmorPenetration(f32),
    StunDuration(f32),
    BonusVsType { target_type: TargetType, multiplier: f32 },
    ReturnOnMiss,
    SilentAttack,
    DrawSpeedBonus(f32),
    ParryWindowBonus(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionAbility {
    pub id: String,
    pub name: String,
    pub faction: Faction,
    pub unlock_skill: FactionSkillId,
    pub cooldown_seconds: f32,
    pub duration_seconds: Option<f32>,
    pub effect: AbilityEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbilityEffect {
    StatBuff { stats: Vec<(Stat, f32)>, duration: f32 },
    DamageBuff { multiplier: f32, duration: f32 },
    DefenseBuff { reduction: f32, duration: f32 },
    AreaEffect { radius: f32, damage: u32, effect_type: EffectType },
    Stealth { duration: f32 },
    Summon { entity_type: String, count: u32, duration: f32 },
    Reveal { radius: f32, target_type: TargetType },
    Teleport { range: f32 },
    Heal { amount: u32, target: HealTarget },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionRelationship {
    pub faction_a: Faction,
    pub faction_b: Faction,
    pub base_standing: i8, // -3 to +3
    pub modifiable: bool,
    pub historical_context: String,
}

impl FactionRelationship {
    pub fn get_default_relationships() -> Vec<Self> {
        vec![
            // Spanish relationships
            Self { faction_a: Faction::Spanish, faction_b: Faction::French, base_standing: -2, modifiable: true, historical_context: "European rivals for New World dominance".to_string() },
            Self { faction_a: Faction::Spanish, faction_b: Faction::English, base_standing: -3, modifiable: false, historical_context: "Protestant-Catholic warfare, privateering".to_string() },
            Self { faction_a: Faction::Spanish, faction_b: Faction::Aztec, base_standing: -3, modifiable: false, historical_context: "Conquest of Mexico, blood feud".to_string() },
            // ... additional relationships
        ]
    }
}
```

---

## Implementation Priority

### Phase 1: Core Framework
- [ ] Faction enum and basic data structures
- [ ] Reputation tracking system
- [ ] Standing calculation and thresholds
- [ ] Basic NPC faction assignment

### Phase 2: European Factions
- [ ] Spanish Conquistadors complete implementation
- [ ] French Coureurs des Bois complete implementation
- [ ] English Colonists skill tree (expand existing)
- [ ] Inter-European relationships and conflicts

### Phase 3: Native Nations
- [ ] Powhatan Confederacy (expand existing NPC villages)
- [ ] Pamunkey special status within Powhatan
- [ ] Tuscarora Nation implementation
- [ ] Cherokee Nation implementation
- [ ] Catawba Nation implementation

### Phase 4: Aztec & Advanced Features
- [ ] Aztec Remnants faction
- [ ] Cross-faction skill learning (Spirit Bridge, etc.)
- [ ] Trade network economics
- [ ] Faction warfare system

### Phase 5: Ultimate Skills & Endgame
- [ ] Ultimate skill implementations (El Conquistador, etc.)
- [ ] Faction leadership positions
- [ ] Dynamic world state based on faction dominance
- [ ] Multi-faction alliance/war systems

---

## Balance Considerations

### Skill Point Distribution
- Tier 1 skills: 1 point
- Tier 2 skills: 2 points
- Tier 3 skills: 3 points
- Tier 4 skills: 5 points
- Tier 5 skills: 8 points
- Tier 6 skills: 15 points

### Reputation Requirements (approximate)
- Tier 1-2: Neutral or better
- Tier 3: Friendly (+1) or better
- Tier 4: Allied (+2) or better
- Tier 5: Allied (+2) with specific quest completion
- Tier 6: Blood Bond (+3) with major questline completion

### Multi-Faction Balance
- Players cannot reach Blood Bond with mutually hostile factions
- Hostile factions detect and respond to enemy faction equipment
- Some skills are mutually exclusive (e.g., Spanish vs Aztec combat trees)
- Neutral factions allow multi-faction progression

---

*Document End - Faction System Specification v1.0*
