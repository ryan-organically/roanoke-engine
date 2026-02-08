//! NPC Manager
//!
//! Central management of all NPC instances and their behaviors.

use super::relationships::{NpcRelationship, RelationshipManager, ScheduleEntry, NpcActivity};
use super::dialogue::DialogueManager;
use super::trading::TradingSystem;
use super::utility_ai::{UtilityEvaluator, build_context, NpcAction};
use crate::character_agent::{
    AgentContext, AgentId, AgentKind, CharacterAgent, EmotionalState,
    OrbVisualData, UnifiedBehaviorState,
};
use crate::progression::reputation::{Faction, ReputationLevel};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Central NPC manager
pub struct NpcManager {
    /// All NPC instances
    pub npcs: HashMap<u32, NpcInstance>,
    /// Next NPC ID
    next_id: u32,
    /// Player relationships with NPCs
    pub relationships: RelationshipManager,
    /// Dialogue system
    pub dialogue: DialogueManager,
    /// Trading system
    pub trading: TradingSystem,
    /// Current game time (hours)
    pub game_time: f32,
    /// Utility AI evaluator for decision making
    utility_evaluator: UtilityEvaluator,
    /// Enable utility AI (can be toggled for debugging)
    pub use_utility_ai: bool,
}

impl NpcManager {
    pub fn new() -> Self {
        let mut manager = Self {
            npcs: HashMap::new(),
            next_id: 1,
            relationships: RelationshipManager::new(),
            dialogue: DialogueManager::new(),
            trading: TradingSystem::new(),
            game_time: 8.0, // Start at 8 AM
            utility_evaluator: UtilityEvaluator::new(),
            use_utility_ai: true, // Enable by default
        };
        manager.initialize_village_npcs();
        manager
    }

