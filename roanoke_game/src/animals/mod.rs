//! Animal System Module
//!
//! Handles dangerous wildlife spawning, AI behavior, and combat.
//!
//! # Architecture
//! - `types`: Species definitions, stats, attacks
//! - `entity`: Runtime animal instances
//! - `spatial`: Spatial hashing for efficient queries
//! - `manager`: Central animal management
//! - `behavior`: AI state machines
//! - `spawner`: Chunk-based spawning
//! - `combat`: Damage and attack processing
//! - `player_tracking`: Animal-player relationship and reputation
//! - `taming`: Wolf taming and domesticated dog system
//! - `breeding`: Dog breeding and lineage tracking
//! - `horses`: Complete horse system with Horse-Encephalon AI
//! - `quadruped_ik`: Runtime IK for ground adaptation

pub mod types;
pub mod entity;
pub mod spatial;
pub mod manager;
pub mod behavior;
pub mod spawner;
pub mod combat;
pub mod player_tracking;
pub mod taming;
pub mod breeding;
pub mod horses;
pub mod quadruped_ik;

pub use types::*;
pub use entity::AnimationState;
pub use manager::AnimalManager;
pub use behavior::{BehaviorState, CuriousState};
pub use spawner::AnimalSpawner;
pub use player_tracking::{PlayerWildlifeReputation, LegendaryAnimal};
pub use taming::{Dog, DogId, DogState, DogCommand, TamingSystem, TamingAction, TamingResult, NaturalistProfile};
pub use breeding::{DogKennel, Puppy, BreedingResult, BreedingStats};

// Quadruped IK exports
pub use quadruped_ik::{
    QuadrupedIK, QuadrupedConfig, FootPlacement, LegBoneIndices,
    TwoBoneIKResult, solve_two_bone_ik, calculate_foot_phase, get_foot_ik_blend,
};

// Horse system exports
pub use horses::{
    // Types
    HorseSpecies, HorseStats, HorseCoat, HorseGender, HorseAge,
    HorseHabitat, HorseUse, HerdType,
    // Entity
    Horse, HorseId, MountState,
    // Encephalon AI
    HorseEncephalon, EmotionalState, PersonalityTrait, MemoryType,
    // Taming
    HorseTamingSystem, TamingPhase, TamingProgress,
    // Training
    TrainingSystem, TrainingSkill, SkillLevel,
    // Perks
    HorsePerkTree, PerkBranch, HorsePerk,
    // Stable
    Stable, StabledHorse,
};
