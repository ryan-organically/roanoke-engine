//! AI behavior system using Hierarchical Finite State Machines

use super::entity::{Animal, AnimalId, PackId, Target};
use super::types::{AggressionType, BehaviorType};
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

// === Pack Coordination System ===

/// Role within a pack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackRole {
    Alpha,
    Beta,
    Hunter,
    Scout,
    Guard,
}

/// Pack tactical state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackTactic {
    Patrolling,
    Hunting,
    Flanking,
    Surrounding,
    Retreating,
    Defending,
}

/// Pack coordination state
#[derive(Debug, Clone)]
pub struct PackState {
    pub pack_id: PackId,
    pub alpha_id: Option<AnimalId>,
    pub member_ids: Vec<AnimalId>,
    pub tactic: PackTactic,
    pub target_pos: Option<Vec3>,
    pub formation_center: Vec3,
    pub morale: f32, // 0.0 = broken, 1.0 = confident
    pub alert_level: f32,
    pub last_kill_time: Option<std::time::Instant>,
}

impl PackState {
    pub fn new(pack_id: PackId, center: Vec3) -> Self {
        Self {
            pack_id,
            alpha_id: None,
            member_ids: Vec::new(),
            tactic: PackTactic::Patrolling,
            target_pos: None,
            formation_center: center,
            morale: 1.0,
            alert_level: 0.0,
            last_kill_time: None,
        }
    }

    /// Add a member to the pack
    pub fn add_member(&mut self, id: AnimalId, is_alpha: bool) {
        self.member_ids.push(id);
        if is_alpha {
            self.alpha_id = Some(id);
        }
    }

    /// Remove a member (death/despawn)
    pub fn remove_member(&mut self, id: AnimalId) {
        self.member_ids.retain(|&m| m != id);

        // If alpha died, pack morale drops significantly
        if self.alpha_id == Some(id) {
            self.alpha_id = None;
            self.morale *= 0.3;

            // Promote strongest remaining as new alpha
            if !self.member_ids.is_empty() {
                self.alpha_id = Some(self.member_ids[0]);
            }
        } else {
            // Regular member death reduces morale slightly
            self.morale *= 0.85;
        }
    }

    /// Update pack morale
    pub fn update_morale(&mut self, dt: f32) {
        // Morale slowly recovers
        self.morale = (self.morale + 0.02 * dt).min(1.0);

        // But caps lower if missing alpha
        if self.alpha_id.is_none() {
            self.morale = self.morale.min(0.5);
        }

        // Recent kill boosts morale
        if let Some(kill_time) = self.last_kill_time {
            if kill_time.elapsed().as_secs_f32() < 30.0 {
                self.morale = self.morale.max(0.8);
            }
        }
    }

    /// Check if pack should retreat
    pub fn should_retreat(&self) -> bool {
        self.morale < 0.25 || self.member_ids.len() <= 1
    }

    /// Get member count
    pub fn member_count(&self) -> usize {
        self.member_ids.len()
    }

    /// Get alpha position from animals list
    pub fn alpha_position(&self, animals: &[Animal]) -> Option<Vec3> {
        self.alpha_id.and_then(|alpha_id| {
            animals.iter().find(|a| a.id == alpha_id).map(|a| a.position)
        })
    }
}

/// Calculate flanking position for pack hunting
pub fn calculate_flank_position(
    target_pos: Vec3,
    pack_center: Vec3,
    member_index: usize,
    total_members: usize,
    desired_distance: f32,
) -> Vec3 {
    if total_members <= 1 {
        return target_pos;
    }

    // Distribute members in an arc around the target
    let base_angle = (target_pos - pack_center).normalize_or_zero();
    let angle_offset = std::f32::consts::PI * 2.0 / total_members as f32;
    let angle = angle_offset * member_index as f32 - std::f32::consts::PI;

    // Rotate the direction
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let rotated = Vec3::new(
        base_angle.x * cos_a - base_angle.z * sin_a,
        0.0,
        base_angle.x * sin_a + base_angle.z * cos_a,
    );

    target_pos + rotated * desired_distance
}

