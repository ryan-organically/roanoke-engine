//! Faction Integration Layer
//!
//! Bridges the faction system with villages, NPCs, quests, and events.
//! Provides pipelines for faction-based game logic.

use super::faction::{Faction, FactionCulture, Standing};
use super::faction_manager::{FactionAction, FactionManager, NpcDisposition, NpcRole as FactionNpcRole};
use super::faction_skills::FactionSkillId;
use crate::npc::npc_manager::NpcRole;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// VILLAGE FACTION AFFILIATION
// ============================================================================

/// Faction affiliation for a village
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VillageFaction {
    /// Primary faction this village belongs to
    pub primary_faction: Faction,
    /// Secondary faction influences (e.g., trade partners)
    pub influences: Vec<(Faction, f32)>, // (faction, influence 0.0-1.0)
    /// Village's internal standing with player (can differ from global faction)
    pub local_reputation: i32,
    /// Whether village acts independently from faction
    pub independent: bool,
    /// Special status (capital, trading post, sacred site, etc.)
    pub status: VillageStatus,
    /// Clan/tribe name within faction
    pub clan_name: Option<String>,
}

impl VillageFaction {
    pub fn new(faction: Faction) -> Self {
        Self {
            primary_faction: faction,
            influences: Vec::new(),
            local_reputation: 0,
            independent: false,
            status: VillageStatus::Normal,
            clan_name: None,
        }
    }

    /// Create a Powhatan village (default for Native settlements)
    pub fn powhatan(clan: &str) -> Self {
        Self {
            primary_faction: Faction::Powhatan,
            influences: vec![(Faction::Pamunkey, 0.3)], // Confederacy connection
            local_reputation: 0,
            independent: false,
            status: VillageStatus::Normal,
            clan_name: Some(clan.to_string()),
        }
    }

    /// Create the Croatoan main village
    pub fn croatoan() -> Self {
        Self {
            primary_faction: Faction::Powhatan,
            influences: vec![
                (Faction::Pamunkey, 0.5),
                (Faction::English, 0.1), // Some contact with colonists
            ],
            local_reputation: 50, // Starts somewhat friendly
            independent: false,
            status: VillageStatus::Capital,
            clan_name: Some("Croatoan".to_string()),
        }
    }

    /// Get effective standing with player
    pub fn effective_standing(&self, global_standing: Standing) -> Standing {
        let local_standing = Standing::from_reputation(self.local_reputation);

        // Blend local and global, with local having slight priority
        let global_val = global_standing.value() as f32;
        let local_val = local_standing.value() as f32;
        let blended = (global_val * 0.4 + local_val * 0.6).round() as i8;

        match blended.clamp(-3, 3) {
            -3 => Standing::War,
            -2 => Standing::Hostile,
            -1 => Standing::Suspicious,
            0 => Standing::Neutral,
            1 => Standing::Friendly,
            2 => Standing::Allied,
            3 => Standing::BloodBond,
            _ => Standing::Neutral,
        }
    }

    /// Modify local reputation
    pub fn modify_local_reputation(&mut self, delta: i32) {
        self.local_reputation = (self.local_reputation + delta).clamp(-2000, 2000);
    }
}

/// Special status for villages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VillageStatus {
    #[default]
    Normal,
    Capital,       // Faction capital, extra important
    TradingPost,   // Major trade hub
    SacredSite,    // Religious significance
    Fortress,      // Military stronghold
    Outpost,       // Small frontier settlement
    Abandoned,     // No longer active
}

// ============================================================================
// NPC FACTION DATA
// ============================================================================

/// Extended NPC data with faction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcFactionData {
    /// NPC ID reference
    pub npc_id: u32,
    /// Primary faction affiliation
    pub faction: Faction,
    /// Role within faction hierarchy
    pub faction_role: FactionNpcRole,
    /// Personal reputation modifier (individual relationship)
    pub personal_rep_modifier: i32,
    /// Special faction titles held
    pub titles: Vec<String>,
    /// Can this NPC grant faction quests
    pub quest_giver: bool,
    /// Can this NPC teach faction skills
    pub skill_trainer: bool,
    /// Specific skills this NPC can teach
    pub teachable_skills: Vec<FactionSkillId>,
    /// Loyalty to faction (affects betrayal chance, etc.)
    pub loyalty: f32,
}

