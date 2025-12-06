//! Visual Communication System for Agent Orbs
//!
//! Renders visual dialogue/awareness links between agents as colored beams.
//! This creates a visual language showing inter-agent relationships:
//! - Pack animals aware of each other
//! - NPCs in conversation
//! - Predator-prey awareness
//! - Alert propagation through groups

use super::{AgentId, AgentKind, EmotionalState};
use glam::Vec3;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Visual communication link between two agents
#[derive(Debug, Clone)]
pub struct CommunicationLink {
    /// Source agent
    pub from: AgentId,
    /// Target agent
    pub to: AgentId,
    /// Type of communication
    pub link_type: LinkType,
    /// Current intensity (0.0 - 1.0)
    pub intensity: f32,
    /// When the link was established
    pub established: Instant,
    /// Link duration (None = persistent until broken)
    pub duration: Option<Duration>,
    /// Visual properties
    pub visual: LinkVisual,
}

impl CommunicationLink {
    pub fn new(from: AgentId, to: AgentId, link_type: LinkType) -> Self {
        let visual = LinkVisual::from_link_type(&link_type);
        Self {
            from,
            to,
            link_type,
            intensity: 1.0,
            established: Instant::now(),
            duration: Some(Duration::from_secs(3)),
            visual,
        }
    }

    /// Create a persistent link (for pack bonds, etc.)
    pub fn persistent(from: AgentId, to: AgentId, link_type: LinkType) -> Self {
        let mut link = Self::new(from, to, link_type);
        link.duration = None;
        link
    }

    /// Check if the link has expired
    pub fn is_expired(&self) -> bool {
        if let Some(duration) = self.duration {
            self.established.elapsed() >= duration
        } else {
            false
        }
    }

    /// Get progress through link lifetime (0.0 - 1.0)
    pub fn lifetime_progress(&self) -> f32 {
        if let Some(duration) = self.duration {
            let elapsed = self.established.elapsed().as_secs_f32();
            let total = duration.as_secs_f32();
            (elapsed / total).min(1.0)
        } else {
            0.0 // Persistent links don't progress
        }
    }

    /// Update visual intensity (fades out as link ages)
    pub fn update_intensity(&mut self) {
        if self.duration.is_some() {
            let progress = self.lifetime_progress();
            // Fade out in last 30% of lifetime
            if progress > 0.7 {
                self.intensity = 1.0 - ((progress - 0.7) / 0.3);
            }
        }
    }
}

/// Type of communication link
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    /// Pack awareness (animals in same pack)
    PackBond,
    /// Alert signal (danger detected)
    Alert,
    /// Friendly greeting
    Greeting,
    /// Hostile awareness
    Threat,
    /// Social conversation
    Conversation,
    /// Trade negotiation
    Trading,
    /// Pursuit connection
    Hunting,
    /// Fear/flight trigger
    Fear,
    /// Territorial warning
    Warning,
    /// General awareness
    Awareness,
}

impl LinkType {
    /// Get the base color for this link type
    pub fn color(&self) -> [f32; 3] {
        match self {
            Self::PackBond => [0.3, 0.7, 0.3],      // Soft green
            Self::Alert => [0.9, 0.8, 0.2],         // Yellow
            Self::Greeting => [0.4, 0.8, 0.4],      // Bright green
            Self::Threat => [0.9, 0.2, 0.2],        // Red
            Self::Conversation => [0.5, 0.6, 0.9],  // Soft blue
            Self::Trading => [0.9, 0.7, 0.3],       // Gold
            Self::Hunting => [0.8, 0.3, 0.2],       // Dark red
            Self::Fear => [0.7, 0.3, 0.7],          // Purple
            Self::Warning => [0.9, 0.5, 0.2],       // Orange
            Self::Awareness => [0.6, 0.6, 0.6],     // Gray
        }
    }

