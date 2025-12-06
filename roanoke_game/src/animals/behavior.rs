//! AI behavior system using Hierarchical Finite State Machines

use super::entity::{Animal, AnimalId, PackId, Target};
use super::types::{AggressionType, BehaviorType};
use crate::player::Player;
use glam::Vec3;

/// High-level behavior states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BehaviorState {
    #[default]
    Idle,
    Patrol,
    Alert(AlertState),
    Pursue(PursueState),
    Attack(AttackState),
    Flee(FleeState),
    Dead,
}

/// Alert sub-states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertState {
    Listening,
    Looking,
    Warning, // Threat display
}

/// Pursue sub-states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PursueState {
    Chasing,
    Stalking,
    Circling,
    Closing,
}

/// Attack sub-states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackState {
    WindingUp,
    Striking,
    Recovering,
}

/// Flee sub-states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FleeState {
    Running,
    Hiding,
    Cornered,
}

/// Context for behavior updates
pub struct BehaviorContext<'a> {
    pub player_pos: Vec3,
    pub player_velocity: Vec3,
    pub dt: f32,
    pub nearby_animals: &'a [AnimalId],
}

/// Update an animal's behavior based on its current state and environment
pub fn update_behavior(animal: &mut Animal, ctx: &BehaviorContext) {
    let species = animal.species;
    let stats = species.base_stats();

    // Calculate player relationship
    let to_player = ctx.player_pos - animal.position;
    let player_dist = to_player.length();
    let player_in_detection = player_dist < stats.detection_range;
    let player_in_attack = player_dist < stats.attack_range;

    // Update awareness (gradual alerting/relaxing)
    update_awareness(animal, player_in_detection, ctx.dt);

    // Check flee condition first
    if animal.current_health <= species.flee_health() && animal.is_alive() {
        if animal.behavior_state != BehaviorState::Flee(FleeState::Running)
            && animal.behavior_state != BehaviorState::Flee(FleeState::Cornered)
        {
            animal.behavior_state = BehaviorState::Flee(FleeState::Running);
            animal.target = Some(Target::FleeFrom(ctx.player_pos));
        }
    }

    // State machine transitions
    let new_state = match animal.behavior_state {
        BehaviorState::Idle => {
            if animal.awareness > 0.8 && should_attack(animal, player_dist) {
                animal.target = Some(Target::Player);
                BehaviorState::Pursue(initial_pursue_state(species.behavior_type()))
            } else if animal.awareness > 0.3 {
                BehaviorState::Alert(AlertState::Looking)
            } else if rand_chance(0.01 * ctx.dt) {
                // Occasionally start patrolling
                BehaviorState::Patrol
            } else {
                BehaviorState::Idle
            }
        }

        BehaviorState::Patrol => {
            if animal.awareness > 0.5 {
                BehaviorState::Alert(AlertState::Listening)
            } else {
                // Continue patrol or return to idle
                if animal.position.distance(animal.home_position) > animal.territory_radius {
                    animal.target = Some(Target::Position(animal.home_position));
                }
                BehaviorState::Patrol
            }
        }

        BehaviorState::Alert(_) => {
            if animal.awareness < 0.2 {
                animal.target = None;
                BehaviorState::Idle
            } else if animal.awareness > 0.9 && should_attack(animal, player_dist) {
                animal.target = Some(Target::Player);
                BehaviorState::Pursue(initial_pursue_state(species.behavior_type()))
            } else {
                // Stay alert, maybe issue warning
                if species.behavior_type() == BehaviorType::Territorial && player_in_detection {
                    BehaviorState::Alert(AlertState::Warning)
                } else {
                    BehaviorState::Alert(AlertState::Looking)
                }
            }
        }

        BehaviorState::Pursue(sub) => {
            if player_in_attack && animal.select_attack(player_dist).is_some() {
                BehaviorState::Attack(AttackState::WindingUp)
            } else if player_dist > stats.detection_range * 2.0 {
                // Lost the player
                animal.awareness = 0.5;
                BehaviorState::Alert(AlertState::Looking)
            } else {
                // Continue pursuit based on behavior type
                update_pursue_state(animal, sub, ctx)
            }
        }

        BehaviorState::Attack(sub) => update_attack_state(animal, sub, player_dist, ctx.dt),

        BehaviorState::Flee(sub) => update_flee_state(animal, sub, ctx),

        BehaviorState::Dead => BehaviorState::Dead,
    };

    animal.behavior_state = new_state;

    // Execute current state behavior (movement, etc.)
    execute_state(animal, ctx);
}

