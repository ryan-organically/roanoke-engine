//! Utility AI System for NPC Decision Making
//!
//! Implements a utility-based decision system where NPCs evaluate
//! available actions and choose the one with highest utility score.
//!
//! # Architecture
//! - `Consideration`: A factor that influences action desirability
//! - `Action`: A potential behavior with associated considerations
//! - `UtilityEvaluator`: Scores actions and selects the best one
//!
//! # Example Flow
//! 1. NPC gathers context (player distance, relationship, time, etc.)
//! 2. Each available action is scored using weighted considerations
//! 3. Highest-scoring action is selected
//! 4. Action maps to NpcBehaviorState transition

use super::npc_manager::{NpcBehaviorState, NpcInstance, NpcRole};
use super::relationships::{NpcRelationship, RelationshipType};
use glam::Vec3;

/// Context for evaluating utility - gathered once per decision cycle
#[derive(Debug, Clone)]
pub struct UtilityContext {
    // Spatial
    pub player_distance: f32,
    pub target_distance: f32,
    pub home_distance: f32,

    // Temporal
    pub current_hour: f32,
    pub is_night: bool,

    // State
    pub health_percent: f32,
    pub alertness: f32,       // 0.0 - 1.0
    pub mood: f32,            // -1.0 to 1.0 (normalized from -100 to 100)

    // Relationship
    pub player_affinity: f32,  // -1.0 to 1.0
    pub player_trust: f32,     // -1.0 to 1.0
    pub player_fear: f32,      // 0.0 to 1.0
    pub relationship_type: RelationshipType,

    // Activity
    pub scheduled_activity: ScheduledActivity,
    pub at_activity_location: bool,

    // Recent events
    pub recently_attacked: bool,
    pub recently_traded: bool,
    pub heard_rumors: bool,
}

/// Simplified activity types for utility calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduledActivity {
    Working,
    Resting,
    Socializing,
    Trading,
    Patrolling,
    Praying,
    None,
}

impl From<super::relationships::NpcActivity> for ScheduledActivity {
    fn from(activity: super::relationships::NpcActivity) -> Self {
        use super::relationships::NpcActivity;
        match activity {
            NpcActivity::Working | NpcActivity::Crafting |
            NpcActivity::Gathering | NpcActivity::Teaching => ScheduledActivity::Working,
            NpcActivity::Sleeping | NpcActivity::Resting | NpcActivity::Eating => ScheduledActivity::Resting,
            NpcActivity::Socializing => ScheduledActivity::Socializing,
            NpcActivity::Trading => ScheduledActivity::Trading,
            NpcActivity::Patrolling => ScheduledActivity::Patrolling,
            NpcActivity::Praying => ScheduledActivity::Praying,
        }
    }
}

/// Available actions an NPC can take
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcAction {
    // Basic states
    Idle,
    WalkToTarget,
    WorkAtLocation,

    // Player interaction
    GreetPlayer,
    ApproachPlayer,
    TradeWithPlayer,
    FleeFromPlayer,
    AttackPlayer,

    // Alert/defensive
    BecomeAlert,
    Investigate,
    ReturnHome,

    // Social
    Socialize,
    ShareGossip,
}

impl NpcAction {
    /// Map action to behavior state
    pub fn to_behavior_state(&self) -> NpcBehaviorState {
        match self {
            NpcAction::Idle => NpcBehaviorState::Idle,
            NpcAction::WalkToTarget => NpcBehaviorState::Walking,
            NpcAction::WorkAtLocation => NpcBehaviorState::Working,
            NpcAction::GreetPlayer => NpcBehaviorState::Greeting,
            NpcAction::ApproachPlayer => NpcBehaviorState::Walking,
            NpcAction::TradeWithPlayer => NpcBehaviorState::Trading,
            NpcAction::FleeFromPlayer => NpcBehaviorState::Fleeing,
            NpcAction::AttackPlayer => NpcBehaviorState::Attacking,
            NpcAction::BecomeAlert => NpcBehaviorState::Alert,
            NpcAction::Investigate => NpcBehaviorState::Walking,
            NpcAction::ReturnHome => NpcBehaviorState::Walking,
            NpcAction::Socialize => NpcBehaviorState::Idle,
            NpcAction::ShareGossip => NpcBehaviorState::Idle,
        }
    }

