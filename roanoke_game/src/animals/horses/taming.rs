//! Horse Taming System
//!
//! Multi-stage taming progression for wild horses:
//! 1. Awareness - Horse notices player, evaluates threat
//! 2. Approach - Player slowly approaches, horse decides fight/flight/curiosity
//! 3. Calming - Reducing horse's fear through patient actions
//! 4. Touch - First physical contact, building trust
//! 5. Haltering - Putting on basic restraint
//! 6. Ground Work - Basic training from the ground
//! 7. Saddling - Introducing saddle
//! 8. Mounting - First mount attempts
//! 9. Riding - Basic riding until trust established
//! 10. Bonded - Horse is tamed

use super::encephalon::{PlayerInteraction, EmotionalState};
use super::entity::{Horse, HorseId, OwnershipState};
use super::types::HorseSpecies;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// The current phase of taming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TamingPhase {
    /// Horse is aware of player but not engaged
    Awareness,
    /// Player is approaching the horse
    Approach,
    /// Player is calming the horse
    Calming,
    /// First physical contact phase
    Touch,
    /// Putting halter/lead on horse
    Haltering,
    /// Ground-based training
    GroundWork,
    /// Introducing saddle and bridle
    Saddling,
    /// First mount attempts
    Mounting,
    /// Learning to ride together
    Riding,
    /// Fully tamed and bonded
    Bonded,
}

impl TamingPhase {
    /// Get the next phase in progression
    pub fn next(&self) -> Option<TamingPhase> {
        match self {
            Self::Awareness => Some(Self::Approach),
            Self::Approach => Some(Self::Calming),
            Self::Calming => Some(Self::Touch),
            Self::Touch => Some(Self::Haltering),
            Self::Haltering => Some(Self::GroundWork),
            Self::GroundWork => Some(Self::Saddling),
            Self::Saddling => Some(Self::Mounting),
            Self::Mounting => Some(Self::Riding),
            Self::Riding => Some(Self::Bonded),
            Self::Bonded => None,
        }
    }

    /// Get the required progress to advance (0.0-1.0)
    pub fn advancement_threshold(&self) -> f32 {
        match self {
            Self::Awareness => 0.5,
            Self::Approach => 0.6,
            Self::Calming => 0.7,
            Self::Touch => 0.6,
            Self::Haltering => 0.5,
            Self::GroundWork => 0.8,
            Self::Saddling => 0.6,
            Self::Mounting => 0.7,
            Self::Riding => 0.9,
            Self::Bonded => 1.0,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Awareness => "Awareness",
            Self::Approach => "Approach",
            Self::Calming => "Calming",
            Self::Touch => "First Touch",
            Self::Haltering => "Haltering",
            Self::GroundWork => "Ground Work",
            Self::Saddling => "Saddling",
            Self::Mounting => "First Mount",
            Self::Riding => "Riding",
            Self::Bonded => "Bonded",
        }
    }

    /// Get description for UI
    pub fn description(&self) -> &'static str {
        match self {
            Self::Awareness => "The horse has noticed you. Move slowly and non-threateningly.",
            Self::Approach => "Approach carefully. Stop if the horse shows signs of stress.",
            Self::Calming => "Use calming techniques to reduce the horse's fear.",
            Self::Touch => "Attempt gentle first contact. Build trust through patience.",
            Self::Haltering => "Introduce the halter. This requires significant trust.",
            Self::GroundWork => "Train the horse from the ground. Build respect and communication.",
            Self::Saddling => "Introduce the saddle and bridle carefully.",
            Self::Mounting => "Attempt your first mount. Be ready for resistance.",
            Self::Riding => "Ride together to build the final bond.",
            Self::Bonded => "The horse is fully tamed and trusts you.",
        }
    }
}

