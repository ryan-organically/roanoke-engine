# City Building Perk Tree Specification

## Overview

The City Building system allows players to establish and grow settlements from humble frontier camps to thriving colonial cities. Progression is tracked through a branching perk tree with five specialization paths, unlocking new building types, production bonuses, and settlement capabilities.

## Design Philosophy

Set in the colonial Roanoke period, settlement building reflects the challenges of establishing civilization in the New World:

- **Frontier Survival**: Early tiers focus on basic shelter and sustenance
- **Community Formation**: Mid tiers introduce specialization and trade
- **Colonial Ambition**: Late tiers unlock advanced infrastructure and defenses
- **Legacy Building**: Legendary tier creates lasting monuments

The system emphasizes meaningful choices - players cannot master all branches, encouraging specialization and trade between settlements.

## Perk Tree Structure

```
                            [Founder's Vision]
                                   │
                    ┌──────────────┼──────────────┐
                    │              │              │
              [Shelter I]    [Stockade I]   [Storehouse I]
                    │              │              │
        ┌───────────┴───────────┐  │  ┌──────────┴──────────┐
        │                       │  │  │                     │
   INFRASTRUCTURE          DEFENSE │ COMMERCE           CULTURE
        │                       │  │  │                     │
   [Housing II]           [Walls II] [Market II]      [Chapel II]
        │                       │  │  │                     │
   [Roads III]           [Tower III] [Warehouse III]  [School III]
        │                       │  │  │                     │
   [Aqueduct IV]        [Garrison IV][Port IV]       [Library IV]
        │                       │  │  │                     │
   [Mill V]              [Arsenal V] [Bank V]        [Theater V]
        │                       │  │  │                     │
        └───────────┬───────────┘  │  └──────────┬──────────┘
                    │              │              │
              [Town Hall VI]  [Citadel VI]  [Guild Hall VI]
                    │              │              │
              [Factory VII] [Fortress VII] [Exchange VII]
                    │              │              │
                    └──────────────┼──────────────┘
                                   │
                         [Governor's Manor VIII]
                                   │
                          [Grand Cathedral IX]
                                   │
                        ══[ROANOKE MONUMENT]══
                           (Legendary Tier)
```

## Branch Descriptions

### Infrastructure Branch
Focus on housing capacity, resource efficiency, and utility buildings.
- **Theme**: Supporting population growth and production
- **Key Bonuses**: Population cap, build speed, resource efficiency

### Defense Branch
Focus on walls, towers, garrisons, and military structures.
- **Theme**: Protecting the settlement from threats
- **Key Bonuses**: Defense rating, garrison capacity, attack resistance

### Commerce Branch
Focus on trade, storage, and economic buildings.
- **Theme**: Generating wealth and enabling trade
- **Key Bonuses**: Trade multipliers, storage capacity, income generation

### Culture Branch
Focus on education, religion, and civic buildings.
- **Theme**: Happiness, loyalty, and technological advancement
- **Key Bonuses**: Happiness, research speed, reputation gains

### Research Branch (Unlocks at Tier IV)
Focus on advanced technology and specialized production.
- **Theme**: Unlocking advanced recipes and automation
- **Key Bonuses**: Craft quality, production speed, unique recipes

## Tier Breakdown

### Tier 1: Frontier Camp
**Point Requirement**: 0 (Starting)

| Perk | Branch | Effect |
|------|--------|--------|
| Founder's Vision | Core | Establish settlement claim (100m radius) |
| Lean-To | Infrastructure | Basic shelter for 2 settlers |
| Campfire | Infrastructure | Cooking station, warmth source |
| Tool Cache | Commerce | Store 50 items |

**Buildings Unlocked**:
- Lean-To (wood x10)
- Campfire (stone x5, wood x5)
- Tool Cache (wood x20)

---

### Tier 2: Homestead
**Point Requirement**: 50

