//! Biped Inverse Kinematics for player ground adaptation
//!
//! Simpler than quadruped IK - only 2 legs, no body pitch, just hip roll.
//! Reuses the two-bone solver from quadruped_ik.

use glam::{Quat, Vec3};

// Reuse the two-bone IK solver from quadruped system
pub use crate::animals::quadruped_ik::{solve_two_bone_ik, FootPlacement, TwoBoneIKResult};

/// Player movement state for IK blend calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerMoveState {
    #[default]
    Idle,
    Walking,
    Running,
    Jumping,
}

/// Leg bone indices for biped skeleton
#[derive(Debug, Clone, Copy, Default)]
pub struct BipedLegBones {
    pub hip: usize,
    pub knee: usize,
    pub foot: usize,
}

/// Biped IK constants (human proportions)
pub mod constants {
    use glam::Vec3;

    pub const UPPER_LEG_LENGTH: f32 = 0.45;
    pub const LOWER_LEG_LENGTH: f32 = 0.42;
    pub const HIP_WIDTH: f32 = 0.3;
    pub const FOOT_HEIGHT: f32 = 0.05;
    pub const SMOOTHING: f32 = 0.15;

    /// Foot offsets from root in rest pose [Left, Right]
    pub const FOOT_OFFSETS: [Vec3; 2] = [
        Vec3::new(-0.15, 0.0, 0.0), // Left
        Vec3::new(0.15, 0.0, 0.0),  // Right
    ];

    /// Total leg length
    pub const LEG_LENGTH: f32 = UPPER_LEG_LENGTH + LOWER_LEG_LENGTH;
}

/// Biped IK runtime state
#[derive(Debug, Clone)]
pub struct BipedIK {
    /// Current foot placements from terrain probing [Left, Right]
    pub foot_placements: [FootPlacement; 2],

    /// Smoothed root height (prevents jitter)
    pub smoothed_root_height: f32,

    /// Smoothed hip roll angle (radians)
    pub smoothed_hip_roll: f32,

    /// Current IK blend factor
    pub ik_blend: f32,
}

impl Default for BipedIK {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl BipedIK {
    /// Create new biped IK state
    pub fn new(initial_height: f32) -> Self {
        Self {
            foot_placements: [FootPlacement::default(); 2],
            smoothed_root_height: initial_height,
            smoothed_hip_roll: 0.0,
            ik_blend: 1.0,
        }
    }

    /// Probe terrain for both feet
    ///
    /// # Arguments
    /// * `player_pos` - World position of player root
    /// * `player_yaw` - Player facing direction (radians)
    /// * `height_fn` - Function returning terrain height at (x, z)
    pub fn probe_terrain(
        &mut self,
        player_pos: Vec3,
        player_yaw: f32,
        height_fn: impl Fn(f32, f32) -> f32,
    ) {
        let player_rot = Quat::from_rotation_y(player_yaw);

        for (i, offset) in constants::FOOT_OFFSETS.iter().enumerate() {
            // Transform foot offset to world space
            let world_foot = player_pos + player_rot * *offset;

            // Get terrain height at foot position
            let ground_height = height_fn(world_foot.x, world_foot.z);

            // Estimate surface normal with small samples
            let dx = 0.1;
            let dz = 0.1;
            let h_px = height_fn(world_foot.x + dx, world_foot.z);
            let h_nx = height_fn(world_foot.x - dx, world_foot.z);
            let h_pz = height_fn(world_foot.x, world_foot.z + dz);
            let h_nz = height_fn(world_foot.x, world_foot.z - dz);

            let normal = Vec3::new(
                (h_nx - h_px) / (2.0 * dx),
                1.0,
                (h_nz - h_pz) / (2.0 * dz),
            )
            .normalize();

            self.foot_placements[i] = FootPlacement {
                ground_height,
                ground_normal: normal,
                is_grounded: true,
            };
        }
    }

    /// Calculate and smooth root height
    pub fn update_root_height(&mut self, smoothing: f32) {
        let avg_ground = (self.foot_placements[0].ground_height
            + self.foot_placements[1].ground_height)
            / 2.0;

        // Root sits at leg length above ground (slightly bent at 95%)
        let target = avg_ground + constants::LEG_LENGTH * 0.95;

        self.smoothed_root_height = lerp(self.smoothed_root_height, target, smoothing);
    }

    /// Calculate and smooth hip roll (left-right tilt)
    ///
    /// No pitch for bipeds - humans stay upright
    pub fn update_hip_roll(&mut self, smoothing: f32) {
        let height_diff =
            self.foot_placements[1].ground_height - self.foot_placements[0].ground_height;

        // Roll angle from height difference and hip width
        let target_roll = (height_diff / constants::HIP_WIDTH)
            .atan()
            .clamp(-0.25, 0.25); // ~14° max

        self.smoothed_hip_roll = lerp(self.smoothed_hip_roll, target_roll, smoothing);
    }

    /// Get IK blend factor based on player state
    pub fn get_blend_for_state(&self, state: PlayerMoveState, on_ground: bool) -> f32 {
        if !on_ground {
            return 0.0;
        }

        match state {
            PlayerMoveState::Idle => 1.0,
            PlayerMoveState::Walking => 0.8,
            PlayerMoveState::Running => 0.5,
            PlayerMoveState::Jumping => 0.0,
        }
    }

    /// Update IK blend based on player state
    pub fn update_blend(&mut self, state: PlayerMoveState, on_ground: bool) {
        self.ik_blend = self.get_blend_for_state(state, on_ground);
    }

