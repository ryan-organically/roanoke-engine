//! World Event System
//!
//! Manages dynamic world events, triggers, and state changes.

use super::reputation::Faction;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Event manager for world state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventManager {
    /// Active events in the world
    pub active_events: HashMap<String, WorldEvent>,
    /// Event history (for narrative tracking)
    pub event_history: VecDeque<EventRecord>,
    /// Scheduled future events
    pub scheduled: Vec<ScheduledEvent>,
    /// World flags (persistent state)
    pub world_flags: HashMap<String, WorldFlag>,
    /// Current world state phase
    pub world_phase: WorldPhase,
    /// In-game time
    pub game_time: f64,
}

impl EventManager {
    pub fn new() -> Self {
        let mut manager = Self::default();
        manager.initialize_events();
        manager
    }

    /// Initialize starting events and world state
    fn initialize_events(&mut self) {
        // Starting world phase
        self.world_phase = WorldPhase::Arrival;

        // Initial world flags
        self.world_flags.insert("colony_discovered".to_string(), WorldFlag::Bool(false));
        self.world_flags.insert("native_contact".to_string(), WorldFlag::Bool(false));
        self.world_flags.insert("first_hunt".to_string(), WorldFlag::Bool(false));

        // Schedule initial events
        self.schedule_event(ScheduledEvent {
            event_id: "tutorial_hunt".to_string(),
            trigger_time: 0.5, // 30 minutes in-game
            event: WorldEvent {
                id: "tutorial_hunt".to_string(),
                name: "A Desperate Hunt".to_string(),
                description: "You're getting hungry. Find something to eat.".to_string(),
                event_type: EventType::Tutorial,
                location: None,
                duration: None,
                effects: vec![
                    EventEffect::SpawnAnimals {
                        species: "Wild Boar".to_string(),
                        count: 2,
                        location: Vec3::new(50.0, 0.0, 50.0),
                        radius: 30.0,
                    },
                ],
                completion_trigger: Some(EventTrigger::Kill {
                    species: "Wild Boar".to_string(),
                    count: 1,
                }),
                rewards: vec![],
            },
        });

        // Random event pool - scheduled periodically
        self.schedule_random_event(1.0); // First random event after 1 day
    }

    /// Update events based on game time
    pub fn update(&mut self, dt: f64, player_pos: Vec3) -> Vec<EventNotification> {
        self.game_time += dt;
        let mut notifications = Vec::new();

        // Check scheduled events
        let ready_events: Vec<_> = self.scheduled
            .iter()
            .filter(|e| self.game_time >= e.trigger_time)
            .cloned()
            .collect();

        for scheduled in ready_events {
            self.scheduled.retain(|e| e.event_id != scheduled.event_id);
            self.start_event(scheduled.event);
            notifications.push(EventNotification::EventStarted(scheduled.event_id));
        }

        // Update active events
        let mut completed_events = Vec::new();
        for (id, event) in &mut self.active_events {
            // Check duration expiry
            if let Some(duration) = event.duration {
                if self.game_time >= duration {
                    completed_events.push(id.clone());
                }
            }
        }

        for id in completed_events {
            self.complete_event(&id);
            notifications.push(EventNotification::EventEnded(id));
        }

        notifications
    }

    /// Start a new event
    pub fn start_event(&mut self, event: WorldEvent) {
        let id = event.id.clone();

        // Apply immediate effects
        for effect in &event.effects {
            // Effects are processed by the game systems
        }

        self.active_events.insert(id.clone(), event);

        // Record in history
        self.event_history.push_back(EventRecord {
            event_id: id,
            start_time: self.game_time,
            end_time: None,
            outcome: EventOutcome::InProgress,
        });

        // Keep history manageable
        if self.event_history.len() > 100 {
            self.event_history.pop_front();
        }
    }

    /// Complete an event
    pub fn complete_event(&mut self, event_id: &str) {
        if let Some(event) = self.active_events.remove(event_id) {
            // Update history
            if let Some(record) = self.event_history.iter_mut().rev().find(|r| r.event_id == event_id) {
                record.end_time = Some(self.game_time);
                record.outcome = EventOutcome::Completed;
            }

            // Apply rewards
            for reward in &event.rewards {
                // Rewards are processed by the game systems
            }
        }
    }