    /// Initialize NPCs for the starting village
    fn initialize_village_npcs(&mut self) {
        // Village Elder
        self.spawn_npc(NpcTemplate {
            name: "Tawenho".to_string(),
            role: NpcRole::Elder,
            position: Vec3::new(0.0, 0.0, 0.0),
            village_id: Some(1),
            dialogue_id: Some("elder_intro".to_string()),
            schedule: vec![
                ScheduleEntry { hour_start: 6, hour_end: 8, activity: NpcActivity::Eating, location: Vec3::new(0.0, 0.0, 5.0) },
                ScheduleEntry { hour_start: 8, hour_end: 12, activity: NpcActivity::Teaching, location: Vec3::new(0.0, 0.0, 0.0) },
                ScheduleEntry { hour_start: 12, hour_end: 14, activity: NpcActivity::Eating, location: Vec3::new(0.0, 0.0, 5.0) },
                ScheduleEntry { hour_start: 14, hour_end: 18, activity: NpcActivity::Socializing, location: Vec3::new(10.0, 0.0, 0.0) },
                ScheduleEntry { hour_start: 18, hour_end: 21, activity: NpcActivity::Praying, location: Vec3::new(-20.0, 0.0, 0.0) },
                ScheduleEntry { hour_start: 21, hour_end: 6, activity: NpcActivity::Sleeping, location: Vec3::new(0.0, 0.0, 10.0) },
            ],
        });

        // Warrior Chief
        self.spawn_npc(NpcTemplate {
            name: "Askook".to_string(),
            role: NpcRole::Warrior,
            position: Vec3::new(15.0, 0.0, 10.0),
            village_id: Some(1),
            dialogue_id: Some("warrior_intro".to_string()),
            schedule: vec![
                ScheduleEntry { hour_start: 5, hour_end: 8, activity: NpcActivity::Patrolling, location: Vec3::new(30.0, 0.0, 30.0) },
                ScheduleEntry { hour_start: 8, hour_end: 12, activity: NpcActivity::Working, location: Vec3::new(15.0, 0.0, 10.0) },
                ScheduleEntry { hour_start: 12, hour_end: 14, activity: NpcActivity::Eating, location: Vec3::new(0.0, 0.0, 5.0) },
                ScheduleEntry { hour_start: 14, hour_end: 20, activity: NpcActivity::Teaching, location: Vec3::new(20.0, 0.0, 0.0) },
                ScheduleEntry { hour_start: 20, hour_end: 5, activity: NpcActivity::Sleeping, location: Vec3::new(15.0, 0.0, 15.0) },
            ],
        });

        // Shaman
        self.spawn_npc(NpcTemplate {
            name: "Kanehti".to_string(),
            role: NpcRole::Shaman,
            position: Vec3::new(-20.0, 0.0, 5.0),
            village_id: Some(1),
            dialogue_id: Some("shaman_intro".to_string()),
            schedule: vec![
                ScheduleEntry { hour_start: 4, hour_end: 7, activity: NpcActivity::Praying, location: Vec3::new(-30.0, 0.0, 0.0) },
                ScheduleEntry { hour_start: 7, hour_end: 12, activity: NpcActivity::Working, location: Vec3::new(-20.0, 0.0, 5.0) },
                ScheduleEntry { hour_start: 12, hour_end: 14, activity: NpcActivity::Eating, location: Vec3::new(-20.0, 0.0, 10.0) },
                ScheduleEntry { hour_start: 14, hour_end: 18, activity: NpcActivity::Gathering, location: Vec3::new(-50.0, 0.0, -20.0) },
                ScheduleEntry { hour_start: 18, hour_end: 22, activity: NpcActivity::Trading, location: Vec3::new(-20.0, 0.0, 5.0) },
                ScheduleEntry { hour_start: 22, hour_end: 4, activity: NpcActivity::Sleeping, location: Vec3::new(-20.0, 0.0, 15.0) },
            ],
        });

        // Farmer
        self.spawn_npc(NpcTemplate {
            name: "Onatah".to_string(),
            role: NpcRole::Farmer,
            position: Vec3::new(30.0, 0.0, -20.0),
            village_id: Some(1),
            dialogue_id: None,
            schedule: vec![
                ScheduleEntry { hour_start: 5, hour_end: 12, activity: NpcActivity::Working, location: Vec3::new(50.0, 0.0, -30.0) },
                ScheduleEntry { hour_start: 12, hour_end: 14, activity: NpcActivity::Eating, location: Vec3::new(30.0, 0.0, -15.0) },
                ScheduleEntry { hour_start: 14, hour_end: 18, activity: NpcActivity::Working, location: Vec3::new(50.0, 0.0, -30.0) },
                ScheduleEntry { hour_start: 18, hour_end: 20, activity: NpcActivity::Socializing, location: Vec3::new(10.0, 0.0, 0.0) },
                ScheduleEntry { hour_start: 20, hour_end: 5, activity: NpcActivity::Sleeping, location: Vec3::new(30.0, 0.0, -10.0) },
            ],
        });

        // Hunter
        self.spawn_npc(NpcTemplate {
            name: "Moheda".to_string(),
            role: NpcRole::Hunter,
            position: Vec3::new(-40.0, 0.0, -30.0),
            village_id: Some(1),
            dialogue_id: Some("hunter_intro".to_string()),
            schedule: vec![
                ScheduleEntry { hour_start: 4, hour_end: 12, activity: NpcActivity::Working, location: Vec3::new(-100.0, 0.0, -100.0) },
                ScheduleEntry { hour_start: 12, hour_end: 14, activity: NpcActivity::Eating, location: Vec3::new(-40.0, 0.0, -25.0) },
                ScheduleEntry { hour_start: 14, hour_end: 18, activity: NpcActivity::Teaching, location: Vec3::new(-40.0, 0.0, -30.0) },
                ScheduleEntry { hour_start: 18, hour_end: 20, activity: NpcActivity::Trading, location: Vec3::new(-40.0, 0.0, -30.0) },
                ScheduleEntry { hour_start: 20, hour_end: 4, activity: NpcActivity::Sleeping, location: Vec3::new(-40.0, 0.0, -20.0) },
            ],
        });

        // Trader
        self.spawn_npc(NpcTemplate {
            name: "Wenona".to_string(),
            role: NpcRole::Trader,
            position: Vec3::new(25.0, 0.0, 5.0),
            village_id: Some(1),
            dialogue_id: None,
            schedule: vec![
                ScheduleEntry { hour_start: 7, hour_end: 18, activity: NpcActivity::Trading, location: Vec3::new(25.0, 0.0, 5.0) },
                ScheduleEntry { hour_start: 18, hour_end: 20, activity: NpcActivity::Eating, location: Vec3::new(25.0, 0.0, 15.0) },
                ScheduleEntry { hour_start: 20, hour_end: 7, activity: NpcActivity::Sleeping, location: Vec3::new(25.0, 0.0, 20.0) },
            ],
        });
    }

