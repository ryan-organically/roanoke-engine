//! Faction Manager
//!
//! Runtime management of faction state, player reputation, and faction interactions.

use super::faction::{
    get_faction_abilities, get_faction_traits, get_faction_weapons, Faction,
    FactionAbility, FactionRelationshipMatrix, FactionTrait, FactionWeapon, Standing,
    TraitEffect,
};
use super::faction_skills::{
    get_faction_skill_tree, FactionSkill, FactionSkillId, PlayerFactionSkills,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// PLAYER FACTION STATE
// ============================================================================

/// Player's reputation with a specific faction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionReputation {
    pub faction: Faction,
    pub reputation: i32,
    pub standing: Standing,
    pub last_interaction: f64, // game time
    pub history: Vec<ReputationEvent>,
    pub max_reputation: i32,
    pub min_reputation: i32,
}

impl FactionReputation {
    pub fn new(faction: Faction) -> Self {
        Self {
            faction,
            reputation: 0,
            standing: Standing::Neutral,
            last_interaction: 0.0,
            history: Vec::new(),
            max_reputation: 0,
            min_reputation: 0,
        }
    }

    /// Modify reputation by delta amount
    pub fn modify(&mut self, delta: i32, reason: &str, game_time: f64) {
        self.reputation += delta;
        self.standing = Standing::from_reputation(self.reputation);
        self.last_interaction = game_time;

        // Track extremes
        if self.reputation > self.max_reputation {
            self.max_reputation = self.reputation;
        }
        if self.reputation < self.min_reputation {
            self.min_reputation = self.reputation;
        }

        // Record event
        self.history.push(ReputationEvent {
            delta,
            reason: reason.to_string(),
            timestamp: game_time,
            resulting_standing: self.standing,
        });

        // Keep only last 50 events
        if self.history.len() > 50 {
            self.history.remove(0);
        }
    }

    /// Check if player can perform action requiring minimum standing
    pub fn can_perform(&self, required: Standing) -> bool {
        self.standing >= required
    }

    /// Get progress to next standing level (0.0 - 1.0)
    pub fn progress_to_next(&self) -> f32 {
        let (current_min, next_min) = match self.standing {
            Standing::War => (-2000, -1000),
            Standing::Hostile => (-1000, -500),
            Standing::Suspicious => (-500, -100),
            Standing::Neutral => (-100, 100),
            Standing::Friendly => (100, 500),
            Standing::Allied => (500, 1000),
            Standing::BloodBond => (1000, 2000),
        };

        let range = (next_min - current_min) as f32;
        let progress = (self.reputation - current_min) as f32;
        (progress / range).clamp(0.0, 1.0)
    }
}

/// Record of a reputation change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub delta: i32,
    pub reason: String,
    pub timestamp: f64,
    pub resulting_standing: Standing,
}

// ============================================================================
// FACTION MANAGER
// ============================================================================

/// Central manager for all faction-related state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionManager {
    /// Player reputation with each faction
    pub reputations: HashMap<Faction, FactionReputation>,
    /// Inter-faction relationship matrix
    pub relationships: FactionRelationshipMatrix,
    /// Player's unlocked faction skills
    pub skills: PlayerFactionSkills,
    /// Player's active faction (primary allegiance)
    pub primary_faction: Option<Faction>,
    /// Cached active trait effects
    #[serde(skip)]
    cached_trait_effects: Option<Vec<TraitEffect>>,
}

impl Default for FactionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FactionManager {
    /// Create a new faction manager with default relationships
    pub fn new() -> Self {
        let mut reputations = HashMap::new();

        // Initialize reputation for all playable factions
        for &faction in Faction::all_playable() {
            reputations.insert(faction, FactionReputation::new(faction));
        }

        Self {
            reputations,
            relationships: FactionRelationshipMatrix::new(),
            skills: PlayerFactionSkills::new(),
            primary_faction: None,
            cached_trait_effects: None,
        }
    }