    /// Get pulse rate for this link type
    pub fn pulse_rate(&self) -> f32 {
        match self {
            Self::PackBond => 0.5,
            Self::Alert => 4.0,
            Self::Greeting => 1.5,
            Self::Threat => 5.0,
            Self::Conversation => 1.0,
            Self::Trading => 0.8,
            Self::Hunting => 3.0,
            Self::Fear => 6.0,
            Self::Warning => 3.5,
            Self::Awareness => 0.3,
        }
    }

    /// Get beam width for this link type
    pub fn beam_width(&self) -> f32 {
        match self {
            Self::PackBond => 0.1,
            Self::Alert => 0.15,
            Self::Greeting => 0.08,
            Self::Threat => 0.2,
            Self::Conversation => 0.05,
            Self::Trading => 0.1,
            Self::Hunting => 0.25,
            Self::Fear => 0.15,
            Self::Warning => 0.18,
            Self::Awareness => 0.03,
        }
    }
}

/// Visual properties for rendering a link
#[derive(Debug, Clone)]
pub struct LinkVisual {
    /// RGB color
    pub color: [f32; 3],
    /// Pulse rate (Hz)
    pub pulse_rate: f32,
    /// Beam width
    pub width: f32,
    /// Whether beam is bidirectional
    pub bidirectional: bool,
    /// Particle effect along beam
    pub particles: bool,
    /// Glow intensity
    pub glow: f32,
}

impl LinkVisual {
    pub fn from_link_type(link_type: &LinkType) -> Self {
        let bidirectional = matches!(
            link_type,
            LinkType::PackBond | LinkType::Conversation | LinkType::Trading
        );

        let particles = matches!(
            link_type,
            LinkType::Alert | LinkType::Threat | LinkType::Fear | LinkType::Hunting
        );

        let glow = match link_type {
            LinkType::Threat | LinkType::Hunting => 0.8,
            LinkType::Alert | LinkType::Fear => 0.6,
            LinkType::Warning => 0.5,
            LinkType::Greeting | LinkType::Trading => 0.3,
            _ => 0.2,
        };

        Self {
            color: link_type.color(),
            pulse_rate: link_type.pulse_rate(),
            width: link_type.beam_width(),
            bidirectional,
            particles,
            glow,
        }
    }
}

/// Render data for a communication beam
#[derive(Debug, Clone)]
pub struct BeamRenderData {
    pub start: Vec3,
    pub end: Vec3,
    pub color: [f32; 3],
    pub width: f32,
    pub intensity: f32,
    pub pulse_phase: f32,
    pub particles: bool,
}

/// Manages all active communication links
#[derive(Debug, Default)]
pub struct CommunicationManager {
    /// All active links
    links: Vec<CommunicationLink>,
    /// Quick lookup by agent
    agent_links: HashMap<AgentId, Vec<usize>>,
    /// Current animation time
    time: f32,
}

