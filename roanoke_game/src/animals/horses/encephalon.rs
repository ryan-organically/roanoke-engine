//! Horse-Encephalon: Advanced AI Behavioral Engine
//!
//! A sophisticated AI system for horse behavior that simulates:
//! - Emotional states with gradual transitions
//! - Personality traits affecting decision making
//! - Memory of player interactions and environmental events
//! - Social awareness with other horses
//! - Environmental perception and threat assessment
//! - Needs system (hunger, thirst, rest, social)

use super::types::{HorseSpecies, TemperamentProfile, HerdType};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Instant;

/// The Horse-Encephalon AI core
/// Manages all cognitive and emotional processing for a horse
#[derive(Debug)]
pub struct HorseEncephalon {
    /// Current emotional state
    pub emotional_state: EmotionalState,
    /// Emotional blend weights (multiple emotions can be active)
    pub emotion_weights: EmotionWeights,
    /// Personality traits (fixed per horse)
    pub personality: PersonalityTraits,
    /// Memory of significant events
    pub memories: MemoryBank,
    /// Current needs levels
    pub needs: NeedsSystem,
    /// Awareness of surroundings
    pub awareness: AwarenessState,
    /// Social context
    pub social: SocialContext,
    /// Decision-making state
    pub decision_state: DecisionState,
    /// Behavioral modifiers from species
    pub species_modifiers: TemperamentProfile,
}

impl HorseEncephalon {
    /// Create a new encephalon for a horse
    pub fn new(species: HorseSpecies, personality_seed: u64) -> Self {
        let temperament = species.temperament();
        let personality = PersonalityTraits::generate(personality_seed, &temperament);

        Self {
            emotional_state: EmotionalState::Calm,
            emotion_weights: EmotionWeights::default(),
            personality,
            memories: MemoryBank::new(),
            needs: NeedsSystem::default(),
            awareness: AwarenessState::default(),
            social: SocialContext::default(),
            decision_state: DecisionState::Idle,
            species_modifiers: temperament,
        }
    }

    /// Main update tick for the encephalon
    pub fn update(
        &mut self,
        dt: f32,
        position: Vec3,
        player_pos: Option<Vec3>,
        nearby_horses: &[Vec3],
        threats: &[ThreatInfo],
        environment: &EnvironmentContext,
    ) {
        // Update needs
        self.needs.update(dt);

        // Update awareness
        self.update_awareness(position, player_pos, nearby_horses, threats, environment);

        // Process emotions
        self.process_emotions(dt);

        // Make decisions
        self.update_decision_state(dt);

        // Decay old memories
        self.memories.decay(dt);
    }

    /// Update awareness of surroundings
    fn update_awareness(
        &mut self,
        position: Vec3,
        player_pos: Option<Vec3>,
        nearby_horses: &[Vec3],
        threats: &[ThreatInfo],
        environment: &EnvironmentContext,
    ) {
        // Player awareness
        if let Some(player) = player_pos {
            let dist = position.distance(player);
            self.awareness.player_distance = Some(dist);
            self.awareness.player_detected = dist < 50.0;

            // Update player familiarity based on proximity
            if dist < 30.0 {
                let familiarity_gain = 0.001 * (1.0 - dist / 30.0);
                self.awareness.player_familiarity =
                    (self.awareness.player_familiarity + familiarity_gain).min(1.0);
            }
        } else {
            self.awareness.player_detected = false;
            self.awareness.player_distance = None;
        }

        // Horse awareness
        self.awareness.nearby_horse_count = nearby_horses.len();
        self.awareness.nearest_horse_distance = nearby_horses
            .iter()
            .map(|h| position.distance(*h))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Threat assessment
        self.awareness.threat_level = self.calculate_threat_level(threats);
        self.awareness.highest_threat = threats
            .iter()
            .max_by(|a, b| a.danger.partial_cmp(&b.danger).unwrap_or(std::cmp::Ordering::Equal))
            .cloned();

        // Environmental awareness
        self.awareness.terrain_type = environment.terrain_type;
        self.awareness.weather_severity = environment.weather_severity;
        self.awareness.time_of_day = environment.time_of_day;
        self.awareness.in_safe_zone = environment.is_safe_zone;
    }

