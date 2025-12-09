//! Game State Integration
//!
//! Central game state that integrates all progression, NPC, and world systems.

use crate::progression::{PlayerProgression, QuestManager, EventManager};
use crate::progression::events::{WorldPhase, PlayerAction, EventNotification, Microcosm, BiomeType};
use crate::progression::reputation::{Faction, ReputationLevel};
use crate::npc::{NpcManager, InteractionSystem};
use crate::animals::{PlayerWildlifeReputation, LegendaryAnimal, AnimalSpecies};
use crate::animals::player_tracking::create_legendary_animals;
use crate::economy::{PlayerEconomy, EconomyManager, LootNotification, CombatLootResult};
use glam::Vec3;
use std::collections::HashMap;

/// Central game progression container
pub struct GameProgression {
    // Player systems
    pub player_progression: PlayerProgression,
    pub wildlife_reputation: PlayerWildlifeReputation,

    // Economy system (dual currency, inventory, loot)
    pub player_economy: PlayerEconomy,
    pub economy_manager: EconomyManager,

    // World systems
    pub quest_manager: QuestManager,
    pub event_manager: EventManager,
    pub npc_manager: NpcManager,

    // NPC Interaction & Dialogue system
    pub interaction_system: InteractionSystem,

    // World data
    pub microcosms: Vec<Microcosm>,
    pub legendary_animals: Vec<LegendaryAnimal>,

    // Time tracking
    pub game_time: f64,  // In-game hours since start
    pub days_passed: u32,
    pub current_hour: u8,

    // Session stats
    pub session_start: std::time::Instant,

    // Notification queue for UI
    pub pending_notifications: Vec<GameNotification>,

    // Pending loot notifications (separate from general notifications for UI)
    pub pending_loot: Vec<LootNotification>,
}

impl GameProgression {
    pub fn new() -> Self {
        let mut state = Self {
            player_progression: PlayerProgression::new(),
            wildlife_reputation: PlayerWildlifeReputation::new(),
            player_economy: PlayerEconomy::new(),
            economy_manager: EconomyManager::new(),
            quest_manager: QuestManager::new(),
            event_manager: EventManager::new(),
            npc_manager: NpcManager::new(),
            interaction_system: InteractionSystem::new(),
            microcosms: create_default_microcosms(),
            legendary_animals: create_legendary_animals(),
            game_time: 8.0, // Start at 8 AM
            days_passed: 0,
            current_hour: 8,
            session_start: std::time::Instant::now(),
            pending_notifications: Vec::new(),
            pending_loot: Vec::new(),
        };

        // Initialize starting reputation
        state.player_progression.modify_reputation(Faction::NativeCouncil, 0);
        state.player_progression.modify_reputation(Faction::Hunters, 0);
        state.player_progression.modify_reputation(Faction::Shamans, 0);

        state
    }

    /// Main update tick
    pub fn update(&mut self, dt: f32, player_pos: Vec3, player_velocity: Vec3, faction_rep: &HashMap<Faction, i32>) {
        // Update game time
        let hours_passed = dt / 3600.0;
        self.game_time += hours_passed as f64;

        let new_hour = (self.game_time % 24.0) as u8;
        if new_hour != self.current_hour {
            self.current_hour = new_hour;
            if new_hour == 0 {
                self.days_passed += 1;
                self.player_progression.days_survived = self.days_passed;
            }
        }

        self.player_progression.in_game_hours = self.game_time as f32;

        // Update wildlife reputation
        self.wildlife_reputation.update(dt, player_pos, self.game_time);

        // Update event manager
        let event_notifications = self.event_manager.update(hours_passed as f64, player_pos);
        for notification in event_notifications {
            self.handle_event_notification(notification);
        }

        // Update NPC manager
        self.npc_manager.update(dt, player_pos, faction_rep);

        // Check microcosm entry
        self.check_microcosm_entry(player_pos);

        // Check legendary spawns
        self.check_legendary_spawns(player_pos);
    }

