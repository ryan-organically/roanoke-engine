//! Quadruped Inverse Kinematics for ground adaptation
//!
//! Runtime IK system that adjusts quadruped leg positions to match terrain.
//! Works as an additive layer on top of baked animations.

use glam::{Quat, Vec3};

/// Result of terrain probe for a single foot
#[derive(Debug, Clone, Copy, Default)]
pub struct FootPlacement {
    /// World-space Y coordinate where foot should plant
    pub ground_height: f32,
    /// Surface normal at contact point (for foot rotation)
    pub ground_normal: Vec3,
    /// Whether raycast hit valid terrain
    pub is_grounded: bool,
}

/// Leg bone indices in the skeleton
#[derive(Debug, Clone, Copy, Default)]
pub struct LegBoneIndices {
    pub hip: usize,
    pub upper: usize,
    pub lower: usize,
    pub foot: usize,
}

/// Configuration for a quadruped species
#[derive(Debug, Clone)]
pub struct QuadrupedConfig {
    /// Upper leg bone length (hip to knee)
    pub upper_leg_length: f32,
    /// Lower leg bone length (knee to foot)
    pub lower_leg_length: f32,
    /// Front-to-back body length
    pub body_length: f32,
    /// Left-to-right body width
    pub body_width: f32,
    /// Height offset for foot above ground
    pub foot_height_offset: f32,
    /// Foot positions in local space [FL, FR, BL, BR]
    pub foot_offsets: [Vec3; 4],
}

impl QuadrupedConfig {
    /// Horse proportions
    pub fn horse() -> Self {
        Self {
            upper_leg_length: 0.6,
            lower_leg_length: 0.5,
            body_length: 1.4,
            body_width: 0.5,
            foot_height_offset: 0.05,
            foot_offsets: [
                Vec3::new(-0.25, 0.0, 0.5),  // Front left
                Vec3::new(0.25, 0.0, 0.5),   // Front right
                Vec3::new(-0.25, 0.0, -0.5), // Back left
                Vec3::new(0.25, 0.0, -0.5),  // Back right
            ],
        }
    }

    /// Wolf proportions
    pub fn wolf() -> Self {
        Self {
            upper_leg_length: 0.35,
            lower_leg_length: 0.3,
            body_length: 0.8,
            body_width: 0.25,
            foot_height_offset: 0.03,
            foot_offsets: [
                Vec3::new(-0.12, 0.0, 0.3),
                Vec3::new(0.12, 0.0, 0.3),
                Vec3::new(-0.12, 0.0, -0.3),
                Vec3::new(0.12, 0.0, -0.3),
            ],
        }
    }

    /// Deer proportions
    pub fn deer() -> Self {
        Self {
            upper_leg_length: 0.45,
            lower_leg_length: 0.4,
            body_length: 1.0,
            body_width: 0.35,
            foot_height_offset: 0.04,
            foot_offsets: [
                Vec3::new(-0.18, 0.0, 0.4),
                Vec3::new(0.18, 0.0, 0.4),
                Vec3::new(-0.18, 0.0, -0.4),
                Vec3::new(0.18, 0.0, -0.4),
            ],
        }
    }

    /// Total leg length (for reach calculations)
    pub fn total_leg_length(&self) -> f32 {
        self.upper_leg_length + self.lower_leg_length
    }
}

/// Per-animal IK runtime state
#[derive(Debug, Clone)]
pub struct QuadrupedIK {
    /// Configuration for this species
    pub config: QuadrupedConfig,
    /// Current foot placements from terrain probing
    pub foot_placements: [FootPlacement; 4],
    /// Smoothed root height (prevents jitter)
    pub smoothed_root_height: f32,
    /// Smoothed pelvis tilt
    pub smoothed_pelvis_tilt: Quat,
    /// IK blend factor (0 = animation only, 1 = full IK)
    pub ik_blend: f32,
}