    /// Calculate overall threat level
    fn calculate_threat_level(&self, threats: &[ThreatInfo]) -> f32 {
        if threats.is_empty() {
            return 0.0;
        }

        let mut total_threat = 0.0;
        for threat in threats {
            // Closer threats are more concerning
            let distance_factor = 1.0 / (1.0 + threat.distance * 0.1);
            // Moving toward us is more threatening
            let approach_factor = if threat.approaching { 1.5 } else { 0.8 };
            total_threat += threat.danger * distance_factor * approach_factor;
        }

        // Personality affects threat perception
        let nervousness_factor = 0.5 + self.personality.nervousness * 0.5;
        let courage_factor = 1.5 - self.personality.courage * 0.5;

        (total_threat * nervousness_factor * courage_factor).min(1.0)
    }

    /// Process emotions based on current state
    fn process_emotions(&mut self, dt: f32) {
        let weights = &mut self.emotion_weights;

        // Threat response
        let threat = self.awareness.threat_level;
        if threat > 0.7 {
            weights.fear = (weights.fear + 0.3 * dt).min(1.0);
            weights.calm = (weights.calm - 0.4 * dt).max(0.0);
        } else if threat > 0.4 {
            weights.anxiety = (weights.anxiety + 0.2 * dt).min(1.0);
            weights.alert = (weights.alert + 0.2 * dt).min(1.0);
        } else if threat < 0.1 {
            weights.fear = (weights.fear - 0.1 * dt).max(0.0);
            weights.anxiety = (weights.anxiety - 0.1 * dt).max(0.0);
        }

        // Social needs affect emotions
        if self.needs.social < 0.3 && self.awareness.nearby_horse_count == 0 {
            weights.lonely = (weights.lonely + 0.05 * dt).min(1.0);
        } else if self.awareness.nearby_horse_count > 0 {
            weights.lonely = (weights.lonely - 0.1 * dt).max(0.0);
            weights.content = (weights.content + 0.05 * dt).min(1.0);
        }

        // Player familiarity affects trust
        if self.awareness.player_familiarity > 0.5 && self.awareness.player_detected {
            weights.trust = (weights.trust + 0.02 * dt).min(1.0);
            weights.curious = (weights.curious + 0.01 * dt).min(0.5);
        }

        // Hunger affects mood
        if self.needs.hunger < 0.3 {
            weights.irritable = (weights.irritable + 0.03 * dt).min(0.5);
            weights.content = (weights.content - 0.02 * dt).max(0.0);
        }

        // Rest affects everything
        if self.needs.rest < 0.2 {
            weights.exhausted = (weights.exhausted + 0.05 * dt).min(1.0);
            // Exhaustion reduces responsiveness
            weights.alert = (weights.alert * 0.95).max(0.0);
        }

        // Natural calm return when safe
        if self.awareness.in_safe_zone && threat < 0.1 {
            weights.calm = (weights.calm + 0.1 * dt).min(1.0);
        }

        // Determine dominant emotional state
        self.emotional_state = self.determine_dominant_emotion();
    }

    /// Determine the dominant emotional state from weights
    fn determine_dominant_emotion(&self) -> EmotionalState {
        let w = &self.emotion_weights;

        // Priority-based state selection
        if w.fear > 0.7 {
            return EmotionalState::Panicked;
        }
        if w.fear > 0.4 {
            return EmotionalState::Frightened;
        }
        if w.exhausted > 0.6 {
            return EmotionalState::Exhausted;
        }
        if w.anxiety > 0.5 {
            return EmotionalState::Nervous;
        }
        if w.alert > 0.6 && w.fear < 0.3 {
            return EmotionalState::Alert;
        }
        if w.curious > 0.5 {
            return EmotionalState::Curious;
        }
        if w.trust > 0.6 && w.calm > 0.4 {
            return EmotionalState::Bonded;
        }
        if w.content > 0.5 && w.calm > 0.5 {
            return EmotionalState::Content;
        }
        if w.lonely > 0.5 {
            return EmotionalState::Lonely;
        }
        if w.irritable > 0.4 {
            return EmotionalState::Irritable;
        }
        if w.calm > 0.3 {
            return EmotionalState::Calm;
        }

        EmotionalState::Neutral
    }