    /// Schedule a future event
    pub fn schedule_event(&mut self, scheduled: ScheduledEvent) {
        self.scheduled.push(scheduled);
        self.scheduled.sort_by(|a, b| a.trigger_time.partial_cmp(&b.trigger_time).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Schedule a random event
    fn schedule_random_event(&mut self, days_from_now: f64) {
        let event = self.generate_random_event();
        self.schedule_event(ScheduledEvent {
            event_id: event.id.clone(),
            trigger_time: self.game_time + days_from_now * 24.0,
            event,
        });
    }

    /// Generate a random world event
    fn generate_random_event(&self) -> WorldEvent {
        // Use world state to determine event pool
        let event_type = match self.world_phase {
            WorldPhase::Arrival => EventType::Environmental,
            WorldPhase::Settlement => EventType::NpcEvent,
            WorldPhase::Conflict => EventType::Crisis,
            WorldPhase::Resolution => EventType::Celebration,
        };

        // Simple random event for now
        WorldEvent {
            id: format!("random_{}", self.game_time as u64),
            name: "Strange Weather".to_string(),
            description: "A sudden fog rolls in from the coast.".to_string(),
            event_type: EventType::Environmental,
            location: None,
            duration: Some(self.game_time + 2.0), // 2 hours
            effects: vec![
                EventEffect::WeatherChange {
                    weather: "fog".to_string(),
                    intensity: 0.8,
                },
            ],
            completion_trigger: None,
            rewards: vec![],
        }
    }

    /// Check and trigger events based on player actions
    pub fn on_player_action(&mut self, action: &PlayerAction) -> Vec<EventNotification> {
        let mut notifications = Vec::new();

        // Check completion triggers for active events
        let mut completed = Vec::new();
        for (id, event) in &self.active_events {
            if let Some(trigger) = &event.completion_trigger {
                if self.check_trigger(trigger, action) {
                    completed.push(id.clone());
                }
            }
        }

        for id in completed {
            self.complete_event(&id);
            notifications.push(EventNotification::EventCompleted(id));
        }

        // Check for new event triggers
        match action {
            PlayerAction::EnterLocation(loc_id) => {
                if let Some(flag) = self.world_flags.get_mut("areas_explored") {
                    if let WorldFlag::Counter(count) = flag {
                        *count += 1;
                        if *count == 5 {
                            // Trigger exploration milestone event
                            notifications.push(EventNotification::MilestoneReached("Explorer".to_string()));
                        }
                    }
                }
            }
            PlayerAction::KillAnimal(species) => {
                if !self.get_flag_bool("first_hunt") {
                    self.set_flag("first_hunt", WorldFlag::Bool(true));
                    notifications.push(EventNotification::FlagSet("first_hunt".to_string()));
                }
            }
            PlayerAction::MeetNpc(npc_id) => {
                if *npc_id > 0 && !self.get_flag_bool("native_contact") {
                    self.set_flag("native_contact", WorldFlag::Bool(true));
                    self.world_phase = WorldPhase::Settlement;
                    notifications.push(EventNotification::PhaseChange(WorldPhase::Settlement));
                }
            }
            _ => {}
        }

        notifications
    }

    /// Check if a trigger condition is met
    fn check_trigger(&self, trigger: &EventTrigger, action: &PlayerAction) -> bool {
        match (trigger, action) {
            (EventTrigger::Kill { species, count }, PlayerAction::KillAnimal(killed_species)) => {
                // Simplified check - actual implementation would track counts
                species == killed_species
            }
            (EventTrigger::ReachLocation(loc_id), PlayerAction::EnterLocation(entered)) => {
                loc_id == entered
            }
            (EventTrigger::TimeElapsed(duration), _) => {
                // Checked in update()
                false
            }
            _ => false,
        }
    }

    /// Get a boolean flag value
    pub fn get_flag_bool(&self, key: &str) -> bool {
        match self.world_flags.get(key) {
            Some(WorldFlag::Bool(v)) => *v,
            _ => false,
        }
    }

    /// Get a counter flag value
    pub fn get_flag_counter(&self, key: &str) -> u32 {
        match self.world_flags.get(key) {
            Some(WorldFlag::Counter(v)) => *v,
            _ => 0,
        }
    }

    /// Set a world flag
    pub fn set_flag(&mut self, key: &str, value: WorldFlag) {
        self.world_flags.insert(key.to_string(), value);
    }

    /// Get active events in a location
    pub fn events_at_location(&self, location_id: u64) -> Vec<&WorldEvent> {
        self.active_events.values()
            .filter(|e| e.location.map(|l| l == location_id).unwrap_or(false))
            .collect()
    }
}

/// World event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub event_type: EventType,
    pub location: Option<u64>,
    pub duration: Option<f64>, // Absolute end time
    pub effects: Vec<EventEffect>,
    pub completion_trigger: Option<EventTrigger>,
    pub rewards: Vec<EventReward>,
}

/// Event type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    Tutorial,       // Guided introductory events
    Environmental,  // Weather, natural disasters
    NpcEvent,       // Village happenings
    AnimalMigration,// Wildlife changes
    Crisis,         // Dangerous situations
    Discovery,      // Hidden areas revealed
    Celebration,    // Positive community events
}

