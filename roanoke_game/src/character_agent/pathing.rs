//! Agent Pathing System
//!
//! Provides meaningful movement and navigation for all character agents.
//! Supports multiple path types:
//! - Schedule paths (NPCs following daily routines)
//! - Patrol paths (animals guarding territory)
//! - Pursuit paths (chasing targets)
//! - Flee paths (escaping threats)
//! - Wander paths (random exploration)

use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Type of path being followed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathType {
    /// Following a daily schedule (NPC)
    Schedule,
    /// Patrolling territory (Animal/Guard)
    Patrol,
    /// Direct pursuit of target
    Pursuit,
    /// Fleeing from threat
    Flee,
    /// Random wandering
    Wander,
    /// Moving to specific destination
    Direct,
    /// Circling around target
    Circle,
    /// Flanking maneuver
    Flank,
}

/// A single waypoint in a path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub position: Vec3,
    /// How long to wait at this waypoint (seconds)
    pub wait_time: f32,
    /// Optional activity to perform at waypoint
    pub activity: Option<WaypointActivity>,
    /// Speed multiplier for reaching this waypoint
    pub speed_mult: f32,
}

impl Waypoint {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            wait_time: 0.0,
            activity: None,
            speed_mult: 1.0,
        }
    }

    pub fn with_wait(mut self, time: f32) -> Self {
        self.wait_time = time;
        self
    }

    pub fn with_activity(mut self, activity: WaypointActivity) -> Self {
        self.activity = Some(activity);
        self
    }
}

/// Activity to perform at a waypoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaypointActivity {
    /// Look around (for patrols)
    LookAround { duration: f32 },
    /// Face a specific direction
    FaceDirection { direction: Vec3 },
    /// Perform work animation
    Work { duration: f32 },
    /// Social interaction position
    Socialize,
    /// Rest position
    Rest,
    /// Guard position
    Guard { alert_radius: f32 },
}

/// Path navigation state
#[derive(Debug, Clone, Default)]
pub struct PathState {
    /// Current path type
    pub path_type: Option<PathType>,
    /// Waypoints to follow
    pub waypoints: VecDeque<Waypoint>,
    /// Current target position
    pub current_target: Option<Vec3>,
    /// Time spent at current waypoint
    pub wait_elapsed: f32,
    /// Whether currently waiting at waypoint
    pub is_waiting: bool,
    /// Path completion callback flag
    pub path_complete: bool,
    /// Smooth path data for bezier curves
    pub smooth_points: Vec<Vec3>,
    /// Progress along smooth path (0.0 - 1.0)
    pub smooth_progress: f32,
}

impl PathState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a new path
    pub fn set_path(&mut self, path_type: PathType, waypoints: Vec<Waypoint>) {
        self.path_type = Some(path_type);
        self.waypoints = waypoints.into();
        self.path_complete = false;
        self.is_waiting = false;
        self.wait_elapsed = 0.0;
        self.current_target = self.waypoints.front().map(|w| w.position);
        self.smooth_points.clear();
        self.smooth_progress = 0.0;
    }

    /// Clear current path
    pub fn clear(&mut self) {
        self.path_type = None;
        self.waypoints.clear();
        self.current_target = None;
        self.path_complete = true;
        self.is_waiting = false;
    }

    /// Check if path is active
    pub fn has_path(&self) -> bool {
        self.path_type.is_some() && !self.path_complete
    }
}

/// Path follower component for agents
#[derive(Debug, Clone, Default)]
pub struct PathFollower {
    pub state: PathState,
    /// Arrival threshold distance
    pub arrival_threshold: f32,
    /// Turn rate (radians per second)
    pub turn_rate: f32,
    /// Whether to smooth the path
    pub use_smoothing: bool,
    /// Obstacle avoidance radius
    pub avoidance_radius: f32,
}

impl PathFollower {
    pub fn new() -> Self {
        Self {
            state: PathState::new(),
            arrival_threshold: 1.0,
            turn_rate: std::f32::consts::PI * 2.0, // 360 deg/sec
            use_smoothing: true,
            avoidance_radius: 2.0,
        }
    }

