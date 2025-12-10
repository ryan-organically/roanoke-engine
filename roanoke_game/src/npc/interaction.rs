//! NPC Interaction System
//!
//! Handles player-to-NPC interactions, dialogue triggering, and relationship effects.
//! This is the bridge between player input, dialogue trees, and relationship consequences.

use super::dialogue::{DialogueContext, DialogueEffect, DialogueManager, DialogueNode};
use super::relationships::{MemoryType, NpcMemory, NpcRelationship, RelationshipManager, RelationshipType};
use crate::progression::reputation::{Faction, ReputationLevel};
use glam::Vec3;
use std::collections::HashMap;

/// NPC interaction system - bridges player actions to dialogue and relationships
#[derive(Debug)]
pub struct InteractionSystem {
    /// Dialogue manager
    pub dialogue: DialogueManager,
    /// Relationship manager
    pub relationships: RelationshipManager,
    /// Currently interacting NPC
    pub interacting_npc: Option<InteractingNpc>,
    /// Pending effects to process
    pub pending_effects: Vec<PendingEffect>,
    /// Recent relationship changes (for UI display)
    pub recent_changes: Vec<RelationshipChange>,
    /// NPC dialogue mappings (NPC name/role -> dialogue tree ID)
    npc_dialogue_map: HashMap<String, String>,
    /// Dynamic dialogue generators
    dynamic_greetings: bool,
}

/// Currently interacting NPC data
#[derive(Debug, Clone)]
pub struct InteractingNpc {
    pub npc_index: usize,
    pub npc_name: String,
    pub npc_role: String,
    pub position: Vec3,
    pub relationship: NpcRelationship,
}

/// Pending effect to be processed by game systems
#[derive(Debug, Clone)]
pub enum PendingEffect {
    ModifyReputation { faction: Faction, delta: i32 },
    GiveItem { item: String, count: u32 },
    TakeItem { item: String, count: u32 },
    StartQuest(String),
    CompleteObjective { quest: String, objective: String },
    SetFlag { flag: String, value: bool },
    UnlockTrading(u32),
    TeachSkill(String),
    Heal(f32),
    ModifyRelationship { npc_id: u32, affinity: i32, trust: i32, respect: i32 },
    SpreadRumor { positive: bool, radius: f32 },
}

/// Recent relationship change for UI feedback
#[derive(Debug, Clone)]
pub struct RelationshipChange {
    pub npc_name: String,
    pub change_type: RelationshipChangeType,
    pub description: String,
    pub timestamp: f64,
}

/// Type of relationship change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipChangeType {
    AffinityUp,
    AffinityDown,
    TrustUp,
    TrustDown,
    RespectUp,
    RespectDown,
    FearUp,
    RelationshipUpgrade,
    RelationshipDowngrade,
}

impl RelationshipChangeType {
    pub fn color(&self) -> [f32; 3] {
        match self {
            Self::AffinityUp | Self::TrustUp | Self::RespectUp => [0.3, 0.8, 0.3],
            Self::AffinityDown | Self::TrustDown | Self::RespectDown | Self::FearUp => [0.8, 0.3, 0.3],
            Self::RelationshipUpgrade => [0.3, 0.8, 0.9],
            Self::RelationshipDowngrade => [0.8, 0.5, 0.2],
        }
    }
}

impl Default for InteractionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractionSystem {
    pub fn new() -> Self {
        let mut system = Self {
            dialogue: DialogueManager::new(),
            relationships: RelationshipManager::new(),
            interacting_npc: None,
            pending_effects: Vec::new(),
            recent_changes: Vec::new(),
            npc_dialogue_map: HashMap::new(),
            dynamic_greetings: true,
        };

        // Map NPC roles to dialogue trees
        system.npc_dialogue_map.insert("Chief".to_string(), "elder_intro".to_string());
        system.npc_dialogue_map.insert("Elder".to_string(), "elder_intro".to_string());
        system.npc_dialogue_map.insert("Shaman".to_string(), "shaman_intro".to_string());
        system.npc_dialogue_map.insert("Hunter".to_string(), "hunter_intro".to_string());
        system.npc_dialogue_map.insert("Warrior".to_string(), "warrior_intro".to_string());
        system.npc_dialogue_map.insert("Farmer".to_string(), "villager_intro".to_string());
        system.npc_dialogue_map.insert("Craftsperson".to_string(), "craftsman_intro".to_string());
        system.npc_dialogue_map.insert("Villager".to_string(), "villager_intro".to_string());

        // Add more dialogue trees
        system.initialize_additional_dialogues();

        system
    }

