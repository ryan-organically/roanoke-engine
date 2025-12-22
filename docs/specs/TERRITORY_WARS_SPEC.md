# Territory Wars - King of the Hill Mode Specification

## Roanoke Engine - Competitive Base Building & Territory Control

**Status:** DRAFT
**Version:** 0.1.0

---

## Overview

Territory Wars is a competitive mode where players/teams race to claim, fortify, and hold strategic zones. Victory requires mastering the City Building perk tree, managing resources under pressure, and outbuilding opponents. Deep integration with progression systems means invested players have meaningful advantages.

### Design Pillars

1. **Build Speed Matters** - Perk tree investment directly impacts construction velocity
2. **Territory = Resources** - Controlling zones grants resource income
3. **Escalating Conflict** - Early game is land rush, late game is siege warfare
4. **Commendation Rewards** - Exceptional play earns persistent progression
5. **Asymmetric Starts** - Faction choice affects available buildings/strategies

---

## Mode Variants

### King of the Hill (Solo/Team)

One central zone. Control it longest to win.

```rust
pub struct KingOfTheHill {
    pub hill_zone: CaptureZone,
    pub control_time: HashMap<TeamId, Duration>,
    pub win_threshold: Duration,      // 15 minutes total control
    pub match_time_limit: Duration,   // 45 minutes max
}
```

### Conquest (Large Team)

Multiple capture zones. Control majority to tick victory points.

```rust
pub struct ConquestMode {
    pub zones: Vec<CaptureZone>,
    pub zone_count: u32,              // 5-7 zones
    pub majority_threshold: u32,      // 3+ zones = ticking
    pub victory_points: HashMap<TeamId, u32>,
    pub points_to_win: u32,           // 1000 points
}
```

### Domination (Faction War)

Territory expands from faction home bases. Eliminate opponents or hold 80%+ map.

```rust
pub struct DominationMode {
    pub faction_territories: HashMap<Faction, TerritoryGrid>,
    pub map_control_win: f32,         // 80% = instant win
    pub elimination_enabled: bool,     // Losing all territory = eliminated
    pub respawn_at_home: bool,
}
```

### Siege (Attack/Defend)

One team builds for 15 minutes, other team attacks. Swap roles. Best defense wins.

```rust
pub struct SiegeMode {
    pub build_phase: Duration,        // 15 minutes
    pub attack_phase: Duration,       // 10 minutes
    pub rounds: u32,                  // 2 (each team attacks once)
    pub scoring: SiegeScoring,
}

pub struct SiegeScoring {
    pub buildings_destroyed: u32,
    pub zones_captured: u32,
    pub defenders_killed: u32,
    pub time_to_capture: Duration,
}
```

---

## Capture Zones

### Zone Types

```rust
pub enum ZoneType {
    /// Central hill - primary objective
    Hill {
        radius: f32,                   // 50m
        capture_time: Duration,        // 60 seconds uncontested
    },

    /// Resource node - grants income while held
    ResourceNode {
        resource_type: ResourceType,
        yield_per_minute: u32,
        radius: f32,
    },

    /// Strategic point - grants buffs while held
    StrategicPoint {
        buff: ZoneBuff,
        radius: f32,
    },

    /// Spawn point - respawn location
    ForwardBase {
        spawn_enabled: bool,
        build_zone: bool,              // Can construct here
    },
}

pub enum ZoneBuff {
    BuildSpeed { percent: f32 },       // +25% construction
    ResourceEfficiency { percent: f32 }, // -15% build costs
    VisionRange { meters: f32 },       // +50m vision
    DefenseBonus { percent: f32 },     // +20% structure health
    HealingAura { hp_per_sec: f32 },   // +2 HP/s
}
```

### Capture Mechanics