    /// Get all standard actions available to NPCs
    pub fn standard_actions() -> &'static [NpcAction] {
        &[
            NpcAction::Idle,
            NpcAction::WalkToTarget,
            NpcAction::WorkAtLocation,
            NpcAction::GreetPlayer,
            NpcAction::ApproachPlayer,
            NpcAction::TradeWithPlayer,
            NpcAction::FleeFromPlayer,
            NpcAction::AttackPlayer,
            NpcAction::BecomeAlert,
            NpcAction::Investigate,
            NpcAction::ReturnHome,
            NpcAction::Socialize,
            NpcAction::ShareGossip,
        ]
    }
}

/// A consideration is a factor that affects action utility
/// Each consideration returns a value from 0.0 to 1.0
#[derive(Debug, Clone, Copy)]
pub enum Consideration {
    // Distance-based
    PlayerNearby,           // High when player is close
    PlayerFar,              // High when player is far
    AtTargetLocation,       // High when at scheduled location
    AwayFromHome,           // High when far from home

    // Relationship-based
    PlayerIsFriend,         // High for positive relationships
    PlayerIsEnemy,          // High for negative relationships
    PlayerIsFeared,         // High when fearing player
    PlayerIsTrusted,        // High when trusting player

    // State-based
    IsAlert,                // High when alertness is high
    IsCalm,                 // High when alertness is low
    HealthLow,              // High when health is critical
    MoodPositive,           // High when mood is good
    MoodNegative,           // High when mood is bad

    // Schedule-based
    ShouldBeWorking,        // High during work hours
    ShouldBeResting,        // High during rest hours
    ShouldBeTrading,        // High during trade hours
    ShouldBeSocializing,    // High during social hours

    // Temporal
    IsDaytime,              // High during day
    IsNighttime,            // High during night

    // Events
    WasRecentlyAttacked,    // High if recently took damage
    HasHeardRumors,         // High if gossip available
}

impl Consideration {
    /// Evaluate this consideration given the context
    /// Returns a value from 0.0 to 1.0
    pub fn evaluate(&self, ctx: &UtilityContext) -> f32 {
        match self {
            // Distance considerations
            Consideration::PlayerNearby => {
                // Peaks at distance 0, drops off at 20 units
                (1.0 - (ctx.player_distance / 20.0).min(1.0)).max(0.0)
            }
            Consideration::PlayerFar => {
                // Low when close, high when far (>30 units)
                ((ctx.player_distance - 10.0) / 30.0).clamp(0.0, 1.0)
            }
            Consideration::AtTargetLocation => {
                if ctx.at_activity_location { 1.0 } else { 0.0 }
            }
            Consideration::AwayFromHome => {
                (ctx.home_distance / 50.0).clamp(0.0, 1.0)
            }

            // Relationship considerations
            Consideration::PlayerIsFriend => {
                ((ctx.player_affinity + 1.0) / 2.0).clamp(0.0, 1.0)
            }
            Consideration::PlayerIsEnemy => {
                ((1.0 - ctx.player_affinity) / 2.0).clamp(0.0, 1.0)
            }
            Consideration::PlayerIsFeared => {
                ctx.player_fear
            }
            Consideration::PlayerIsTrusted => {
                ((ctx.player_trust + 1.0) / 2.0).clamp(0.0, 1.0)
            }

            // State considerations
            Consideration::IsAlert => ctx.alertness,
            Consideration::IsCalm => 1.0 - ctx.alertness,
            Consideration::HealthLow => {
                (1.0 - ctx.health_percent).clamp(0.0, 1.0)
            }
            Consideration::MoodPositive => {
                ((ctx.mood + 1.0) / 2.0).clamp(0.0, 1.0)
            }
            Consideration::MoodNegative => {
                ((1.0 - ctx.mood) / 2.0).clamp(0.0, 1.0)
            }

            // Schedule considerations
            Consideration::ShouldBeWorking => {
                if ctx.scheduled_activity == ScheduledActivity::Working { 1.0 } else { 0.0 }
            }
            Consideration::ShouldBeResting => {
                if ctx.scheduled_activity == ScheduledActivity::Resting { 1.0 } else { 0.0 }
            }
            Consideration::ShouldBeTrading => {
                if ctx.scheduled_activity == ScheduledActivity::Trading { 1.0 } else { 0.0 }
            }
            Consideration::ShouldBeSocializing => {
                if ctx.scheduled_activity == ScheduledActivity::Socializing { 1.0 } else { 0.0 }
            }

            // Temporal considerations
            Consideration::IsDaytime => {
                if ctx.current_hour >= 6.0 && ctx.current_hour < 20.0 { 1.0 } else { 0.0 }
            }
            Consideration::IsNighttime => {
                if ctx.current_hour < 6.0 || ctx.current_hour >= 20.0 { 1.0 } else { 0.0 }
            }

            // Event considerations
            Consideration::WasRecentlyAttacked => {
                if ctx.recently_attacked { 1.0 } else { 0.0 }
            }
            Consideration::HasHeardRumors => {
                if ctx.heard_rumors { 1.0 } else { 0.0 }
            }
        }
    }
}