| Perk | Branch | Effect |
|------|--------|--------|
| Timber Frame | Infrastructure | Unlock Cabin (+4 population cap) |
| Palisade Stakes | Defense | Basic perimeter (+10 defense) |
| Root Cellar | Commerce | Food storage (spoilage -50%) |
| Shrine | Culture | Prayer site (+5% reputation gain) |

**Buildings Unlocked**:
- Cabin (wood x40, stone x10)
- Palisade Section (wood x15 per 5m)
- Root Cellar (wood x20, dig labor)
- Shrine (wood x10, stone x5)

**Unlock Conditions**:
- Population >= 4
- Days survived >= 7

---

### Tier 3: Village
**Point Requirement**: 125

| Perk | Branch | Effect |
|------|--------|--------|
| Longhouse | Infrastructure | Large dwelling (+8 population) |
| Watchtower | Defense | Vision range +50m, early warning |
| Trading Post | Commerce | Enable NPC trade caravans |
| Schoolhouse | Culture | Child education, +10% research |

**Buildings Unlocked**:
- Longhouse (wood x80, thatch x40)
- Watchtower (wood x60, stone x20)
- Trading Post (wood x50, cloth x10)
- Schoolhouse (wood x40, paper x20)

**Unlock Conditions**:
- Population >= 10
- Completed trade with any faction

---

### Tier 4: Town
**Point Requirement**: 250

| Perk | Branch | Effect |
|------|--------|--------|
| Stone Masonry | Infrastructure | Unlock stone buildings |
| Curtain Wall | Defense | Stone walls (+30 defense per section) |
| Marketplace | Commerce | Daily market, +25% trade prices |
| Chapel | Culture | Worship services, +15% happiness |
| Workshop | Research | **NEW BRANCH** - Unlock crafting stations |

**Buildings Unlocked**:
- Stone House (stone x60, wood x20, +6 population)
- Curtain Wall Section (stone x40 per 5m)
- Marketplace (stone x30, wood x40, cloth x20)
- Chapel (stone x50, wood x30, glass x10)
- Workshop (stone x20, wood x40, iron x10)

**Unlock Conditions**:
- Population >= 20
- Wealth >= 500
- Faction standing >= Friendly with any faction

---

### Tier 5: Prosperous Town
**Point Requirement**: 450

| Perk | Branch | Effect |
|------|--------|--------|
| Aqueduct | Infrastructure | Water supply, +20% crop yield |
| Gatehouse | Defense | Fortified entrance (+50 defense) |
| Warehouse | Commerce | Bulk storage (500 items) |
| Library | Culture | Book collection, +25% research |
| Smithy | Research | Metal crafting, weapon/tool quality +1 |

**Buildings Unlocked**:
- Aqueduct (stone x100, lead pipe x20)
- Gatehouse (stone x80, iron x30)
- Warehouse (wood x100, stone x40)
- Library (stone x60, wood x40, paper x50)
- Smithy (stone x40, iron x50, coal x20)

**Unlock Conditions**:
- Population >= 35
- Defense rating >= 50
- Completed 5 trade transactions

---

### Tier 6: Regional Center
**Point Requirement**: 700

| Perk | Branch | Effect |
|------|--------|--------|
| Town Hall | Infrastructure | Governance center, unlock edicts |
| Barracks | Defense | Train militia (+10 garrison cap) |
| Bank | Commerce | Wealth storage, interest (+2%/week) |
| Theater | Culture | Entertainment, +25% happiness |
| Foundry | Research | Advanced metalwork, steel production |

**Buildings Unlocked**:
- Town Hall (stone x120, wood x60, glass x20)
- Barracks (stone x80, wood x60, iron x40)
- Bank (stone x100, iron x60, gold x10)
- Theater (wood x100, stone x60, cloth x40)
- Foundry (stone x80, iron x100, coal x50)

**Edicts Unlocked** (via Town Hall):
- Tax Rate adjustment
- Curfew (defense +20%, happiness -10%)
- Festival (+30% happiness for 3 days, costs gold)
- Conscription (emergency militia)