/// Update awareness level based on player proximity
fn update_awareness(animal: &mut Animal, player_detected: bool, dt: f32) {
    if player_detected {
        // Alert rate depends on behavior type
        let alert_rate = match animal.species.behavior_type() {
            BehaviorType::Stalker | BehaviorType::Ambush => 0.3, // Slow to reveal
            BehaviorType::Aggressive => 1.5,                     // Quick to alert
            _ => 0.8,
        };
        animal.awareness = (animal.awareness + alert_rate * dt).min(1.0);
    } else {
        // Relax over time
        animal.awareness = (animal.awareness - 0.1 * dt).max(0.0);
    }

    // Recent damage keeps awareness high
    if let Some(damage_time) = animal.last_damage_time {
        if damage_time.elapsed().as_secs_f32() < 10.0 {
            animal.awareness = animal.awareness.max(0.8);
        }
    }
}

/// Determine if animal should attack based on aggression type
fn should_attack(animal: &Animal, player_dist: f32) -> bool {
    let stats = animal.species.base_stats();

    match animal.species.aggression_type() {
        AggressionType::Predatory => true,
        AggressionType::Aggressive => player_dist < stats.detection_range,
        AggressionType::Territorial => {
            let dist_from_home = animal.position.distance(animal.home_position);
            dist_from_home < animal.territory_radius
        }
        AggressionType::Defensive => animal.last_damage_time.is_some(),
        AggressionType::Cautious => {
            // More likely to attack if player is close or wounded
            // For now, simple distance check
            player_dist < stats.detection_range * 0.5
        }
    }
}

/// Get initial pursue state based on behavior type
fn initial_pursue_state(behavior: BehaviorType) -> PursueState {
    match behavior {
        BehaviorType::Stalker => PursueState::Stalking,
        BehaviorType::PackHunter => PursueState::Circling,
        _ => PursueState::Chasing,
    }
}

/// Update pursue sub-state
fn update_pursue_state(
    animal: &mut Animal,
    current: PursueState,
    ctx: &BehaviorContext,
) -> BehaviorState {
    let behavior_type = animal.species.behavior_type();
    let player_dist = animal.position.distance(ctx.player_pos);
    let stats = animal.species.base_stats();

    match behavior_type {
        BehaviorType::Stalker => {
            // Stalkers maintain distance and wait for opportunity
            match current {
                PursueState::Stalking => {
                    let ideal_dist = stats.detection_range * 0.7;
                    if player_dist < ideal_dist * 0.5 {
                        // Too close, back off or pounce
                        if animal.attack_ready(0) {
                            BehaviorState::Attack(AttackState::WindingUp)
                        } else {
                            animal.target = Some(Target::FleeFrom(ctx.player_pos));
                            BehaviorState::Pursue(PursueState::Stalking)
                        }
                    } else if player_dist > ideal_dist * 1.2 {
                        // Close in
                        BehaviorState::Pursue(PursueState::Closing)
                    } else {
                        BehaviorState::Pursue(PursueState::Stalking)
                    }
                }
                PursueState::Closing => {
                    if player_dist < stats.attack_range * 2.0 {
                        BehaviorState::Pursue(PursueState::Stalking)
                    } else {
                        BehaviorState::Pursue(PursueState::Closing)
                    }
                }
                _ => BehaviorState::Pursue(PursueState::Stalking),
            }
        }

        BehaviorType::PackHunter => {
            // Pack hunters try to flank
            // For now, just chase (pack coordination in pack module)
            BehaviorState::Pursue(PursueState::Chasing)
        }

        _ => {
            // Direct chase
            BehaviorState::Pursue(PursueState::Chasing)
        }
    }
}

/// Update attack sub-state
fn update_attack_state(
    animal: &mut Animal,
    current: AttackState,
    player_dist: f32,
    dt: f32,
) -> BehaviorState {
    match current {
        AttackState::WindingUp => {
            // Wind-up complete, strike
            animal.animation_time += dt;
            if animal.animation_time > 0.3 {
                animal.animation_time = 0.0;
                BehaviorState::Attack(AttackState::Striking)
            } else {
                BehaviorState::Attack(AttackState::WindingUp)
            }
        }

        AttackState::Striking => {
            // Attack lands, go to recovery
            // Damage is applied in combat.rs
            animal.animation_time = 0.0;
            BehaviorState::Attack(AttackState::Recovering)
        }

        AttackState::Recovering => {
            animal.animation_time += dt;
            if animal.animation_time > 0.5 {
                animal.animation_time = 0.0;
                // Return to pursuit or attack again
                let stats = animal.species.base_stats();
                if player_dist < stats.attack_range {
                    if animal.select_attack(player_dist).is_some() {
                        BehaviorState::Attack(AttackState::WindingUp)
                    } else {
                        BehaviorState::Pursue(PursueState::Closing)
                    }
                } else {
                    BehaviorState::Pursue(PursueState::Chasing)
                }
            } else {
                BehaviorState::Attack(AttackState::Recovering)
            }
        }
    }
}

