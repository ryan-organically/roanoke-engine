//! Unified Character Agent Manager
//!
//! Central hub for managing all character agents in the game world.
//! Coordinates between NPCs, animals, and event entities.
//! Handles cross-system communication and shared spatial awareness.

use super::communication::{CommunicationManager, BeamRenderData, LinkType};
use super::pathing::{PathFollower, PathFollowResult, SteeringBehaviors};
use super::{AgentContext, AgentId, AgentKind, CharacterAgent, EmotionalState, OrbVisualData, UnifiedBehaviorState};
use crate::animals::spatial::SpatialHash;
use crate::progression::events::WorldPhase;
use glam::Vec3;
use std::collections::HashMap;

/// Spatial cell size for agent queries
const SPATIAL_CELL_SIZE: f32 = 32.0;

/// Detection radius for agent awareness
const DEFAULT_DETECTION_RADIUS: f32 = 50.0;

/// Unified manager for all character agents
pub struct UnifiedAgentManager {
    /// Spatial hash for quick position queries
    spatial: SpatialHash<AgentId>,

    /// Communication links between agents
    pub communication: CommunicationManager,

    /// Path followers for each agent
    path_followers: HashMap<AgentId, PathFollower>,

    /// Cached awareness relationships
    awareness_cache: HashMap<AgentId, Vec<AwarenessEntry>>,

    /// Frame counter for cache invalidation
    frame: u64,

    /// Current world phase affecting behaviors
    pub world_phase: WorldPhase,

    /// Debug statistics
    pub stats: ManagerStats,
}

/// Entry in the awareness cache
#[derive(Debug, Clone)]
struct AwarenessEntry {
    target: AgentId,
    distance: f32,
    awareness_level: f32,
    emotional_response: EmotionalState,
}

/// Debug statistics
#[derive(Debug, Default, Clone)]
pub struct ManagerStats {
    pub total_agents: usize,
    pub active_links: usize,
    pub spatial_queries: usize,
    pub awareness_updates: usize,
}

impl UnifiedAgentManager {
    pub fn new() -> Self {
        Self {
            spatial: SpatialHash::new(SPATIAL_CELL_SIZE),
            communication: CommunicationManager::new(),
            path_followers: HashMap::new(),
            awareness_cache: HashMap::new(),
            frame: 0,
            world_phase: WorldPhase::Arrival,
            stats: ManagerStats::default(),
        }
    }

    /// Register an agent in the spatial system
    pub fn register_agent(&mut self, id: AgentId, position: Vec3) {
        self.spatial.insert(id, position);
        self.path_followers.insert(id, PathFollower::new());
    }

    /// Remove an agent from the system
    pub fn unregister_agent(&mut self, id: AgentId) {
        self.spatial.remove(id);
        self.path_followers.remove(&id);
        self.awareness_cache.remove(&id);
        self.communication.remove_agent(id);
    }

    /// Update agent position in spatial hash
    pub fn update_position(&mut self, id: AgentId, position: Vec3) {
        self.spatial.update(id, position);
    }

    /// Query agents within radius of a position
    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<AgentId> {
        self.spatial.query_radius(center, radius)
    }

    /// Get path follower for an agent
    pub fn path_follower(&self, id: AgentId) -> Option<&PathFollower> {
        self.path_followers.get(&id)
    }

    /// Get mutable path follower for an agent
    pub fn path_follower_mut(&mut self, id: AgentId) -> Option<&mut PathFollower> {
        self.path_followers.get_mut(&id)
    }

    /// Set agent to move to a target
    pub fn move_to(&mut self, id: AgentId, target: Vec3) {
        if let Some(follower) = self.path_followers.get_mut(&id) {
            follower.go_to(target);
        }
    }

    /// Set agent to flee from a threat
    pub fn flee_from(&mut self, id: AgentId, current_pos: Vec3, threat_pos: Vec3, distance: f32) {
        if let Some(follower) = self.path_followers.get_mut(&id) {
            follower.flee_from(current_pos, threat_pos, distance);
        }
    }

    /// Set agent to patrol around a center
    pub fn patrol(&mut self, id: AgentId, center: Vec3, radius: f32) {
        if let Some(follower) = self.path_followers.get_mut(&id) {
            follower.set_patrol(center, radius, 6);
        }
    }

