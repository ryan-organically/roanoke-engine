//! Colonial Dynamics System
//!
//! Models the complex three-way relationships between:
//! - English colonists (Roanoke settlers, later Jamestown)
//! - Spanish explorers (Florida bases, competing claims)
//! - Native American nations (Powhatan Confederacy, Tuscarora, Cherokee, etc.)
//!
//! Historical context: 1580s-1600s Virginia/Carolina coast
//! - English attempting permanent settlement
//! - Spanish protecting Caribbean trade routes
//! - Native nations navigating European arrival

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use glam::Vec3;

use super::faction::{Faction, Standing};

// ============================================================================
// COLONIAL POWER STRUCTURES
// ============================================================================

/// Major colonial powers and their goals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColonialPower {
    /// English Virginia Company - settlement and trade
    English,
    /// Spanish Crown - protect treasure fleets, Catholic missions
    Spanish,
    /// French traders - fur trade, native alliances
    French,
}

impl ColonialPower {
    pub fn primary_goal(&self) -> &'static str {
        match self {
            Self::English => "Establish permanent colonies and find gold",
            Self::Spanish => "Protect trade routes and spread Catholicism",
            Self::French => "Fur trade and native alliances",
        }
    }

    pub fn attitude_to_natives(&self) -> NativePolicy {
        match self {
            Self::English => NativePolicy::Displacement, // Take land for farming
            Self::Spanish => NativePolicy::Conversion,   // Missions and encomienda
            Self::French => NativePolicy::Alliance,      // Trade partners
        }
    }

    pub fn to_faction(&self) -> Faction {
        match self {
            Self::English => Faction::English,
            Self::Spanish => Faction::Spanish,
            Self::French => Faction::French,
        }
    }
}

/// How a colonial power treats native peoples
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativePolicy {
    /// Seek to remove natives from desired land
    Displacement,
    /// Seek to convert and subjugate
    Conversion,
    /// Seek trade relationships
    Alliance,
    /// Active warfare
    Extermination,
}

// ============================================================================
// NATIVE NATION STRUCTURES
// ============================================================================

/// Native American nations with distinct cultures and territories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NativeNation {
    /// Powhatan Confederacy - tidewater Virginia
    Powhatan,
    /// Tuscarora - North Carolina interior
    Tuscarora,
    /// Cherokee - Appalachian mountains
    Cherokee,
    /// Catawba - Carolina piedmont
    Catawba,
    /// Pamunkey - part of Powhatan, often separate dealings
    Pamunkey,
    /// Croatan/Croatoan - Outer Banks (historically tied to Lost Colony)
    Croatan,
}

impl NativeNation {
    pub fn to_faction(&self) -> Faction {
        match self {
            Self::Powhatan => Faction::Powhatan,
            Self::Tuscarora => Faction::Tuscarora,
            Self::Cherokee => Faction::Cherokee,
            Self::Catawba => Faction::Catawba,
            Self::Pamunkey => Faction::Pamunkey,
            Self::Croatan => Faction::Powhatan, // Aligned with Powhatan
        }
    }