```rust
pub struct CaptureZone {
    pub zone_type: ZoneType,
    pub position: Vec3,
    pub controller: Option<TeamId>,
    pub capture_progress: f32,         // 0.0 to 1.0
    pub contesting_teams: Vec<TeamId>,
    pub structures_inside: Vec<StructureId>,
}

impl CaptureZone {
    /// Capture speed scales with presence
    pub fn capture_rate(&self, team: TeamId, presence: &ZonePresence) -> f32 {
        let base_rate = 1.0 / 60.0;    // 60 seconds base

        // More players = faster capture
        let player_multiplier = match presence.player_count {
            1 => 1.0,
            2 => 1.5,
            3 => 1.8,
            _ => 2.0,
        };

        // Structures in zone accelerate capture
        let structure_bonus = presence.capture_structures as f32 * 0.1;

        // City Building perks affect capture
        let perk_bonus = self.calculate_perk_capture_bonus(team);

        base_rate * player_multiplier * (1.0 + structure_bonus + perk_bonus)
    }

    /// Contested zones tick slower
    pub fn update(&mut self, dt: f32, presence: &HashMap<TeamId, ZonePresence>) {
        let teams_present: Vec<_> = presence.iter()
            .filter(|(_, p)| p.player_count > 0)
            .collect();

        match teams_present.len() {
            0 => {
                // Empty - decay toward neutral
                self.capture_progress *= 0.99;
                if self.capture_progress < 0.01 {
                    self.controller = None;
                }
            }
            1 => {
                // Uncontested capture
                let (team, p) = teams_present[0];
                let rate = self.capture_rate(*team, p);
                self.progress_capture(*team, rate * dt);
            }
            _ => {
                // Contested - no progress, structures take damage
                self.contesting_teams = teams_present.iter().map(|(t, _)| **t).collect();
                self.damage_structures_in_zone(dt);
            }
        }
    }
}

pub struct ZonePresence {
    pub player_count: u32,
    pub total_player_score: u32,       // Sum of relevant perk points
    pub capture_structures: u32,       // Flags, totems, etc.
    pub defensive_structures: u32,     // Walls, towers
}
```

---

## Building System Integration

### Construction Speed Bonuses

City Building perk tree directly affects Territory Wars performance.

```rust
pub struct TerritoryBuildSpeed {
    /// Base build speed multiplier from perks
    pub fn calculate_speed(skills: &CityBuildingSkills) -> f32 {
        let mut multiplier = 1.0;

        // Tier bonuses (cumulative)
        if skills.unlocked.contains(&CityPerk::TimberFrame) {
            multiplier += 0.10;  // +10%
        }
        if skills.unlocked.contains(&CityPerk::StoneMasonry) {
            multiplier += 0.15;  // +15%
        }
        if skills.unlocked.contains(&CityPerk::TownHall) {
            multiplier += 0.20;  // +20%
        }
        if skills.unlocked.contains(&CityPerk::Factory) {
            multiplier += 0.30;  // +30%
        }
        if skills.unlocked.contains(&CityPerk::GovernorsManor) {
            multiplier += 0.25;  // +25%
        }

        // Branch specialization bonus
        let infra_points = skills.points_per_branch[CityBranch::Infrastructure as usize];
        multiplier += (infra_points as f32 / 500.0) * 0.20; // Up to +20% from points

        multiplier
    }
}

/// Building tier availability in Territory Wars
pub fn available_buildings(skills: &CityBuildingSkills, mode: &TerritoryMode) -> Vec<BuildingType> {
    let mut available = vec![
        // Always available
        BuildingType::Campfire,
        BuildingType::LeanTo,
        BuildingType::ToolCache,
    ];

    // Perk-gated buildings
    if skills.unlocked.contains(&CityPerk::PalisadeStakes) {
        available.push(BuildingType::PalisadeSection);
    }
    if skills.unlocked.contains(&CityPerk::Watchtower) {
        available.push(BuildingType::Watchtower);
    }
    if skills.unlocked.contains(&CityPerk::CurtainWall) {
        available.push(BuildingType::CurtainWallSection);
    }
    if skills.unlocked.contains(&CityPerk::Gatehouse) {
        available.push(BuildingType::Gatehouse);
    }
    if skills.unlocked.contains(&CityPerk::Barracks) {
        available.push(BuildingType::Barracks);
    }
    if skills.unlocked.contains(&CityPerk::FortressWalls) {
        available.push(BuildingType::FortressWallSection);
    }
    if skills.unlocked.contains(&CityPerk::Citadel) {
        available.push(BuildingType::Citadel);
    }

    // Mode-specific structures
    available.extend(mode.special_structures());

    available
}
```