    /// Spawn a new NPC from template
    pub fn spawn_npc(&mut self, template: NpcTemplate) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let npc = NpcInstance {
            id,
            name: template.name,
            role: template.role,
            position: template.position,
            home_position: template.position,
            current_activity: NpcActivity::Resting,
            behavior_state: NpcBehaviorState::Idle,
            schedule: template.schedule,
            village_id: template.village_id,
            dialogue_id: template.dialogue_id,
            health: 100.0,
            max_health: 100.0,
            is_essential: matches!(template.role, NpcRole::Elder | NpcRole::Chief),
            mood: 50,
            alertness: 0,
            awareness: 0.0,
            emotional_state: EmotionalState::Neutral,
            target: None,
            velocity: Vec3::ZERO,
            look_direction: Vec3::Z,
        };

        self.npcs.insert(id, npc);
        id
    }

    /// Update all NPCs
    pub fn update(&mut self, dt: f32, player_pos: Vec3, player_faction_rep: &HashMap<Faction, i32>) {
        self.game_time += dt / 3600.0; // Convert seconds to hours
        if self.game_time >= 24.0 {
            self.game_time -= 24.0;
            // Daily restock
            self.trading.daily_restock();
            // Relationship decay
            self.relationships.decay_all(1.0);
        }

        let current_hour = self.game_time as u8;
        let npc_ids: Vec<u32> = self.npcs.keys().copied().collect();

        // Collect socializing NPC pairs for gossip propagation
        let socializing_pairs: Vec<(u32, u32)> = self.collect_socializing_pairs();

        for id in npc_ids {
            // Get relationship for this NPC (needs separate scope for borrow checker)
            let relationship = self.relationships.get(id).cloned();

            if let Some(npc) = self.npcs.get_mut(&id) {
                // Update activity based on schedule
                npc.update_activity(current_hour);

                if self.use_utility_ai {
                    // Use utility AI for decision making
                    let recently_attacked = npc.alertness > 50; // Use alertness as proxy
                    let ctx = build_context(
                        npc,
                        relationship.as_ref(),
                        player_pos,
                        self.game_time,
                        recently_attacked,
                    );

                    let (action, _score) = self.utility_evaluator.select_action(&ctx, npc.role);

                    // Apply action to NPC state
                    npc.apply_utility_action(action, player_pos, dt);
                } else {
                    // Legacy behavior-based update
                    let dist_to_player = npc.position.distance(player_pos);
                    npc.update_behavior(dt, player_pos, dist_to_player, relationship.as_ref());
                }

                // Apply movement
                npc.position += npc.velocity * dt;
            }
        }

        // Propagate gossip among socializing NPCs (once per update cycle)
        if !socializing_pairs.is_empty() {
            self.relationships.propagate_gossip(&socializing_pairs, self.game_time as f64);
        }
    }

    /// Collect pairs of NPCs who are currently socializing near each other
    fn collect_socializing_pairs(&self) -> Vec<(u32, u32)> {
        let socializing_npcs: Vec<(u32, Vec3)> = self.npcs.iter()
            .filter(|(_, npc)| matches!(npc.current_activity, NpcActivity::Socializing))
            .map(|(&id, npc)| (id, npc.position))
            .collect();

        let mut pairs = Vec::new();

        // Find NPCs within talking distance (10 units)
        for i in 0..socializing_npcs.len() {
            for j in (i + 1)..socializing_npcs.len() {
                let (id_a, pos_a) = socializing_npcs[i];
                let (id_b, pos_b) = socializing_npcs[j];

                if pos_a.distance(pos_b) < 10.0 {
                    pairs.push((id_a, id_b));
                    pairs.push((id_b, id_a)); // Bidirectional gossip
                }
            }
        }

        pairs
    }

    /// Get NPC by ID
    pub fn get(&self, id: u32) -> Option<&NpcInstance> {
        self.npcs.get(&id)
    }

    /// Get mutable NPC by ID
    pub fn get_mut(&mut self, id: u32) -> Option<&mut NpcInstance> {
        self.npcs.get_mut(&id)
    }

    /// Get NPCs near a position
    pub fn npcs_near(&self, pos: Vec3, radius: f32) -> Vec<&NpcInstance> {
        self.npcs.values()
            .filter(|npc| npc.position.distance(pos) <= radius)
            .collect()
    }

    /// Get NPCs in a village
    pub fn npcs_in_village(&self, village_id: u32) -> Vec<&NpcInstance> {
        self.npcs.values()
            .filter(|npc| npc.village_id == Some(village_id))
            .collect()
    }

    /// Start interaction with an NPC
    pub fn interact(&mut self, npc_id: u32, game_time: f64) -> Option<InteractionResult> {
        let npc = self.npcs.get(&npc_id)?;

        // Check if NPC allows interaction
        if !npc.current_activity.allows_interaction() {
            return Some(InteractionResult::Busy(npc.current_activity));
        }

        // Record interaction
        self.relationships.record_interaction(npc_id, true, "spoke with", game_time);

        // Return dialogue or trade options
        if let Some(dialogue_id) = &npc.dialogue_id {
            Some(InteractionResult::Dialogue(dialogue_id.clone()))
        } else {
            Some(InteractionResult::Generic)
        }
    }

    /// Check if can trade with NPC
    pub fn can_trade(&self, npc_id: u32, player_reputation: ReputationLevel) -> bool {
        self.npcs.get(&npc_id)
            .map(|npc| {
                matches!(npc.role, NpcRole::Trader | NpcRole::Hunter | NpcRole::Shaman | NpcRole::Elder)
                    && npc.current_activity.allows_interaction()
            })
            .unwrap_or(false)
            && self.trading.inventories.get(&npc_id)
                .map(|inv| player_reputation >= inv.required_reputation)
                .unwrap_or(false)
    }

    /// Alert NPCs in radius to a threat
    pub fn alert_npcs(&mut self, threat_pos: Vec3, radius: f32) {
        for npc in self.npcs.values_mut() {
            if npc.position.distance(threat_pos) <= radius {
                npc.alertness = 100;
                npc.behavior_state = NpcBehaviorState::Alert;
                npc.target = Some(threat_pos);
            }
        }
    }

    /// Get NPCs who witnessed an event
    pub fn witnesses(&self, event_pos: Vec3, radius: f32) -> Vec<u32> {
        self.npcs.values()
            .filter(|npc| {
                npc.position.distance(event_pos) <= radius
                    && !matches!(npc.current_activity, NpcActivity::Sleeping)
            })
            .map(|npc| npc.id)
            .collect()
    }
}