    /// Update decision state based on emotions and needs
    fn update_decision_state(&mut self, _dt: f32) {
        self.decision_state = match self.emotional_state {
            EmotionalState::Panicked => DecisionState::Fleeing,
            EmotionalState::Frightened => {
                if self.personality.courage > 0.7 {
                    DecisionState::Alert
                } else {
                    DecisionState::PreparingToFlee
                }
            }
            EmotionalState::Nervous => DecisionState::Alert,
            EmotionalState::Alert => {
                if self.awareness.threat_level > 0.2 {
                    DecisionState::Assessing
                } else {
                    DecisionState::Alert
                }
            }
            EmotionalState::Curious => DecisionState::Investigating,
            EmotionalState::Bonded => {
                if self.awareness.player_detected {
                    DecisionState::Following
                } else {
                    DecisionState::Idle
                }
            }
            EmotionalState::Content | EmotionalState::Calm => {
                // Check needs
                if self.needs.hunger < 0.4 {
                    DecisionState::Grazing
                } else if self.needs.thirst < 0.4 {
                    DecisionState::SeekingWater
                } else if self.needs.rest < 0.3 {
                    DecisionState::Resting
                } else if self.needs.social < 0.3 && self.awareness.nearby_horse_count == 0 {
                    DecisionState::SeekingHerd
                } else {
                    DecisionState::Idle
                }
            }
            EmotionalState::Exhausted => DecisionState::Resting,
            EmotionalState::Lonely => DecisionState::SeekingHerd,
            EmotionalState::Irritable => DecisionState::Idle,
            EmotionalState::Neutral => DecisionState::Idle,
        };
    }

    /// Record an interaction with the player
    pub fn record_player_interaction(&mut self, interaction: PlayerInteraction, success: bool) {
        let memory = Memory {
            memory_type: MemoryType::PlayerInteraction(interaction),
            emotional_impact: if success { 0.2 } else { -0.15 },
            strength: 1.0,
            timestamp: Instant::now(),
        };

        self.memories.add(memory);

        // Immediate emotional impact
        if success {
            self.emotion_weights.trust += 0.05;
            self.emotion_weights.fear = (self.emotion_weights.fear - 0.03).max(0.0);
        } else {
            self.emotion_weights.fear += 0.1;
            self.emotion_weights.trust = (self.emotion_weights.trust - 0.05).max(0.0);
        }
    }

    /// Record a threat encounter
    pub fn record_threat(&mut self, threat_type: ThreatType, escaped: bool) {
        let memory = Memory {
            memory_type: MemoryType::ThreatEncounter(threat_type),
            emotional_impact: if escaped { -0.1 } else { -0.3 },
            strength: 1.0,
            timestamp: Instant::now(),
        };

        self.memories.add(memory);

        // Immediate trauma/relief
        if escaped {
            self.emotion_weights.calm += 0.1;
        } else {
            self.emotion_weights.fear += 0.2;
            self.emotion_weights.anxiety += 0.15;
        }
    }

    /// Get behavior speed modifier based on emotional state
    pub fn movement_modifier(&self) -> f32 {
        match self.emotional_state {
            EmotionalState::Panicked => 1.4,
            EmotionalState::Frightened => 1.2,
            EmotionalState::Alert | EmotionalState::Nervous => 1.0,
            EmotionalState::Curious => 0.7,
            EmotionalState::Content | EmotionalState::Calm => 0.8,
            EmotionalState::Exhausted => 0.5,
            EmotionalState::Bonded => 0.9,
            _ => 0.8,
        }
    }

    /// Get the current trust level with player
    pub fn player_trust(&self) -> f32 {
        self.emotion_weights.trust * self.awareness.player_familiarity
    }