    /// Create with English as default primary faction
    pub fn new_english_start() -> Self {
        let mut manager = Self::new();
        manager.primary_faction = Some(Faction::English);

        // Start with neutral-friendly reputation with English
        if let Some(rep) = manager.reputations.get_mut(&Faction::English) {
            rep.modify(150, "Starting colonist", 0.0);
        }

        // Add starting skills
        manager.skills.add_points(Faction::English, 5);

        manager
    }

    // ========== REPUTATION MANAGEMENT ==========

    /// Get player's standing with a faction
    pub fn get_standing(&self, faction: Faction) -> Standing {
        self.reputations
            .get(&faction)
            .map(|r| r.standing)
            .unwrap_or(Standing::Neutral)
    }

    /// Get player's reputation value with a faction
    pub fn get_reputation(&self, faction: Faction) -> i32 {
        self.reputations
            .get(&faction)
            .map(|r| r.reputation)
            .unwrap_or(0)
    }

    /// Modify reputation with a faction
    pub fn modify_reputation(&mut self, faction: Faction, delta: i32, reason: &str, game_time: f64) {
        if let Some(rep) = self.reputations.get_mut(&faction) {
            rep.modify(delta, reason, game_time);
            self.invalidate_cache();
        }
    }

    /// Apply reputation effects from an action
    pub fn apply_action(&mut self, action: FactionAction, game_time: f64) {
        for (faction, delta) in action.effects() {
            self.modify_reputation(faction, delta, action.reason(), game_time);
        }
    }

    // ========== FACTION RELATIONSHIPS ==========

    /// Get relationship between two factions
    pub fn get_faction_relationship(&self, a: Faction, b: Faction) -> Standing {
        self.relationships.get_standing(a, b)
    }

    /// Check if player's standing with one faction affects another
    pub fn get_relationship_modifier(&self, from: Faction, to: Faction) -> f32 {
        let relationship = self.relationships.get_standing(from, to);

        // Allied factions share reputation effects
        match relationship {
            Standing::War => -0.5,        // Helping enemies hurts
            Standing::Hostile => -0.3,
            Standing::Suspicious => -0.1,
            Standing::Neutral => 0.0,
            Standing::Friendly => 0.1,
            Standing::Allied => 0.3,
            Standing::BloodBond => 0.5,   // Helping allies helps
        }
    }

    /// Get all factions hostile to player
    pub fn get_hostile_factions(&self) -> Vec<Faction> {
        self.reputations
            .iter()
            .filter(|(_, rep)| rep.standing.attacks_on_sight())
            .map(|(f, _)| *f)
            .collect()
    }

    /// Get all factions allied with player
    pub fn get_allied_factions(&self) -> Vec<Faction> {
        self.reputations
            .iter()
            .filter(|(_, rep)| matches!(rep.standing, Standing::Allied | Standing::BloodBond))
            .map(|(f, _)| *f)
            .collect()
    }

    // ========== TRAIT EFFECTS ==========

    /// Get all active trait effects from primary faction and allied factions
    pub fn get_active_traits(&self) -> Vec<FactionTrait> {
        let mut traits = Vec::new();

        // Primary faction traits
        if let Some(primary) = self.primary_faction {
            traits.extend(get_faction_traits(primary));
        }

        traits
    }

    /// Calculate total stat modifier from traits
    pub fn get_stat_modifier(&self, stat: super::faction::StatType) -> f32 {
        let mut total = 1.0;

        for trait_info in self.get_active_traits() {
            if let TraitEffect::StatBonus { stat: trait_stat, bonus } = trait_info.effect {
                if trait_stat == stat {
                    total += bonus;
                }
            }
        }

        total
    }

    /// Get trade price modifier from all sources
    pub fn get_trade_modifier(&self, trading_faction: Faction) -> f32 {
        let mut modifier = 1.0;

        // Standing modifier
        let standing = self.get_standing(trading_faction);
        modifier *= standing.trade_multiplier();

        // Trait modifiers
        for trait_info in self.get_active_traits() {
            if let TraitEffect::TradeModifier { multiplier } = trait_info.effect {
                modifier *= multiplier;
            }
        }

        modifier
    }