/// NPC instance runtime data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcInstance {
    pub id: u32,
    pub name: String,
    pub role: NpcRole,
    pub position: Vec3,
    pub home_position: Vec3,
    pub current_activity: NpcActivity,
    pub behavior_state: NpcBehaviorState,
    pub schedule: Vec<ScheduleEntry>,
    pub village_id: Option<u32>,
    pub dialogue_id: Option<String>,
    pub health: f32,
    pub max_health: f32,
    pub is_essential: bool,
    pub mood: i32,         // -100 to 100
    pub alertness: i32,    // 0 to 100 (legacy, use awareness for CharacterAgent)
    pub awareness: f32,    // 0.0 to 1.0 - unified awareness for CharacterAgent
    pub emotional_state: EmotionalState, // Emotional state for visual representation
    pub target: Option<Vec3>,
    pub velocity: Vec3,
    pub look_direction: Vec3,
}

impl NpcInstance {
    /// Update current activity based on schedule
    pub fn update_activity(&mut self, current_hour: u8) {
        for entry in &self.schedule {
            let in_range = if entry.hour_start <= entry.hour_end {
                current_hour >= entry.hour_start && current_hour < entry.hour_end
            } else {
                // Crosses midnight
                current_hour >= entry.hour_start || current_hour < entry.hour_end
            };

            if in_range {
                self.current_activity = entry.activity;
                self.target = Some(entry.location);
                return;
            }
        }
    }

