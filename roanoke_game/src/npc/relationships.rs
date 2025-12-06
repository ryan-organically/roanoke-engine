//! NPC Relationship System
//!
//! Tracks individual NPC relationships, memories, and dispositions.

use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Individual NPC relationship with the player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcRelationship {
    pub npc_id: u32,
    pub relationship_type: RelationshipType,
    pub affinity: i32,           // -100 to 100
    pub trust: i32,              // -100 to 100
    pub fear: i32,               // 0 to 100
    pub respect: i32,            // -100 to 100
    pub interactions: u32,
    pub last_interaction: f64,   // Game time
    pub memories: Vec<NpcMemory>,
    pub gifts_received: Vec<String>,
    pub quests_completed: Vec<String>,
}

impl Default for NpcRelationship {
    fn default() -> Self {
        Self {
            npc_id: 0,
            relationship_type: RelationshipType::Stranger,
            affinity: 0,
            trust: 0,
            fear: 0,
            respect: 0,
            interactions: 0,
            last_interaction: 0.0,
            memories: Vec::new(),
            gifts_received: Vec::new(),
            quests_completed: Vec::new(),
        }
    }
}

impl NpcRelationship {
    pub fn new(npc_id: u32) -> Self {
        Self {
            npc_id,
            ..Default::default()
        }
    }

    /// Update relationship type based on current values
    pub fn update_type(&mut self) {
        let total_positive = self.affinity + self.trust + self.respect;
        let total_negative = -self.affinity.min(0) - self.trust.min(0) - self.respect.min(0);

        self.relationship_type = if self.fear > 50 && total_negative > 100 {
            RelationshipType::Feared
        } else if total_negative > 150 {
            RelationshipType::Enemy
        } else if total_negative > 50 {
            RelationshipType::Rival
        } else if total_positive < 50 {
            RelationshipType::Stranger
        } else if total_positive < 150 {
            RelationshipType::Acquaintance
        } else if total_positive < 250 {
            RelationshipType::Friend
        } else if self.trust > 80 && self.respect > 80 {
            RelationshipType::CloseFriend
        } else if self.affinity > 90 && self.trust > 70 {
            RelationshipType::Romantic
        } else {
            RelationshipType::Friend
        };
    }

    /// Record a positive interaction
    pub fn positive_interaction(&mut self, reason: &str, game_time: f64) {
        self.affinity = (self.affinity + 5).min(100);
        self.trust = (self.trust + 3).min(100);
        self.interactions += 1;
        self.last_interaction = game_time;

        self.add_memory(NpcMemory {
            memory_type: MemoryType::Positive,
            description: reason.to_string(),
            timestamp: game_time,
            impact: 5,
        });

        self.update_type();
    }

    /// Record a negative interaction
    pub fn negative_interaction(&mut self, reason: &str, game_time: f64) {
        self.affinity = (self.affinity - 10).max(-100);
        self.trust = (self.trust - 8).max(-100);
        self.interactions += 1;
        self.last_interaction = game_time;

        self.add_memory(NpcMemory {
            memory_type: MemoryType::Negative,
            description: reason.to_string(),
            timestamp: game_time,
            impact: -10,
        });

        self.update_type();
    }

    /// Record a threatening action
    pub fn threaten(&mut self, reason: &str, game_time: f64) {
        self.fear = (self.fear + 20).min(100);
        self.trust = (self.trust - 15).max(-100);
        self.affinity = (self.affinity - 5).max(-100);

        self.add_memory(NpcMemory {
            memory_type: MemoryType::Threatening,
            description: reason.to_string(),
            timestamp: game_time,
            impact: -15,
        });

        self.update_type();
    }