### Territory-Specific Structures

```rust
/// Structures unique to Territory Wars
pub enum TerritoryStructure {
    /// Claims territory, accelerates capture
    ClaimFlag {
        capture_bonus: f32,            // +20% capture speed
        health: u32,                   // 500 HP
        build_time: Duration,          // 30 seconds
    },

    /// Reveals enemies in range
    ScoutTower {
        vision_range: f32,             // 100m
        health: u32,
        build_time: Duration,
    },

    /// Damages enemies in zone
    DefensiveTurret {
        damage_per_second: f32,
        range: f32,
        health: u32,
        build_time: Duration,
    },

    /// Spawns allied NPCs
    BarracksOutpost {
        spawn_rate: Duration,          // 1 soldier per 2 min
        max_spawned: u32,              // 4 max
        health: u32,
        build_time: Duration,
    },

    /// Heals allies in radius
    MedicTent {
        heal_per_second: f32,
        radius: f32,
        health: u32,
        build_time: Duration,
    },

    /// Resource generation
    ResourceCache {
        resource_type: ResourceType,
        per_minute: u32,
        health: u32,
        build_time: Duration,
    },

    /// Blocks projectiles
    Barricade {
        health: u32,
        blocks_movement: bool,
        build_time: Duration,
    },

    /// Victory condition structure
    Headquarters {
        health: u32,                   // 5000 HP - very durable
        respawn_enabled: bool,
        build_time: Duration,
        destruction_ends_game: bool,
    },
}

impl TerritoryStructure {
    /// Apply perk bonuses to structure stats
    pub fn with_perks(&self, skills: &CityBuildingSkills) -> Self {
        let mut s = self.clone();

        // Defense perks boost health
        if skills.unlocked.contains(&CityPerk::FortressWalls) {
            s.multiply_health(1.25);
        }
        if skills.unlocked.contains(&CityPerk::Citadel) {
            s.multiply_health(1.50);
        }

        // Infrastructure reduces build time
        let speed_mult = TerritoryBuildSpeed::calculate_speed(skills);
        s.reduce_build_time(speed_mult);

        s
    }
}
```

---

## Resource Economy

### Starting Resources

```rust
pub struct TerritoryStartingResources {
    pub wood: u32,
    pub stone: u32,
    pub iron: u32,
    pub cloth: u32,
}

impl TerritoryStartingResources {
    /// Base starting resources (modified by perks)
    pub fn for_mode(mode: &TerritoryMode, skills: &CityBuildingSkills) -> Self {
        let base = match mode {
            TerritoryMode::KingOfTheHill => Self {
                wood: 200, stone: 100, iron: 25, cloth: 20,
            },
            TerritoryMode::Conquest => Self {
                wood: 300, stone: 150, iron: 50, cloth: 30,
            },
            TerritoryMode::Siege => Self {
                wood: 500, stone: 300, iron: 100, cloth: 50,
            },
            TerritoryMode::Domination => Self {
                wood: 400, stone: 200, iron: 75, cloth: 40,
            },
        };

        // Commerce branch grants starting bonus
        let commerce_bonus = skills.points_per_branch[CityBranch::Commerce as usize];
        let mult = 1.0 + (commerce_bonus as f32 / 1000.0) * 0.30; // Up to +30%

        Self {
            wood: (base.wood as f32 * mult) as u32,
            stone: (base.stone as f32 * mult) as u32,
            iron: (base.iron as f32 * mult) as u32,
            cloth: (base.cloth as f32 * mult) as u32,
        }
    }
}
```

### Resource Nodes

```rust
pub struct ResourceNode {
    pub resource_type: ResourceType,
    pub base_yield: u32,               // Per minute
    pub current_controller: Option<TeamId>,
    pub depletion: f32,                // 0.0 = full, 1.0 = empty
    pub regeneration_rate: f32,        // Per minute when uncontrolled
}

impl ResourceNode {
    pub fn yield_for_team(&self, team: TeamId, skills: &CityBuildingSkills) -> u32 {
        let mut yield_amount = self.base_yield as f32;

        // Depletion reduces yield
        yield_amount *= 1.0 - (self.depletion * 0.5);

        // Commerce perks boost yield
        if skills.unlocked.contains(&CityPerk::TradingPost) {
            yield_amount *= 1.15;
        }
        if skills.unlocked.contains(&CityPerk::Marketplace) {
            yield_amount *= 1.20;
        }
        if skills.unlocked.contains(&CityPerk::Warehouse) {
            yield_amount *= 1.10;
        }
        if skills.unlocked.contains(&CityPerk::Exchange) {
            yield_amount *= 1.25;
        }

        yield_amount as u32
    }
}
```