    pub fn territory_description(&self) -> &'static str {
        match self {
            Self::Powhatan => "Tidewater Virginia, Chesapeake Bay tributaries",
            Self::Tuscarora => "North Carolina interior, Neuse and Tar rivers",
            Self::Cherokee => "Appalachian highlands, Tennessee River headwaters",
            Self::Catawba => "Carolina piedmont, Catawba River valley",
            Self::Pamunkey => "Pamunkey River, York River tributaries",
            Self::Croatan => "Outer Banks, Roanoke Island area",
        }
    }

    pub fn government_type(&self) -> GovernmentType {
        match self {
            Self::Powhatan => GovernmentType::Confederacy, // Paramount chiefdom
            Self::Tuscarora => GovernmentType::Confederacy,
            Self::Cherokee => GovernmentType::CouncilRule, // Town councils
            Self::Catawba => GovernmentType::Chiefdom,
            Self::Pamunkey => GovernmentType::Chiefdom,    // Under Powhatan
            Self::Croatan => GovernmentType::Chiefdom,
        }
    }

    pub fn population_estimate(&self) -> u32 {
        match self {
            Self::Powhatan => 15000,   // Confederacy total
            Self::Tuscarora => 5000,
            Self::Cherokee => 20000,   // Larger nation
            Self::Catawba => 5000,
            Self::Pamunkey => 1000,    // Single tribe
            Self::Croatan => 500,      // Small coastal group
        }
    }

    pub fn warrior_count(&self) -> u32 {
        // Roughly 20-25% of population
        self.population_estimate() / 4
    }

    pub fn all() -> &'static [NativeNation] {
        &[
            Self::Powhatan,
            Self::Tuscarora,
            Self::Cherokee,
            Self::Catawba,
            Self::Pamunkey,
            Self::Croatan,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernmentType {
    /// Multiple tribes under paramount chief
    Confederacy,
    /// Single chief rules
    Chiefdom,
    /// Decisions by council of elders
    CouncilRule,
}

// ============================================================================
// TERRITORIAL CLAIMS
// ============================================================================

/// A claimed territory by a faction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritorialClaim {
    pub id: u32,
    pub name: String,
    pub claimant: Faction,
    pub center: [f32; 3],
    pub radius: f32,
    /// Strength of claim (0.0 = contested, 1.0 = undisputed)
    pub claim_strength: f32,
    /// How long the claim has been held (game days)
    pub held_days: f32,
    /// Competing claims on this territory
    pub contested_by: Vec<Faction>,
    /// Resources in this territory
    pub resources: TerritoryResources,
    /// Settlements in this territory
    pub settlements: Vec<u32>,
    /// Historical native territory
    pub native_homeland: Option<NativeNation>,
}

impl TerritorialClaim {
    pub fn new(
        id: u32,
        name: &str,
        claimant: Faction,
        center: [f32; 3],
        radius: f32,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            claimant,
            center,
            radius,
            claim_strength: 0.5,
            held_days: 0.0,
            contested_by: Vec::new(),
            resources: TerritoryResources::default(),
            settlements: Vec::new(),
            native_homeland: None,
        }
    }

    pub fn with_native_homeland(mut self, nation: NativeNation) -> Self {
        self.native_homeland = Some(nation);
        self
    }

    pub fn with_resources(mut self, resources: TerritoryResources) -> Self {
        self.resources = resources;
        self
    }

    /// Check if a position is within this territory
    pub fn contains(&self, position: [f32; 3]) -> bool {
        let dx = position[0] - self.center[0];
        let dz = position[2] - self.center[2];
        (dx * dx + dz * dz).sqrt() <= self.radius
    }

    /// Update claim strength over time
    pub fn update(&mut self, delta_days: f32) {
        self.held_days += delta_days;

        // Claims strengthen over time if uncontested
        if self.contested_by.is_empty() {
            self.claim_strength = (self.claim_strength + 0.01 * delta_days).min(1.0);
        } else {
            // Contested claims weaken
            let contest_factor = self.contested_by.len() as f32 * 0.02;
            self.claim_strength = (self.claim_strength - contest_factor * delta_days).max(0.1);
        }
    }

    /// Add a contesting faction
    pub fn contest(&mut self, faction: Faction) {
        if faction != self.claimant && !self.contested_by.contains(&faction) {
            self.contested_by.push(faction);
        }
    }

    /// Resolve contest in favor of a faction
    pub fn resolve_contest(&mut self, winner: Faction) {
        if winner != self.claimant {
            self.claimant = winner;
            self.claim_strength = 0.3;
            self.held_days = 0.0;
        }
        self.contested_by.clear();
    }
}

/// Resources available in a territory
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerritoryResources {
    /// Timber availability (0-1)
    pub timber: f32,
    /// Iron ore deposits (0-1)
    pub iron_ore: f32,
    /// Gold deposits (0-1, usually very low)
    pub gold: f32,
    /// Fur-bearing animal density (0-1)
    pub furs: f32,
    /// Arable land quality (0-1)
    pub farmland: f32,
    /// Fish/shellfish availability (0-1)
    pub fishing: f32,
    /// Fresh water access (0-1)
    pub fresh_water: f32,
    /// Strategic value (harbors, passes) (0-1)
    pub strategic: f32,
}

