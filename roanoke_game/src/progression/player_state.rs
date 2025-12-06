//! Player Progression State
//!
//! Central tracking for all player advancement, skills, and world relationships.

use super::skills::{HuntingSkills, ArchaeologySkills};
use super::reputation::{Reputation, Faction as LegacyFaction, ReputationLevel};
use super::faction_manager::FactionManager;
use super::faction_integration::{FactionEventProcessor, FactionEvent, VillageFaction, NpcFactionData};
use super::faction::{Faction, Standing};
use super::faction_skills::FactionSkillId;
use super::faction_pipeline::{FactionPipelineCoordinator, ReputationSource, PipelineResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Maximum skill points that can be accumulated
pub const MAX_SKILL_POINTS: u32 = 10_000;

/// Experience required per level
pub const XP_PER_LEVEL: [u32; 50] = [
    100, 200, 350, 550, 800,      // 1-5
    1100, 1450, 1850, 2300, 2800, // 6-10
    3350, 3950, 4600, 5300, 6050, // 11-15
    6850, 7700, 8600, 9550, 10550,// 16-20
    11600, 12700, 13850, 15050, 16300, // 21-25
    17600, 18950, 20350, 21800, 23300, // 26-30
    24850, 26450, 28100, 29800, 31550, // 31-35
    33350, 35200, 37100, 39050, 41050, // 36-40
    43100, 45200, 47350, 49550, 51800, // 41-45
    54100, 56450, 58850, 61300, 63800, // 46-50
];

/// Master player progression container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProgression {
    // Skill trees
    pub hunting: HuntingSkills,
    pub archaeology: ArchaeologySkills,

    // Experience and leveling
    pub total_experience: u32,
    pub current_level: u32,

    // Legacy reputation with factions (kept for backward compatibility)
    pub reputation: HashMap<LegacyFaction, Reputation>,

    // New faction system
    #[serde(default)]
    pub faction_manager: FactionManager,
    #[serde(skip)]
    pub faction_events: FactionEventProcessor,
    #[serde(skip)]
    pub faction_pipeline: FactionPipelineCoordinator,
    #[serde(default)]
    pub village_factions: HashMap<u32, VillageFaction>,
    #[serde(default)]
    pub npc_factions: HashMap<u32, NpcFactionData>,

    // Discovery tracking
    pub discovered_locations: HashSet<LocationId>,
    pub discovered_species: HashSet<String>,
    pub discovered_fossils: HashSet<String>,
    pub discovered_microcosms: HashSet<u64>,

    // Kill/harvest tracking (for skill unlocks)
    pub kills_by_species: HashMap<String, u32>,
    pub stealth_kills: u32,
    pub trap_captures: u32,
    pub perfect_kills: u32,
    pub dens_discovered: u32,
    pub headshot_kills: u32,
    pub critical_kills: u32,

    // Combat tracking
    pub damage_dealt_by_weapon: HashMap<String, f32>,
    pub largest_kill_streak: u32,
    pub current_kill_streak: u32,
    pub boss_kills: u32,

    // Fossil tracking
    pub fossils_extracted: u32,
    pub perfect_extractions: u32,
    pub fossil_types_found: HashSet<String>,
    pub fossils_traded_value: u32,
    pub dig_sites_excavated: u32,
    pub rare_fossils_found: u32,

    // Campaign progress
    pub main_quest_chapter: u32,
    pub completed_quests: HashSet<String>,
    pub active_quests: Vec<String>,

    // Achievements
    pub achievements: HashSet<String>,
    pub achievement_progress: HashMap<String, u32>,

    // Milestones
    pub milestones_reached: Vec<Milestone>,

    // Time played
    pub days_survived: u32,
    pub in_game_hours: f32,
    pub real_play_time_seconds: f64,

    // Statistics
    pub stats: PlayerStats,

    // Event log for important actions
    pub event_log: Vec<ProgressionEvent>,
}

/// Unique location identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationId(pub u64);

/// A milestone achievement in player progression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub description: String,
    pub timestamp: f64,
    pub category: MilestoneCategory,
}

/// Categories of milestones
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneCategory {
    Story,
    Hunting,
    Archaeology,
    Exploration,
    Combat,
    Social,
    Legendary,
}

/// Events tracked in progression log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionEvent {
    pub event_type: ProgressionEventType,
    pub timestamp: f64,
    pub details: String,
}

/// Types of progression events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressionEventType {
    LevelUp(u32),
    SkillUnlocked(String),
    AchievementEarned(String),
    QuestCompleted(String),
    ReputationChanged(Faction, i32),
    LegendaryKilled(String),
    MilestoneReached(String),
    FirstKill(String),
    BossDefeated(String),
    AreaDiscovered(String),
}

