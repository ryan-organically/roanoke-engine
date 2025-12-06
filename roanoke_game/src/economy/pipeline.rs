//! Economy Data Pipeline System
//!
//! Robust data flow management for all economy operations with:
//! - Event-driven architecture
//! - Transaction processing with rollback
//! - Data validation and integrity checks
//! - Analytics and telemetry
//! - Error recovery

use super::{
    item::{Item, ItemId, Rarity, Quality, ItemType},
    inventory::{Inventory, InventoryError, EquipSlot},
    currency::{Wallet, TransactionType, CurrencyTransaction},
    loot::{DropReward, PityTracker},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Maximum events in the pipeline queue
const MAX_PIPELINE_EVENTS: usize = 1000;
/// Maximum transaction history for rollback
const MAX_TRANSACTION_HISTORY: usize = 100;
/// Maximum analytics samples to retain
const MAX_ANALYTICS_SAMPLES: usize = 10000;
/// Validation thresholds
const MAX_SINGLE_WAMPUM_TRANSACTION: u64 = 10_000_000;
const MAX_SINGLE_TOBACCO_TRANSACTION: u64 = 100_000;
const MAX_ITEM_VALUE: u32 = 100_000_000;
const MAX_STACK_SIZE: u32 = 9999;

// ============================================================================
// EVENT TYPES
// ============================================================================

/// All economy events that flow through the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EconomyEvent {
    // Loot events
    LootDropped(LootDropEvent),
    LootCollected(LootCollectEvent),

    // Inventory events
    ItemAdded(ItemAddedEvent),
    ItemRemoved(ItemRemovedEvent),
    ItemMoved(ItemMovedEvent),
    ItemEquipped(ItemEquippedEvent),
    ItemUnequipped(ItemUnequippedEvent),
    ItemStacked(ItemStackedEvent),
    ItemSplit(ItemSplitEvent),
    ItemDestroyed(ItemDestroyedEvent),
    ItemRepaired(ItemRepairedEvent),
    ItemDamaged(ItemDamagedEvent),

    // Currency events
    WampumEarned(CurrencyEvent),
    WampumSpent(CurrencyEvent),
    TobaccoEarned(CurrencyEvent),
    TobaccoSpent(CurrencyEvent),
    CurrencyConverted(CurrencyConversionEvent),

    // Trading events
    TradeInitiated(TradeEvent),
    TradeCompleted(TradeCompletedEvent),
    TradeCancelled(TradeEvent),
    NpcPurchase(NpcTradeEvent),
    NpcSale(NpcTradeEvent),

    // Market events
    MarketListing(MarketEvent),
    MarketSale(MarketEvent),
    MarketCancelled(MarketEvent),
    PriceUpdated(PriceUpdateEvent),

    // System events
    PityTriggered(PityEvent),
    RareDropOccurred(RareDropEvent),
    ValidationFailed(ValidationFailedEvent),
    TransactionRolledBack(RollbackEvent),
}

/// Loot drop event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootDropEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub source_type: LootSource,
    pub source_id: String,
    pub position: [f32; 3],
    pub items: Vec<ItemId>,
    pub total_wampum: u64,
    pub total_tobacco: u64,
}

/// Loot collection event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootCollectEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub player_id: u64,
    pub item_id: ItemId,
    pub auto_collected: bool,
}

/// Source of loot drops
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LootSource {
    AnimalKill { species: String, was_legendary: bool },
    FossilDig { site_id: String },
    ChestOpen { tier: u32 },
    QuestReward { quest_id: String },
    CraftingResult,
    EventReward { event_id: String },
    WorldDrop,
}

/// Item added to inventory event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAddedEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub item_id: ItemId,
    pub template_id: String,
    pub rarity: Rarity,
    pub slot_index: usize,
    pub source: ItemSource,
}

/// Source of items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemSource {
    Loot,
    Trade,
    Purchase,
    Craft,
    Quest,
    Admin,
}

/// Item removed from inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRemovedEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub item_id: ItemId,
    pub reason: ItemRemovalReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemRemovalReason {
    Sold,
    Traded,
    Destroyed,
    Consumed,
    Dropped,
    QuestTurnIn,
}

/// Item moved between slots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMovedEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub item_id: ItemId,
    pub from_slot: usize,
    pub to_slot: usize,
}

/// Item equipped
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemEquippedEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub item_id: ItemId,
    pub slot: EquipSlot,
    pub previous_item: Option<ItemId>,
}

/// Item unequipped
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemUnequippedEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub item_id: ItemId,
    pub slot: EquipSlot,
    pub to_inventory_slot: usize,
}

