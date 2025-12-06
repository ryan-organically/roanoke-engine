//! Game Data Pipeline System
//!
//! Centralized data flow management connecting all game systems:
//! - Faction events → Audio events
//! - Progression events → Audio events
//! - NPC interactions → Audio events
//! - Village/Biome changes → Audio events
//!
//! Provides validation, error handling, and event batching for efficiency.

use crate::audio_events::{AudioEvent, AudioBiome, ThreatLevel, FactionTheme};
use crate::audio_system::AudioSystem;
use crate::progression::{
    Faction, Standing, FactionManager,
    faction_integration::{FactionEvent, VillageFaction, NpcFactionData, VillageStatus},
};
use glam::Vec3;
use std::collections::{HashMap, VecDeque};

// ============================================================================
// PIPELINE ERROR HANDLING
// ============================================================================

/// Errors that can occur in the data pipeline
#[derive(Debug, Clone)]
pub enum PipelineError {
    /// Event validation failed
    ValidationFailed { event: String, reason: String },
    /// Target system not available
    SystemUnavailable { system: String },
    /// Event processing failed
    ProcessingFailed { event: String, reason: String },
    /// Rate limit exceeded
    RateLimitExceeded { event_type: String },
    /// Invalid state transition
    InvalidTransition { from: String, to: String },
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValidationFailed { event, reason } =>
                write!(f, "Validation failed for '{}': {}", event, reason),
            Self::SystemUnavailable { system } =>
                write!(f, "System unavailable: {}", system),
            Self::ProcessingFailed { event, reason } =>
                write!(f, "Processing failed for '{}': {}", event, reason),
            Self::RateLimitExceeded { event_type } =>
                write!(f, "Rate limit exceeded for event type: {}", event_type),
            Self::InvalidTransition { from, to } =>
                write!(f, "Invalid state transition from '{}' to '{}'", from, to),
        }
    }
}

pub type PipelineResult<T> = Result<T, PipelineError>;

// ============================================================================
// UNIFIED GAME EVENT SYSTEM
// ============================================================================

/// Unified game events that flow through the pipeline
#[derive(Debug, Clone)]
pub enum GameEvent {
    // === FACTION EVENTS ===
    FactionReputationChanged {
        faction: Faction,
        old_rep: i32,
        new_rep: i32,
        source: ReputationSource,
    },
    FactionStandingChanged {
        faction: Faction,
        old_standing: Standing,
        new_standing: Standing,
    },
    FactionTerritoryEntered {
        faction: Faction,
        village_name: Option<String>,
    },
    FactionTerritoryExited {
        faction: Faction,
    },
    FactionWarDeclared {
        faction: Faction,
    },
    FactionAllianceFormed {
        faction: Faction,
    },

    // === PROGRESSION EVENTS ===
    SkillUnlocked {
        skill_id: String,
        skill_name: String,
        category: SkillCategory,
    },
    SkillLevelUp {
        skill_id: String,
        skill_name: String,
        new_level: u32,
    },
    QuestStarted {
        quest_id: String,
        quest_name: String,
        faction: Option<Faction>,
    },
    QuestObjectiveCompleted {
        quest_id: String,
        objective_name: String,
    },
    QuestCompleted {
        quest_id: String,
        quest_name: String,
        rewards: Vec<String>,
    },
    QuestFailed {
        quest_id: String,
        reason: String,
    },
    AchievementUnlocked {
        achievement_id: String,
        achievement_name: String,
        rarity: f32, // 0.0 = common, 1.0 = legendary
    },

    // === NPC EVENTS ===
    NpcGreeting {
        npc_id: u32,
        npc_name: String,
        disposition: NpcDispositionLevel,
    },
    NpcDialogueStarted {
        npc_id: u32,
        npc_name: String,
    },
    NpcDialogueEnded {
        npc_id: u32,
    },
    NpcTradeStarted {
        npc_id: u32,
        npc_name: String,
    },
    NpcTradeCompleted {
        npc_id: u32,
        profit: bool,
        value: i32,
    },
    NpcTradeFailed {
        npc_id: u32,
        reason: String,
    },
    NpcQuestOffered {
        npc_id: u32,
        quest_id: String,
    },
    NpcRelationshipChanged {
        npc_id: u32,
        old_level: i32,
        new_level: i32,
    },

