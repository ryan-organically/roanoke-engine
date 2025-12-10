# Biped Inverse Kinematics Specification

Ground adaptation for the player character so feet plant correctly on uneven terrain.

---

## Overview

Baked animations assume flat ground. Runtime IK adjusts leg positions so feet plant on terrain.

```
ANIMATION POSE      +      TERRAIN IK      =      FINAL POSE
(flat ground)              (raycasts)             (terrain-matched)
```

---

## Data Structures

```rust
#[derive(Default, Clone, Copy)]
pub struct FootPlacement {
    pub ground_height: f32,
    pub ground_normal: Vec3,
    pub is_grounded: bool,
}

pub struct BipedIK {
    /// Left/Right foot offsets from root (rest pose)
    pub foot_offsets: [Vec3; 2],  // [Left, Right]

    /// Leg segment lengths
    pub upper_leg_length: f32,
    pub lower_leg_length: f32,

    /// Current terrain contact
    pub foot_placements: [FootPlacement; 2],

    /// Smoothed values (prevent jitter)
    pub smoothed_root_height: f32,
    pub smoothed_hip_roll: f32,
}
```

---

## Step 1: Terrain Probing

Two raycasts per frame—one under each foot.

```rust
pub fn probe_feet(
    player_pos: Vec3,
    player_rot: Quat,
    foot_offsets: &[Vec3; 2],
    terrain: &Terrain,
) -> [FootPlacement; 2] {
    let mut placements = [FootPlacement::default(); 2];

    for (i, offset) in foot_offsets.iter().enumerate() {
        let world_foot = player_pos + player_rot * *offset;
        let ray_origin = Vec3::new(world_foot.x, player_pos.y + 1.5, world_foot.z);

        if let Some(hit) = terrain.raycast(ray_origin, Vec3::NEG_Y, 3.0) {
            placements[i] = FootPlacement {
                ground_height: hit.point.y,
                ground_normal: hit.normal,
                is_grounded: true,
            };
        }
    }

    placements
}
```

---

## Step 2: Body Adjustment

### Root Height

Position root so legs can reach ground without over-extending.

```rust
pub fn calculate_root_height(
    placements: &[FootPlacement; 2],
    leg_length: f32,
    current: f32,
    smoothing: f32,
) -> f32 {
    let avg_ground = (placements[0].ground_height + placements[1].ground_height) / 2.0;
    let target = avg_ground + leg_length * 0.95;
    lerp(current, target, smoothing)
}
```

### Hip Roll

Tilt hips left/right to match slope. No pitch needed—humans stay upright.

```rust
pub fn calculate_hip_roll(
    placements: &[FootPlacement; 2],
    hip_width: f32,
    current_roll: f32,
    smoothing: f32,
) -> f32 {
    let height_diff = placements[1].ground_height - placements[0].ground_height;
    let target_roll = (height_diff / hip_width).atan().clamp(-0.25, 0.25);  // ~14° max
    lerp(current_roll, target_roll, smoothing)
}
```

---

## Step 3: Two-Bone IK Solver

Same law of cosines approach as quadruped. Knees always bend forward.

```rust
pub fn solve_leg_ik(
    hip_pos: Vec3,
    foot_target: Vec3,
    upper_len: f32,
    lower_len: f32,
) -> Option<(Quat, Quat)> {
    let to_target = foot_target - hip_pos;
    let dist = to_target.length();

    let max_reach = upper_len + lower_len - 0.01;
    if dist > max_reach {
        return None;  // Unreachable
    }

    // Law of cosines for knee angle
    let knee_cos = (upper_len.powi(2) + lower_len.powi(2) - dist.powi(2))
        / (2.0 * upper_len * lower_len);
    let knee_angle = PI - knee_cos.clamp(-1.0, 1.0).acos();

    // Hip offset angle
    let hip_cos = (upper_len.powi(2) + dist.powi(2) - lower_len.powi(2))
        / (2.0 * upper_len * dist);
    let hip_offset = hip_cos.clamp(-1.0, 1.0).acos();

    let to_target_dir = to_target.normalize();
    let hip_base = Quat::from_rotation_arc(Vec3::NEG_Y, to_target_dir);

    // Knee bends forward (positive Z)
    let bend_axis = to_target_dir.cross(Vec3::Z).normalize();
    let hip_rot = hip_base * Quat::from_axis_angle(bend_axis, hip_offset);
    let knee_rot = Quat::from_rotation_x(knee_angle);

    Some((hip_rot, knee_rot))
}
```