/// Items stacked together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStackedEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub target_item_id: ItemId,
    pub source_item_id: ItemId,
    pub quantity_added: u32,
    pub new_stack_size: u32,
}

/// Item stack split
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSplitEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub source_item_id: ItemId,
    pub new_item_id: ItemId,
    pub quantity_split: u32,
}

/// Item destroyed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDestroyedEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub item_id: ItemId,
    pub reason: String,
    pub value_lost: u32,
}

/// Item repaired
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRepairedEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub item_id: ItemId,
    pub durability_restored: u32,
    pub repair_cost: u64,
    pub repair_count: u32,
}

/// Item took durability damage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDamagedEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub item_id: ItemId,
    pub damage_amount: u32,
    pub remaining_durability: u32,
}

/// Currency event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub amount: u64,
    pub reason: TransactionType,
    pub description: String,
    pub balance_after: u64,
}

/// Currency conversion event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyConversionEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub from_currency: CurrencyType,
    pub from_amount: u64,
    pub to_currency: CurrencyType,
    pub to_amount: u64,
    pub tax_burned: u64,
    pub exchange_rate: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CurrencyType {
    Wampum,
    Tobacco,
}

/// Trade initiation/cancellation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub trade_id: u64,
    pub initiator_id: u64,
    pub target_id: u64,
}

/// Trade completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCompletedEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub trade_id: u64,
    pub initiator_items: Vec<ItemId>,
    pub initiator_wampum: u64,
    pub target_items: Vec<ItemId>,
    pub target_wampum: u64,
}

/// NPC trade event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcTradeEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub npc_id: u32,
    pub item_id: ItemId,
    pub quantity: u32,
    pub price: u64,
    pub reputation_modifier: f32,
}

/// Market event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub listing_id: u64,
    pub seller_id: u64,
    pub item_id: ItemId,
    pub price: u64,
    pub currency: CurrencyType,
}

/// Price update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdateEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub item_template_id: String,
    pub old_price: u64,
    pub new_price: u64,
    pub reason: PriceChangeReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceChangeReason {
    SupplyChange,
    DemandChange,
    EventModifier,
    AdminOverride,
    MarketCorrection,
}

/// Pity trigger event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PityEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub rarity_guaranteed: Rarity,
    pub drops_since_last: u32,
    pub karma_accumulated: f32,
}

/// Rare drop event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RareDropEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub item_id: ItemId,
    pub rarity: Rarity,
    pub source: LootSource,
    pub was_pity: bool,
}

/// Validation failure event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFailedEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub operation: String,
    pub reason: ValidationError,
    pub context: String,
}

/// Rollback event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub transaction_id: u64,
    pub reason: String,
    pub events_reverted: u32,
}

// ============================================================================
// VALIDATION SYSTEM
// ============================================================================

/// Validation errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    // Item validation
    InvalidItemId,
    InvalidRarity,
    InvalidQuality(u8),
    InvalidStackSize(u32),
    ItemValueTooHigh(u32),
    DuplicateItemId(ItemId),

    // Currency validation
    InsufficientWampum { required: u64, available: u64 },
    InsufficientTobacco { required: u64, available: u64 },
    TransactionTooLarge { amount: u64, max: u64 },
    NegativeBalance,

    // Inventory validation
    InventoryFull,
    SlotOccupied(usize),
    InvalidSlot(usize),
    ItemNotFound(ItemId),
    CannotStack { item_a: ItemId, item_b: ItemId },

    // Trade validation
    TradeNotFound(u64),
    TradeAlreadyComplete,
    TradeMismatch,
    InvalidTradePartner,

    // Data integrity
    ChecksumMismatch,
    VersionMismatch { expected: u32, found: u32 },
    CorruptedData(String),

    // Rate limiting
    TooManyOperations,
    CooldownActive { remaining_ms: u64 },
}

/// Validation result
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Data validator for economy operations
pub struct DataValidator {
    /// Known item IDs for duplicate detection
    known_items: HashMap<ItemId, u64>, // ItemId -> timestamp first seen

    /// Operation rate limiting
    operation_counts: HashMap<String, (u64, u32)>, // operation -> (window_start, count)

    /// Rate limit window in milliseconds
    rate_limit_window_ms: u64,

    /// Max operations per window
    max_operations_per_window: u32,
}

impl Default for DataValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl DataValidator {
    pub fn new() -> Self {
        Self {
            known_items: HashMap::new(),
            operation_counts: HashMap::new(),
            rate_limit_window_ms: 1000,
            max_operations_per_window: 100,
        }
    }