    /// Update path following, returns desired velocity
    pub fn update(&mut self, current_pos: Vec3, base_speed: f32, dt: f32) -> PathFollowResult {
        if !self.state.has_path() {
            return PathFollowResult::Idle;
        }

        // Check if waiting at waypoint
        if self.state.is_waiting {
            if let Some(waypoint) = self.state.waypoints.front() {
                self.state.wait_elapsed += dt;
                if self.state.wait_elapsed >= waypoint.wait_time {
                    self.state.is_waiting = false;
                    self.state.wait_elapsed = 0.0;
                    self.state.waypoints.pop_front();
                    self.state.current_target = self.state.waypoints.front().map(|w| w.position);

                    if self.state.waypoints.is_empty() {
                        self.state.path_complete = true;
                        return PathFollowResult::Complete;
                    }
                } else {
                    return PathFollowResult::Waiting {
                        activity: waypoint.activity.clone(),
                    };
                }
            }
        }

        // Get current target
        let Some(target) = self.state.current_target else {
            self.state.path_complete = true;
            return PathFollowResult::Complete;
        };

        // Calculate distance to target
        let to_target = target - current_pos;
        let distance = to_target.length();

        // Check arrival
        if distance < self.arrival_threshold {
            if let Some(waypoint) = self.state.waypoints.front() {
                if waypoint.wait_time > 0.0 {
                    self.state.is_waiting = true;
                    self.state.wait_elapsed = 0.0;
                    return PathFollowResult::Arrived {
                        activity: waypoint.activity.clone(),
                    };
                }
            }

            // Move to next waypoint
            self.state.waypoints.pop_front();
            self.state.current_target = self.state.waypoints.front().map(|w| w.position);

            if self.state.waypoints.is_empty() {
                self.state.path_complete = true;
                return PathFollowResult::Complete;
            }
        }

        // Calculate velocity toward target
        let direction = if distance > 0.01 {
            to_target / distance
        } else {
            Vec3::ZERO
        };

        // Get speed multiplier from current waypoint
        let speed_mult = self
            .state
            .waypoints
            .front()
            .map(|w| w.speed_mult)
            .unwrap_or(1.0);

        let velocity = direction * base_speed * speed_mult;

        PathFollowResult::Moving {
            velocity,
            target_direction: direction,
            distance_remaining: distance,
        }
    }

    /// Set a direct path to a position
    pub fn go_to(&mut self, target: Vec3) {
        self.state.set_path(PathType::Direct, vec![Waypoint::new(target)]);
    }

    /// Set a flee path away from a threat
    pub fn flee_from(&mut self, current_pos: Vec3, threat_pos: Vec3, flee_distance: f32) {
        let direction = (current_pos - threat_pos).normalize_or_zero();
        let flee_target = current_pos + direction * flee_distance;
        self.state.set_path(PathType::Flee, vec![Waypoint::new(flee_target)]);
    }

    /// Set a patrol path (circular around center)
    pub fn set_patrol(&mut self, center: Vec3, radius: f32, num_points: usize) {
        let mut waypoints = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let angle = (i as f32 / num_points as f32) * std::f32::consts::TAU;
            let pos = center + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
            waypoints.push(
                Waypoint::new(pos)
                    .with_wait(1.0)
                    .with_activity(WaypointActivity::LookAround { duration: 1.0 }),
            );
        }
        self.state.set_path(PathType::Patrol, waypoints);
    }

    /// Set a circling path around a target
    pub fn set_circle(&mut self, target: Vec3, radius: f32, clockwise: bool) {
        let mut waypoints = Vec::with_capacity(8);
        let step = if clockwise {
            -std::f32::consts::FRAC_PI_4
        } else {
            std::f32::consts::FRAC_PI_4
        };

        for i in 0..8 {
            let angle = i as f32 * step;
            let pos = target + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
            waypoints.push(Waypoint::new(pos));
        }
        self.state.set_path(PathType::Circle, waypoints);
    }
}

/// Result of path following update
#[derive(Debug, Clone)]
pub enum PathFollowResult {
    /// No path, agent is idle
    Idle,
    /// Moving toward target
    Moving {
        velocity: Vec3,
        target_direction: Vec3,
        distance_remaining: f32,
    },
    /// Arrived at waypoint
    Arrived {
        activity: Option<WaypointActivity>,
    },
    /// Waiting at waypoint
    Waiting {
        activity: Option<WaypointActivity>,
    },
    /// Path completed
    Complete,
}

/// Path generator for creating meaningful paths
pub struct PathGenerator;