impl TerritoryResources {
    pub fn coastal() -> Self {
        Self {
            timber: 0.7,
            iron_ore: 0.1,
            gold: 0.0,
            furs: 0.4,
            farmland: 0.5,
            fishing: 0.9,
            fresh_water: 0.6,
            strategic: 0.8,
        }
    }

    pub fn forest() -> Self {
        Self {
            timber: 0.95,
            iron_ore: 0.2,
            gold: 0.05,
            furs: 0.8,
            farmland: 0.3,
            fishing: 0.2,
            fresh_water: 0.7,
            strategic: 0.3,
        }
    }

    pub fn river_valley() -> Self {
        Self {
            timber: 0.6,
            iron_ore: 0.15,
            gold: 0.02,
            furs: 0.6,
            farmland: 0.9,
            fishing: 0.7,
            fresh_water: 0.95,
            strategic: 0.6,
        }
    }

    pub fn mountains() -> Self {
        Self {
            timber: 0.5,
            iron_ore: 0.7,
            gold: 0.15,
            furs: 0.5,
            farmland: 0.1,
            fishing: 0.1,
            fresh_water: 0.8,
            strategic: 0.4,
        }
    }

    /// Get total economic value
    pub fn total_value(&self) -> f32 {
        self.timber * 1.0
            + self.iron_ore * 2.0
            + self.gold * 10.0
            + self.furs * 1.5
            + self.farmland * 1.2
            + self.fishing * 0.8
            + self.fresh_water * 0.5
            + self.strategic * 2.0
    }
}

// ============================================================================
// INTER-FACTION CONFLICTS
// ============================================================================

/// Active conflict between factions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionConflict {
    pub id: u32,
    pub name: String,
    pub aggressor: Faction,
    pub defender: Faction,
    pub conflict_type: ConflictType,
    pub intensity: ConflictIntensity,
    pub cause: ConflictCause,
    /// Territory being fought over (if any)
    pub disputed_territory: Option<u32>,
    /// Days since conflict started
    pub duration_days: f32,
    /// Casualties on each side
    pub aggressor_casualties: u32,
    pub defender_casualties: u32,
    /// Current momentum (-1.0 defender winning, 1.0 aggressor winning)
    pub momentum: f32,
    /// Peace terms being offered
    pub peace_terms: Option<PeaceTerms>,
}