/// Player statistics tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStats {
    // Combat stats
    pub total_damage_dealt: f32,
    pub total_damage_taken: f32,
    pub highest_single_hit: f32,
    pub animals_killed: u32,
    pub animals_fled_from: u32,
    pub deaths: u32,
    pub times_downed: u32,
    pub successful_dodges: u32,
    pub successful_blocks: u32,

    // Exploration stats
    pub distance_traveled: f32,
    pub distance_sprinted: f32,
    pub distance_swam: f32,
    pub highest_altitude_reached: f32,
    pub lowest_altitude_reached: f32,
    pub caves_explored: u32,

    // Crafting and trading
    pub items_crafted: u32,
    pub items_traded: u32,
    pub gold_earned: u32,
    pub gold_spent: u32,

    // Social stats
    pub npcs_befriended: u32,
    pub npcs_angered: u32,
    pub dialogues_completed: u32,
    pub gifts_given: u32,

    // Discovery stats
    pub villages_discovered: u32,
    pub secret_areas_found: u32,
    pub collectibles_found: u32,

    // Legendary encounters
    pub legendary_kills: Vec<String>,
    pub legendary_encounters: u32,
    pub legendary_escapes: u32,

    // Session tracking
    pub longest_session_minutes: u32,
    pub total_sessions: u32,
}

impl Default for PlayerProgression {
    fn default() -> Self {
        Self {
            hunting: HuntingSkills::default(),
            archaeology: ArchaeologySkills::default(),
            total_experience: 0,
            current_level: 1,
            reputation: HashMap::new(),
            faction_manager: FactionManager::new(),
            faction_events: FactionEventProcessor::new(),
            faction_pipeline: FactionPipelineCoordinator::new(),
            village_factions: HashMap::new(),
            npc_factions: HashMap::new(),
            discovered_locations: HashSet::new(),
            discovered_species: HashSet::new(),
            discovered_fossils: HashSet::new(),
            discovered_microcosms: HashSet::new(),
            kills_by_species: HashMap::new(),
            stealth_kills: 0,
            trap_captures: 0,
            perfect_kills: 0,
            dens_discovered: 0,
            headshot_kills: 0,
            critical_kills: 0,
            damage_dealt_by_weapon: HashMap::new(),
            largest_kill_streak: 0,
            current_kill_streak: 0,
            boss_kills: 0,
            fossils_extracted: 0,
            perfect_extractions: 0,
            fossil_types_found: HashSet::new(),
            fossils_traded_value: 0,
            dig_sites_excavated: 0,
            rare_fossils_found: 0,
            main_quest_chapter: 0,
            completed_quests: HashSet::new(),
            active_quests: Vec::new(),
            achievements: HashSet::new(),
            achievement_progress: HashMap::new(),
            milestones_reached: Vec::new(),
            days_survived: 0,
            in_game_hours: 0.0,
            real_play_time_seconds: 0.0,
            stats: PlayerStats::default(),
            event_log: Vec::new(),
        }
    }
}

impl PlayerProgression {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the player's hunting skill level (0-100)
    pub fn get_hunting_level(&self) -> u32 {
        // Calculate from hunting skill points, capped at 100
        self.hunting.points.min(100)
    }

    /// Get the player's current luck stat (0.0-1.0)
    pub fn get_luck_stat(&self) -> f32 {
        // Base luck + bonus from skills and equipment
        0.0 + if self.hunting.legendary_hunter { 0.15 } else { 0.0 }
    }

    /// Record a kill for skill progression
    pub fn record_kill(&mut self, species: &str, was_stealth: bool, was_perfect: bool) {
        *self.kills_by_species.entry(species.to_string()).or_insert(0) += 1;
        self.stats.animals_killed += 1;

        if was_stealth {
            self.stealth_kills += 1;
        }
        if was_perfect {
            self.perfect_kills += 1;
        }

        // Award hunting skill points
        let points = 10 + if was_stealth { 20 } else { 0 } + if was_perfect { 25 } else { 0 };

        // First kill of species bonus
        if !self.discovered_species.contains(species) {
            self.discovered_species.insert(species.to_string());
            self.hunting.points += 50;
        }

        self.hunting.points += points;

        // Check for hunting skill unlocks
        self.check_hunting_unlocks(species);
    }

    /// Record a fossil extraction
    pub fn record_fossil_extraction(&mut self, fossil_type: &str, was_perfect: bool, quality: u8) {
        self.fossils_extracted += 1;

        if was_perfect {
            self.perfect_extractions += 1;
        }

        // First fossil type bonus
        if !self.fossil_types_found.contains(fossil_type) {
            self.fossil_types_found.insert(fossil_type.to_string());
            self.archaeology.points += 25;
        }

        // Points based on success
        let points = 5 + if was_perfect { 10 } else { 0 } + (quality as u32 * 2);
        self.archaeology.points += points;

        // Check archaeology unlocks
        self.check_archaeology_unlocks();
    }

    /// Record discovering a den/nest
    pub fn record_den_discovery(&mut self) {
        self.dens_discovered += 1;
        self.hunting.points += 30;
        self.check_hunting_unlocks("");
    }

    /// Record trap capture
    pub fn record_trap_capture(&mut self, species: &str) {
        self.trap_captures += 1;
        self.hunting.points += 15;
        self.check_hunting_unlocks(species);
    }

