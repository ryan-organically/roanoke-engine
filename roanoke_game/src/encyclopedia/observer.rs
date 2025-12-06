//! Real-time observation system for wildlife and flora study
//!
//! Handles the mechanics of studying creatures in the field:
//! - Line of sight checks
//! - Distance-based observation quality
//! - Behavior documentation
//! - Observation interruption

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::animals::types::AnimalSpecies;
use crate::flora::FloraSpecies;

/// Active observation session for a specific target
#[derive(Debug, Clone)]
pub struct ObservationSession {
    /// What is being observed
    pub target: ObservationTarget,
    /// Cumulative observation time this session
    pub session_time: f32,
    /// Distance to target (affects quality)
    pub current_distance: f32,
    /// Is target currently in line of sight
    pub has_line_of_sight: bool,
    /// Time since target was last visible
    pub time_since_visible: f32,
    /// Behaviors witnessed this session
    pub behaviors_witnessed: Vec<WitnessedBehavior>,
    /// Quality multiplier based on conditions
    pub quality_multiplier: f32,
    /// Whether player is using observation tools (spyglass, etc)
    pub using_tools: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObservationTarget {
    Fauna { species: AnimalSpecies, entity_id: u64 },
    Flora { species: FloraSpecies, position: [f32; 3] },
}

/// A behavior witnessed during observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessedBehavior {
    pub behavior_type: BehaviorWitnessType,
    pub timestamp: f32,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorWitnessType {
    // Fauna behaviors
    Feeding,
    Hunting,
    Sleeping,
    Grooming,
    TerritorialDisplay,
    PackCommunication,
    Mating,
    ParentalCare,
    FleeingPredator,
    AttackingPrey,
    // Flora "behaviors" (seasonal/environmental responses)
    Blooming,
    Fruiting,
    Wilting,
    Shedding,
    Dormancy,
}

impl BehaviorWitnessType {
    /// Get observation XP bonus for witnessing this behavior
    pub fn xp_bonus(&self) -> f32 {
        match self {
            Self::Feeding | Self::Sleeping | Self::Grooming => 1.0,
            Self::Hunting | Self::TerritorialDisplay => 1.5,
            Self::PackCommunication | Self::FleeingPredator => 1.25,
            Self::Mating | Self::ParentalCare => 2.0,
            Self::AttackingPrey => 1.75,
            Self::Blooming | Self::Fruiting => 1.5,
            Self::Wilting | Self::Shedding | Self::Dormancy => 1.25,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Feeding => "Observed feeding behavior",
            Self::Hunting => "Witnessed active hunting",
            Self::Sleeping => "Found resting/sleeping",
            Self::Grooming => "Self-grooming behavior",
            Self::TerritorialDisplay => "Territorial marking or display",
            Self::PackCommunication => "Pack communication (howls, calls)",
            Self::Mating => "Courtship or mating behavior",
            Self::ParentalCare => "Caring for young",
            Self::FleeingPredator => "Fleeing from threat",
            Self::AttackingPrey => "Attacking prey",
            Self::Blooming => "Flowering/blooming",
            Self::Fruiting => "Producing fruit/seeds",
            Self::Wilting => "Wilting or stress response",
            Self::Shedding => "Shedding leaves/bark",
            Self::Dormancy => "Entering dormancy",
        }
    }
}

/// The observation manager tracks all active observation sessions
#[derive(Debug, Default)]
pub struct ObservationManager {
    /// Currently active observation sessions
    pub active_sessions: HashMap<u64, ObservationSession>,
    /// Next session ID
    next_session_id: u64,
    /// Player's current position (for distance calculations)
    pub player_position: [f32; 3],
    /// Player's look direction (for LoS)
    pub player_look_dir: [f32; 3],
    /// Field of view angle in radians
    pub field_of_view: f32,
}

impl ObservationManager {
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
            next_session_id: 0,
            player_position: [0.0, 0.0, 0.0],
            player_look_dir: [0.0, 0.0, 1.0],
            field_of_view: std::f32::consts::PI / 3.0, // 60 degrees
        }
    }

    /// Start observing a fauna target
    pub fn start_fauna_observation(
        &mut self,
        species: AnimalSpecies,
        entity_id: u64,
        target_position: [f32; 3],
    ) -> u64 {
        let session_id = self.next_session_id;
        self.next_session_id += 1;

        let distance = self.distance_to(target_position);

        self.active_sessions.insert(session_id, ObservationSession {
            target: ObservationTarget::Fauna { species, entity_id },
            session_time: 0.0,
            current_distance: distance,
            has_line_of_sight: true,
            time_since_visible: 0.0,
            behaviors_witnessed: Vec::new(),
            quality_multiplier: self.calculate_quality_multiplier(distance, true),
            using_tools: false,
        });

        session_id
    }

    /// Start observing a flora target
    pub fn start_flora_observation(
        &mut self,
        species: FloraSpecies,
        position: [f32; 3],
    ) -> u64 {
        let session_id = self.next_session_id;
        self.next_session_id += 1;

        let distance = self.distance_to(position);

        self.active_sessions.insert(session_id, ObservationSession {
            target: ObservationTarget::Flora { species, position },
            session_time: 0.0,
            current_distance: distance,
            has_line_of_sight: true,
            time_since_visible: 0.0,
            behaviors_witnessed: Vec::new(),
            quality_multiplier: self.calculate_quality_multiplier(distance, true),
            using_tools: false,
        });

        session_id
    }

    /// Update player position and look direction
    pub fn update_player(&mut self, position: [f32; 3], look_dir: [f32; 3]) {
        self.player_position = position;
        self.player_look_dir = look_dir;
    }

    /// Update an observation session
    pub fn update_session(
        &mut self,
        session_id: u64,
        target_position: [f32; 3],
        has_los: bool,
        delta_time: f32,
    ) -> Option<f32> {
        // Calculate values before borrowing session mutably
        let distance = self.distance_to(target_position);
        let in_fov = self.is_in_fov(target_position);

        let session = self.active_sessions.get_mut(&session_id)?;
        let using_tools = session.using_tools;

        session.current_distance = distance;
        session.has_line_of_sight = has_los && in_fov;

        if session.has_line_of_sight {
            session.time_since_visible = 0.0;
            session.quality_multiplier = Self::calc_quality_multiplier_static(distance, using_tools);

            // Add time based on quality
            let effective_time = delta_time * session.quality_multiplier;
            session.session_time += effective_time;

            Some(effective_time)
        } else {
            session.time_since_visible += delta_time;
            None
        }
    }

    /// Record a witnessed behavior
    pub fn record_behavior(
        &mut self,
        session_id: u64,
        behavior: BehaviorWitnessType,
        game_time: f32,
    ) {
        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            // Don't record duplicate behaviors
            if !session.behaviors_witnessed.iter().any(|b| b.behavior_type == behavior) {
                session.behaviors_witnessed.push(WitnessedBehavior {
                    behavior_type: behavior,
                    timestamp: game_time,
                    description: behavior.description().to_string(),
                });
            }
        }
    }

    /// Set whether player is using observation tools
    pub fn set_using_tools(&mut self, session_id: u64, using: bool) {
        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            session.using_tools = using;
        }
    }

    /// End an observation session and return collected data
    pub fn end_session(&mut self, session_id: u64) -> Option<ObservationSession> {
        self.active_sessions.remove(&session_id)
    }

    /// Check if session should be auto-ended (target out of sight too long)
    pub fn should_end_session(&self, session_id: u64) -> bool {
        if let Some(session) = self.active_sessions.get(&session_id) {
            // End if target out of sight for more than 5 seconds
            session.time_since_visible > 5.0
        } else {
            true
        }
    }

    /// Calculate quality multiplier based on distance and tools
    fn calculate_quality_multiplier(&self, distance: f32, using_tools: bool) -> f32 {
        Self::calc_quality_multiplier_static(distance, using_tools)
    }

    /// Static version to avoid borrow issues
    fn calc_quality_multiplier_static(distance: f32, using_tools: bool) -> f32 {
        // Base quality drops with distance
        let base_quality = if distance < 5.0 {
            1.5 // Very close, excellent observation
        } else if distance < 15.0 {
            1.0 // Good range
        } else if distance < 30.0 {
            0.7 // Medium range
        } else if distance < 50.0 {
            0.4 // Far
        } else {
            0.2 // Very far
        };

        // Tools (spyglass) extend effective range
        let tool_bonus = if using_tools { 1.5 } else { 1.0 };

        base_quality * tool_bonus
    }

    /// Calculate distance to a point from player
    fn distance_to(&self, target: [f32; 3]) -> f32 {
        let dx = target[0] - self.player_position[0];
        let dy = target[1] - self.player_position[1];
        let dz = target[2] - self.player_position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Check if target is within field of view
    fn is_in_fov(&self, target: [f32; 3]) -> bool {
        let dx = target[0] - self.player_position[0];
        let dy = target[1] - self.player_position[1];
        let dz = target[2] - self.player_position[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist < 0.001 {
            return true;
        }

        // Normalize direction to target
        let to_target = [dx / dist, dy / dist, dz / dist];

        // Dot product with look direction
        let dot = self.player_look_dir[0] * to_target[0]
            + self.player_look_dir[1] * to_target[1]
            + self.player_look_dir[2] * to_target[2];

        // Check if within FOV
        dot.acos() < self.field_of_view / 2.0
    }

    /// Get all active fauna sessions for a species
    pub fn get_fauna_sessions(&self, species: AnimalSpecies) -> Vec<u64> {
        self.active_sessions
            .iter()
            .filter_map(|(id, session)| {
                if let ObservationTarget::Fauna { species: s, .. } = &session.target {
                    if *s == species {
                        return Some(*id);
                    }
                }
                None
            })
            .collect()
    }
}

/// Observation tools that enhance study capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationTool {
    /// Basic unaided observation
    None,
    /// Extends visual range, improves observation quality at distance
    Spyglass,
    /// For flora study, reveals more details
    MagnifyingGlass,
    /// For recording observations, grants XP bonus
    FieldJournal,
    /// For plant samples, speeds flora study
    HerbariumPress,
    /// For detailed anatomical study of harvested fauna
    DissectionKit,
}

impl ObservationTool {
    pub fn quality_bonus(&self) -> f32 {
        match self {
            Self::None => 1.0,
            Self::Spyglass => 1.5,
            Self::MagnifyingGlass => 1.3,
            Self::FieldJournal => 1.2,
            Self::HerbariumPress => 1.4,
            Self::DissectionKit => 1.5,
        }
    }

    pub fn effective_for_fauna(&self) -> bool {
        matches!(self, Self::None | Self::Spyglass | Self::FieldJournal | Self::DissectionKit)
    }

    pub fn effective_for_flora(&self) -> bool {
        matches!(self, Self::None | Self::MagnifyingGlass | Self::FieldJournal | Self::HerbariumPress)
    }
}