impl FactionConflict {
    pub fn new(
        id: u32,
        name: &str,
        aggressor: Faction,
        defender: Faction,
        conflict_type: ConflictType,
        cause: ConflictCause,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            aggressor,
            defender,
            conflict_type,
            intensity: ConflictIntensity::Skirmishes,
            cause,
            disputed_territory: None,
            duration_days: 0.0,
            aggressor_casualties: 0,
            defender_casualties: 0,
            momentum: 0.0,
            peace_terms: None,
        }
    }

    pub fn with_territory(mut self, territory_id: u32) -> Self {
        self.disputed_territory = Some(territory_id);
        self
    }

    /// Update conflict state
    pub fn update(&mut self, delta_days: f32) {
        self.duration_days += delta_days;

        // Long conflicts tend to de-escalate (war weariness)
        if self.duration_days > 180.0 {
            self.intensity = self.intensity.decrease();
        }
    }

    /// Record a battle outcome
    pub fn record_battle(&mut self, aggressor_won: bool, aggressor_losses: u32, defender_losses: u32) {
        self.aggressor_casualties += aggressor_losses;
        self.defender_casualties += defender_losses;

        // Update momentum
        let momentum_shift = if aggressor_won { 0.1 } else { -0.1 };
        self.momentum = (self.momentum + momentum_shift).clamp(-1.0, 1.0);

        // Decisive victories can escalate
        if (aggressor_losses == 0 && defender_losses > 10)
            || (defender_losses == 0 && aggressor_losses > 10)
        {
            self.intensity = self.intensity.increase();
        }
    }

    /// Check if conflict should end
    pub fn should_end(&self) -> bool {
        // End if one side has overwhelming momentum
        self.momentum.abs() > 0.8 && self.duration_days > 30.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Small-scale raids
    Raiding,
    /// Territorial dispute
    TerritorialWar,
    /// Trade route control
    TradeWar,
    /// Religious/cultural conflict
    HolyWar,
    /// Survival conflict
    ExterminationWar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictIntensity {
    /// Occasional small raids
    Skirmishes,
    /// Regular attacks
    OpenHostilities,
    /// Major military campaigns
    FullWar,
    /// Total war, destruction of settlements
    TotalWar,
}

impl ConflictIntensity {
    pub fn increase(self) -> Self {
        match self {
            Self::Skirmishes => Self::OpenHostilities,
            Self::OpenHostilities => Self::FullWar,
            Self::FullWar => Self::TotalWar,
            Self::TotalWar => Self::TotalWar,
        }
    }

    pub fn decrease(self) -> Self {
        match self {
            Self::Skirmishes => Self::Skirmishes,
            Self::OpenHostilities => Self::Skirmishes,
            Self::FullWar => Self::OpenHostilities,
            Self::TotalWar => Self::FullWar,
        }
    }

    pub fn spawn_rate_modifier(&self) -> f32 {
        match self {
            Self::Skirmishes => 0.1,
            Self::OpenHostilities => 0.3,
            Self::FullWar => 0.6,
            Self::TotalWar => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictCause {
    TerritorialExpansion,
    ResourceCompetition,
    ReligiousDifferences,
    Revenge,
    BrokenTreaty,
    Kidnapping,
    Murder,
    TradeDispute,
}

/// Terms for ending a conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceTerms {
    pub proposer: Faction,
    pub territory_cessions: Vec<u32>,
    pub tribute_amount: u32,
    pub prisoner_exchange: bool,
    pub trade_agreement: bool,
    pub alliance_clause: bool,
}

// ============================================================================
// DIPLOMATIC ACTIONS
// ============================================================================

/// Actions players can take in colonial diplomacy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiplomaticAction {
    /// Propose alliance
    ProposeAlliance,
    /// Offer trade agreement
    ProposeTrade,
    /// Declare war
    DeclareWar,
    /// Sue for peace
    ProposePeace,
    /// Gift goods to improve relations
    SendGift,
    /// Share intelligence about another faction
    ShareIntelligence,
    /// Betray ally to another faction
    Betray,
    /// Offer to mediate conflict
    MediateConflict,
    /// Claim territory
    ClaimTerritory,
    /// Renounce claim to territory
    RenounceClaim,
    /// Request military aid
    RequestAid,
    /// Marry into faction (blood bond)
    ProposeMarriage,
}

impl DiplomaticAction {
    /// Get reputation requirement to perform this action
    pub fn required_standing(&self) -> Standing {
        match self {
            Self::ProposeAlliance => Standing::Friendly,
            Self::ProposeTrade => Standing::Neutral,
            Self::DeclareWar => Standing::Hostile, // Can declare from any standing
            Self::ProposePeace => Standing::Hostile,
            Self::SendGift => Standing::Suspicious,
            Self::ShareIntelligence => Standing::Neutral,
            Self::Betray => Standing::Neutral,
            Self::MediateConflict => Standing::Friendly,
            Self::ClaimTerritory => Standing::Neutral,
            Self::RenounceClaim => Standing::Neutral,
            Self::RequestAid => Standing::Allied,
            Self::ProposeMarriage => Standing::Allied,
        }
    }

    /// Get reputation change if action succeeds
    pub fn success_reputation(&self) -> i32 {
        match self {
            Self::ProposeAlliance => 200,
            Self::ProposeTrade => 50,
            Self::DeclareWar => -500,
            Self::ProposePeace => 100,
            Self::SendGift => 30,
            Self::ShareIntelligence => 40,
            Self::Betray => -1000,
            Self::MediateConflict => 150,
            Self::ClaimTerritory => -20,
            Self::RenounceClaim => 50,
            Self::RequestAid => 0,
            Self::ProposeMarriage => 500,
        }
    }

    /// Get reputation change if action fails/rejected
    pub fn failure_reputation(&self) -> i32 {
        match self {
            Self::ProposeAlliance => -20,
            Self::ProposeTrade => -5,
            Self::DeclareWar => 0,
            Self::ProposePeace => -30,
            Self::SendGift => 0, // Can't really fail
            Self::ShareIntelligence => -10,
            Self::Betray => -200, // Caught trying
            Self::MediateConflict => -10,
            Self::ClaimTerritory => -50,
            Self::RenounceClaim => 0,
            Self::RequestAid => -10,
            Self::ProposeMarriage => -50,
        }
    }
}

// ============================================================================
// COLONIAL DYNAMICS MANAGER
// ============================================================================

/// Central manager for colonial faction dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColonialDynamicsManager {
    /// All territorial claims
    pub territories: HashMap<u32, TerritorialClaim>,
    next_territory_id: u32,

    /// Active conflicts
    pub conflicts: HashMap<u32, FactionConflict>,
    next_conflict_id: u32,

    /// Historical events (for narrative)
    pub history: Vec<HistoricalEvent>,

    /// Current colonial balance of power
    pub power_balance: PowerBalance,

    /// Trade agreements in effect
    pub trade_agreements: Vec<TradeAgreement>,

    /// Alliances in effect
    pub alliances: Vec<Alliance>,
}

impl Default for ColonialDynamicsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ColonialDynamicsManager {
    pub fn new() -> Self {
        let mut manager = Self {
            territories: HashMap::new(),
            next_territory_id: 0,
            conflicts: HashMap::new(),
            next_conflict_id: 0,
            history: Vec::new(),
            power_balance: PowerBalance::default(),
            trade_agreements: Vec::new(),
            alliances: Vec::new(),
        };

        // Initialize historical territories
        manager.initialize_territories();
        manager
    }

    fn initialize_territories(&mut self) {
        // Powhatan Confederacy lands
        let powhatan_id = self.create_territory(
            "Tsenacommacah",
            Faction::Powhatan,
            [0.0, 0.0, 0.0],
            2000.0,
        );
        if let Some(t) = self.territories.get_mut(&powhatan_id) {
            t.native_homeland = Some(NativeNation::Powhatan);
            t.resources = TerritoryResources::forest();
        }

        // Roanoke area (Croatan)
        let roanoke = self.create_territory(
            "Roanoke",
            Faction::Powhatan,
            [3000.0, 0.0, 2000.0],
            500.0,
        );
        if let Some(t) = self.territories.get_mut(&roanoke) {
            t.native_homeland = Some(NativeNation::Croatan);
            t.resources = TerritoryResources::coastal();
        }

        // Tuscarora lands
        let tuscarora = self.create_territory(
            "Tuscarora Lands",
            Faction::Tuscarora,
            [2000.0, 0.0, -1500.0],
            1500.0,
        );
        if let Some(t) = self.territories.get_mut(&tuscarora) {
            t.native_homeland = Some(NativeNation::Tuscarora);
            t.resources = TerritoryResources::river_valley();
        }

        // Cherokee mountains
        let cherokee = self.create_territory(
            "Cherokee Highlands",
            Faction::Cherokee,
            [-3000.0, 0.0, -2000.0],
            2500.0,
        );
        if let Some(t) = self.territories.get_mut(&cherokee) {
            t.native_homeland = Some(NativeNation::Cherokee);
            t.resources = TerritoryResources::mountains();
        }
    }

    /// Create a new territorial claim
    pub fn create_territory(
        &mut self,
        name: &str,
        claimant: Faction,
        center: [f32; 3],
        radius: f32,
    ) -> u32 {
        let id = self.next_territory_id;
        self.next_territory_id += 1;

        let claim = TerritorialClaim::new(id, name, claimant, center, radius);
        self.territories.insert(id, claim);

        id
    }

    /// Start a conflict between factions
    pub fn start_conflict(
        &mut self,
        name: &str,
        aggressor: Faction,
        defender: Faction,
        conflict_type: ConflictType,
        cause: ConflictCause,
        game_time: f64,
    ) -> u32 {
        let id = self.next_conflict_id;
        self.next_conflict_id += 1;

        let conflict = FactionConflict::new(id, name, aggressor, defender, conflict_type, cause);
        self.conflicts.insert(id, conflict);

        // Record in history
        self.history.push(HistoricalEvent {
            timestamp: game_time,
            event_type: HistoricalEventType::WarDeclared,
            factions_involved: vec![aggressor, defender],
            description: format!("{} declared war on {}: {}",
                aggressor.name(), defender.name(), name),
        });

        id
    }

    /// End a conflict
    pub fn end_conflict(&mut self, conflict_id: u32, winner: Faction, game_time: f64) {
        if let Some(conflict) = self.conflicts.remove(&conflict_id) {
            let loser = if winner == conflict.aggressor {
                conflict.defender
            } else {
                conflict.aggressor
            };

            // Resolve any disputed territory
            if let Some(territory_id) = conflict.disputed_territory {
                if let Some(territory) = self.territories.get_mut(&territory_id) {
                    territory.resolve_contest(winner);
                }
            }

            // Record in history
            self.history.push(HistoricalEvent {
                timestamp: game_time,
                event_type: HistoricalEventType::WarEnded,
                factions_involved: vec![winner, loser],
                description: format!("{} defeated {} in {}",
                    winner.name(), loser.name(), conflict.name),
            });
        }
    }

    /// Update all dynamics
    pub fn update(&mut self, delta_days: f32, _game_time: f64) {
        // Update territories
        for territory in self.territories.values_mut() {
            territory.update(delta_days);
        }

        // Update conflicts
        let ended_conflicts: Vec<u32> = self.conflicts
            .iter()
            .filter(|(_, c)| c.should_end())
            .map(|(id, _)| *id)
            .collect();

        for conflict in self.conflicts.values_mut() {
            conflict.update(delta_days);
        }

        // Update power balance
        self.power_balance.recalculate(&self.territories, &self.conflicts);
    }

    /// Get territory at a position
    pub fn get_territory_at(&self, position: [f32; 3]) -> Option<&TerritorialClaim> {
        self.territories.values().find(|t| t.contains(position))
    }

    /// Get controlling faction at a position
    pub fn get_controlling_faction(&self, position: [f32; 3]) -> Option<Faction> {
        self.get_territory_at(position).map(|t| t.claimant)
    }

    /// Check if two factions are at war
    pub fn are_at_war(&self, a: Faction, b: Faction) -> bool {
        self.conflicts.values().any(|c| {
            (c.aggressor == a && c.defender == b) || (c.aggressor == b && c.defender == a)
        })
    }

    /// Get conflicts involving a faction
    pub fn get_faction_conflicts(&self, faction: Faction) -> Vec<&FactionConflict> {
        self.conflicts
            .values()
            .filter(|c| c.aggressor == faction || c.defender == faction)
            .collect()
    }

    /// Form an alliance
    pub fn form_alliance(&mut self, a: Faction, b: Faction, game_time: f64) {
        self.alliances.push(Alliance {
            factions: (a, b),
            formed: game_time,
            alliance_type: AllianceType::Defensive,
        });

        self.history.push(HistoricalEvent {
            timestamp: game_time,
            event_type: HistoricalEventType::AllianceFormed,
            factions_involved: vec![a, b],
            description: format!("{} and {} formed an alliance", a.name(), b.name()),
        });
    }

    /// Establish trade agreement
    pub fn establish_trade(&mut self, a: Faction, b: Faction, game_time: f64) {
        self.trade_agreements.push(TradeAgreement {
            factions: (a, b),
            established: game_time,
            goods_a_to_b: Vec::new(),
            goods_b_to_a: Vec::new(),
        });
    }
}

/// Balance of power between colonial factions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PowerBalance {
    pub english_power: f32,
    pub spanish_power: f32,
    pub french_power: f32,
    pub native_power: f32,
}