    /// Update behavior based on environment
    pub fn update_behavior(&mut self, dt: f32, player_pos: Vec3, player_dist: f32, relationship: Option<&NpcRelationship>) {
        // Decay alertness
        self.alertness = (self.alertness - (10.0 * dt) as i32).max(0);

        // Update behavior state
        match self.behavior_state {
            NpcBehaviorState::Idle => {
                if let Some(target) = self.target {
                    let dist = self.position.distance(target);
                    if dist > 2.0 {
                        self.behavior_state = NpcBehaviorState::Walking;
                    }
                }

                // React to nearby player
                if player_dist < 5.0 {
                    self.look_direction = (player_pos - self.position).normalize();

                    if let Some(rel) = relationship {
                        if matches!(rel.relationship_type, super::relationships::RelationshipType::Friend |
                                                           super::relationships::RelationshipType::CloseFriend) {
                            self.behavior_state = NpcBehaviorState::Greeting;
                        }
                    }
                }
            }

            NpcBehaviorState::Walking => {
                if let Some(target) = self.target {
                    let to_target = target - self.position;
                    let dist = to_target.length();

                    if dist < 1.0 {
                        self.velocity = Vec3::ZERO;
                        self.behavior_state = NpcBehaviorState::Idle;
                    } else {
                        let direction = to_target / dist;
                        self.velocity = direction * 3.0; // Walking speed
                        self.look_direction = direction;
                    }
                }
            }

            NpcBehaviorState::Alert => {
                if self.alertness <= 0 {
                    self.behavior_state = NpcBehaviorState::Idle;
                } else if let Some(threat) = self.target {
                    self.look_direction = (threat - self.position).normalize();
                }
            }

            NpcBehaviorState::Fleeing => {
                if let Some(threat) = self.target {
                    let away = (self.position - threat).normalize();
                    self.velocity = away * 6.0; // Run speed
                    self.look_direction = away;

                    if self.position.distance(threat) > 50.0 {
                        self.behavior_state = NpcBehaviorState::Idle;
                        self.velocity = Vec3::ZERO;
                    }
                }
            }

            NpcBehaviorState::Greeting => {
                // Face player briefly then return to normal
                self.velocity = Vec3::ZERO;
                self.look_direction = (player_pos - self.position).normalize();

                if player_dist > 8.0 {
                    self.behavior_state = NpcBehaviorState::Idle;
                }
            }

            NpcBehaviorState::Working | NpcBehaviorState::Trading => {
                self.velocity = Vec3::ZERO;

                // Turn to face player if they're interacting
                if player_dist < 3.0 {
                    self.look_direction = (player_pos - self.position).normalize();
                }
            }

            NpcBehaviorState::Attacking => {
                // Combat behavior - move toward target
                if let Some(target) = self.target {
                    let to_target = target - self.position;
                    let dist = to_target.length();

                    if dist > 2.0 {
                        let direction = to_target / dist;
                        self.velocity = direction * 4.0; // Combat movement
                        self.look_direction = direction;
                    } else {
                        self.velocity = Vec3::ZERO;
                    }
                }
            }
        }
    }

