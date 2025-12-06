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

pub mod types;
pub mod entity;
pub mod spatial;
pub mod manager;
pub mod behavior;
pub mod spawner;
pub mod combat;

pub use types::*;
pub use entity::{Animal, AnimalId, Target, DamageSource};
pub use spatial::SpatialHash;
pub use manager::AnimalManager;
pub use behavior::BehaviorState;
pub use spawner::AnimalSpawner;