    /// Check and apply hunting skill unlocks
    fn check_hunting_unlocks(&mut self, last_species: &str) {
        let kills = &self.kills_by_species;

        // Boar Hunter: Kill first wild boar
        if !self.hunting.boar_hunter && kills.get("Wild Boar").copied().unwrap_or(0) >= 1 {
            self.hunting.boar_hunter = true;
        }

        // Wolf Tracker: Survive encounter with 3+ wolves
        if !self.hunting.wolf_tracker && kills.get("Gray Wolf").copied().unwrap_or(0) >= 3 {
            self.hunting.wolf_tracker = true;
        }

        // Beast Slayer: Kill one of each predator
        let predator_kills = [
            kills.get("Black Bear").copied().unwrap_or(0) >= 1,
            kills.get("Eastern Cougar").copied().unwrap_or(0) >= 1,
            kills.get("Gray Wolf").copied().unwrap_or(0) >= 1,
            kills.get("Timber Rattlesnake").copied().unwrap_or(0) >= 1,
            kills.get("American Alligator").copied().unwrap_or(0) >= 1,
        ];
        if !self.hunting.beast_slayer && predator_kills.iter().all(|&x| x) {
            self.hunting.beast_slayer = true;
        }

        // Shadow Hunter: 5 stealth kills
        if !self.hunting.shadow_hunter && self.stealth_kills >= 5 {
            self.hunting.shadow_hunter = true;
        }

        // Big Game Hunter: Kill bear or alligator solo
        if !self.hunting.big_game_hunter {
            if kills.get("Black Bear").copied().unwrap_or(0) >= 1
               || kills.get("American Alligator").copied().unwrap_or(0) >= 1 {
                self.hunting.big_game_hunter = true;
            }
        }

        // Trap Setter: 5 trap captures
        if !self.hunting.trap_setter && self.trap_captures >= 5 {
            self.hunting.trap_setter = true;
        }

        // Snare Master: 20 trap captures
        if !self.hunting.snare_master && self.trap_captures >= 20 {
            self.hunting.snare_master = true;
        }

        // Wilderness Scout: 5 dens discovered
        if !self.hunting.wilderness_scout && self.dens_discovered >= 5 {
            self.hunting.wilderness_scout = true;
        }

        // Predator Sense: 3 different predator species killed
        let predator_count = predator_kills.iter().filter(|&&x| x).count();
        if !self.hunting.predator_sense && predator_count >= 3 {
            self.hunting.predator_sense = true;
        }

        // Prey Instinct: 10 prey animal kills
        let prey_kills = kills.get("Wild Boar").copied().unwrap_or(0);
        if !self.hunting.prey_instinct && prey_kills >= 10 {
            self.hunting.prey_instinct = true;
        }

        // Apex Predator: Beast Slayer + Shadow Hunter
        if !self.hunting.apex_predator && self.hunting.beast_slayer && self.hunting.shadow_hunter {
            self.hunting.apex_predator = true;
        }

        // Master Trapper: Snare Master + Lure Crafter
        if !self.hunting.master_trapper && self.hunting.snare_master && self.hunting.lure_crafter {
            self.hunting.master_trapper = true;
        }

        // Legendary Hunter: Apex Predator + Master Trapper
        if !self.hunting.legendary_hunter && self.hunting.apex_predator && self.hunting.master_trapper {
            self.hunting.legendary_hunter = true;
        }
    }

    /// Check and apply archaeology skill unlocks
    fn check_archaeology_unlocks(&mut self) {
        let types = &self.fossil_types_found;

        // Megalodon Hunter: Find first megalodon tooth
        if !self.archaeology.megalodon_hunter && types.iter().any(|t| t.contains("Megalodon")) {
            self.archaeology.megalodon_hunter = true;
        }

        // Mastodon Seeker: Find first mastodon bone
        if !self.archaeology.mastodon_seeker && types.iter().any(|t| t.contains("Mastodon")) {
            self.archaeology.mastodon_seeker = true;
        }

        // Curious Eye: 5 different fossil types
        if !self.archaeology.curious_eye && types.len() >= 5 {
            self.archaeology.curious_eye = true;
        }

        // Field Scholar: 10 different fossil types
        if !self.archaeology.field_scholar && types.len() >= 10 {
            self.archaeology.field_scholar = true;
        }

        // Keen Collector: 500 gold worth of fossils traded
        if !self.archaeology.keen_collector && self.fossils_traded_value >= 500 {
            self.archaeology.keen_collector = true;
        }
    }

    /// Get reputation with a legacy faction
    pub fn get_reputation(&self, faction: &LegacyFaction) -> i32 {
        self.reputation.get(faction).map(|r| r.value).unwrap_or(0)
    }

    /// Modify reputation with a legacy faction
    pub fn modify_reputation(&mut self, faction: LegacyFaction, delta: i32) {
        let rep = self.reputation.entry(faction).or_insert(Reputation::default());
        rep.modify(delta);
    }

    /// Check if player has discovered a location
    pub fn has_discovered(&self, location: LocationId) -> bool {
        self.discovered_locations.contains(&location)
    }

    /// Discover a new location
    pub fn discover_location(&mut self, location: LocationId) -> bool {
        if self.discovered_locations.insert(location) {
            // Award exploration bonus
            self.hunting.points += 10;
            true
        } else {
            false
        }
    }