    /// Validate an item
    pub fn validate_item(&mut self, item: &Item) -> ValidationResult<()> {
        // Check for duplicate ID
        let now = Self::timestamp();
        if let Some(&first_seen) = self.known_items.get(&item.id) {
            // Allow if same item (updating), reject if new item with same ID
            if now - first_seen > 1000 {
                return Err(ValidationError::DuplicateItemId(item.id));
            }
        }

        // Validate quality
        if item.quality.0 > 100 {
            return Err(ValidationError::InvalidQuality(item.quality.0));
        }

        // Validate stack size
        if item.stack_size > MAX_STACK_SIZE || item.stack_size > item.max_stack {
            return Err(ValidationError::InvalidStackSize(item.stack_size));
        }

        // Validate value
        let value = item.calculate_value();
        if value > MAX_ITEM_VALUE {
            return Err(ValidationError::ItemValueTooHigh(value));
        }

        // Register item
        self.known_items.insert(item.id, now);

        Ok(())
    }

    /// Validate currency transaction
    pub fn validate_currency_transaction(
        &self,
        wallet: &Wallet,
        currency: CurrencyType,
        amount: u64,
        is_spend: bool,
    ) -> ValidationResult<()> {
        let max_transaction = match currency {
            CurrencyType::Wampum => MAX_SINGLE_WAMPUM_TRANSACTION,
            CurrencyType::Tobacco => MAX_SINGLE_TOBACCO_TRANSACTION,
        };

        if amount > max_transaction {
            return Err(ValidationError::TransactionTooLarge {
                amount,
                max: max_transaction,
            });
        }

        if is_spend {
            let balance = match currency {
                CurrencyType::Wampum => wallet.wampum,
                CurrencyType::Tobacco => wallet.tobacco,
            };

            if amount > balance {
                match currency {
                    CurrencyType::Wampum => return Err(ValidationError::InsufficientWampum {
                        required: amount,
                        available: balance,
                    }),
                    CurrencyType::Tobacco => return Err(ValidationError::InsufficientTobacco {
                        required: amount,
                        available: balance,
                    }),
                }
            }
        }

        Ok(())
    }

    /// Validate inventory operation
    pub fn validate_inventory_add(
        &self,
        inventory: &Inventory,
        item: &Item,
    ) -> ValidationResult<usize> {
        // Check for free slot
        for (idx, slot) in inventory.slots.iter().enumerate() {
            if slot.is_none() {
                return Ok(idx);
            }

            // Check for stackable
            if let Some(existing) = slot {
                if existing.template_id == item.template_id
                    && existing.rarity == item.rarity
                    && existing.stack_size < existing.max_stack
                {
                    return Ok(idx);
                }
            }
        }

        Err(ValidationError::InventoryFull)
    }

    /// Check rate limiting
    pub fn check_rate_limit(&mut self, operation: &str) -> ValidationResult<()> {
        let now = Self::timestamp();

        let (window_start, count) = self.operation_counts
            .entry(operation.to_string())
            .or_insert((now, 0));

        // Reset window if expired
        if now - *window_start > self.rate_limit_window_ms {
            *window_start = now;
            *count = 0;
        }

        if *count >= self.max_operations_per_window {
            return Err(ValidationError::TooManyOperations);
        }

        *count += 1;
        Ok(())
    }

    /// Get current timestamp in milliseconds
    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Clear old item registrations (call periodically)
    pub fn cleanup(&mut self, max_age_ms: u64) {
        let now = Self::timestamp();
        self.known_items.retain(|_, &mut ts| now - ts < max_age_ms);
    }
}

// ============================================================================
// TRANSACTION SYSTEM
// ============================================================================

/// Transaction state for rollback support
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: u64,
    pub started_at: u64,
    pub events: Vec<EconomyEvent>,
    pub state_snapshot: Option<TransactionSnapshot>,
    pub committed: bool,
}

/// Snapshot of state for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSnapshot {
    pub wampum: u64,
    pub tobacco: u64,
    pub inventory_checksums: Vec<u64>,
    pub pity_state: PityTracker,
}

