//! NPC Manager
//!
//! Central management of all NPC instances and their behaviors.

use super::relationships::{NpcRelationship, RelationshipManager, ScheduleEntry, NpcActivity};
use super::dialogue::DialogueManager;
use super::trading::TradingSystem;
use crate::progression::reputation::{Faction, ReputationLevel};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Central NPC manager
#[derive(Debug)]
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

        for id in npc_ids {
            if let Some(npc) = self.npcs.get_mut(&id) {
                // Update activity based on schedule
                npc.update_activity(current_hour);

                // Update behavior based on player proximity and relationship
                let dist_to_player = npc.position.distance(player_pos);
                let relationship = self.relationships.get(id);

                npc.update_behavior(dt, player_pos, dist_to_player, relationship);

                // Apply movement
                npc.position += npc.velocity * dt;
            }
        }
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
    pub alertness: i32,    // 0 to 100
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
