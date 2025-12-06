//! NPC Interaction System
//!
//! Handles dialogue, trading, relationships, and NPC behaviors.

pub mod dialogue;
pub mod trading;
pub mod relationships;
pub mod npc_manager;

pub use dialogue::DialogueManager;
pub use npc_manager::NpcManager;
