//! Player Progression System
//!
//! Tracks player skills, reputation, quests, and world state.
//! This is the core system for campaign progression and deep interactions.

pub mod events;
pub mod faction;
pub mod faction_integration;
pub mod faction_manager;
pub mod faction_pipeline;
pub mod faction_skills;
pub mod player_state;
pub mod quests;
pub mod reputation;
pub mod skills;

// Colonial expansion systems
pub mod colonial_dynamics;
pub mod settlement;
pub mod resource_gathering;

pub use events::EventManager;
pub use player_state::PlayerProgression;
pub use quests::QuestManager;

// Faction system exports
pub use faction::{
    get_default_relationships, get_faction_abilities, get_faction_traits, get_faction_weapons,
    AbilityEffect, Faction, FactionAbility, FactionCulture, FactionRelationshipMatrix,
    FactionTrait, FactionWeapon, Standing, TraitEffect, WeaponCategory, WeaponProperty,
};
pub use faction_manager::{
    FactionAction, FactionError, FactionManager, FactionReputation, NpcDisposition, NpcRole,
    SkillUnlockError,
};
pub use faction_skills::{
    get_faction_skill_tree, FactionSkill, FactionSkillId, PlayerFactionSkills, SkillEffect,
    UnlockCondition,
};
pub use faction_integration::{
    FactionEvent, FactionEventProcessor, FactionNotification, FactionSaveData, FactionUIData,
    NpcFactionData, VillageFaction, VillageStatus,
};
pub use faction_pipeline::{
    FactionPipelineCoordinator, PersistencePipeline, ReputationPipeline, ReputationSource,
    NotificationDispatcher, EventValidator, FactionSyncPipeline, PipelineError, PipelineResult,
    PipelineHealthReport, SyncOperation, ReputationChangeResult,
    // Transaction system
    TransactionManager, ReputationTransaction, TransactionState,
    // Undo system
    UndoManager, UndoEntry,
    // Enhanced metrics
    PipelineMetrics, FactionMetrics, GlobalMetrics, RateMetrics,
};

// Colonial dynamics exports
pub use colonial_dynamics::{
    ColonialPower, NativeNation, NativePolicy, GovernmentType,
    ColonialDynamicsManager, TerritorialClaim, TerritoryResources,
    FactionConflict, ConflictType, ConflictIntensity, ConflictCause,
    DiplomaticAction, PeaceTerms, PowerBalance, HistoricalEvent, HistoricalEventType,
    Alliance, AllianceType, TradeAgreement,
};
pub use settlement::{
    SettlementStyle, BuildingType, BuildingEffect, ConstructionCost, ResourceType,
    Settlement, Building, SettlementResources, SettlementManager, ConstructionError,
    ConstructionProject,
};
pub use resource_gathering::{
    GatherableResource, ResourceCategory, BiomeType, GatheringTool, ToolQuality,
    ResourceNode, GatherResult, ResourceGatheringManager, GatheringProgress, GatheringStats,
    GatheringCamp, GatheringCampType,
};