/// Actions the player can take during taming
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TamingAction {
    // Approach phase actions
    StandStill,
    CrouchDown,
    TurnSideways,
    WalkSlowly,
    BackAway,

    // Calming phase actions
    SpeakSoftly,
    Whistle,
    OfferHand,
    MakeCalm,

    // Touch phase actions
    ReachOut,
    TouchNeck,
    TouchShoulder,
    StrokeGently,

    // Feeding actions (work in multiple phases)
    OfferFood,
    OfferTreat,
    OfferWater,

    // Haltering actions
    ShowHalter,
    ApproachWithHalter,
    PlaceHalter,

    // Ground work actions
    LeadWalk,
    DirectionChange,
    StopCommand,
    BackUpCommand,
    CircleWork,

    // Saddling actions
    ShowBlanket,
    PlaceBlanket,
    ShowSaddle,
    PlaceSaddle,
    CinchSaddle,

    // Mounting actions
    ApproachMount,
    PutFootInStirrup,
    SwingOver,
    SitDown,

    // Riding actions
    WalkCommand,
    TrotCommand,
    WholeHalt,
    TurnLeft,
    TurnRight,
}

impl TamingAction {
    /// Get the phases where this action is valid
    pub fn valid_phases(&self) -> &'static [TamingPhase] {
        match self {
            // Approach actions
            Self::StandStill | Self::CrouchDown | Self::TurnSideways |
            Self::WalkSlowly | Self::BackAway => &[
                TamingPhase::Awareness, TamingPhase::Approach, TamingPhase::Calming
            ],

            // Calming actions
            Self::SpeakSoftly | Self::Whistle | Self::MakeCalm => &[
                TamingPhase::Approach, TamingPhase::Calming, TamingPhase::Touch
            ],
            Self::OfferHand => &[TamingPhase::Calming, TamingPhase::Touch],

            // Feeding - works in many phases
            Self::OfferFood | Self::OfferTreat | Self::OfferWater => &[
                TamingPhase::Approach, TamingPhase::Calming, TamingPhase::Touch,
                TamingPhase::Haltering, TamingPhase::GroundWork, TamingPhase::Saddling,
            ],

            // Touch actions
            Self::ReachOut | Self::TouchNeck | Self::TouchShoulder |
            Self::StrokeGently => &[TamingPhase::Touch, TamingPhase::Haltering],

            // Haltering
            Self::ShowHalter | Self::ApproachWithHalter | Self::PlaceHalter => &[
                TamingPhase::Haltering
            ],

            // Ground work
            Self::LeadWalk | Self::DirectionChange | Self::StopCommand |
            Self::BackUpCommand | Self::CircleWork => &[
                TamingPhase::GroundWork, TamingPhase::Saddling
            ],

            // Saddling
            Self::ShowBlanket | Self::PlaceBlanket | Self::ShowSaddle |
            Self::PlaceSaddle | Self::CinchSaddle => &[TamingPhase::Saddling],

            // Mounting
            Self::ApproachMount | Self::PutFootInStirrup |
            Self::SwingOver | Self::SitDown => &[TamingPhase::Mounting],

            // Riding
            Self::WalkCommand | Self::TrotCommand | Self::WholeHalt |
            Self::TurnLeft | Self::TurnRight => &[
                TamingPhase::Riding, TamingPhase::Bonded
            ],
        }
    }

    /// Get base progress for this action
    pub fn base_progress(&self) -> f32 {
        match self {
            Self::StandStill => 0.02,
            Self::CrouchDown => 0.04,
            Self::TurnSideways => 0.03,
            Self::WalkSlowly => 0.03,
            Self::BackAway => 0.01,
            Self::SpeakSoftly => 0.04,
            Self::Whistle => 0.03,
            Self::OfferHand => 0.05,
            Self::MakeCalm => 0.06,
            Self::ReachOut => 0.04,
            Self::TouchNeck => 0.08,
            Self::TouchShoulder => 0.06,
            Self::StrokeGently => 0.10,
            Self::OfferFood => 0.12,
            Self::OfferTreat => 0.15,
            Self::OfferWater => 0.08,
            Self::ShowHalter => 0.05,
            Self::ApproachWithHalter => 0.08,
            Self::PlaceHalter => 0.15,
            Self::LeadWalk => 0.06,
            Self::DirectionChange => 0.08,
            Self::StopCommand => 0.07,
            Self::BackUpCommand => 0.09,
            Self::CircleWork => 0.10,
            Self::ShowBlanket => 0.05,
            Self::PlaceBlanket => 0.10,
            Self::ShowSaddle => 0.06,
            Self::PlaceSaddle => 0.12,
            Self::CinchSaddle => 0.08,
            Self::ApproachMount => 0.05,
            Self::PutFootInStirrup => 0.08,
            Self::SwingOver => 0.12,
            Self::SitDown => 0.10,
            Self::WalkCommand => 0.08,
            Self::TrotCommand => 0.10,
            Self::WholeHalt => 0.06,
            Self::TurnLeft | Self::TurnRight => 0.05,
        }
    }

    /// Get required distance for this action (-1 = any)
    pub fn required_distance(&self) -> (f32, f32) {
        match self {
            // Far actions
            Self::StandStill | Self::BackAway => (10.0, 50.0),
            Self::WalkSlowly | Self::CrouchDown | Self::TurnSideways => (5.0, 30.0),
            Self::SpeakSoftly | Self::Whistle => (3.0, 25.0),

            // Medium actions
            Self::OfferFood | Self::OfferTreat | Self::OfferWater => (2.0, 8.0),
            Self::OfferHand | Self::MakeCalm => (2.0, 6.0),

            // Close actions
            Self::ReachOut | Self::TouchNeck | Self::TouchShoulder |
            Self::StrokeGently => (0.5, 3.0),
            Self::ShowHalter | Self::ApproachWithHalter => (1.0, 4.0),
            Self::PlaceHalter => (0.5, 2.0),

            // Ground work needs lead rope length
            Self::LeadWalk | Self::DirectionChange | Self::StopCommand |
            Self::BackUpCommand | Self::CircleWork => (2.0, 10.0),

            // Saddling
            Self::ShowBlanket | Self::ShowSaddle => (1.0, 4.0),
            Self::PlaceBlanket | Self::PlaceSaddle | Self::CinchSaddle => (0.5, 2.0),

            // Mounting (must be right next to horse)
            Self::ApproachMount | Self::PutFootInStirrup |
            Self::SwingOver | Self::SitDown => (0.0, 2.0),

            // Riding (mounted, distance irrelevant)
            Self::WalkCommand | Self::TrotCommand | Self::WholeHalt |
            Self::TurnLeft | Self::TurnRight => (0.0, 0.0),
        }
    }
}

