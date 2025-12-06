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
// TRANSACTION SUPPORT
// ============================================================================

/// A transaction groups multiple reputation changes for atomic commit/rollback
#[derive(Debug, Clone)]
pub struct ReputationTransaction {
    /// Transaction ID
    pub id: u64,
    /// Changes in this transaction
    changes: Vec<ReputationChange>,
    /// Transaction state
    state: TransactionState,
    /// Creation timestamp
    created_at: f64,
    /// Description
    description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Pending,
    Committed,
    RolledBack,
}

impl ReputationTransaction {
    fn new(id: u64, description: &str, timestamp: f64) -> Self {
        Self {
            id,
            changes: Vec::new(),
            state: TransactionState::Pending,
            created_at: timestamp,
            description: description.to_string(),
        }
    }

    /// Add a change to this transaction
    pub fn add_change(&mut self, change: ReputationChange) {
        if self.state == TransactionState::Pending {
            self.changes.push(change);
        }
    }

    /// Get all changes in this transaction
    pub fn changes(&self) -> &[ReputationChange] {
        &self.changes
    }
}

/// Transaction manager for atomic reputation operations
#[derive(Debug, Clone, Default)]
pub struct TransactionManager {
    /// Active transactions
    active: HashMap<u64, ReputationTransaction>,
    /// Completed transactions (for audit)
    completed: VecDeque<ReputationTransaction>,
    /// Next transaction ID
    next_id: u64,
    /// Max completed history
    max_history: usize,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            active: HashMap::new(),
            completed: VecDeque::with_capacity(100),
            next_id: 1,
            max_history: 100,
        }
    }

    /// Begin a new transaction
    pub fn begin(&mut self, description: &str, timestamp: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.active.insert(id, ReputationTransaction::new(id, description, timestamp));
        id
    }

    /// Add a change to a transaction
    pub fn add_to_transaction(&mut self, tx_id: u64, change: ReputationChange) -> PipelineResult<()> {
        if let Some(tx) = self.active.get_mut(&tx_id) {
            tx.add_change(change);
            Ok(())
        } else {
            Err(PipelineError::ValidationFailed(format!("Transaction {} not found", tx_id)))
        }
    }

    /// Commit a transaction - returns changes to apply
    pub fn commit(&mut self, tx_id: u64) -> PipelineResult<Vec<ReputationChange>> {
        if let Some(mut tx) = self.active.remove(&tx_id) {
            tx.state = TransactionState::Committed;
            let changes = tx.changes.clone();
            self.archive_transaction(tx);
            Ok(changes)
        } else {
            Err(PipelineError::ValidationFailed(format!("Transaction {} not found", tx_id)))
        }
    }

    /// Rollback a transaction - discards all changes
    pub fn rollback(&mut self, tx_id: u64) -> PipelineResult<()> {
        if let Some(mut tx) = self.active.remove(&tx_id) {
            tx.state = TransactionState::RolledBack;
            self.archive_transaction(tx);
            Ok(())
        } else {
            Err(PipelineError::ValidationFailed(format!("Transaction {} not found", tx_id)))
        }
    }

    fn archive_transaction(&mut self, tx: ReputationTransaction) {
        self.completed.push_back(tx);
        while self.completed.len() > self.max_history {
            self.completed.pop_front();
        }
    }

    /// Get active transaction count
    pub fn active_count(&self) -> usize {
        self.active.len()
    }
}

// ============================================================================
// UNDO SYSTEM
// ============================================================================

/// Undo manager for reversing reputation changes
#[derive(Debug, Clone)]
pub struct UndoManager {
    /// Undo stack
    undo_stack: VecDeque<UndoEntry>,
    /// Redo stack
    redo_stack: VecDeque<UndoEntry>,
    /// Maximum undo depth
    max_depth: usize,
}

#[derive(Debug, Clone)]
pub struct UndoEntry {
    /// Original change that was applied
    pub change: ReputationChange,
    /// The result of applying the change
    pub result: ReputationChangeResult,
    /// Timestamp when change was applied
    pub applied_at: f64,
}