    // === DISCOVERY EVENTS ===
    LocationDiscovered {
        location_type: LocationType,
        name: String,
        first_discovery: bool,
    },
    EncyclopediaEntryUnlocked {
        category: String,
        entry_name: String,
        completion_percent: f32,
    },
    SecretFound {
        secret_type: String,
        rarity: f32,
    },

    // === COMBAT EVENTS ===
    CombatStarted {
        enemy_type: String,
        enemy_count: u32,
    },
    CombatVictory {
        enemy_type: String,
        xp_gained: u32,
    },
    CombatDefeat,
    CombatFled,
    BossEncounter {
        boss_name: String,
    },

    // === ENVIRONMENT EVENTS ===
    BiomeChanged {
        from: AudioBiome,
        to: AudioBiome,
    },
    WeatherChanged {
        new_weather: String,
        intensity: f32,
    },
    TimeOfDayChanged {
        period: TimePeriod,
    },
    CaveEntered {
        cave_name: Option<String>,
        depth: f32,
    },
    CaveExited,
    WaterfallDiscovered {
        distance: f32,
    },
}

/// Source of reputation change for detailed tracking
#[derive(Debug, Clone, Copy)]
pub enum ReputationSource {
    Quest,
    Trade,
    Gift,
    Combat,
    Discovery,
    Dialogue,
    Crime,
    TimeDecay,
}

/// Skill categories for audio mapping
#[derive(Debug, Clone, Copy)]
pub enum SkillCategory {
    Combat,
    Survival,
    Crafting,
    Social,
    Exploration,
    Faction,
}

/// NPC disposition levels
#[derive(Debug, Clone, Copy)]
pub enum NpcDispositionLevel {
    Hostile,
    Suspicious,
    Neutral,
    Friendly,
    Trusted,
    Devoted,
}

/// Location types for discovery events
#[derive(Debug, Clone, Copy)]
pub enum LocationType {
    Village,
    Cave,
    Ruin,
    SacredSite,
    Waterfall,
    Viewpoint,
    Camp,
    Dungeon,
}

/// Time periods
#[derive(Debug, Clone, Copy)]
pub enum TimePeriod {
    Dawn,
    Morning,
    Noon,
    Afternoon,
    Dusk,
    Evening,
    Night,
    Midnight,
}

// ============================================================================
// DATA PIPELINE MANAGER
// ============================================================================

/// Rate limiter for event types
struct RateLimiter {
    last_event_time: HashMap<String, f32>,
    cooldowns: HashMap<String, f32>,
}

impl RateLimiter {
    fn new() -> Self {
        let mut cooldowns = HashMap::new();
        // Define cooldowns for various event types
        cooldowns.insert("biome_change".to_string(), 2.0);
        cooldowns.insert("discovery".to_string(), 1.5);
        cooldowns.insert("combat_start".to_string(), 0.5);
        cooldowns.insert("skill_levelup".to_string(), 0.3);
        cooldowns.insert("npc_greeting".to_string(), 3.0);

        Self {
            last_event_time: HashMap::new(),
            cooldowns,
        }
    }

    fn check(&mut self, event_type: &str, current_time: f32) -> bool {
        let cooldown = self.cooldowns.get(event_type).copied().unwrap_or(0.1);
        let last_time = self.last_event_time.get(event_type).copied().unwrap_or(-100.0);

        if current_time - last_time >= cooldown {
            self.last_event_time.insert(event_type.to_string(), current_time);
            true
        } else {
            false
        }
    }
}

/// Central data pipeline manager
pub struct DataPipeline {
    /// Pending events waiting to be processed
    pending_events: VecDeque<GameEvent>,
    /// Event history for debugging
    event_history: VecDeque<(f32, GameEvent)>,
    /// Maximum history size
    max_history: usize,
    /// Rate limiter
    rate_limiter: RateLimiter,
    /// Current game time
    current_time: f32,
    /// Processing statistics
    stats: PipelineStats,
    /// Current faction context
    current_faction_territory: Option<Faction>,
    /// Current village context
    current_village: Option<String>,
    /// Error log
    errors: VecDeque<PipelineError>,
    max_errors: usize,
}

/// Pipeline statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    pub events_processed: u64,
    pub events_dropped: u64,
    pub events_rate_limited: u64,
    pub errors_encountered: u64,
    pub audio_events_generated: u64,
}