/// Progress tracking for taming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamingProgress {
    /// Current phase
    pub phase: TamingPhase,
    /// Progress within current phase (0.0-1.0)
    pub phase_progress: f32,
    /// Overall taming progress (0.0-1.0)
    pub total_progress: f32,
    /// Trust accumulated
    pub trust: f32,
    /// Respect accumulated
    pub respect: f32,
    /// Number of successful interactions
    pub successful_interactions: u32,
    /// Number of failed interactions
    pub failed_interactions: u32,
    /// Time spent taming
    pub time_spent: f32,
    /// Phase-specific data
    pub halter_placed: bool,
    pub saddle_placed: bool,
    pub mounted_successfully: bool,
    /// Timestamp of start
    #[serde(skip)]
    pub started: Option<Instant>,
    /// Last interaction time
    #[serde(skip)]
    pub last_interaction: Option<Instant>,
}

impl Default for TamingProgress {
    fn default() -> Self {
        Self {
            phase: TamingPhase::Awareness,
            phase_progress: 0.0,
            total_progress: 0.0,
            trust: 0.0,
            respect: 0.0,
            successful_interactions: 0,
            failed_interactions: 0,
            time_spent: 0.0,
            halter_placed: false,
            saddle_placed: false,
            mounted_successfully: false,
            started: Some(Instant::now()),
            last_interaction: None,
        }
    }
}