### Resource Balance Table

| Zone Type | Wood/min | Stone/min | Iron/min | Notes |
|-----------|----------|-----------|----------|-------|
| Lumber Camp | 30 | 0 | 0 | Common |
| Quarry | 0 | 25 | 5 | Uncommon |
| Iron Mine | 0 | 10 | 15 | Rare |
| Mixed Deposit | 15 | 15 | 5 | Central |

---

## Team Composition

### Squad Roles

```rust
pub struct TerritorySquad {
    pub players: Vec<PlayerId>,
    pub role_assignments: HashMap<PlayerId, SquadRole>,
}

pub enum SquadRole {
    /// Primary builder - gets resource priority
    Builder {
        resource_share: f32,           // 40% of team income
        build_speed_bonus: f32,        // +10% personal
    },

    /// Front-line fighter - zone capture focus
    Assault {
        capture_speed_bonus: f32,      // +15%
        damage_bonus: f32,             // +10%
    },

    /// Defensive specialist - protects structures
    Defender {
        structure_repair_speed: f32,   // +25%
        damage_taken_near_structures: f32, // -15%
    },

    /// Resource gatherer - gathers faster
    Harvester {
        gather_speed: f32,             // +40%
        carry_capacity: f32,           // +30%
    },

    /// Commander - buffs nearby allies
    Commander {
        aura_radius: f32,              // 30m
        ally_buff: TeamBuff,
    },
}

pub struct TeamBuff {
    pub damage_bonus: f32,
    pub defense_bonus: f32,
    pub build_speed_bonus: f32,
    pub capture_speed_bonus: f32,
}
```

### Team Perk Pooling

```rust
/// Team shares building unlocks from highest member
pub struct TeamBuildingPool {
    pub team_id: TeamId,
    pub pooled_unlocks: HashSet<BuildingType>,
    pub pooled_speed_bonus: f32,
}

impl TeamBuildingPool {
    pub fn calculate(team: &[PlayerId], player_skills: &HashMap<PlayerId, CityBuildingSkills>) -> Self {
        let mut unlocks = HashSet::new();
        let mut max_speed = 1.0;

        for player in team {
            if let Some(skills) = player_skills.get(player) {
                // Pool all unlocked buildings
                for building in skills.unlocked_buildings() {
                    unlocks.insert(building);
                }

                // Take highest speed bonus
                let speed = TerritoryBuildSpeed::calculate_speed(skills);
                max_speed = max_speed.max(speed);
            }
        }

        Self {
            team_id: team[0].team_id(), // Assume same team
            pooled_unlocks: unlocks,
            pooled_speed_bonus: max_speed,
        }
    }
}
```

---

## Commendation System

Exceptional performance earns commendations that persist across matches.

### Commendation Types