**Unlock Conditions**:
- Population >= 50
- Wealth >= 2000
- All Tier 4 buildings constructed

---

### Tier 7: City
**Point Requirement**: 1000

| Perk | Branch | Effect |
|------|--------|--------|
| Tenement Block | Infrastructure | Dense housing (+20 population) |
| Fortress Walls | Defense | Advanced fortification (+100 defense) |
| Exchange | Commerce | Regional trade hub, rare goods access |
| University | Culture | Advanced research, +50% speed |
| Factory | Research | Mass production, -30% craft time |

**Buildings Unlocked**:
- Tenement Block (stone x150, wood x80, glass x30)
- Fortress Wall Section (stone x100, iron x20 per 5m)
- Exchange (stone x120, wood x80, gold x30)
- University (stone x150, wood x100, paper x100, glass x40)
- Factory (stone x100, iron x150, coal x100)

**Unlock Conditions**:
- Population >= 75
- Wealth >= 5000
- Faction standing >= Allied with any faction
- Research projects completed >= 5

---

### Tier 8: Colonial Capital
**Point Requirement**: 1400

| Perk | Branch | Effect |
|------|--------|--------|
| Governor's Manor | Core | Seat of power, all bonuses +10% |
| Citadel | Defense | Ultimate fortification (+200 defense) |
| Mint | Commerce | Coin production, wealth +5%/week |
| Grand Cathedral | Culture | Religious center, +50% reputation |
| Arsenal | Research | Advanced weapons, unique recipes |

**Buildings Unlocked**:
- Governor's Manor (stone x200, wood x100, gold x50, glass x50)
- Citadel (stone x300, iron x100)
- Mint (stone x100, gold x100, iron x50)
- Grand Cathedral (stone x250, wood x100, glass x100, gold x30)
- Arsenal (stone x150, iron x200, gunpowder x50)

**Governor Powers** (via Manor):
- Appoint advisors (specialist NPCs)
- Declare war/peace with factions
- Commission expeditions
- Grant land charters (satellite settlements)

**Unlock Conditions**:
- Population >= 100
- Wealth >= 10000
- Defense rating >= 200
- All faction standings >= Neutral

---

### Tier 9: Metropolis
**Point Requirement**: 2000

| Perk | Branch | Effect |
|------|--------|--------|
| District Planning | Infrastructure | Unlock specialized districts |
| Harbor Fort | Defense | Naval defense, cannon emplacements |
| Trade Company HQ | Commerce | Monopoly rights, +100% specific trade |
| Opera House | Culture | Cultural wonder, happiness cap +50 |
| Academy of Sciences | Research | Breakthrough research, unique tech |

**Districts Unlocked**:
- **Industrial District**: +50% production, -20% happiness in area
- **Merchant Quarter**: +30% trade, attracts wealthy NPCs
- **Noble Quarter**: +40% happiness, high upkeep
- **Docklands**: Ship construction, overseas trade
- **Scholar's Row**: +75% research, attracts academics

**Unlock Conditions**:
- Population >= 150
- Wealth >= 25000
- Buildings constructed >= 50
- Unique achievements >= 3

---

### Tier 10: Legendary - Roanoke Monument
**Point Requirement**: 3000

| Perk | Effect |
|------|--------|
| Roanoke Monument | Construct the legendary monument to the lost colony |
| Eternal Settlement | Settlement cannot be destroyed, persists across saves |
| Founder's Legacy | All future settlements start at Tier 2 |
| Colony Beacon | Attracts legendary NPCs and unique events |

**Monument Construction**:
- Requires ALL Tier 8 buildings
- Resources: Stone x1000, Gold x500, Iron x300, Glass x200
- Construction time: 30 in-game days
- Triggers unique "Roanoke Mystery" questline