---

## Step 4: IK Blend

Reduce IK influence when feet are lifting.

| State | IK Blend |
|-------|----------|
| Standing | 1.0 |
| Walking | 0.8 |
| Running | 0.5 |
| Jumping | 0.0 |

```rust
pub fn get_foot_ik_blend(state: PlayerState, foot_phase: f32) -> f32 {
    let base = match state {
        PlayerState::Idle => 1.0,
        PlayerState::Walking => 0.8,
        PlayerState::Running => 0.5,
        PlayerState::Jumping => 0.0,
    };

    // Reduce during swing phase (foot in air)
    let phase_mult = if foot_phase < 0.5 { 1.0 } else { 0.2 };
    base * phase_mult
}
```

---

## Step 5: Apply IK

```rust
pub fn apply_biped_ik(
    skeleton: &mut Skeleton,
    anim_pose: &Pose,
    ik: &BipedIK,
    state: PlayerState,
    anim_time: f32,
) {
    // Body adjustments
    skeleton.root_position.y = ik.smoothed_root_height;
    skeleton.bones[PELVIS].local_rotation *= Quat::from_rotation_z(ik.smoothed_hip_roll);

    // Each leg
    for (i, placement) in ik.foot_placements.iter().enumerate() {
        if !placement.is_grounded {
            continue;
        }

        let phase_offset = if i == 0 { 0.0 } else { 0.5 };
        let foot_phase = (anim_time + phase_offset).fract();
        let blend = get_foot_ik_blend(state, foot_phase);

        if blend < 0.01 {
            continue;
        }

        let hip_world = skeleton.get_bone_world_position(LEG_BONES[i].hip);
        let foot_target = Vec3::new(
            hip_world.x,
            placement.ground_height + FOOT_HEIGHT,
            hip_world.z,
        );

        if let Some((hip_rot, knee_rot)) = solve_leg_ik(
            hip_world,
            foot_target,
            ik.upper_leg_length,
            ik.lower_leg_length,
        ) {
            let bones = &LEG_BONES[i];
            skeleton.bones[bones.hip].local_rotation =
                anim_pose.bones[bones.hip].local_rotation.slerp(hip_rot, blend);
            skeleton.bones[bones.knee].local_rotation =
                anim_pose.bones[bones.knee].local_rotation.slerp(knee_rot, blend);
        }
    }
}
```

---

## Bone Requirements

```
Root
└── Pelvis
    ├── Spine...
    ├── Hip.L
    │   └── UpperLeg.L
    │       └── LowerLeg.L
    │           └── Foot.L
    └── Hip.R
        └── UpperLeg.R
            └── LowerLeg.R
                └── Foot.R
```

---

## Constants

```rust
const UPPER_LEG_LENGTH: f32 = 0.45;
const LOWER_LEG_LENGTH: f32 = 0.42;
const HIP_WIDTH: f32 = 0.3;
const FOOT_HEIGHT: f32 = 0.05;
const SMOOTHING: f32 = 0.15;

const FOOT_OFFSETS: [Vec3; 2] = [
    Vec3::new(-0.15, 0.0, 0.0),  // Left
    Vec3::new(0.15, 0.0, 0.0),   // Right
];
```

---

## Performance

| Operation | Count |
|-----------|-------|
| Raycasts | 2/frame |
| IK solves | 2/frame |
| Bone transforms | ~8 |

Single player = negligible cost. No LOD needed.