impl NpcFactionData {
    pub fn from_role(npc_id: u32, role: NpcRole, village_faction: Faction) -> Self {
        let (faction_role, quest_giver, skill_trainer) = match role {
            NpcRole::Elder => (FactionNpcRole::Elder, true, true),
            NpcRole::Chief => (FactionNpcRole::Chief, true, true),
            NpcRole::Shaman => (FactionNpcRole::Shaman, true, true),
            NpcRole::Warrior => (FactionNpcRole::Warrior, true, false),
            NpcRole::Hunter => (FactionNpcRole::Farmer, true, false), // Hunters can give hunting quests
            NpcRole::Farmer => (FactionNpcRole::Farmer, false, false),
            NpcRole::Craftsperson => (FactionNpcRole::Trader, false, true),
            NpcRole::Trader => (FactionNpcRole::Trader, true, false),
            NpcRole::Child => (FactionNpcRole::Villager, false, false),
            NpcRole::Villager => (FactionNpcRole::Villager, false, false),
        };

        Self {
            npc_id,
            faction: village_faction,
            faction_role,
            personal_rep_modifier: 0,
            titles: Vec::new(),
            quest_giver,
            skill_trainer,
            teachable_skills: Vec::new(),
            loyalty: 0.8, // Default high loyalty
        }
    }

    /// Get disposition toward player based on faction standing
    pub fn get_disposition(&self, faction_manager: &FactionManager) -> NpcDisposition {
        let standing = faction_manager.get_standing(self.faction);

        // Apply personal modifier
        let modified_standing = if self.personal_rep_modifier != 0 {
            let base_rep = faction_manager.get_reputation(self.faction);
            let modified_rep = base_rep + self.personal_rep_modifier;
            Standing::from_reputation(modified_rep)
        } else {
            standing
        };

        NpcDisposition::from_standing(modified_standing, self.faction_role)
    }

    /// Modify personal reputation with this NPC
    pub fn modify_personal_rep(&mut self, delta: i32) {
        self.personal_rep_modifier = (self.personal_rep_modifier + delta).clamp(-500, 500);
    }
}

// ============================================================================
// FACTION EVENTS PIPELINE
// ============================================================================