    /// Take damage
    pub fn take_damage(&mut self, amount: f32) -> bool {
        self.health -= amount;
        self.alertness = 100;
        self.mood = (self.mood - 20).max(-100);

        if self.health <= 0.0 {
            if self.is_essential {
                self.health = 1.0; // Essential NPCs don't die
                self.behavior_state = NpcBehaviorState::Fleeing;
                false
            } else {
                true // NPC died
            }
        } else {
            self.behavior_state = NpcBehaviorState::Fleeing;
            false
        }
    }

    /// Apply a utility AI action to this NPC
    pub fn apply_utility_action(&mut self, action: NpcAction, player_pos: Vec3, dt: f32) {
        // Decay alertness
        self.alertness = (self.alertness - (10.0 * dt) as i32).max(0);

        // Update behavior state based on action
        self.behavior_state = action.to_behavior_state();

        // Configure movement and facing based on action
        match action {
            NpcAction::Idle => {
                self.velocity = Vec3::ZERO;
            }
            NpcAction::WalkToTarget => {
                if let Some(target) = self.target {
                    let to_target = target - self.position;
                    let dist = to_target.length();
                    if dist > 1.0 {
                        let direction = to_target / dist;
                        self.velocity = direction * 3.0; // Walking speed
                        self.look_direction = direction;
                    } else {
                        self.velocity = Vec3::ZERO;
                    }
                }
            }
            NpcAction::WorkAtLocation => {
                self.velocity = Vec3::ZERO;
            }
            NpcAction::GreetPlayer => {
                self.velocity = Vec3::ZERO;
                let to_player = (player_pos - self.position).normalize();
                if to_player.length_squared() > 0.001 {
                    self.look_direction = to_player;
                }
            }
            NpcAction::ApproachPlayer => {
                let to_player = player_pos - self.position;
                let dist = to_player.length();
                if dist > 3.0 {
                    let direction = to_player / dist;
                    self.velocity = direction * 2.5; // Slower approach
                    self.look_direction = direction;
                } else {
                    self.velocity = Vec3::ZERO;
                    self.look_direction = to_player.normalize();
                }
            }
            NpcAction::TradeWithPlayer => {
                self.velocity = Vec3::ZERO;
                let to_player = (player_pos - self.position).normalize();
                if to_player.length_squared() > 0.001 {
                    self.look_direction = to_player;
                }
            }
            NpcAction::FleeFromPlayer => {
                let away = (self.position - player_pos).normalize();
                self.velocity = away * 6.0; // Run speed
                self.look_direction = away;
                self.target = Some(self.home_position); // Flee toward home
            }
            NpcAction::AttackPlayer => {
                let to_player = player_pos - self.position;
                let dist = to_player.length();
                if dist > 2.0 {
                    let direction = to_player / dist;
                    self.velocity = direction * 4.0; // Combat movement
                    self.look_direction = direction;
                } else {
                    self.velocity = Vec3::ZERO;
                    self.look_direction = to_player.normalize();
                }
            }
            NpcAction::BecomeAlert => {
                self.velocity = Vec3::ZERO;
                self.alertness = 100;
                let to_player = (player_pos - self.position).normalize();
                if to_player.length_squared() > 0.001 {
                    self.look_direction = to_player;
                }
            }
            NpcAction::Investigate => {
                // Move toward last known disturbance (use player pos as proxy)
                let to_target = player_pos - self.position;
                let dist = to_target.length();
                if dist > 5.0 {
                    let direction = to_target / dist;
                    self.velocity = direction * 2.0; // Cautious movement
                    self.look_direction = direction;
                } else {
                    self.velocity = Vec3::ZERO;
                }
            }
            NpcAction::ReturnHome => {
                let to_home = self.home_position - self.position;
                let dist = to_home.length();
                if dist > 2.0 {
                    let direction = to_home / dist;
                    self.velocity = direction * 3.0;
                    self.look_direction = direction;
                } else {
                    self.velocity = Vec3::ZERO;
                }
            }
            NpcAction::Socialize | NpcAction::ShareGossip => {
                self.velocity = Vec3::ZERO;
                // Would look at other NPCs, but we don't have that info here
            }
        }
    }
}

/// NPC behavior states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NpcBehaviorState {
    #[default]
    Idle,
    Walking,
    Working,
    Trading,
    Alert,
    Fleeing,
    Greeting,
    Attacking,
}

