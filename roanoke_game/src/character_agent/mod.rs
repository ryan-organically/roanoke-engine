//! Unified Character Agent System
//!
//! Provides a common interface for all agents in the game world:
//! - Human NPCs (villagers, traders, hunters)
//! - Animals (predators, prey, pack members)
//! - Event-driven entities (campaign characters, quest NPCs)
//!
//! This system enables:
//! - Unified spatial awareness and pathing
//! - Cross-agent communication (visual orb dialogues)
//! - Consistent behavior state machines
//! - Campaign-aware agent behaviors

pub mod pathing;
pub mod communication;
pub mod unified_manager;

use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Unique identifier for any agent in the character universe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId {
    pub kind: AgentKind,
    pub id: u64,
}

impl AgentId {
    pub fn npc(id: u32) -> Self {
        Self { kind: AgentKind::Npc, id: id as u64 }
    }

    pub fn animal(id: u64) -> Self {
        Self { kind: AgentKind::Animal, id }
    }

    pub fn event(id: u64) -> Self {
        Self { kind: AgentKind::Event, id }
    }
}

/// Type of agent in the character universe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentKind {
    /// Human NPCs - villagers, traders, quest givers
    Npc,
    /// Wildlife - predators, prey, pack animals
    Animal,
    /// Event entities - campaign characters, temporary spawns
    Event,
    /// Player representation (for inter-agent queries)
    Player,
}

/// Unified behavior state across all agent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UnifiedBehaviorState {
    /// No activity, stationary
    #[default]
    Idle,
    /// Moving along scheduled/patrol path
    Patrolling,
    /// Transitioning between schedule locations
    Traveling,
    /// Performing work/activity at location
    Working,
    /// Detected something of interest
    Alert,
    /// Actively observing target
    Observing,
    /// Moving toward target (friendly)
    Approaching,
    /// Moving toward target (hostile)
    Pursuing,
    /// Running away from threat
    Fleeing,
    /// Engaging in combat
    Attacking,
    /// Recovering from action
    Recovering,
    /// Engaged in dialogue/trade
    Interacting,
    /// Resting/sleeping
    Resting,
    /// Dead/incapacitated
    Dead,
}

impl UnifiedBehaviorState {
    /// Whether this state allows interaction with player
    pub fn allows_interaction(&self) -> bool {
        matches!(
            self,
            Self::Idle | Self::Patrolling | Self::Working | Self::Observing | Self::Interacting
        )
    }

    /// Whether this state involves movement
    pub fn is_moving(&self) -> bool {
        matches!(
            self,
            Self::Patrolling | Self::Traveling | Self::Approaching
                | Self::Pursuing | Self::Fleeing
        )
    }

    /// Whether this state is combat-related
    pub fn is_combat(&self) -> bool {
        matches!(self, Self::Pursuing | Self::Attacking | Self::Fleeing)
    }

    /// Speed multiplier for this state
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            Self::Idle | Self::Working | Self::Interacting | Self::Resting | Self::Dead => 0.0,
            Self::Patrolling | Self::Observing => 0.5,
            Self::Traveling | Self::Approaching => 0.8,
            Self::Alert | Self::Recovering => 0.3,
            Self::Pursuing | Self::Attacking => 1.0,
            Self::Fleeing => 1.2,
        }
    }
}

/// Emotional state affecting visual representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EmotionalState {
    #[default]
    Neutral,
    Friendly,
    Curious,
    Alert,
    Hostile,
    Fearful,
    Calm,
    Excited,
}

impl EmotionalState {
    /// Get the orb color for this emotional state (RGB)
    pub fn orb_color(&self) -> [f32; 3] {
        match self {
            Self::Neutral => [0.7, 0.7, 0.7],   // Gray
            Self::Friendly => [0.2, 0.8, 0.3],  // Green
            Self::Curious => [0.3, 0.5, 0.9],   // Blue
            Self::Alert => [0.9, 0.8, 0.2],     // Yellow
            Self::Hostile => [0.9, 0.2, 0.2],   // Red
            Self::Fearful => [0.7, 0.3, 0.8],   // Purple
            Self::Calm => [0.4, 0.7, 0.6],      // Teal
            Self::Excited => [0.9, 0.6, 0.2],   // Orange
        }
    }

    /// Get the pulse rate for orb animation (Hz)
    pub fn pulse_rate(&self) -> f32 {
        match self {
            Self::Neutral => 0.5,
            Self::Friendly => 1.0,
            Self::Curious => 1.5,
            Self::Alert => 3.0,
            Self::Hostile => 4.0,
            Self::Fearful => 5.0,
            Self::Calm => 0.3,
            Self::Excited => 2.5,
        }
    }

    /// Get emissive intensity for glow effect
    pub fn emissive_intensity(&self) -> f32 {
        match self {
            Self::Neutral => 0.0,
            Self::Friendly => 0.2,
            Self::Curious => 0.3,
            Self::Alert => 0.5,
            Self::Hostile => 0.8,
            Self::Fearful => 0.6,
            Self::Calm => 0.1,
            Self::Excited => 0.4,
        }
    }
}