**Companion Unlock**: **The Keeper**
- Spectral NPC representing original Roanoke colonist
- Provides lore, hints at hidden locations
- Unlocks "CROATOAN" secret areas

---

## Point Acquisition

| Action | Points | Notes |
|--------|--------|-------|
| Construct building | 5-50 | Based on building tier |
| Population milestone | 25 | Every 10 new settlers |
| Survive attack | 30 | Successfully defend settlement |
| Complete trade | 5 | Per transaction |
| Research completion | 20 | Per project |
| Faction alliance | 50 | Reach Allied standing |
| Unique event | 10-100 | Story events, discoveries |
| Daily survival | 1 | Per day with positive happiness |

---

## Data Structures

```rust
use std::collections::HashSet;
use serde::{Deserialize, Serialize};

/// Main perk tree container
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CityBuildingSkills {
    /// Set of unlocked perks
    unlocked: HashSet<CityPerk>,

    /// Points earned per branch [Infrastructure, Defense, Commerce, Culture, Research]
    points_per_branch: [u32; 5],

    /// Total points earned across all branches
    total_points_earned: u32,

    /// Current settlement tier (1-10)
    settlement_tier: u8,

    /// Active edicts
    active_edicts: Vec<Edict>,
}

/// All available perks
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CityPerk {
    // Tier 1 - Frontier Camp
    FoundersVision,
    LeanTo,
    Campfire,
    ToolCache,

    // Tier 2 - Homestead
    TimberFrame,
    PalisadeStakes,
    RootCellar,
    Shrine,

    // Tier 3 - Village
    Longhouse,
    Watchtower,
    TradingPost,
    Schoolhouse,

    // Tier 4 - Town
    StoneMasonry,
    CurtainWall,
    Marketplace,
    Chapel,
    Workshop,

    // Tier 5 - Prosperous Town
    Aqueduct,
    Gatehouse,
    Warehouse,
    Library,
    Smithy,

    // Tier 6 - Regional Center
    TownHall,
    Barracks,
    Bank,
    Theater,
    Foundry,

    // Tier 7 - City
    TenementBlock,
    FortressWalls,
    Exchange,
    University,
    Factory,

    // Tier 8 - Colonial Capital
    GovernorsManor,
    Citadel,
    Mint,
    GrandCathedral,
    Arsenal,

    // Tier 9 - Metropolis
    DistrictPlanning,
    HarborFort,
    TradeCompanyHQ,
    OperaHouse,
    AcademyOfSciences,

    // Tier 10 - Legendary
    RoanokeMonument,
    EternalSettlement,
    FoundersLegacy,
    ColonyBeacon,
}

/// Perk branch categories
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CityBranch {
    Infrastructure = 0,
    Defense = 1,
    Commerce = 2,
    Culture = 3,
    Research = 4,
}

/// Perk definition with requirements and effects
#[derive(Clone, Debug)]
pub struct CityPerkDef {
    pub id: CityPerk,
    pub name: &'static str,
    pub description: &'static str,
    pub branch: CityBranch,
    pub tier: u8,
    pub point_cost: u32,
    pub prerequisites: &'static [CityPerk],
    pub unlock_condition: BuildingUnlockCondition,
    pub effects: &'static [BuildingEffect],
}

/// Conditions required to unlock perks/buildings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BuildingUnlockCondition {
    None,
    PopulationCount { min: u32 },
    WealthAmount { min: u32 },
    DaysSurvived { min: u32 },
    DefenseRating { min: u32 },
    HappinessLevel { min: u32 },
    FactionStanding { faction: Faction, min_standing: Standing },
    BuildingConstructed { building: BuildingType },
    BuildingsConstructedCount { min: u32 },
    TradeCount { min: u32 },
    ResearchCount { min: u32 },
    AchievementUnlocked { achievement: &'static str },
    AllOf(Vec<BuildingUnlockCondition>),
    AnyOf(Vec<BuildingUnlockCondition>),
}

/// Effects granted by perks and buildings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BuildingEffect {
    PopulationCapacity { increase: u32 },
    DefenseRating { increase: u32 },
    StorageCapacity { increase: u32 },
    HappinessBonus { percent: i32 },
    TradeMultiplier { percent: i32 },
    ResearchSpeed { percent: i32 },
    ProductionSpeed { percent: i32 },
    ResourceEfficiency { resource: ResourceType, percent: i32 },
    WealthGeneration { per_week: u32 },
    ReputationGain { percent: i32 },
    VisionRange { increase: f32 },
    GarrisonCapacity { increase: u32 },
    UnlockBuilding { building: BuildingType },
    UnlockEdict { edict: EdictType },
    UnlockDistrict { district: DistrictType },
    CraftQualityBonus { bonus: i32 },
    SpoilageReduction { percent: i32 },
}

/// Building types constructable by the player
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    // Tier 1
    LeanTo,
    Campfire,
    ToolCache,

    // Tier 2
    Cabin,
    PalisadeSection,
    RootCellar,
    Shrine,

    // Tier 3
    Longhouse,
    Watchtower,
    TradingPost,
    Schoolhouse,

    // Tier 4
    StoneHouse,
    CurtainWallSection,
    Marketplace,
    Chapel,
    Workshop,

    // Tier 5
    Aqueduct,
    Gatehouse,
    Warehouse,
    Library,
    Smithy,

    // Tier 6
    TownHall,
    Barracks,
    Bank,
    Theater,
    Foundry,

    // Tier 7
    TenementBlock,
    FortressWallSection,
    Exchange,
    University,
    Factory,

    // Tier 8
    GovernorsManor,
    Citadel,
    Mint,
    GrandCathedral,
    Arsenal,

    // Tier 9+
    HarborFort,
    TradeCompanyHQ,
    OperaHouse,
    AcademyOfSciences,
    RoanokeMonument,
}

/// Edicts available through Town Hall
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdictType {
    TaxRateLow,
    TaxRateMedium,
    TaxRateHigh,
    Curfew,
    Festival,
    Conscription,
    Rationing,
    OpenBorders,
    ClosedBorders,
    MarketDay,
}

/// District types for Tier 9
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DistrictType {
    Industrial,
    MerchantQuarter,
    NobleQuarter,
    Docklands,
    ScholarsRow,
}

/// Player settlement state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerSettlement {
    pub id: SettlementId,
    pub name: String,
    pub center: Vec3,
    pub radius: f32,
    pub founded_day: u32,

    /// All constructed buildings
    pub buildings: Vec<ConstructedBuilding>,

    /// Current population
    pub population: u32,
    pub population_cap: u32,

    /// Resources and wealth
    pub wealth: u32,
    pub stored_resources: HashMap<ResourceType, u32>,

    /// Settlement stats
    pub defense_rating: u32,
    pub happiness: i32,
    pub research_points: u32,

    /// Active effects from buildings/edicts
    pub active_effects: Vec<BuildingEffect>,

    /// Districts (Tier 9+)
    pub districts: Vec<District>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstructedBuilding {
    pub building_type: BuildingType,
    pub position: Vec3,
    pub rotation: f32,
    pub health: f32,
    pub constructed_day: u32,
    pub upgrade_level: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct District {
    pub district_type: DistrictType,
    pub center: Vec3,
    pub radius: f32,
    pub buildings: Vec<ConstructedBuilding>,
}

pub type SettlementId = u32;
```

