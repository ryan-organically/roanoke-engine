//! AI behavior system using Hierarchical Finite State Machines

use super::entity::{Animal, AnimalId, PackId, Target};
use super::types::{AggressionType, BehaviorType, WolfGroupType};
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
    // Wolf-specific states
    Curious(CuriousState),
    Approaching,
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

/// Curious sub-states (for lone wolves)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CuriousState {
    Watching,      // Observing from distance
    Investigating, // Moving closer to investigate
    Circling,      // Circling around the player
    Sniffing,      // Close proximity, checking player out
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

        // Wolf-specific states - handled by update_wolf_behavior
        // If regular behavior gets these states, transition to appropriate standard state
        BehaviorState::Curious(_) => {
            if animal.awareness > 0.8 {
                BehaviorState::Alert(AlertState::Looking)
            } else {
                BehaviorState::Idle
            }
        }
        BehaviorState::Approaching => {
            if player_dist < stats.attack_range {
                BehaviorState::Alert(AlertState::Warning)
            } else {
                BehaviorState::Idle
            }
        }
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

// === Wolf-Specific Behavior System ===

/// Update wolf behavior based on group type
/// This is called instead of standard update_behavior for wolves
pub fn update_wolf_behavior(animal: &mut Animal, ctx: &BehaviorContext) {
    match animal.wolf_group_type {
        Some(WolfGroupType::Lone) => update_lone_wolf_behavior(animal, ctx),
        Some(WolfGroupType::Pair) => update_wolf_pair_behavior(animal, ctx),
        Some(WolfGroupType::SmallPack) | Some(WolfGroupType::LargePack) => {
            // Use standard pack behavior for packs
            update_behavior(animal, ctx);
        }
        None => {
            // Not a wolf or unknown type, use standard behavior
            update_behavior(animal, ctx);
        }
    }
}

/// Lone wolf behavior - curious, potentially tameable
/// Lone wolves observe the player from a distance, may approach if player is non-threatening
fn update_lone_wolf_behavior(animal: &mut Animal, ctx: &BehaviorContext) {
    let stats = animal.species.base_stats();
    let to_player = ctx.player_pos - animal.position;
    let player_dist = to_player.length();
    let player_in_detection = player_dist < stats.detection_range * 1.5; // Extended range for curiosity

    // Check if recently damaged - revert to defensive behavior
    if let Some(damage_time) = animal.last_damage_time {
        if damage_time.elapsed().as_secs_f32() < 30.0 {
            // Recently hurt - flee and don't be curious
            animal.behavior_state = BehaviorState::Flee(FleeState::Running);
            animal.target = Some(Target::FleeFrom(ctx.player_pos));
            animal.curiosity_level = (animal.curiosity_level - 0.3).max(0.0);
            execute_state(animal, ctx);
            return;
        }
    }

    // Flee condition check
    if animal.current_health <= animal.species.flee_health() && animal.is_alive() {
        animal.behavior_state = BehaviorState::Flee(FleeState::Running);
        animal.target = Some(Target::FleeFrom(ctx.player_pos));
        execute_state(animal, ctx);
        return;
    }

    // Update curiosity based on player proximity and behavior
    if player_in_detection {
        // Player detected - increase curiosity if moving slowly or stationary
        let player_speed = ctx.player_velocity.length();
        if player_speed < 3.0 {
            // Player is slow/stationary - increase curiosity
            animal.curiosity_level = (animal.curiosity_level + 0.05 * ctx.dt).min(1.0);
        } else if player_speed > 8.0 {
            // Player running - decrease curiosity, become wary
            animal.curiosity_level = (animal.curiosity_level - 0.1 * ctx.dt).max(0.0);
        }
    } else {
        // Player not detected - slowly decrease curiosity
        animal.curiosity_level = (animal.curiosity_level - 0.02 * ctx.dt).max(0.0);
    }

    // State machine for lone wolf
    let new_state = match animal.behavior_state {
        BehaviorState::Idle => {
            if player_in_detection && animal.curiosity_level > 0.3 {
                // Become curious
                BehaviorState::Curious(CuriousState::Watching)
            } else if rand_chance(0.01 * ctx.dt) {
                BehaviorState::Patrol
            } else {
                BehaviorState::Idle
            }
        }

        BehaviorState::Patrol => {
            if player_in_detection && animal.curiosity_level > 0.2 {
                BehaviorState::Curious(CuriousState::Watching)
            } else if animal.position.distance(animal.home_position) > animal.territory_radius {
                animal.target = Some(Target::Position(animal.home_position));
                BehaviorState::Patrol
            } else {
                BehaviorState::Patrol
            }
        }

        BehaviorState::Curious(curious_state) => {
            update_curious_state(animal, curious_state, ctx)
        }

        BehaviorState::Alert(alert_state) => {
            if animal.curiosity_level > 0.5 {
                // Transition to curious instead of aggressive
                BehaviorState::Curious(CuriousState::Watching)
            } else if animal.awareness < 0.2 {
                BehaviorState::Idle
            } else {
                BehaviorState::Alert(alert_state)
            }
        }

        BehaviorState::Approaching => {
            // Approaching player for potential taming
            if player_dist < 5.0 {
                // Close enough - sniff/investigate
                BehaviorState::Curious(CuriousState::Sniffing)
            } else if player_dist > stats.detection_range * 2.0 {
                // Lost interest
                BehaviorState::Idle
            } else {
                BehaviorState::Approaching
            }
        }

        // Fall through to standard behaviors
        _ => {
            update_behavior(animal, ctx);
            return;
        }
    };

    animal.behavior_state = new_state;
    execute_state(animal, ctx);
}