    /// Get hip roll as quaternion
    pub fn get_hip_roll_quat(&self) -> Quat {
        Quat::from_rotation_z(self.smoothed_hip_roll)
    }

    /// Full IK update - call once per frame
    ///
    /// # Arguments
    /// * `player_pos` - World position of player
    /// * `player_yaw` - Player facing direction (radians)
    /// * `state` - Current movement state
    /// * `on_ground` - Whether player is on ground
    /// * `height_fn` - Terrain height function
    pub fn update(
        &mut self,
        player_pos: Vec3,
        player_yaw: f32,
        state: PlayerMoveState,
        on_ground: bool,
        height_fn: impl Fn(f32, f32) -> f32,
    ) {
        // Update blend first
        self.update_blend(state, on_ground);

        // Skip expensive calculations if blend is too low
        if self.ik_blend < 0.05 {
            return;
        }

        // Probe terrain
        self.probe_terrain(player_pos, player_yaw, &height_fn);

        // Update body adjustments
        self.update_root_height(constants::SMOOTHING);
        self.update_hip_roll(constants::SMOOTHING);
    }

    /// Solve IK for a single leg
    ///
    /// # Arguments
    /// * `hip_world_pos` - World position of hip joint
    /// * `foot_index` - 0 for left, 1 for right
    ///
    /// # Returns
    /// IK result with hip and knee rotations, or None if unreachable
    pub fn solve_leg(&self, hip_world_pos: Vec3, foot_index: usize) -> Option<TwoBoneIKResult> {
        let placement = &self.foot_placements[foot_index];
        if !placement.is_grounded {
            return None;
        }

        // Target: ground height + foot offset, directly below hip
        let foot_target = Vec3::new(
            hip_world_pos.x,
            placement.ground_height + constants::FOOT_HEIGHT,
            hip_world_pos.z,
        );

        // Knees bend forward (positive Z in local space)
        let result = solve_two_bone_ik(
            hip_world_pos,
            foot_target,
            constants::UPPER_LEG_LENGTH,
            constants::LOWER_LEG_LENGTH,
            Vec3::Z,
        );

        if result.success {
            Some(result)
        } else {
            None
        }
    }
}

/// Get IK blend for a specific foot based on animation phase
///
/// # Arguments
/// * `base_blend` - Overall IK blend for current state
/// * `foot_phase` - 0.0-1.0 where 0.0-0.5 = stance, 0.5-1.0 = swing
pub fn get_foot_ik_blend(base_blend: f32, foot_phase: f32) -> f32 {
    // During stance (foot planted): full blend
    // During swing (foot lifting): reduced blend
    let phase_mult = if foot_phase < 0.5 { 1.0 } else { 0.2 };
    base_blend * phase_mult
}

/// Calculate foot phase in walk/run cycle
///
/// # Arguments
/// * `animation_time` - Current animation time
/// * `foot_index` - 0 for left, 1 for right
/// * `cycle_duration` - Duration of one full walk cycle
pub fn calculate_foot_phase(animation_time: f32, foot_index: usize, cycle_duration: f32) -> f32 {
    // Left and right feet are opposite phase
    let phase_offset = if foot_index == 0 { 0.0 } else { 0.5 };
    (animation_time / cycle_duration + phase_offset).fract()
}

/// Linear interpolation helper
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biped_ik_creation() {
        let ik = BipedIK::new(1.0);
        assert_eq!(ik.smoothed_root_height, 1.0);
        assert_eq!(ik.smoothed_hip_roll, 0.0);
        assert_eq!(ik.ik_blend, 1.0);
    }

    #[test]
    fn test_blend_for_state() {
        let ik = BipedIK::new(1.0);

        assert_eq!(ik.get_blend_for_state(PlayerMoveState::Idle, true), 1.0);
        assert_eq!(ik.get_blend_for_state(PlayerMoveState::Walking, true), 0.8);
        assert_eq!(ik.get_blend_for_state(PlayerMoveState::Running, true), 0.5);
        assert_eq!(ik.get_blend_for_state(PlayerMoveState::Jumping, true), 0.0);

        // Not on ground = no IK
        assert_eq!(ik.get_blend_for_state(PlayerMoveState::Idle, false), 0.0);
    }

    #[test]
    fn test_foot_phase_opposite() {
        let left = calculate_foot_phase(0.0, 0, 1.0);
        let right = calculate_foot_phase(0.0, 1, 1.0);

        // Left and right should be 0.5 apart
        assert!((right - left - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_terrain_probing() {
        let mut ik = BipedIK::new(1.0);

        // Flat terrain at height 0
        ik.probe_terrain(Vec3::new(0.0, 1.0, 0.0), 0.0, |_, _| 0.0);

        assert!(ik.foot_placements[0].is_grounded);
        assert!(ik.foot_placements[1].is_grounded);
        assert_eq!(ik.foot_placements[0].ground_height, 0.0);
        assert_eq!(ik.foot_placements[1].ground_height, 0.0);
    }

    #[test]
    fn test_hip_roll_on_slope() {
        let mut ik = BipedIK::new(1.0);

        // Sloped terrain - right foot higher
        ik.probe_terrain(Vec3::new(0.0, 1.0, 0.0), 0.0, |x, _| {
            if x > 0.0 { 0.1 } else { 0.0 }
        });

        ik.update_hip_roll(1.0); // Instant update

        // Roll should be positive (tilting right up)
        assert!(ik.smoothed_hip_roll > 0.0);
    }
}