```rust
pub enum Commendation {
    // Building commendations
    MasterBuilder {
        structures_built: u32,
        tier_required: u32,            // 100+ structures
    },
    FortuneBuilder {
        structures_survived: u32,      // Structures that lasted entire match
    },
    SpeedBuilder {
        fastest_structure: Duration,   // Under 10s for complex structure
    },
    ArchitectSupreme {
        base_rating: u32,              // Highest "base score" calculated
    },

    // Combat commendations
    Conqueror {
        zones_captured: u32,           // 10+ in one match
    },
    LastStand {
        solo_defense: Duration,        // Defended zone solo for 2+ min
    },
    Demolisher {
        structures_destroyed: u32,     // 20+ enemy structures
    },
    Untouchable {
        zero_deaths: bool,             // Won match without dying
    },

    // Resource commendations
    Tycoon {
        resources_gathered: u32,       // 5000+ total
    },
    Efficient {
        resource_efficiency: f32,      // Built more with less
    },
    Supplier {
        resources_shared: u32,         // Gave to teammates
    },

    // Team commendations
    ShotCaller {
        pings_followed: u32,           // Team acted on your calls
    },
    Clutch {
        comeback_wins: u32,            // Won from 30%+ deficit
    },
    Mentor {
        new_player_assisted: bool,     // Helped sub-Tier-3 player win
    },

    // Legendary commendations
    Undefeated {
        win_streak: u32,               // 10+ wins in a row
    },
    GrandArchitect {
        tier_10_structures: u32,       // Built Tier 10 structures in match
    },
    OneManArmy {
        solo_vs_team_win: bool,        // Won 1vX
    },
}

impl Commendation {
    pub fn xp_reward(&self) -> u32 {
        match self {
            Self::MasterBuilder { .. } => 500,
            Self::Conqueror { .. } => 400,
            Self::Undefeated { .. } => 2000,
            Self::GrandArchitect { .. } => 1500,
            Self::OneManArmy { .. } => 3000,
            _ => 250,
        }
    }

    pub fn perk_point_reward(&self, branch: CityBranch) -> u32 {
        match self {
            Self::MasterBuilder { .. } => 50,        // Infrastructure
            Self::FortuneBuilder { .. } => 30,       // Defense
            Self::ArchitectSupreme { .. } => 100,    // Infrastructure
            Self::Tycoon { .. } => 50,               // Commerce
            _ => 10,
        }
    }
}
```

### Commendation Progress Tracking

```rust
pub struct CommendationProgress {
    pub player_id: PlayerId,
    pub lifetime_stats: TerritoryLifetimeStats,
    pub earned_commendations: Vec<(Commendation, DateTime)>,
    pub active_challenges: Vec<CommendationChallenge>,
}

pub struct TerritoryLifetimeStats {
    // Building
    pub structures_built: u32,
    pub structures_lost: u32,
    pub total_build_time: Duration,
    pub fastest_structure: Duration,

    // Combat
    pub zones_captured: u32,
    pub zones_lost: u32,
    pub enemies_killed: u32,
    pub deaths: u32,

    // Resources
    pub resources_gathered: u32,
    pub resources_spent: u32,
    pub resources_shared: u32,

    // Matches
    pub matches_played: u32,
    pub matches_won: u32,
    pub current_win_streak: u32,
    pub best_win_streak: u32,
}

pub struct CommendationChallenge {
    pub commendation: Commendation,
    pub progress: f32,                 // 0.0 to 1.0
    pub expires: Option<DateTime>,     // Some are time-limited
}
```

### Commendation Rewards

```rust
pub struct CommendationRewards {
    pub xp_earned: u32,
    pub perk_points: HashMap<CityBranch, u32>,
    pub titles: Vec<String>,
    pub cosmetics: Vec<CosmeticId>,
    pub blueprint_unlocks: Vec<BuildingType>,
}

/// Special buildings unlocked via commendations
pub fn commendation_exclusive_buildings() -> Vec<(Commendation, BuildingType)> {
    vec![
        // Master Builder unlocks decorative variants
        (Commendation::MasterBuilder { structures_built: 500, tier_required: 0 },
         BuildingType::GildedWatchtower),

        // Architect Supreme unlocks grand structures
        (Commendation::ArchitectSupreme { base_rating: 1000 },
         BuildingType::TriumphalArch),

        // Undefeated unlocks victory monuments
        (Commendation::Undefeated { win_streak: 10 },
         BuildingType::VictoryObelisk),

        // Grand Architect unlocks legendary structures
        (Commendation::GrandArchitect { tier_10_structures: 5 },
         BuildingType::ColossalStatue),
    ]
}
```

---

## Faction Integration

### Faction-Specific Buildings