/// Event effects that change the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventEffect {
    SpawnAnimals {
        species: String,
        count: u32,
        location: Vec3,
        radius: f32,
    },
    DespawnAnimals {
        species: String,
        location: Vec3,
        radius: f32,
    },
    WeatherChange {
        weather: String,
        intensity: f32,
    },
    NpcMoodChange {
        npc_id: u32,
        mood_delta: i32,
    },
    VillageAlert {
        village_id: u32,
        alert_level: u8,
    },
    SpawnItem {
        item: String,
        location: Vec3,
    },
    UnlockLocation {
        location_id: u64,
    },
    ModifyReputation {
        faction: Faction,
        delta: i32,
    },
    StartQuest {
        quest_id: String,
    },
    Dialogue {
        npc_id: u32,
        dialogue_id: String,
    },
}

/// Trigger conditions for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventTrigger {
    Kill {
        species: String,
        count: u32,
    },
    ReachLocation(u64),
    TimeElapsed(f64),
    FlagSet(String),
    ReputationReached {
        faction: Faction,
        level: i32,
    },
    QuestComplete(String),
    ItemObtained(String),
}

/// Event rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventReward {
    Item { item: String, count: u32 },
    Experience(u32),
    Reputation { faction: Faction, delta: i32 },
    SkillPoints { tree: String, points: u32 },
    UnlockSkill(String),
}

/// Scheduled future event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub event_id: String,
    pub trigger_time: f64,
    pub event: WorldEvent,
}

/// Event history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub event_id: String,
    pub start_time: f64,
    pub end_time: Option<f64>,
    pub outcome: EventOutcome,
}

/// Event outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventOutcome {
    InProgress,
    Completed,
    Failed,
    Abandoned,
}

/// World phase (campaign progression)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorldPhase {
    #[default]
    Arrival,    // Player just arrived
    Settlement, // Building trust with natives
    Conflict,   // Main conflict emerges
    Resolution, // Climactic events
}

/// World flag types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldFlag {
    Bool(bool),
    Counter(u32),
    Text(String),
    Number(f32),
}

/// Player action for event triggers
#[derive(Debug, Clone)]
pub enum PlayerAction {
    EnterLocation(u64),
    ExitLocation(u64),
    KillAnimal(String),
    GatherItem(String),
    CraftItem(String),
    MeetNpc(u32),
    CompleteQuest(String),
    DiscoverSecret(String),
    TradeWith(u32),
    DamageDealt(f32),
    DamageTaken(f32),
}

/// Event notification for UI/audio
#[derive(Debug, Clone)]
pub enum EventNotification {
    EventStarted(String),
    EventEnded(String),
    EventCompleted(String),
    EventFailed(String),
    FlagSet(String),
    PhaseChange(WorldPhase),
    MilestoneReached(String),
}

/// Microcosm/biome region data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Microcosm {
    pub id: u64,
    pub name: String,
    pub biome_type: BiomeType,
    pub center: Vec3,
    pub radius: f32,
    /// Unique properties
    pub properties: MicrocosmProperties,
    /// Events specific to this region
    pub local_events: Vec<String>,
    /// Species that spawn here
    pub fauna: Vec<(String, f32)>, // (species, spawn_weight)
    /// Ambient sounds
    pub ambient_sounds: Vec<String>,
}