/// Events that affect faction standing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactionEvent {
    // ===== REPUTATION CHANGES =====
    /// Player completed a quest for faction
    QuestCompleted {
        faction: Faction,
        quest_id: String,
        reputation_gain: i32,
    },
    /// Player failed/abandoned a quest
    QuestFailed {
        faction: Faction,
        quest_id: String,
        reputation_loss: i32,
    },
    /// Player traded with faction
    TradeCompleted {
        faction: Faction,
        value: i32,
        fair_trade: bool,
    },
    /// Player gifted item to faction member
    GiftGiven {
        faction: Faction,
        npc_id: u32,
        item_value: i32,
    },
    /// Player attacked faction member
    MemberAttacked {
        faction: Faction,
        npc_id: u32,
        damage: f32,
        witnesses: Vec<u32>,
    },
    /// Player killed faction member
    MemberKilled {
        faction: Faction,
        npc_id: u32,
        witnesses: Vec<u32>,
    },
    /// Player defended faction settlement
    SettlementDefended {
        faction: Faction,
        village_id: u32,
        enemies_killed: u32,
    },
    /// Player desecrated sacred site
    SacredSiteDesecrated {
        faction: Faction,
        site_id: u32,
    },
    /// Player discovered something valuable for faction
    DiscoveryMade {
        faction: Faction,
        discovery_type: DiscoveryType,
    },

    // ===== STANDING CHANGES =====
    /// Standing changed to new level
    StandingChanged {
        faction: Faction,
        old_standing: Standing,
        new_standing: Standing,
    },
    /// Reached Blood Bond status
    BloodBondFormed {
        faction: Faction,
    },
    /// War declared
    WarDeclared {
        faction: Faction,
        reason: String,
    },

    // ===== SKILL/ABILITY UNLOCKS =====
    /// Faction skill unlocked
    SkillUnlocked {
        faction: Faction,
        skill_id: FactionSkillId,
    },
    /// Faction ability available
    AbilityUnlocked {
        faction: Faction,
        ability_id: String,
    },
    /// Faction weapon available
    WeaponUnlocked {
        faction: Faction,
        weapon_id: String,
    },

    // ===== POLITICAL EVENTS =====
    /// Faction relationship changed
    FactionRelationshipChanged {
        faction_a: Faction,
        faction_b: Faction,
        old_standing: Standing,
        new_standing: Standing,
    },
    /// Player chose primary faction
    PrimaryFactionChosen {
        faction: Faction,
        conflicts: Vec<Faction>,
    },
    /// Player betrayed faction
    FactionBetrayed {
        faction: Faction,
        severity: BetrayalSeverity,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryType {
    Fossil,
    SacredSite,
    Resource,
    Territory,
    Secret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BetrayalSeverity {
    Minor,  // Shared information
    Major,  // Active sabotage
    Total,  // Joined enemy faction
}

// ============================================================================
// FACTION NOTIFICATION SYSTEM
// ============================================================================

/// Notifications to display to player about faction changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionNotification {
    pub faction: Faction,
    pub notification_type: FactionNotificationType,
    pub message: String,
    pub importance: NotificationImportance,
    pub timestamp: f64,
    /// Whether notification has been shown to player
    pub displayed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactionNotificationType {
    ReputationGain,
    ReputationLoss,
    StandingUp,
    StandingDown,
    SkillUnlock,
    WeaponUnlock,
    AbilityUnlock,
    QuestAvailable,
    WarDeclared,
    PeaceDeclared,
    TitleGranted,
    AccessGranted,
    AccessRevoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NotificationImportance {
    Low,
    Medium,
    High,
    Critical,
}

// ============================================================================
// FACTION EVENT PROCESSOR
// ============================================================================

/// Processes faction events and updates game state
#[derive(Debug, Default, Clone)]
pub struct FactionEventProcessor {
    /// Queue of pending events
    pending_events: Vec<FactionEvent>,
    /// Generated notifications
    notifications: Vec<FactionNotification>,
    /// Event history for debugging/quests
    event_history: Vec<(f64, FactionEvent)>,
}

impl FactionEventProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a faction event for processing
    pub fn queue_event(&mut self, event: FactionEvent) {
        self.pending_events.push(event);
    }

    /// Process all queued events
    pub fn process_events(&mut self, faction_manager: &mut FactionManager, game_time: f64) {
        let events: Vec<FactionEvent> = self.pending_events.drain(..).collect();

        for event in events {
            self.process_single_event(&event, faction_manager, game_time);
            self.event_history.push((game_time, event));
        }

        // Trim history to last 100 events
        if self.event_history.len() > 100 {
            self.event_history.drain(0..self.event_history.len() - 100);
        }
    }

    fn process_single_event(
        &mut self,
        event: &FactionEvent,
        faction_manager: &mut FactionManager,
        game_time: f64,
    ) {
        match event {
            FactionEvent::QuestCompleted {
                faction,
                quest_id,
                reputation_gain,
            } => {
                let old_standing = faction_manager.get_standing(*faction);
                faction_manager.modify_reputation(
                    *faction,
                    *reputation_gain,
                    &format!("Completed quest: {}", quest_id),
                    game_time,
                );
                let new_standing = faction_manager.get_standing(*faction);

                self.add_notification(
                    *faction,
                    FactionNotificationType::ReputationGain,
                    format!("+{} reputation with {}", reputation_gain, faction.display_name()),
                    NotificationImportance::Medium,
                    game_time,
                );

                self.check_standing_change(*faction, old_standing, new_standing, game_time);
            }

            FactionEvent::QuestFailed {
                faction,
                reputation_loss,
                ..
            } => {
                let old_standing = faction_manager.get_standing(*faction);
                faction_manager.modify_reputation(
                    *faction,
                    -*reputation_loss,
                    "Failed quest",
                    game_time,
                );
                let new_standing = faction_manager.get_standing(*faction);

                self.add_notification(
                    *faction,
                    FactionNotificationType::ReputationLoss,
                    format!("-{} reputation with {}", reputation_loss, faction.display_name()),
                    NotificationImportance::Medium,
                    game_time,
                );

                self.check_standing_change(*faction, old_standing, new_standing, game_time);
            }

            FactionEvent::TradeCompleted {
                faction,
                value,
                fair_trade,
            } => {
                let rep_change = if *fair_trade {
                    (*value / 100).max(1).min(10)
                } else {
                    -((*value / 50).max(5).min(25))
                };

                faction_manager.modify_reputation(
                    *faction,
                    rep_change,
                    if *fair_trade { "Fair trade" } else { "Unfair trade" },
                    game_time,
                );
            }

            FactionEvent::GiftGiven {
                faction,
                item_value,
                ..
            } => {
                let rep_gain = (*item_value / 10).max(5).min(50);
                faction_manager.modify_reputation(*faction, rep_gain, "Gift given", game_time);

                self.add_notification(
                    *faction,
                    FactionNotificationType::ReputationGain,
                    format!("{} appreciates your gift", faction.display_name()),
                    NotificationImportance::Low,
                    game_time,
                );
            }

            FactionEvent::MemberAttacked {
                faction,
                witnesses,
                ..
            } => {
                let base_loss = -100;
                let witness_multiplier = 1.0 + (witnesses.len() as f32 * 0.2);
                let total_loss = (base_loss as f32 * witness_multiplier) as i32;

                let old_standing = faction_manager.get_standing(*faction);
                faction_manager.modify_reputation(
                    *faction,
                    total_loss,
                    "Attacked faction member",
                    game_time,
                );
                let new_standing = faction_manager.get_standing(*faction);

                self.add_notification(
                    *faction,
                    FactionNotificationType::ReputationLoss,
                    format!("{} is angered by your attack!", faction.display_name()),
                    NotificationImportance::High,
                    game_time,
                );

                self.check_standing_change(*faction, old_standing, new_standing, game_time);

                // Propagate to allied factions
                for allied in faction_manager.get_allied_factions() {
                    if allied != *faction {
                        faction_manager.modify_reputation(
                            allied,
                            total_loss / 3,
                            "Attack on ally",
                            game_time,
                        );
                    }
                }
            }

            FactionEvent::MemberKilled {
                faction,
                witnesses,
                ..
            } => {
                let base_loss = -200;
                let witness_multiplier = 1.0 + (witnesses.len() as f32 * 0.3);
                let total_loss = (base_loss as f32 * witness_multiplier) as i32;

                let old_standing = faction_manager.get_standing(*faction);
                faction_manager.modify_reputation(
                    *faction,
                    total_loss,
                    "Killed faction member",
                    game_time,
                );
                let new_standing = faction_manager.get_standing(*faction);

                self.add_notification(
                    *faction,
                    FactionNotificationType::ReputationLoss,
                    format!("{} demands blood for blood!", faction.display_name()),
                    NotificationImportance::Critical,
                    game_time,
                );

                self.check_standing_change(*faction, old_standing, new_standing, game_time);
            }

            FactionEvent::SettlementDefended {
                faction,
                enemies_killed,
                ..
            } => {
                let rep_gain = 100 + (*enemies_killed as i32 * 10);

                let old_standing = faction_manager.get_standing(*faction);
                faction_manager.modify_reputation(
                    *faction,
                    rep_gain,
                    "Defended settlement",
                    game_time,
                );
                let new_standing = faction_manager.get_standing(*faction);

                self.add_notification(
                    *faction,
                    FactionNotificationType::ReputationGain,
                    format!("{} honors your bravery!", faction.display_name()),
                    NotificationImportance::High,
                    game_time,
                );

                self.check_standing_change(*faction, old_standing, new_standing, game_time);
            }

            FactionEvent::SacredSiteDesecrated { faction, .. } => {
                let old_standing = faction_manager.get_standing(*faction);
                faction_manager.modify_reputation(
                    *faction,
                    -300,
                    "Desecrated sacred site",
                    game_time,
                );
                let new_standing = faction_manager.get_standing(*faction);

                self.add_notification(
                    *faction,
                    FactionNotificationType::ReputationLoss,
                    format!("You have desecrated what {} holds sacred!", faction.display_name()),
                    NotificationImportance::Critical,
                    game_time,
                );

                self.check_standing_change(*faction, old_standing, new_standing, game_time);

                // Also affects related factions
                if faction.culture() == FactionCulture::NativeAmerican {
                    for related in Faction::all_playable() {
                        if related.culture() == FactionCulture::NativeAmerican && *related != *faction {
                            faction_manager.modify_reputation(
                                *related,
                                -50,
                                "Desecrated Native sacred site",
                                game_time,
                            );
                        }
                    }
                }
            }

            FactionEvent::SkillUnlocked { faction, skill_id } => {
                self.add_notification(
                    *faction,
                    FactionNotificationType::SkillUnlock,
                    format!("Unlocked: {}", skill_id.display_name()),
                    NotificationImportance::High,
                    game_time,
                );
            }

            FactionEvent::StandingChanged {
                faction,
                old_standing,
                new_standing,
            } => {
                let notification_type = if new_standing > old_standing {
                    FactionNotificationType::StandingUp
                } else {
                    FactionNotificationType::StandingDown
                };

                self.add_notification(
                    *faction,
                    notification_type,
                    format!(
                        "Standing with {} changed: {:?} -> {:?}",
                        faction.display_name(),
                        old_standing,
                        new_standing
                    ),
                    NotificationImportance::High,
                    game_time,
                );
            }

            FactionEvent::WarDeclared { faction, reason } => {
                self.add_notification(
                    *faction,
                    FactionNotificationType::WarDeclared,
                    format!("{} has declared war! Reason: {}", faction.display_name(), reason),
                    NotificationImportance::Critical,
                    game_time,
                );
            }

            _ => {} // Handle other events as needed
        }
    }

    fn check_standing_change(
        &mut self,
        faction: Faction,
        old: Standing,
        new: Standing,
        game_time: f64,
    ) {
        if old != new {
            self.queue_event(FactionEvent::StandingChanged {
                faction,
                old_standing: old,
                new_standing: new,
            });

            // Check for war declaration
            if new == Standing::War && old != Standing::War {
                self.queue_event(FactionEvent::WarDeclared {
                    faction,
                    reason: "Reputation too low".to_string(),
                });
            }

            // Check for blood bond
            if new == Standing::BloodBond && old != Standing::BloodBond {
                self.queue_event(FactionEvent::BloodBondFormed { faction });
            }
        }
    }

    fn add_notification(
        &mut self,
        faction: Faction,
        notification_type: FactionNotificationType,
        message: String,
        importance: NotificationImportance,
        game_time: f64,
    ) {
        self.notifications.push(FactionNotification {
            faction,
            notification_type,
            message,
            importance,
            timestamp: game_time,
            displayed: false,
        });
    }

    /// Get pending notifications
    pub fn get_notifications(&mut self) -> Vec<FactionNotification> {
        let notifications: Vec<_> = self
            .notifications
            .iter()
            .filter(|n| !n.displayed)
            .cloned()
            .collect();

        // Mark as displayed
        for n in &mut self.notifications {
            n.displayed = true;
        }

        notifications
    }

    /// Clear old notifications
    pub fn clear_old_notifications(&mut self, current_time: f64, max_age: f64) {
        self.notifications
            .retain(|n| current_time - n.timestamp < max_age || !n.displayed);
    }
}

// ============================================================================
// FACTION SAVE/LOAD DATA
// ============================================================================

/// Complete faction state for save/load
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionSaveData {
    /// Version for migration support
    pub version: u32,
    /// Timestamp of save
    pub timestamp: f64,
    /// Core faction manager state
    pub manager: FactionManagerSaveData,
    /// Village faction assignments
    pub village_factions: HashMap<u32, VillageFaction>,
    /// NPC faction data
    pub npc_faction_data: HashMap<u32, NpcFactionData>,
    /// Event history (last N events)
    pub recent_events: Vec<(f64, FactionEvent)>,
    /// Pending notifications
    pub pending_notifications: Vec<FactionNotification>,
}

/// Serializable faction manager data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionManagerSaveData {
    /// Player reputation with each faction
    pub reputations: HashMap<Faction, FactionReputationSaveData>,
    /// Player's unlocked skills per faction
    pub unlocked_skills: HashMap<Faction, Vec<FactionSkillId>>,
    /// Skill points per faction
    pub skill_points: HashMap<Faction, u32>,
    /// Player's primary faction
    pub primary_faction: Option<Faction>,
    /// Modified inter-faction relationships (only non-default)
    pub modified_relationships: Vec<ModifiedRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionReputationSaveData {
    pub reputation: i32,
    pub max_reached: i32,
    pub min_reached: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedRelationship {
    pub faction_a: Faction,
    pub faction_b: Faction,
    pub current_standing: Standing,
}

impl FactionSaveData {
    pub const CURRENT_VERSION: u32 = 1;

    /// Create save data from current game state
    pub fn from_state(
        faction_manager: &FactionManager,
        village_factions: &HashMap<u32, VillageFaction>,
        npc_faction_data: &HashMap<u32, NpcFactionData>,
        event_processor: &FactionEventProcessor,
        game_time: f64,
    ) -> Self {
        // Extract reputation data
        let reputations: HashMap<Faction, FactionReputationSaveData> = faction_manager
            .reputations
            .iter()
            .map(|(f, r)| {
                (
                    *f,
                    FactionReputationSaveData {
                        reputation: r.reputation,
                        max_reached: r.max_reputation,
                        min_reached: r.min_reputation,
                    },
                )
            })
            .collect();

        // Extract skills
        let unlocked_skills: HashMap<Faction, Vec<FactionSkillId>> = faction_manager
            .skills
            .unlocked
            .iter()
            .map(|(f, skills)| (*f, skills.iter().copied().collect()))
            .collect();

        Self {
            version: Self::CURRENT_VERSION,
            timestamp: game_time,
            manager: FactionManagerSaveData {
                reputations,
                unlocked_skills,
                skill_points: faction_manager.skills.skill_points.clone(),
                primary_faction: faction_manager.primary_faction,
                modified_relationships: Vec::new(), // TODO: Track modified relationships
            },
            village_factions: village_factions.clone(),
            npc_faction_data: npc_faction_data.clone(),
            recent_events: event_processor.event_history.clone(),
            pending_notifications: event_processor.notifications.clone(),
        }
    }

    /// Restore game state from save data
    pub fn restore(&self, faction_manager: &mut FactionManager) {
        // Restore reputations
        for (faction, data) in &self.manager.reputations {
            if let Some(rep) = faction_manager.reputations.get_mut(faction) {
                rep.reputation = data.reputation;
                rep.max_reputation = data.max_reached;
                rep.min_reputation = data.min_reached;
                rep.standing = Standing::from_reputation(data.reputation);
            }
        }

        // Restore skills
        for (faction, skills) in &self.manager.unlocked_skills {
            faction_manager
                .skills
                .unlocked
                .insert(*faction, skills.iter().copied().collect());
        }

        faction_manager.skills.skill_points = self.manager.skill_points.clone();
        faction_manager.primary_faction = self.manager.primary_faction;
    }
}

// ============================================================================
// FACTION UI DATA STRUCTURES
// ============================================================================

/// Data for displaying faction in UI
#[derive(Debug, Clone)]
pub struct FactionUIData {
    pub faction: Faction,
    pub display_name: String,
    pub motto: String,
    pub culture: FactionCulture,
    pub standing: Standing,
    pub reputation: i32,
    pub reputation_progress: f32, // 0.0-1.0 to next standing
    pub is_primary: bool,
    pub skills_unlocked: usize,
    pub skills_available: usize,
    pub skill_points: u32,
    pub relationships: Vec<FactionRelationshipUI>,
}

#[derive(Debug, Clone)]
pub struct FactionRelationshipUI {
    pub other_faction: Faction,
    pub standing: Standing,
    pub description: String,
}

/// Generate UI data for all factions
pub fn generate_faction_ui_data(faction_manager: &FactionManager) -> Vec<FactionUIData> {
    Faction::all_playable()
        .iter()
        .map(|&faction| {
            let standing = faction_manager.get_standing(faction);
            let reputation = faction_manager.get_reputation(faction);

            // Calculate progress to next standing
            let reputation_progress = faction_manager
                .reputations
                .get(&faction)
                .map(|r| r.progress_to_next())
                .unwrap_or(0.0);

            // Count skills
            let skills_unlocked = faction_manager.skills.skill_count(faction);
            let skills_available = faction_manager.get_available_skills(faction).len();
            let skill_points = faction_manager
                .skills
                .skill_points
                .get(&faction)
                .copied()
                .unwrap_or(0);

            // Get relationships with other factions
            let relationships: Vec<FactionRelationshipUI> = Faction::all_playable()
                .iter()
                .filter(|&&other| other != faction)
                .map(|&other| {
                    let rel_standing = faction_manager.get_faction_relationship(faction, other);
                    FactionRelationshipUI {
                        other_faction: other,
                        standing: rel_standing,
                        description: format!(
                            "{} is {:?} with {}",
                            faction.display_name(),
                            rel_standing,
                            other.display_name()
                        ),
                    }
                })
                .collect();

            FactionUIData {
                faction,
                display_name: faction.display_name().to_string(),
                motto: faction.motto().to_string(),
                culture: faction.culture(),
                standing,
                reputation,
                reputation_progress,
                is_primary: faction_manager.primary_faction == Some(faction),
                skills_unlocked,
                skills_available,
                skill_points,
                relationships,
            }
        })
        .collect()
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Map NpcRole from npc_manager to FactionNpcRole
pub fn map_npc_role(role: NpcRole) -> FactionNpcRole {
    match role {
        NpcRole::Elder => FactionNpcRole::Elder,
        NpcRole::Chief => FactionNpcRole::Chief,
        NpcRole::Shaman => FactionNpcRole::Shaman,
        NpcRole::Warrior => FactionNpcRole::Warrior,
        NpcRole::Hunter | NpcRole::Farmer => FactionNpcRole::Farmer,
        NpcRole::Craftsperson | NpcRole::Trader => FactionNpcRole::Trader,
        NpcRole::Child | NpcRole::Villager => FactionNpcRole::Villager,
    }
}

/// Determine village faction based on location and world context
pub fn determine_village_faction(village_name: &str, position: Vec3, seed: u32) -> Faction {
    // Special case for known villages
    match village_name.to_lowercase().as_str() {
        "croatoan" => return Faction::Powhatan,
        "pamunkey" | "werowocomoco" => return Faction::Pamunkey,
        "tuscarora" => return Faction::Tuscarora,
        "cherokee" => return Faction::Cherokee,
        "catawba" => return Faction::Catawba,
        _ => {}
    }

    // Determine by region/position
    // Use position to create regional faction zones
    let region_x = (position.x / 500.0).floor() as i32;
    let region_z = (position.z / 500.0).floor() as i32;
    let region_hash = ((region_x.wrapping_mul(73856093)) ^ (region_z.wrapping_mul(19349663))) as u32;
    let combined = region_hash.wrapping_add(seed);

    // Distribute factions by region
    match combined % 5 {
        0 => Faction::Powhatan,
        1 => Faction::Tuscarora,
        2 => Faction::Cherokee,
        3 => Faction::Catawba,
        _ => Faction::Pamunkey,
    }
}

/// Get reputation change for hunting an animal near a village
pub fn get_hunting_reputation_change(
    animal_type: &str,
    near_village: bool,
    village_faction: Option<Faction>,
) -> Vec<(Faction, i32, &'static str)> {
    let mut changes = Vec::new();

    match animal_type.to_lowercase().as_str() {
        // Predators - positive if near village
        "wolf" | "cougar" | "bear" | "alligator" => {
            if near_village {
                if let Some(faction) = village_faction {
                    changes.push((faction, 15, "Killed predator near village"));
                }
            }
            changes.push((Faction::Wildlife, -20, "Killed predator"));
        }
        // Game animals - neutral unless overhunting
        "deer" | "boar" | "turkey" | "rabbit" => {
            changes.push((Faction::Wildlife, -5, "Hunted game animal"));
        }
        _ => {}
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_village_faction_creation() {
        let village = VillageFaction::croatoan();
        assert_eq!(village.primary_faction, Faction::Powhatan);
        assert_eq!(village.status, VillageStatus::Capital);
        assert_eq!(village.local_reputation, 50);
    }

    #[test]
    fn test_npc_faction_data_from_role() {
        let data = NpcFactionData::from_role(1, NpcRole::Shaman, Faction::Cherokee);
        assert_eq!(data.faction, Faction::Cherokee);
        assert!(data.quest_giver);
        assert!(data.skill_trainer);
    }

    #[test]
    fn test_event_processor() {
        let mut processor = FactionEventProcessor::new();
        let mut manager = FactionManager::new();

        processor.queue_event(FactionEvent::QuestCompleted {
            faction: Faction::Powhatan,
            quest_id: "test_quest".to_string(),
            reputation_gain: 100,
        });

        processor.process_events(&mut manager, 0.0);

        assert!(manager.get_reputation(Faction::Powhatan) > 0);
        assert!(!processor.notifications.is_empty());
    }

    #[test]
    fn test_faction_ui_data() {
        let manager = FactionManager::new();
        let ui_data = generate_faction_ui_data(&manager);

        assert_eq!(ui_data.len(), 9); // 9 playable factions
        assert!(ui_data.iter().all(|d| d.standing == Standing::Neutral));
    }
}