impl PowerBalance {
    pub fn recalculate(
        &mut self,
        territories: &HashMap<u32, TerritorialClaim>,
        conflicts: &HashMap<u32, FactionConflict>,
    ) {
        self.english_power = 0.0;
        self.spanish_power = 0.0;
        self.french_power = 0.0;
        self.native_power = 0.0;

        // Territory control
        for territory in territories.values() {
            let value = territory.resources.total_value() * territory.claim_strength;
            match territory.claimant {
                Faction::English => self.english_power += value,
                Faction::Spanish => self.spanish_power += value,
                Faction::French => self.french_power += value,
                Faction::Powhatan | Faction::Tuscarora | Faction::Cherokee
                | Faction::Catawba | Faction::Pamunkey => self.native_power += value,
                _ => {}
            }
        }

        // War penalties
        for conflict in conflicts.values() {
            let penalty = conflict.intensity.spawn_rate_modifier() * 2.0;
            match conflict.aggressor {
                Faction::English => self.english_power -= penalty,
                Faction::Spanish => self.spanish_power -= penalty,
                Faction::French => self.french_power -= penalty,
                _ => self.native_power -= penalty,
            }
            match conflict.defender {
                Faction::English => self.english_power -= penalty,
                Faction::Spanish => self.spanish_power -= penalty,
                Faction::French => self.french_power -= penalty,
                _ => self.native_power -= penalty,
            }
        }
    }

