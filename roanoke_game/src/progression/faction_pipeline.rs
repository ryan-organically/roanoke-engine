//! Faction Data Pipelines
//!
//! Hardened data pipelines for faction system with validation, error recovery,
//! and robust cross-system communication.

use super::faction::{Faction, Standing};
use super::faction_integration::{
    FactionEvent, FactionNotification, FactionSaveData, NpcFactionData, VillageFaction,
    NotificationImportance, FactionNotificationType,
};
use super::faction_manager::FactionManager;
use super::faction_skills::FactionSkillId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

// ============================================================================
// PIPELINE ERROR TYPES
// ============================================================================

/// Errors that can occur in faction pipelines
#[derive(Debug, Clone)]
pub enum PipelineError {
    /// Invalid faction reference
    InvalidFaction(String),
    /// Validation failed
    ValidationFailed(String),
    /// Data corruption detected
    DataCorruption(String),
    /// Pipeline overflow (too many items)
    PipelineOverflow { capacity: usize, attempted: usize },
    /// Sync failure between systems
    SyncFailure(String),
    /// Save/load error
    PersistenceError(String),
    /// Event processing error
    EventProcessingError(String),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFaction(s) => write!(f, "Invalid faction: {}", s),
            Self::ValidationFailed(s) => write!(f, "Validation failed: {}", s),
            Self::DataCorruption(s) => write!(f, "Data corruption: {}", s),
            Self::PipelineOverflow { capacity, attempted } => {
                write!(f, "Pipeline overflow: {} items attempted, {} capacity", attempted, capacity)
            }
            Self::SyncFailure(s) => write!(f, "Sync failure: {}", s),
            Self::PersistenceError(s) => write!(f, "Persistence error: {}", s),
            Self::EventProcessingError(s) => write!(f, "Event processing error: {}", s),
        }
    }
}

pub type PipelineResult<T> = Result<T, PipelineError>;

// ============================================================================
// REPUTATION CHANGE PIPELINE
// ============================================================================

/// A validated reputation change request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChange {
    /// Target faction
    pub faction: Faction,
    /// Delta value (positive or negative)
    pub delta: i32,
    /// Reason for change
    pub reason: String,
    /// Source of change
    pub source: ReputationSource,
    /// Game time when change occurred
    pub timestamp: f64,
    /// Whether this change has been applied
    pub applied: bool,
    /// Change ID for tracking
    pub change_id: u64,
}

/// Source of reputation change for auditing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReputationSource {
    Quest,
    Combat,
    Trade,
    Gift,
    Discovery,
    Dialogue,
    Event,
    System,
    Debug,
}

/// Pipeline for processing reputation changes with validation
#[derive(Debug, Clone)]
pub struct ReputationPipeline {
    /// Pending changes to apply
    pending: VecDeque<ReputationChange>,
    /// Applied changes history (for debugging/undo)
    history: VecDeque<ReputationChange>,
    /// Maximum pending changes before forced flush
    max_pending: usize,
    /// Maximum history size
    max_history: usize,
    /// Next change ID
    next_id: u64,
    /// Pipeline statistics
    stats: PipelineStats,
}

#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    pub total_processed: u64,
    pub total_rejected: u64,
    pub total_applied: u64,
    pub last_process_time: f64,
}

impl Default for ReputationPipeline {
    fn default() -> Self {
        Self {
            pending: VecDeque::with_capacity(100),
            history: VecDeque::with_capacity(500),
            max_pending: 100,
            max_history: 500,
            next_id: 1,
            stats: PipelineStats::default(),
        }
    }
}