impl Transaction {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            events: Vec::new(),
            state_snapshot: None,
            committed: false,
        }
    }

    pub fn add_event(&mut self, event: EconomyEvent) {
        self.events.push(event);
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

/// Transaction manager for atomic operations
pub struct TransactionManager {
    /// Current transaction ID counter
    next_transaction_id: u64,

    /// Active transactions
    active_transactions: HashMap<u64, Transaction>,

    /// Completed transaction history for audit
    transaction_history: VecDeque<Transaction>,

    /// Failed transactions for debugging
    failed_transactions: VecDeque<(Transaction, String)>,
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            next_transaction_id: 1,
            active_transactions: HashMap::new(),
            transaction_history: VecDeque::new(),
            failed_transactions: VecDeque::new(),
        }
    }

    /// Begin a new transaction
    pub fn begin(&mut self) -> u64 {
        let id = self.next_transaction_id;
        self.next_transaction_id += 1;

        let transaction = Transaction::new(id);
        self.active_transactions.insert(id, transaction);

        id
    }

    /// Begin transaction with state snapshot for rollback
    pub fn begin_with_snapshot(
        &mut self,
        wallet: &Wallet,
        inventory: &Inventory,
        pity: &PityTracker,
    ) -> u64 {
        let id = self.begin();

        if let Some(tx) = self.active_transactions.get_mut(&id) {
            tx.state_snapshot = Some(TransactionSnapshot {
                wampum: wallet.wampum,
                tobacco: wallet.tobacco,
                inventory_checksums: Self::compute_inventory_checksums(inventory),
                pity_state: pity.clone(),
            });
        }

        id
    }

    /// Add event to transaction
    pub fn add_event(&mut self, transaction_id: u64, event: EconomyEvent) -> bool {
        if let Some(tx) = self.active_transactions.get_mut(&transaction_id) {
            tx.add_event(event);
            true
        } else {
            false
        }
    }

    /// Commit transaction
    pub fn commit(&mut self, transaction_id: u64) -> Option<Transaction> {
        if let Some(mut tx) = self.active_transactions.remove(&transaction_id) {
            tx.committed = true;

            // Add to history
            self.transaction_history.push_back(tx.clone());
            if self.transaction_history.len() > MAX_TRANSACTION_HISTORY {
                self.transaction_history.pop_front();
            }

            Some(tx)
        } else {
            None
        }
    }

    /// Rollback transaction
    pub fn rollback(&mut self, transaction_id: u64, reason: &str) -> Option<TransactionSnapshot> {
        if let Some(tx) = self.active_transactions.remove(&transaction_id) {
            let snapshot = tx.state_snapshot.clone();

            // Record failed transaction
            self.failed_transactions.push_back((tx, reason.to_string()));
            if self.failed_transactions.len() > 50 {
                self.failed_transactions.pop_front();
            }

            snapshot
        } else {
            None
        }
    }

    /// Get active transaction
    pub fn get_transaction(&self, transaction_id: u64) -> Option<&Transaction> {
        self.active_transactions.get(&transaction_id)
    }

    /// Compute checksums for inventory slots
    fn compute_inventory_checksums(inventory: &Inventory) -> Vec<u64> {
        inventory.slots.iter()
            .map(|slot| {
                slot.as_ref()
                    .map(|item| item.id.0)
                    .unwrap_or(0)
            })
            .collect()
    }

    /// Get transaction statistics
    pub fn stats(&self) -> TransactionStats {
        TransactionStats {
            active_count: self.active_transactions.len(),
            completed_count: self.transaction_history.len(),
            failed_count: self.failed_transactions.len(),
            next_id: self.next_transaction_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionStats {
    pub active_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub next_id: u64,
}

// ============================================================================
// ANALYTICS PIPELINE
// ============================================================================

/// Analytics sample point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSample {
    pub timestamp: u64,
    pub metric_type: MetricType,
    pub value: f64,
    pub tags: HashMap<String, String>,
}

/// Types of metrics tracked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    // Currency metrics
    WampumEarned,
    WampumSpent,
    TobaccoEarned,
    TobaccoSpent,
    WampumBalance,
    TobaccoBalance,

    // Loot metrics
    ItemsLooted,
    RareDrops,
    LegendaryDrops,
    PityTriggers,

    // Trading metrics
    NpcPurchases,
    NpcSales,
    PlayerTrades,
    MarketSales,

    // Inventory metrics
    InventoryValue,
    InventorySlotUsage,
    ItemsDestroyed,

    // Session metrics
    SessionDuration,
    ActionsPerMinute,

    // Economy health
    InflationRate,
    VelocityOfMoney,
    GiniCoefficient,
}

/// Aggregated analytics data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsAggregates {
    // Totals
    pub total_wampum_generated: u64,
    pub total_wampum_burned: u64,
    pub total_tobacco_generated: u64,
    pub total_tobacco_burned: u64,
    pub total_items_created: u64,
    pub total_items_destroyed: u64,

    // Rates (per hour)
    pub wampum_generation_rate: f64,
    pub wampum_burn_rate: f64,
    pub item_creation_rate: f64,
    pub item_destruction_rate: f64,

    // Distribution
    pub rarity_distribution: [u64; 8], // Crude to Primordial
    pub average_item_quality: f64,

    // Pity system
    pub total_pity_triggers: u64,
    pub average_drops_between_rare: f64,

    // Trading
    pub total_trades: u64,
    pub total_trade_volume_wampum: u64,
    pub average_trade_value: f64,
}