/// Biome types for microcosms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiomeType {
    CoastalMarsh,
    DenseForest,
    MountainPeak,
    RiverValley,
    Swampland,
    OpenMeadow,
    RockyOutcrop,
    SacredGrove,
    AncientRuins,
    CaveSystem,
}

impl BiomeType {
    /// Get default fauna for this biome
    pub fn default_fauna(&self) -> Vec<(String, f32)> {
        match self {
            Self::CoastalMarsh => vec![
                ("American Alligator".to_string(), 0.4),
                ("Cottonmouth".to_string(), 0.3),
                ("Wild Boar".to_string(), 0.2),
            ],
            Self::DenseForest => vec![
                ("Black Bear".to_string(), 0.2),
                ("Eastern Cougar".to_string(), 0.1),
                ("Gray Wolf".to_string(), 0.25),
                ("Wild Boar".to_string(), 0.3),
                ("Copperhead".to_string(), 0.15),
            ],
            Self::MountainPeak => vec![
                ("Eastern Cougar".to_string(), 0.4),
                ("Bobcat".to_string(), 0.3),
                ("Timber Rattlesnake".to_string(), 0.3),
            ],
            Self::RiverValley => vec![
                ("Black Bear".to_string(), 0.25),
                ("Wild Boar".to_string(), 0.35),
                ("Cottonmouth".to_string(), 0.2),
                ("American Alligator".to_string(), 0.2),
            ],
            Self::Swampland => vec![
                ("American Alligator".to_string(), 0.5),
                ("Cottonmouth".to_string(), 0.3),
                ("Red Wolf".to_string(), 0.1),
                ("Wild Boar".to_string(), 0.1),
            ],
            Self::OpenMeadow => vec![
                ("Wild Boar".to_string(), 0.4),
                ("Gray Wolf".to_string(), 0.3),
                ("Copperhead".to_string(), 0.2),
                ("Bobcat".to_string(), 0.1),
            ],
            Self::RockyOutcrop => vec![
                ("Timber Rattlesnake".to_string(), 0.5),
                ("Bobcat".to_string(), 0.3),
                ("Eastern Cougar".to_string(), 0.2),
            ],
            Self::SacredGrove => vec![
                ("Black Bear".to_string(), 0.4),
                ("Gray Wolf".to_string(), 0.3),
                ("Eastern Cougar".to_string(), 0.2),
                ("Timber Rattlesnake".to_string(), 0.1),
            ],
            Self::AncientRuins => vec![
                ("Copperhead".to_string(), 0.4),
                ("Timber Rattlesnake".to_string(), 0.3),
                ("Bobcat".to_string(), 0.2),
                ("Wild Boar".to_string(), 0.1),
            ],
            Self::CaveSystem => vec![
                ("Black Bear".to_string(), 0.5),
                ("Copperhead".to_string(), 0.3),
                ("Timber Rattlesnake".to_string(), 0.2),
            ],
        }
    }

    /// Get danger level (1-10)
    pub fn danger_level(&self) -> u8 {
        match self {
            Self::OpenMeadow => 3,
            Self::RiverValley => 4,
            Self::DenseForest => 5,
            Self::CoastalMarsh => 6,
            Self::RockyOutcrop => 6,
            Self::Swampland => 7,
            Self::AncientRuins => 7,
            Self::SacredGrove => 7,
            Self::MountainPeak => 8,
            Self::CaveSystem => 9,
        }
    }

    /// Get fossil spawn chance
    pub fn fossil_chance(&self) -> f32 {
        match self {
            Self::CoastalMarsh => 0.3,
            Self::RiverValley => 0.2,
            Self::CaveSystem => 0.5,
            Self::AncientRuins => 0.4,
            Self::RockyOutcrop => 0.25,
            _ => 0.1,
        }
    }
}

/// Special properties for a microcosm
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MicrocosmProperties {
    /// Whether this is a sacred/special location
    pub is_sacred: bool,
    /// Whether it's a village territory
    pub village_territory: Option<u32>,
    /// Visibility modifier (fog, etc.)
    pub visibility_modifier: f32,
    /// Animal behavior modifier
    pub aggression_modifier: f32,
    /// Special loot table
    pub loot_modifier: f32,
    /// Weather effects
    pub weather_bias: Option<String>,
    /// Time of day effects
    pub time_sensitive: bool,
    /// Legendary spawn possible
    pub legendary_spawn: Option<String>,
}