    /// Main update tick
    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        player_velocity: Vec3,
        game_time: f64,
        agents: &mut dyn AgentCollection,
    ) {
        self.frame += 1;
        self.stats = ManagerStats::default();

        // Phase 1: Update communication links
        self.communication.update(dt);
        self.stats.active_links = self.communication.link_count();

        // Phase 2: Batch spatial queries
        let all_positions: Vec<(AgentId, Vec3)> = agents
            .iter_ids()
            .filter_map(|id| {
                agents.get_agent(id).map(|a| (id, a.position()))
            })
            .collect();

        self.stats.total_agents = all_positions.len();

        // Phase 3: Calculate awareness for each agent
        self.awareness_cache.clear();
        for &(agent_id, agent_pos) in &all_positions {
            let nearby = self.spatial.query_radius(agent_pos, DEFAULT_DETECTION_RADIUS);
            self.stats.spatial_queries += 1;

            let mut awareness_entries = Vec::new();

            for &other_id in &nearby {
                if other_id == agent_id {
                    continue;
                }

                if let Some(other_pos) = self.spatial.get_position(other_id) {
                    let distance = agent_pos.distance(other_pos);
                    let awareness_level = 1.0 - (distance / DEFAULT_DETECTION_RADIUS);

                    // Determine emotional response based on agent types
                    let emotional_response = self.determine_emotional_response(
                        agent_id.kind,
                        other_id.kind,
                        distance,
                    );

                    awareness_entries.push(AwarenessEntry {
                        target: other_id,
                        distance,
                        awareness_level,
                        emotional_response,
                    });

                    self.stats.awareness_updates += 1;
                }
            }

            self.awareness_cache.insert(agent_id, awareness_entries);
        }

        // Phase 4: Update each agent's behavior
        let context = AgentContext {
            player_pos,
            player_velocity,
            game_time,
            dt,
            nearby_agents: &[], // Will be filled per-agent
            world_phase: self.world_phase,
        };

        for (agent_id, agent_pos) in &all_positions {
            // Get awareness data for this agent
            let awareness = self.awareness_cache.get(agent_id).cloned().unwrap_or_default();

            // Update path following
            if let (Some(agent), Some(follower)) = (
                agents.get_agent_mut(*agent_id),
                self.path_followers.get_mut(agent_id),
            ) {
                let result = follower.update(agent.position(), agent.base_speed(), dt);

                match result {
                    PathFollowResult::Moving { velocity, target_direction, .. } => {
                        agent.set_velocity(velocity);
                        if target_direction.length_squared() > 0.01 {
                            agent.look_at(agent.position() + target_direction);
                        }
                    }
                    PathFollowResult::Idle | PathFollowResult::Complete => {
                        agent.set_velocity(Vec3::ZERO);
                    }
                    PathFollowResult::Arrived { .. } | PathFollowResult::Waiting { .. } => {
                        agent.set_velocity(Vec3::ZERO);
                    }
                }

                // Apply position update
                let new_pos = agent.position() + agent.velocity() * dt;
                agent.set_position(new_pos);
                self.spatial.update(*agent_id, new_pos);

                // Update emotional state based on awareness
                if let Some(closest) = awareness.iter().min_by(|a, b| {
                    a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    agent.set_emotional_state(closest.emotional_response);

                    // Create awareness links for close agents
                    if closest.distance < 20.0 && closest.awareness_level > 0.5 {
                        let link_type = match closest.emotional_response {
                            EmotionalState::Friendly => LinkType::Greeting,
                            EmotionalState::Hostile => LinkType::Threat,
                            EmotionalState::Fearful => LinkType::Fear,
                            EmotionalState::Alert => LinkType::Alert,
                            _ => LinkType::Awareness,
                        };
                        self.communication.add_link(
                            super::communication::CommunicationLink::new(*agent_id, closest.target, link_type),
                        );
                    }
                }
            }
        }
    }

    /// Determine emotional response between agent types
    fn determine_emotional_response(
        &self,
        perceiver: AgentKind,
        target: AgentKind,
        distance: f32,
    ) -> EmotionalState {
        match (perceiver, target) {
            // Animals seeing player
            (AgentKind::Animal, AgentKind::Player) => {
                if distance < 15.0 {
                    EmotionalState::Hostile
                } else if distance < 30.0 {
                    EmotionalState::Alert
                } else {
                    EmotionalState::Curious
                }
            }

            // NPCs seeing player
            (AgentKind::Npc, AgentKind::Player) => {
                match self.world_phase {
                    WorldPhase::Arrival => {
                        if distance < 10.0 {
                            EmotionalState::Alert
                        } else {
                            EmotionalState::Curious
                        }
                    }
                    WorldPhase::Settlement => EmotionalState::Friendly,
                    WorldPhase::Conflict => EmotionalState::Alert,
                    WorldPhase::Resolution => EmotionalState::Calm,
                }
            }

            // Animals seeing animals
            (AgentKind::Animal, AgentKind::Animal) => {
                if distance < 10.0 {
                    EmotionalState::Alert
                } else {
                    EmotionalState::Neutral
                }
            }

            // NPCs seeing NPCs
            (AgentKind::Npc, AgentKind::Npc) => EmotionalState::Friendly,

            // Events
            (_, AgentKind::Event) | (AgentKind::Event, _) => EmotionalState::Curious,

            _ => EmotionalState::Neutral,
        }
    }

    /// Get all orb visual data for rendering
    pub fn get_orb_visuals(&self, agents: &dyn AgentCollection) -> Vec<OrbVisualData> {
        agents
            .iter_ids()
            .filter_map(|id| agents.get_agent(id))
            .filter(|a| a.is_alive())
            .map(|a| a.orb_data())
            .collect()
    }

    /// Get all beam render data for communication links
    pub fn get_beam_visuals(&self, agents: &dyn AgentCollection) -> Vec<BeamRenderData> {
        let positions: HashMap<AgentId, Vec3> = agents
            .iter_ids()
            .filter_map(|id| {
                agents.get_agent(id).map(|a| (id, a.position()))
            })
            .collect();

        self.communication.get_render_data(&positions)
    }

    /// Trigger an alert propagation
    pub fn alert_group(&mut self, source: AgentId, radius: f32) {
        if let Some(source_pos) = self.spatial.get_position(source) {
            let nearby = self.spatial.query_radius(source_pos, radius);
            let same_kind: Vec<AgentId> = nearby
                .into_iter()
                .filter(|&id| id != source && id.kind == source.kind)
                .collect();

            self.communication.propagate_alert(source, &same_kind);
        }
    }

    /// Create pack bonds between animals
    pub fn bond_pack(&mut self, members: &[AgentId]) {
        for i in 0..members.len() {
            for j in (i + 1)..members.len() {
                self.communication.bond(members[i], members[j]);
            }
        }
    }

    /// Get debug stats
    pub fn debug_info(&self) -> String {
        format!(
            "Agents: {} | Links: {} | Queries: {} | Updates: {}",
            self.stats.total_agents,
            self.stats.active_links,
            self.stats.spatial_queries,
            self.stats.awareness_updates,
        )
    }
}