impl DataPipeline {
    pub fn new() -> Self {
        Self {
            pending_events: VecDeque::with_capacity(64),
            event_history: VecDeque::with_capacity(100),
            max_history: 100,
            rate_limiter: RateLimiter::new(),
            current_time: 0.0,
            stats: PipelineStats::default(),
            current_faction_territory: None,
            current_village: None,
            errors: VecDeque::with_capacity(20),
            max_errors: 20,
        }
    }

    /// Push a game event into the pipeline
    pub fn push_event(&mut self, event: GameEvent) {
        // Validate event before adding
        if let Err(e) = self.validate_event(&event) {
            self.log_error(e);
            self.stats.events_dropped += 1;
            return;
        }

        self.pending_events.push_back(event);
    }

    /// Push a faction event (converts to GameEvent)
    pub fn push_faction_event(&mut self, event: FactionEvent, faction_manager: &FactionManager) {
        match event {
            FactionEvent::QuestCompleted { faction, quest_id, reputation_gain } => {
                let old_rep = faction_manager.get_reputation(faction);
                self.push_event(GameEvent::FactionReputationChanged {
                    faction,
                    old_rep,
                    new_rep: old_rep + reputation_gain,
                    source: ReputationSource::Quest,
                });
                self.push_event(GameEvent::QuestCompleted {
                    quest_id: quest_id.clone(),
                    quest_name: quest_id,
                    rewards: vec![format!("+{} {} reputation", reputation_gain, format!("{:?}", faction))],
                });
            }
            FactionEvent::QuestFailed { faction, quest_id, reputation_loss } => {
                let old_rep = faction_manager.get_reputation(faction);
                self.push_event(GameEvent::FactionReputationChanged {
                    faction,
                    old_rep,
                    new_rep: old_rep - reputation_loss,
                    source: ReputationSource::Quest,
                });
                self.push_event(GameEvent::QuestFailed {
                    quest_id,
                    reason: "Quest objectives not met".to_string(),
                });
            }
            FactionEvent::TradeCompleted { faction, value, fair_trade } => {
                let old_rep = faction_manager.get_reputation(faction);
                let rep_change = if fair_trade { value / 100 } else { -(value / 50) };
                self.push_event(GameEvent::FactionReputationChanged {
                    faction,
                    old_rep,
                    new_rep: old_rep + rep_change,
                    source: ReputationSource::Trade,
                });
            }
            FactionEvent::MemberAttacked { faction, npc_id, damage, .. } => {
                let old_rep = faction_manager.get_reputation(faction);
                let rep_loss = -(damage as i32 / 2).max(5);
                self.push_event(GameEvent::FactionReputationChanged {
                    faction,
                    old_rep,
                    new_rep: old_rep + rep_loss,
                    source: ReputationSource::Crime,
                });
                self.push_event(GameEvent::CombatStarted {
                    enemy_type: format!("{:?} member", faction),
                    enemy_count: 1,
                });
            }
            FactionEvent::MemberKilled { faction, npc_id, .. } => {
                let old_rep = faction_manager.get_reputation(faction);
                self.push_event(GameEvent::FactionReputationChanged {
                    faction,
                    old_rep,
                    new_rep: old_rep - 100,
                    source: ReputationSource::Crime,
                });
            }
            FactionEvent::GiftGiven { faction, npc_id, item_value } => {
                let old_rep = faction_manager.get_reputation(faction);
                self.push_event(GameEvent::FactionReputationChanged {
                    faction,
                    old_rep,
                    new_rep: old_rep + (item_value / 10).max(1),
                    source: ReputationSource::Gift,
                });
                self.push_event(GameEvent::NpcRelationshipChanged {
                    npc_id,
                    old_level: 0,
                    new_level: item_value / 20,
                });
            }
            FactionEvent::DiscoveryMade { faction, .. } => {
                let old_rep = faction_manager.get_reputation(faction);
                self.push_event(GameEvent::FactionReputationChanged {
                    faction,
                    old_rep,
                    new_rep: old_rep + 25,
                    source: ReputationSource::Discovery,
                });
            }
            FactionEvent::SettlementDefended { faction, village_id, enemies_killed } => {
                let old_rep = faction_manager.get_reputation(faction);
                self.push_event(GameEvent::FactionReputationChanged {
                    faction,
                    old_rep,
                    new_rep: old_rep + (enemies_killed as i32 * 10),
                    source: ReputationSource::Combat,
                });
                self.push_event(GameEvent::CombatVictory {
                    enemy_type: "invaders".to_string(),
                    xp_gained: enemies_killed * 50,
                });
            }
            _ => {} // Handle other events as needed
        }
    }