    // ========== SKILL MANAGEMENT ==========

    /// Check if player has a specific faction skill
    pub fn has_skill(&self, skill_id: FactionSkillId) -> bool {
        self.skills.has_skill(skill_id)
    }

    /// Check if player can unlock a skill
    pub fn can_unlock_skill(&self, skill_id: FactionSkillId) -> bool {
        let faction = skill_id.faction();
        let standing = self.get_standing(faction);

        let skills = get_faction_skill_tree(faction);
        if let Some(skill) = skills.iter().find(|s| s.id == skill_id) {
            self.skills.can_unlock(skill, standing)
        } else {
            false
        }
    }

    /// Unlock a skill
    pub fn unlock_skill(&mut self, skill_id: FactionSkillId) -> Result<(), SkillUnlockError> {
        let faction = skill_id.faction();
        let standing = self.get_standing(faction);

        let skills = get_faction_skill_tree(faction);
        let skill = skills
            .iter()
            .find(|s| s.id == skill_id)
            .ok_or(SkillUnlockError::SkillNotFound)?;

        if standing < skill.required_standing {
            return Err(SkillUnlockError::InsufficientStanding);
        }

        for prereq in &skill.prerequisites {
            if !self.skills.has_skill(*prereq) {
                return Err(SkillUnlockError::MissingPrerequisite(*prereq));
            }
        }

        let points = self.skills.skill_points.get(&faction).copied().unwrap_or(0);
        if points < skill.skill_point_cost {
            return Err(SkillUnlockError::InsufficientPoints);
        }

        if self.skills.has_skill(skill_id) {
            return Err(SkillUnlockError::AlreadyUnlocked);
        }

        // All checks passed, unlock
        self.skills.unlock_skill(skill);
        self.invalidate_cache();

        Ok(())
    }

    /// Add skill points for a faction
    pub fn add_skill_points(&mut self, faction: Faction, points: u32) {
        self.skills.add_points(faction, points);
    }

    /// Get available skills to unlock for a faction
    pub fn get_available_skills(&self, faction: Faction) -> Vec<FactionSkill> {
        let standing = self.get_standing(faction);
        let all_skills = get_faction_skill_tree(faction);

        all_skills
            .into_iter()
            .filter(|skill| self.skills.can_unlock(skill, standing))
            .collect()
    }

    // ========== WEAPONS & ABILITIES ==========

    /// Get weapons available to player from factions
    pub fn get_available_weapons(&self) -> Vec<FactionWeapon> {
        let mut weapons = Vec::new();

        for &faction in Faction::all_playable() {
            let standing = self.get_standing(faction);
            for weapon in get_faction_weapons(faction) {
                if standing >= weapon.required_standing {
                    weapons.push(weapon);
                }
            }
        }

        weapons
    }

    /// Get abilities available to player from factions
    pub fn get_available_abilities(&self) -> Vec<FactionAbility> {
        let mut abilities = Vec::new();

        for &faction in Faction::all_playable() {
            let standing = self.get_standing(faction);
            for ability in get_faction_abilities(faction) {
                if standing >= ability.required_standing {
                    // Also check skill tier requirement
                    let highest_tier = self.skills.highest_tier(faction);
                    if highest_tier >= ability.skill_tier {
                        abilities.push(ability);
                    }
                }
            }
        }

        abilities
    }

    // ========== PRIMARY FACTION ==========

    /// Set player's primary faction
    pub fn set_primary_faction(&mut self, faction: Faction) -> Result<(), FactionError> {
        let standing = self.get_standing(faction);

        if standing < Standing::Friendly {
            return Err(FactionError::InsufficientStanding);
        }

        self.primary_faction = Some(faction);
        self.skills.primary_faction = Some(faction);
        self.invalidate_cache();

        Ok(())
    }