impl TamingProgress {
    /// Check if taming is complete
    pub fn is_complete(&self) -> bool {
        self.phase == TamingPhase::Bonded
    }

    /// Advance to next phase if ready
    pub fn try_advance(&mut self) -> bool {
        if self.phase_progress >= self.phase.advancement_threshold() {
            if let Some(next) = self.phase.next() {
                self.phase = next;
                self.phase_progress = 0.0;
                return true;
            }
        }
        false
    }

    /// Add progress to current phase
    pub fn add_progress(&mut self, amount: f32, trust_gain: f32, respect_gain: f32) {
        self.phase_progress = (self.phase_progress + amount).min(1.0);
        self.trust = (self.trust + trust_gain).min(1.0);
        self.respect = (self.respect + respect_gain).min(1.0);
        self.successful_interactions += 1;
        self.last_interaction = Some(Instant::now());

        // Update total progress
        let phase_weight = match self.phase {
            TamingPhase::Awareness => 0.05,
            TamingPhase::Approach => 0.08,
            TamingPhase::Calming => 0.10,
            TamingPhase::Touch => 0.10,
            TamingPhase::Haltering => 0.10,
            TamingPhase::GroundWork => 0.15,
            TamingPhase::Saddling => 0.12,
            TamingPhase::Mounting => 0.12,
            TamingPhase::Riding => 0.13,
            TamingPhase::Bonded => 0.05,
        };
        self.total_progress = (self.total_progress + amount * phase_weight).min(1.0);
    }

    /// Record a failed interaction
    pub fn record_failure(&mut self, setback: f32) {
        self.phase_progress = (self.phase_progress - setback).max(0.0);
        self.trust = (self.trust - setback * 0.5).max(0.0);
        self.failed_interactions += 1;
        self.last_interaction = Some(Instant::now());
    }
}

/// Result of a taming action
#[derive(Debug, Clone)]
pub enum TamingResult {
    /// Action succeeded
    Success {
        progress_gained: f32,
        message: &'static str,
        phase_advanced: bool,
    },
    /// Action failed
    Failed {
        reason: &'static str,
        setback: f32,
    },
    /// Horse fled
    HorseFled {
        reason: &'static str,
    },
    /// Horse attacked
    HorseAttacked {
        damage: f32,
    },
    /// Wrong phase for this action
    WrongPhase {
        current: TamingPhase,
        required: &'static [TamingPhase],
    },
    /// Too far/close for action
    WrongDistance {
        current: f32,
        required: (f32, f32),
    },
    /// Taming complete!
    Complete,
}

/// The taming system manager
#[derive(Debug)]
pub struct HorseTamingSystem {
    /// Player skill modifiers
    pub player_skill: f32,
    /// Patience bonus (standing still duration)
    pub patience_accumulated: f32,
}

impl Default for HorseTamingSystem {
    fn default() -> Self {
        Self {
            player_skill: 1.0,
            patience_accumulated: 0.0,
        }
    }
}