/// Analytics collector and processor
pub struct AnalyticsPipeline {
    /// Raw samples
    samples: VecDeque<AnalyticsSample>,

    /// Aggregated data
    aggregates: AnalyticsAggregates,

    /// Hourly snapshots
    hourly_snapshots: VecDeque<AnalyticsAggregates>,

    /// Last aggregation time
    last_aggregation: u64,

    /// Aggregation interval (ms)
    aggregation_interval_ms: u64,
}

impl Default for AnalyticsPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalyticsPipeline {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            aggregates: AnalyticsAggregates::default(),
            hourly_snapshots: VecDeque::new(),
            last_aggregation: 0,
            aggregation_interval_ms: 60_000, // 1 minute
        }
    }

    /// Record a metric sample
    pub fn record(&mut self, metric_type: MetricType, value: f64, tags: HashMap<String, String>) {
        let sample = AnalyticsSample {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            metric_type,
            value,
            tags,
        };

        self.samples.push_back(sample);

        if self.samples.len() > MAX_ANALYTICS_SAMPLES {
            self.samples.pop_front();
        }

        // Update running aggregates
        self.update_aggregates(metric_type, value);
    }

    /// Record simple metric without tags
    pub fn record_simple(&mut self, metric_type: MetricType, value: f64) {
        self.record(metric_type, value, HashMap::new());
    }

    /// Update running aggregates
    fn update_aggregates(&mut self, metric_type: MetricType, value: f64) {
        match metric_type {
            MetricType::WampumEarned => {
                self.aggregates.total_wampum_generated += value as u64;
            }
            MetricType::WampumSpent => {
                self.aggregates.total_wampum_burned += value as u64;
            }
            MetricType::TobaccoEarned => {
                self.aggregates.total_tobacco_generated += value as u64;
            }
            MetricType::TobaccoSpent => {
                self.aggregates.total_tobacco_burned += value as u64;
            }
            MetricType::ItemsLooted => {
                self.aggregates.total_items_created += value as u64;
            }
            MetricType::ItemsDestroyed => {
                self.aggregates.total_items_destroyed += value as u64;
            }
            MetricType::PityTriggers => {
                self.aggregates.total_pity_triggers += 1;
            }
            MetricType::PlayerTrades => {
                self.aggregates.total_trades += 1;
            }
            _ => {}
        }
    }

    /// Record loot drop with full details
    pub fn record_loot_drop(&mut self, rarity: Rarity, quality: u8, wampum: u64, was_pity: bool) {
        self.record_simple(MetricType::ItemsLooted, 1.0);
        self.record_simple(MetricType::WampumEarned, wampum as f64);

        // Update rarity distribution
        let rarity_idx = rarity as usize;
        if rarity_idx < 8 {
            self.aggregates.rarity_distribution[rarity_idx] += 1;
        }

        // Track rare drops
        if rarity >= Rarity::Rare {
            self.record_simple(MetricType::RareDrops, 1.0);
        }
        if rarity >= Rarity::Legendary {
            self.record_simple(MetricType::LegendaryDrops, 1.0);
        }

        // Track pity
        if was_pity {
            self.record_simple(MetricType::PityTriggers, 1.0);
        }

        // Update average quality
        let total_items = self.aggregates.total_items_created as f64;
        let old_avg = self.aggregates.average_item_quality;
        self.aggregates.average_item_quality =
            (old_avg * (total_items - 1.0) + quality as f64) / total_items;
    }

    /// Get current aggregates
    pub fn get_aggregates(&self) -> &AnalyticsAggregates {
        &self.aggregates
    }

    /// Get samples in time range
    pub fn get_samples(&self, start: u64, end: u64) -> Vec<&AnalyticsSample> {
        self.samples.iter()
            .filter(|s| s.timestamp >= start && s.timestamp <= end)
            .collect()
    }

    /// Get samples by metric type
    pub fn get_samples_by_type(&self, metric_type: MetricType) -> Vec<&AnalyticsSample> {
        self.samples.iter()
            .filter(|s| s.metric_type == metric_type)
            .collect()
    }

    /// Calculate rate over time window
    pub fn calculate_rate(&self, metric_type: MetricType, window_ms: u64) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let start = now.saturating_sub(window_ms);

        let sum: f64 = self.samples.iter()
            .filter(|s| s.metric_type == metric_type && s.timestamp >= start)
            .map(|s| s.value)
            .sum();

        let hours = window_ms as f64 / 3_600_000.0;
        if hours > 0.0 { sum / hours } else { 0.0 }
    }

    /// Get summary statistics
    pub fn summary(&self) -> AnalyticsSummary {
        let hour_ms = 3_600_000u64;

        AnalyticsSummary {
            total_samples: self.samples.len(),
            wampum_per_hour: self.calculate_rate(MetricType::WampumEarned, hour_ms),
            items_per_hour: self.calculate_rate(MetricType::ItemsLooted, hour_ms),
            rare_drops_per_hour: self.calculate_rate(MetricType::RareDrops, hour_ms),
            aggregates: self.aggregates.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalyticsSummary {
    pub total_samples: usize,
    pub wampum_per_hour: f64,
    pub items_per_hour: f64,
    pub rare_drops_per_hour: f64,
    pub aggregates: AnalyticsAggregates,
}

// ============================================================================
// EVENT PIPELINE
// ============================================================================

/// Event handler trait
pub trait EconomyEventHandler: Send + Sync {
    fn handle(&mut self, event: &EconomyEvent);
    fn name(&self) -> &str;
}

/// Main event pipeline
pub struct EventPipeline {
    /// Event queue
    event_queue: VecDeque<EconomyEvent>,

    /// Event ID counter
    next_event_id: u64,

    /// Validator
    validator: DataValidator,

    /// Transaction manager
    transactions: TransactionManager,

    /// Analytics pipeline
    analytics: AnalyticsPipeline,

    /// Event history for replay/debugging
    event_history: VecDeque<EconomyEvent>,

    /// Error log
    error_log: VecDeque<(u64, ValidationError, String)>,
}

impl Default for EventPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl EventPipeline {
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            next_event_id: 1,
            validator: DataValidator::new(),
            transactions: TransactionManager::new(),
            analytics: AnalyticsPipeline::new(),
            event_history: VecDeque::new(),
            error_log: VecDeque::new(),
        }
    }

    /// Generate next event ID
    pub fn next_event_id(&mut self) -> u64 {
        let id = self.next_event_id;
        self.next_event_id += 1;
        id
    }

    /// Get current timestamp
    pub fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Queue an event for processing
    pub fn queue_event(&mut self, event: EconomyEvent) {
        if self.event_queue.len() < MAX_PIPELINE_EVENTS {
            self.event_queue.push_back(event);
        }
    }

    /// Process all queued events
    pub fn process_events(&mut self) -> Vec<EconomyEvent> {
        let events: Vec<_> = self.event_queue.drain(..).collect();

        for event in &events {
            // Record in analytics
            self.record_event_analytics(event);

            // Add to history
            self.event_history.push_back(event.clone());
            if self.event_history.len() > MAX_PIPELINE_EVENTS {
                self.event_history.pop_front();
            }
        }

        events
    }

    /// Record event in analytics
    fn record_event_analytics(&mut self, event: &EconomyEvent) {
        match event {
            EconomyEvent::WampumEarned(e) => {
                self.analytics.record_simple(MetricType::WampumEarned, e.amount as f64);
            }
            EconomyEvent::WampumSpent(e) => {
                self.analytics.record_simple(MetricType::WampumSpent, e.amount as f64);
            }
            EconomyEvent::TobaccoEarned(e) => {
                self.analytics.record_simple(MetricType::TobaccoEarned, e.amount as f64);
            }
            EconomyEvent::TobaccoSpent(e) => {
                self.analytics.record_simple(MetricType::TobaccoSpent, e.amount as f64);
            }
            EconomyEvent::ItemAdded(e) => {
                self.analytics.record_simple(MetricType::ItemsLooted, 1.0);
            }
            EconomyEvent::ItemDestroyed(_) => {
                self.analytics.record_simple(MetricType::ItemsDestroyed, 1.0);
            }
            EconomyEvent::NpcPurchase(_) => {
                self.analytics.record_simple(MetricType::NpcPurchases, 1.0);
            }
            EconomyEvent::NpcSale(_) => {
                self.analytics.record_simple(MetricType::NpcSales, 1.0);
            }
            EconomyEvent::TradeCompleted(_) => {
                self.analytics.record_simple(MetricType::PlayerTrades, 1.0);
            }
            EconomyEvent::PityTriggered(_) => {
                self.analytics.record_simple(MetricType::PityTriggers, 1.0);
            }
            EconomyEvent::RareDropOccurred(e) => {
                if e.rarity >= Rarity::Legendary {
                    self.analytics.record_simple(MetricType::LegendaryDrops, 1.0);
                } else {
                    self.analytics.record_simple(MetricType::RareDrops, 1.0);
                }
            }
            _ => {}
        }
    }

    /// Begin a transaction
    pub fn begin_transaction(&mut self) -> u64 {
        self.transactions.begin()
    }

    /// Begin transaction with snapshot
    pub fn begin_transaction_with_snapshot(
        &mut self,
        wallet: &Wallet,
        inventory: &Inventory,
        pity: &PityTracker,
    ) -> u64 {
        self.transactions.begin_with_snapshot(wallet, inventory, pity)
    }

    /// Add event to transaction
    pub fn add_to_transaction(&mut self, tx_id: u64, event: EconomyEvent) {
        self.transactions.add_event(tx_id, event);
    }

    /// Commit transaction
    pub fn commit_transaction(&mut self, tx_id: u64) -> bool {
        self.transactions.commit(tx_id).is_some()
    }

    /// Rollback transaction
    pub fn rollback_transaction(&mut self, tx_id: u64, reason: &str) -> Option<TransactionSnapshot> {
        let snapshot = self.transactions.rollback(tx_id, reason);

        // Record rollback event
        let event = EconomyEvent::TransactionRolledBack(RollbackEvent {
            event_id: self.next_event_id(),
            timestamp: Self::timestamp(),
            transaction_id: tx_id,
            reason: reason.to_string(),
            events_reverted: 0,
        });
        self.queue_event(event);

        snapshot
    }

    /// Validate item
    pub fn validate_item(&mut self, item: &Item) -> ValidationResult<()> {
        self.validator.validate_item(item)
    }

    /// Validate currency transaction
    pub fn validate_currency(
        &self,
        wallet: &Wallet,
        currency: CurrencyType,
        amount: u64,
        is_spend: bool,
    ) -> ValidationResult<()> {
        self.validator.validate_currency_transaction(wallet, currency, amount, is_spend)
    }

    /// Validate inventory add
    pub fn validate_inventory_add(&self, inventory: &Inventory, item: &Item) -> ValidationResult<usize> {
        self.validator.validate_inventory_add(inventory, item)
    }

    /// Log validation error
    pub fn log_error(&mut self, error: ValidationError, context: &str) {
        let timestamp = Self::timestamp();
        self.error_log.push_back((timestamp, error.clone(), context.to_string()));

        if self.error_log.len() > 1000 {
            self.error_log.pop_front();
        }

        // Queue validation failed event
        let event = EconomyEvent::ValidationFailed(ValidationFailedEvent {
            event_id: self.next_event_id(),
            timestamp,
            operation: "validation".to_string(),
            reason: error,
            context: context.to_string(),
        });
        self.queue_event(event);
    }

    /// Get analytics
    pub fn analytics(&self) -> &AnalyticsPipeline {
        &self.analytics
    }

    /// Get mutable analytics
    pub fn analytics_mut(&mut self) -> &mut AnalyticsPipeline {
        &mut self.analytics
    }

    /// Get transaction stats
    pub fn transaction_stats(&self) -> TransactionStats {
        self.transactions.stats()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.error_log.len()
    }

    /// Get recent errors
    pub fn recent_errors(&self, count: usize) -> Vec<&(u64, ValidationError, String)> {
        self.error_log.iter().rev().take(count).collect()
    }

    /// Cleanup old data
    pub fn cleanup(&mut self) {
        self.validator.cleanup(3_600_000); // 1 hour
    }
}