impl Default for UndoManager {
    fn default() -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(50),
            redo_stack: VecDeque::with_capacity(50),
            max_depth: 50,
        }
    }
}

impl UndoManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a change that was applied (for future undo)
    pub fn record(&mut self, change: ReputationChange, result: ReputationChangeResult, game_time: f64) {
        self.undo_stack.push_back(UndoEntry {
            change,
            result,
            applied_at: game_time,
        });

        // Clear redo stack on new change
        self.redo_stack.clear();

        // Trim undo stack
        while self.undo_stack.len() > self.max_depth {
            self.undo_stack.pop_front();
        }
    }

    /// Pop the last change for undo - returns the reverse delta to apply
    pub fn pop_for_undo(&mut self) -> Option<(Faction, i32, String)> {
        if let Some(entry) = self.undo_stack.pop_back() {
            let reverse_delta = -entry.change.delta;
            let reason = format!("Undo: {}", entry.change.reason);

            // Move to redo stack
            self.redo_stack.push_back(entry.clone());

            Some((entry.change.faction, reverse_delta, reason))
        } else {
            None
        }
    }

    /// Pop from redo stack to redo a change
    pub fn pop_for_redo(&mut self) -> Option<(Faction, i32, String)> {
        if let Some(entry) = self.redo_stack.pop_back() {
            let reason = format!("Redo: {}", entry.change.reason);

            // Move back to undo stack
            self.undo_stack.push_back(entry.clone());

            Some((entry.change.faction, entry.change.delta, reason))
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get undo stack depth
    pub fn undo_depth(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack depth
    pub fn redo_depth(&self) -> usize {
        self.redo_stack.len()
    }
}

// ============================================================================
// ENHANCED METRICS
// ============================================================================

/// Detailed metrics tracking per faction
#[derive(Debug, Clone, Default)]
pub struct FactionMetrics {
    /// Total positive reputation gained
    pub total_gained: i64,
    /// Total negative reputation lost
    pub total_lost: i64,
    /// Number of changes applied
    pub change_count: u32,
    /// Peak reputation reached
    pub peak_reputation: i32,
    /// Lowest reputation reached
    pub lowest_reputation: i32,
    /// Standing changes count
    pub standing_changes: u32,
    /// Last change timestamp
    pub last_change_time: f64,
}

/// Comprehensive pipeline metrics
#[derive(Debug, Clone, Default)]
pub struct PipelineMetrics {
    /// Per-faction metrics
    pub by_faction: HashMap<Faction, FactionMetrics>,
    /// Global statistics
    pub global: GlobalMetrics,
    /// Rate metrics (changes per minute)
    pub rate: RateMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalMetrics {
    /// Total changes processed
    pub total_changes: u64,
    /// Total changes rejected
    pub total_rejected: u64,
    /// Total standing upgrades
    pub standing_upgrades: u32,
    /// Total standing downgrades
    pub standing_downgrades: u32,
    /// Transactions committed
    pub transactions_committed: u32,
    /// Transactions rolled back
    pub transactions_rolled_back: u32,
    /// Undo operations
    pub undo_count: u32,
    /// Redo operations
    pub redo_count: u32,
}

#[derive(Debug, Clone)]
pub struct RateMetrics {
    /// Window start time
    window_start: f64,
    /// Changes in current window
    window_count: u32,
    /// Changes per minute (rolling average)
    pub changes_per_minute: f32,
    /// Peak changes per minute
    pub peak_per_minute: f32,
}

impl Default for RateMetrics {
    fn default() -> Self {
        Self {
            window_start: 0.0,
            window_count: 0,
            changes_per_minute: 0.0,
            peak_per_minute: 0.0,
        }
    }
}

impl RateMetrics {
    /// Record a change and update rate
    pub fn record(&mut self, timestamp: f64) {
        // Reset window if needed (1 minute windows)
        if timestamp - self.window_start >= 60.0 {
            // Update rolling average
            self.changes_per_minute = self.window_count as f32;
            if self.changes_per_minute > self.peak_per_minute {
                self.peak_per_minute = self.changes_per_minute;
            }
            self.window_start = timestamp;
            self.window_count = 0;
        }
        self.window_count += 1;
    }
}

impl PipelineMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a reputation change result
    pub fn record_change(&mut self, result: &ReputationChangeResult, timestamp: f64) {
        // Update faction metrics
        let faction_metrics = self.by_faction
            .entry(result.faction)
            .or_insert_with(FactionMetrics::default);

        faction_metrics.change_count += 1;
        faction_metrics.last_change_time = timestamp;

        let delta = result.new_reputation - result.old_reputation;
        if delta > 0 {
            faction_metrics.total_gained += delta as i64;
        } else {
            faction_metrics.total_lost += (-delta) as i64;
        }

        if result.new_reputation > faction_metrics.peak_reputation {
            faction_metrics.peak_reputation = result.new_reputation;
        }
        if result.new_reputation < faction_metrics.lowest_reputation {
            faction_metrics.lowest_reputation = result.new_reputation;
        }

        if result.standing_changed {
            faction_metrics.standing_changes += 1;
            if result.new_standing > result.old_standing {
                self.global.standing_upgrades += 1;
            } else {
                self.global.standing_downgrades += 1;
            }
        }

        // Update global metrics
        self.global.total_changes += 1;
        self.rate.record(timestamp);
    }

    /// Record a rejected change
    pub fn record_rejection(&mut self) {
        self.global.total_rejected += 1;
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Changes: {} ({} rejected), Upgrades: {}, Downgrades: {}, Rate: {:.1}/min",
            self.global.total_changes,
            self.global.total_rejected,
            self.global.standing_upgrades,
            self.global.standing_downgrades,
            self.rate.changes_per_minute
        )
    }
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

/// Priority queue for notification dispatch with grouping and deduplication
#[derive(Debug, Clone)]
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
    total_deduplicated: u64,
    total_grouped: u64,
    /// Deduplication tracking: (faction, notification_type) -> last timestamp
    recent_notifications: HashMap<(Faction, std::mem::Discriminant<FactionNotificationType>), f64>,
    /// Deduplication window in seconds
    dedup_window: f64,
    /// Grouping enabled
    grouping_enabled: bool,
    /// Per-faction throttle (max notifications per faction per second)
    faction_throttle: HashMap<Faction, FactionThrottle>,
    max_per_faction_per_second: f32,
}

#[derive(Debug, Clone)]
struct FactionThrottle {
    window_start: f64,
    count: u32,
}

#[derive(Debug, Clone)]
pub struct NotificationHandler {
    pub name: String,
    pub min_importance: NotificationImportance,
    pub faction_filter: Option<Faction>,
}

impl Default for NotificationDispatcher {
    fn default() -> Self {
        Self::new()
    }
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
            total_deduplicated: 0,
            total_grouped: 0,
            recent_notifications: HashMap::new(),
            dedup_window: 5.0, // 5 second dedup window
            grouping_enabled: true,
            faction_throttle: HashMap::new(),
            max_per_faction_per_second: 3.0,
        }
    }

    /// Queue a notification with deduplication and throttling
    pub fn queue(&mut self, notification: FactionNotification) {
        let timestamp = notification.timestamp;

        // Check per-faction throttle
        if !self.check_faction_throttle(notification.faction, timestamp) {
            self.total_dropped += 1;
            return;
        }

        // Check deduplication
        if self.is_duplicate(&notification, timestamp) {
            self.total_deduplicated += 1;
            return;
        }

        // Try to group with existing notification
        if self.grouping_enabled && self.try_group_notification(&notification) {
            self.total_grouped += 1;
            return;
        }

        // Record for future dedup checks
        let key = (notification.faction, std::mem::discriminant(&notification.notification_type));
        self.recent_notifications.insert(key, timestamp);

        // Update throttle counter
        self.update_faction_throttle(notification.faction, timestamp);

        // Add to appropriate queue
        self.add_to_queue(notification);
    }

    /// Queue without dedup/throttle (for internal use)
    fn add_to_queue(&mut self, notification: FactionNotification) {
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

    /// Check if notification is a duplicate
    fn is_duplicate(&self, notification: &FactionNotification, current_time: f64) -> bool {
        let key = (notification.faction, std::mem::discriminant(&notification.notification_type));

        if let Some(&last_time) = self.recent_notifications.get(&key) {
            if current_time - last_time < self.dedup_window {
                return true;
            }
        }
        false
    }

    /// Try to group notification with existing one
    fn try_group_notification(&mut self, notification: &FactionNotification) -> bool {
        // Only group reputation-related notifications
        let can_group = matches!(
            notification.notification_type,
            FactionNotificationType::ReputationGain | FactionNotificationType::ReputationLoss
        );

        if !can_group {
            return false;
        }

        // Check all queues for a matching notification to group with
        for queue in [&mut self.high, &mut self.medium, &mut self.low] {
            for existing in queue.iter_mut() {
                if existing.faction == notification.faction
                    && std::mem::discriminant(&existing.notification_type) == std::mem::discriminant(&notification.notification_type)
                {
                    // Update the message to indicate grouping
                    existing.message = format!("{} (multiple)", existing.message.trim_end_matches(" (multiple)"));
                    return true;
                }
            }
        }

        false
    }

    /// Check per-faction throttle
    fn check_faction_throttle(&self, faction: Faction, current_time: f64) -> bool {
        if let Some(throttle) = self.faction_throttle.get(&faction) {
            if current_time - throttle.window_start < 1.0 {
                return (throttle.count as f32) < self.max_per_faction_per_second;
            }
        }
        true
    }

    /// Update per-faction throttle counter
    fn update_faction_throttle(&mut self, faction: Faction, current_time: f64) {
        let throttle = self.faction_throttle.entry(faction).or_insert(FactionThrottle {
            window_start: current_time,
            count: 0,
        });

        if current_time - throttle.window_start >= 1.0 {
            throttle.window_start = current_time;
            throttle.count = 0;
        }
        throttle.count += 1;
    }

    /// Clean up old deduplication entries
    pub fn cleanup_old_entries(&mut self, current_time: f64) {
        self.recent_notifications.retain(|_, &mut timestamp| {
            current_time - timestamp < self.dedup_window * 2.0
        });
    }

    /// Set deduplication window
    pub fn set_dedup_window(&mut self, seconds: f64) {
        self.dedup_window = seconds.max(0.1);
    }

    /// Enable/disable notification grouping
    pub fn set_grouping_enabled(&mut self, enabled: bool) {
        self.grouping_enabled = enabled;
    }

    /// Set per-faction throttle rate
    pub fn set_faction_throttle_rate(&mut self, max_per_second: f32) {
        self.max_per_faction_per_second = max_per_second.max(0.5);
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
    /// Transaction manager for atomic operations
    pub transactions: TransactionManager,
    /// Undo manager for reversing changes
    pub undo: UndoManager,
    /// Enhanced metrics tracking
    pub metrics: PipelineMetrics,
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
            transactions: TransactionManager::new(),
            undo: UndoManager::new(),
            metrics: PipelineMetrics::new(),
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

        // Get pending changes for undo recording
        let pending_changes: Vec<_> = self.reputation.pending.iter().cloned().collect();

        let results = self.reputation.process_all(faction_manager, game_time);

        // Record in undo system and metrics
        for (i, result) in results.iter().enumerate() {
            // Record metrics
            self.metrics.record_change(result, game_time);

            // Record for undo (only successful changes)
            if result.success {
                if let Some(change) = pending_changes.get(i) {
                    self.undo.record(change.clone(), result.clone(), game_time);
                }
            }

            // Queue standing change notifications
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

    // ============================================
    // TRANSACTION METHODS
    // ============================================

    /// Begin a new transaction for atomic reputation changes
    pub fn begin_transaction(&mut self, description: &str, game_time: f64) -> u64 {
        self.transactions.begin(description, game_time)
    }

    /// Add a reputation change to a transaction
    pub fn add_to_transaction(
        &mut self,
        tx_id: u64,
        faction: Faction,
        delta: i32,
        reason: &str,
        source: ReputationSource,
        timestamp: f64,
    ) -> PipelineResult<()> {
        let change = ReputationChange {
            faction,
            delta,
            reason: reason.to_string(),
            source,
            timestamp,
            applied: false,
            change_id: 0, // Will be assigned on commit
        };
        self.transactions.add_to_transaction(tx_id, change)
    }

    /// Commit a transaction - all changes are queued for processing
    pub fn commit_transaction(&mut self, tx_id: u64) -> PipelineResult<usize> {
        let changes = self.transactions.commit(tx_id)?;
        let count = changes.len();

        // Queue all changes from the transaction
        for mut change in changes {
            let id = self.reputation.next_id;
            self.reputation.next_id += 1;
            change.change_id = id;
            self.reputation.pending.push_back(change);
        }

        self.metrics.global.transactions_committed += 1;
        self.persistence.mark_dirty();

        Ok(count)
    }

    /// Rollback a transaction - all changes are discarded
    pub fn rollback_transaction(&mut self, tx_id: u64) -> PipelineResult<()> {
        self.transactions.rollback(tx_id)?;
        self.metrics.global.transactions_rolled_back += 1;
        Ok(())
    }

    // ============================================
    // UNDO/REDO METHODS
    // ============================================

    /// Undo the last reputation change
    pub fn undo_last_change(&mut self, game_time: f64) -> Option<PipelineResult<u64>> {
        if let Some((faction, delta, reason)) = self.undo.pop_for_undo() {
            self.metrics.global.undo_count += 1;
            Some(self.process_reputation_change(faction, delta, &reason, ReputationSource::System, game_time))
        } else {
            None
        }
    }

    /// Redo the last undone change
    pub fn redo_change(&mut self, game_time: f64) -> Option<PipelineResult<u64>> {
        if let Some((faction, delta, reason)) = self.undo.pop_for_redo() {
            self.metrics.global.redo_count += 1;
            Some(self.process_reputation_change(faction, delta, &reason, ReputationSource::System, game_time))
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.undo.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.undo.can_redo()
    }

    // ============================================
    // METRICS METHODS
    // ============================================

    /// Get metrics summary string
    pub fn metrics_summary(&self) -> String {
        self.metrics.summary()
    }

    /// Get faction-specific metrics
    pub fn get_faction_metrics(&self, faction: Faction) -> Option<&FactionMetrics> {
        self.metrics.by_faction.get(&faction)
    }

    // ============================================
    // CONTROL METHODS
    // ============================================

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
            // Enhanced metrics
            active_transactions: self.transactions.active_count(),
            undo_depth: self.undo.undo_depth(),
            redo_depth: self.undo.redo_depth(),
            total_changes: self.metrics.global.total_changes,
            changes_per_minute: self.metrics.rate.changes_per_minute,
            standing_upgrades: self.metrics.global.standing_upgrades,
            standing_downgrades: self.metrics.global.standing_downgrades,
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
    // Enhanced metrics
    pub active_transactions: usize,
    pub undo_depth: usize,
    pub redo_depth: usize,
    pub total_changes: u64,
    pub changes_per_minute: f32,
    pub standing_upgrades: u32,
    pub standing_downgrades: u32,
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

    // ============================================
    // TRANSACTION TESTS
    // ============================================

    #[test]
    fn test_transaction_commit() {
        let mut coordinator = FactionPipelineCoordinator::new();
        let mut manager = FactionManager::new();

        // Begin transaction
        let tx_id = coordinator.begin_transaction("Test transaction", 0.0);
        assert_eq!(tx_id, 1);

        // Add changes
        coordinator.add_to_transaction(tx_id, Faction::Powhatan, 50, "Gift", ReputationSource::Gift, 0.0).unwrap();
        coordinator.add_to_transaction(tx_id, Faction::Cherokee, 30, "Trade", ReputationSource::Trade, 0.0).unwrap();

        // Commit
        let count = coordinator.commit_transaction(tx_id).unwrap();
        assert_eq!(count, 2);
        assert_eq!(coordinator.reputation.pending_count(), 2);

        // Flush and verify
        let results = coordinator.flush_reputation_changes(&mut manager, 0.0);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_transaction_rollback() {
        let mut coordinator = FactionPipelineCoordinator::new();

        let tx_id = coordinator.begin_transaction("Rollback test", 0.0);
        coordinator.add_to_transaction(tx_id, Faction::Powhatan, 100, "Test", ReputationSource::Quest, 0.0).unwrap();

        // Rollback
        coordinator.rollback_transaction(tx_id).unwrap();

        // Verify nothing was queued
        assert_eq!(coordinator.reputation.pending_count(), 0);
    }

    // ============================================
    // UNDO/REDO TESTS
    // ============================================

    #[test]
    fn test_undo_manager() {
        let mut undo = UndoManager::new();

        let change = ReputationChange {
            faction: Faction::Powhatan,
            delta: 100,
            reason: "Test".to_string(),
            source: ReputationSource::Quest,
            timestamp: 0.0,
            applied: true,
            change_id: 1,
        };

        let result = ReputationChangeResult {
            change_id: 1,
            faction: Faction::Powhatan,
            old_reputation: 0,
            new_reputation: 100,
            old_standing: Standing::Neutral,
            new_standing: Standing::Neutral,
            standing_changed: false,
            success: true,
            error: None,
        };

        undo.record(change, result, 0.0);
        assert!(undo.can_undo());
        assert!(!undo.can_redo());

        // Undo
        let (faction, delta, _reason) = undo.pop_for_undo().unwrap();
        assert_eq!(faction, Faction::Powhatan);
        assert_eq!(delta, -100); // Reverse delta
        assert!(undo.can_redo());

        // Redo
        let (faction, delta, _reason) = undo.pop_for_redo().unwrap();
        assert_eq!(faction, Faction::Powhatan);
        assert_eq!(delta, 100);
    }

    // ============================================
    // METRICS TESTS
    // ============================================

    #[test]
    fn test_pipeline_metrics() {
        let mut metrics = PipelineMetrics::new();

        let result = ReputationChangeResult {
            change_id: 1,
            faction: Faction::Powhatan,
            old_reputation: 0,
            new_reputation: 50,
            old_standing: Standing::Neutral,
            new_standing: Standing::Friendly,
            standing_changed: true,
            success: true,
            error: None,
        };

        metrics.record_change(&result, 0.0);

        assert_eq!(metrics.global.total_changes, 1);
        assert_eq!(metrics.global.standing_upgrades, 1);
        assert!(metrics.by_faction.contains_key(&Faction::Powhatan));

        let faction_metrics = metrics.by_faction.get(&Faction::Powhatan).unwrap();
        assert_eq!(faction_metrics.total_gained, 50);
        assert_eq!(faction_metrics.peak_reputation, 50);
    }

    // ============================================
    // NOTIFICATION GROUPING TESTS
    // ============================================

    #[test]
    fn test_notification_deduplication() {
        let mut dispatcher = NotificationDispatcher::new();

        // Queue same notification twice quickly
        let notif = FactionNotification {
            faction: Faction::Powhatan,
            notification_type: FactionNotificationType::ReputationGain,
            message: "Test".to_string(),
            importance: NotificationImportance::Medium,
            timestamp: 0.0,
            displayed: false,
        };

        dispatcher.queue(notif.clone());
        dispatcher.queue(notif); // Should be deduplicated

        assert_eq!(dispatcher.pending_count(), 1);
    }

    #[test]
    fn test_notification_throttle() {
        let mut dispatcher = NotificationDispatcher::new();
        dispatcher.set_faction_throttle_rate(2.0); // 2 per second max

        // Queue 5 notifications for same faction at same time
        for i in 0..5 {
            dispatcher.queue(FactionNotification {
                faction: Faction::Powhatan,
                notification_type: FactionNotificationType::ReputationGain,
                message: format!("Test {}", i),
                importance: NotificationImportance::Medium,
                timestamp: 0.0,
                displayed: false,
            });
        }

        // Should have throttled some
        assert!(dispatcher.pending_count() < 5);
    }

    #[test]
    fn test_health_report() {
        let coordinator = FactionPipelineCoordinator::new();
        let report = coordinator.health_report();

        assert_eq!(report.reputation_pending, 0);
        assert_eq!(report.notification_pending, 0);
        assert!(!report.paused);
        assert_eq!(report.active_transactions, 0);
        assert_eq!(report.undo_depth, 0);
    }
}