impl QuadrupedIK {
    /// Create new IK state for a quadruped
    pub fn new(config: QuadrupedConfig, initial_height: f32) -> Self {
        Self {
            config,
            foot_placements: [FootPlacement::default(); 4],
            smoothed_root_height: initial_height,
            smoothed_pelvis_tilt: Quat::IDENTITY,
            ik_blend: 1.0,
        }
    }

    /// Probe terrain for all four feet
    pub fn probe_terrain(
        &mut self,
        animal_pos: Vec3,
        animal_rot: Quat,
        height_fn: impl Fn(f32, f32) -> f32,
    ) {
        for (i, offset) in self.config.foot_offsets.iter().enumerate() {
            // Transform foot offset to world space
            let world_foot = animal_pos + animal_rot * *offset;

            // Get terrain height at foot position
            let ground_height = height_fn(world_foot.x, world_foot.z);

            // Simple normal estimation (could be improved with multi-sample)
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
                is_grounded: true, // Assume always grounded for now
            };
        }
    }

    /// Calculate root height adjustment
    pub fn update_root_height(&mut self, smoothing: f32) {
        let grounded: Vec<f32> = self
            .foot_placements
            .iter()
            .filter(|p| p.is_grounded)
            .map(|p| p.ground_height)
            .collect();

        if grounded.is_empty() {
            return;
        }

        let avg_ground = grounded.iter().sum::<f32>() / grounded.len() as f32;
        let leg_length = self.config.total_leg_length();

        // Root sits at leg_length above average ground (slightly bent)
        let target_height = avg_ground + leg_length * 0.92;

        // Smooth transition
        self.smoothed_root_height =
            lerp(self.smoothed_root_height, target_height, smoothing);
    }

    /// Calculate pelvis tilt to match slope
    pub fn update_pelvis_tilt(&mut self, smoothing: f32) {
        let placements = &self.foot_placements;

        // Front-back tilt (pitch)
        let front_avg =
            (placements[0].ground_height + placements[1].ground_height) / 2.0;
        let back_avg =
            (placements[2].ground_height + placements[3].ground_height) / 2.0;
        let pitch = (back_avg - front_avg).atan2(self.config.body_length);

        // Left-right tilt (roll)
        let left_avg =
            (placements[0].ground_height + placements[2].ground_height) / 2.0;
        let right_avg =
            (placements[1].ground_height + placements[3].ground_height) / 2.0;
        let roll = (right_avg - left_avg).atan2(self.config.body_width);

        // Clamp to reasonable angles
        let pitch = pitch.clamp(-0.4, 0.4); // ~23 degrees max
        let roll = roll.clamp(-0.3, 0.3); // ~17 degrees max

        let target_tilt = Quat::from_euler(glam::EulerRot::XZY, pitch, 0.0, roll);

        // Smooth rotation
        self.smoothed_pelvis_tilt = self.smoothed_pelvis_tilt.slerp(target_tilt, smoothing);
    }

    /// Get IK blend factor based on gait and speed
    pub fn get_blend_for_gait(&self, speed: f32, is_airborne: bool) -> f32 {
        if is_airborne {
            return 0.0;
        }

        // Higher speed = less IK (more animation freedom)
        match speed {
            s if s < 0.5 => 1.0,  // Standing/grazing
            s if s < 3.0 => 0.8,  // Walking
            s if s < 6.0 => 0.6,  // Trotting
            s if s < 10.0 => 0.3, // Cantering
            _ => 0.15,            // Galloping
        }
    }
}

/// Two-bone IK solver result
#[derive(Debug, Clone, Copy)]
pub struct TwoBoneIKResult {
    pub hip_rotation: Quat,
    pub knee_rotation: Quat,
    pub success: bool,
}

