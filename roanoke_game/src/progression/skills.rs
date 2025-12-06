//! Skill Tree System
//!
//! Implements hunting and archaeology skill trees with unlock tracking.

use serde::{Deserialize, Serialize};

/// Skill unlock notification
#[derive(Debug, Clone)]
pub struct SkillUnlock {
    pub skill_name: String,
    pub tree: SkillTree,
    pub description: String,
    pub tier: u8,
}

/// Skill tree type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillTree {
    Hunting,
    Archaeology,
    Survival,
    Crafting,
    Social,
}

/// Hunting skill tree - all skills from the spec
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HuntingSkills {
    pub points: u32,

    // Tier 1 - Foundation
    pub basic_tracker: bool,

    // Tier 2 - Prey Specialization
    pub boar_hunter: bool,
    pub deer_stalker: bool,

    // Tier 3 - Predator Awareness
    pub wolf_tracker: bool,
    pub serpent_eye: bool,

    // Tier 4 - Intermediate
    pub predator_sense: bool,
    pub prey_instinct: bool,

    // Tier 5 - Advanced
    pub wilderness_scout: bool,

    // Tier 6 - Specialization
    pub big_game_hunter: bool,
    pub trap_setter: bool,

    // Tier 7 - Master
    pub beast_slayer: bool,
    pub shadow_hunter: bool,
    pub snare_master: bool,
    pub lure_crafter: bool,

    // Tier 8 - Ultimate
    pub apex_predator: bool,
    pub master_trapper: bool,

    // Tier 9 - Legendary
    pub legendary_hunter: bool,

    // Spirit companion (from Legendary Hunter)
    pub spirit_animal: Option<SpiritAnimal>,

    // Wolf companion (from Apex Predator)
    pub wolf_companion: Option<WolfCompanion>,
}

impl HuntingSkills {
    /// Initialize with basic tracker unlocked
    pub fn new() -> Self {
        Self {
            basic_tracker: true,
            ..Default::default()
        }
    }

    /// Get the current tier based on unlocked skills
    pub fn current_tier(&self) -> u8 {
        if self.legendary_hunter { 9 }
        else if self.apex_predator || self.master_trapper { 8 }
        else if self.beast_slayer || self.shadow_hunter || self.snare_master || self.lure_crafter { 7 }
        else if self.big_game_hunter || self.trap_setter { 6 }
        else if self.wilderness_scout { 5 }
        else if self.predator_sense || self.prey_instinct { 4 }
        else if self.wolf_tracker || self.serpent_eye { 3 }
        else if self.boar_hunter || self.deer_stalker { 2 }
        else { 1 }
    }

    /// Get points required for next tier
    pub fn points_for_tier(tier: u8) -> u32 {
        match tier {
            1 => 0,
            2 => 100,
            3 => 200,
            4 => 400,
            5 => 750,
            6 => 1250,
            7 => 2000,
            8 => 3500,
            9 => 6000,
            _ => u32::MAX,
        }
    }

    /// Check if player can unlock a skill
    pub fn can_unlock(&self, skill: &str) -> bool {
        let tier = self.current_tier();
        let points = self.points;

        match skill {
            "boar_hunter" => !self.boar_hunter && self.basic_tracker,
            "deer_stalker" => !self.deer_stalker && self.basic_tracker,
            "wolf_tracker" => !self.wolf_tracker && self.basic_tracker && points >= 200,
            "serpent_eye" => !self.serpent_eye && self.basic_tracker && points >= 200,
            "predator_sense" => !self.predator_sense && self.wolf_tracker && self.serpent_eye,
            "prey_instinct" => !self.prey_instinct && self.boar_hunter && self.deer_stalker,
            "wilderness_scout" => !self.wilderness_scout && self.predator_sense && self.prey_instinct,
            "big_game_hunter" => !self.big_game_hunter && self.wilderness_scout,
            "trap_setter" => !self.trap_setter && self.wilderness_scout,
            "beast_slayer" => !self.beast_slayer && self.big_game_hunter,
            "shadow_hunter" => !self.shadow_hunter && self.big_game_hunter,
            "snare_master" => !self.snare_master && self.trap_setter,
            "lure_crafter" => !self.lure_crafter && self.trap_setter,
            "apex_predator" => !self.apex_predator && self.beast_slayer && self.shadow_hunter,
            "master_trapper" => !self.master_trapper && self.snare_master && self.lure_crafter,
            "legendary_hunter" => !self.legendary_hunter && self.apex_predator && self.master_trapper,
            _ => false,
        }
    }

    /// Get damage modifier for stealth attacks
    pub fn stealth_damage_modifier(&self) -> f32 {
        if self.shadow_hunter {
            3.0 // 3x damage
        } else {
            1.5 // Base stealth bonus
        }
    }