```rust
pub fn faction_buildings(faction: Faction) -> Vec<FactionBuilding> {
    match faction {
        Faction::English => vec![
            FactionBuilding {
                building: BuildingType::PalisadeSection,
                faction_variant: "English Stockade",
                stat_modifier: BuildingModifier::Health(1.15),
            },
            FactionBuilding {
                building: BuildingType::Watchtower,
                faction_variant: "English Lookout",
                stat_modifier: BuildingModifier::VisionRange(1.25),
            },
        ],
        Faction::Spanish => vec![
            FactionBuilding {
                building: BuildingType::CurtainWallSection,
                faction_variant: "Spanish Rampart",
                stat_modifier: BuildingModifier::Defense(1.30),
            },
            FactionBuilding {
                building: BuildingType::Gatehouse,
                faction_variant: "Presidio Gate",
                stat_modifier: BuildingModifier::Health(1.40),
            },
        ],
        Faction::French => vec![
            FactionBuilding {
                building: BuildingType::TradingPost,
                faction_variant: "French Trading House",
                stat_modifier: BuildingModifier::ResourceYield(1.35),
            },
            FactionBuilding {
                building: BuildingType::Cabin,
                faction_variant: "Coureur's Lodge",
                stat_modifier: BuildingModifier::BuildSpeed(1.20),
            },
        ],
        Faction::Powhatan | Faction::Cherokee | Faction::Tuscarora => vec![
            FactionBuilding {
                building: BuildingType::Longhouse,
                faction_variant: "Tribal Longhouse",
                stat_modifier: BuildingModifier::Capacity(1.50),
            },
            FactionBuilding {
                building: BuildingType::Watchtower,
                faction_variant: "Scout Platform",
                stat_modifier: BuildingModifier::BuildSpeed(1.40),
            },
        ],
        Faction::Aztec => vec![
            FactionBuilding {
                building: BuildingType::Citadel,
                faction_variant: "Temple Fortress",
                stat_modifier: BuildingModifier::Intimidation(1.50),
            },
            FactionBuilding {
                building: BuildingType::Barracks,
                faction_variant: "Warrior House",
                stat_modifier: BuildingModifier::SpawnRate(1.25),
            },
        ],
        _ => vec![],
    }
}
```

### Faction Skill Synergies

```rust
/// Faction skills that boost Territory Wars performance
pub fn faction_territory_bonuses(faction_skills: &HashMap<Faction, Vec<FactionSkillId>>) -> TerritoryBonuses {
    let mut bonuses = TerritoryBonuses::default();

    // English colonist bonuses
    if faction_skills.get(&Faction::English).map_or(false, |s| s.contains(&FactionSkillId::MasterBuilder)) {
        bonuses.build_speed += 0.25;
        bonuses.structure_health += 0.15;
    }
    if faction_skills.get(&Faction::English).map_or(false, |s| s.contains(&FactionSkillId::MilitiaCaptain)) {
        bonuses.npc_spawn_rate += 0.20;
        bonuses.defense_rating += 0.20;
    }

    // Spanish conquistador bonuses
    if faction_skills.get(&Faction::Spanish).map_or(false, |s| s.contains(&FactionSkillId::TercioFormation)) {
        bonuses.team_damage_near_structures += 0.20;
    }

    // French coureur bonuses
    if faction_skills.get(&Faction::French).map_or(false, |s| s.contains(&FactionSkillId::TradeEmpire)) {
        bonuses.resource_yield += 0.30;
        bonuses.starting_resources += 0.25;
    }

    // Native faction bonuses
    if faction_skills.get(&Faction::Cherokee).map_or(false, |s| s.contains(&FactionSkillId::RedWarChief)) {
        bonuses.capture_speed += 0.25;
        bonuses.zone_contest_damage += 0.15;
    }

    bonuses
}

#[derive(Default)]
pub struct TerritoryBonuses {
    pub build_speed: f32,
    pub structure_health: f32,
    pub resource_yield: f32,
    pub capture_speed: f32,
    pub defense_rating: f32,
    pub npc_spawn_rate: f32,
    pub starting_resources: f32,
    pub team_damage_near_structures: f32,
    pub zone_contest_damage: f32,
}
```

---

## Match Flow

### King of the Hill Example