/// Update pack hunting behavior
pub fn update_pack_hunting(
    pack: &mut PackState,
    animals: &mut [Animal],
    player_pos: Vec3,
    dt: f32,
) {
    // Update morale
    pack.update_morale(dt);

    // Find alpha position first (avoid borrow issues)
    let alpha_pos = pack.alpha_id
        .and_then(|alpha_id| animals.iter().find(|a| a.id == alpha_id).map(|a| a.position))
        .unwrap_or(pack.formation_center);
    let player_dist = alpha_pos.distance(player_pos);

    if player_dist < 100.0 {
        pack.alert_level = (pack.alert_level + 0.5 * dt).min(1.0);
    } else {
        pack.alert_level = (pack.alert_level - 0.1 * dt).max(0.0);
    }

    // Determine pack tactic based on state
    pack.tactic = if pack.should_retreat() {
        PackTactic::Retreating
    } else if pack.alert_level > 0.8 && player_dist < 50.0 {
        PackTactic::Surrounding
    } else if pack.alert_level > 0.5 {
        PackTactic::Hunting
    } else if pack.alert_level > 0.2 {
        PackTactic::Flanking
    } else {
        PackTactic::Patrolling
    };

    // Assign positions based on tactic
    let members: Vec<AnimalId> = pack.member_ids.clone();
    let member_count = members.len();
    let tactic = pack.tactic;
    let formation_center = pack.formation_center;
    let pack_alpha_id = pack.alpha_id;
    let alert_level = pack.alert_level;

    for (idx, &member_id) in members.iter().enumerate() {
        if let Some(animal) = animals.iter_mut().find(|a| a.id == member_id) {
            let role = if pack_alpha_id == Some(member_id) {
                PackRole::Alpha
            } else if idx == 1 {
                PackRole::Beta
            } else if idx % 3 == 0 {
                PackRole::Scout
            } else {
                PackRole::Hunter
            };

            match tactic {
                PackTactic::Patrolling => {
                    // Patrol around formation center
                    let patrol_radius = 30.0 + (idx as f32 * 5.0);
                    let angle = (idx as f32 * 2.4) + (alert_level * 10.0);
                    let patrol_pos = formation_center + Vec3::new(
                        angle.cos() * patrol_radius,
                        0.0,
                        angle.sin() * patrol_radius,
                    );
                    animal.target = Some(Target::Position(patrol_pos));
                }

                PackTactic::Hunting => {
                    // Alpha leads, others follow
                    if role == PackRole::Alpha {
                        animal.target = Some(Target::Player);
                        animal.behavior_state = BehaviorState::Pursue(PursueState::Stalking);
                    } else {
                        // Follow alpha at distance (using cached alpha_pos)
                        let follow_pos = alpha_pos + Vec3::new(
                            (idx as f32 * 1.5).sin() * 10.0,
                            0.0,
                            (idx as f32 * 1.5).cos() * 10.0,
                        );
                        animal.target = Some(Target::Position(follow_pos));
                    }
                }

                PackTactic::Flanking => {
                    // Move to flanking positions around target
                    let flank_pos = calculate_flank_position(
                        player_pos,
                        alpha_pos,
                        idx,
                        member_count,
                        25.0,
                    );
                    animal.target = Some(Target::Position(flank_pos));
                    animal.behavior_state = BehaviorState::Pursue(PursueState::Circling);
                }

                PackTactic::Surrounding => {
                    // Close surround for attack
                    let attack_pos = calculate_flank_position(
                        player_pos,
                        alpha_pos,
                        idx,
                        member_count,
                        8.0,
                    );
                    animal.target = Some(Target::Position(attack_pos));
                    animal.awareness = 1.0;

                    // Attack if in range
                    if animal.position.distance(player_pos) < animal.species.base_stats().attack_range * 1.5 {
                        animal.target = Some(Target::Player);
                        animal.behavior_state = BehaviorState::Attack(AttackState::WindingUp);
                    }
                }

                PackTactic::Retreating => {
                    // Flee away from player
                    let retreat_dir = (alpha_pos - player_pos).normalize_or_zero();
                    let retreat_pos = alpha_pos + retreat_dir * 100.0;
                    animal.target = Some(Target::Position(retreat_pos));
                    animal.behavior_state = BehaviorState::Flee(FleeState::Running);
                }

                PackTactic::Defending => {
                    // Defend territory
                    let defend_pos = formation_center + Vec3::new(
                        (idx as f32 * 1.2).cos() * 15.0,
                        0.0,
                        (idx as f32 * 1.2).sin() * 15.0,
                    );
                    animal.target = Some(Target::Position(defend_pos));
                    animal.behavior_state = BehaviorState::Alert(AlertState::Warning);
                }
            }
        }
    }
}

// === Territory System ===

/// Territory claim
#[derive(Debug, Clone)]
pub struct Territory {
    pub owner_id: AnimalId,
    pub center: Vec3,
    pub radius: f32,
    pub strength: f32, // How aggressively defended
    pub established_time: f32,
}

impl Territory {
    pub fn new(owner_id: AnimalId, center: Vec3, radius: f32) -> Self {
        Self {
            owner_id,
            center,
            radius,
            strength: 1.0,
            established_time: 0.0,
        }
    }