    /// Get detection range reduction
    pub fn detection_reduction(&self) -> f32 {
        let mut reduction: f32 = 0.0;

        if self.basic_tracker {
            reduction += 0.25;
        }
        if self.deer_stalker {
            reduction += 0.30;
        }
        if self.shadow_hunter {
            reduction += 0.50;
        }

        reduction.min(0.80) // Max 80% reduction
    }

    /// Get trap damage bonus
    pub fn trap_damage_bonus(&self) -> f32 {
        let mut bonus = 1.0;

        if self.trap_setter {
            bonus += 0.50;
        }
        if self.snare_master {
            bonus += 0.50;
        }

        bonus
    }

    /// Get effective level based on skills and points
    pub fn effective_level(&self) -> u32 {
        let tier = self.current_tier() as u32;
        let point_bonus = (self.points / 500).min(10);
        tier + point_bonus
    }

    /// Calculate luck bonus for rare drops/finds
    pub fn calculate_luck_bonus(&self) -> f32 {
        let mut bonus = 0.0;

        // Each tier adds luck
        bonus += self.current_tier() as f32 * 0.02;

        // Specific skill bonuses
        if self.wilderness_scout {
            bonus += 0.05;
        }
        if self.predator_sense {
            bonus += 0.03;
        }
        if self.prey_instinct {
            bonus += 0.03;
        }
        if self.legendary_hunter {
            bonus += 0.10;
        }

        bonus.min(0.5) // Cap at 50% bonus
    }
}

/// Archaeology skill tree
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchaeologySkills {
    pub points: u32,

    // Tier 1
    pub novice_digger: bool,

    // Tier 2
    pub megalodon_hunter: bool,
    pub mastodon_seeker: bool,

    // Tier 3
    pub curious_eye: bool,

    // Tier 4
    pub field_scholar: bool,
    pub keen_collector: bool,

    // Tier 5
    pub bone_reader: bool,
    pub stone_sage: bool,
    pub fossil_smith: bool,
    pub curio_trader: bool,

    // Tier 6
    pub ancient_lore: bool,
    pub relic_artisan: bool,

    // Tier 7
    pub master_antiquarian: bool,

    // Spirit companions from max tier
    pub mastodon_spirit: bool,
    pub megalodon_blessing: bool,
}

impl ArchaeologySkills {
    pub fn new() -> Self {
        Self {
            novice_digger: true,
            ..Default::default()
        }
    }

    /// Get extraction success modifier
    pub fn extraction_modifier(&self) -> f32 {
        if self.master_antiquarian { 1.0 }
        else if self.ancient_lore { 0.95 }
        else if self.bone_reader || self.stone_sage { 0.85 }
        else if self.field_scholar { 0.70 }
        else if self.curious_eye { 0.55 }
        else if self.megalodon_hunter || self.mastodon_seeker { 0.40 }
        else { 0.25 }
    }

    /// Get fossil quality bonus
    pub fn quality_bonus(&self) -> i32 {
        let mut bonus = 0;

        if self.field_scholar { bonus += 1; }
        if self.ancient_lore { bonus += 2; }
        if self.relic_artisan { bonus += 1; }

        bonus
    }

    /// Get trade value modifier
    pub fn trade_value_modifier(&self) -> f32 {
        let mut modifier = 1.0;

        if self.keen_collector { modifier += 0.25; }
        if self.curio_trader { modifier += 0.50; }

        modifier
    }

    /// Check if player can see dig sites of given depth
    pub fn can_see_depth(&self, depth: DigSiteDepth) -> bool {
        match depth {
            DigSiteDepth::Surface | DigSiteDepth::Shallow => self.novice_digger,
            DigSiteDepth::Standard => self.curious_eye,
            DigSiteDepth::Deep => self.field_scholar,
            DigSiteDepth::Legendary => self.ancient_lore,
        }
    }
}

/// Dig site depth types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DigSiteDepth {
    Surface,   // 0 ft
    Shallow,   // 1-2 ft
    Standard,  // 2-3 ft
    Deep,      // 3-4 ft
    Legendary, // 5+ ft
}

/// Spirit animal types (from Legendary Hunter)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpiritAnimal {
    Bear,     // +50% health
    Cougar,   // +50% speed
    Wolf,     // +50% damage
    Serpent,  // Poison immunity
    Alligator, // Water breathing
}