    /// Handle player killing an animal
    pub fn on_animal_killed(
        &mut self,
        species: AnimalSpecies,
        position: Vec3,
        was_stealth: bool,
        was_perfect: bool,
        was_critical: bool,
        weapon_used: &str,
        kill_time_seconds: f32,
        player_health: f32,
    ) {
        let species_name = species.name();

        // Update wildlife reputation
        self.wildlife_reputation.record_kill(species, position, was_stealth, self.game_time);

        // Update player progression
        self.player_progression.record_kill(species_name, was_stealth, was_perfect);

        // Update quests
        let completed = self.quest_manager.on_kill(species_name, was_stealth, false);
        for obj_id in completed {
            self.pending_notifications.push(GameNotification::ObjectiveComplete(obj_id));
        }

        // Check for skill unlocks
        if was_stealth && !self.player_progression.hunting.shadow_hunter {
            if self.player_progression.stealth_kills >= 5 {
                self.player_progression.hunting.shadow_hunter = true;
                self.pending_notifications.push(GameNotification::SkillUnlocked("Shadow Hunter".to_string()));
            }
        }

        // Notify nearby NPCs (reputation impact)
        let witnesses = self.npc_manager.witnesses(position, 100.0);
        if !witnesses.is_empty() {
            // Hunting near village can affect reputation
            self.player_progression.modify_reputation(Faction::Hunters, 5);
        }

        // ========== ECONOMY: Process loot drops ==========
        let hunting_level = self.player_progression.hunting.effective_level();
        let luck = self.player_progression.hunting.calculate_luck_bonus();

        let loot_notifications = self.economy_manager.process_animal_kill(
            &mut self.player_economy,
            species,
            position,
            was_stealth,
            was_perfect,
            was_critical,
            weapon_used,
            kill_time_seconds,
            player_health,
            hunting_level,
            luck,
        );

        // Add loot notifications
        for notification in &loot_notifications {
            // Push to loot queue for UI
            self.pending_loot.push(notification.clone());

            // Check for rare drop notifications
            if notification.item.rarity >= crate::economy::Rarity::Rare {
                self.pending_notifications.push(GameNotification::RareLootDrop(
                    notification.item.full_name(),
                    notification.item.rarity,
                ));
            }

            // First discovery of template
            if notification.is_new_template {
                self.pending_notifications.push(GameNotification::NewItemDiscovered(
                    notification.item.template_id.clone(),
                ));
            }
        }

        // Track legendary kills
        for legendary in &mut self.legendary_animals {
            if legendary.species == species && legendary.is_spawned && !legendary.is_killed {
                if position.distance(legendary.position) < 50.0 {
                    legendary.is_killed = true;
                    self.pending_notifications.push(GameNotification::LegendaryKilled(legendary.name.clone()));

                    // Award legendary rewards
                    self.player_progression.stats.legendary_kills.push(legendary.name.clone());
                    self.player_progression.hunting.points += 500;

                    // Bonus wampum for legendary kill
                    self.player_economy.wallet.add_wampum(5000);
                    self.player_economy.wallet.add_tobacco(100);
                }
            }
        }
    }

    /// Simplified version for backward compatibility
    pub fn on_animal_killed_simple(&mut self, species: AnimalSpecies, position: Vec3, was_stealth: bool, was_perfect: bool) {
        self.on_animal_killed(
            species,
            position,
            was_stealth,
            was_perfect,
            false,  // was_critical
            "unknown",
            5.0,    // default kill time
            100.0,  // full health
        );
    }

    /// Handle player taking damage from animal
    pub fn on_damage_taken(&mut self, species: AnimalSpecies, amount: f32) {
        self.wildlife_reputation.record_damage_taken(species, amount, self.game_time);
        self.player_progression.stats.total_damage_taken += amount;
    }

    /// Handle player gathering an item
    pub fn on_item_gathered(&mut self, item: &str) {
        let completed = self.quest_manager.on_gather(item);
        for obj_id in completed {
            self.pending_notifications.push(GameNotification::ObjectiveComplete(obj_id));
        }

        // Track fossil discoveries
        if item.contains("fossil") || item.contains("tooth") || item.contains("bone") {
            let was_perfect = self.player_progression.extraction_success_rate() > 0.7;
            self.player_progression.record_fossil_extraction(item, was_perfect, 3);
        }

        // Event action
        self.event_manager.on_player_action(&PlayerAction::GatherItem(item.to_string()));
    }

    /// Handle player discovering a location
    pub fn on_location_discovered(&mut self, location_id: u64, position: Vec3) {
        use crate::progression::player_state::LocationId;

        if self.player_progression.discover_location(LocationId(location_id)) {
            self.pending_notifications.push(GameNotification::LocationDiscovered(location_id));

            // Update quests
            let completed = self.quest_manager.on_location_reached(location_id, position);
            for obj_id in completed {
                self.pending_notifications.push(GameNotification::ObjectiveComplete(obj_id));
            }

            // Event action
            self.event_manager.on_player_action(&PlayerAction::EnterLocation(location_id));

            // Count discovered villages
            if location_id < 100 { // Village IDs are low
                self.player_progression.stats.villages_discovered += 1;
            }
        }
    }