    /// Get hunting damage bonus based on skills
    pub fn hunting_damage_bonus(&self, species: &str) -> f32 {
        let mut bonus = 1.0;

        // Species-specific bonuses
        match species {
            "Wild Boar" if self.hunting.boar_hunter => bonus += 0.25,
            "Gray Wolf" | "Red Wolf" if self.hunting.wolf_tracker => bonus += 0.20,
            _ => {}
        }

        // Category bonuses
        if self.hunting.beast_slayer {
            bonus += 0.50; // +50% vs predators
        }

        if self.hunting.big_game_hunter {
            if matches!(species, "Black Bear" | "American Alligator" | "Eastern Cougar") {
                bonus += 0.35;
            }
        }

        // Stealth bonus (checked separately in combat)

        bonus
    }

    /// Get skinning yield modifier
    pub fn skinning_yield_modifier(&self) -> f32 {
        let mut modifier = 0.5; // Base 50% yield

        if self.hunting.basic_tracker {
            modifier = 0.5;
        }
        if self.hunting.prey_instinct {
            modifier = 0.75; // 1.5x for prey
        }
        if self.hunting.beast_slayer {
            modifier = 0.75; // 1.5x for predators
        }
        if self.hunting.legendary_hunter {
            modifier = 1.0; // 2x for all
        }

        modifier
    }

    /// Get fossil extraction success rate
    pub fn extraction_success_rate(&self) -> f32 {
        let mut rate = 0.25; // Base 25%

        if self.archaeology.megalodon_hunter || self.archaeology.mastodon_seeker {
            rate = 0.40;
        }
        if self.archaeology.curious_eye {
            rate = 0.55;
        }
        if self.archaeology.field_scholar {
            rate = 0.70;
        }
        if self.archaeology.bone_reader || self.archaeology.stone_sage {
            rate = 0.85;
        }
        if self.archaeology.ancient_lore {
            rate = 0.95;
        }
        if self.archaeology.master_antiquarian {
            rate = 1.0;
        }

        rate
    }

    /// Check if player can access trading with NPCs (legacy)
    pub fn can_trade(&self, faction: &LegacyFaction) -> bool {
        self.get_reputation(faction) >= -50
    }

    /// Check if player can access quests from faction (legacy)
    pub fn can_accept_quests(&self, faction: &LegacyFaction) -> bool {
        self.get_reputation(faction) >= 0
    }

    /// Check if player has unlocked skill training (legacy)
    pub fn can_train_skills(&self, faction: &LegacyFaction) -> bool {
        self.get_reputation(faction) >= 100
    }

    // === Experience and Leveling ===

    /// Add experience points and check for level up
    pub fn add_experience(&mut self, amount: u32, game_time: f64) -> Option<u32> {
        self.total_experience = self.total_experience.saturating_add(amount);

        // Check for level up
        let new_level = self.calculate_level();
        if new_level > self.current_level {
            let old_level = self.current_level;
            self.current_level = new_level;

            // Log the level up event
            self.event_log.push(ProgressionEvent {
                event_type: ProgressionEventType::LevelUp(new_level),
                timestamp: game_time,
                details: format!("Reached level {} from {}", new_level, old_level),
            });

            // Award skill points on level up
            let bonus_points = (new_level - old_level) * 25;
            self.hunting.points += bonus_points;
            self.archaeology.points += bonus_points;

            return Some(new_level);
        }
        None
    }

    /// Calculate level from total experience
    fn calculate_level(&self) -> u32 {
        let mut cumulative = 0u32;
        for (level_idx, &xp_required) in XP_PER_LEVEL.iter().enumerate() {
            cumulative = cumulative.saturating_add(xp_required);
            if self.total_experience < cumulative {
                return (level_idx + 1) as u32;
            }
        }
        50 // Max level
    }

    /// Get experience needed for next level
    pub fn xp_for_next_level(&self) -> u32 {
        if self.current_level >= 50 {
            return 0;
        }
        let idx = (self.current_level - 1) as usize;
        if idx < XP_PER_LEVEL.len() {
            XP_PER_LEVEL[idx]
        } else {
            0
        }
    }

    /// Get current progress toward next level (0.0 - 1.0)
    pub fn level_progress(&self) -> f32 {
        if self.current_level >= 50 {
            return 1.0;
        }

        let mut cumulative = 0u32;
        for i in 0..((self.current_level - 1) as usize).min(XP_PER_LEVEL.len()) {
            cumulative += XP_PER_LEVEL[i];
        }

        let xp_into_level = self.total_experience.saturating_sub(cumulative);
        let xp_needed = self.xp_for_next_level();

        if xp_needed > 0 {
            (xp_into_level as f32 / xp_needed as f32).min(1.0)
        } else {
            1.0
        }
    }

    // === Achievement System ===

