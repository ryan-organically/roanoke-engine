//! Player Progression System
//!
//! Tracks player skills, reputation, quests, and world state.
//! This is the core system for campaign progression and deep interactions.

pub mod events;
pub mod faction;
pub mod faction_manager;
pub mod faction_skills;
pub mod player_state;
pub mod quests;
pub mod reputation;
pub mod skills;

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