// ============================================================================
// PERSISTENCE PIPELINE
// ============================================================================

/// Serializable economy state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyPersistenceData {
    pub version: u32,
    pub timestamp: u64,
    pub checksum: u64,
    pub wallet: Wallet,
    pub inventory: Inventory,
    pub pity_tracker: PityTracker,
    pub analytics_aggregates: AnalyticsAggregates,
}

impl EconomyPersistenceData {
    pub const CURRENT_VERSION: u32 = 1;

    /// Create from current state
    pub fn from_state(
        wallet: &Wallet,
        inventory: &Inventory,
        pity: &PityTracker,
        aggregates: &AnalyticsAggregates,
    ) -> Self {
        let mut data = Self {
            version: Self::CURRENT_VERSION,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            checksum: 0,
            wallet: wallet.clone(),
            inventory: inventory.clone(),
            pity_tracker: pity.clone(),
            analytics_aggregates: aggregates.clone(),
        };

        data.checksum = data.compute_checksum();
        data
    }

    /// Compute checksum for data integrity
    fn compute_checksum(&self) -> u64 {
        let mut hash = 0u64;

        // Mix in key values
        hash ^= self.wallet.wampum.wrapping_mul(0x517cc1b727220a95);
        hash ^= self.wallet.tobacco.wrapping_mul(0x9e3779b97f4a7c15);
        hash ^= (self.inventory.slots.len() as u64).wrapping_mul(0xbf58476d1ce4e5b9);
        hash ^= self.pity_tracker.lifetime_drops.wrapping_mul(0x94d049bb133111eb);
        hash ^= self.timestamp.wrapping_mul(0x9e3779b97f4a7c15);

        // Mix in item IDs
        for slot in &self.inventory.slots {
            if let Some(item) = slot {
                hash ^= item.id.0.rotate_left(13);
            }
        }

        hash
    }