    /// Handle NPC interaction
    pub fn on_npc_interaction(&mut self, npc_id: u32) {
        if let Some(result) = self.npc_manager.interact(npc_id, self.game_time) {
            match result {
                crate::npc::npc_manager::InteractionResult::Dialogue(dialogue_id) => {
                    self.npc_manager.dialogue.start_dialogue(&dialogue_id);
                }
                crate::npc::npc_manager::InteractionResult::Trade(_) => {
                    // Trading handled separately
                }
                _ => {}
            }
        }

        // Event action
        self.event_manager.on_player_action(&PlayerAction::MeetNpc(npc_id));
    }

    /// Complete a quest
    pub fn complete_quest(&mut self, quest_id: &str) -> bool {
        if let Some(rewards) = self.quest_manager.complete_quest(quest_id) {
            // Apply rewards
            for (faction, delta) in &rewards.reputation {
                self.player_progression.modify_reputation(*faction, *delta);
            }

            if let Some((tree, points)) = &rewards.skill_points {
                match tree.as_str() {
                    "hunting" => self.player_progression.hunting.points += points,
                    "archaeology" => self.player_progression.archaeology.points += points,
                    _ => {}
                }
            }

            self.pending_notifications.push(GameNotification::QuestComplete(quest_id.to_string()));

            // Event action
            self.event_manager.on_player_action(&PlayerAction::CompleteQuest(quest_id.to_string()));

            return true;
        }
        false
    }

    /// Handle event notification
    fn handle_event_notification(&mut self, notification: EventNotification) {
        match notification {
            EventNotification::EventStarted(id) => {
                self.pending_notifications.push(GameNotification::EventStarted(id));
            }
            EventNotification::EventCompleted(id) => {
                self.pending_notifications.push(GameNotification::EventComplete(id));
            }
            EventNotification::PhaseChange(phase) => {
                self.pending_notifications.push(GameNotification::WorldPhaseChanged(phase));
            }
            EventNotification::MilestoneReached(name) => {
                self.pending_notifications.push(GameNotification::MilestoneReached(name));
            }
            _ => {}
        }
    }

    /// Check if player entered a microcosm
    fn check_microcosm_entry(&mut self, player_pos: Vec3) {
        for microcosm in &self.microcosms {
            if player_pos.distance(microcosm.center) < microcosm.radius {
                // Player is in this microcosm - could trigger events
                if microcosm.properties.is_sacred {
                    // Special behavior in sacred areas
                }
            }
        }
    }

    /// Check if legendary animals should spawn
    fn check_legendary_spawns(&mut self, player_pos: Vec3) {
        for legendary in &mut self.legendary_animals {
            if legendary.is_killed || legendary.is_spawned {
                continue;
            }

            // Check if player is near spawn location and has required skills
            let dist = player_pos.distance(legendary.position);
            if dist < 200.0 {
                // Check requirements (e.g., Legendary Hunter skill)
                if self.player_progression.hunting.legendary_hunter {
                    legendary.is_spawned = true;
                    self.pending_notifications.push(GameNotification::LegendarySpawned(legendary.name.clone()));
                }
            }
        }
    }

    /// Get current world phase
    pub fn world_phase(&self) -> WorldPhase {
        self.event_manager.world_phase
    }

    /// Get reputation level with faction
    pub fn reputation_level(&self, faction: &Faction) -> ReputationLevel {
        ReputationLevel::from_value(self.player_progression.get_reputation(faction))
    }

    /// Drain pending notifications
    pub fn drain_notifications(&mut self) -> Vec<GameNotification> {
        std::mem::take(&mut self.pending_notifications)
    }

    /// Get save data
    pub fn save_data(&self) -> ProgressionSaveData {
        ProgressionSaveData {
            player_progression: self.player_progression.clone(),
            wildlife_reputation: self.wildlife_reputation.clone(),
            player_economy: self.player_economy.clone(),
            game_time: self.game_time,
            days_passed: self.days_passed,
            world_phase: self.event_manager.world_phase,
            completed_quests: self.quest_manager.completed.clone(),
            active_quests: self.quest_manager.active.clone(),
        }
    }