/// Update curious sub-state for lone wolves
fn update_curious_state(
    animal: &mut Animal,
    current: CuriousState,
    ctx: &BehaviorContext,
) -> BehaviorState {
    let player_dist = animal.position.distance(ctx.player_pos);
    let player_speed = ctx.player_velocity.length();
    let stats = animal.species.base_stats();

    // If player runs at the wolf, flee
    if player_speed > 6.0 && player_dist < 15.0 {
        let player_dir = ctx.player_velocity.normalize_or_zero();
        let to_wolf = (animal.position - ctx.player_pos).normalize_or_zero();
        let approaching = player_dir.dot(to_wolf) > 0.5;

        if approaching {
            animal.curiosity_level = (animal.curiosity_level - 0.2).max(0.0);
            return BehaviorState::Flee(FleeState::Running);
        }
    }

    match current {
        CuriousState::Watching => {
            // Watch from a safe distance (15-25 units)
            let safe_distance = 20.0;
            animal.look_at(ctx.player_pos);

            if animal.curiosity_level > 0.6 && player_dist > safe_distance * 0.8 {
                // Very curious - start investigating
                BehaviorState::Curious(CuriousState::Investigating)
            } else if animal.curiosity_level < 0.2 {
                // Lost interest
                BehaviorState::Idle
            } else if player_dist < safe_distance * 0.5 {
                // Player too close - back off
                let away_dir = (animal.position - ctx.player_pos).normalize_or_zero();
                animal.target = Some(Target::Position(animal.position + away_dir * 10.0));
                BehaviorState::Curious(CuriousState::Watching)
            } else {
                // Maintain watching distance
                if player_dist > safe_distance * 1.5 {
                    // Move closer to watch
                    let toward_player = (ctx.player_pos - animal.position).normalize_or_zero();
                    animal.target = Some(Target::Position(
                        ctx.player_pos - toward_player * safe_distance,
                    ));
                }
                BehaviorState::Curious(CuriousState::Watching)
            }
        }

        CuriousState::Investigating => {
            // Move closer, circle around player
            let investigate_distance = 12.0;
            animal.look_at(ctx.player_pos);

            if animal.curiosity_level > 0.8 && player_dist < investigate_distance {
                // Very curious and close - start circling
                BehaviorState::Curious(CuriousState::Circling)
            } else if animal.curiosity_level < 0.4 {
                // Lost some interest - go back to watching
                BehaviorState::Curious(CuriousState::Watching)
            } else {
                // Move toward investigate distance
                let toward_player = (ctx.player_pos - animal.position).normalize_or_zero();
                animal.target = Some(Target::Position(
                    ctx.player_pos - toward_player * investigate_distance,
                ));
                BehaviorState::Curious(CuriousState::Investigating)
            }
        }

        CuriousState::Circling => {
            // Circle around the player at close range
            let circle_dist = 8.0;
            animal.look_at(ctx.player_pos);

            // Calculate circle position
            let circle_pos = calculate_circle_position(
                animal.position,
                ctx.player_pos,
                circle_dist,
                stats.speed * 0.5,
                ctx.dt,
            );
            animal.target = Some(Target::Position(circle_pos));

            if animal.curiosity_level > 0.95 && player_speed < 2.0 {
                // Maximum curiosity and player is still - approach for sniffing
                BehaviorState::Curious(CuriousState::Sniffing)
            } else if animal.curiosity_level < 0.5 {
                // Back off to investigating
                BehaviorState::Curious(CuriousState::Investigating)
            } else {
                BehaviorState::Curious(CuriousState::Circling)
            }
        }

        CuriousState::Sniffing => {
            // Very close, sniffing the player - potential taming moment
            let sniff_distance = 4.0;
            animal.look_at(ctx.player_pos);

            if player_dist > sniff_distance * 2.0 {
                // Player moved away
                BehaviorState::Curious(CuriousState::Circling)
            } else if player_speed > 4.0 {
                // Player moving too fast
                BehaviorState::Curious(CuriousState::Watching)
            } else {
                // Stay close, this is the taming opportunity
                let toward_player = (ctx.player_pos - animal.position).normalize_or_zero();
                animal.target = Some(Target::Position(
                    ctx.player_pos - toward_player * sniff_distance,
                ));

                // Increase taming progress while sniffing
                animal.advance_taming(0.01 * ctx.dt);

                BehaviorState::Curious(CuriousState::Sniffing)
            }
        }
    }
}