impl PathGenerator {
    /// Generate a schedule path for NPCs
    pub fn schedule_path(entries: &[(Vec3, f32, Option<WaypointActivity>)]) -> Vec<Waypoint> {
        entries
            .iter()
            .map(|(pos, wait, activity)| {
                let mut wp = Waypoint::new(*pos).with_wait(*wait);
                if let Some(act) = activity {
                    wp = wp.with_activity(act.clone());
                }
                wp
            })
            .collect()
    }

    /// Generate a random wander path within bounds
    pub fn wander_path(center: Vec3, radius: f32, num_points: usize, rng_seed: u64) -> Vec<Waypoint> {
        let mut waypoints = Vec::with_capacity(num_points);
        let mut state = rng_seed;

        for _ in 0..num_points {
            // Fast hash-based RNG
            state = state.wrapping_mul(0x5851F42D4C957F2D).wrapping_add(0x14057B7EF767814F);
            let x_rand = ((state >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;

            state = state.wrapping_mul(0x5851F42D4C957F2D).wrapping_add(0x14057B7EF767814F);
            let z_rand = ((state >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;

            let pos = center + Vec3::new(x_rand * radius, 0.0, z_rand * radius);
            waypoints.push(Waypoint::new(pos).with_wait(0.5));
        }

        waypoints
    }

    /// Generate flanking positions around a target
    pub fn flank_positions(
        attacker_pos: Vec3,
        target_pos: Vec3,
        num_flankers: usize,
        flank_distance: f32,
    ) -> Vec<Vec3> {
        let base_dir = (target_pos - attacker_pos).normalize_or_zero();
        let mut positions = Vec::with_capacity(num_flankers);

        for i in 0..num_flankers {
            let angle_offset =
                std::f32::consts::PI * 2.0 / num_flankers as f32 * i as f32 - std::f32::consts::PI;

            let cos_a = angle_offset.cos();
            let sin_a = angle_offset.sin();

            let rotated = Vec3::new(
                base_dir.x * cos_a - base_dir.z * sin_a,
                0.0,
                base_dir.x * sin_a + base_dir.z * cos_a,
            );

            positions.push(target_pos + rotated * flank_distance);
        }

        positions
    }

    /// Generate ambush positions using terrain prediction
    pub fn ambush_position(
        stalker_pos: Vec3,
        target_pos: Vec3,
        target_velocity: Vec3,
        prediction_time: f32,
        ambush_distance: f32,
    ) -> Vec3 {
        // Predict where target will be
        let predicted_pos = target_pos + target_velocity * prediction_time;

        // Position ahead and to the side
        let to_predicted = (predicted_pos - stalker_pos).normalize_or_zero();
        predicted_pos - to_predicted * ambush_distance
    }
}

/// Steering behaviors for smooth movement
pub struct SteeringBehaviors;

impl SteeringBehaviors {
    /// Seek toward target
    pub fn seek(current_pos: Vec3, target_pos: Vec3, max_speed: f32) -> Vec3 {
        let desired = (target_pos - current_pos).normalize_or_zero() * max_speed;
        desired
    }

    /// Flee from target
    pub fn flee(current_pos: Vec3, threat_pos: Vec3, max_speed: f32) -> Vec3 {
        let desired = (current_pos - threat_pos).normalize_or_zero() * max_speed;
        desired
    }

    /// Arrive at target (slow down as we approach)
    pub fn arrive(current_pos: Vec3, target_pos: Vec3, max_speed: f32, slow_radius: f32) -> Vec3 {
        let to_target = target_pos - current_pos;
        let distance = to_target.length();

        if distance < 0.01 {
            return Vec3::ZERO;
        }

        let speed = if distance < slow_radius {
            max_speed * (distance / slow_radius)
        } else {
            max_speed
        };

        (to_target / distance) * speed
    }

    /// Pursue a moving target (predict future position)
    pub fn pursue(
        pursuer_pos: Vec3,
        target_pos: Vec3,
        target_velocity: Vec3,
        max_speed: f32,
    ) -> Vec3 {
        let to_target = target_pos - pursuer_pos;
        let distance = to_target.length();

        // Prediction time based on distance
        let prediction_time = distance / max_speed;
        let predicted_pos = target_pos + target_velocity * prediction_time * 0.5;

        Self::seek(pursuer_pos, predicted_pos, max_speed)
    }

    /// Evade a pursuer
    pub fn evade(
        evader_pos: Vec3,
        pursuer_pos: Vec3,
        pursuer_velocity: Vec3,
        max_speed: f32,
    ) -> Vec3 {
        let to_pursuer = pursuer_pos - evader_pos;
        let distance = to_pursuer.length();

        let prediction_time = distance / max_speed;
        let predicted_pos = pursuer_pos + pursuer_velocity * prediction_time * 0.5;

        Self::flee(evader_pos, predicted_pos, max_speed)
    }

    /// Wander randomly
    pub fn wander(
        current_pos: Vec3,
        current_direction: Vec3,
        wander_radius: f32,
        wander_distance: f32,
        wander_jitter: f32,
        rng_seed: u64,
    ) -> Vec3 {
        // Generate random offset
        let mut state = rng_seed;
        state = state.wrapping_mul(0x5851F42D4C957F2D).wrapping_add(0x14057B7EF767814F);
        let x_jitter = ((state >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;

        state = state.wrapping_mul(0x5851F42D4C957F2D).wrapping_add(0x14057B7EF767814F);
        let z_jitter = ((state >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;

        let jitter = Vec3::new(x_jitter * wander_jitter, 0.0, z_jitter * wander_jitter);

        // Project wander circle in front of agent
        let wander_center = current_pos + current_direction * wander_distance;
        let wander_target = wander_center + jitter.normalize_or_zero() * wander_radius;

        (wander_target - current_pos).normalize_or_zero()
    }

    /// Separation from nearby agents
    pub fn separation(current_pos: Vec3, nearby_positions: &[Vec3], separation_radius: f32) -> Vec3 {
        let mut steering = Vec3::ZERO;
        let mut count = 0;

        for &other_pos in nearby_positions {
            let offset = current_pos - other_pos;
            let distance = offset.length();

            if distance > 0.01 && distance < separation_radius {
                // Weight by inverse distance
                let weight = 1.0 - (distance / separation_radius);
                steering += offset.normalize_or_zero() * weight;
                count += 1;
            }
        }

        if count > 0 {
            steering / count as f32
        } else {
            Vec3::ZERO
        }
    }

    /// Cohesion toward center of group
    pub fn cohesion(current_pos: Vec3, nearby_positions: &[Vec3]) -> Vec3 {
        if nearby_positions.is_empty() {
            return Vec3::ZERO;
        }

        let center: Vec3 = nearby_positions.iter().copied().sum::<Vec3>()
            / nearby_positions.len() as f32;

        (center - current_pos).normalize_or_zero()
    }

    /// Alignment with nearby agents' velocities
    pub fn alignment(nearby_velocities: &[Vec3]) -> Vec3 {
        if nearby_velocities.is_empty() {
            return Vec3::ZERO;
        }

        let avg_velocity: Vec3 =
            nearby_velocities.iter().copied().sum::<Vec3>() / nearby_velocities.len() as f32;

        avg_velocity.normalize_or_zero()
    }
}

/// Obstacle avoidance helper
pub struct ObstacleAvoidance;

impl ObstacleAvoidance {
    /// Simple raycast-style avoidance
    pub fn avoid(
        current_pos: Vec3,
        current_velocity: Vec3,
        obstacles: &[(Vec3, f32)], // (position, radius)
        look_ahead: f32,
    ) -> Vec3 {
        let velocity_length = current_velocity.length();
        if velocity_length < 0.01 {
            return Vec3::ZERO;
        }

        let direction = current_velocity / velocity_length;
        let look_ahead_pos = current_pos + direction * look_ahead;

        let mut avoidance = Vec3::ZERO;

        for &(obs_pos, obs_radius) in obstacles {
            let to_obs = obs_pos - current_pos;
            let dot = to_obs.dot(direction);

            // Only consider obstacles ahead
            if dot > 0.0 && dot < look_ahead {
                let closest_point = current_pos + direction * dot;
                let to_closest = obs_pos - closest_point;
                let dist_to_line = to_closest.length();

                if dist_to_line < obs_radius + 1.0 {
                    // Avoid perpendicular to obstacle
                    let avoid_dir = -to_closest.normalize_or_zero();
                    let urgency = 1.0 - (dist_to_line / (obs_radius + 1.0));
                    avoidance += avoid_dir * urgency;
                }
            }
        }

        avoidance
    }
}