    /// Record receiving a gift
    pub fn receive_gift(&mut self, item: &str, value: u32, game_time: f64) {
        let affinity_gain = (value / 10).min(15) as i32;
        self.affinity = (self.affinity + affinity_gain).min(100);
        self.trust = (self.trust + 2).min(100);
        self.gifts_received.push(item.to_string());

        self.add_memory(NpcMemory {
            memory_type: MemoryType::Gift,
            description: format!("Received gift: {}", item),
            timestamp: game_time,
            impact: affinity_gain,
        });

        self.update_type();
    }

    /// Record completing a quest for this NPC
    pub fn complete_quest(&mut self, quest_id: &str, game_time: f64) {
        self.respect = (self.respect + 15).min(100);
        self.trust = (self.trust + 10).min(100);
        self.affinity = (self.affinity + 5).min(100);
        self.quests_completed.push(quest_id.to_string());

        self.add_memory(NpcMemory {
            memory_type: MemoryType::QuestComplete,
            description: format!("Completed quest: {}", quest_id),
            timestamp: game_time,
            impact: 15,
        });

        self.update_type();
    }

    /// Add a memory, keeping only recent ones
    fn add_memory(&mut self, memory: NpcMemory) {
        self.memories.push(memory);

        // Keep only last 20 memories
        if self.memories.len() > 20 {
            // Remove oldest low-impact memories first
            self.memories.sort_by(|a, b| b.impact.abs().cmp(&a.impact.abs()));
            self.memories.truncate(20);
            self.memories.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
        }
    }

    /// Get greeting based on relationship
    pub fn greeting(&self) -> &'static str {
        match self.relationship_type {
            RelationshipType::Enemy => "You dare show your face here?",
            RelationshipType::Feared => "*backs away nervously*",
            RelationshipType::Rival => "What do you want?",
            RelationshipType::Stranger => "Greetings, traveler.",
            RelationshipType::Acquaintance => "Ah, you again. What brings you?",
            RelationshipType::Friend => "Good to see you, friend!",
            RelationshipType::CloseFriend => "My friend! It has been too long!",
            RelationshipType::Romantic => "My heart is glad to see you.",
        }
    }

    /// Check if NPC will help in combat
    pub fn will_assist_combat(&self) -> bool {
        matches!(self.relationship_type, RelationshipType::CloseFriend | RelationshipType::Friend)
            && self.trust > 50
            && self.fear < 30
    }

    /// Check if NPC offers discounts
    pub fn trade_discount(&self) -> f32 {
        match self.relationship_type {
            RelationshipType::Enemy | RelationshipType::Rival => 0.0, // Won't trade
            RelationshipType::Feared => 0.8, // Fear discount
            RelationshipType::Stranger => 1.0,
            RelationshipType::Acquaintance => 0.95,
            RelationshipType::Friend => 0.9,
            RelationshipType::CloseFriend | RelationshipType::Romantic => 0.8,
        }
    }

    /// Check if NPC shares information
    pub fn shares_secrets(&self) -> bool {
        self.trust > 60 && matches!(
            self.relationship_type,
            RelationshipType::Friend | RelationshipType::CloseFriend | RelationshipType::Romantic
        )
    }

    /// Decay relationship over time (call periodically)
    pub fn decay(&mut self, days_passed: f32) {
        // Relationships decay toward neutral if not maintained
        let decay_rate = 0.5 * days_passed;

        if self.affinity > 0 {
            self.affinity = ((self.affinity as f32) - decay_rate).max(0.0) as i32;
        } else if self.affinity < 0 {
            self.affinity = ((self.affinity as f32) + decay_rate).min(0.0) as i32;
        }

        if self.fear > 0 {
            self.fear = ((self.fear as f32) - decay_rate * 0.5).max(0.0) as i32;
        }

        // Trust decays slower
        if self.trust > 0 {
            self.trust = ((self.trust as f32) - decay_rate * 0.3).max(0.0) as i32;
        }

        self.update_type();
    }
}