    /// Load save data
    pub fn load_save(&mut self, data: ProgressionSaveData) {
        self.player_progression = data.player_progression;
        self.wildlife_reputation = data.wildlife_reputation;
        self.player_economy = data.player_economy;
        self.player_economy.on_load(); // Rebuild inventory indices
        self.game_time = data.game_time;
        self.days_passed = data.days_passed;
        self.event_manager.world_phase = data.world_phase;
        self.quest_manager.completed = data.completed_quests;
        self.quest_manager.active = data.active_quests;
    }
}

/// Create default microcosms for the world
fn create_default_microcosms() -> Vec<Microcosm> {
    use crate::progression::events::MicrocosmProperties;

    vec![
        Microcosm {
            id: 1,
            name: "Croatoan Village".to_string(),
            biome_type: BiomeType::OpenMeadow,
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 150.0,
            properties: MicrocosmProperties {
                is_sacred: false,
                village_territory: Some(1),
                visibility_modifier: 1.0,
                aggression_modifier: 0.5, // Animals less aggressive near village
                loot_modifier: 1.0,
                weather_bias: None,
                time_sensitive: false,
                legendary_spawn: None,
            },
            local_events: vec![],
            fauna: BiomeType::OpenMeadow.default_fauna(),
            ambient_sounds: vec!["village_ambient".to_string()],
        },
        Microcosm {
            id: 2,
            name: "The Dark Forest".to_string(),
            biome_type: BiomeType::DenseForest,
            center: Vec3::new(-300.0, 50.0, -300.0),
            radius: 300.0,
            properties: MicrocosmProperties {
                is_sacred: false,
                village_territory: None,
                visibility_modifier: 0.6,
                aggression_modifier: 1.3,
                loot_modifier: 1.2,
                weather_bias: Some("fog".to_string()),
                time_sensitive: true,
                legendary_spawn: Some("fenrir".to_string()),
            },
            local_events: vec!["wolf_hunt".to_string()],
            fauna: BiomeType::DenseForest.default_fauna(),
            ambient_sounds: vec!["forest_dense".to_string(), "wolf_howl".to_string()],
        },
        Microcosm {
            id: 3,
            name: "The Sacred Grove".to_string(),
            biome_type: BiomeType::SacredGrove,
            center: Vec3::new(200.0, 30.0, 200.0),
            radius: 100.0,
            properties: MicrocosmProperties {
                is_sacred: true,
                village_territory: Some(1),
                visibility_modifier: 1.2,
                aggression_modifier: 0.3,
                loot_modifier: 1.5,
                weather_bias: None,
                time_sensitive: true,
                legendary_spawn: None,
            },
            local_events: vec!["shaman_vision".to_string()],
            fauna: BiomeType::SacredGrove.default_fauna(),
            ambient_sounds: vec!["sacred_ambient".to_string()],
        },
        Microcosm {
            id: 4,
            name: "Serpent's Swamp".to_string(),
            biome_type: BiomeType::Swampland,
            center: Vec3::new(200.0, 0.0, -400.0),
            radius: 250.0,
            properties: MicrocosmProperties {
                is_sacred: false,
                village_territory: None,
                visibility_modifier: 0.5,
                aggression_modifier: 1.5,
                loot_modifier: 1.3,
                weather_bias: Some("mist".to_string()),
                time_sensitive: false,
                legendary_spawn: Some("swamp_king".to_string()),
            },
            local_events: vec!["swamp_fog".to_string()],
            fauna: BiomeType::Swampland.default_fauna(),
            ambient_sounds: vec!["swamp_ambient".to_string(), "gator_growl".to_string()],
        },
        Microcosm {
            id: 5,
            name: "Mountain's Peak".to_string(),
            biome_type: BiomeType::MountainPeak,
            center: Vec3::new(500.0, 150.0, 500.0),
            radius: 200.0,
            properties: MicrocosmProperties {
                is_sacred: false,
                village_territory: None,
                visibility_modifier: 1.5,
                aggression_modifier: 1.2,
                loot_modifier: 1.4,
                weather_bias: Some("wind".to_string()),
                time_sensitive: true,
                legendary_spawn: Some("ghost_cougar".to_string()),
            },
            local_events: vec!["mountain_storm".to_string()],
            fauna: BiomeType::MountainPeak.default_fauna(),
            ambient_sounds: vec!["mountain_wind".to_string()],
        },
        Microcosm {
            id: 6,
            name: "The Ancient Cave".to_string(),
            biome_type: BiomeType::CaveSystem,
            center: Vec3::new(-500.0, 80.0, 300.0),
            radius: 100.0,
            properties: MicrocosmProperties {
                is_sacred: false,
                village_territory: None,
                visibility_modifier: 0.2,
                aggression_modifier: 1.8,
                loot_modifier: 2.0,
                weather_bias: None,
                time_sensitive: false,
                legendary_spawn: Some("old_silverback".to_string()),
            },
            local_events: vec!["cave_echo".to_string()],
            fauna: BiomeType::CaveSystem.default_fauna(),
            ambient_sounds: vec!["cave_ambient".to_string(), "bear_growl".to_string()],
        },
        Microcosm {
            id: 7,
            name: "Coastal Marshes".to_string(),
            biome_type: BiomeType::CoastalMarsh,
            center: Vec3::new(400.0, 5.0, -200.0),
            radius: 200.0,
            properties: MicrocosmProperties {
                is_sacred: false,
                village_territory: None,
                visibility_modifier: 0.7,
                aggression_modifier: 1.4,
                loot_modifier: 1.2,
                weather_bias: Some("rain".to_string()),
                time_sensitive: false,
                legendary_spawn: None,
            },
            local_events: vec![],
            fauna: BiomeType::CoastalMarsh.default_fauna(),
            ambient_sounds: vec!["marsh_ambient".to_string()],
        },
        Microcosm {
            id: 8,
            name: "Ancient Ruins".to_string(),
            biome_type: BiomeType::AncientRuins,
            center: Vec3::new(-200.0, 40.0, 500.0),
            radius: 120.0,
            properties: MicrocosmProperties {
                is_sacred: true,
                village_territory: None,
                visibility_modifier: 0.8,
                aggression_modifier: 0.8,
                loot_modifier: 2.5, // High fossil chance
                weather_bias: None,
                time_sensitive: true,
                legendary_spawn: None,
            },
            local_events: vec!["ruins_discovery".to_string()],
            fauna: BiomeType::AncientRuins.default_fauna(),
            ambient_sounds: vec!["ruins_ambient".to_string()],
        },
    ]
}