/// NPC roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcRole {
    Elder,
    Chief,
    Shaman,
    Warrior,
    Hunter,
    Farmer,
    Craftsperson,
    Trader,
    Child,
    Villager,
}

impl NpcRole {
    /// Get faction for this role
    pub fn faction(&self) -> Faction {
        match self {
            Self::Elder | Self::Chief | Self::Villager | Self::Child | Self::Farmer => Faction::NativeCouncil,
            Self::Shaman => Faction::Shamans,
            Self::Warrior => Faction::Warriors,
            Self::Hunter => Faction::Hunters,
            Self::Craftsperson | Self::Trader => Faction::Traders,
        }
    }
}

/// Template for spawning NPCs
pub struct NpcTemplate {
    pub name: String,
    pub role: NpcRole,
    pub position: Vec3,
    pub village_id: Option<u32>,
    pub dialogue_id: Option<String>,
    pub schedule: Vec<ScheduleEntry>,
}

/// Result of interacting with an NPC
#[derive(Debug, Clone)]
pub enum InteractionResult {
    Dialogue(String),
    Trade(u32),
    Quest(String),
    Generic,
    Busy(NpcActivity),
    Hostile,
}

// ============================================================================
// Behavior State Conversion
// ============================================================================

impl NpcBehaviorState {
    /// Convert NpcBehaviorState to UnifiedBehaviorState
    pub fn to_unified(&self) -> UnifiedBehaviorState {
        match self {
            NpcBehaviorState::Idle => UnifiedBehaviorState::Idle,
            NpcBehaviorState::Walking => UnifiedBehaviorState::Traveling,
            NpcBehaviorState::Working => UnifiedBehaviorState::Working,
            NpcBehaviorState::Trading => UnifiedBehaviorState::Interacting,
            NpcBehaviorState::Alert => UnifiedBehaviorState::Alert,
            NpcBehaviorState::Fleeing => UnifiedBehaviorState::Fleeing,
            NpcBehaviorState::Greeting => UnifiedBehaviorState::Interacting,
            NpcBehaviorState::Attacking => UnifiedBehaviorState::Attacking,
        }
    }

    /// Create NpcBehaviorState from UnifiedBehaviorState
    pub fn from_unified(unified: UnifiedBehaviorState) -> Self {
        match unified {
            UnifiedBehaviorState::Idle => NpcBehaviorState::Idle,
            UnifiedBehaviorState::Patrolling => NpcBehaviorState::Walking,
            UnifiedBehaviorState::Traveling => NpcBehaviorState::Walking,
            UnifiedBehaviorState::Working => NpcBehaviorState::Working,
            UnifiedBehaviorState::Alert => NpcBehaviorState::Alert,
            UnifiedBehaviorState::Observing => NpcBehaviorState::Alert,
            UnifiedBehaviorState::Approaching => NpcBehaviorState::Walking,
            UnifiedBehaviorState::Pursuing => NpcBehaviorState::Walking,
            UnifiedBehaviorState::Fleeing => NpcBehaviorState::Fleeing,
            UnifiedBehaviorState::Attacking => NpcBehaviorState::Attacking,
            UnifiedBehaviorState::Recovering => NpcBehaviorState::Idle,
            UnifiedBehaviorState::Interacting => NpcBehaviorState::Trading,
            UnifiedBehaviorState::Resting => NpcBehaviorState::Idle,
            UnifiedBehaviorState::Dead => NpcBehaviorState::Idle, // NPCs handle death separately
        }
    }
}

// ============================================================================
// CharacterAgent Implementation for NpcInstance
// ============================================================================