/// Awareness level toward a specific target
#[derive(Debug, Clone)]
pub struct AwarenessTarget {
    pub target_id: AgentId,
    pub awareness_level: f32,      // 0.0 = unaware, 1.0 = fully aware
    pub last_known_position: Vec3,
    pub last_seen: Option<Instant>,
    pub emotional_response: EmotionalState,
}

/// Context for agent updates
pub struct AgentContext<'a> {
    pub player_pos: Vec3,
    pub player_velocity: Vec3,
    pub game_time: f64,
    pub dt: f32,
    pub nearby_agents: &'a [AgentId],
    pub world_phase: crate::progression::events::WorldPhase,
}

/// Core trait for all character agents
pub trait CharacterAgent {
    /// Get unique agent identifier
    fn agent_id(&self) -> AgentId;

    /// Get current world position
    fn position(&self) -> Vec3;

    /// Set world position
    fn set_position(&mut self, pos: Vec3);

    /// Get current velocity
    fn velocity(&self) -> Vec3;

    /// Set velocity
    fn set_velocity(&mut self, vel: Vec3);

    /// Get base movement speed
    fn base_speed(&self) -> f32;

    /// Get current awareness level (0.0 - 1.0)
    fn awareness(&self) -> f32;

    /// Set awareness level
    fn set_awareness(&mut self, level: f32);

    /// Get current behavior state
    fn behavior_state(&self) -> UnifiedBehaviorState;

    /// Set behavior state
    fn set_behavior_state(&mut self, state: UnifiedBehaviorState);

    /// Get emotional state
    fn emotional_state(&self) -> EmotionalState;

    /// Set emotional state
    fn set_emotional_state(&mut self, state: EmotionalState);

    /// Get detection radius
    fn detection_radius(&self) -> f32;

    /// Whether this agent is alive/active
    fn is_alive(&self) -> bool;

    /// Get look direction (for facing)
    fn look_direction(&self) -> Vec3;

    /// Face toward a position
    fn look_at(&mut self, target: Vec3);

    /// Update agent state
    fn update(&mut self, ctx: &AgentContext, dt: f32);

    /// Whether this agent can communicate with another
    fn can_communicate_with(&self, other_kind: AgentKind) -> bool {
        match self.agent_id().kind {
            AgentKind::Npc => matches!(other_kind, AgentKind::Npc | AgentKind::Player),
            AgentKind::Animal => matches!(other_kind, AgentKind::Animal),
            AgentKind::Event => true, // Events can affect everything
            AgentKind::Player => true,
        }
    }

    /// Get orb visual representation data
    fn orb_data(&self) -> OrbVisualData {
        let emotion = self.emotional_state();
        OrbVisualData {
            position: self.position(),
            color: emotion.orb_color(),
            emissive: emotion.emissive_intensity(),
            pulse_rate: emotion.pulse_rate(),
            scale: self.orb_scale(),
        }
    }

    /// Get orb scale based on agent type/size
    fn orb_scale(&self) -> f32 {
        1.0 // Default, override for larger/smaller agents
    }
}

/// Visual data for rendering an agent's orb representation
#[derive(Debug, Clone, Copy)]
pub struct OrbVisualData {
    pub position: Vec3,
    pub color: [f32; 3],
    pub emissive: f32,
    pub pulse_rate: f32,
    pub scale: f32,
}

/// Agent perception result
#[derive(Debug, Clone)]
pub struct PerceptionResult {
    pub perceived_agent: AgentId,
    pub distance: f32,
    pub direction: Vec3,
    pub awareness_increase: f32,
    pub suggested_response: EmotionalState,
}

/// Calculate awareness increase based on distance and agent types
pub fn calculate_awareness_gain(
    perceiver_kind: AgentKind,
    target_kind: AgentKind,
    distance: f32,
    detection_radius: f32,
    dt: f32,
) -> f32 {
    if distance > detection_radius {
        return 0.0;
    }

    let distance_factor = 1.0 - (distance / detection_radius);
    let base_rate = match (perceiver_kind, target_kind) {
        (AgentKind::Animal, AgentKind::Player) => 1.5,   // Animals detect player quickly
        (AgentKind::Npc, AgentKind::Player) => 1.0,      // NPCs at normal rate
        (AgentKind::Animal, AgentKind::Animal) => 0.8,   // Pack awareness
        (AgentKind::Npc, AgentKind::Npc) => 0.5,         // Social awareness
        _ => 0.5,
    };

    base_rate * distance_factor * dt
}

/// Calculate emotional response based on agent relationship
pub fn calculate_emotional_response(
    perceiver: &dyn CharacterAgent,
    target_kind: AgentKind,
    distance: f32,
    relationship_value: i32, // -100 to 100
) -> EmotionalState {
    // Base response on relationship
    let base_response = if relationship_value > 50 {
        EmotionalState::Friendly
    } else if relationship_value > 0 {
        EmotionalState::Neutral
    } else if relationship_value > -50 {
        EmotionalState::Alert
    } else {
        EmotionalState::Hostile
    };

    // Modify based on current state and distance
    match perceiver.behavior_state() {
        UnifiedBehaviorState::Fleeing => EmotionalState::Fearful,
        UnifiedBehaviorState::Attacking | UnifiedBehaviorState::Pursuing => EmotionalState::Hostile,
        UnifiedBehaviorState::Resting if distance < 10.0 => EmotionalState::Alert,
        _ => base_response,
    }
}