/// Pheasant behavior - ground bird that flees short distances when startled
/// Stays near home territory, can be domesticated with patience
pub fn update_pheasant_behavior(animal: &mut Animal, ctx: &BehaviorContext) {
    let stats = animal.species.base_stats();
    let to_player = ctx.player_pos - animal.position;
    let player_dist = to_player.length();
    let player_in_detection = player_dist < stats.detection_range;

    // Update awareness
    update_awareness(animal, player_in_detection, ctx.dt);

    // Domesticated pheasants don't flee
    if animal.is_tamed() {
        // Tamed pheasant follows player loosely, forages nearby
        let follow_dist = 8.0;
        if player_dist > follow_dist * 2.0 {
            animal.target = Some(Target::Position(ctx.player_pos));
            animal.behavior_state = BehaviorState::Patrol;
        } else if player_dist < follow_dist {
            animal.target = None;
            animal.behavior_state = BehaviorState::Idle;
        }
        execute_state(animal, ctx);
        return;
    }

    // Short flee distance for pheasants - 15-25 meters, never more than 30m from home
    let max_flee_distance = 25.0;
    let home_dist = animal.position.distance(animal.home_position);

    let new_state = match animal.behavior_state {
        BehaviorState::Idle => {
            if player_in_detection && animal.awareness > 0.4 {
                // Player detected - burst flee!
                animal.target = Some(Target::FleeFrom(ctx.player_pos));
                BehaviorState::Flee(FleeState::Running)
            } else if rand_chance(0.005 * ctx.dt) {
                // Occasionally forage around
                BehaviorState::Patrol
            } else {
                BehaviorState::Idle
            }
        }

        BehaviorState::Patrol => {
            if player_in_detection && animal.awareness > 0.3 {
                // Startle and flee
                animal.target = Some(Target::FleeFrom(ctx.player_pos));
                BehaviorState::Flee(FleeState::Running)
            } else if home_dist > animal.territory_radius {
                // Return home
                animal.target = Some(Target::Position(animal.home_position));
                BehaviorState::Patrol
            } else {
                BehaviorState::Patrol
            }
        }

        BehaviorState::Alert(_) => {
            if animal.awareness > 0.5 {
                animal.target = Some(Target::FleeFrom(ctx.player_pos));
                BehaviorState::Flee(FleeState::Running)
            } else if animal.awareness < 0.2 {
                BehaviorState::Idle
            } else {
                BehaviorState::Alert(AlertState::Looking)
            }
        }

        BehaviorState::Flee(FleeState::Running) => {
            // Pheasants only flee a short distance
            let flee_dist = animal.position.distance(ctx.player_pos);

            if flee_dist > max_flee_distance {
                // Far enough, stop and hide
                BehaviorState::Flee(FleeState::Hiding)
            } else if home_dist > animal.territory_radius * 1.5 {
                // Getting too far from home - stop fleeing, crouch/hide
                BehaviorState::Flee(FleeState::Hiding)
            } else {
                // Keep fleeing but limit distance
                animal.target = Some(Target::FleeFrom(ctx.player_pos));
                BehaviorState::Flee(FleeState::Running)
            }
        }

        BehaviorState::Flee(FleeState::Hiding) => {
            // Crouching/hiding - pheasants freeze when hiding
            animal.velocity = Vec3::ZERO;
            animal.awareness = (animal.awareness - 0.15 * ctx.dt).max(0.0);

            if player_dist < stats.detection_range * 0.5 {
                // Player got too close while hiding - burst flee again
                animal.target = Some(Target::FleeFrom(ctx.player_pos));
                BehaviorState::Flee(FleeState::Running)
            } else if animal.awareness < 0.1 {
                // Calmed down
                if home_dist > animal.territory_radius * 0.5 {
                    // Return toward home
                    animal.target = Some(Target::Position(animal.home_position));
                    BehaviorState::Patrol
                } else {
                    BehaviorState::Idle
                }
            } else {
                BehaviorState::Flee(FleeState::Hiding)
            }
        }

        BehaviorState::Flee(FleeState::Cornered) => {
            // Pheasants don't fight back, they just freeze
            BehaviorState::Flee(FleeState::Hiding)
        }

        _ => BehaviorState::Idle,
    };

    animal.behavior_state = new_state;

    // Pheasants advance taming when player is nearby and they're calm
    if player_dist < 10.0 && animal.awareness < 0.3 && !matches!(animal.behavior_state, BehaviorState::Flee(_)) {
        animal.advance_taming(0.005 * ctx.dt);
    }

    execute_state(animal, ctx);
}