impl ReputationPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a reputation change with validation
    pub fn queue_change(
        &mut self,
        faction: Faction,
        delta: i32,
        reason: &str,
        source: ReputationSource,
        timestamp: f64,
    ) -> PipelineResult<u64> {
        // Validate delta
        if delta == 0 {
            return Err(PipelineError::ValidationFailed(
                "Delta cannot be zero".to_string(),
            ));
        }

        // Clamp extreme values
        let clamped_delta = delta.clamp(-1000, 1000);
        if clamped_delta != delta {
            log::warn!(
                "Reputation delta clamped from {} to {} for {:?}",
                delta, clamped_delta, faction
            );
        }

        // Check for pipeline overflow
        if self.pending.len() >= self.max_pending {
            return Err(PipelineError::PipelineOverflow {
                capacity: self.max_pending,
                attempted: self.pending.len() + 1,
            });
        }

        let change_id = self.next_id;
        self.next_id += 1;

        self.pending.push_back(ReputationChange {
            faction,
            delta: clamped_delta,
            reason: reason.to_string(),
            source,
            timestamp,
            applied: false,
            change_id,
        });

        Ok(change_id)
    }

    /// Process all pending changes
    pub fn process_all(
        &mut self,
        faction_manager: &mut FactionManager,
        game_time: f64,
    ) -> Vec<ReputationChangeResult> {
        let mut results = Vec::with_capacity(self.pending.len());

        while let Some(mut change) = self.pending.pop_front() {
            let result = self.apply_change(&mut change, faction_manager, game_time);
            results.push(result);

            // Move to history
            change.applied = true;
            self.history.push_back(change);

            // Trim history if needed
            while self.history.len() > self.max_history {
                self.history.pop_front();
            }
        }

        self.stats.total_processed += results.len() as u64;
        self.stats.total_applied += results.iter().filter(|r| r.success).count() as u64;
        self.stats.last_process_time = game_time;

        results
    }

    fn apply_change(
        &mut self,
        change: &mut ReputationChange,
        faction_manager: &mut FactionManager,
        game_time: f64,
    ) -> ReputationChangeResult {
        let old_standing = faction_manager.get_standing(change.faction);
        let old_reputation = faction_manager.get_reputation(change.faction);

        faction_manager.modify_reputation(
            change.faction,
            change.delta,
            &change.reason,
            game_time,
        );

        let new_standing = faction_manager.get_standing(change.faction);
        let new_reputation = faction_manager.get_reputation(change.faction);

        ReputationChangeResult {
            change_id: change.change_id,
            faction: change.faction,
            old_reputation,
            new_reputation,
            old_standing,
            new_standing,
            standing_changed: old_standing != new_standing,
            success: true,
            error: None,
        }
    }

    /// Get pipeline statistics
    pub fn stats(&self) -> &PipelineStats {
        &self.stats
    }

    /// Get pending count
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Clear all pending changes (emergency use only)
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }
}

/// Result of a reputation change
#[derive(Debug, Clone)]
pub struct ReputationChangeResult {
    pub change_id: u64,
    pub faction: Faction,
    pub old_reputation: i32,
    pub new_reputation: i32,
    pub old_standing: Standing,
    pub new_standing: Standing,
    pub standing_changed: bool,
    pub success: bool,
    pub error: Option<String>,
}

// ============================================================================
// EVENT VALIDATION PIPELINE
// ============================================================================

/// Validates faction events before processing
#[derive(Debug, Clone, Default)]
pub struct EventValidator {
    /// Event types that are temporarily disabled
    disabled_events: Vec<String>,
    /// Rate limiters per event type
    rate_limits: HashMap<String, RateLimiter>,
}

#[derive(Debug, Clone)]
struct RateLimiter {
    max_per_second: f32,
    last_event_time: f64,
    event_count: u32,
    window_start: f64,
}

impl RateLimiter {
    fn new(max_per_second: f32) -> Self {
        Self {
            max_per_second,
            last_event_time: 0.0,
            event_count: 0,
            window_start: 0.0,
        }
    }

    fn allow(&mut self, current_time: f64) -> bool {
        // Reset window if needed
        if current_time - self.window_start >= 1.0 {
            self.window_start = current_time;
            self.event_count = 0;
        }

        if (self.event_count as f32) < self.max_per_second {
            self.event_count += 1;
            self.last_event_time = current_time;
            true
        } else {
            false
        }
    }
}

