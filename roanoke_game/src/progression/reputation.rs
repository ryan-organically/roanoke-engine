//! Reputation System
//!
//! Tracks player standing with various factions and NPCs.

use serde::{Deserialize, Serialize};

/// Faction types in the game world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    // Native Villages
    NativeVillage(u32), // Village ID
    NativeCouncil,      // Overall native reputation

    // Colonial
    EnglishSettlers,
    SpanishExplorers,
    FrenchTraders,

    // Specialized groups
    Hunters,
    Traders,
    Shamans,
    Warriors,

    // Wildlife (affects animal behavior)
    Wildlife,
}

/// Reputation level thresholds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReputationLevel {
    Hated,      // -1000 to -500
    Hostile,    // -500 to -200
    Unfriendly, // -200 to -50
    Neutral,    // -50 to 50
    Friendly,   // 50 to 200
    Respected,  // 200 to 500
    Honored,    // 500 to 1000
    Legendary,  // 1000+
}

impl ReputationLevel {
    pub fn from_value(value: i32) -> Self {
        match value {
            v if v <= -500 => Self::Hated,
            v if v <= -200 => Self::Hostile,
            v if v <= -50 => Self::Unfriendly,
            v if v <= 50 => Self::Neutral,
            v if v <= 200 => Self::Friendly,
            v if v <= 500 => Self::Respected,
            v if v <= 1000 => Self::Honored,
            _ => Self::Legendary,
        }
    }

    /// Get description of this reputation level
    pub fn description(&self) -> &'static str {
        match self {
            Self::Hated => "They want you dead",
            Self::Hostile => "Attacks on sight",
            Self::Unfriendly => "Won't trade or help",
            Self::Neutral => "Cautious but fair",
            Self::Friendly => "Willing to trade",
            Self::Respected => "Offers quests and training",
            Self::Honored => "Trusted ally",
            Self::Legendary => "Hero of the people",
        }
    }

    /// Get trade price modifier
    pub fn trade_modifier(&self) -> f32 {
        match self {
            Self::Hated | Self::Hostile => 0.0, // Won't trade
            Self::Unfriendly => 1.5, // 50% markup
            Self::Neutral => 1.2,    // 20% markup
            Self::Friendly => 1.0,   // Normal prices
            Self::Respected => 0.9,  // 10% discount
            Self::Honored => 0.8,    // 20% discount
            Self::Legendary => 0.7,  // 30% discount
        }
    }
}

/// Reputation with a specific faction
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Reputation {
    pub value: i32,
    pub level: ReputationLevel,
    /// Recent actions that affected reputation
    pub history: Vec<ReputationChange>,
    /// Maximum historical value (for achievements)
    pub max_reached: i32,
    /// Minimum historical value
    pub min_reached: i32,
}

impl Default for ReputationLevel {
    fn default() -> Self {
        Self::Neutral
    }
}

/// Record of a reputation change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChange {
    pub delta: i32,
    pub reason: String,
    pub timestamp: f64, // In-game time
}

impl Reputation {
    pub fn new() -> Self {
        Self::default()
    }

    /// Modify reputation value
    pub fn modify(&mut self, delta: i32) {
        self.value += delta;
        self.level = ReputationLevel::from_value(self.value);

        if self.value > self.max_reached {
            self.max_reached = self.value;
        }
        if self.value < self.min_reached {
            self.min_reached = self.value;
        }
    }

    /// Modify with reason tracking
    pub fn modify_with_reason(&mut self, delta: i32, reason: &str, game_time: f64) {
        self.modify(delta);
        self.history.push(ReputationChange {
            delta,
            reason: reason.to_string(),
            timestamp: game_time,
        });

        // Keep only last 20 changes
        if self.history.len() > 20 {
            self.history.remove(0);
        }
    }

    /// Check if can perform action requiring minimum level
    pub fn can_perform(&self, required: ReputationLevel) -> bool {
        self.level >= required
    }

