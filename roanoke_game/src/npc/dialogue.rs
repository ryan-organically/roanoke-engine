//! Dialogue System
//!
//! Branching dialogue trees with condition checks and effects.

use crate::progression::reputation::{Faction, ReputationLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dialogue manager for all conversations
#[derive(Debug, Clone, Default)]
pub struct DialogueManager {
    /// All dialogue trees indexed by ID
    pub trees: HashMap<String, DialogueTree>,
    /// Current active dialogue
    pub active_dialogue: Option<ActiveDialogue>,
    /// Dialogue history for this session
    pub history: Vec<DialogueHistoryEntry>,
}

impl DialogueManager {
    pub fn new() -> Self {
        let mut manager = Self::default();
        manager.initialize_dialogues();
        manager
    }

    /// Initialize all dialogue trees
    fn initialize_dialogues(&mut self) {
        // Elder introduction dialogue
        self.add_tree(DialogueTree {
            id: "elder_intro".to_string(),
            npc_id: 1,
            nodes: vec![
                DialogueNode {
                    id: "start".to_string(),
                    speaker: "Village Elder".to_string(),
                    text: "Ah, a stranger from across the great water. I am Tawenho, elder of this village. What brings you to our lands?".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I am searching for the lost English colony.".to_string(),
                            next_node: Some("colony_question".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "I mean no harm. I am lost and hungry.".to_string(),
                            next_node: Some("hungry_response".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::NativeCouncil, delta: 5 },
                            ],
                        },
                        DialogueChoice {
                            text: "What is this place?".to_string(),
                            next_node: Some("place_info".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("elder_neutral".to_string()),
                },
                DialogueNode {
                    id: "colony_question".to_string(),
                    speaker: "Village Elder".to_string(),
                    text: "The pale ones who came before you? Some joined our people. Others... the forest claimed them. We speak of this only to those who prove themselves friends.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "How can I prove myself?".to_string(),
                            next_node: Some("prove_yourself".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::StartQuest("main_02_proving".to_string()),
                            ],
                        },
                        DialogueChoice {
                            text: "I understand. Thank you for speaking with me.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::NativeCouncil, delta: 10 },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("elder_serious".to_string()),
                },
                DialogueNode {
                    id: "hungry_response".to_string(),
                    speaker: "Village Elder".to_string(),
                    text: "The forest provides for those who respect her. Take this, stranger. Rest. When you are ready, we may have tasks that need doing.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "Thank you for your kindness.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::GiveItem { item: "dried_meat".to_string(), count: 3 },
                                DialogueEffect::ModifyReputation { faction: Faction::NativeCouncil, delta: 15 },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("elder_kind".to_string()),
                },
                DialogueNode {
                    id: "place_info".to_string(),
                    speaker: "Village Elder".to_string(),
                    text: "This is Croatoan, land of the turtle clan. We have lived here since before memory. The waters teem with fish, the forests with game. But danger lurks as well.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "What dangers?".to_string(),
                            next_node: Some("dangers_info".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "Tell me about the turtle clan.".to_string(),
                            next_node: Some("clan_info".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("elder_neutral".to_string()),
                },
                DialogueNode {
                    id: "dangers_info".to_string(),
                    speaker: "Village Elder".to_string(),
                    text: "Great cats stalk the mountains. Wolves hunt in packs. The swamps hold scaled beasts with terrible jaws. And serpents... they hide everywhere. Learn our ways, or the land will claim you.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "Can your hunters teach me?".to_string(),
                            next_node: Some("hunter_offer".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "I appreciate the warning.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("elder_serious".to_string()),
                },
                DialogueNode {
                    id: "clan_info".to_string(),
                    speaker: "Village Elder".to_string(),
                    text: "The turtle is patient and wise. It carries its home upon its back, never lost. We are the same - wherever we go, our ancestors travel with us. This land holds their bones and their spirits.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "A beautiful tradition.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::NativeCouncil, delta: 10 },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("elder_kind".to_string()),
                },
                DialogueNode {
                    id: "prove_yourself".to_string(),
                    speaker: "Village Elder".to_string(),
                    text: "Our village needs food. The wild boar are plentiful this season. Hunt three and bring them to us. Also, our healer needs herbs from the forest. Do these things, and we will speak further.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I will do as you ask.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::CompleteObjective { quest: "main_01_lost_colony".to_string(), objective: "speak_elder".to_string() },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("elder_neutral".to_string()),
                },
                DialogueNode {
                    id: "hunter_offer".to_string(),
                    speaker: "Village Elder".to_string(),
                    text: "Our hunters might teach you... if you earn their respect. Speak with Moheda, our hunt master. But first, prove you are no threat to our people.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "How do I prove myself?".to_string(),
                            next_node: Some("prove_yourself".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("elder_neutral".to_string()),
                },
            ],
            entry_node: "start".to_string(),
            requirements: vec![],
        });

        // Hunter NPC dialogue
        self.add_tree(DialogueTree {
            id: "hunter_intro".to_string(),
            npc_id: 5,
            nodes: vec![
                DialogueNode {
                    id: "start".to_string(),
                    speaker: "Moheda".to_string(),
                    text: "You move like a deer in the forest - loud and careless. The predators must think you a gift from the spirits.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "Teach me to hunt properly.".to_string(),
                            next_node: Some("teach_request".to_string()),
                            conditions: vec![
                                DialogueCondition::ReputationLevel { faction: Faction::NativeCouncil, min_level: ReputationLevel::Friendly },
                            ],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "I can handle myself.".to_string(),
                            next_node: Some("arrogant_response".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::Hunters, delta: -5 },
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
                    portrait: Some("hunter_skeptical".to_string()),
                },
                DialogueNode {
                    id: "teach_request".to_string(),
                    speaker: "Moheda".to_string(),
                    text: "Hmm. The elder speaks well of you. Very well. First, show me you understand the basics. Track a deer without alerting it. Perform a silent kill. Skin your prey without wasting the hide. Then we talk.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I accept your challenge.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::StartQuest("side_hunting_01".to_string()),
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("hunter_neutral".to_string()),
                },
                DialogueNode {
                    id: "arrogant_response".to_string(),
                    speaker: "Moheda".to_string(),
                    text: "*laughs* The bears will feast well on your bones. Come back when you've learned humility - if you live that long.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "[Leave]".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("hunter_amused".to_string()),
                },
            ],
            entry_node: "start".to_string(),
            requirements: vec![],
        });

        // Shaman dialogue
        self.add_tree(DialogueTree {
            id: "shaman_intro".to_string(),
            npc_id: 3,
            nodes: vec![
                DialogueNode {
                    id: "start".to_string(),
                    speaker: "Kanehti".to_string(),
                    text: "*peers at you with ancient eyes* The spirits told me you would come. A wanderer between worlds. What do you seek from the keeper of sacred mysteries?".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "The spirits spoke of me?".to_string(),
                            next_node: Some("spirits_speak".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "I found these old bones. What are they?".to_string(),
                            next_node: Some("fossil_question".to_string()),
                            conditions: vec![
                                DialogueCondition::HasItem("fossil".to_string()),
                            ],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "I seek healing.".to_string(),
                            next_node: Some("healing_request".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("shaman_mysterious".to_string()),
                },
                DialogueNode {
                    id: "spirits_speak".to_string(),
                    speaker: "Kanehti".to_string(),
                    text: "They speak of many things to those who listen. Fire in the water. Giants walking. The tree that bleeds. Your coming was foreseen long ago. You have a part to play in what comes.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "What comes?".to_string(),
                            next_node: Some("prophecy".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "I don't believe in spirits.".to_string(),
                            next_node: Some("skeptic_response".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::Shamans, delta: -10 },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("shaman_mysterious".to_string()),
                },
                DialogueNode {
                    id: "fossil_question".to_string(),
                    speaker: "Kanehti".to_string(),
                    text: "*takes the bone reverently* These are the tongue stones - teeth of the great serpent that swam in the waters before the world was young. They hold power against venom. And these... the bones of giants who walked when the ice covered the land.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "Giants? The ice? Tell me more.".to_string(),
                            next_node: Some("ancient_lore".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "Are they valuable?".to_string(),
                            next_node: Some("fossil_value".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("shaman_reverent".to_string()),
                },
                DialogueNode {
                    id: "ancient_lore".to_string(),
                    speaker: "Kanehti".to_string(),
                    text: "In the time before time, great beasts roamed. Hairy giants with curved tusks. Wolves larger than men. Cats with teeth like knives. The spirits tell us our ancestors hunted them. Then the ice came, and the great ones passed into the earth. Now only their bones remain, holding echoes of their power.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "Can this power be used?".to_string(),
                            next_node: Some("fossil_power".to_string()),
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::StartQuest("side_dig_01".to_string()),
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("shaman_teaching".to_string()),
                },
                DialogueNode {
                    id: "fossil_power".to_string(),
                    speaker: "Kanehti".to_string(),
                    text: "For those who know the ways, yes. A tongue stone worn close to the heart protects against serpent venom. A giant's tooth grants courage in battle. If you bring me more of these relics, I will teach you their secrets.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I will search for more.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::ModifyReputation { faction: Faction::Shamans, delta: 20 },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("shaman_pleased".to_string()),
                },
                DialogueNode {
                    id: "fossil_value".to_string(),
                    speaker: "Kanehti".to_string(),
                    text: "*frowns* Value is not measured in beads and shells. But yes, the pale traders from across the water pay much for such curiosities. They do not understand what they buy.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I meant no disrespect.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("shaman_disapproving".to_string()),
                },
                DialogueNode {
                    id: "prophecy".to_string(),
                    speaker: "Kanehti".to_string(),
                    text: "A reckoning. The balance shifts. Old powers stir. But the future is not fixed - it bends like a river around stones. Your choices matter, stranger. Choose wisely.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I will try.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![
                                DialogueEffect::SetFlag { flag: "heard_prophecy".to_string(), value: true },
                            ],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("shaman_serious".to_string()),
                },
                DialogueNode {
                    id: "skeptic_response".to_string(),
                    speaker: "Kanehti".to_string(),
                    text: "Belief is not required for truth. The sun rises whether you believe in it or not. Go. Perhaps experience will teach you what words cannot.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "[Leave]".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("shaman_dismissive".to_string()),
                },
                DialogueNode {
                    id: "healing_request".to_string(),
                    speaker: "Kanehti".to_string(),
                    text: "The body's wounds or the spirit's? I can tend both, for a price. Not gold - that means nothing. But service. Fetch me herbs, bring me bones of power, and I will mend what is broken.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "What do you need?".to_string(),
                            next_node: Some("healing_task".to_string()),
                            conditions: vec![],
                            effects: vec![],
                        },
                        DialogueChoice {
                            text: "I'll manage on my own.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("shaman_neutral".to_string()),
                },
                DialogueNode {
                    id: "healing_task".to_string(),
                    speaker: "Kanehti".to_string(),
                    text: "Goldenseal grows near running water. Bring me five bundles, and I will prepare a healing poultice that can mend even grievous wounds.".to_string(),
                    choices: vec![
                        DialogueChoice {
                            text: "I'll find them.".to_string(),
                            next_node: None,
                            conditions: vec![],
                            effects: vec![],
                        },
                    ],
                    auto_continue: false,
                    portrait: Some("shaman_neutral".to_string()),
                },
            ],
            entry_node: "start".to_string(),
            requirements: vec![],
        });
    }

    /// Add a dialogue tree
    pub fn add_tree(&mut self, tree: DialogueTree) {
        self.trees.insert(tree.id.clone(), tree);
    }

    /// Start a dialogue with an NPC
    pub fn start_dialogue(&mut self, dialogue_id: &str) -> Option<&DialogueNode> {
        let tree = self.trees.get(dialogue_id)?;
        let entry_node = tree.entry_node.clone();

        self.active_dialogue = Some(ActiveDialogue {
            tree_id: dialogue_id.to_string(),
            current_node: entry_node.clone(),
            visited_nodes: vec![entry_node.clone()],
        });

        tree.nodes.iter().find(|n| n.id == entry_node)
    }

    /// Get current dialogue node
    pub fn current_node(&self) -> Option<&DialogueNode> {
        let active = self.active_dialogue.as_ref()?;
        let tree = self.trees.get(&active.tree_id)?;
        tree.nodes.iter().find(|n| n.id == active.current_node)
    }

    /// Make a choice in the current dialogue
    pub fn make_choice(&mut self, choice_index: usize) -> Option<Vec<DialogueEffect>> {
        let active = self.active_dialogue.as_mut()?;
        let tree = self.trees.get(&active.tree_id)?;
        let current = tree.nodes.iter().find(|n| n.id == active.current_node)?;

        let choice = current.choices.get(choice_index)?;
        let effects = choice.effects.clone();

        // Record in history
        self.history.push(DialogueHistoryEntry {
            tree_id: active.tree_id.clone(),
            node_id: active.current_node.clone(),
            choice_text: choice.text.clone(),
        });

        // Navigate to next node
        if let Some(next) = &choice.next_node {
            active.current_node = next.clone();
            active.visited_nodes.push(next.clone());
        } else {
            // Dialogue ended
            self.active_dialogue = None;
        }

        Some(effects)
    }

    /// Check if dialogue is active
    pub fn is_active(&self) -> bool {
        self.active_dialogue.is_some()
    }

    /// End current dialogue
    pub fn end_dialogue(&mut self) {
        self.active_dialogue = None;
    }

    /// Get available dialogue choices (filtered by conditions)
    pub fn available_choices(&self, context: &DialogueContext) -> Vec<(usize, &DialogueChoice)> {
        let Some(node) = self.current_node() else {
            return Vec::new();
        };

        node.choices
            .iter()
            .enumerate()
            .filter(|(_, choice)| {
                choice.conditions.iter().all(|cond| cond.check(context))
            })
            .collect()
    }
}

/// Active dialogue state
#[derive(Debug, Clone)]
pub struct ActiveDialogue {
    pub tree_id: String,
    pub current_node: String,
    pub visited_nodes: Vec<String>,
}

/// Dialogue history entry
#[derive(Debug, Clone)]
pub struct DialogueHistoryEntry {
    pub tree_id: String,
    pub node_id: String,
    pub choice_text: String,
}

/// Complete dialogue tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTree {
    pub id: String,
    pub npc_id: u32,
    pub nodes: Vec<DialogueNode>,
    pub entry_node: String,
    pub requirements: Vec<DialogueCondition>,
}

/// Single dialogue node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: String,
    pub speaker: String,
    pub text: String,
    pub choices: Vec<DialogueChoice>,
    pub auto_continue: bool,
    pub portrait: Option<String>,
}

/// Player dialogue choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub text: String,
    pub next_node: Option<String>,
    pub conditions: Vec<DialogueCondition>,
    pub effects: Vec<DialogueEffect>,
}

/// Conditions for dialogue options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueCondition {
    ReputationLevel {
        faction: Faction,
        min_level: ReputationLevel,
    },
    HasItem(String),
    HasSkill(String),
    QuestComplete(String),
    QuestActive(String),
    FlagSet(String),
    StatCheck {
        stat: String,
        min_value: i32,
    },
}

impl DialogueCondition {
    /// Check if condition is met
    pub fn check(&self, context: &DialogueContext) -> bool {
        match self {
            Self::ReputationLevel { faction, min_level } => {
                context.reputation_levels.get(faction)
                    .map(|level| level >= min_level)
                    .unwrap_or(false)
            }
            Self::HasItem(item) => context.inventory.contains(item),
            Self::HasSkill(skill) => context.skills.contains(skill),
            Self::QuestComplete(quest) => context.completed_quests.contains(quest),
            Self::QuestActive(quest) => context.active_quests.contains(quest),
            Self::FlagSet(flag) => context.flags.contains(flag),
            Self::StatCheck { stat, min_value } => {
                context.stats.get(stat).copied().unwrap_or(0) >= *min_value
            }
        }
    }
}

/// Effects triggered by dialogue choices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueEffect {
    ModifyReputation {
        faction: Faction,
        delta: i32,
    },
    GiveItem {
        item: String,
        count: u32,
    },
    TakeItem {
        item: String,
        count: u32,
    },
    StartQuest(String),
    CompleteObjective {
        quest: String,
        objective: String,
    },
    SetFlag {
        flag: String,
        value: bool,
    },
    UnlockTrading(u32),
    TeachSkill(String),
    Heal(f32),
    TeleportPlayer {
        x: f32,
        y: f32,
        z: f32,
    },
    SpawnNpc {
        npc_type: String,
        x: f32,
        z: f32,
    },
    PlaySound(String),
    StartEvent(String),
}

/// Context for evaluating dialogue conditions
#[derive(Debug, Clone, Default)]
pub struct DialogueContext {
    pub reputation_levels: HashMap<Faction, ReputationLevel>,
    pub inventory: Vec<String>,
    pub skills: Vec<String>,
    pub completed_quests: Vec<String>,
    pub active_quests: Vec<String>,
    pub flags: Vec<String>,
    pub stats: HashMap<String, i32>,
}