impl CharacterAgent for NpcInstance {
    fn agent_id(&self) -> AgentId {
        AgentId::npc(self.id)
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    fn velocity(&self) -> Vec3 {
        self.velocity
    }

    fn set_velocity(&mut self, vel: Vec3) {
        self.velocity = vel;
    }

    fn base_speed(&self) -> f32 {
        // NPCs have role-based speeds
        match self.role {
            NpcRole::Warrior | NpcRole::Hunter => 5.0,
            NpcRole::Elder | NpcRole::Child => 2.5,
            _ => 3.5,
        }
    }

    fn awareness(&self) -> f32 {
        self.awareness
    }

    fn set_awareness(&mut self, level: f32) {
        self.awareness = level.clamp(0.0, 1.0);
        // Sync with legacy alertness
        self.alertness = (level * 100.0) as i32;
    }

    fn behavior_state(&self) -> UnifiedBehaviorState {
        self.behavior_state.to_unified()
    }

    fn set_behavior_state(&mut self, state: UnifiedBehaviorState) {
        self.behavior_state = NpcBehaviorState::from_unified(state);
    }

    fn emotional_state(&self) -> EmotionalState {
        self.emotional_state
    }

    fn set_emotional_state(&mut self, state: EmotionalState) {
        self.emotional_state = state;
    }

    fn detection_radius(&self) -> f32 {
        // Role-based detection ranges
        match self.role {
            NpcRole::Hunter | NpcRole::Warrior => 40.0,
            NpcRole::Shaman => 30.0,
            _ => 20.0,
        }
    }

    fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    fn look_direction(&self) -> Vec3 {
        self.look_direction
    }

    fn look_at(&mut self, target: Vec3) {
        let direction = (target - self.position).normalize();
        if direction.length_squared() > 0.001 {
            self.look_direction = direction;
        }
    }

    fn update(&mut self, ctx: &AgentContext, dt: f32) {
        // Sync awareness from legacy alertness if it changed externally
        let alertness_as_awareness = self.alertness as f32 / 100.0;
        if (self.awareness - alertness_as_awareness).abs() > 0.01 {
            self.awareness = alertness_as_awareness;
        }

        // Update emotional state based on context
        self.update_emotional_state(ctx);

        // Awareness decay over time
        if self.awareness > 0.0 {
            self.awareness = (self.awareness - 0.05 * dt).max(0.0);
        }

        // Distance-based awareness gain from player
        let player_dist = self.position.distance(ctx.player_pos);
        let detection = self.detection_radius();
        if player_dist < detection {
            let awareness_gain = crate::character_agent::calculate_awareness_gain(
                AgentKind::Npc,
                AgentKind::Player,
                player_dist,
                detection,
                dt,
            );
            self.awareness = (self.awareness + awareness_gain).min(1.0);
        }
    }

    fn orb_scale(&self) -> f32 {
        // Vary orb size by role importance
        match self.role {
            NpcRole::Elder | NpcRole::Chief => 1.3,
            NpcRole::Shaman => 1.2,
            NpcRole::Child => 0.7,
            _ => 1.0,
        }
    }
}

impl NpcInstance {
    /// Update emotional state based on context and relationships
    fn update_emotional_state(&mut self, ctx: &AgentContext) {
        let player_dist = self.position.distance(ctx.player_pos);
        let detection = self.detection_radius();

        // Determine emotional response
        self.emotional_state = if self.health < self.max_health * 0.3 {
            // Low health = fearful
            EmotionalState::Fearful
        } else if matches!(self.behavior_state, NpcBehaviorState::Fleeing) {
            EmotionalState::Fearful
        } else if matches!(self.behavior_state, NpcBehaviorState::Attacking) {
            EmotionalState::Hostile
        } else if matches!(self.behavior_state, NpcBehaviorState::Alert) {
            EmotionalState::Alert
        } else if matches!(self.behavior_state, NpcBehaviorState::Greeting) {
            EmotionalState::Friendly
        } else if matches!(self.behavior_state, NpcBehaviorState::Trading) {
            EmotionalState::Friendly
        } else if player_dist < detection * 0.5 && self.awareness > 0.3 {
            // Player nearby and aware
            EmotionalState::Curious
        } else if self.mood > 30 {
            EmotionalState::Calm
        } else if self.mood < -30 {
            EmotionalState::Alert
        } else {
            EmotionalState::Neutral
        };
    }
}