    /// Check if joining a faction would cause conflicts
    pub fn check_faction_conflicts(&self, faction: Faction) -> Vec<(Faction, Standing)> {
        let mut conflicts = Vec::new();

        for &other in Faction::all_playable() {
            if other == faction {
                continue;
            }

            let relationship = self.relationships.get_standing(faction, other);
            let player_standing = self.get_standing(other);

            // Joining a faction at war with an allied faction is a conflict
            if relationship == Standing::War && player_standing >= Standing::Allied {
                conflicts.push((other, relationship));
            }
        }

        conflicts
    }

    // ========== INTERNAL ==========

    fn invalidate_cache(&mut self) {
        self.cached_trait_effects = None;
    }
}

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillUnlockError {
    SkillNotFound,
    InsufficientStanding,
    MissingPrerequisite(FactionSkillId),
    InsufficientPoints,
    AlreadyUnlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionError {
    InsufficientStanding,
    ConflictingAlliances,
}

// ============================================================================
// FACTION ACTIONS
// ============================================================================

/// Predefined actions that affect faction reputation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionAction {
    // General actions
    CompleteQuest(Faction),
    FailQuest(Faction),
    TradeFairly(Faction),
    TradeUnfairly(Faction),
    GiftItem(Faction),
    StealFrom(Faction),
    AttackMember(Faction),
    KillMember(Faction),
    DefendSettlement(Faction),
    HealMember(Faction),

    // Specific actions
    DesecrateSacredSite,
    DiscoverFossil,
    HuntPredatorNearVillage(Faction),
    OverhuntArea,
    RescueCaptive(Faction),
    BetraySecrets(Faction),
    CaptureEnemy(Faction),
}

impl FactionAction {
    /// Get reputation effects for this action
    pub fn effects(&self) -> Vec<(Faction, i32)> {
        match self {
            Self::CompleteQuest(f) => vec![(*f, 50)],
            Self::FailQuest(f) => vec![(*f, -25)],
            Self::TradeFairly(f) => vec![(*f, 5)],
            Self::TradeUnfairly(f) => vec![(*f, -10)],
            Self::GiftItem(f) => vec![(*f, 15)],
            Self::StealFrom(f) => vec![(*f, -50)],
            Self::AttackMember(f) => vec![(*f, -100)],
            Self::KillMember(f) => vec![(*f, -200)],
            Self::DefendSettlement(f) => vec![(*f, 100)],
            Self::HealMember(f) => vec![(*f, 20)],
            Self::DesecrateSacredSite => vec![
                (Faction::Powhatan, -100),
                (Faction::Cherokee, -50),
                (Faction::Tuscarora, -50),
            ],
            Self::DiscoverFossil => vec![
                (Faction::Powhatan, 5),
            ],
            Self::HuntPredatorNearVillage(f) => vec![(*f, 30)],
            Self::OverhuntArea => vec![
                (Faction::Powhatan, -20),
                (Faction::Wildlife, -30),
            ],
            Self::RescueCaptive(f) => vec![(*f, 150)],
            Self::BetraySecrets(f) => vec![(*f, -500)],
            Self::CaptureEnemy(f) => vec![(*f, 25)],
        }
    }

    /// Get description of this action
    pub fn reason(&self) -> &'static str {
        match self {
            Self::CompleteQuest(_) => "Completed quest",
            Self::FailQuest(_) => "Failed quest",
            Self::TradeFairly(_) => "Fair trade",
            Self::TradeUnfairly(_) => "Unfair trade",
            Self::GiftItem(_) => "Gifted valuable item",
            Self::StealFrom(_) => "Theft",
            Self::AttackMember(_) => "Attacked faction member",
            Self::KillMember(_) => "Killed faction member",
            Self::DefendSettlement(_) => "Defended settlement",
            Self::HealMember(_) => "Healed faction member",
            Self::DesecrateSacredSite => "Desecrated sacred site",
            Self::DiscoverFossil => "Discovered archaeological find",
            Self::HuntPredatorNearVillage(_) => "Killed predator near village",
            Self::OverhuntArea => "Overhunted area",
            Self::RescueCaptive(_) => "Rescued captive",
            Self::BetraySecrets(_) => "Betrayed faction secrets",
            Self::CaptureEnemy(_) => "Captured enemy alive",
        }
    }
}