    /// Check and award an achievement
    pub fn check_achievement(&mut self, achievement_id: &str, game_time: f64) -> bool {
        if self.achievements.contains(achievement_id) {
            return false;
        }

        let earned = match achievement_id {
            // Combat achievements
            "first_blood" => self.stats.animals_killed >= 1,
            "seasoned_hunter" => self.stats.animals_killed >= 50,
            "master_hunter" => self.stats.animals_killed >= 200,
            "apex_predator_achievement" => self.boss_kills >= 1,
            "legendary_slayer" => !self.stats.legendary_kills.is_empty(),
            "undying" => self.stats.deaths == 0 && self.days_survived >= 7,

            // Stealth achievements
            "silent_killer" => self.stealth_kills >= 10,
            "ghost_hunter" => self.stealth_kills >= 50,
            "unseen_death" => self.perfect_kills >= 25,

            // Exploration achievements
            "explorer" => self.discovered_locations.len() >= 10,
            "cartographer" => self.discovered_locations.len() >= 50,
            "world_walker" => self.stats.distance_traveled >= 100_000.0,

            // Archaeology achievements
            "fossil_finder" => self.fossils_extracted >= 1,
            "amateur_archaeologist" => self.fossils_extracted >= 25,
            "professional_archaeologist" => self.fossils_extracted >= 100,
            "megalodon_discovery" => self.archaeology.megalodon_hunter,
            "mastodon_discovery" => self.archaeology.mastodon_seeker,

            // Social achievements
            "diplomat" => self.stats.npcs_befriended >= 5,
            "beloved" => self.reputation.values().filter(|r| r.level >= ReputationLevel::Honored).count() >= 3,
            "trader" => self.stats.items_traded >= 100,

            // Survival achievements
            "survivor" => self.days_survived >= 1,
            "week_survivor" => self.days_survived >= 7,
            "month_survivor" => self.days_survived >= 30,
            "veteran" => self.days_survived >= 100,

            // Kill streak achievements
            "rampage" => self.largest_kill_streak >= 5,
            "massacre" => self.largest_kill_streak >= 10,
            "unstoppable" => self.largest_kill_streak >= 25,

            // Trap achievements
            "trapper" => self.trap_captures >= 1,
            "expert_trapper" => self.trap_captures >= 25,

            _ => false,
        };

        if earned {
            self.achievements.insert(achievement_id.to_string());
            self.event_log.push(ProgressionEvent {
                event_type: ProgressionEventType::AchievementEarned(achievement_id.to_string()),
                timestamp: game_time,
                details: format!("Achievement unlocked: {}", achievement_id),
            });
            true
        } else {
            false
        }
    }