    /// Check if position is within territory
    pub fn contains(&self, pos: Vec3) -> bool {
        pos.distance(self.center) < self.radius
    }

    /// Get aggression level for intruder at position
    pub fn aggression_at(&self, pos: Vec3) -> f32 {
        let dist = pos.distance(self.center);
        if dist >= self.radius {
            return 0.0;
        }

        // More aggressive closer to center
        let closeness = 1.0 - (dist / self.radius);
        self.strength * closeness * (1.0 + self.established_time * 0.1).min(2.0)
    }
}

/// Update territorial behavior for a single animal
pub fn update_territorial_behavior(
    animal: &mut Animal,
    territory: &Territory,
    intruder_pos: Vec3,
    intruder_in_territory: bool,
    dt: f32,
) {
    if !intruder_in_territory {
        // No intruder, patrol territory
        if animal.behavior_state == BehaviorState::Idle {
            if rand_chance(0.02 * dt) {
                animal.behavior_state = BehaviorState::Patrol;
                // Pick random patrol point in territory using hash-based angle
                let hash = animal.id.0.wrapping_mul(0x9E3779B97F4A7C15);
                let angle = (hash as f32 / u64::MAX as f32) * std::f32::consts::TAU;
                let dist = territory.radius * 0.7;
                let patrol_target = territory.center + Vec3::new(
                    angle.cos() * dist,
                    0.0,
                    angle.sin() * dist,
                );
                animal.target = Some(Target::Position(patrol_target));
            }
        }
        return;
    }

    // Intruder detected!
    let aggression = territory.aggression_at(intruder_pos);

    if aggression > 0.7 {
        // Attack intruder
        animal.awareness = 1.0;
        animal.target = Some(Target::Player);
        animal.behavior_state = BehaviorState::Pursue(PursueState::Chasing);
    } else if aggression > 0.4 {
        // Warning display
        animal.awareness = (animal.awareness + 0.3 * dt).min(1.0);
        animal.behavior_state = BehaviorState::Alert(AlertState::Warning);
        animal.look_at(intruder_pos);
    } else if aggression > 0.1 {
        // Watch carefully
        animal.awareness = (animal.awareness + 0.1 * dt).min(0.8);
        animal.behavior_state = BehaviorState::Alert(AlertState::Looking);
        animal.look_at(intruder_pos);
    }
}

// === Hunting Patterns ===

/// Calculate ambush position for stalkers
pub fn calculate_ambush_position(
    stalker_pos: Vec3,
    target_pos: Vec3,
    target_velocity: Vec3,
    terrain_height_fn: impl Fn(f32, f32) -> f32,
) -> Vec3 {
    // Predict where target will be
    let prediction_time = 3.0;
    let predicted_pos = target_pos + target_velocity * prediction_time;

    // Find point ahead of target that's elevated (for pouncing)
    let to_predicted = (predicted_pos - stalker_pos).normalize_or_zero();
    let ambush_dist = 15.0;

    let candidate_pos = predicted_pos - to_predicted * ambush_dist;
    let ground_height = terrain_height_fn(candidate_pos.x, candidate_pos.z);

    Vec3::new(candidate_pos.x, ground_height + 1.0, candidate_pos.z)
}

/// Check if animal has line of sight to target (simplified)
pub fn has_line_of_sight(from: Vec3, to: Vec3, _terrain_height_fn: impl Fn(f32, f32) -> f32) -> bool {
    // Simplified - just check distance and height difference
    let dist = from.distance(to);
    let height_diff = (from.y - to.y).abs();

    // If target is too far down or occluded, no LOS
    dist < 150.0 && height_diff < 20.0
}

/// Calculate circling position for predators
pub fn calculate_circle_position(
    current_pos: Vec3,
    target_pos: Vec3,
    circle_radius: f32,
    circle_speed: f32,
    dt: f32,
) -> Vec3 {
    let to_target = target_pos - current_pos;
    let current_dist = to_target.length();

    if current_dist < 0.1 {
        return current_pos;
    }

    // Calculate perpendicular direction for circling
    let forward = to_target.normalize_or_zero();
    let right = Vec3::new(-forward.z, 0.0, forward.x);

    // Spiral inward if too far, outward if too close
    let radial_adjust = if current_dist > circle_radius {
        1.0
    } else if current_dist < circle_radius * 0.5 {
        -0.5
    } else {
        0.0
    };

    let movement = (right + forward * radial_adjust).normalize_or_zero() * circle_speed * dt;
    current_pos + movement
}