/// A weighted consideration for scoring
#[derive(Debug, Clone, Copy)]
pub struct WeightedConsideration {
    pub consideration: Consideration,
    pub weight: f32,
    pub curve: ResponseCurve,
}

/// Response curve transforms raw consideration values
#[derive(Debug, Clone, Copy)]
pub enum ResponseCurve {
    Linear,           // y = x
    Quadratic,        // y = x^2 (slow start, fast end)
    InverseQuadratic, // y = 1 - (1-x)^2 (fast start, slow end)
    Sigmoid,          // S-curve, smooth transitions
    Threshold(f32),   // Binary: 0 below threshold, 1 above
}

impl ResponseCurve {
    pub fn apply(&self, value: f32) -> f32 {
        let v = value.clamp(0.0, 1.0);
        match self {
            ResponseCurve::Linear => v,
            ResponseCurve::Quadratic => v * v,
            ResponseCurve::InverseQuadratic => 1.0 - (1.0 - v) * (1.0 - v),
            ResponseCurve::Sigmoid => {
                // Simple sigmoid approximation
                let x = (v - 0.5) * 6.0; // Scale to [-3, 3]
                1.0 / (1.0 + (-x).exp())
            }
            ResponseCurve::Threshold(t) => {
                if v >= *t { 1.0 } else { 0.0 }
            }
        }
    }
}

/// Action profile defines how an action is scored
pub struct ActionProfile {
    pub action: NpcAction,
    pub base_score: f32,
    pub considerations: Vec<WeightedConsideration>,
    /// Roles that can perform this action (empty = all roles)
    pub allowed_roles: Vec<NpcRole>,
    /// Minimum score threshold to be considered
    pub min_threshold: f32,
}

impl ActionProfile {
    /// Evaluate the total utility score for this action
    pub fn evaluate(&self, ctx: &UtilityContext) -> f32 {
        let mut score = self.base_score;

        for wc in &self.considerations {
            let raw_value = wc.consideration.evaluate(ctx);
            let curved_value = wc.curve.apply(raw_value);
            score *= 1.0 + (curved_value - 0.5) * wc.weight;
        }

        score.max(0.0)
    }

    /// Check if this action is available for a given role
    pub fn is_available_for_role(&self, role: NpcRole) -> bool {
        self.allowed_roles.is_empty() || self.allowed_roles.contains(&role)
    }
}

/// The main utility evaluator
pub struct UtilityEvaluator {
    pub profiles: Vec<ActionProfile>,
}

impl UtilityEvaluator {
    /// Create a default evaluator with standard action profiles
    pub fn new() -> Self {
        Self {
            profiles: Self::default_profiles(),
        }
    }

    /// Evaluate all actions and return the best one
    pub fn select_action(&self, ctx: &UtilityContext, role: NpcRole) -> (NpcAction, f32) {
        let mut best_action = NpcAction::Idle;
        let mut best_score = 0.0;

        for profile in &self.profiles {
            if !profile.is_available_for_role(role) {
                continue;
            }

            let score = profile.evaluate(ctx);

            if score > profile.min_threshold && score > best_score {
                best_score = score;
                best_action = profile.action;
            }
        }

        (best_action, best_score)
    }

    /// Get all action scores for debugging
    pub fn evaluate_all(&self, ctx: &UtilityContext, role: NpcRole) -> Vec<(NpcAction, f32)> {
        self.profiles
            .iter()
            .filter(|p| p.is_available_for_role(role))
            .map(|p| (p.action, p.evaluate(ctx)))
            .collect()
    }