    /// Validate data integrity
    pub fn validate(&self) -> ValidationResult<()> {
        // Check version
        if self.version > Self::CURRENT_VERSION {
            return Err(ValidationError::VersionMismatch {
                expected: Self::CURRENT_VERSION,
                found: self.version,
            });
        }

        // Verify checksum
        let computed = self.compute_checksum();
        if computed != self.checksum {
            return Err(ValidationError::ChecksumMismatch);
        }

        Ok(())
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, String> {
        let data: Self = serde_json::from_str(json)
            .map_err(|e| format!("Deserialization error: {}", e))?;

        data.validate()
            .map_err(|e| format!("Validation error: {:?}", e))?;

        Ok(data)
    }

    /// Serialize to binary (more compact)
    pub fn to_binary(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self)
            .map_err(|e| format!("Binary serialization error: {}", e))
    }

    /// Deserialize from binary
    pub fn from_binary(data: &[u8]) -> Result<Self, String> {
        let state: Self = bincode::deserialize(data)
            .map_err(|e| format!("Binary deserialization error: {}", e))?;

        state.validate()
            .map_err(|e| format!("Validation error: {:?}", e))?;

        Ok(state)
    }
}

/// Persistence manager
pub struct PersistenceManager {
    /// Auto-save interval in seconds
    auto_save_interval: u64,