/// Solve two-bone IK for a single leg
///
/// # Arguments
/// * `hip_pos` - World position of hip joint
/// * `foot_target` - World position where foot should be
/// * `upper_length` - Length of upper leg bone
/// * `lower_length` - Length of lower leg bone
/// * `knee_forward` - Direction knee should bend (Z for front legs, -Z for back)
pub fn solve_two_bone_ik(
    hip_pos: Vec3,
    foot_target: Vec3,
    upper_length: f32,
    lower_length: f32,
    knee_forward: Vec3,
) -> TwoBoneIKResult {
    let to_target = foot_target - hip_pos;
    let dist = to_target.length();

    // Check reachability
    let max_reach = upper_length + lower_length - 0.01;
    let min_reach = (upper_length - lower_length).abs() + 0.01;

    if dist > max_reach || dist < min_reach || dist < 0.001 {
        return TwoBoneIKResult {
            hip_rotation: Quat::IDENTITY,
            knee_rotation: Quat::IDENTITY,
            success: false,
        };
    }

    // Law of cosines for knee angle
    let knee_cos = (upper_length.powi(2) + lower_length.powi(2) - dist.powi(2))
        / (2.0 * upper_length * lower_length);
    let knee_angle = std::f32::consts::PI - knee_cos.clamp(-1.0, 1.0).acos();

    // Hip angle offset
    let hip_cos = (upper_length.powi(2) + dist.powi(2) - lower_length.powi(2))
        / (2.0 * upper_length * dist);
    let hip_offset = hip_cos.clamp(-1.0, 1.0).acos();

    // Build rotations
    let to_target_dir = to_target.normalize();

    // Hip points toward target
    let hip_base = Quat::from_rotation_arc(Vec3::NEG_Y, to_target_dir);

    // Determine bend axis from knee hint
    let bend_axis = to_target_dir.cross(knee_forward).normalize_or_zero();
    let hip_rotation = if bend_axis.length_squared() > 0.001 {
        hip_base * Quat::from_axis_angle(bend_axis, hip_offset)
    } else {
        hip_base
    };

    // Knee rotation around local X
    let knee_rotation = Quat::from_rotation_x(knee_angle);

    TwoBoneIKResult {
        hip_rotation,
        knee_rotation,
        success: true,
    }
}

/// Foot phase in animation cycle (0-1, where 0-0.5 = stance, 0.5-1 = swing)
pub fn calculate_foot_phase(animation_time: f32, leg_index: usize, cycle_duration: f32) -> f32 {
    // Diagonal pairs move together
    let phase_offset = match leg_index {
        0 => 0.0,   // Front left
        1 => 0.5,   // Front right (opposite phase)
        2 => 0.5,   // Back left (opposite phase)
        3 => 0.0,   // Back right (same as front left)
        _ => 0.0,
    };

    let raw_phase = (animation_time / cycle_duration + phase_offset).fract();
    raw_phase
}

/// Get IK blend for a specific foot based on phase
pub fn get_foot_ik_blend(base_blend: f32, foot_phase: f32) -> f32 {
    // During stance (foot down): full blend
    // During swing (foot up): reduced blend
    let phase_multiplier = if foot_phase < 0.5 {
        1.0 // Stance
    } else {
        0.3 // Swing
    };

    base_blend * phase_multiplier
}

/// Linear interpolation helper
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_bone_ik_reachable() {
        let result = solve_two_bone_ik(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 0.2),
            0.6,
            0.5,
            Vec3::Z,
        );
        assert!(result.success);
    }

    #[test]
    fn test_two_bone_ik_unreachable() {
        let result = solve_two_bone_ik(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0), // Too far
            0.6,
            0.5,
            Vec3::Z,
        );
        assert!(!result.success);
    }

    #[test]
    fn test_foot_phase_diagonal() {
        // Front left and back right should have same phase
        let fl = calculate_foot_phase(0.0, 0, 1.0);
        let br = calculate_foot_phase(0.0, 3, 1.0);
        assert!((fl - br).abs() < 0.001);

        // Front right and back left should have same phase
        let fr = calculate_foot_phase(0.0, 1, 1.0);
        let bl = calculate_foot_phase(0.0, 2, 1.0);
        assert!((fr - bl).abs() < 0.001);
    }
}
