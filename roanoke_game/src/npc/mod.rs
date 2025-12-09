//! NPC Interaction System
//!
//! Handles dialogue, trading, relationships, and NPC behaviors.

pub mod dialogue;
pub mod trading;
pub mod relationships;
pub mod npc_manager;
pub mod interaction;

pub use dialogue::DialogueManager;
pub use npc_manager::NpcManager;
pub use interaction::InteractionSystem;
