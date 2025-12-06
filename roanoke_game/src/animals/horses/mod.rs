//! Horse System Module
//!
//! Implements multiple horse species with the Horse-Encephalon advanced AI engine,
//! comprehensive taming, training, and perk progression systems.
//!
//! # Species
//! - Banker Horse: Coastal/beach adapted, hardy, good stamina
//! - Carolina Marsh Tacky: Wetland specialist, calm temperament
//! - Colonial Spanish: Versatile utility horse, intelligent
//! - Chincoteague Pony: Wild island breed, spirited
//! - Virginia Draught: Heavy work horse, plowing/hauling
//! - Chickasaw: Swift racing breed, high speed
//!
//! # Architecture
//! - `types`: Horse species, stats, and trait definitions
//! - `entity`: Runtime horse instances with Horse-Encephalon AI
//! - `encephalon`: Advanced AI behavioral engine
//! - `taming`: Multi-stage taming progression
//! - `training`: Skill development and specialization
//! - `perks`: Perk tree with 5 progression branches

pub mod types;
pub mod entity;
pub mod encephalon;
pub mod taming;
pub mod training;
pub mod perks;
pub mod stable;

pub use types::*;
pub use entity::{Horse, HorseId, MountState};
pub use encephalon::{HorseEncephalon, EmotionalState, PersonalityTrait, MemoryType};
pub use taming::{HorseTamingSystem, TamingPhase, TamingProgress};
pub use training::{TrainingSystem, TrainingSkill, SkillLevel};
pub use perks::{HorsePerkTree, PerkBranch, HorsePerk};
pub use stable::{Stable, StabledHorse};