impl CommunicationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new communication link
    pub fn add_link(&mut self, link: CommunicationLink) {
        // Don't add duplicate links
        if self.links.iter().any(|l| l.from == link.from && l.to == link.to && l.link_type == link.link_type) {
            return;
        }

        let idx = self.links.len();
        self.agent_links.entry(link.from).or_default().push(idx);
        self.agent_links.entry(link.to).or_default().push(idx);
        self.links.push(link);
    }

    /// Create a quick alert link
    pub fn alert(&mut self, from: AgentId, to: AgentId) {
        self.add_link(CommunicationLink::new(from, to, LinkType::Alert));
    }

    /// Create a greeting link
    pub fn greet(&mut self, from: AgentId, to: AgentId) {
        self.add_link(CommunicationLink::new(from, to, LinkType::Greeting));
    }

    /// Create a threat link
    pub fn threaten(&mut self, from: AgentId, to: AgentId) {
        self.add_link(CommunicationLink::new(from, to, LinkType::Threat));
    }

    /// Create a pack bond link
    pub fn bond(&mut self, from: AgentId, to: AgentId) {
        self.add_link(CommunicationLink::persistent(from, to, LinkType::PackBond));
    }

    /// Create an awareness link
    pub fn aware(&mut self, observer: AgentId, observed: AgentId) {
        self.add_link(CommunicationLink::new(observer, observed, LinkType::Awareness));
    }

    /// Create conversation link (bidirectional)
    pub fn converse(&mut self, a: AgentId, b: AgentId) {
        self.add_link(CommunicationLink::new(a, b, LinkType::Conversation));
    }

    /// Create hunting link
    pub fn hunt(&mut self, predator: AgentId, prey: AgentId) {
        self.add_link(CommunicationLink::new(predator, prey, LinkType::Hunting));
    }

    /// Create fear link
    pub fn fear(&mut self, afraid: AgentId, scary: AgentId) {
        self.add_link(CommunicationLink::new(afraid, scary, LinkType::Fear));
    }

    /// Propagate alert through a group
    pub fn propagate_alert(&mut self, source: AgentId, group: &[AgentId]) {
        for &member in group {
            if member != source {
                self.alert(source, member);
            }
        }
    }

    /// Remove all links involving an agent
    pub fn remove_agent(&mut self, agent: AgentId) {
        self.links.retain(|l| l.from != agent && l.to != agent);
        self.agent_links.remove(&agent);
        // Rebuild index (simplified - could be optimized)
        self.rebuild_index();
    }

    /// Remove a specific link
    pub fn remove_link(&mut self, from: AgentId, to: AgentId, link_type: LinkType) {
        self.links.retain(|l| !(l.from == from && l.to == to && l.link_type == link_type));
        self.rebuild_index();
    }

    /// Rebuild the agent lookup index
    fn rebuild_index(&mut self) {
        self.agent_links.clear();
        for (idx, link) in self.links.iter().enumerate() {
            self.agent_links.entry(link.from).or_default().push(idx);
            self.agent_links.entry(link.to).or_default().push(idx);
        }
    }

    /// Update all links, removing expired ones
    pub fn update(&mut self, dt: f32) {
        self.time += dt;

        // Update intensities
        for link in &mut self.links {
            link.update_intensity();
        }

        // Remove expired links
        let before_count = self.links.len();
        self.links.retain(|l| !l.is_expired());

        if self.links.len() != before_count {
            self.rebuild_index();
        }
    }

    /// Get all links for an agent
    pub fn get_links(&self, agent: AgentId) -> Vec<&CommunicationLink> {
        self.agent_links
            .get(&agent)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.links.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get render data for all visible beams
    pub fn get_render_data(
        &self,
        agent_positions: &HashMap<AgentId, Vec3>,
    ) -> Vec<BeamRenderData> {
        let mut beams = Vec::with_capacity(self.links.len());

        for link in &self.links {
            let Some(&start) = agent_positions.get(&link.from) else {
                continue;
            };
            let Some(&end) = agent_positions.get(&link.to) else {
                continue;
            };

            // Calculate pulse phase
            let pulse_phase = (self.time * link.visual.pulse_rate * std::f32::consts::TAU).sin() * 0.5 + 0.5;

            beams.push(BeamRenderData {
                start: start + Vec3::Y * 1.5, // Offset to orb height
                end: end + Vec3::Y * 1.5,
                color: link.visual.color,
                width: link.visual.width,
                intensity: link.intensity * (0.7 + pulse_phase * 0.3),
                pulse_phase,
                particles: link.visual.particles,
            });
        }

        beams
    }

    /// Get total active link count
    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    /// Check if two agents have any link
    pub fn are_linked(&self, a: AgentId, b: AgentId) -> bool {
        self.links.iter().any(|l| {
            (l.from == a && l.to == b) || (l.from == b && l.to == a)
        })
    }
}

/// Orb dialogue state for visual conversations
#[derive(Debug, Clone)]
pub struct OrbDialogue {
    pub participants: Vec<AgentId>,
    pub current_speaker: Option<AgentId>,
    pub emotion_sequence: Vec<(AgentId, EmotionalState)>,
    pub sequence_index: usize,
    pub time_per_emotion: f32,
    pub elapsed: f32,
}

impl OrbDialogue {
    /// Create a new dialogue between agents
    pub fn new(participants: Vec<AgentId>) -> Self {
        Self {
            participants,
            current_speaker: None,
            emotion_sequence: Vec::new(),
            sequence_index: 0,
            time_per_emotion: 2.0,
            elapsed: 0.0,
        }
    }