impl HorseTamingSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start taming a wild horse
    pub fn start_taming(&self, horse: &mut Horse) -> bool {
        if horse.ownership != OwnershipState::Wild {
            return false;
        }

        horse.taming_progress = Some(TamingProgress::default());
        horse.ownership = OwnershipState::BeingTamed;
        true
    }

    /// Attempt a taming action
    pub fn attempt_action(
        &mut self,
        horse: &mut Horse,
        action: TamingAction,
        player_pos: Vec3,
    ) -> TamingResult {
        // First check if horse is being tamed and get needed info
        let (current_phase, interactions) = match &horse.taming_progress {
            Some(p) => (p.phase, p.successful_interactions + p.failed_interactions),
            None => return TamingResult::Failed {
                reason: "Horse is not being tamed",
                setback: 0.0,
            },
        };

        // Check if action is valid for current phase
        let valid_phases = action.valid_phases();
        if !valid_phases.contains(&current_phase) {
            return TamingResult::WrongPhase {
                current: current_phase,
                required: valid_phases,
            };
        }

        // Check distance
        let distance = horse.position.distance(player_pos);
        let (min_dist, max_dist) = action.required_distance();
        if max_dist > 0.0 && (distance < min_dist || distance > max_dist) {
            return TamingResult::WrongDistance {
                current: distance,
                required: (min_dist, max_dist),
            };
        }

        // Calculate success based on horse state and player skill
        let success_chance = self.calculate_success_chance(horse, action);

        // Use deterministic check based on horse id and interaction count
        let roll = deterministic_roll(horse.id.0, interactions as u64);

        if roll < success_chance {
            // Success!
            self.apply_success(horse, action)
        } else {
            // Failure
            self.apply_failure(horse, action)
        }
    }

    /// Calculate success chance for an action
    fn calculate_success_chance(&self, horse: &Horse, action: TamingAction) -> f32 {
        let progress = horse.taming_progress.as_ref().unwrap();
        let mut chance = 0.5;

        // Player skill bonus
        chance += self.player_skill * 0.1;

        // Trust bonus
        chance += progress.trust * 0.2;

        // Patience bonus
        chance += (self.patience_accumulated * 0.1).min(0.15);

        // Horse emotional state penalty
        match horse.encephalon.emotional_state {
            EmotionalState::Panicked => chance -= 0.4,
            EmotionalState::Frightened => chance -= 0.25,
            EmotionalState::Nervous => chance -= 0.15,
            EmotionalState::Alert => chance -= 0.05,
            EmotionalState::Curious => chance += 0.1,
            EmotionalState::Calm | EmotionalState::Content => chance += 0.15,
            _ => {}
        }

        // Species difficulty modifier
        chance -= horse.species.taming_difficulty() * 0.2;

        // Personality modifiers
        chance -= horse.encephalon.personality.stubbornness * 0.1;
        chance -= horse.encephalon.personality.nervousness * 0.1;
        chance += horse.encephalon.personality.curiosity * 0.05;

        // Action-specific modifiers
        match action {
            TamingAction::OfferFood | TamingAction::OfferTreat => chance += 0.15,
            TamingAction::CrouchDown => chance += 0.1,
            TamingAction::SwingOver | TamingAction::PlaceHalter => chance -= 0.1,
            _ => {}
        }

        chance.clamp(0.1, 0.95)
    }

    /// Apply successful action effects
    fn apply_success(&mut self, horse: &mut Horse, action: TamingAction) -> TamingResult {
        let progress = horse.taming_progress.as_mut().unwrap();

        let base_progress = action.base_progress();
        let trust_gain = base_progress * 0.5;
        let respect_gain = base_progress * 0.3;

        // Apply progress
        progress.add_progress(base_progress, trust_gain, respect_gain);

        // Record positive interaction in encephalon
        let interaction = match action {
            TamingAction::OfferFood | TamingAction::OfferTreat | TamingAction::OfferWater
                => PlayerInteraction::Fed,
            TamingAction::TouchNeck | TamingAction::TouchShoulder | TamingAction::StrokeGently
                => PlayerInteraction::Petted,
            TamingAction::SwingOver | TamingAction::SitDown
                => PlayerInteraction::Mounted,
            _ => PlayerInteraction::Approached,
        };
        horse.encephalon.record_player_interaction(interaction, true);

        // Update horse trust
        horse.trust_level = (horse.trust_level + trust_gain).min(1.0);
        horse.respect_level = (horse.respect_level + respect_gain).min(1.0);

        // Check for phase-specific state changes
        match action {
            TamingAction::PlaceHalter => {
                progress.halter_placed = true;
            }
            TamingAction::PlaceSaddle | TamingAction::CinchSaddle => {
                progress.saddle_placed = true;
            }
            TamingAction::SitDown => {
                progress.mounted_successfully = true;
            }
            _ => {}
        }

        // Try to advance phase
        let phase_advanced = progress.try_advance();

        // Check for completion
        if progress.phase == TamingPhase::Bonded {
            horse.ownership = OwnershipState::Owned;
            horse.bond_level = progress.trust;
            return TamingResult::Complete;
        }

        let message = match action {
            TamingAction::OfferFood => "The horse accepts the food gratefully.",
            TamingAction::TouchNeck => "The horse allows your touch.",
            TamingAction::PlaceHalter => "The halter is in place!",
            TamingAction::SitDown => "You're mounted! The horse accepts you.",
            TamingAction::CrouchDown => "The horse seems less threatened.",
            TamingAction::WalkCommand => "The horse responds to your command.",
            _ => "The horse responds positively.",
        };

        TamingResult::Success {
            progress_gained: base_progress,
            message,
            phase_advanced,
        }
    }

    /// Apply failure effects
    fn apply_failure(&mut self, horse: &mut Horse, action: TamingAction) -> TamingResult {
        let progress = horse.taming_progress.as_mut().unwrap();

        // Determine severity of failure
        let setback = match action {
            TamingAction::SwingOver | TamingAction::PlaceHalter => 0.15,
            TamingAction::TouchNeck | TamingAction::TouchShoulder => 0.08,
            TamingAction::WalkSlowly => 0.03,
            _ => 0.05,
        };

        progress.record_failure(setback);

        // Record negative interaction
        let interaction = match action {
            TamingAction::SwingOver => PlayerInteraction::Mounted,
            _ => PlayerInteraction::Spooked,
        };
        horse.encephalon.record_player_interaction(interaction, false);

        // Update horse trust
        horse.trust_level = (horse.trust_level - setback * 0.5).max(0.0);

        // Emotional impact
        horse.encephalon.emotion_weights.fear += setback;
        horse.encephalon.emotion_weights.trust =
            (horse.encephalon.emotion_weights.trust - setback * 0.3).max(0.0);

        // Check if horse flees or attacks
        if horse.encephalon.emotion_weights.fear > 0.8 {
            // Reset to being tamed but horse fled
            horse.ownership = OwnershipState::Wild;
            horse.taming_progress = None;
            return TamingResult::HorseFled {
                reason: "The horse was too frightened and fled!",
            };
        }

        if horse.encephalon.personality.aggression > 0.6 && setback > 0.1 {
            return TamingResult::HorseAttacked {
                damage: 15.0 + horse.encephalon.personality.aggression * 20.0,
            };
        }

        let reason = match action {
            TamingAction::SwingOver => "The horse sidesteps and you can't mount.",
            TamingAction::PlaceHalter => "The horse tosses its head, rejecting the halter.",
            TamingAction::TouchNeck => "The horse shies away from your touch.",
            _ => "The horse is uncooperative.",
        };

        TamingResult::Failed { reason, setback }
    }

    /// Update patience bonus when standing still
    pub fn update_patience(&mut self, dt: f32, player_moving: bool) {
        if player_moving {
            self.patience_accumulated = (self.patience_accumulated - dt * 0.5).max(0.0);
        } else {
            self.patience_accumulated = (self.patience_accumulated + dt * 0.1).min(1.0);
        }
    }
}

/// Deterministic roll based on seed
fn deterministic_roll(seed: u64, iteration: u64) -> f32 {
    let combined = seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(iteration);
    let hash = (combined ^ (combined >> 33)).wrapping_mul(0xFF51AFD7ED558CCD);
    let final_hash = (hash ^ (hash >> 33)).wrapping_mul(0xC4CEB9FE1A85EC53);
    final_hash as f32 / u64::MAX as f32
}