impl EventValidator {
    pub fn new() -> Self {
        let mut validator = Self::default();

        // Set default rate limits
        validator.set_rate_limit("MemberAttacked", 10.0);
        validator.set_rate_limit("MemberKilled", 5.0);
        validator.set_rate_limit("TradeCompleted", 20.0);
        validator.set_rate_limit("GiftGiven", 10.0);

        validator
    }

    /// Set rate limit for an event type
    pub fn set_rate_limit(&mut self, event_type: &str, max_per_second: f32) {
        self.rate_limits.insert(
            event_type.to_string(),
            RateLimiter::new(max_per_second),
        );
    }

    /// Validate an event
    pub fn validate(&mut self, event: &FactionEvent, game_time: f64) -> PipelineResult<()> {
        let event_type = self.get_event_type(event);

        // Check if disabled
        if self.disabled_events.contains(&event_type) {
            return Err(PipelineError::ValidationFailed(format!(
                "Event type {} is disabled",
                event_type
            )));
        }

        // Check rate limit
        if let Some(limiter) = self.rate_limits.get_mut(&event_type) {
            if !limiter.allow(game_time) {
                return Err(PipelineError::ValidationFailed(format!(
                    "Rate limit exceeded for {}",
                    event_type
                )));
            }
        }

        // Validate event-specific constraints
        self.validate_event_data(event)?;

        Ok(())
    }

    fn get_event_type(&self, event: &FactionEvent) -> String {
        match event {
            FactionEvent::QuestCompleted { .. } => "QuestCompleted".to_string(),
            FactionEvent::QuestFailed { .. } => "QuestFailed".to_string(),
            FactionEvent::TradeCompleted { .. } => "TradeCompleted".to_string(),
            FactionEvent::GiftGiven { .. } => "GiftGiven".to_string(),
            FactionEvent::MemberAttacked { .. } => "MemberAttacked".to_string(),
            FactionEvent::MemberKilled { .. } => "MemberKilled".to_string(),
            FactionEvent::SettlementDefended { .. } => "SettlementDefended".to_string(),
            FactionEvent::SacredSiteDesecrated { .. } => "SacredSiteDesecrated".to_string(),
            FactionEvent::DiscoveryMade { .. } => "DiscoveryMade".to_string(),
            FactionEvent::StandingChanged { .. } => "StandingChanged".to_string(),
            FactionEvent::BloodBondFormed { .. } => "BloodBondFormed".to_string(),
            FactionEvent::WarDeclared { .. } => "WarDeclared".to_string(),
            FactionEvent::SkillUnlocked { .. } => "SkillUnlocked".to_string(),
            FactionEvent::AbilityUnlocked { .. } => "AbilityUnlocked".to_string(),
            FactionEvent::WeaponUnlocked { .. } => "WeaponUnlocked".to_string(),
            FactionEvent::FactionRelationshipChanged { .. } => "FactionRelationshipChanged".to_string(),
            FactionEvent::PrimaryFactionChosen { .. } => "PrimaryFactionChosen".to_string(),
            FactionEvent::FactionBetrayed { .. } => "FactionBetrayed".to_string(),
        }
    }