    /// Check all achievements and return newly earned ones
    pub fn check_all_achievements(&mut self, game_time: f64) -> Vec<String> {
        let achievement_ids = [
            "first_blood", "seasoned_hunter", "master_hunter", "apex_predator_achievement",
            "legendary_slayer", "undying", "silent_killer", "ghost_hunter", "unseen_death",
            "explorer", "cartographer", "world_walker", "fossil_finder", "amateur_archaeologist",
            "professional_archaeologist", "megalodon_discovery", "mastodon_discovery",
            "diplomat", "beloved", "trader", "survivor", "week_survivor", "month_survivor",
            "veteran", "rampage", "massacre", "unstoppable", "trapper", "expert_trapper",
        ];

        achievement_ids.iter()
            .filter_map(|&id| {
                if self.check_achievement(id, game_time) {
                    Some(id.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    // === Combat Tracking ===

    /// Record combat damage dealt
    pub fn record_damage_dealt(&mut self, amount: f32, weapon: &str) {
        self.stats.total_damage_dealt += amount;
        if amount > self.stats.highest_single_hit {
            self.stats.highest_single_hit = amount;
        }
        *self.damage_dealt_by_weapon.entry(weapon.to_string()).or_insert(0.0) += amount;
    }

    /// Record kill for streak tracking
    pub fn record_kill_for_streak(&mut self, game_time: f64) {
        self.current_kill_streak += 1;
        if self.current_kill_streak > self.largest_kill_streak {
            self.largest_kill_streak = self.current_kill_streak;
        }
    }

    /// Reset kill streak (called when player takes significant damage or time passes)
    pub fn reset_kill_streak(&mut self) {
        self.current_kill_streak = 0;
    }

    /// Record critical hit
    pub fn record_critical_kill(&mut self) {
        self.critical_kills += 1;
    }

    /// Record headshot
    pub fn record_headshot(&mut self) {
        self.headshot_kills += 1;
    }

    // === Milestone System ===

    /// Add a milestone
    pub fn add_milestone(&mut self, id: &str, name: &str, description: &str, category: MilestoneCategory, game_time: f64) {
        if self.milestones_reached.iter().any(|m| m.id == id) {
            return; // Already have this milestone
        }

        self.milestones_reached.push(Milestone {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            timestamp: game_time,
            category,
        });

        self.event_log.push(ProgressionEvent {
            event_type: ProgressionEventType::MilestoneReached(name.to_string()),
            timestamp: game_time,
            details: description.to_string(),
        });
    }

    // === Validation and State Checks ===

    /// Validate progression state (fix any inconsistencies)
    pub fn validate(&mut self) {
        // Ensure level matches experience
        self.current_level = self.calculate_level();

        // Cap skill points
        self.hunting.points = self.hunting.points.min(MAX_SKILL_POINTS);
        self.archaeology.points = self.archaeology.points.min(MAX_SKILL_POINTS);

        // Ensure kill streak is valid
        if self.current_kill_streak > self.largest_kill_streak {
            self.largest_kill_streak = self.current_kill_streak;
        }

        // Validate reputation bounds
        for rep in self.reputation.values_mut() {
            rep.value = rep.value.clamp(-1000, 1000);
        }

        // Trim event log if too long
        const MAX_EVENTS: usize = 1000;
        if self.event_log.len() > MAX_EVENTS {
            self.event_log.drain(0..(self.event_log.len() - MAX_EVENTS));
        }
    }

    /// Get a summary of player progression for UI display
    pub fn summary(&self) -> ProgressionSummary {
        ProgressionSummary {
            level: self.current_level,
            total_xp: self.total_experience,
            level_progress: self.level_progress(),
            hunting_points: self.hunting.points,
            archaeology_points: self.archaeology.points,
            total_kills: self.stats.animals_killed,
            discoveries: self.discovered_locations.len() as u32,
            achievements_earned: self.achievements.len() as u32,
            days_survived: self.days_survived,
            main_quest_progress: self.main_quest_chapter,
        }
    }

    /// Update play time tracking
    pub fn update_play_time(&mut self, real_dt_seconds: f64, game_hours: f32) {
        self.real_play_time_seconds += real_dt_seconds;
        self.in_game_hours = game_hours;

        // Update longest session
        let session_minutes = (self.real_play_time_seconds / 60.0) as u32;
        if session_minutes > self.stats.longest_session_minutes {
            self.stats.longest_session_minutes = session_minutes;
        }
    }

    /// Get list of unlocked skill names for UI
    pub fn unlocked_hunting_skills(&self) -> Vec<&'static str> {
        let mut skills = Vec::new();
        if self.hunting.basic_tracker { skills.push("Basic Tracker"); }
        if self.hunting.boar_hunter { skills.push("Boar Hunter"); }
        if self.hunting.deer_stalker { skills.push("Deer Stalker"); }
        if self.hunting.wolf_tracker { skills.push("Wolf Tracker"); }
        if self.hunting.beast_slayer { skills.push("Beast Slayer"); }
        if self.hunting.shadow_hunter { skills.push("Shadow Hunter"); }
        if self.hunting.big_game_hunter { skills.push("Big Game Hunter"); }
        if self.hunting.predator_sense { skills.push("Predator Sense"); }
        if self.hunting.prey_instinct { skills.push("Prey Instinct"); }
        if self.hunting.trap_setter { skills.push("Trap Setter"); }
        if self.hunting.snare_master { skills.push("Snare Master"); }
        if self.hunting.lure_crafter { skills.push("Lure Crafter"); }
        if self.hunting.wilderness_scout { skills.push("Wilderness Scout"); }
        if self.hunting.apex_predator { skills.push("Apex Predator"); }
        if self.hunting.master_trapper { skills.push("Master Trapper"); }
        if self.hunting.legendary_hunter { skills.push("Legendary Hunter"); }
        skills
    }

    /// Check if player meets requirements for specific content
    pub fn meets_requirements(&self, req: &ProgressionRequirement) -> bool {
        match req {
            ProgressionRequirement::Level(min_level) => self.current_level >= *min_level,
            ProgressionRequirement::Reputation(faction, min_rep) => {
                self.faction_manager.get_reputation(*faction) >= *min_rep
            }
            ProgressionRequirement::Skill(skill_name) => self.has_skill(skill_name),
            ProgressionRequirement::Quest(quest_id) => self.completed_quests.contains(quest_id),
            ProgressionRequirement::Achievement(achievement_id) => self.achievements.contains(achievement_id),
            ProgressionRequirement::Kills(species, count) => {
                self.kills_by_species.get(species).copied().unwrap_or(0) >= *count
            }
        }
    }

    /// Check if player has a specific skill
    pub fn has_skill(&self, skill_name: &str) -> bool {
        match skill_name {
            "basic_tracker" => self.hunting.basic_tracker,
            "boar_hunter" => self.hunting.boar_hunter,
            "deer_stalker" => self.hunting.deer_stalker,
            "wolf_tracker" => self.hunting.wolf_tracker,
            "beast_slayer" => self.hunting.beast_slayer,
            "shadow_hunter" => self.hunting.shadow_hunter,
            "big_game_hunter" => self.hunting.big_game_hunter,
            "predator_sense" => self.hunting.predator_sense,
            "prey_instinct" => self.hunting.prey_instinct,
            "trap_setter" => self.hunting.trap_setter,
            "snare_master" => self.hunting.snare_master,
            "lure_crafter" => self.hunting.lure_crafter,
            "wilderness_scout" => self.hunting.wilderness_scout,
            "apex_predator" => self.hunting.apex_predator,
            "master_trapper" => self.hunting.master_trapper,
            "legendary_hunter" => self.hunting.legendary_hunter,
            "megalodon_hunter" => self.archaeology.megalodon_hunter,
            "mastodon_seeker" => self.archaeology.mastodon_seeker,
            "curious_eye" => self.archaeology.curious_eye,
            "field_scholar" => self.archaeology.field_scholar,
            "keen_collector" => self.archaeology.keen_collector,
            "bone_reader" => self.archaeology.bone_reader,
            "stone_sage" => self.archaeology.stone_sage,
            "ancient_lore" => self.archaeology.ancient_lore,
            "master_antiquarian" => self.archaeology.master_antiquarian,
            _ => false,
        }
    }

    // ============================================================================
    // FACTION SYSTEM INTEGRATION
    // ============================================================================

    /// Process pending faction events and update state
    pub fn update_faction_system(&mut self, game_time: f64) {
        // Process legacy event system
        self.faction_events.process_events(&mut self.faction_manager, game_time);

        // Process pipeline reputation changes - flush to faction manager
        let _results = self.faction_pipeline.flush_reputation_changes(&mut self.faction_manager, game_time);

        // Dispatch notifications
        let _dispatched = self.faction_pipeline.dispatch_notifications();

        // Process sync operations (logged but not acted upon here - handled by coordinator)
        let _sync_ops = self.faction_pipeline.sync.take_pending();
    }

    /// Queue a faction event for processing
    pub fn queue_faction_event(&mut self, event: FactionEvent) {
        self.faction_events.queue_event(event);
    }

    /// Queue a reputation change through the hardened pipeline
    pub fn queue_reputation_change(
        &mut self,
        faction: Faction,
        delta: i32,
        reason: &str,
        source: ReputationSource,
        game_time: f64,
    ) -> PipelineResult<u64> {
        self.faction_pipeline.process_reputation_change(faction, delta, reason, source, game_time)
    }

    /// Get standing with a specific faction
    pub fn get_faction_standing(&self, faction: Faction) -> Standing {
        self.faction_manager.get_standing(faction)
    }

    /// Get reputation points with a faction
    pub fn get_faction_reputation(&self, faction: Faction) -> i32 {
        self.faction_manager.get_reputation(faction)
    }

    /// Modify reputation with a faction directly (bypasses pipeline)
    pub fn modify_faction_reputation(&mut self, faction: Faction, delta: i32, reason: &str, game_time: f64) {
        self.faction_manager.modify_reputation(faction, delta, reason, game_time);
    }

    /// Modify reputation through the validated pipeline
    pub fn modify_faction_reputation_validated(
        &mut self,
        faction: Faction,
        delta: i32,
        reason: &str,
        source: ReputationSource,
        game_time: f64,
    ) -> PipelineResult<u64> {
        self.faction_pipeline.process_reputation_change(faction, delta, reason, source, game_time)
    }

    /// Get pipeline health report
    pub fn get_pipeline_health(&self) -> super::faction_pipeline::PipelineHealthReport {
        self.faction_pipeline.health_report()
    }

    /// Pause all faction pipelines
    pub fn pause_faction_pipelines(&mut self) {
        self.faction_pipeline.pause();
    }

    /// Resume all faction pipelines
    pub fn resume_faction_pipelines(&mut self) {
        self.faction_pipeline.resume();
    }

    /// Check if player can trade with faction
    pub fn can_trade_with_faction(&self, faction: Faction) -> bool {
        self.faction_manager.can_trade(faction)
    }

    /// Check if player can access faction quests
    pub fn can_access_faction_quests(&self, faction: Faction) -> bool {
        self.faction_manager.can_access_quests(faction)
    }

    /// Check if player can train faction skills
    pub fn can_train_faction_skills(&self, faction: Faction) -> bool {
        self.faction_manager.can_train_skills(faction)
    }

    /// Get trade price modifier for faction
    pub fn get_faction_trade_modifier(&self, faction: Faction) -> f32 {
        self.faction_manager.get_trade_modifier(faction)
    }

    /// Unlock a faction skill
    pub fn unlock_faction_skill(&mut self, skill_id: FactionSkillId) -> Result<(), super::faction_manager::SkillUnlockError> {
        self.faction_manager.unlock_skill(skill_id)
    }

    /// Check if a faction skill is unlocked
    pub fn has_faction_skill(&self, skill_id: FactionSkillId) -> bool {
        self.faction_manager.has_skill(skill_id)
    }

    /// Get available skills for a faction
    pub fn get_available_faction_skills(&self, faction: Faction) -> Vec<FactionSkillId> {
        self.faction_manager.get_available_skills(faction)
            .into_iter()
            .map(|s| s.id)
            .collect()
    }

    /// Register a village's faction affiliation
    pub fn register_village_faction(&mut self, village_id: u32, village_faction: VillageFaction) {
        self.village_factions.insert(village_id, village_faction);
    }

    /// Get a village's faction data
    pub fn get_village_faction(&self, village_id: u32) -> Option<&VillageFaction> {
        self.village_factions.get(&village_id)
    }

    /// Get a mutable reference to village faction data
    pub fn get_village_faction_mut(&mut self, village_id: u32) -> Option<&mut VillageFaction> {
        self.village_factions.get_mut(&village_id)
    }

    /// Register an NPC's faction data
    pub fn register_npc_faction(&mut self, npc_id: u32, npc_data: NpcFactionData) {
        self.npc_factions.insert(npc_id, npc_data);
    }

    /// Get an NPC's faction data
    pub fn get_npc_faction(&self, npc_id: u32) -> Option<&NpcFactionData> {
        self.npc_factions.get(&npc_id)
    }

    /// Get pending faction notifications
    pub fn get_faction_notifications(&mut self) -> Vec<super::faction_integration::FactionNotification> {
        self.faction_events.get_notifications()
    }

    /// Set player's primary faction
    pub fn set_primary_faction(&mut self, faction: Faction) {
        self.faction_manager.primary_faction = Some(faction);
    }

    /// Get player's primary faction
    pub fn get_primary_faction(&self) -> Option<Faction> {
        self.faction_manager.primary_faction
    }

    /// Get all factions the player is hostile with
    pub fn get_hostile_factions(&self) -> Vec<Faction> {
        self.faction_manager.get_hostile_factions()
    }

    /// Get all factions the player is allied with
    pub fn get_allied_factions(&self) -> Vec<Faction> {
        self.faction_manager.get_allied_factions()
    }

    /// Check relationship between two factions
    pub fn get_faction_relationship(&self, a: Faction, b: Faction) -> Standing {
        self.faction_manager.get_faction_relationship(a, b)
    }

    // ============================================================================
    // CROSS-FILE SYNC & VERSIONING
    // ============================================================================

    /// Sync legacy reputation system with new faction system
    /// Call this when loading old save files
    pub fn sync_legacy_to_faction(&mut self) {
        // Map legacy factions to new factions where possible
        for (legacy_faction, rep) in &self.reputation {
            let new_faction = match legacy_faction {
                LegacyFaction::EnglishSettlers => Some(Faction::English),
                LegacyFaction::SpanishExplorers => Some(Faction::Spanish),
                LegacyFaction::FrenchTraders => Some(Faction::French),
                LegacyFaction::NativeCouncil => Some(Faction::Powhatan),
                _ => None,
            };

            if let Some(faction) = new_faction {
                // Only sync if not already set
                if self.faction_manager.get_reputation(faction) == 0 {
                    self.faction_manager.modify_reputation(
                        faction,
                        rep.value,
                        "Migrated from legacy save",
                        0.0,
                    );
                }
            }
        }
    }

    /// Sync new faction system to legacy for backward compatibility
    pub fn sync_faction_to_legacy(&mut self) {
        let mappings = [
            (Faction::English, LegacyFaction::EnglishSettlers),
            (Faction::Spanish, LegacyFaction::SpanishExplorers),
            (Faction::French, LegacyFaction::FrenchTraders),
            (Faction::Powhatan, LegacyFaction::NativeCouncil),
        ];

        for (new_faction, legacy_faction) in mappings {
            let rep_value = self.faction_manager.get_reputation(new_faction);
            let legacy_rep = self.reputation.entry(legacy_faction).or_default();
            legacy_rep.value = rep_value;
            legacy_rep.level = ReputationLevel::from_value(rep_value);
        }
    }

    /// Get faction save data for serialization
    pub fn get_faction_save_data(&self, game_time: f64) -> super::faction_integration::FactionSaveData {
        super::faction_integration::FactionSaveData::from_state(
            &self.faction_manager,
            &self.village_factions,
            &self.npc_factions,
            &self.faction_events,
            game_time,
        )
    }

    /// Restore faction state from save data
    pub fn restore_faction_save_data(&mut self, save_data: &super::faction_integration::FactionSaveData) {
        save_data.restore(&mut self.faction_manager);
        self.village_factions = save_data.village_factions.clone();
        self.npc_factions = save_data.npc_faction_data.clone();
    }

    /// Get the current version of the progression system
    pub const fn progression_version() -> u32 {
        2 // Bumped for faction system integration
    }

    /// Migrate from older save format
    pub fn migrate_from_version(&mut self, version: u32) {
        match version {
            0 | 1 => {
                // Old saves before faction system - sync legacy data
                self.sync_legacy_to_faction();
                log::info!("Migrated save from version {} to {}", version, Self::progression_version());
            }
            2 => {
                // Current version, no migration needed
            }
            _ => {
                log::warn!("Unknown save version {}, attempting best-effort load", version);
            }
        }
    }
}

/// Summary of progression for UI
#[derive(Debug, Clone)]
pub struct ProgressionSummary {
    pub level: u32,
    pub total_xp: u32,
    pub level_progress: f32,
    pub hunting_points: u32,
    pub archaeology_points: u32,
    pub total_kills: u32,
    pub discoveries: u32,
    pub achievements_earned: u32,
    pub days_survived: u32,
    pub main_quest_progress: u32,
}

/// Requirements for content gating
#[derive(Debug, Clone)]
pub enum ProgressionRequirement {
    Level(u32),
    Reputation(Faction, i32),
    Skill(String),
    Quest(String),
    Achievement(String),
    Kills(String, u32),
}