    /// Check if horse will accept player approach
    pub fn will_accept_approach(&self, distance: f32) -> bool {
        let trust = self.player_trust();
        let base_threshold = 10.0 + trust * 20.0; // 10-30 units depending on trust
        let nervousness_penalty = self.personality.nervousness * 5.0;
        let threshold = base_threshold - nervousness_penalty;

        distance > threshold || trust > 0.7
    }

    /// Check if horse is ready for taming interaction
    pub fn taming_readiness(&self) -> f32 {
        let mut readiness = 0.0;

        // Trust is essential
        readiness += self.emotion_weights.trust * 0.3;

        // Calm helps
        readiness += self.emotion_weights.calm * 0.2;

        // Curiosity helps
        readiness += self.emotion_weights.curious * 0.15;

        // Fear hurts
        readiness -= self.emotion_weights.fear * 0.4;
        readiness -= self.emotion_weights.anxiety * 0.2;

        // Player familiarity is critical
        readiness += self.awareness.player_familiarity * 0.25;

        // Personality affects base readiness
        readiness -= self.personality.nervousness * 0.1;
        readiness -= self.personality.stubbornness * 0.1;

        readiness.clamp(0.0, 1.0)
    }
}

/// Emotional states for horses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionalState {
    /// Relaxed and at ease
    Calm,
    /// No strong emotion
    Neutral,
    /// Aware and watchful
    Alert,
    /// Slightly worried
    Nervous,
    /// Scared but not panicked
    Frightened,
    /// Full flight response
    Panicked,
    /// Interested in something
    Curious,
    /// Happy and satisfied
    Content,
    /// Tired and needing rest
    Exhausted,
    /// Wants herd company
    Lonely,
    /// Slightly aggressive
    Irritable,
    /// Strongly bonded with player
    Bonded,
}

impl EmotionalState {
    /// Get behavior modifier for this state
    pub fn behavior_modifier(&self) -> f32 {
        match self {
            Self::Calm => 1.0,
            Self::Neutral => 1.0,
            Self::Alert => 1.1,
            Self::Nervous => 1.2,
            Self::Frightened => 1.4,
            Self::Panicked => 1.6,
            Self::Curious => 0.9,
            Self::Content => 0.9,
            Self::Exhausted => 0.6,
            Self::Lonely => 0.95,
            Self::Irritable => 1.1,
            Self::Bonded => 1.0,
        }
    }
}

/// Emotional weights for blending states
#[derive(Debug, Clone, Default)]
pub struct EmotionWeights {
    pub fear: f32,
    pub anxiety: f32,
    pub calm: f32,
    pub alert: f32,
    pub curious: f32,
    pub content: f32,
    pub trust: f32,
    pub lonely: f32,
    pub exhausted: f32,
    pub irritable: f32,
}

impl EmotionWeights {
    /// Create calm default state
    pub fn calm() -> Self {
        Self {
            calm: 0.5,
            ..Default::default()
        }
    }
}

/// Personality traits (fixed per horse)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    /// How easily spooked (0-1)
    pub nervousness: f32,
    /// Interest in investigating new things (0-1)
    pub curiosity: f32,
    /// Resistance to commands (0-1)
    pub stubbornness: f32,
    /// Bravery in face of threats (0-1)
    pub courage: f32,
    /// Energy level and activity (0-1)
    pub energy: f32,
    /// Preference for other horses (0-1)
    pub sociability: f32,
    /// Tendency toward aggression (0-1)
    pub aggression: f32,
    /// Openness to new experiences (0-1)
    pub adventurous: f32,
}

impl PersonalityTraits {
    /// Generate personality from seed and species temperament
    pub fn generate(seed: u64, temperament: &TemperamentProfile) -> Self {
        // Use seed for deterministic but varied personalities
        let hash = |i: u64| -> f32 {
            let h = seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(i);
            let mixed = (h ^ (h >> 33)).wrapping_mul(0xFF51AFD7ED558CCD);
            let final_hash = (mixed ^ (mixed >> 33)).wrapping_mul(0xC4CEB9FE1A85EC53);
            final_hash as f32 / u64::MAX as f32
        };

        // Generate with variation around species base
        let vary = |base: f32, index: u64| -> f32 {
            let variation = (hash(index) - 0.5) * 0.4; // +/- 20%
            (base + variation).clamp(0.0, 1.0)
        };

        Self {
            nervousness: vary(temperament.base_nervousness, 0),
            curiosity: vary(temperament.curiosity, 1),
            stubbornness: vary(temperament.stubbornness, 2),
            courage: vary(1.0 - temperament.base_nervousness, 3),
            energy: vary(0.6, 4),
            sociability: vary(temperament.sociability, 5),
            aggression: vary(temperament.aggression, 6),
            adventurous: vary(temperament.curiosity * 0.8, 7),
        }
    }
}