impl Default for UnifiedAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for collections of agents (allows abstraction over NpcManager/AnimalManager)
pub trait AgentCollection {
    /// Iterate over all agent IDs
    fn iter_ids(&self) -> Box<dyn Iterator<Item = AgentId> + '_>;

    /// Get an agent by ID
    fn get_agent(&self, id: AgentId) -> Option<&dyn CharacterAgent>;

    /// Get mutable agent by ID
    fn get_agent_mut(&mut self, id: AgentId) -> Option<&mut dyn CharacterAgent>;
}

/// Adapter to integrate NPC system with unified manager
pub struct NpcAgentAdapter<'a> {
    pub npcs: &'a mut crate::npc::NpcManager,
}

/// Adapter to integrate animal system with unified manager
pub struct AnimalAgentAdapter<'a> {
    pub animals: &'a mut crate::animals::AnimalManager,
}

/// Combined adapter for both systems
pub struct CombinedAgentAdapter<'a> {
    pub npcs: &'a mut crate::npc::NpcManager,
    pub animals: &'a mut crate::animals::AnimalManager,
}

// Note: Full trait implementations would require the NPC and Animal systems
// to implement CharacterAgent trait. This is a proof-of-concept structure.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = UnifiedAgentManager::new();
        assert_eq!(manager.stats.total_agents, 0);
    }

    #[test]
    fn test_agent_registration() {
        let mut manager = UnifiedAgentManager::new();
        let id = AgentId::npc(1);
        let pos = Vec3::new(10.0, 0.0, 10.0);

        manager.register_agent(id, pos);

        let nearby = manager.query_radius(pos, 5.0);
        assert!(nearby.contains(&id));
    }

    #[test]
    fn test_path_following() {
        let mut manager = UnifiedAgentManager::new();
        let id = AgentId::npc(1);
        let pos = Vec3::ZERO;

        manager.register_agent(id, pos);
        manager.move_to(id, Vec3::new(10.0, 0.0, 10.0));

        let follower = manager.path_follower(id).unwrap();
        assert!(follower.state.has_path());
    }
}