    /// Create default action profiles
    fn default_profiles() -> Vec<ActionProfile> {
        vec![
            // Idle - fallback action
            ActionProfile {
                action: NpcAction::Idle,
                base_score: 0.3,
                considerations: vec![
                    WeightedConsideration {
                        consideration: Consideration::IsCalm,
                        weight: 0.5,
                        curve: ResponseCurve::Linear,
                    },
                    WeightedConsideration {
                        consideration: Consideration::AtTargetLocation,
                        weight: 0.3,
                        curve: ResponseCurve::Linear,
                    },
                ],
                allowed_roles: vec![],
                min_threshold: 0.0,
            },

            // Walk to target - schedule adherence
            ActionProfile {
                action: NpcAction::WalkToTarget,
                base_score: 0.5,
                considerations: vec![
                    WeightedConsideration {
                        consideration: Consideration::AtTargetLocation,
                        weight: -1.0, // Don't walk if already there
                        curve: ResponseCurve::Linear,
                    },
                    WeightedConsideration {
                        consideration: Consideration::IsCalm,
                        weight: 0.3,
                        curve: ResponseCurve::Linear,
                    },
                ],
                allowed_roles: vec![],
                min_threshold: 0.2,
            },

            // Work at location
            ActionProfile {
                action: NpcAction::WorkAtLocation,
                base_score: 0.6,
                considerations: vec![
                    WeightedConsideration {
                        consideration: Consideration::ShouldBeWorking,
                        weight: 1.5,
                        curve: ResponseCurve::Threshold(0.5),
                    },
                    WeightedConsideration {
                        consideration: Consideration::AtTargetLocation,
                        weight: 1.0,
                        curve: ResponseCurve::Threshold(0.5),
                    },
                    WeightedConsideration {
                        consideration: Consideration::IsCalm,
                        weight: 0.3,
                        curve: ResponseCurve::Linear,
                    },
                ],
                allowed_roles: vec![],
                min_threshold: 0.4,
            },

            // Greet player - friendly interaction
            ActionProfile {
                action: NpcAction::GreetPlayer,
                base_score: 0.4,
                considerations: vec![
                    WeightedConsideration {
                        consideration: Consideration::PlayerNearby,
                        weight: 1.5,
                        curve: ResponseCurve::InverseQuadratic,
                    },
                    WeightedConsideration {
                        consideration: Consideration::PlayerIsFriend,
                        weight: 1.2,
                        curve: ResponseCurve::Sigmoid,
                    },
                    WeightedConsideration {
                        consideration: Consideration::MoodPositive,
                        weight: 0.5,
                        curve: ResponseCurve::Linear,
                    },
                    WeightedConsideration {
                        consideration: Consideration::IsCalm,
                        weight: 0.4,
                        curve: ResponseCurve::Linear,
                    },
                ],
                allowed_roles: vec![],
                min_threshold: 0.3,
            },

            // Trade with player
            ActionProfile {
                action: NpcAction::TradeWithPlayer,
                base_score: 0.5,
                considerations: vec![
                    WeightedConsideration {
                        consideration: Consideration::PlayerNearby,
                        weight: 1.2,
                        curve: ResponseCurve::Threshold(0.3),
                    },
                    WeightedConsideration {
                        consideration: Consideration::ShouldBeTrading,
                        weight: 1.5,
                        curve: ResponseCurve::Threshold(0.5),
                    },
                    WeightedConsideration {
                        consideration: Consideration::PlayerIsTrusted,
                        weight: 0.8,
                        curve: ResponseCurve::Linear,
                    },
                    WeightedConsideration {
                        consideration: Consideration::PlayerIsEnemy,
                        weight: -1.5,
                        curve: ResponseCurve::Threshold(0.7),
                    },
                ],
                allowed_roles: vec![NpcRole::Trader, NpcRole::Hunter, NpcRole::Shaman, NpcRole::Elder],
                min_threshold: 0.4,
            },

            // Flee from player - self preservation
            ActionProfile {
                action: NpcAction::FleeFromPlayer,
                base_score: 0.2,
                considerations: vec![
                    WeightedConsideration {
                        consideration: Consideration::PlayerNearby,
                        weight: 1.0,
                        curve: ResponseCurve::InverseQuadratic,
                    },
                    WeightedConsideration {
                        consideration: Consideration::WasRecentlyAttacked,
                        weight: 2.0,
                        curve: ResponseCurve::Threshold(0.5),
                    },
                    WeightedConsideration {
                        consideration: Consideration::HealthLow,
                        weight: 1.5,
                        curve: ResponseCurve::Quadratic,
                    },
                    WeightedConsideration {
                        consideration: Consideration::PlayerIsFeared,
                        weight: 1.2,
                        curve: ResponseCurve::Sigmoid,
                    },
                    WeightedConsideration {
                        consideration: Consideration::PlayerIsEnemy,
                        weight: 0.8,
                        curve: ResponseCurve::Linear,
                    },
                ],
                allowed_roles: vec![],
                min_threshold: 0.3,
            },

            // Attack player - defensive warriors only
            ActionProfile {
                action: NpcAction::AttackPlayer,
                base_score: 0.3,
                considerations: vec![
                    WeightedConsideration {
                        consideration: Consideration::PlayerNearby,
                        weight: 1.2,
                        curve: ResponseCurve::Threshold(0.4),
                    },
                    WeightedConsideration {
                        consideration: Consideration::WasRecentlyAttacked,
                        weight: 2.0,
                        curve: ResponseCurve::Threshold(0.5),
                    },
                    WeightedConsideration {
                        consideration: Consideration::PlayerIsEnemy,
                        weight: 1.5,
                        curve: ResponseCurve::Sigmoid,
                    },
                    WeightedConsideration {
                        consideration: Consideration::HealthLow,
                        weight: -1.0, // Don't attack when low health
                        curve: ResponseCurve::Threshold(0.5),
                    },
                ],
                allowed_roles: vec![NpcRole::Warrior, NpcRole::Hunter],
                min_threshold: 0.5,
            },

            // Become alert
            ActionProfile {
                action: NpcAction::BecomeAlert,
                base_score: 0.3,
                considerations: vec![
                    WeightedConsideration {
                        consideration: Consideration::PlayerNearby,
                        weight: 0.8,
                        curve: ResponseCurve::Linear,
                    },
                    WeightedConsideration {
                        consideration: Consideration::IsCalm,
                        weight: 0.5, // More likely when calm (transition to alert)
                        curve: ResponseCurve::Linear,
                    },
                    WeightedConsideration {
                        consideration: Consideration::PlayerIsEnemy,
                        weight: 1.0,
                        curve: ResponseCurve::Sigmoid,
                    },
                    WeightedConsideration {
                        consideration: Consideration::IsNighttime,
                        weight: 0.4,
                        curve: ResponseCurve::Linear,
                    },
                ],
                allowed_roles: vec![],
                min_threshold: 0.25,
            },

            // Return home
            ActionProfile {
                action: NpcAction::ReturnHome,
                base_score: 0.35,
                considerations: vec![
                    WeightedConsideration {
                        consideration: Consideration::AwayFromHome,
                        weight: 1.0,
                        curve: ResponseCurve::Sigmoid,
                    },
                    WeightedConsideration {
                        consideration: Consideration::ShouldBeResting,
                        weight: 1.5,
                        curve: ResponseCurve::Threshold(0.5),
                    },
                    WeightedConsideration {
                        consideration: Consideration::IsNighttime,
                        weight: 0.8,
                        curve: ResponseCurve::Linear,
                    },
                    WeightedConsideration {
                        consideration: Consideration::HealthLow,
                        weight: 0.6,
                        curve: ResponseCurve::Quadratic,
                    },
                ],
                allowed_roles: vec![],
                min_threshold: 0.3,
            },

            // Socialize
            ActionProfile {
                action: NpcAction::Socialize,
                base_score: 0.35,
                considerations: vec![
                    WeightedConsideration {
                        consideration: Consideration::ShouldBeSocializing,
                        weight: 1.5,
                        curve: ResponseCurve::Threshold(0.5),
                    },
                    WeightedConsideration {
                        consideration: Consideration::MoodPositive,
                        weight: 0.6,
                        curve: ResponseCurve::Linear,
                    },
                    WeightedConsideration {
                        consideration: Consideration::IsCalm,
                        weight: 0.4,
                        curve: ResponseCurve::Linear,
                    },
                ],
                allowed_roles: vec![],
                min_threshold: 0.3,
            },

            // Share gossip
            ActionProfile {
                action: NpcAction::ShareGossip,
                base_score: 0.25,
                considerations: vec![
                    WeightedConsideration {
                        consideration: Consideration::HasHeardRumors,
                        weight: 1.5,
                        curve: ResponseCurve::Threshold(0.5),
                    },
                    WeightedConsideration {
                        consideration: Consideration::ShouldBeSocializing,
                        weight: 1.0,
                        curve: ResponseCurve::Sigmoid,
                    },
                    WeightedConsideration {
                        consideration: Consideration::PlayerNearby,
                        weight: -0.5, // Don't gossip when player is near
                        curve: ResponseCurve::Threshold(0.5),
                    },
                ],
                allowed_roles: vec![],
                min_threshold: 0.3,
            },
        ]
    }
}