/// Memory bank for storing experiences
#[derive(Debug, Default)]
pub struct MemoryBank {
    memories: VecDeque<Memory>,
    max_memories: usize,
}

impl MemoryBank {
    pub fn new() -> Self {
        Self {
            memories: VecDeque::new(),
            max_memories: 50,
        }
    }

    pub fn add(&mut self, memory: Memory) {
        self.memories.push_front(memory);
        if self.memories.len() > self.max_memories {
            self.memories.pop_back();
        }
    }

    pub fn decay(&mut self, dt: f32) {
        for memory in &mut self.memories {
            memory.strength -= dt * 0.001;
        }
        self.memories.retain(|m| m.strength > 0.0);
    }

    /// Get total emotional impact of memories
    pub fn emotional_sum(&self) -> f32 {
        self.memories.iter()
            .map(|m| m.emotional_impact * m.strength)
            .sum()
    }

    /// Count positive interactions with player
    pub fn positive_player_interactions(&self) -> usize {
        self.memories.iter()
            .filter(|m| matches!(m.memory_type, MemoryType::PlayerInteraction(_)) && m.emotional_impact > 0.0)
            .count()
    }
}

/// A single memory
#[derive(Debug, Clone)]
pub struct Memory {
    pub memory_type: MemoryType,
    pub emotional_impact: f32,
    pub strength: f32,
    #[allow(dead_code)]
    pub timestamp: Instant,
}

/// Types of memories horses can form
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryType {
    PlayerInteraction(PlayerInteraction),
    ThreatEncounter(ThreatType),
    LocationDiscovery(LocationType),
    SocialEvent(SocialEvent),
    FoodSource(Vec3),
    WaterSource(Vec3),
}

/// Types of player interactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerInteraction {
    Approached,
    Fed,
    Petted,
    Spooked,
    Attacked,
    Mounted,
    Dismounted,
    TrainedSuccessfully,
    TrainingFailed,
    Groomed,
    Healed,
}

/// Types of threats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatType {
    Predator,
    LoudNoise,
    Fire,
    Storm,
    UnknownCreature,
    HostileHuman,
}

/// Types of locations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationType {
    GoodGrazing,
    WaterSource,
    Shelter,
    DangerZone,
    PlayerHome,
}

/// Social events with other horses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocialEvent {
    MetNewHorse,
    PlayedWithHorse,
    FoughtWithHorse,
    LostHerdMember,
    JoinedHerd,
}

/// Current needs levels
#[derive(Debug, Clone, Default)]
pub struct NeedsSystem {
    /// Hunger level (1.0 = full, 0.0 = starving)
    pub hunger: f32,
    /// Thirst level (1.0 = hydrated, 0.0 = dehydrated)
    pub thirst: f32,
    /// Rest level (1.0 = well rested, 0.0 = exhausted)
    pub rest: f32,
    /// Social need (1.0 = socially fulfilled, 0.0 = lonely)
    pub social: f32,
    /// Exercise/movement need
    pub exercise: f32,
}