    /// Get progress to next level (0.0 - 1.0)
    pub fn progress_to_next(&self) -> f32 {
        let current_min = match self.level {
            ReputationLevel::Hated => -1000,
            ReputationLevel::Hostile => -500,
            ReputationLevel::Unfriendly => -200,
            ReputationLevel::Neutral => -50,
            ReputationLevel::Friendly => 50,
            ReputationLevel::Respected => 200,
            ReputationLevel::Honored => 500,
            ReputationLevel::Legendary => 1000,
        };

        let next_min = match self.level {
            ReputationLevel::Hated => -500,
            ReputationLevel::Hostile => -200,
            ReputationLevel::Unfriendly => -50,
            ReputationLevel::Neutral => 50,
            ReputationLevel::Friendly => 200,
            ReputationLevel::Respected => 500,
            ReputationLevel::Honored => 1000,
            ReputationLevel::Legendary => 2000,
        };

        let range = (next_min - current_min) as f32;
        let progress = (self.value - current_min) as f32;
        (progress / range).clamp(0.0, 1.0)
    }
}

/// Reputation effects for specific actions
#[derive(Debug, Clone)]
pub struct ReputationAction {
    pub action: &'static str,
    pub faction_effects: &'static [(Faction, i32)],
}

/// Common reputation-affecting actions
pub fn get_action_effects(action: &str) -> Vec<(Faction, i32)> {
    match action {
        // Positive actions
        "complete_quest" => vec![(Faction::NativeCouncil, 25)],
        "trade_fair" => vec![(Faction::Traders, 5)],
        "defend_village" => vec![(Faction::NativeCouncil, 50), (Faction::Warriors, 30)],
        "heal_npc" => vec![(Faction::NativeCouncil, 10)],
        "gift_to_shaman" => vec![(Faction::Shamans, 20)],
        "discover_fossil" => vec![(Faction::Shamans, 5)],
        "hunt_predator_near_village" => vec![(Faction::NativeCouncil, 15)],

        // Negative actions
        "attack_npc" => vec![(Faction::NativeCouncil, -100), (Faction::Warriors, -50)],
        "steal" => vec![(Faction::NativeCouncil, -30), (Faction::Traders, -20)],
        "kill_npc" => vec![(Faction::NativeCouncil, -500)],
        "desecrate_sacred_site" => vec![(Faction::Shamans, -100), (Faction::NativeCouncil, -50)],
        "overhunt_area" => vec![(Faction::NativeCouncil, -20), (Faction::Wildlife, -30)],

        // Neutral/situational
        "enter_territory" => vec![], // Depends on permission
        "trade_unfair" => vec![(Faction::Traders, -10)],

        _ => vec![],
    }
}

/// NPC disposition based on player reputation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcDisposition {
    Attacking,   // Will attack on sight
    Fleeing,     // Runs from player
    Hostile,     // Won't interact, threatens
    Wary,        // Limited interaction
    Neutral,     // Normal interaction
    Friendly,    // Offers help, lower prices
    Devoted,     // Will follow/defend player
}

impl NpcDisposition {
    /// Get disposition based on reputation level
    pub fn from_reputation(level: ReputationLevel, npc_role: &str) -> Self {
        match (level, npc_role) {
            (ReputationLevel::Hated, "warrior") => Self::Attacking,
            (ReputationLevel::Hated, _) => Self::Fleeing,
            (ReputationLevel::Hostile, "warrior") => Self::Attacking,
            (ReputationLevel::Hostile, _) => Self::Hostile,
            (ReputationLevel::Unfriendly, _) => Self::Wary,
            (ReputationLevel::Neutral, _) => Self::Neutral,
            (ReputationLevel::Friendly, _) => Self::Friendly,
            (ReputationLevel::Respected, _) => Self::Friendly,
            (ReputationLevel::Honored, _) => Self::Devoted,
            (ReputationLevel::Legendary, _) => Self::Devoted,
        }
    }
}