## Implementation Methods

```rust
impl CityBuildingSkills {
    pub fn new() -> Self {
        let mut skills = Self::default();
        // Start with Founder's Vision unlocked
        skills.unlocked.insert(CityPerk::FoundersVision);
        skills.settlement_tier = 1;
        skills
    }

    /// Check if a perk can be unlocked
    pub fn can_unlock(&self, perk: CityPerk, settlement: &PlayerSettlement) -> bool {
        let def = perk.definition();

        // Already unlocked
        if self.unlocked.contains(&perk) {
            return false;
        }

        // Check tier points
        if self.total_points_earned < Self::points_for_tier(def.tier) {
            return false;
        }

        // Check prerequisites
        for prereq in def.prerequisites {
            if !self.unlocked.contains(prereq) {
                return false;
            }
        }

        // Check unlock conditions
        def.unlock_condition.is_satisfied(settlement)
    }

    /// Unlock a perk
    pub fn unlock(&mut self, perk: CityPerk) -> Result<(), UnlockError> {
        let def = perk.definition();
        self.unlocked.insert(perk);
        self.points_per_branch[def.branch as usize] += def.point_cost;

        // Check for tier advancement
        self.update_tier();
        Ok(())
    }

    /// Award points for an action
    pub fn award_points(&mut self, branch: CityBranch, amount: u32) {
        self.points_per_branch[branch as usize] += amount;
        self.total_points_earned += amount;
        self.update_tier();
    }

    fn update_tier(&mut self) {
        for tier in (1..=10).rev() {
            if self.total_points_earned >= Self::points_for_tier(tier) {
                self.settlement_tier = tier;
                break;
            }
        }
    }

    pub fn points_for_tier(tier: u8) -> u32 {
        match tier {
            1 => 0,
            2 => 50,
            3 => 125,
            4 => 250,
            5 => 450,
            6 => 700,
            7 => 1000,
            8 => 1400,
            9 => 2000,
            10 => 3000,
            _ => u32::MAX,
        }
    }

    pub fn unlocked_buildings(&self) -> Vec<BuildingType> {
        self.unlocked
            .iter()
            .flat_map(|perk| {
                perk.definition()
                    .effects
                    .iter()
                    .filter_map(|effect| {
                        if let BuildingEffect::UnlockBuilding { building } = effect {
                            Some(*building)
                        } else {
                            None
                        }
                    })
            })
            .collect()
    }
}

impl BuildingUnlockCondition {
    pub fn is_satisfied(&self, settlement: &PlayerSettlement) -> bool {
        match self {
            Self::None => true,
            Self::PopulationCount { min } => settlement.population >= *min,
            Self::WealthAmount { min } => settlement.wealth >= *min,
            Self::DefenseRating { min } => settlement.defense_rating >= *min,
            Self::HappinessLevel { min } => settlement.happiness >= *min as i32,
            Self::BuildingsConstructedCount { min } => {
                settlement.buildings.len() >= *min as usize
            }
            Self::AllOf(conditions) => conditions.iter().all(|c| c.is_satisfied(settlement)),
            Self::AnyOf(conditions) => conditions.iter().any(|c| c.is_satisfied(settlement)),
            // ... other conditions
            _ => true,
        }
    }
}
```