    /// Validate an event before processing
    fn validate_event(&self, event: &GameEvent) -> PipelineResult<()> {
        match event {
            GameEvent::FactionReputationChanged { old_rep, new_rep, .. } => {
                // Rep change shouldn't be too extreme in one event
                let delta = (new_rep - old_rep).abs();
                if delta > 500 {
                    return Err(PipelineError::ValidationFailed {
                        event: "FactionReputationChanged".to_string(),
                        reason: format!("Reputation change {} exceeds maximum of 500", delta),
                    });
                }
            }
            GameEvent::SkillLevelUp { new_level, .. } => {
                if *new_level > 100 {
                    return Err(PipelineError::ValidationFailed {
                        event: "SkillLevelUp".to_string(),
                        reason: format!("Skill level {} exceeds maximum of 100", new_level),
                    });
                }
            }
            GameEvent::CombatStarted { enemy_count, .. } => {
                if *enemy_count == 0 {
                    return Err(PipelineError::ValidationFailed {
                        event: "CombatStarted".to_string(),
                        reason: "Enemy count cannot be zero".to_string(),
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Process all pending events and generate audio events
    pub fn process(&mut self, dt: f32) -> Vec<AudioEvent> {
        self.current_time += dt;
        let mut audio_events = Vec::new();

        while let Some(event) = self.pending_events.pop_front() {
            // Check rate limiting
            let event_type = self.get_event_type(&event);
            if !self.rate_limiter.check(&event_type, self.current_time) {
                self.stats.events_rate_limited += 1;
                continue;
            }

            // Convert to audio event(s)
            match self.convert_to_audio(&event) {
                Ok(events) => {
                    self.stats.audio_events_generated += events.len() as u64;
                    audio_events.extend(events);
                }
                Err(e) => {
                    self.log_error(e);
                    self.stats.errors_encountered += 1;
                }
            }

            // Update context
            self.update_context(&event);

            // Record in history
            if self.event_history.len() >= self.max_history {
                self.event_history.pop_front();
            }
            self.event_history.push_back((self.current_time, event));
            self.stats.events_processed += 1;
        }

        audio_events
    }

    /// Get event type string for rate limiting
    fn get_event_type(&self, event: &GameEvent) -> String {
        match event {
            GameEvent::FactionReputationChanged { .. } => "faction_rep".to_string(),
            GameEvent::FactionStandingChanged { .. } => "faction_standing".to_string(),
            GameEvent::FactionTerritoryEntered { .. } => "territory_enter".to_string(),
            GameEvent::SkillUnlocked { .. } => "skill_unlock".to_string(),
            GameEvent::SkillLevelUp { .. } => "skill_levelup".to_string(),
            GameEvent::QuestStarted { .. } => "quest_start".to_string(),
            GameEvent::QuestCompleted { .. } => "quest_complete".to_string(),
            GameEvent::NpcGreeting { .. } => "npc_greeting".to_string(),
            GameEvent::NpcDialogueStarted { .. } => "dialogue_start".to_string(),
            GameEvent::NpcTradeStarted { .. } => "trade_start".to_string(),
            GameEvent::LocationDiscovered { .. } => "discovery".to_string(),
            GameEvent::CombatStarted { .. } => "combat_start".to_string(),
            GameEvent::BiomeChanged { .. } => "biome_change".to_string(),
            _ => "generic".to_string(),
        }
    }

    /// Convert a game event to audio event(s)
    fn convert_to_audio(&self, event: &GameEvent) -> PipelineResult<Vec<AudioEvent>> {
        let mut audio_events = Vec::new();

        match event {
            // === FACTION AUDIO ===
            GameEvent::FactionReputationChanged { faction, old_rep, new_rep, .. } => {
                let positive = new_rep > old_rep;
                audio_events.push(AudioEvent::FactionReputationChanged {
                    faction: format!("{:?}", faction),
                    positive,
                });
            }

            GameEvent::FactionStandingChanged { faction, old_standing, new_standing } => {
                // Major standing changes trigger special audio
                if *new_standing == Standing::Allied || *new_standing == Standing::BloodBond {
                    audio_events.push(AudioEvent::FactionAlliance);
                } else if *new_standing == Standing::Hostile || *new_standing == Standing::War {
                    audio_events.push(AudioEvent::FactionHostile);
                }
            }

            GameEvent::FactionTerritoryEntered { faction, village_name } => {
                // Get standing for audio intensity
                audio_events.push(AudioEvent::FactionTerritoryEntered {
                    faction: format!("{:?}", faction),
                    standing: 0.0, // Would need faction manager access for real value
                });

                if let Some(name) = village_name {
                    audio_events.push(AudioEvent::VillageEntered { population: 20 });
                }
            }

            GameEvent::FactionTerritoryExited { .. } => {
                if self.current_village.is_some() {
                    audio_events.push(AudioEvent::VillageExited);
                }
            }

            GameEvent::FactionWarDeclared { .. } => {
                audio_events.push(AudioEvent::FactionHostile);
            }

            GameEvent::FactionAllianceFormed { .. } => {
                audio_events.push(AudioEvent::FactionAlliance);
            }

            // === PROGRESSION AUDIO ===
            GameEvent::SkillUnlocked { skill_name, category, .. } => {
                let rarity = match category {
                    SkillCategory::Faction => 0.8,
                    SkillCategory::Combat => 0.5,
                    _ => 0.3,
                };
                audio_events.push(AudioEvent::DiscoveryMade {
                    category: format!("{:?}", category),
                    rarity,
                });
            }

            GameEvent::SkillLevelUp { skill_name, new_level, .. } => {
                audio_events.push(AudioEvent::SkillLevelUp {
                    skill: skill_name.clone(),
                });
            }

            GameEvent::QuestStarted { quest_name, faction, .. } => {
                audio_events.push(AudioEvent::QuestAccepted);
            }

            GameEvent::QuestCompleted { quest_name, .. } => {
                audio_events.push(AudioEvent::QuestCompleted);
            }

            GameEvent::QuestFailed { .. } => {
                audio_events.push(AudioEvent::QuestFailed);
            }

            GameEvent::AchievementUnlocked { achievement_name, rarity, .. } => {
                audio_events.push(AudioEvent::DiscoveryMade {
                    category: "achievement".to_string(),
                    rarity: *rarity,
                });
            }

            // === NPC AUDIO ===
            GameEvent::NpcGreeting { disposition, .. } => {
                audio_events.push(AudioEvent::NpcGreeting);
            }

            GameEvent::NpcDialogueStarted { .. } => {
                audio_events.push(AudioEvent::DialogueStarted);
            }

            GameEvent::NpcDialogueEnded { .. } => {
                audio_events.push(AudioEvent::DialogueEnded);
            }

            GameEvent::NpcTradeStarted { .. } => {
                audio_events.push(AudioEvent::TradeStarted);
            }

            GameEvent::NpcTradeCompleted { profit, .. } => {
                audio_events.push(AudioEvent::TradeCompleted { profit: *profit });
            }

            // === DISCOVERY AUDIO ===
            GameEvent::LocationDiscovered { location_type, first_discovery, .. } => {
                let rarity = match location_type {
                    LocationType::SacredSite | LocationType::Dungeon => 0.9,
                    LocationType::Ruin | LocationType::Cave => 0.6,
                    LocationType::Waterfall | LocationType::Viewpoint => 0.4,
                    _ => 0.2,
                };

                if *first_discovery {
                    audio_events.push(AudioEvent::DiscoveryMade {
                        category: format!("{:?}", location_type),
                        rarity,
                    });
                }
            }

            GameEvent::EncyclopediaEntryUnlocked { entry_name, category, .. } => {
                audio_events.push(AudioEvent::EncyclopediaUnlock {
                    entry_type: category.clone(),
                });
            }

            GameEvent::SecretFound { rarity, .. } => {
                audio_events.push(AudioEvent::DiscoveryMade {
                    category: "secret".to_string(),
                    rarity: *rarity,
                });
            }

            // === COMBAT AUDIO ===
            GameEvent::CombatStarted { enemy_type, .. } => {
                audio_events.push(AudioEvent::AnimalCombatStart {
                    species: enemy_type.clone(),
                });
            }

            GameEvent::CombatVictory { enemy_type, .. } => {
                audio_events.push(AudioEvent::AnimalCombatEnd { victory: true });
            }

            GameEvent::CombatDefeat => {
                audio_events.push(AudioEvent::AnimalCombatEnd { victory: false });
            }

            GameEvent::BossEncounter { boss_name } => {
                audio_events.push(AudioEvent::AnimalCombatStart {
                    species: boss_name.clone(),
                });
                // Boss encounters also get discovery feel
                audio_events.push(AudioEvent::DiscoveryMade {
                    category: "boss".to_string(),
                    rarity: 1.0,
                });
            }

            // === ENVIRONMENT AUDIO ===
            GameEvent::BiomeChanged { from, to } => {
                audio_events.push(AudioEvent::BiomeEntered(*to));
            }

            GameEvent::TimeOfDayChanged { period } => {
                match period {
                    TimePeriod::Dawn => audio_events.push(AudioEvent::SunriseBegins),
                    TimePeriod::Dusk => audio_events.push(AudioEvent::SunsetBegins),
                    _ => {}
                }
            }

            GameEvent::CaveEntered { depth, .. } => {
                audio_events.push(AudioEvent::CaveEntered { depth: *depth });
            }

            GameEvent::CaveExited => {
                audio_events.push(AudioEvent::CaveExited);
            }

            GameEvent::WaterfallDiscovered { distance } => {
                audio_events.push(AudioEvent::WaterfallNearby { distance: *distance });
            }

            GameEvent::WeatherChanged { new_weather, intensity } => {
                if new_weather == "storm" && *intensity > 0.5 {
                    audio_events.push(AudioEvent::StormApproaching);
                }
            }

            _ => {}
        }

        Ok(audio_events)
    }

    /// Update internal context based on event
    fn update_context(&mut self, event: &GameEvent) {
        match event {
            GameEvent::FactionTerritoryEntered { faction, village_name } => {
                self.current_faction_territory = Some(*faction);
                self.current_village = village_name.clone();
            }
            GameEvent::FactionTerritoryExited { .. } => {
                self.current_faction_territory = None;
                self.current_village = None;
            }
            _ => {}
        }
    }

    /// Log an error
    fn log_error(&mut self, error: PipelineError) {
        log::warn!("[PIPELINE] {}", error);
        if self.errors.len() >= self.max_errors {
            self.errors.pop_front();
        }
        self.errors.push_back(error);
    }

    /// Get pipeline statistics
    pub fn stats(&self) -> &PipelineStats {
        &self.stats
    }

    /// Get recent errors
    pub fn recent_errors(&self) -> &VecDeque<PipelineError> {
        &self.errors
    }

    /// Get current faction territory
    pub fn current_territory(&self) -> Option<Faction> {
        self.current_faction_territory
    }

    /// Get current village name
    pub fn current_village(&self) -> Option<&String> {
        self.current_village.as_ref()
    }
}

impl Default for DataPipeline {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// NPC AUDIO INTEGRATION
// ============================================================================

/// Handles NPC-specific audio triggers
pub struct NpcAudioIntegration {
    /// Recent interactions for cooldown tracking
    recent_interactions: HashMap<u32, f32>,
    /// Cooldown between NPC audio triggers
    interaction_cooldown: f32,
}

impl NpcAudioIntegration {
    pub fn new() -> Self {
        Self {
            recent_interactions: HashMap::new(),
            interaction_cooldown: 2.0,
        }
    }

    /// Check if NPC interaction should trigger audio
    pub fn should_trigger(&mut self, npc_id: u32, current_time: f32) -> bool {
        let last_time = self.recent_interactions.get(&npc_id).copied().unwrap_or(-100.0);
        if current_time - last_time >= self.interaction_cooldown {
            self.recent_interactions.insert(npc_id, current_time);
            true
        } else {
            false
        }
    }

    /// Generate NPC approach audio event
    pub fn on_npc_approach(&mut self, npc_id: u32, npc_data: &NpcFactionData, current_time: f32) -> Option<GameEvent> {
        if self.should_trigger(npc_id, current_time) {
            let disposition = if npc_data.personal_rep_modifier > 20 {
                NpcDispositionLevel::Friendly
            } else if npc_data.personal_rep_modifier < -20 {
                NpcDispositionLevel::Suspicious
            } else {
                NpcDispositionLevel::Neutral
            };

            Some(GameEvent::NpcGreeting {
                npc_id,
                npc_name: format!("NPC_{}", npc_id),
                disposition,
            })
        } else {
            None
        }
    }

    /// Cleanup old entries
    pub fn cleanup(&mut self, current_time: f32) {
        self.recent_interactions.retain(|_, time| current_time - *time < 60.0);
    }
}

impl Default for NpcAudioIntegration {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PROGRESSION AUDIO BRIDGE
// ============================================================================

/// Bridges progression events to audio system
pub struct ProgressionAudioBridge {
    /// Skill level thresholds for special audio
    milestone_levels: Vec<u32>,
    /// Quest completion streaks
    quest_streak: u32,
    /// Last skill levels for comparison
    last_skill_levels: HashMap<String, u32>,
}

impl ProgressionAudioBridge {
    pub fn new() -> Self {
        Self {
            milestone_levels: vec![5, 10, 25, 50, 75, 100],
            quest_streak: 0,
            last_skill_levels: HashMap::new(),
        }
    }

    /// Check if skill level is a milestone
    pub fn is_milestone_level(&self, level: u32) -> bool {
        self.milestone_levels.contains(&level)
    }

    /// Track quest completion for streak bonus
    pub fn on_quest_complete(&mut self) {
        self.quest_streak += 1;
    }

    /// Reset quest streak on failure
    pub fn on_quest_fail(&mut self) {
        self.quest_streak = 0;
    }

    /// Get current quest streak
    pub fn quest_streak(&self) -> u32 {
        self.quest_streak
    }

    /// Check if skill leveled up and should trigger audio
    pub fn check_skill_levelup(&mut self, skill_id: &str, new_level: u32) -> Option<GameEvent> {
        let old_level = self.last_skill_levels.get(skill_id).copied().unwrap_or(0);
        self.last_skill_levels.insert(skill_id.to_string(), new_level);

        if new_level > old_level {
            Some(GameEvent::SkillLevelUp {
                skill_id: skill_id.to_string(),
                skill_name: skill_id.to_string(),
                new_level,
            })
        } else {
            None
        }
    }
}

impl Default for ProgressionAudioBridge {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// FACTION AUDIO BRIDGE
// ============================================================================

/// Bridges faction system to audio
pub struct FactionAudioBridge {
    /// Tracked faction standings
    last_standings: HashMap<Faction, Standing>,
    /// Territory entry/exit tracking
    in_territory: HashMap<Faction, bool>,
}

impl FactionAudioBridge {
    pub fn new() -> Self {
        Self {
            last_standings: HashMap::new(),
            in_territory: HashMap::new(),
        }
    }

    /// Check for standing changes and generate events
    pub fn check_standing_change(&mut self, faction: Faction, new_standing: Standing) -> Option<GameEvent> {
        let old_standing = self.last_standings.get(&faction).copied().unwrap_or(Standing::Neutral);
        self.last_standings.insert(faction, new_standing);

        if old_standing != new_standing {
            Some(GameEvent::FactionStandingChanged {
                faction,
                old_standing,
                new_standing,
            })
        } else {
            None
        }
    }

    /// Check for territory entry
    pub fn check_territory_entry(&mut self, faction: Faction, village_name: Option<String>) -> Option<GameEvent> {
        let was_in = self.in_territory.get(&faction).copied().unwrap_or(false);
        if !was_in {
            self.in_territory.insert(faction, true);
            Some(GameEvent::FactionTerritoryEntered { faction, village_name })
        } else {
            None
        }
    }

    /// Check for territory exit
    pub fn check_territory_exit(&mut self, faction: Faction) -> Option<GameEvent> {
        let was_in = self.in_territory.get(&faction).copied().unwrap_or(false);
        if was_in {
            self.in_territory.insert(faction, false);
            Some(GameEvent::FactionTerritoryExited { faction })
        } else {
            None
        }
    }
}

impl Default for FactionAudioBridge {
    fn default() -> Self {
        Self::new()
    }
}