/// Wolf pair behavior - usually flee, sometimes aggressive
/// Pairs are skittish but can be dangerous if they decide to attack
fn update_wolf_pair_behavior(animal: &mut Animal, ctx: &BehaviorContext) {
    let stats = animal.species.base_stats();
    let to_player = ctx.player_pos - animal.position;
    let player_dist = to_player.length();
    let player_in_detection = player_dist < stats.detection_range;

    // Check flee/attack threshold
    // flee_chance_roll < 0.7 = flee (70%), >= 0.7 = aggressive (30%)
    let should_flee = animal.flee_chance_roll < 0.7;

    // Flee condition check
    if animal.current_health <= animal.species.flee_health() && animal.is_alive() {
        animal.behavior_state = BehaviorState::Flee(FleeState::Running);
        animal.target = Some(Target::FleeFrom(ctx.player_pos));
        execute_state(animal, ctx);
        return;
    }

    // Update awareness
    update_awareness(animal, player_in_detection, ctx.dt);

    let new_state = match animal.behavior_state {
        BehaviorState::Idle => {
            if player_in_detection && animal.awareness > 0.5 {
                if should_flee {
                    // Pair decides to flee
                    animal.target = Some(Target::FleeFrom(ctx.player_pos));
                    BehaviorState::Flee(FleeState::Running)
                } else {
                    // Pair decides to be aggressive
                    animal.target = Some(Target::Player);
                    BehaviorState::Alert(AlertState::Warning)
                }
            } else if rand_chance(0.01 * ctx.dt) {
                BehaviorState::Patrol
            } else {
                BehaviorState::Idle
            }
        }

        BehaviorState::Patrol => {
            if player_in_detection && animal.awareness > 0.3 {
                if should_flee {
                    BehaviorState::Alert(AlertState::Looking)
                } else {
                    BehaviorState::Alert(AlertState::Warning)
                }
            } else {
                BehaviorState::Patrol
            }
        }

        BehaviorState::Alert(alert_state) => {
            if animal.awareness > 0.8 {
                if should_flee {
                    animal.target = Some(Target::FleeFrom(ctx.player_pos));
                    BehaviorState::Flee(FleeState::Running)
                } else {
                    // Aggressive pair attacks
                    animal.target = Some(Target::Player);
                    BehaviorState::Pursue(PursueState::Chasing)
                }
            } else if animal.awareness < 0.2 {
                BehaviorState::Idle
            } else {
                BehaviorState::Alert(alert_state)
            }
        }

        BehaviorState::Flee(FleeState::Running) => {
            if player_dist > stats.detection_range * 3.0 {
                // Escaped far enough
                BehaviorState::Flee(FleeState::Hiding)
            } else {
                animal.target = Some(Target::FleeFrom(ctx.player_pos));
                BehaviorState::Flee(FleeState::Running)
            }
        }

        BehaviorState::Flee(FleeState::Hiding) => {
            animal.awareness = (animal.awareness - 0.1 * ctx.dt).max(0.0);
            if animal.awareness < 0.1 {
                BehaviorState::Idle
            } else if player_dist < stats.detection_range {
                BehaviorState::Flee(FleeState::Running)
            } else {
                BehaviorState::Flee(FleeState::Hiding)
            }
        }

        // Fall through to standard behavior for attack states
        _ => {
            update_behavior(animal, ctx);
            return;
        }
    };

    animal.behavior_state = new_state;
    execute_state(animal, ctx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animals::entity::{Animal, AnimalId};
    use crate::animals::types::{AnimalSpecies, WolfGroupType};

    /// Helper to create a test animal with default settings
    fn create_test_animal(species: AnimalSpecies) -> Animal {
        let stats = species.base_stats();
        Animal::new(
            AnimalId(1),
            species,
            Vec3::ZERO,
            stats.health,
            (0, 0),
        )
    }

    /// Helper to create a test wolf with group type
    fn create_test_wolf(group_type: WolfGroupType, flee_roll: f32) -> Animal {
        let species = AnimalSpecies::GrayWolf;
        let stats = species.base_stats();
        Animal::new_wolf(
            AnimalId(1),
            species,
            Vec3::ZERO,
            stats.health,
            (0, 0),
            group_type,
            flee_roll,
        )
    }

    /// Helper to create a behavior context
    fn create_context(player_pos: Vec3, player_velocity: Vec3, dt: f32) -> BehaviorContext<'static> {
        BehaviorContext {
            player_pos,
            player_velocity,
            dt,
            nearby_animals: &[],
        }
    }

    // ===========================================
    // State Transition Tests
    // ===========================================

    #[test]
    fn test_idle_to_alert_transition() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.behavior_state = BehaviorState::Idle;
        animal.awareness = 0.35; // Above 0.3 threshold for alert

        // Player within detection range
        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        update_behavior(&mut animal, &ctx);

        assert!(
            matches!(animal.behavior_state, BehaviorState::Alert(_)),
            "Animal should transition from Idle to Alert when awareness > 0.3"
        );
    }

    #[test]
    fn test_idle_to_pursue_high_awareness() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.behavior_state = BehaviorState::Idle;
        animal.awareness = 0.85; // Above 0.8 threshold for pursue

        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        update_behavior(&mut animal, &ctx);

        assert!(
            matches!(animal.behavior_state, BehaviorState::Pursue(_)),
            "Aggressive animal should transition from Idle to Pursue when awareness > 0.8"
        );
    }

    #[test]
    fn test_alert_to_idle_when_awareness_drops() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.behavior_state = BehaviorState::Alert(AlertState::Looking);
        animal.awareness = 0.15; // Below 0.2 threshold to return to idle

        // Player far away (outside detection range)
        let player_pos = Vec3::new(100.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        update_behavior(&mut animal, &ctx);

        assert!(
            matches!(animal.behavior_state, BehaviorState::Idle),
            "Animal should transition from Alert to Idle when awareness < 0.2"
        );
    }

    #[test]
    fn test_alert_to_pursue_transition() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.behavior_state = BehaviorState::Alert(AlertState::Looking);
        animal.awareness = 0.95; // Above 0.9 threshold for pursue from alert

        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        update_behavior(&mut animal, &ctx);

        assert!(
            matches!(animal.behavior_state, BehaviorState::Pursue(_)),
            "Animal should transition from Alert to Pursue when awareness > 0.9"
        );
    }

    #[test]
    fn test_pursue_to_attack_when_in_range() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.behavior_state = BehaviorState::Pursue(PursueState::Chasing);
        animal.awareness = 1.0;
        // Ensure attack is ready
        animal.attack_cooldowns = vec![0.0; 2];

        // Player within attack range (wolf attack range is 2.0)
        let player_pos = Vec3::new(1.5, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        update_behavior(&mut animal, &ctx);

        assert!(
            matches!(animal.behavior_state, BehaviorState::Attack(_)),
            "Animal should transition from Pursue to Attack when player in attack range"
        );
    }

    #[test]
    fn test_pursue_to_alert_when_player_escapes() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.behavior_state = BehaviorState::Pursue(PursueState::Chasing);
        animal.awareness = 1.0;

        // Player beyond 2x detection range (wolf detection is 25.0, so > 50.0)
        let player_pos = Vec3::new(60.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        update_behavior(&mut animal, &ctx);

        assert!(
            matches!(animal.behavior_state, BehaviorState::Alert(_)),
            "Animal should transition from Pursue to Alert when player escapes"
        );
        assert!(
            animal.awareness < 1.0,
            "Awareness should be reduced when player escapes"
        );
    }

    #[test]
    fn test_attack_state_progression() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);

        // Test WindingUp -> Striking
        animal.behavior_state = BehaviorState::Attack(AttackState::WindingUp);
        animal.animation_time = 0.0;
        let player_pos = Vec3::new(1.0, 0.0, 0.0);

        // Simulate time passing during wind-up
        let state = update_attack_state(&mut animal, AttackState::WindingUp, 1.0, 0.35);
        assert!(
            matches!(state, BehaviorState::Attack(AttackState::Striking)),
            "Attack should progress from WindingUp to Striking after 0.3s"
        );

        // Test Striking -> Recovering
        animal.animation_time = 0.0;
        let state = update_attack_state(&mut animal, AttackState::Striking, 1.0, 0.016);
        assert!(
            matches!(state, BehaviorState::Attack(AttackState::Recovering)),
            "Attack should progress from Striking to Recovering"
        );

        // Test Recovering -> back to Pursue (player outside attack range)
        animal.animation_time = 0.0;
        let state = update_attack_state(&mut animal, AttackState::Recovering, 5.0, 0.6);
        assert!(
            matches!(state, BehaviorState::Pursue(PursueState::Chasing)),
            "After recovery, should return to Pursue if player outside attack range"
        );
    }

    // ===========================================
    // Flee Behavior Tests
    // ===========================================

    #[test]
    fn test_flee_trigger_on_low_health() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.behavior_state = BehaviorState::Pursue(PursueState::Chasing);
        // Set health below flee threshold (wolf flee_health is 15.0)
        animal.current_health = 10.0;
        animal.awareness = 1.0;

        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        update_behavior(&mut animal, &ctx);

        assert!(
            matches!(animal.behavior_state, BehaviorState::Flee(FleeState::Running)),
            "Animal should flee when health drops below threshold"
        );
        assert!(
            matches!(animal.target, Some(Target::FleeFrom(_))),
            "Animal should have flee target set"
        );
    }

    #[test]
    fn test_flee_running_to_hiding() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.behavior_state = BehaviorState::Flee(FleeState::Running);
        animal.current_health = 10.0; // Below flee threshold to stay in flee mode

        // Player very far away (> 3x detection range = 75.0)
        let player_pos = Vec3::new(80.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        let new_state = update_flee_state(&mut animal, FleeState::Running, &ctx);

        assert!(
            matches!(new_state, BehaviorState::Flee(FleeState::Hiding)),
            "Animal should transition to Hiding when far enough from player"
        );
    }

    #[test]
    fn test_flee_hiding_to_idle() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.behavior_state = BehaviorState::Flee(FleeState::Hiding);
        animal.awareness = 0.05; // Very low awareness

        // Player far away
        let player_pos = Vec3::new(100.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.5);

        let new_state = update_flee_state(&mut animal, FleeState::Hiding, &ctx);

        assert!(
            matches!(new_state, BehaviorState::Idle),
            "Animal should return to Idle from Hiding when awareness is very low"
        );
    }

    #[test]
    fn test_flee_hiding_back_to_running() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.behavior_state = BehaviorState::Flee(FleeState::Hiding);
        animal.awareness = 0.5;

        // Player comes back within detection range
        let player_pos = Vec3::new(20.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        let new_state = update_flee_state(&mut animal, FleeState::Hiding, &ctx);

        assert!(
            matches!(new_state, BehaviorState::Flee(FleeState::Running)),
            "Animal should resume Running if player found while Hiding"
        );
    }

    #[test]
    fn test_cornered_animal_fights_back() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.behavior_state = BehaviorState::Flee(FleeState::Cornered);
        animal.attack_cooldowns = vec![0.0; 2]; // Attacks ready

        // Player within attack range
        let player_pos = Vec3::new(1.5, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        let new_state = update_flee_state(&mut animal, FleeState::Cornered, &ctx);

        assert!(
            matches!(new_state, BehaviorState::Attack(AttackState::WindingUp)),
            "Cornered animal should fight back"
        );
    }

    // ===========================================
    // Awareness Accumulation Tests
    // ===========================================

    #[test]
    fn test_awareness_increases_when_player_detected() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.awareness = 0.0;

        // Player is detected
        update_awareness(&mut animal, true, 1.0);

        assert!(
            animal.awareness > 0.0,
            "Awareness should increase when player detected"
        );
    }

    #[test]
    fn test_awareness_decreases_when_player_not_detected() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.awareness = 0.5;

        // Player not detected
        update_awareness(&mut animal, false, 1.0);

        assert!(
            animal.awareness < 0.5,
            "Awareness should decrease when player not detected"
        );
    }

    #[test]
    fn test_awareness_capped_at_one() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.awareness = 0.9;

        // High dt to push awareness over 1.0
        update_awareness(&mut animal, true, 10.0);

        assert!(
            animal.awareness <= 1.0,
            "Awareness should be capped at 1.0"
        );
    }

    #[test]
    fn test_awareness_minimum_zero() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.awareness = 0.05;

        // High dt to push awareness below 0.0
        update_awareness(&mut animal, false, 10.0);

        assert!(
            animal.awareness >= 0.0,
            "Awareness should not go below 0.0"
        );
    }

    #[test]
    fn test_awareness_rate_varies_by_behavior_type() {
        // Test stalker (slow awareness)
        let mut stalker = create_test_animal(AnimalSpecies::EasternCougar);
        stalker.awareness = 0.0;
        update_awareness(&mut stalker, true, 1.0);
        let stalker_awareness = stalker.awareness;

        // Test aggressive (fast awareness)
        let mut aggressive = create_test_animal(AnimalSpecies::WildBoar);
        aggressive.awareness = 0.0;
        update_awareness(&mut aggressive, true, 1.0);
        let aggressive_awareness = aggressive.awareness;

        assert!(
            aggressive_awareness > stalker_awareness,
            "Aggressive animals should alert faster than stalkers"
        );
    }

    #[test]
    fn test_awareness_stays_high_after_damage() {
        let mut animal = create_test_animal(AnimalSpecies::GrayWolf);
        animal.awareness = 0.5;
        animal.last_damage_time = Some(std::time::Instant::now());

        // Even without player detected, awareness should stay high
        update_awareness(&mut animal, false, 1.0);

        assert!(
            animal.awareness >= 0.8,
            "Awareness should stay high after recent damage"
        );
    }

    // ===========================================
    // Wolf Curiosity Behavior Tests
    // ===========================================

    #[test]
    fn test_lone_wolf_starts_curious() {
        let wolf = create_test_wolf(WolfGroupType::Lone, 0.5);

        assert!(
            wolf.curiosity_level == 0.5,
            "Lone wolf should start with curiosity level 0.5"
        );
    }

    #[test]
    fn test_lone_wolf_idle_to_curious() {
        let mut wolf = create_test_wolf(WolfGroupType::Lone, 0.5);
        wolf.behavior_state = BehaviorState::Idle;
        wolf.curiosity_level = 0.35; // Above 0.3 threshold

        // Player within extended detection range (1.5x normal)
        let player_pos = Vec3::new(30.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        update_lone_wolf_behavior(&mut wolf, &ctx);

        assert!(
            matches!(wolf.behavior_state, BehaviorState::Curious(_)),
            "Lone wolf should become curious when player is nearby"
        );
    }

    #[test]
    fn test_lone_wolf_curiosity_increases_with_slow_player() {
        let mut wolf = create_test_wolf(WolfGroupType::Lone, 0.5);
        wolf.behavior_state = BehaviorState::Curious(CuriousState::Watching);
        let initial_curiosity = wolf.curiosity_level;

        // Player moving slowly within detection
        let player_pos = Vec3::new(20.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::new(1.0, 0.0, 0.0), 1.0); // slow movement

        update_lone_wolf_behavior(&mut wolf, &ctx);

        assert!(
            wolf.curiosity_level > initial_curiosity,
            "Curiosity should increase when player moves slowly"
        );
    }

    #[test]
    fn test_lone_wolf_curiosity_decreases_with_fast_player() {
        let mut wolf = create_test_wolf(WolfGroupType::Lone, 0.5);
        wolf.behavior_state = BehaviorState::Curious(CuriousState::Watching);
        wolf.curiosity_level = 0.5;

        // Player running fast
        let player_pos = Vec3::new(20.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::new(10.0, 0.0, 0.0), 1.0); // fast movement

        update_lone_wolf_behavior(&mut wolf, &ctx);

        assert!(
            wolf.curiosity_level < 0.5,
            "Curiosity should decrease when player runs"
        );
    }

    #[test]
    fn test_lone_wolf_flees_when_player_charges() {
        let mut wolf = create_test_wolf(WolfGroupType::Lone, 0.5);
        wolf.behavior_state = BehaviorState::Curious(CuriousState::Watching);
        wolf.position = Vec3::new(10.0, 0.0, 0.0);

        // Player charging toward wolf
        let player_pos = Vec3::ZERO;
        let player_vel = Vec3::new(8.0, 0.0, 0.0); // Running toward wolf
        let ctx = create_context(player_pos, player_vel, 0.016);

        let new_state = update_curious_state(&mut wolf, CuriousState::Watching, &ctx);

        assert!(
            matches!(new_state, BehaviorState::Flee(FleeState::Running)),
            "Lone wolf should flee when player charges at it"
        );
    }

    #[test]
    fn test_curious_watching_to_investigating() {
        let mut wolf = create_test_wolf(WolfGroupType::Lone, 0.5);
        wolf.curiosity_level = 0.65; // Above 0.6 threshold
        wolf.position = Vec3::new(25.0, 0.0, 0.0);

        let player_pos = Vec3::ZERO;
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        let new_state = update_curious_state(&mut wolf, CuriousState::Watching, &ctx);

        assert!(
            matches!(new_state, BehaviorState::Curious(CuriousState::Investigating)),
            "High curiosity wolf should start investigating"
        );
    }

    #[test]
    fn test_curious_investigating_to_circling() {
        let mut wolf = create_test_wolf(WolfGroupType::Lone, 0.5);
        wolf.curiosity_level = 0.85; // Above 0.8 threshold
        wolf.position = Vec3::new(10.0, 0.0, 0.0); // Close to player

        let player_pos = Vec3::ZERO;
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        let new_state = update_curious_state(&mut wolf, CuriousState::Investigating, &ctx);

        assert!(
            matches!(new_state, BehaviorState::Curious(CuriousState::Circling)),
            "Very curious wolf close to player should start circling"
        );
    }

    #[test]
    fn test_curious_circling_to_sniffing() {
        let mut wolf = create_test_wolf(WolfGroupType::Lone, 0.5);
        wolf.curiosity_level = 0.96; // Above 0.95 threshold
        wolf.position = Vec3::new(5.0, 0.0, 0.0);

        // Player stationary
        let player_pos = Vec3::ZERO;
        let ctx = create_context(player_pos, Vec3::new(1.0, 0.0, 0.0), 0.016); // Very slow movement

        let new_state = update_curious_state(&mut wolf, CuriousState::Circling, &ctx);

        assert!(
            matches!(new_state, BehaviorState::Curious(CuriousState::Sniffing)),
            "Maximum curiosity wolf with stationary player should start sniffing"
        );
    }

    #[test]
    fn test_lone_wolf_flees_when_damaged_recently() {
        let mut wolf = create_test_wolf(WolfGroupType::Lone, 0.5);
        wolf.behavior_state = BehaviorState::Curious(CuriousState::Watching);
        wolf.last_damage_time = Some(std::time::Instant::now());
        wolf.curiosity_level = 0.8;

        let player_pos = Vec3::new(20.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        update_lone_wolf_behavior(&mut wolf, &ctx);

        assert!(
            matches!(wolf.behavior_state, BehaviorState::Flee(FleeState::Running)),
            "Recently damaged lone wolf should flee"
        );
        assert!(
            wolf.curiosity_level < 0.8,
            "Curiosity should decrease after damage"
        );
    }

    // ===========================================
    // Wolf Pair Behavior Tests
    // ===========================================

    #[test]
    fn test_wolf_pair_flees_with_low_roll() {
        let mut wolf = create_test_wolf(WolfGroupType::Pair, 0.5); // < 0.7, should flee
        wolf.behavior_state = BehaviorState::Idle;
        wolf.awareness = 0.6; // Above 0.5 threshold

        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        update_wolf_pair_behavior(&mut wolf, &ctx);

        assert!(
            matches!(wolf.behavior_state, BehaviorState::Flee(FleeState::Running)),
            "Wolf pair with low flee roll should flee"
        );
    }

    #[test]
    fn test_wolf_pair_aggressive_with_high_roll() {
        let mut wolf = create_test_wolf(WolfGroupType::Pair, 0.8); // >= 0.7, should be aggressive
        wolf.behavior_state = BehaviorState::Idle;
        wolf.awareness = 0.6; // Above 0.5 threshold

        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        let ctx = create_context(player_pos, Vec3::ZERO, 0.016);

        update_wolf_pair_behavior(&mut wolf, &ctx);

        assert!(
            matches!(wolf.behavior_state, BehaviorState::Alert(AlertState::Warning)),
            "Wolf pair with high flee roll should be aggressive"
        );
    }

    // ===========================================
    // Should Attack Logic Tests
    // ===========================================

    #[test]
    fn test_predatory_always_attacks() {
        let animal = create_test_animal(AnimalSpecies::EasternCougar); // Predatory

        let result = should_attack(&animal, 100.0); // Any distance

        assert!(result, "Predatory animals should always attack");
    }

    #[test]
    fn test_territorial_attacks_in_territory() {
        let mut animal = create_test_animal(AnimalSpecies::BlackBear); // Territorial/Defensive
        animal.home_position = Vec3::ZERO;
        animal.territory_radius = 30.0;
        animal.position = Vec3::new(10.0, 0.0, 0.0); // Within territory

        // For defensive animals (BlackBear), they need damage to attack
        animal.last_damage_time = Some(std::time::Instant::now());

        let result = should_attack(&animal, 15.0);

        assert!(result, "Defensive animal should attack after being damaged");
    }

    #[test]
    fn test_defensive_attacks_only_after_damage() {
        let mut animal = create_test_animal(AnimalSpecies::BlackBear); // Defensive

        // No damage taken
        let result_no_damage = should_attack(&animal, 10.0);

        // After damage
        animal.last_damage_time = Some(std::time::Instant::now());
        let result_with_damage = should_attack(&animal, 10.0);

        assert!(!result_no_damage, "Defensive animal should not attack without damage");
        assert!(result_with_damage, "Defensive animal should attack after taking damage");
    }

    #[test]
    fn test_cautious_attacks_when_close() {
        let animal = create_test_animal(AnimalSpecies::RedWolf); // Cautious
        let stats = animal.species.base_stats();

        // Too far (> 0.5 * detection_range)
        let result_far = should_attack(&animal, stats.detection_range * 0.6);

        // Close enough (< 0.5 * detection_range)
        let result_close = should_attack(&animal, stats.detection_range * 0.4);

        assert!(!result_far, "Cautious animal should not attack when far");
        assert!(result_close, "Cautious animal should attack when close");
    }

    // ===========================================
    // Initial Pursue State Tests
    // ===========================================

    #[test]
    fn test_stalker_initial_pursue_state() {
        let state = initial_pursue_state(BehaviorType::Stalker);
        assert_eq!(state, PursueState::Stalking, "Stalkers should start with Stalking");
    }

    #[test]
    fn test_pack_hunter_initial_pursue_state() {
        let state = initial_pursue_state(BehaviorType::PackHunter);
        assert_eq!(state, PursueState::Circling, "Pack hunters should start with Circling");
    }

    #[test]
    fn test_aggressive_initial_pursue_state() {
        let state = initial_pursue_state(BehaviorType::Aggressive);
        assert_eq!(state, PursueState::Chasing, "Aggressive animals should start with Chasing");
    }

    // ===========================================
    // Pack System Tests
    // ===========================================

    #[test]
    fn test_pack_state_creation() {
        let pack = PackState::new(PackId(1), Vec3::new(100.0, 0.0, 100.0));

        assert_eq!(pack.pack_id, PackId(1));
        assert_eq!(pack.morale, 1.0);
        assert!(pack.alpha_id.is_none());
        assert!(pack.member_ids.is_empty());
        assert_eq!(pack.tactic, PackTactic::Patrolling);
    }

    #[test]
    fn test_pack_add_member() {
        let mut pack = PackState::new(PackId(1), Vec3::ZERO);

        pack.add_member(AnimalId(1), true);
        pack.add_member(AnimalId(2), false);
        pack.add_member(AnimalId(3), false);

        assert_eq!(pack.member_count(), 3);
        assert_eq!(pack.alpha_id, Some(AnimalId(1)));
    }

    #[test]
    fn test_pack_remove_member() {
        let mut pack = PackState::new(PackId(1), Vec3::ZERO);
        pack.add_member(AnimalId(1), true);
        pack.add_member(AnimalId(2), false);
        pack.morale = 1.0;

        pack.remove_member(AnimalId(2));

        assert_eq!(pack.member_count(), 1);
        assert!(pack.morale < 1.0, "Morale should drop when member dies");
    }

    #[test]
    fn test_pack_alpha_death_severe_morale_drop() {
        let mut pack = PackState::new(PackId(1), Vec3::ZERO);
        pack.add_member(AnimalId(1), true);
        pack.add_member(AnimalId(2), false);
        pack.morale = 1.0;

        pack.remove_member(AnimalId(1)); // Alpha dies

        assert!(pack.morale <= 0.3, "Morale should drop severely when alpha dies");
        assert_eq!(pack.alpha_id, Some(AnimalId(2)), "New alpha should be promoted");
    }

    #[test]
    fn test_pack_should_retreat() {
        let mut pack = PackState::new(PackId(1), Vec3::ZERO);
        pack.add_member(AnimalId(1), true);
        pack.add_member(AnimalId(2), false);
        pack.morale = 1.0;

        assert!(!pack.should_retreat(), "Pack with good morale should not retreat");

        pack.morale = 0.2;
        assert!(pack.should_retreat(), "Pack with low morale should retreat");

        pack.morale = 1.0;
        pack.remove_member(AnimalId(2)); // Only 1 member left
        assert!(pack.should_retreat(), "Pack with only 1 member should retreat");
    }

    #[test]
    fn test_pack_morale_recovery() {
        let mut pack = PackState::new(PackId(1), Vec3::ZERO);
        pack.add_member(AnimalId(1), true);
        pack.morale = 0.5;

        pack.update_morale(1.0);

        assert!(pack.morale > 0.5, "Morale should slowly recover over time");
    }

    #[test]
    fn test_pack_morale_capped_without_alpha() {
        let mut pack = PackState::new(PackId(1), Vec3::ZERO);
        pack.add_member(AnimalId(1), false); // No alpha
        pack.morale = 1.0;

        pack.update_morale(1.0);

        assert!(pack.morale <= 0.5, "Morale should be capped at 0.5 without alpha");
    }

    // ===========================================
    // Territory System Tests
    // ===========================================

    #[test]
    fn test_territory_contains() {
        let territory = Territory::new(AnimalId(1), Vec3::ZERO, 50.0);

        assert!(territory.contains(Vec3::new(25.0, 0.0, 25.0)));
        assert!(!territory.contains(Vec3::new(60.0, 0.0, 0.0)));
    }

    #[test]
    fn test_territory_aggression_at() {
        let territory = Territory::new(AnimalId(1), Vec3::ZERO, 50.0);

        let center_aggression = territory.aggression_at(Vec3::ZERO);
        let edge_aggression = territory.aggression_at(Vec3::new(40.0, 0.0, 0.0));
        let outside_aggression = territory.aggression_at(Vec3::new(60.0, 0.0, 0.0));

        assert!(center_aggression > edge_aggression, "Aggression should be higher at center");
        assert_eq!(outside_aggression, 0.0, "No aggression outside territory");
    }

    // ===========================================
    // Flanking Position Tests
    // ===========================================

    #[test]
    fn test_flank_positions_distributed() {
        let target = Vec3::new(50.0, 0.0, 50.0);
        let pack_center = Vec3::ZERO;
        let members = 4;
        let distance = 10.0;

        let positions: Vec<Vec3> = (0..members)
            .map(|i| calculate_flank_position(target, pack_center, i, members, distance))
            .collect();

        // Check that positions are roughly the desired distance from target
        for pos in &positions {
            let dist = pos.distance(target);
            assert!(
                (dist - distance).abs() < 1.0,
                "Flank position should be near desired distance from target"
            );
        }

        // Check that positions are spread out (not all the same)
        assert!(
            positions[0].distance(positions[1]) > 1.0,
            "Flank positions should be spread out"
        );
    }

    #[test]
    fn test_flank_position_single_member() {
        let target = Vec3::new(50.0, 0.0, 50.0);
        let pack_center = Vec3::ZERO;

        let position = calculate_flank_position(target, pack_center, 0, 1, 10.0);

        assert_eq!(position, target, "Single member should go directly to target");
    }

    // ===========================================
    // Circle Position Tests
    // ===========================================

    #[test]
    fn test_circle_position_moves_perpendicular() {
        let current = Vec3::new(10.0, 0.0, 0.0);
        let target = Vec3::ZERO;
        let radius = 10.0;
        let speed = 5.0;
        let dt = 1.0;

        let new_pos = calculate_circle_position(current, target, radius, speed, dt);

        // Should move perpendicular to target direction (circling)
        let initial_dist = current.distance(target);
        let new_dist = new_pos.distance(target);

        // Distance should stay roughly the same when at correct radius
        assert!(
            (new_dist - initial_dist).abs() < 2.0,
            "Circling should maintain approximate distance"
        );
    }

    // ===========================================
    // Random Chance Tests (Determinism)
    // ===========================================

    #[test]
    fn test_rand_chance_bounds() {
        // Test that rand_chance returns reasonable results
        let mut true_count = 0;
        let iterations = 1000;

        for _ in 0..iterations {
            if rand_chance(0.5) {
                true_count += 1;
            }
        }

        // With 50% probability, should be roughly 500 +/- significant margin
        // This is a very loose test since it's pseudo-random
        assert!(true_count > 200 && true_count < 800,
            "rand_chance should produce reasonable distribution");
    }

    #[test]
    fn test_rand_chance_seeded_deterministic() {
        let seed = 12345u64;
        let probability = 0.5;

        let result1 = rand_chance_seeded(probability, seed);
        let result2 = rand_chance_seeded(probability, seed);

        assert_eq!(result1, result2, "Same seed should produce same result");
    }

    #[test]
    fn test_rand_chance_seeded_varies_with_seed() {
        let probability = 0.5;
        let mut different_results = false;

        // Check that different seeds can produce different results
        for seed in 0..100u64 {
            let result1 = rand_chance_seeded(probability, seed);
            let result2 = rand_chance_seeded(probability, seed + 1000);
            if result1 != result2 {
                different_results = true;
                break;
            }
        }

        assert!(different_results, "Different seeds should sometimes produce different results");
    }
}