// ============================================================================
// NPC DISPOSITION
// ============================================================================

/// How an NPC should behave toward the player
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcDisposition {
    /// Will attack immediately
    Attacking,
    /// Will flee from player
    Fleeing,
    /// Hostile but won't attack first
    Hostile,
    /// Cautious, limited interaction
    Wary,
    /// Normal interaction
    Neutral,
    /// Friendly, offers help
    Friendly,
    /// Devoted, will defend player
    Devoted,
}

impl NpcDisposition {
    /// Get disposition based on faction standing and NPC role
    pub fn from_standing(standing: Standing, npc_role: NpcRole) -> Self {
        match (standing, npc_role) {
            (Standing::War, NpcRole::Warrior) => Self::Attacking,
            (Standing::War, _) => Self::Fleeing,
            (Standing::Hostile, NpcRole::Warrior) => Self::Attacking,
            (Standing::Hostile, _) => Self::Hostile,
            (Standing::Suspicious, _) => Self::Wary,
            (Standing::Neutral, _) => Self::Neutral,
            (Standing::Friendly, _) => Self::Friendly,
            (Standing::Allied, _) => Self::Friendly,
            (Standing::BloodBond, _) => Self::Devoted,
        }
    }

    /// Check if NPC will attack on sight
    pub fn will_attack(&self) -> bool {
        matches!(self, Self::Attacking)
    }

    /// Check if NPC will help player in combat
    pub fn will_help(&self) -> bool {
        matches!(self, Self::Devoted)
    }
}

/// NPC roles affecting disposition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcRole {
    Warrior,
    Chief,
    Trader,
    Farmer,
    Shaman,
    Elder,
    Child,
    Villager,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faction_manager_creation() {
        let manager = FactionManager::new();

        // All factions should start neutral
        for &faction in Faction::all_playable() {
            assert_eq!(manager.get_standing(faction), Standing::Neutral);
        }
    }

    #[test]
    fn test_reputation_modification() {
        let mut manager = FactionManager::new();

        manager.modify_reputation(Faction::French, 200, "Test", 0.0);
        assert_eq!(manager.get_standing(Faction::French), Standing::Friendly);

        manager.modify_reputation(Faction::French, 500, "Test", 1.0);
        assert_eq!(manager.get_standing(Faction::French), Standing::Allied);
    }

    #[test]
    fn test_english_start() {
        let manager = FactionManager::new_english_start();

        assert_eq!(manager.primary_faction, Some(Faction::English));
        assert_eq!(manager.get_standing(Faction::English), Standing::Friendly);
    }

    #[test]
    fn test_action_effects() {
        let mut manager = FactionManager::new();

        manager.apply_action(FactionAction::DefendSettlement(Faction::Powhatan), 0.0);
        assert!(manager.get_reputation(Faction::Powhatan) > 0);
    }

    #[test]
    fn test_hostile_faction_list() {
        let mut manager = FactionManager::new();

        manager.modify_reputation(Faction::Spanish, -1500, "Test", 0.0);

        let hostile = manager.get_hostile_factions();
        assert!(hostile.contains(&Faction::Spanish));
    }

    #[test]
    fn test_npc_disposition() {
        assert_eq!(
            NpcDisposition::from_standing(Standing::War, NpcRole::Warrior),
            NpcDisposition::Attacking
        );
        assert_eq!(
            NpcDisposition::from_standing(Standing::War, NpcRole::Farmer),
            NpcDisposition::Fleeing
        );
        assert_eq!(
            NpcDisposition::from_standing(Standing::Friendly, NpcRole::Trader),
            NpcDisposition::Friendly
        );
    }
}