    fn validate_event_data(&self, event: &FactionEvent) -> PipelineResult<()> {
        match event {
            FactionEvent::QuestCompleted { reputation_gain, .. } => {
                if *reputation_gain < 0 {
                    return Err(PipelineError::ValidationFailed(
                        "Quest completion cannot have negative reputation".to_string(),
                    ));
                }
                if *reputation_gain > 500 {
                    return Err(PipelineError::ValidationFailed(
                        "Quest reputation gain too high".to_string(),
                    ));
                }
            }
            FactionEvent::TradeCompleted { value, .. } => {
                if *value < 0 {
                    return Err(PipelineError::ValidationFailed(
                        "Trade value cannot be negative".to_string(),
                    ));
                }
            }
            FactionEvent::MemberAttacked { damage, .. } => {
                if *damage < 0.0 {
                    return Err(PipelineError::ValidationFailed(
                        "Damage cannot be negative".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Disable an event type temporarily
    pub fn disable_event_type(&mut self, event_type: &str) {
        if !self.disabled_events.contains(&event_type.to_string()) {
            self.disabled_events.push(event_type.to_string());
        }
    }

    /// Enable an event type
    pub fn enable_event_type(&mut self, event_type: &str) {
        self.disabled_events.retain(|e| e != event_type);
    }
}

// ============================================================================
// NOTIFICATION DISPATCH SYSTEM
// ============================================================================

/// Priority queue for notification dispatch
#[derive(Debug, Clone, Default)]
pub struct NotificationDispatcher {
    /// Queued notifications by priority
    critical: VecDeque<FactionNotification>,
    high: VecDeque<FactionNotification>,
    medium: VecDeque<FactionNotification>,
    low: VecDeque<FactionNotification>,
    /// Maximum notifications to dispatch per frame
    max_per_frame: usize,
    /// Notification handlers
    handlers: Vec<NotificationHandler>,
    /// Stats
    total_dispatched: u64,
    total_dropped: u64,
}

#[derive(Debug, Clone)]
pub struct NotificationHandler {
    pub name: String,
    pub min_importance: NotificationImportance,
    pub faction_filter: Option<Faction>,
}

impl NotificationDispatcher {
    pub fn new() -> Self {
        Self {
            critical: VecDeque::with_capacity(10),
            high: VecDeque::with_capacity(50),
            medium: VecDeque::with_capacity(100),
            low: VecDeque::with_capacity(100),
            max_per_frame: 5,
            handlers: Vec::new(),
            total_dispatched: 0,
            total_dropped: 0,
        }
    }

    /// Queue a notification for dispatch
    pub fn queue(&mut self, notification: FactionNotification) {
        let queue = match notification.importance {
            NotificationImportance::Critical => &mut self.critical,
            NotificationImportance::High => &mut self.high,
            NotificationImportance::Medium => &mut self.medium,
            NotificationImportance::Low => &mut self.low,
        };

        // Check capacity
        let max_size = match notification.importance {
            NotificationImportance::Critical => 10,
            NotificationImportance::High => 50,
            NotificationImportance::Medium => 100,
            NotificationImportance::Low => 100,
        };

        if queue.len() >= max_size {
            // Drop oldest for non-critical
            if notification.importance != NotificationImportance::Critical {
                queue.pop_front();
                self.total_dropped += 1;
            }
        }

        queue.push_back(notification);
    }

    /// Dispatch queued notifications
    pub fn dispatch(&mut self) -> Vec<FactionNotification> {
        let mut dispatched = Vec::with_capacity(self.max_per_frame);

        // Critical first
        while dispatched.len() < self.max_per_frame {
            if let Some(n) = self.critical.pop_front() {
                dispatched.push(n);
            } else {
                break;
            }
        }

        // Then high
        while dispatched.len() < self.max_per_frame {
            if let Some(n) = self.high.pop_front() {
                dispatched.push(n);
            } else {
                break;
            }
        }

        // Then medium
        while dispatched.len() < self.max_per_frame {
            if let Some(n) = self.medium.pop_front() {
                dispatched.push(n);
            } else {
                break;
            }
        }

        // Then low
        while dispatched.len() < self.max_per_frame {
            if let Some(n) = self.low.pop_front() {
                dispatched.push(n);
            } else {
                break;
            }
        }

        self.total_dispatched += dispatched.len() as u64;
        dispatched
    }

    /// Get total pending count
    pub fn pending_count(&self) -> usize {
        self.critical.len() + self.high.len() + self.medium.len() + self.low.len()
    }

    /// Set max notifications per frame
    pub fn set_max_per_frame(&mut self, max: usize) {
        self.max_per_frame = max.max(1);
    }

    /// Get dispatch statistics
    pub fn stats(&self) -> (u64, u64) {
        (self.total_dispatched, self.total_dropped)
    }
}

// ============================================================================
// FACTION SYNC PIPELINE
// ============================================================================

/// Sync state between different faction-aware systems
#[derive(Debug, Clone, Default)]
pub struct FactionSyncPipeline {
    /// Pending sync operations
    pending_syncs: Vec<SyncOperation>,
    /// Last sync times per system
    last_sync: HashMap<String, f64>,
    /// Sync interval in seconds
    sync_interval: f64,
    /// Whether sync is currently in progress
    syncing: bool,
}

#[derive(Debug, Clone)]
pub enum SyncOperation {
    /// Sync village faction data
    SyncVillage {
        village_id: u32,
        faction: Faction,
        local_reputation: i32,
    },
    /// Sync NPC faction data
    SyncNpc {
        npc_id: u32,
        faction: Faction,
        personal_rep: i32,
    },
    /// Sync player standing to all systems
    SyncPlayerStanding {
        faction: Faction,
        standing: Standing,
    },
    /// Full faction state sync
    FullSync,
}

impl FactionSyncPipeline {
    pub fn new() -> Self {
        Self {
            pending_syncs: Vec::new(),
            last_sync: HashMap::new(),
            sync_interval: 1.0, // Sync every second
            syncing: false,
        }
    }

    /// Queue a sync operation
    pub fn queue_sync(&mut self, op: SyncOperation) {
        // Deduplicate similar syncs
        let dominated = self.pending_syncs.iter().any(|existing| {
            // FullSync dominates everything
            if matches!(&op, SyncOperation::FullSync) {
                return true;
            }
            // Check if same faction standing sync already exists
            if let (SyncOperation::SyncPlayerStanding { faction: f1, .. },
                    SyncOperation::SyncPlayerStanding { faction: f2, .. }) = (&op, existing) {
                return f1 == f2;
            }
            false
        });

        if !dominated {
            self.pending_syncs.push(op);
        }
    }

    /// Check if sync is needed for a system
    pub fn needs_sync(&self, system: &str, current_time: f64) -> bool {
        self.last_sync
            .get(system)
            .map(|t| current_time - t > self.sync_interval)
            .unwrap_or(true)
    }

    /// Mark system as synced
    pub fn mark_synced(&mut self, system: &str, current_time: f64) {
        self.last_sync.insert(system.to_string(), current_time);
    }

    /// Get pending sync operations
    pub fn take_pending(&mut self) -> Vec<SyncOperation> {
        std::mem::take(&mut self.pending_syncs)
    }

    /// Set sync interval
    pub fn set_interval(&mut self, seconds: f64) {
        self.sync_interval = seconds.max(0.1);
    }
}

// ============================================================================
// SAVE/LOAD PIPELINE WITH ERROR RECOVERY
// ============================================================================

/// Hardened save/load pipeline with validation and recovery
#[derive(Debug, Clone, Default)]
pub struct PersistencePipeline {
    /// Backup data for recovery
    backup: Option<FactionSaveData>,
    /// Validation errors from last operation
    last_errors: Vec<String>,
    /// Whether data is dirty (needs save)
    dirty: bool,
    /// Last successful save time
    last_save_time: f64,
    /// Auto-save interval
    auto_save_interval: f64,
}

impl PersistencePipeline {
    pub fn new() -> Self {
        Self {
            backup: None,
            last_errors: Vec::new(),
            dirty: false,
            last_save_time: 0.0,
            auto_save_interval: 300.0, // 5 minutes
        }
    }

    /// Validate save data before writing
    pub fn validate_save_data(&mut self, data: &FactionSaveData) -> PipelineResult<()> {
        self.last_errors.clear();

        // Check version
        if data.version == 0 {
            self.last_errors.push("Invalid version 0".to_string());
        }

        // Check for data corruption
        for (faction, rep_data) in &data.manager.reputations {
            if rep_data.reputation < -10000 || rep_data.reputation > 10000 {
                self.last_errors.push(format!(
                    "Invalid reputation {} for {:?}",
                    rep_data.reputation, faction
                ));
            }
            if rep_data.max_reached < rep_data.min_reached {
                self.last_errors.push(format!(
                    "max_reached < min_reached for {:?}",
                    faction
                ));
            }
        }

        // Check village factions
        for (id, village) in &data.village_factions {
            if village.local_reputation < -10000 || village.local_reputation > 10000 {
                self.last_errors.push(format!(
                    "Invalid village {} local reputation: {}",
                    id, village.local_reputation
                ));
            }
        }

        // Check NPC data
        for (id, npc) in &data.npc_faction_data {
            if npc.loyalty < 0.0 || npc.loyalty > 1.0 {
                self.last_errors.push(format!(
                    "Invalid NPC {} loyalty: {}",
                    id, npc.loyalty
                ));
            }
        }

        if self.last_errors.is_empty() {
            Ok(())
        } else {
            Err(PipelineError::ValidationFailed(
                self.last_errors.join("; ")
            ))
        }
    }

    /// Prepare data for save (with backup)
    pub fn prepare_save(&mut self, data: &FactionSaveData) -> PipelineResult<FactionSaveData> {
        // Validate first
        self.validate_save_data(data)?;

        // Create backup
        self.backup = Some(data.clone());

        // Mark as not dirty (will be saved)
        self.dirty = false;

        Ok(data.clone())
    }

    /// Validate and sanitize loaded data
    pub fn process_load(&mut self, mut data: FactionSaveData) -> PipelineResult<FactionSaveData> {
        // Validate
        if let Err(e) = self.validate_save_data(&data) {
            log::warn!("Loaded save has issues: {}", e);
            // Try to fix recoverable issues
            self.attempt_recovery(&mut data);
        }

        // Migrate if needed
        if data.version < FactionSaveData::CURRENT_VERSION {
            data = self.migrate_data(data)?;
        }

        Ok(data)
    }

    /// Attempt to recover from data issues
    fn attempt_recovery(&self, data: &mut FactionSaveData) {
        // Clamp reputation values
        for rep in data.manager.reputations.values_mut() {
            rep.reputation = rep.reputation.clamp(-2000, 2000);
            rep.max_reached = rep.max_reached.max(rep.reputation);
            rep.min_reached = rep.min_reached.min(rep.reputation);
        }

        // Clamp village reputations
        for village in data.village_factions.values_mut() {
            village.local_reputation = village.local_reputation.clamp(-2000, 2000);
        }

        // Fix NPC loyalty
        for npc in data.npc_faction_data.values_mut() {
            npc.loyalty = npc.loyalty.clamp(0.0, 1.0);
        }

        log::info!("Attempted data recovery on loaded save");
    }

    /// Migrate data from older version
    fn migrate_data(&self, mut data: FactionSaveData) -> PipelineResult<FactionSaveData> {
        let from_version = data.version;

        // Version 0 -> 1: Add default skill points
        if data.version == 0 {
            for faction in Faction::all_playable() {
                data.manager.skill_points.entry(*faction).or_insert(0);
            }
            data.version = 1;
        }

        log::info!("Migrated save data from v{} to v{}", from_version, data.version);
        Ok(data)
    }

    /// Restore from backup
    pub fn restore_backup(&mut self) -> Option<FactionSaveData> {
        self.backup.take()
    }

    /// Check if auto-save is needed
    pub fn needs_auto_save(&self, current_time: f64) -> bool {
        self.dirty && (current_time - self.last_save_time > self.auto_save_interval)
    }

    /// Mark data as dirty (needs save)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Mark save completed
    pub fn mark_saved(&mut self, time: f64) {
        self.dirty = false;
        self.last_save_time = time;
    }

    /// Get last errors
    pub fn last_errors(&self) -> &[String] {
        &self.last_errors
    }
}

// ============================================================================
// MASTER PIPELINE COORDINATOR
// ============================================================================

/// Coordinates all faction pipelines
#[derive(Debug, Clone)]
pub struct FactionPipelineCoordinator {
    pub reputation: ReputationPipeline,
    pub validator: EventValidator,
    pub notifications: NotificationDispatcher,
    pub sync: FactionSyncPipeline,
    pub persistence: PersistencePipeline,
    /// Whether pipelines are paused
    paused: bool,
    /// Error log
    error_log: VecDeque<(f64, PipelineError)>,
}

impl Default for FactionPipelineCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl FactionPipelineCoordinator {
    pub fn new() -> Self {
        Self {
            reputation: ReputationPipeline::new(),
            validator: EventValidator::new(),
            notifications: NotificationDispatcher::new(),
            sync: FactionSyncPipeline::new(),
            persistence: PersistencePipeline::new(),
            paused: false,
            error_log: VecDeque::with_capacity(100),
        }
    }

    /// Process reputation change through pipeline
    pub fn process_reputation_change(
        &mut self,
        faction: Faction,
        delta: i32,
        reason: &str,
        source: ReputationSource,
        game_time: f64,
    ) -> PipelineResult<u64> {
        if self.paused {
            return Err(PipelineError::ValidationFailed("Pipelines paused".to_string()));
        }

        let result = self.reputation.queue_change(faction, delta, reason, source, game_time);

        if let Err(ref e) = result {
            self.log_error(game_time, e.clone());
        }

        // Mark data as dirty
        self.persistence.mark_dirty();

        result
    }

    /// Validate and queue a faction event
    pub fn queue_validated_event(
        &mut self,
        event: FactionEvent,
        game_time: f64,
    ) -> PipelineResult<()> {
        if self.paused {
            return Err(PipelineError::ValidationFailed("Pipelines paused".to_string()));
        }

        // Validate event
        self.validator.validate(&event, game_time)?;

        // Mark dirty
        self.persistence.mark_dirty();

        Ok(())
    }

    /// Queue a notification for dispatch
    pub fn queue_notification(&mut self, notification: FactionNotification) {
        self.notifications.queue(notification);
    }

    /// Dispatch pending notifications
    pub fn dispatch_notifications(&mut self) -> Vec<FactionNotification> {
        self.notifications.dispatch()
    }

    /// Process all pending reputation changes
    pub fn flush_reputation_changes(
        &mut self,
        faction_manager: &mut FactionManager,
        game_time: f64,
    ) -> Vec<ReputationChangeResult> {
        if self.paused {
            return Vec::new();
        }

        let results = self.reputation.process_all(faction_manager, game_time);

        // Queue standing change notifications
        for result in &results {
            if result.standing_changed {
                self.queue_notification(FactionNotification {
                    faction: result.faction,
                    notification_type: if result.new_standing > result.old_standing {
                        FactionNotificationType::StandingUp
                    } else {
                        FactionNotificationType::StandingDown
                    },
                    message: format!(
                        "Standing with {} changed to {:?}",
                        result.faction.display_name(),
                        result.new_standing
                    ),
                    importance: NotificationImportance::High,
                    timestamp: game_time,
                    displayed: false,
                });

                // Queue sync
                self.sync.queue_sync(SyncOperation::SyncPlayerStanding {
                    faction: result.faction,
                    standing: result.new_standing,
                });
            }
        }

        results
    }

    /// Pause all pipelines
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume all pipelines
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Log an error
    fn log_error(&mut self, time: f64, error: PipelineError) {
        log::error!("[FactionPipeline] {}", error);
        self.error_log.push_back((time, error));

        // Trim log
        while self.error_log.len() > 100 {
            self.error_log.pop_front();
        }
    }

    /// Get recent errors
    pub fn recent_errors(&self) -> &VecDeque<(f64, PipelineError)> {
        &self.error_log
    }

    /// Get pipeline health report
    pub fn health_report(&self) -> PipelineHealthReport {
        PipelineHealthReport {
            reputation_pending: self.reputation.pending_count(),
            notification_pending: self.notifications.pending_count(),
            sync_pending: self.sync.pending_syncs.len(),
            error_count: self.error_log.len(),
            paused: self.paused,
            dirty: self.persistence.dirty,
            reputation_stats: self.reputation.stats().clone(),
            notification_stats: self.notifications.stats(),
        }
    }
}

/// Health report for pipeline monitoring
#[derive(Debug, Clone)]
pub struct PipelineHealthReport {
    pub reputation_pending: usize,
    pub notification_pending: usize,
    pub sync_pending: usize,
    pub error_count: usize,
    pub paused: bool,
    pub dirty: bool,
    pub reputation_stats: PipelineStats,
    pub notification_stats: (u64, u64), // (dispatched, dropped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_pipeline() {
        let mut pipeline = ReputationPipeline::new();

        let id = pipeline.queue_change(
            Faction::Powhatan,
            100,
            "Test",
            ReputationSource::Quest,
            0.0,
        ).unwrap();

        assert_eq!(id, 1);
        assert_eq!(pipeline.pending_count(), 1);
    }

    #[test]
    fn test_reputation_validation() {
        let mut pipeline = ReputationPipeline::new();

        // Zero delta should fail
        let result = pipeline.queue_change(
            Faction::Powhatan,
            0,
            "Test",
            ReputationSource::Quest,
            0.0,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_event_validator() {
        let mut validator = EventValidator::new();

        let event = FactionEvent::QuestCompleted {
            faction: Faction::Cherokee,
            quest_id: "test".to_string(),
            reputation_gain: 50,
        };

        assert!(validator.validate(&event, 0.0).is_ok());
    }

    #[test]
    fn test_event_validation_negative_quest_rep() {
        let mut validator = EventValidator::new();

        let event = FactionEvent::QuestCompleted {
            faction: Faction::Cherokee,
            quest_id: "test".to_string(),
            reputation_gain: -50, // Invalid
        };

        assert!(validator.validate(&event, 0.0).is_err());
    }

    #[test]
    fn test_notification_dispatcher() {
        let mut dispatcher = NotificationDispatcher::new();

        dispatcher.queue(FactionNotification {
            faction: Faction::Powhatan,
            notification_type: FactionNotificationType::ReputationGain,
            message: "Test".to_string(),
            importance: NotificationImportance::High,
            timestamp: 0.0,
            displayed: false,
        });

        let dispatched = dispatcher.dispatch();
        assert_eq!(dispatched.len(), 1);
    }

    #[test]
    fn test_notification_priority() {
        let mut dispatcher = NotificationDispatcher::new();
        dispatcher.set_max_per_frame(1);

        // Queue low priority first
        dispatcher.queue(FactionNotification {
            faction: Faction::Powhatan,
            notification_type: FactionNotificationType::ReputationGain,
            message: "Low".to_string(),
            importance: NotificationImportance::Low,
            timestamp: 0.0,
            displayed: false,
        });

        // Queue critical second
        dispatcher.queue(FactionNotification {
            faction: Faction::Cherokee,
            notification_type: FactionNotificationType::WarDeclared,
            message: "Critical".to_string(),
            importance: NotificationImportance::Critical,
            timestamp: 0.0,
            displayed: false,
        });

        // Critical should dispatch first
        let dispatched = dispatcher.dispatch();
        assert_eq!(dispatched.len(), 1);
        assert_eq!(dispatched[0].message, "Critical");
    }

    #[test]
    fn test_coordinator() {
        let mut coordinator = FactionPipelineCoordinator::new();
        let mut manager = FactionManager::new();

        coordinator.process_reputation_change(
            Faction::Powhatan,
            100,
            "Test",
            ReputationSource::Quest,
            0.0,
        ).unwrap();

        let results = coordinator.flush_reputation_changes(&mut manager, 0.0);
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }
}