/// Relationship types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    Enemy,       // Actively hostile
    Feared,      // Afraid of player
    Rival,       // Competitive/unfriendly
    Stranger,    // No relationship
    Acquaintance,// Knows player
    Friend,      // Positive relationship
    CloseFriend, // Strong bond
    Romantic,    // Romantic interest
}

impl Default for RelationshipType {
    fn default() -> Self {
        Self::Stranger
    }
}

/// NPC memory of an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcMemory {
    pub memory_type: MemoryType,
    pub description: String,
    pub timestamp: f64,
    pub impact: i32,
}

/// Types of memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    Positive,
    Negative,
    Threatening,
    Gift,
    QuestComplete,
    Trade,
    Conversation,
    Witnessed,    // Witnessed player action
    HeardRumor,   // Heard about player from others
}

/// NPC relationship manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipManager {
    pub relationships: HashMap<u32, NpcRelationship>,
}

impl RelationshipManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create relationship with NPC
    pub fn get_or_create(&mut self, npc_id: u32) -> &mut NpcRelationship {
        self.relationships.entry(npc_id).or_insert_with(|| NpcRelationship::new(npc_id))
    }

    /// Get relationship (read only)
    pub fn get(&self, npc_id: u32) -> Option<&NpcRelationship> {
        self.relationships.get(&npc_id)
    }

    /// Record interaction
    pub fn record_interaction(&mut self, npc_id: u32, positive: bool, reason: &str, game_time: f64) {
        let rel = self.get_or_create(npc_id);
        if positive {
            rel.positive_interaction(reason, game_time);
        } else {
            rel.negative_interaction(reason, game_time);
        }
    }

    /// Spread reputation among nearby NPCs
    pub fn spread_reputation(&mut self, source_npc: u32, witnesses: &[u32], positive: bool, game_time: f64) {
        let impact = if positive { 3 } else { -5 };

        for &witness_id in witnesses {
            if witness_id != source_npc {
                let rel = self.get_or_create(witness_id);
                rel.affinity = (rel.affinity + impact).clamp(-100, 100);
                rel.add_memory(NpcMemory {
                    memory_type: MemoryType::Witnessed,
                    description: format!(
                        "Saw {} interaction with another villager",
                        if positive { "positive" } else { "negative" }
                    ),
                    timestamp: game_time,
                    impact,
                });
                rel.update_type();
            }
        }
    }

    /// Decay all relationships
    pub fn decay_all(&mut self, days_passed: f32) {
        for rel in self.relationships.values_mut() {
            rel.decay(days_passed);
        }
    }

    /// Get all NPCs with specific relationship type
    pub fn npcs_with_relationship(&self, rel_type: RelationshipType) -> Vec<u32> {
        self.relationships.iter()
            .filter(|(_, rel)| rel.relationship_type == rel_type)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get average reputation across all NPCs
    pub fn average_reputation(&self) -> i32 {
        if self.relationships.is_empty() {
            return 0;
        }

        let total: i32 = self.relationships.values()
            .map(|r| r.affinity + r.trust + r.respect)
            .sum();

        total / (self.relationships.len() as i32 * 3)
    }
}

/// NPC daily schedule entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub hour_start: u8,
    pub hour_end: u8,
    pub activity: NpcActivity,
    pub location: Vec3,
}

/// NPC activities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcActivity {
    Sleeping,
    Eating,
    Working,
    Socializing,
    Praying,
    Patrolling,
    Trading,
    Crafting,
    Gathering,
    Teaching,
    Resting,
}

impl NpcActivity {
    /// Whether player can interact during this activity
    pub fn allows_interaction(&self) -> bool {
        !matches!(self, Self::Sleeping | Self::Praying | Self::Patrolling)
    }

    /// Greeting modifier during activity
    pub fn greeting_modifier(&self) -> &'static str {
        match self {
            Self::Working => "I'm busy, make it quick. ",
            Self::Eating => "*continues eating* Yes? ",
            Self::Resting => "*sighs* What is it? ",
            _ => "",
        }
    }
}