    /// Add an emotion beat to the dialogue
    pub fn add_beat(&mut self, speaker: AgentId, emotion: EmotionalState) {
        self.emotion_sequence.push((speaker, emotion));
    }

    /// Update dialogue progression
    pub fn update(&mut self, dt: f32) -> Option<(AgentId, EmotionalState)> {
        self.elapsed += dt;

        if self.elapsed >= self.time_per_emotion {
            self.elapsed = 0.0;
            self.sequence_index += 1;
        }

        self.emotion_sequence.get(self.sequence_index).cloned()
    }

    /// Check if dialogue is complete
    pub fn is_complete(&self) -> bool {
        self.sequence_index >= self.emotion_sequence.len()
    }
}

/// Helper to create common dialogue patterns
pub struct DialoguePatterns;

impl DialoguePatterns {
    /// Create a friendly greeting dialogue
    pub fn greeting(a: AgentId, b: AgentId) -> OrbDialogue {
        let mut dialogue = OrbDialogue::new(vec![a, b]);
        dialogue.add_beat(a, EmotionalState::Friendly);
        dialogue.add_beat(b, EmotionalState::Curious);
        dialogue.add_beat(b, EmotionalState::Friendly);
        dialogue.add_beat(a, EmotionalState::Calm);
        dialogue
    }

    /// Create a threatening encounter
    pub fn threat(aggressor: AgentId, target: AgentId) -> OrbDialogue {
        let mut dialogue = OrbDialogue::new(vec![aggressor, target]);
        dialogue.add_beat(aggressor, EmotionalState::Alert);
        dialogue.add_beat(target, EmotionalState::Curious);
        dialogue.add_beat(aggressor, EmotionalState::Hostile);
        dialogue.add_beat(target, EmotionalState::Fearful);
        dialogue
    }

    /// Create a pack alert sequence
    pub fn pack_alert(alpha: AgentId, members: &[AgentId]) -> OrbDialogue {
        let mut participants = vec![alpha];
        participants.extend(members.iter().copied());

        let mut dialogue = OrbDialogue::new(participants);
        dialogue.time_per_emotion = 0.5; // Quick alert sequence

        // Alpha alerts
        dialogue.add_beat(alpha, EmotionalState::Alert);

        // Each member responds
        for &member in members {
            dialogue.add_beat(member, EmotionalState::Alert);
        }

        // All become hostile/ready
        dialogue.add_beat(alpha, EmotionalState::Hostile);
        for &member in members {
            dialogue.add_beat(member, EmotionalState::Hostile);
        }

        dialogue
    }

    /// Create a trade negotiation
    pub fn trade(buyer: AgentId, seller: AgentId) -> OrbDialogue {
        let mut dialogue = OrbDialogue::new(vec![buyer, seller]);
        dialogue.add_beat(buyer, EmotionalState::Curious);
        dialogue.add_beat(seller, EmotionalState::Friendly);
        dialogue.add_beat(buyer, EmotionalState::Excited);
        dialogue.add_beat(seller, EmotionalState::Calm);
        dialogue.add_beat(buyer, EmotionalState::Friendly);
        dialogue.add_beat(seller, EmotionalState::Friendly);
        dialogue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_creation() {
        let mut manager = CommunicationManager::new();
        let npc1 = AgentId::npc(1);
        let npc2 = AgentId::npc(2);

        manager.greet(npc1, npc2);
        assert_eq!(manager.link_count(), 1);
        assert!(manager.are_linked(npc1, npc2));
    }

    #[test]
    fn test_dialogue_progression() {
        let a = AgentId::npc(1);
        let b = AgentId::npc(2);

        let mut dialogue = DialoguePatterns::greeting(a, b);

        // Should have 4 beats
        assert!(!dialogue.is_complete());

        // Advance through dialogue
        for _ in 0..5 {
            dialogue.update(2.1);
        }

        assert!(dialogue.is_complete());
    }
}