```rust
pub struct KOTHMatch {
    pub phase: KOTHPhase,
    pub hill: CaptureZone,
    pub teams: Vec<Team>,
    pub control_scores: HashMap<TeamId, Duration>,
    pub match_time: Duration,
}

pub enum KOTHPhase {
    /// 2 minute setup - build near hill
    Setup {
        time_remaining: Duration,
        build_zone_active: bool,
    },

    /// Main game - capture and hold
    Active {
        hill_controller: Option<TeamId>,
        control_timer: Duration,
    },

    /// Overtime - hill locked, sudden death
    Overtime {
        locked_controller: TeamId,
        elimination_enabled: bool,
    },

    /// Match complete
    Complete {
        winner: TeamId,
        final_scores: HashMap<TeamId, Duration>,
    },
}

impl KOTHMatch {
    pub fn update(&mut self, dt: Duration) {
        match &mut self.phase {
            KOTHPhase::Setup { time_remaining, .. } => {
                *time_remaining = time_remaining.saturating_sub(dt);
                if time_remaining.is_zero() {
                    self.phase = KOTHPhase::Active {
                        hill_controller: None,
                        control_timer: Duration::ZERO,
                    };
                }
            }
            KOTHPhase::Active { hill_controller, control_timer } => {
                self.hill.update(dt.as_secs_f32(), &self.calculate_presence());

                if let Some(controller) = self.hill.controller {
                    *hill_controller = Some(controller);
                    *control_timer += dt;

                    // Add to total score
                    *self.control_scores.entry(controller).or_default() += dt;

                    // Check win condition
                    if self.control_scores[&controller] >= Duration::from_secs(900) { // 15 min
                        self.phase = KOTHPhase::Complete {
                            winner: controller,
                            final_scores: self.control_scores.clone(),
                        };
                    }
                } else {
                    *hill_controller = None;
                    *control_timer = Duration::ZERO;
                }

                // Check time limit
                if self.match_time >= Duration::from_secs(2700) { // 45 min
                    self.enter_overtime();
                }
            }
            KOTHPhase::Overtime { .. } => {
                // Elimination mode - last team standing
            }
            KOTHPhase::Complete { .. } => {}
        }

        self.match_time += dt;
    }
}
```

### Siege Mode Flow

```rust
pub struct SiegeMatch {
    pub round: u32,
    pub phase: SiegePhase,
    pub attacker: TeamId,
    pub defender: TeamId,
    pub round_scores: Vec<SiegeRoundScore>,
}

pub enum SiegePhase {
    /// Defender builds fortifications
    BuildPhase {
        defender: TeamId,
        time_remaining: Duration,      // 15 minutes
        resources_granted: u32,
    },

    /// Attacker assaults
    AttackPhase {
        attacker: TeamId,
        time_remaining: Duration,      // 10 minutes
        objectives: Vec<SiegeObjective>,
    },

    /// Swap teams
    RoleSwap,

    /// Compare scores
    Scoring,
}

pub struct SiegeObjective {
    pub objective_type: ObjectiveType,
    pub position: Vec3,
    pub captured: bool,
    pub capture_time: Option<Duration>,
}

pub enum ObjectiveType {
    CapturePoint,
    DestroyStructure { target: StructureId },
    EliminateDefenders { count: u32 },
    HoldPosition { duration: Duration },
}
```

---

## Scoring & Leaderboards

### Match Scoring

```rust
pub struct MatchScore {
    pub player_id: PlayerId,
    pub team_id: TeamId,

    // Core metrics
    pub zones_captured: u32,
    pub zone_time_held: Duration,
    pub structures_built: u32,
    pub structures_lost: u32,
    pub structures_destroyed: u32,

    // Combat
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub damage_dealt: u32,
    pub damage_to_structures: u32,

    // Economy
    pub resources_gathered: u32,
    pub resources_spent: u32,
    pub resource_efficiency: f32,

    // Team play
    pub objectives_completed: u32,
    pub teammates_revived: u32,
    pub buff_time_provided: Duration,
}

impl MatchScore {
    pub fn total_score(&self) -> u32 {
        let capture_score = self.zones_captured * 500 + self.zone_time_held.as_secs() as u32;
        let build_score = self.structures_built * 100 - self.structures_lost * 50;
        let combat_score = self.kills * 100 + self.assists * 50 - self.deaths * 25;
        let econ_score = (self.resource_efficiency * 500.0) as u32;
        let team_score = self.objectives_completed * 200 + self.teammates_revived * 100;

        capture_score + build_score.max(0) as u32 + combat_score.max(0) as u32 + econ_score + team_score
    }
}
```