## Integration with Existing Systems

### PlayerProgression (player_state.rs)

```rust
pub struct PlayerProgression {
    pub hunting: HuntingSkills,
    pub archaeology: ArchaeologySkills,
    pub city_building: CityBuildingSkills,  // ADD
    // ...
}
```

### Building Generation (croatoan_procgen/building.rs)

Extend `BuildingRecipe` to support player buildings:

```rust
impl BuildingType {
    pub fn to_recipe(&self, upgrade_level: u8) -> BuildingRecipe {
        let base = match self {
            Self::Cabin => BuildingRecipe {
                style: ArchStyle::Rustic,
                floors: 1,
                width: 6.0,
                depth: 8.0,
                ..Default::default()
            },
            Self::StoneHouse => BuildingRecipe {
                style: ArchStyle::Colonial,
                floors: 2,
                width: 8.0,
                depth: 10.0,
                ..Default::default()
            },
            // ...
        };

        // Apply upgrade scaling
        base.with_upgrade_level(upgrade_level)
    }
}
```

### Village Manager (village_manager.rs)

Add player settlement tracking:

```rust
pub struct VillageManager {
    pub npc_villages: Vec<WorldVillage>,
    pub player_settlements: Vec<PlayerSettlement>,  // ADD
    // ...
}
```