/// Game notifications for UI display
#[derive(Debug, Clone)]
pub enum GameNotification {
    // Quest notifications
    QuestStarted(String),
    QuestComplete(String),
    QuestFailed(String),
    ObjectiveComplete(String),

    // Skill notifications
    SkillUnlocked(String),
    SkillPointsGained(String, u32),

    // Reputation notifications
    ReputationGained(Faction, i32),
    ReputationLost(Faction, i32),
    ReputationLevelUp(Faction),

    // Discovery notifications
    LocationDiscovered(u64),
    SpeciesDiscovered(String),
    FossilDiscovered(String),
    NewItemDiscovered(String),

    // Event notifications
    EventStarted(String),
    EventComplete(String),
    WorldPhaseChanged(WorldPhase),
    MilestoneReached(String),

    // Legendary notifications
    LegendarySpawned(String),
    LegendaryKilled(String),

    // Combat notifications
    CriticalHit,
    StealthKill,
    PerfectKill,

    // Economy notifications
    RareLootDrop(String, crate::economy::Rarity),
    WampumEarned(u64),
    TobaccoEarned(u64),
    InventoryFull,
    PityTriggered(crate::economy::Rarity),
}

/// Save game data for progression system
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgressionSaveData {
    pub player_progression: PlayerProgression,
    pub wildlife_reputation: PlayerWildlifeReputation,
    pub player_economy: PlayerEconomy,
    pub game_time: f64,
    pub days_passed: u32,
    pub world_phase: WorldPhase,
    pub completed_quests: Vec<String>,
    pub active_quests: Vec<String>,
}

// Economy helper methods
impl GameProgression {
    /// Get player's current wampum balance
    pub fn wampum(&self) -> u64 {
        self.player_economy.wallet.wampum
    }

    /// Get player's current tobacco balance
    pub fn tobacco(&self) -> u64 {
        self.player_economy.wallet.tobacco
    }

    /// Get player's inventory value
    pub fn inventory_value(&self) -> u64 {
        self.player_economy.inventory.total_value()
    }

    /// Get number of free inventory slots
    pub fn free_inventory_slots(&self) -> usize {
        self.player_economy.inventory.free_slots()
    }

    /// Drain pending loot notifications
    pub fn drain_loot_notifications(&mut self) -> Vec<LootNotification> {
        std::mem::take(&mut self.pending_loot)
    }
}