### Leaderboard Categories

```rust
pub enum LeaderboardCategory {
    // Overall
    TotalScore,
    WinRate,
    MatchesWon,

    // Building
    StructuresBuilt,
    BuildEfficiency,        // Score per resource spent
    FastestBuilder,         // Avg build time

    // Territory
    ZonesCaptured,
    TotalHoldTime,
    ContestWins,            // Capturing contested zones

    // Combat
    KillDeathRatio,
    StructuresDestroyed,
    DefensiveKills,         // Kills near own structures

    // Faction-specific
    FactionWinRate { faction: Faction },
    FactionBuildScore { faction: Faction },
}
```

---

## Map Design Requirements

### Zone Placement

```rust
pub struct TerritoryMap {
    pub name: String,
    pub size: MapSize,
    pub zones: Vec<ZonePlacement>,
    pub spawn_areas: Vec<SpawnArea>,
    pub resource_nodes: Vec<ResourceNodePlacement>,
}

pub struct ZonePlacement {
    pub zone_type: ZoneType,
    pub position: Vec3,
    pub terrain_advantage: TerrainAdvantage,
    pub natural_cover: f32,            // 0.0 to 1.0
}

pub enum TerrainAdvantage {
    HighGround,                        // +15% defense
    LowGround,                         // -10% defense
    WaterAdjacent,                     // French bonus
    ForestCover,                       // Native bonus
    OpenField,                         // Spanish bonus
    Fortifiable,                       // English bonus
}

pub enum MapSize {
    Small { diameter: f32 },           // 500m - 2v2
    Medium { diameter: f32 },          // 1000m - 4v4
    Large { diameter: f32 },           // 2000m - 8v8
    Massive { diameter: f32 },         // 4000m - Domination
}
```

### Example Map: "Three Rivers Confluence"

```
Map: Three Rivers Confluence (Medium - 4v4)

     [NORTH SPAWN]
          |
     [Lumber Camp]
          |
    [Scout Post]----[HILL]----[Scout Post]
          |           |           |
     [Quarry]    [Iron Mine]   [Quarry]
          |           |           |
    [Forward Base]   |    [Forward Base]
          \         |         /
           \   [Trading]    /
            \    Post     /
             \    |      /
              \   |     /
               \  |    /
            [SOUTH SPAWN]

Zone Distribution:
- 1 Central Hill (primary objective)
- 2 Forward Bases (spawn points, build zones)
- 2 Scout Posts (vision buffs)
- 2 Quarries (stone income)
- 1 Lumber Camp (wood income)
- 1 Iron Mine (iron income - contested)
- 1 Trading Post (resource conversion)
```

---

## Implementation Checklist

### Phase 1: Core Systems

- [ ] Capture zone implementation
- [ ] Resource node mechanics
- [ ] Team spawning and respawn
- [ ] Basic structure placement in zones

### Phase 2: Perk Integration

- [ ] City Building perk speed bonuses
- [ ] Building unlock gating
- [ ] Team perk pooling
- [ ] Faction building variants

### Phase 3: Game Modes

- [ ] King of the Hill implementation
- [ ] Conquest mode
- [ ] Siege mode
- [ ] Domination mode

### Phase 4: Commendations

- [ ] Progress tracking
- [ ] Reward distribution
- [ ] Exclusive building unlocks
- [ ] Leaderboard integration

### Phase 5: Polish

- [ ] Map design tools
- [ ] Spectator mode
- [ ] Match replay
- [ ] Ranked matchmaking

---

## Files to Create

```
roanoke_game/src/territory_wars/
├── mod.rs
├── capture_zone.rs        // Zone mechanics
├── resources.rs           // Economy
├── structures.rs          // Territory-specific buildings
├── scoring.rs             // Match scoring
├── commendations.rs       // Progression rewards
├── perk_integration.rs    // City Building integration
├── faction_bonuses.rs     // Faction synergies
├── modes/
│   ├── mod.rs
│   ├── king_of_hill.rs
│   ├── conquest.rs
│   ├── siege.rs
│   └── domination.rs
└── maps/
    ├── mod.rs
    └── three_rivers.rs
```

---

*End of Territory Wars Specification*