    pub fn dominant_power(&self) -> &'static str {
        let max = self.english_power
            .max(self.spanish_power)
            .max(self.french_power)
            .max(self.native_power);

        if max == self.english_power { "English" }
        else if max == self.spanish_power { "Spanish" }
        else if max == self.french_power { "French" }
        else { "Native Nations" }
    }
}

/// Historical event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEvent {
    pub timestamp: f64,
    pub event_type: HistoricalEventType,
    pub factions_involved: Vec<Faction>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoricalEventType {
    WarDeclared,
    WarEnded,
    TerritoryConquered,
    AllianceFormed,
    AllianceBroken,
    TreatySignaed,
    MassacreOccurred,
    SettlementFounded,
    SettlementDestroyed,
    LeaderDied,
    PlagueSpread,
}

/// Active alliance between factions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alliance {
    pub factions: (Faction, Faction),
    pub formed: f64,
    pub alliance_type: AllianceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllianceType {
    Defensive,
    Offensive,
    Full,
    Marriage,
}

/// Trade agreement between factions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAgreement {
    pub factions: (Faction, Faction),
    pub established: f64,
    pub goods_a_to_b: Vec<String>,
    pub goods_b_to_a: Vec<String>,
}

// ============================================================================
// FACTION EXTENSION
// ============================================================================

