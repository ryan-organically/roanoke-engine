//! Roanoke Economy System
//!
//! Core economic infrastructure for items, currency, and trading.
//! Implements dual-currency system (Wampum/Tobacco) with full provenance tracking.
//!
//! # Module Structure
//! - `item` - Core item definitions, rarity, quality, provenance
//! - `inventory` - Player inventory management with equipment slots
//! - `currency` - Dual currency (Wampum/Tobacco) wallet system
//! - `loot` - Procedural loot generation with pity system
//! - `integration` - Connects economy to combat, trading, progression
//! - `pipeline` - Data validation, transactions, analytics, persistence
//! - `drops` - Dropped item entities in the world
//! - `storage` - World-placed storage containers (chests, crates, barrels)

pub mod item;
pub mod inventory;
pub mod currency;
pub mod loot;
pub mod integration;
pub mod pipeline;
pub mod drops;
pub mod storage;

// Core exports
pub use item::*;
pub use inventory::*;
pub use currency::*;
pub use loot::*;

// Integration exports
pub use integration::{
    PlayerEconomy,
    EconomyManager,
    LootNotification,
    CombatLootResult,
    CombatEconomyExt,
    SessionEconomyStats,
};
pub use integration::trading_bridge;

// Drops exports
pub use drops::{DroppedItem, DroppedItemManager, DropId};

// Storage exports
pub use storage::{StorageContainer, StorageManager, ContainerId, ContainerType, StorageError};

// Pipeline exports
pub use pipeline::{
    // Event types
    EconomyEvent,
    LootDropEvent,
    LootCollectEvent,
    LootSource,
    ItemAddedEvent,
    ItemRemovedEvent,
    ItemRemovalReason,
    ItemSource,
    CurrencyEvent,
    CurrencyType,
    // Validation
    ValidationError,
    ValidationResult,
    DataValidator,
    // Transactions
    Transaction,
    TransactionSnapshot,
    TransactionManager,
    TransactionStats,
    // Analytics
    AnalyticsPipeline,
    AnalyticsSample,
    AnalyticsAggregates,
    AnalyticsSummary,
    MetricType,
    // Pipeline
    EventPipeline,
    // Persistence
    EconomyPersistenceData,
    PersistenceManager,
};