    /// Initialize additional dialogue trees for various NPCs
    fn initialize_additional_dialogues(&mut self) {
        use super::dialogue::{DialogueChoice, DialogueNode, DialogueTree, DialogueCondition};

        // Warrior dialogue
        self.dialogue.add_tree(DialogueTree {
            id: "warrior_intro".to_string(),
            npc_id: 2,
            nodes: vec![
                DialogueNode {
                    id: "start".to_string(),
                    speaker: "Warrior".to_string(),
                    text: "*eyes you warily* Another outsider. State your purpose.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I come in peace. I seek only shelter.".to_string(),
                            next_node: Some("peaceful".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::Warriors, delta: 5 },
                            ],
                        },
                        DialogueChoice {
                            text: "I'm looking for the lost colony.".to_string(),
                            next_node: Some("colony".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "I don't answer to you.".to_string(),
                            next_node: Some("hostile".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::Warriors, delta: -15 },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("warrior_stern".to_string()),
                },
                DialogueNode {
                    id: "peaceful".to_string(),
                    speaker: "Warrior".to_string(),
                    text: "*relaxes slightly* Peace is good. The elder may speak with you. But know this - if you bring harm to our people, you will answer to my blade.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I understand. Thank you.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::NativeCouncil, delta: 5 },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("warrior_neutral".to_string()),
                },
                DialogueNode {
                    id: "colony".to_string(),
                    speaker: "Warrior".to_string(),
                    text: "The pale ones? *spits* Some were friends. Others brought only death and disease. The spirits took them, one way or another. Speak to the elder if you must know more.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "Where can I find the elder?".to_string(),
                            next_node: Some("elder_location".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "I'm sorry for what my people did.".to_string(),
                            next_node: Some("apology".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::NativeCouncil, delta: 10 },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("warrior_bitter".to_string()),
                },
                DialogueNode {
                    id: "hostile".to_string(),
                    speaker: "Warrior".to_string(),
                    text: "*hand moves to weapon* You will learn respect, outsider. Or you will learn to run. Your choice.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "My apologies. I spoke rashly.".to_string(),
                            next_node: Some("apologize_warrior".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::Warriors, delta: 5 },
                            ],
                        },
                        DialogueChoice {
                            text: "[Leave]".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("warrior_angry".to_string()),
                },
                DialogueNode {
                    id: "elder_location".to_string(),
                    speaker: "Warrior".to_string(),
                    text: "The great longhouse at the village center. Tawenho sits there most days. Show respect when you speak to him.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I will. Thank you.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("warrior_neutral".to_string()),
                },
                DialogueNode {
                    id: "apology".to_string(),
                    speaker: "Warrior".to_string(),
                    text: "*studies you* Words are easy. Actions speak truth. Prove yourself friend to our people, and perhaps... perhaps we can begin again.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I intend to.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::SetFlag { flag: "apologized_to_warrior".to_string(), value: true },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("warrior_thoughtful".to_string()),
                },
                DialogueNode {
                    id: "apologize_warrior".to_string(),
                    speaker: "Warrior".to_string(),
                    text: "*grunts* Good. Pride is a poor shield against arrows. Remember that.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "[Nod and leave]".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("warrior_stern".to_string()),
                },
            ],
            entry_node: "start".to_string(),
            requirements: vec![],
        });

        // Generic villager dialogue
        self.dialogue.add_tree(DialogueTree {
            id: "villager_intro".to_string(),
            npc_id: 0,
            nodes: vec![
                DialogueNode {
                    id: "start".to_string(),
                    speaker: "Villager".to_string(),
                    text: "*looks up from work* Oh! A stranger. We don't see many outsiders here.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I'm new to these lands. Can you tell me about your village?".to_string(),
                            next_node: Some("village_info".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "What are you working on?".to_string(),
                            next_node: Some("work_info".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::NativeCouncil, delta: 3 },
                            ],
                        },
                        DialogueChoice {
                            text: "Never mind.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("villager_curious".to_string()),
                },
                DialogueNode {
                    id: "village_info".to_string(),
                    speaker: "Villager".to_string(),
                    text: "This is Croatoan, home of the Turtle Clan. We fish, farm, and hunt. The elder Tawenho leads our council. The shaman Kanehti speaks with spirits. We are a simple people, but proud.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "It seems peaceful here.".to_string(),
                            next_node: Some("peaceful_response".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::NativeCouncil, delta: 5 },
                            ],
                        },
                        DialogueChoice {
                            text: "Thank you for the information.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("villager_proud".to_string()),
                },
                DialogueNode {
                    id: "work_info".to_string(),
                    speaker: "Villager".to_string(),
                    text: "*smiles* Preparing hides for winter clothing. The work is hard but satisfying. Each piece I make will keep someone warm when the cold winds blow.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "Can you teach me?".to_string(),
                            next_node: Some("teach_craft".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "That's admirable work.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::NativeCouncil, delta: 5 },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("villager_working".to_string()),
                },
                DialogueNode {
                    id: "peaceful_response".to_string(),
                    speaker: "Villager".to_string(),
                    text: "Most days, yes. But dangers lurk in the forest, and not all tribes are friendly. We must always be watchful.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I'll keep that in mind.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("villager_thoughtful".to_string()),
                },
                DialogueNode {
                    id: "teach_craft".to_string(),
                    speaker: "Villager".to_string(),
                    text: "Perhaps... if you prove yourself a friend to our village. Bring me five deer hides and I will show you the basics of our craft.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I'll gather the hides.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::StartQuest("side_crafting_01".to_string()),
                            ],
                        },
                        DialogueChoice {
                            text: "Maybe another time.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("villager_considering".to_string()),
                },
            ],
            entry_node: "start".to_string(),
            requirements: vec![],
        });