### Faction Integration

Building certain structures affects faction relations:

| Building | Faction Effect |
|----------|---------------|
| Chapel | +10 English reputation |
| Trading Post | +5 with trade partner faction |
| Fortress Walls | -5 with neighboring hostile factions |
| Grand Cathedral | +25 English, +10 Spanish |

### Cross-System Synergies

| System | Synergy |
|--------|---------|
| Mining | Foundry requires Mining Tier 4, ore quality affects production |
| Hunting | Tannery building requires Hunting Tier 3 |
| Archaeology | Museum building displays finds, +research per artifact |
| Horse Perks | Stable building unlocks horse breeding |

---

## Implementation Phases

### Phase 1: Core Data Structures
- [ ] Create `city_building.rs` in `roanoke_game/src/progression/`
- [ ] Implement `CityBuildingSkills`, `CityPerk`, `BuildingType` enums
- [ ] Add to `PlayerProgression` struct
- [ ] Implement serialization/deserialization

### Phase 2: Settlement Foundation
- [ ] Create `PlayerSettlement` struct
- [ ] Add settlement claiming mechanic (place claim stake)
- [ ] Implement settlement radius and boundary system
- [ ] Add to `VillageManager`

### Phase 3: Building Construction
- [ ] Building placement UI/system
- [ ] Resource cost checking and consumption
- [ ] Integrate with `BuildingRecipe` generation
- [ ] Construction time and progress tracking

### Phase 4: Perk Unlocking
- [ ] Implement `can_unlock()` logic
- [ ] Point award system for actions
- [ ] Tier advancement tracking
- [ ] UI for perk tree display

### Phase 5: Building Effects
- [ ] Apply `BuildingEffect` modifiers to settlement
- [ ] Population cap management
- [ ] Defense rating calculation
- [ ] Happiness and research tracking

### Phase 6: Edicts & Governance
- [ ] Town Hall edict system
- [ ] Active edict effects
- [ ] Edict cooldowns and costs

### Phase 7: Advanced Features
- [ ] District system (Tier 9)
- [ ] Governor powers (Tier 8)
- [ ] Roanoke Monument questline (Tier 10)
- [ ] The Keeper companion

### Phase 8: Polish & Balance
- [ ] Building upgrade system
- [ ] Attack/defense mechanics
- [ ] NPC settler recruitment
- [ ] Save/load settlement state

---

## Balance Considerations

### Resource Scaling
- Early tiers use common resources (wood, stone)
- Mid tiers introduce processed materials (iron, cloth)
- Late tiers require rare materials (gold, glass, gunpowder)

### Point Economy
- Average player should reach Tier 5 in ~10 hours
- Tier 8 represents major commitment (~40 hours)
- Tier 10 is prestige achievement (~100+ hours)

### Building Limits
- Settlement radius limits total buildings
- Upkeep costs prevent overbuilding
- Population requires housing before growth

### Defense Balance
- Attacks scale with settlement value
- Higher tier = more frequent/dangerous attacks
- Defense investment necessary for survival

---

## UI Considerations

### Settlement View
- Overhead map showing building placement
- Radius boundary visualization
- Resource/population indicators

### Perk Tree View
- Visual tree matching ASCII diagram
- Branch color coding
- Locked/unlocked/available states
- Point requirements display

### Building Placement
- Ghost preview of building
- Valid/invalid placement feedback
- Resource cost display
- Construction time estimate

---

## Future Enhancements

- **Multiple Settlements**: Found satellite settlements with land charters
- **Siege Mechanics**: Detailed attack/defense gameplay
- **NPC Governance**: Appoint mayors to automate settlements
- **Trade Routes**: Establish routes between settlements
- **Seasonal Events**: Harvest festivals, winter preparation
- **Legacy System**: Bonuses carry to new games