impl Default for UtilityEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Build UtilityContext from NPC state
pub fn build_context(
    npc: &NpcInstance,
    relationship: Option<&NpcRelationship>,
    player_pos: Vec3,
    current_hour: f32,
    recently_attacked: bool,
) -> UtilityContext {
    let rel = relationship.cloned().unwrap_or_default();

    UtilityContext {
        player_distance: npc.position.distance(player_pos),
        target_distance: npc.target.map(|t| npc.position.distance(t)).unwrap_or(0.0),
        home_distance: npc.position.distance(npc.home_position),
        current_hour,
        is_night: current_hour < 6.0 || current_hour >= 20.0,
        health_percent: npc.health / npc.max_health,
        alertness: npc.alertness as f32 / 100.0,
        mood: npc.mood as f32 / 100.0,
        player_affinity: rel.affinity as f32 / 100.0,
        player_trust: rel.trust as f32 / 100.0,
        player_fear: rel.fear as f32 / 100.0,
        relationship_type: rel.relationship_type,
        scheduled_activity: npc.current_activity.into(),
        at_activity_location: npc.target.map(|t| npc.position.distance(t) < 2.0).unwrap_or(true),
        recently_attacked,
        recently_traded: false, // TODO: track this
        heard_rumors: rel.memories.iter().any(|m| {
            matches!(m.memory_type, super::relationships::MemoryType::HeardRumor)
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_context() -> UtilityContext {
        UtilityContext {
            player_distance: 10.0,
            target_distance: 5.0,
            home_distance: 20.0,
            current_hour: 12.0,
            is_night: false,
            health_percent: 0.8,
            alertness: 0.2,
            mood: 0.3,
            player_affinity: 0.5,
            player_trust: 0.3,
            player_fear: 0.1,
            relationship_type: RelationshipType::Acquaintance,
            scheduled_activity: ScheduledActivity::Working,
            at_activity_location: true,
            recently_attacked: false,
            recently_traded: false,
            heard_rumors: false,
        }
    }

    #[test]
    fn test_consideration_player_nearby() {
        let mut ctx = test_context();

        ctx.player_distance = 0.0;
        assert!((Consideration::PlayerNearby.evaluate(&ctx) - 1.0).abs() < 0.01);

        ctx.player_distance = 20.0;
        assert!((Consideration::PlayerNearby.evaluate(&ctx) - 0.0).abs() < 0.01);

        ctx.player_distance = 10.0;
        assert!((Consideration::PlayerNearby.evaluate(&ctx) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_response_curves() {
        assert!((ResponseCurve::Linear.apply(0.5) - 0.5).abs() < 0.01);
        assert!((ResponseCurve::Quadratic.apply(0.5) - 0.25).abs() < 0.01);
        assert!((ResponseCurve::Threshold(0.3).apply(0.5) - 1.0).abs() < 0.01);
        assert!((ResponseCurve::Threshold(0.7).apply(0.5) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_evaluator_select_action() {
        let evaluator = UtilityEvaluator::new();
        let ctx = test_context();

        let (action, score) = evaluator.select_action(&ctx, NpcRole::Villager);

        // Should select work since it's work hours and at location
        assert!(score > 0.0);
        println!("Selected: {:?} with score {}", action, score);
    }

    #[test]
    fn test_flee_when_attacked() {
        let evaluator = UtilityEvaluator::new();
        let mut ctx = test_context();

        ctx.recently_attacked = true;
        ctx.player_distance = 5.0;
        ctx.health_percent = 0.3;

        let (action, _score) = evaluator.select_action(&ctx, NpcRole::Villager);

        // Should flee when recently attacked and low health
        assert_eq!(action, NpcAction::FleeFromPlayer);
    }
}