impl SpiritAnimal {
    /// Get the passive bonus description
    pub fn passive_bonus(&self) -> &'static str {
        match self {
            Self::Bear => "+50% health",
            Self::Cougar => "+50% movement speed",
            Self::Wolf => "+50% damage",
            Self::Serpent => "Poison immunity",
            Self::Alligator => "Water breathing, swim speed",
        }
    }

    /// Get the active ability name
    pub fn active_ability(&self) -> &'static str {
        match self {
            Self::Bear => "Roar: Stun all enemies 3s",
            Self::Cougar => "Pounce: Teleport to target",
            Self::Wolf => "Pack Call: Summon wolf spirits",
            Self::Serpent => "Venomous Strike: DoT attack",
            Self::Alligator => "Death Roll: Massive damage",
        }
    }
}

/// Wolf companion data (from Apex Predator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolfCompanion {
    pub name: String,
    pub health: f32,
    pub max_health: f32,
    pub loyalty: f32, // 0.0 - 1.0
    pub state: CompanionState,
}

/// Companion behavioral state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompanionState {
    Following,
    Attacking,
    Guarding,
    Resting,
    Hunting,
}

impl WolfCompanion {
    pub fn new(name: String) -> Self {
        Self {
            name,
            health: 80.0,
            max_health: 80.0,
            loyalty: 0.5,
            state: CompanionState::Following,
        }
    }

    /// Feed the companion to increase loyalty
    pub fn feed(&mut self) {
        self.loyalty = (self.loyalty + 0.1).min(1.0);
        self.health = (self.health + 20.0).min(self.max_health);
    }

    /// Check if companion will obey commands
    pub fn will_obey(&self) -> bool {
        self.loyalty >= 0.3
    }

    /// Get damage bonus from loyalty
    pub fn damage_bonus(&self) -> f32 {
        20.0 * self.loyalty
    }
}

/// Trap type for crafting/placement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl TrapType {
    /// Get base damage
    pub fn base_damage(&self) -> f32 {
        match self {
            Self::Snare => 0.0, // Capture only
            Self::Deadfall => 50.0,
            Self::PitTrap => 75.0,
            Self::JawTrap => 40.0,
            Self::NetTrap => 0.0, // Capture only
            Self::BearTrap => 60.0,
            Self::VenomSnare => 30.0, // + poison
            Self::AlarmTrap => 0.0, // Alert only
            Self::ComboTrap => 50.0,
        }
    }

    /// Get required materials
    pub fn materials(&self) -> &'static [(&'static str, u32)] {
        match self {
            Self::Snare => &[("rope", 1), ("stake", 1)],
            Self::Deadfall => &[("log", 2), ("rope", 1), ("bait", 1)],
            Self::PitTrap => &[("shovel", 1), ("stake", 4)],
            Self::JawTrap => &[("iron", 2), ("spring", 1)],
            Self::NetTrap => &[("rope", 3), ("frame", 1)],
            Self::BearTrap => &[("iron", 3), ("chain", 1)],
            Self::VenomSnare => &[("trap", 1), ("venom", 1)],
            Self::AlarmTrap => &[("bells", 2), ("wire", 1)],
            Self::ComboTrap => &[("trap", 1), ("net", 1)],
        }
    }
}

/// Lure/bait types for hunting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LureType {
    MeatScraps,
    BloodBait,
    FishBait,
    MuskLure,
    RodentLure,
    HoneyBait,
}

impl LureType {
    /// Get species attracted by this lure
    pub fn attracts(&self) -> &'static [&'static str] {
        match self {
            Self::MeatScraps => &["Wolf", "Cougar", "Bear", "Bobcat"],
            Self::BloodBait => &["Wolf", "Cougar"],
            Self::FishBait => &["Alligator", "Bear"],
            Self::MuskLure => &["Boar", "Deer"],
            Self::RodentLure => &["Snake", "Bobcat"],
            Self::HoneyBait => &["Bear"],
        }
    }

    /// Get effective range in units
    pub fn range(&self) -> f32 {
        30.0
    }
}

/// Animal call types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimalCall {
    DeerBleat,
    BoarGrunt,
    WolfHowl,
    BearRoar,
    DistressCall,
}

impl AnimalCall {
    /// Get species attracted by this call
    pub fn attracts(&self) -> &'static [&'static str] {
        match self {
            Self::DeerBleat => &["Deer"],
            Self::BoarGrunt => &["Wild Boar"],
            Self::WolfHowl => &["Gray Wolf", "Red Wolf"],
            Self::BearRoar => &["Black Bear"],
            Self::DistressCall => &["Wolf", "Cougar", "Bear"], // Predators investigate
        }
    }

    /// Get risk level (chance of aggressive response)
    pub fn risk_level(&self) -> f32 {
        match self {
            Self::DeerBleat => 0.1, // May attract predators
            Self::BoarGrunt => 0.5, // Aggressive response
            Self::WolfHowl => 0.8,  // Entire pack responds
            Self::BearRoar => 0.9,  // Very dangerous
            Self::DistressCall => 0.6,
        }
    }
}