        // Craftsman dialogue
        self.dialogue.add_tree(DialogueTree {
            id: "craftsman_intro".to_string(),
            npc_id: 4,
            nodes: vec![
                DialogueNode {
                    id: "start".to_string(),
                    speaker: "Craftsman".to_string(),
                    text: "*examining a half-finished bow* Hmm? What do you need, stranger?".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I need weapons and tools.".to_string(),
                            next_node: Some("trade_offer".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "Your craftsmanship looks impressive.".to_string(),
                            next_node: Some("compliment".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::NativeCouncil, delta: 5 },
                            ],
                        },
                        DialogueChoice {
                            text: "Nothing, sorry to bother you.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("craftsman_busy".to_string()),
                },
                DialogueNode {
                    id: "trade_offer".to_string(),
                    speaker: "Craftsman".to_string(),
                    text: "I make the best bows in three villages. But quality costs. What do you have to trade?".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "[Open Trade]".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::UnlockTrading(4),
                            ],
                        },
                        DialogueChoice {
                            text: "I'll come back when I have more.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("craftsman_business".to_string()),
                },
                DialogueNode {
                    id: "compliment".to_string(),
                    speaker: "Craftsman".to_string(),
                    text: "*beams with pride* My father taught me, and his father before him. Each bow carries the spirit of our ancestors. Here - *hands you an arrowhead* - take this. May it fly true.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "Thank you. I'll treasure it.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::GiveItem { item: "flint_arrowhead".to_string(), count: 1 },
                                DialogueEffect::ModifyReputation { faction: Faction::NativeCouncil, delta: 5 },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("craftsman_pleased".to_string()),
                },
            ],
            entry_node: "start".to_string(),
            requirements: vec![],
        });
    }

    /// Start interaction with an NPC
    pub fn start_interaction(
        &mut self,
        npc_index: usize,
        npc_name: &str,
        npc_role: &str,
        position: Vec3,
        game_time: f64,
    ) -> Option<DialogueUIData> {
        // Get or create NPC ID based on name hash
        let npc_id = self.name_to_id(npc_name);

        // Get relationship
        let relationship = self.relationships.get_or_create(npc_id).clone();

        // Store interacting NPC
        self.interacting_npc = Some(InteractingNpc {
            npc_index,
            npc_name: npc_name.to_string(),
            npc_role: npc_role.to_string(),
            position,
            relationship: relationship.clone(),
        });

        // Determine dialogue tree based on role and relationship
        let dialogue_id = self.get_dialogue_for_npc(npc_role, &relationship);

        // Compute greeting prefix before mutable borrow of dialogue
        let greeting_prefix = if self.dynamic_greetings {
            self.get_relationship_greeting(&relationship)
        } else {
            String::new()
        };

        // Start dialogue - clone node data to avoid borrow conflict
        let node_data = self.dialogue.start_dialogue(&dialogue_id).map(|node| {
            (node.speaker.clone(), node.text.clone(), node.portrait.clone())
        });

        if let Some((speaker, text, portrait)) = node_data {
            // Record interaction
            self.relationships.get_or_create(npc_id).interactions += 1;
            self.relationships.get_or_create(npc_id).last_interaction = game_time;

            // Get available choices from dialogue
            let context = self.create_dialogue_context();
            let available_choices = self.dialogue.available_choices(&context);

            let ui_data = DialogueUIData {
                speaker_name: speaker,
                text: format!("{}{}", greeting_prefix, text),
                portrait,
                choices: available_choices
                    .iter()
                    .map(|(idx, choice)| DialogueChoiceUI {
                        index: *idx,
                        text: choice.text.clone(),
                        has_effect: !choice.effects.is_empty(),
                        locked: false,
                        lock_reason: None,
                    })
                    .collect(),
                relationship_status: Some(RelationshipStatusUI {
                    relationship_type: format!("{:?}", relationship.relationship_type),
                    affinity: relationship.affinity,
                    trust: relationship.trust,
                    respect: relationship.respect,
                }),
            };
            return Some(ui_data);
        }

        None
    }

    /// Get appropriate dialogue tree for NPC based on role and relationship
    fn get_dialogue_for_npc(&self, role: &str, relationship: &NpcRelationship) -> String {
        // Check for relationship-specific dialogues first
        let rel_suffix = match relationship.relationship_type {
            RelationshipType::Enemy => "_enemy",
            RelationshipType::Feared => "_feared",
            RelationshipType::Friend | RelationshipType::CloseFriend => "_friend",
            RelationshipType::Romantic => "_romantic",
            _ => "",
        };

        // Try relationship-specific dialogue first
        let rel_specific = format!("{}{}", role.to_lowercase(), rel_suffix);
        if self.dialogue.trees.contains_key(&rel_specific) {
            return rel_specific;
        }

        // Fall back to role-based dialogue
        self.npc_dialogue_map
            .get(role)
            .cloned()
            .unwrap_or_else(|| "villager_intro".to_string())
    }

    /// Generate a greeting based on relationship
    fn get_relationship_greeting(&self, relationship: &NpcRelationship) -> String {
        match relationship.relationship_type {
            RelationshipType::Enemy => {
                if relationship.fear > 50 {
                    "*backs away fearfully* Y-you...! ".to_string()
                } else {
                    "*scowls with hatred* You dare show your face? ".to_string()
                }
            }
            RelationshipType::Feared => "*trembles* P-please don't hurt me... ".to_string(),
            RelationshipType::Rival => "*narrows eyes* What do you want? ".to_string(),
            RelationshipType::Stranger => String::new(), // Use default
            RelationshipType::Acquaintance => "Ah, you again. ".to_string(),
            RelationshipType::Friend => "*smiles warmly* Good to see you, friend! ".to_string(),
            RelationshipType::CloseFriend => "*embraces you* My dear friend! It's been too long! ".to_string(),
            RelationshipType::Romantic => "*eyes light up* My heart sings to see you... ".to_string(),
        }
    }

    /// Create UI data from dialogue node
    fn create_ui_data(&self, node: &DialogueNode, greeting_prefix: &str, relationship: &NpcRelationship) -> DialogueUIData {
        let context = self.create_dialogue_context();
        let available_choices = self.dialogue.available_choices(&context);

        DialogueUIData {
            speaker_name: node.speaker.clone(),
            text: format!("{}{}", greeting_prefix, node.text),
            portrait: node.portrait.clone(),
            choices: available_choices
                .iter()
                .map(|(idx, choice)| DialogueChoiceUI {
                    index: *idx,
                    text: choice.text.clone(),
                    has_effect: !choice.effects.is_empty(),
                    locked: false,
                    lock_reason: None,
                })
                .collect(),
            relationship_status: Some(RelationshipStatusUI {
                relationship_type: format!("{:?}", relationship.relationship_type),
                affinity: relationship.affinity,
                trust: relationship.trust,
                respect: relationship.respect,
            }),
        }
    }

    /// Create dialogue context from current game state
    fn create_dialogue_context(&self) -> DialogueContext {
        // This should be populated from actual game state
        DialogueContext::default()
    }

    /// Make a dialogue choice
    pub fn make_choice(&mut self, choice_index: usize, game_time: f64) -> Option<DialogueUIData> {
        let Some(interacting) = &self.interacting_npc else {
            return None;
        };

        let npc_id = self.name_to_id(&interacting.npc_name);
        let npc_name = interacting.npc_name.clone();
        let relationship = interacting.relationship.clone();

        // Make the choice and get effects
        if let Some(effects) = self.dialogue.make_choice(choice_index) {
            // Process effects
            for effect in effects {
                self.process_dialogue_effect(effect, npc_id, &npc_name, game_time);
            }
        }

        // Check if dialogue continues
        if let Some(node) = self.dialogue.current_node() {
            let greeting = String::new(); // No greeting on continuation
            Some(self.create_ui_data(node, &greeting, &relationship))
        } else {
            // Dialogue ended
            self.end_interaction();
            None
        }
    }

    /// Process a dialogue effect
    fn process_dialogue_effect(&mut self, effect: DialogueEffect, npc_id: u32, npc_name: &str, game_time: f64) {
        match effect {
            DialogueEffect::ModifyReputation { faction, delta } => {
                // Add to pending effects for game to process
                self.pending_effects.push(PendingEffect::ModifyReputation { faction, delta });

                // Also modify individual NPC relationship
                let rel = self.relationships.get_or_create(npc_id);
                if delta > 0 {
                    rel.positive_interaction("Positive dialogue choice", game_time);
                    self.add_relationship_change(npc_name, RelationshipChangeType::AffinityUp,
                        format!("+{} affinity", delta.abs()));
                } else {
                    rel.negative_interaction("Negative dialogue choice", game_time);
                    self.add_relationship_change(npc_name, RelationshipChangeType::AffinityDown,
                        format!("-{} affinity", delta.abs()));
                }
            }
            DialogueEffect::GiveItem { item, count } => {
                self.pending_effects.push(PendingEffect::GiveItem { item: item.clone(), count });

                // Record as gift received (from NPC to player doesn't affect relationship much)
            }
            DialogueEffect::TakeItem { item, count } => {
                self.pending_effects.push(PendingEffect::TakeItem { item, count });
            }
            DialogueEffect::StartQuest(quest) => {
                self.pending_effects.push(PendingEffect::StartQuest(quest));
            }
            DialogueEffect::CompleteObjective { quest, objective } => {
                self.pending_effects.push(PendingEffect::CompleteObjective { quest, objective });
            }
            DialogueEffect::SetFlag { flag, value } => {
                self.pending_effects.push(PendingEffect::SetFlag { flag, value });
            }
            DialogueEffect::UnlockTrading(trader_id) => {
                self.pending_effects.push(PendingEffect::UnlockTrading(trader_id));
            }
            DialogueEffect::TeachSkill(skill) => {
                let skill_name = skill.clone();
                self.pending_effects.push(PendingEffect::TeachSkill(skill));

                // Teaching increases respect
                let rel = self.relationships.get_or_create(npc_id);
                rel.respect = (rel.respect + 10).min(100);
                rel.add_memory(NpcMemory {
                    memory_type: MemoryType::Positive,
                    description: format!("Taught player: {}", skill_name),
                    timestamp: game_time,
                    impact: 10,
                });
                rel.update_type();

                self.add_relationship_change(npc_name, RelationshipChangeType::RespectUp,
                    format!("Learned {} (+10 respect)", skill_name));
            }
            DialogueEffect::Heal(amount) => {
                self.pending_effects.push(PendingEffect::Heal(amount));
            }
            _ => {}
        }
    }

    /// Add a relationship change for UI display
    fn add_relationship_change(&mut self, npc_name: &str, change_type: RelationshipChangeType, description: String) {
        self.recent_changes.push(RelationshipChange {
            npc_name: npc_name.to_string(),
            change_type,
            description,
            timestamp: 0.0, // Will be set by caller
        });

        // Keep only recent changes
        if self.recent_changes.len() > 10 {
            self.recent_changes.remove(0);
        }
    }

    /// End current interaction
    pub fn end_interaction(&mut self) {
        self.dialogue.end_dialogue();
        self.interacting_npc = None;
    }

    /// Check if currently in dialogue
    pub fn is_in_dialogue(&self) -> bool {
        self.dialogue.is_active()
    }

    /// Take pending effects (clears them)
    pub fn take_pending_effects(&mut self) -> Vec<PendingEffect> {
        std::mem::take(&mut self.pending_effects)
    }

    /// Take recent changes (clears them)
    pub fn take_recent_changes(&mut self) -> Vec<RelationshipChange> {
        std::mem::take(&mut self.recent_changes)
    }

    /// Convert NPC name to consistent ID
    fn name_to_id(&self, name: &str) -> u32 {
        let mut hash: u32 = 0;
        for byte in name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }

    /// Gift an item to an NPC
    pub fn give_gift(&mut self, npc_name: &str, item: &str, value: u32, game_time: f64) {
        let npc_id = self.name_to_id(npc_name);
        let rel = self.relationships.get_or_create(npc_id);
        rel.receive_gift(item, value, game_time);

        let change_type = if value >= 50 {
            RelationshipChangeType::TrustUp
        } else {
            RelationshipChangeType::AffinityUp
        };

        self.add_relationship_change(npc_name, change_type,
            format!("Received gift: {}", item));
    }

    /// Threaten an NPC
    pub fn threaten_npc(&mut self, npc_name: &str, reason: &str, game_time: f64) {
        let npc_id = self.name_to_id(npc_name);
        let rel = self.relationships.get_or_create(npc_id);
        rel.threaten(reason, game_time);

        self.add_relationship_change(npc_name, RelationshipChangeType::FearUp,
            format!("Threatened: {}", reason));

        // Spread negative reputation to witnesses
        self.pending_effects.push(PendingEffect::SpreadRumor {
            positive: false,
            radius: 30.0
        });
    }

    /// Attack an NPC (severe relationship damage)
    pub fn attack_npc(&mut self, npc_name: &str, game_time: f64) {
        let npc_id = self.name_to_id(npc_name);
        let rel = self.relationships.get_or_create(npc_id);

        rel.affinity = -100;
        rel.trust = -100;
        rel.fear = 80;
        rel.add_memory(NpcMemory {
            memory_type: MemoryType::Threatening,
            description: "Attacked by player".to_string(),
            timestamp: game_time,
            impact: -100,
        });
        rel.update_type();

        self.add_relationship_change(npc_name, RelationshipChangeType::RelationshipDowngrade,
            "ATTACKED - Now enemies!".to_string());

        // Major negative reputation spread
        self.pending_effects.push(PendingEffect::SpreadRumor {
            positive: false,
            radius: 100.0
        });
        self.pending_effects.push(PendingEffect::ModifyReputation {
            faction: Faction::NativeCouncil,
            delta: -50
        });
    }

    /// Complete a quest for an NPC
    pub fn complete_quest_for_npc(&mut self, npc_name: &str, quest_id: &str, game_time: f64) {
        let npc_id = self.name_to_id(npc_name);
        let rel = self.relationships.get_or_create(npc_id);
        rel.complete_quest(quest_id, game_time);

        self.add_relationship_change(npc_name, RelationshipChangeType::RespectUp,
            format!("Quest complete: {} (+15 respect)", quest_id));
    }

    /// Decay all relationships (call daily)
    pub fn daily_update(&mut self, days_passed: f32) {
        self.relationships.decay_all(days_passed);
    }

    /// Get relationship summary for an NPC
    pub fn get_relationship_summary(&self, npc_name: &str) -> Option<RelationshipSummary> {
        let npc_id = self.name_to_id(npc_name);
        self.relationships.get(npc_id).map(|rel| RelationshipSummary {
            npc_name: npc_name.to_string(),
            relationship_type: rel.relationship_type,
            affinity: rel.affinity,
            trust: rel.trust,
            respect: rel.respect,
            fear: rel.fear,
            interactions: rel.interactions,
            will_trade: rel.trade_discount() > 0.0,
            trade_discount: 1.0 - rel.trade_discount(),
            shares_secrets: rel.shares_secrets(),
            will_help_combat: rel.will_assist_combat(),
        })
    }
}

/// Dialogue UI data for rendering
#[derive(Debug, Clone)]
pub struct DialogueUIData {
    pub speaker_name: String,
    pub text: String,
    pub portrait: Option<String>,
    pub choices: Vec<DialogueChoiceUI>,
    pub relationship_status: Option<RelationshipStatusUI>,
}

/// Single dialogue choice for UI
#[derive(Debug, Clone)]
pub struct DialogueChoiceUI {
    pub index: usize,
    pub text: String,
    pub has_effect: bool,
    pub locked: bool,
    pub lock_reason: Option<String>,
}

/// Relationship status for UI
#[derive(Debug, Clone)]
pub struct RelationshipStatusUI {
    pub relationship_type: String,
    pub affinity: i32,
    pub trust: i32,
    pub respect: i32,
}

/// Relationship summary
#[derive(Debug, Clone)]
pub struct RelationshipSummary {
    pub npc_name: String,
    pub relationship_type: RelationshipType,
    pub affinity: i32,
    pub trust: i32,
    pub respect: i32,
    pub fear: i32,
    pub interactions: u32,
    pub will_trade: bool,
    pub trade_discount: f32,
    pub shares_secrets: bool,
    pub will_help_combat: bool,
}