/// Update flee sub-state
fn update_flee_state(animal: &mut Animal, current: FleeState, ctx: &BehaviorContext) -> BehaviorState {
    let player_dist = animal.position.distance(ctx.player_pos);
    let stats = animal.species.base_stats();

    match current {
        FleeState::Running => {
            if player_dist > stats.detection_range * 3.0 {
                // Lost the pursuer, hide
                BehaviorState::Flee(FleeState::Hiding)
            } else if is_cornered(animal, ctx.player_pos) {
                // Cornered, fight back
                BehaviorState::Flee(FleeState::Cornered)
            } else {
                // Keep running
                animal.target = Some(Target::FleeFrom(ctx.player_pos));
                BehaviorState::Flee(FleeState::Running)
            }
        }

        FleeState::Hiding => {
            if player_dist < stats.detection_range {
                // Found again
                BehaviorState::Flee(FleeState::Running)
            } else {
                // Stay hidden, gradually calm down
                animal.awareness = (animal.awareness - 0.05 * ctx.dt).max(0.0);
                if animal.awareness < 0.1 {
                    BehaviorState::Idle
                } else {
                    BehaviorState::Flee(FleeState::Hiding)
                }
            }
        }

        FleeState::Cornered => {
            // Fight back desperately
            animal.target = Some(Target::Player);
            if player_dist < stats.attack_range && animal.select_attack(player_dist).is_some() {
                BehaviorState::Attack(AttackState::WindingUp)
            } else {
                BehaviorState::Flee(FleeState::Cornered)
            }
        }
    }
}

/// Check if animal is cornered (simplified)
fn is_cornered(_animal: &Animal, _player_pos: Vec3) -> bool {
    // TODO: Implement proper corner detection using terrain
    false
}

/// Execute movement and actions for current state
fn execute_state(animal: &mut Animal, ctx: &BehaviorContext) {
    let stats = animal.species.base_stats();
    let speed = animal.current_speed();

    match &animal.target {
        Some(Target::Player) | Some(Target::Position(_)) => {
            let target_pos = match &animal.target {
                Some(Target::Player) => ctx.player_pos,
                Some(Target::Position(p)) => *p,
                _ => return,
            };

            let to_target = target_pos - animal.position;
            let dist = to_target.length();

            if dist > 0.5 {
                let direction = to_target / dist;
                animal.velocity = direction * speed;
                animal.look_at(target_pos);
            } else {
                animal.velocity = Vec3::ZERO;
            }
        }

        Some(Target::FleeFrom(threat_pos)) => {
            let away_from_threat = (animal.position - *threat_pos).normalize_or_zero();
            animal.velocity = away_from_threat * speed * 1.2; // Flee faster
            animal.look_at(animal.position + away_from_threat);
        }

        None => {
            // No target, slow down
            animal.velocity = animal.velocity * 0.9;
            if animal.velocity.length() < 0.1 {
                animal.velocity = Vec3::ZERO;
            }
        }

        _ => {}
    }
}

/// Ultra-fast hash-based random chance - O(1) with zero syscalls
/// Uses spatial-temporal hashing for deterministic yet varied results
#[inline(always)]
fn rand_chance(probability: f32) -> bool {
    // Use thread-local frame counter for temporal variation
    // Combined with fast integer hash - no syscalls, no allocations
    static FRAME_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let frame = FRAME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // PCG-inspired fast hash - extremely fast, good distribution
    let mut state = frame.wrapping_mul(0x5851F42D4C957F2D);
    state = state.wrapping_add(0x14057B7EF767814F);
    state = (state ^ (state >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94D049BB133111EB);
    state = state ^ (state >> 31);

    (state as f32 / u64::MAX as f32) < probability
}

/// Context-aware random for behavior variation based on animal state
/// Provides deterministic randomness that's consistent per-animal per-frame
#[inline(always)]
pub fn rand_chance_seeded(probability: f32, seed: u64) -> bool {
    // Combine seed with golden ratio hash for excellent distribution
    let hash = seed.wrapping_mul(0x9E3779B97F4A7C15);
    let mixed = (hash ^ (hash >> 33)).wrapping_mul(0xFF51AFD7ED558CCD);
    let final_hash = (mixed ^ (mixed >> 33)).wrapping_mul(0xC4CEB9FE1A85EC53);

    (final_hash as f32 / u64::MAX as f32) < probability
}