impl NeedsSystem {
    pub fn full() -> Self {
        Self {
            hunger: 1.0,
            thirst: 1.0,
            rest: 1.0,
            social: 0.8,
            exercise: 0.7,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Needs decay over time
        self.hunger = (self.hunger - dt * 0.002).max(0.0);
        self.thirst = (self.thirst - dt * 0.003).max(0.0);
        self.rest = (self.rest - dt * 0.001).max(0.0);
        self.social = (self.social - dt * 0.0005).max(0.0);
        self.exercise = (self.exercise - dt * 0.001).max(0.0);
    }

    pub fn feed(&mut self, quality: f32) {
        self.hunger = (self.hunger + quality * 0.4).min(1.0);
    }

    pub fn drink(&mut self) {
        self.thirst = (self.thirst + 0.5).min(1.0);
    }

    pub fn rest_tick(&mut self, dt: f32) {
        self.rest = (self.rest + dt * 0.01).min(1.0);
    }

    pub fn social_tick(&mut self, nearby_horses: usize) {
        if nearby_horses > 0 {
            self.social = (self.social + 0.005 * nearby_horses as f32).min(1.0);
        }
    }
}

/// Current awareness state
#[derive(Debug, Clone, Default)]
pub struct AwarenessState {
    pub player_detected: bool,
    pub player_distance: Option<f32>,
    pub player_familiarity: f32,
    pub nearby_horse_count: usize,
    pub nearest_horse_distance: Option<f32>,
    pub threat_level: f32,
    pub highest_threat: Option<ThreatInfo>,
    pub terrain_type: TerrainType,
    pub weather_severity: f32,
    pub time_of_day: TimeOfDay,
    pub in_safe_zone: bool,
}

/// Information about a detected threat
#[derive(Debug, Clone)]
pub struct ThreatInfo {
    pub threat_type: ThreatType,
    pub position: Vec3,
    pub distance: f32,
    pub danger: f32,
    pub approaching: bool,
}

/// Environmental context
#[derive(Debug, Clone, Default)]
pub struct EnvironmentContext {
    pub terrain_type: TerrainType,
    pub weather_severity: f32,
    pub time_of_day: TimeOfDay,
    pub is_safe_zone: bool,
    pub near_water: bool,
    pub near_food: bool,
}

/// Terrain types for environmental awareness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TerrainType {
    #[default]
    Grassland,
    Forest,
    Beach,
    Marsh,
    Mountain,
    River,
    Road,
}

/// Time of day categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeOfDay {
    Dawn,
    #[default]
    Day,
    Dusk,
    Night,
}

/// Social context with other horses
#[derive(Debug, Clone, Default)]
pub struct SocialContext {
    pub in_herd: bool,
    pub herd_type: Option<HerdType>,
    pub is_lead: bool,
    pub herd_size: usize,
    pub bonded_horse_nearby: bool,
}

/// Current decision/behavioral state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DecisionState {
    #[default]
    Idle,
    Grazing,
    SeekingWater,
    Resting,
    SeekingHerd,
    Alert,
    Assessing,
    PreparingToFlee,
    Fleeing,
    Investigating,
    Following,
    PlayingWithHerd,
    Defending,
}

impl DecisionState {
    /// Get movement speed modifier
    pub fn speed_modifier(&self) -> f32 {
        match self {
            Self::Idle | Self::Resting => 0.0,
            Self::Grazing | Self::SeekingWater => 0.3,
            Self::Alert | Self::Assessing => 0.0,
            Self::Following | Self::Investigating => 0.5,
            Self::PreparingToFlee => 0.2,
            Self::Fleeing => 1.5,
            Self::SeekingHerd => 0.7,
            Self::PlayingWithHerd => 0.8,
            Self::Defending => 0.6,
        }
    }

    /// Check if in active state
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Idle | Self::Resting | Self::Grazing)
    }
}

/// Personality trait identifiers for UI/perks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonalityTrait {
    Nervous,
    Curious,
    Stubborn,
    Courageous,
    Energetic,
    Social,
    Aggressive,
    Adventurous,
    Calm,
    Gentle,
    Wild,
    Loyal,
}

impl PersonalityTrait {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Nervous => "Nervous",
            Self::Curious => "Curious",
            Self::Stubborn => "Stubborn",
            Self::Courageous => "Courageous",
            Self::Energetic => "Energetic",
            Self::Social => "Social",
            Self::Aggressive => "Aggressive",
            Self::Adventurous => "Adventurous",
            Self::Calm => "Calm",
            Self::Gentle => "Gentle",
            Self::Wild => "Wild",
            Self::Loyal => "Loyal",
        }
    }
}