    /// Last save timestamp
    last_save: u64,

    /// Pending changes flag
    has_pending_changes: bool,

    /// Backup count to maintain
    backup_count: usize,
}

impl Default for PersistenceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PersistenceManager {
    pub fn new() -> Self {
        Self {
            auto_save_interval: 300, // 5 minutes
            last_save: 0,
            has_pending_changes: false,
            backup_count: 3,
        }
    }

    /// Mark that changes need saving
    pub fn mark_dirty(&mut self) {
        self.has_pending_changes = true;
    }

    /// Check if auto-save is due
    pub fn should_auto_save(&self) -> bool {
        if !self.has_pending_changes {
            return false;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.last_save >= self.auto_save_interval
    }

    /// Record that save completed
    pub fn on_save_complete(&mut self) {
        self.last_save = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.has_pending_changes = false;
    }

    /// Get time until next auto-save
    pub fn time_until_auto_save(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let elapsed = now - self.last_save;
        if elapsed >= self.auto_save_interval {
            0
        } else {
            self.auto_save_interval - elapsed
        }
    }

    /// Set auto-save interval
    pub fn set_auto_save_interval(&mut self, seconds: u64) {
        self.auto_save_interval = seconds;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let mut validator = DataValidator::new();

        let item = Item::new("test", "Test Item", ItemType::Material, 100);
        assert!(validator.validate_item(&item).is_ok());
    }

    #[test]
    fn test_transaction_lifecycle() {
        let mut tm = TransactionManager::new();

        let tx_id = tm.begin();
        assert!(tm.get_transaction(tx_id).is_some());

        tm.add_event(tx_id, EconomyEvent::WampumEarned(CurrencyEvent {
            event_id: 1,
            timestamp: 0,
            amount: 100,
            reason: TransactionType::LootDrop,
            description: "Test".to_string(),
            balance_after: 100,
        }));

        let tx = tm.commit(tx_id);
        assert!(tx.is_some());
        assert!(tx.unwrap().committed);
    }

    #[test]
    fn test_analytics() {
        let mut analytics = AnalyticsPipeline::new();

        analytics.record_simple(MetricType::WampumEarned, 100.0);
        analytics.record_simple(MetricType::WampumEarned, 200.0);

        assert_eq!(analytics.get_aggregates().total_wampum_generated, 300);
    }

    #[test]
    fn test_persistence_checksum() {
        let wallet = Wallet::new();
        let inventory = Inventory::new();
        let pity = PityTracker::default();
        let aggregates = AnalyticsAggregates::default();

        let data = EconomyPersistenceData::from_state(&wallet, &inventory, &pity, &aggregates);
        assert!(data.validate().is_ok());
    }
}