impl Faction {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Spanish => "Spanish Crown",
            Self::French => "French Traders",
            Self::English => "English Colonists",
            Self::Aztec => "Aztec Empire",
            Self::Powhatan => "Powhatan Confederacy",
            Self::Tuscarora => "Tuscarora Nation",
            Self::Cherokee => "Cherokee Nation",
            Self::Catawba => "Catawba People",
            Self::Pamunkey => "Pamunkey Tribe",
            Self::Independent => "Independent",
            Self::Wildlife => "Wildlife",
        }
    }

    pub fn is_european(&self) -> bool {
        matches!(self, Self::English | Self::Spanish | Self::French)
    }

    pub fn is_native(&self) -> bool {
        matches!(
            self,
            Self::Powhatan | Self::Tuscarora | Self::Cherokee | Self::Catawba | Self::Pamunkey
        )
    }

    pub fn default_relationship(&self, other: &Faction) -> Standing {
        // Europeans vs Natives
        if self.is_european() && other.is_native() {
            return Standing::Suspicious;
        }
        if self.is_native() && other.is_european() {
            return Standing::Suspicious;
        }

        // Spanish vs English (rivals)
        if (*self == Faction::Spanish && *other == Faction::English)
            || (*self == Faction::English && *other == Faction::Spanish)
        {
            return Standing::Hostile;
        }

        // French generally friendly with natives
        if *self == Faction::French && other.is_native() {
            return Standing::Friendly;
        }
        if self.is_native() && *other == Faction::French {
            return Standing::Friendly;
        }

        // Native nations - complex relationships
        match (self, other) {
            // Powhatan confederacy members
            (Faction::Powhatan, Faction::Pamunkey) => Standing::Allied,
            (Faction::Pamunkey, Faction::Powhatan) => Standing::Allied,

            // Traditional rivalries
            (Faction::Cherokee, Faction::Catawba) => Standing::Hostile,
            (Faction::Catawba, Faction::Cherokee) => Standing::Hostile,

            _ => Standing::Neutral,
        }
    }
}
