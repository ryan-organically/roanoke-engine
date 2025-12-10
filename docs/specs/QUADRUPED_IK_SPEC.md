# Quadruped Inverse Kinematics Specification

Ground adaptation system for quadruped animals (horses, deer, wolves, etc.) to naturally conform to terrain variance.

---

## Overview

Baked animations assume flat ground. Runtime IK adjusts leg positions so feet plant correctly on uneven terrain.

```
BAKED ANIMATION          RUNTIME IK LAYER           FINAL POSE
(from Blender)           (engine calculates)        (rendered)
      |                        |                        |
      v                        v                        v
  Leg cycles at      +    Foot targets from    =    Legs match
  flat ground             terrain raycasts          actual ground
```

---

## System Architecture

```
                         QUADRUPED IK SYSTEM
                                 |
        +------------------------+------------------------+
        |                        |                        |
        v                        v                        v
   TERRAIN PROBING         BODY ADJUSTMENT          LEG IK SOLVING
   (4 raycasts per         (root height +          (two-bone IK per
    animal per frame)       pelvis tilt)            leg chain)
```

### Components

| Component | Input | Output | Frequency |
|-----------|-------|--------|-----------|
| Terrain Probing | Animal position, leg offsets | 4x FootPlacement | Per frame |
| Body Adjustment | FootPlacement array | Root height, pelvis rotation | Per frame |
| Two-Bone IK | Hip position, foot target, leg lengths | Hip + knee rotations | Per leg per frame |
| Animation Blend | IK pose, animation pose, blend factor | Final skeleton pose | Per frame |

---

## Data Structures

### FootPlacement

Result of a single terrain probe for one foot.

```rust
#[derive(Default, Clone, Copy)]
pub struct FootPlacement {
    /// World-space Y coordinate where foot should plant
    pub ground_height: f32,

    /// Surface normal at contact point (for foot rotation)
    pub ground_normal: Vec3,

    /// Whether raycast hit terrain (false if over void/water)
    pub is_grounded: bool,
}
```

### QuadrupedIK

Per-animal IK state.

```rust
pub struct QuadrupedIK {
    /// Foot offsets in local space (from skeleton rest pose)
    pub foot_offsets: [Vec3; 4],  // FL, FR, BL, BR

    /// Upper leg bone length (hip to knee)
    pub upper_leg_length: f32,

    /// Lower leg bone length (knee to foot)
    pub lower_leg_length: f32,

    /// Current foot placements from terrain probing
    pub foot_placements: [FootPlacement; 4],

    /// Smoothed root height (prevents jitter)
    pub smoothed_root_height: f32,

    /// Smoothed pelvis tilt
    pub smoothed_pelvis_tilt: Quat,
}
```

### LegBoneIndices

Bone hierarchy indices for one leg.

```rust
pub struct LegBoneIndices {
    pub hip: usize,
    pub upper: usize,
    pub lower: usize,
    pub foot: usize,
}
```

---

## Step 1: Terrain Probing

Cast rays from above each foot's rest position to find ground contact.

### Algorithm

```rust
pub fn probe_terrain_for_feet(
    animal_pos: Vec3,
    animal_rot: Quat,
    foot_offsets: &[Vec3; 4],
    terrain: &Terrain,
) -> [FootPlacement; 4] {
    let mut placements = [FootPlacement::default(); 4];

    for (i, offset) in foot_offsets.iter().enumerate() {
        // Transform foot offset to world space
        let world_foot = animal_pos + animal_rot * *offset;

        // Ray starts above animal, casts down
        let ray_origin = Vec3::new(world_foot.x, animal_pos.y + 2.0, world_foot.z);
        let ray_dir = Vec3::NEG_Y;
        let max_dist = 4.0;

        if let Some(hit) = terrain.raycast(ray_origin, ray_dir, max_dist) {
            placements[i] = FootPlacement {
                ground_height: hit.point.y,
                ground_normal: hit.normal,
                is_grounded: true,
            };
        } else {
            placements[i] = FootPlacement {
                ground_height: animal_pos.y,
                ground_normal: Vec3::Y,
                is_grounded: false,
            };
        }
    }

    placements
}
```

### Performance Notes

- 4 raycasts per quadruped per frame
- With 50 animals visible: 200 raycasts/frame
- Use spatial acceleration (chunk-based terrain lookup)
- Consider staggering: update 2 feet per frame, alternating

---

## Step 2: Body Adjustment

### Root Height Calculation

Position animal root so legs can reach ground without over-extension.

```rust
pub fn calculate_root_height(
    placements: &[FootPlacement; 4],
    leg_length: f32,  // upper + lower
    current_height: f32,
    smoothing: f32,   // 0.1 = smooth, 1.0 = instant
) -> f32 {
    // Average ground height under grounded feet
    let grounded: Vec<f32> = placements.iter()
        .filter(|p| p.is_grounded)
        .map(|p| p.ground_height)
        .collect();

    if grounded.is_empty() {
        return current_height;  // Airborne, maintain height
    }

    let avg_ground = grounded.iter().sum::<f32>() / grounded.len() as f32;

    // Root sits at leg_length above average ground
    // Subtract small offset so legs aren't fully extended
    let target_height = avg_ground + leg_length * 0.95;

    // Smooth to prevent jitter
    lerp(current_height, target_height, smoothing)
}
```

### Pelvis Tilt Calculation

Tilt body to match terrain slope.

```rust
pub fn calculate_pelvis_tilt(
    placements: &[FootPlacement; 4],
    body_length: f32,   // front-to-back distance
    body_width: f32,    // left-to-right distance
    current_tilt: Quat,
    smoothing: f32,
) -> Quat {
    // Front-back tilt (pitch)
    let front_avg = (placements[0].ground_height + placements[1].ground_height) / 2.0;
    let back_avg = (placements[2].ground_height + placements[3].ground_height) / 2.0;
    let pitch = (back_avg - front_avg).atan2(body_length);

    // Left-right tilt (roll)
    let left_avg = (placements[0].ground_height + placements[2].ground_height) / 2.0;
    let right_avg = (placements[1].ground_height + placements[3].ground_height) / 2.0;
    let roll = (right_avg - left_avg).atan2(body_width);

    // Clamp to reasonable angles (prevent extreme tilting)
    let pitch = pitch.clamp(-0.4, 0.4);  // ~23 degrees max
    let roll = roll.clamp(-0.3, 0.3);    // ~17 degrees max

    let target_tilt = Quat::from_euler(EulerRot::XZY, pitch, 0.0, roll);

    // Smooth rotation
    current_tilt.slerp(target_tilt, smoothing)
}
```

---

## Step 3: Two-Bone IK Solver

Solve leg joint angles to reach foot target.

### Geometry

```
       HIP (h) - attached to pelvis
        o
        |\
        | \  upper leg (length: a)
        |  \
        |   o KNEE (k) - solve this angle
        |  /
        | /  lower leg (length: b)
        |/
        o FOOT (f) - target position

   Distance hip to target: d = |f - h|
```

### Algorithm (Law of Cosines)

```rust
pub fn solve_two_bone_ik(
    hip_pos: Vec3,
    foot_target: Vec3,
    upper_length: f32,  // a
    lower_length: f32,  // b
    knee_hint: Vec3,    // Direction knee should bend (forward for front legs, back for rear)
) -> Option<(Quat, Quat)> {
    let to_target = foot_target - hip_pos;
    let dist = to_target.length();

    // Check reachability
    let max_reach = upper_length + lower_length - 0.01;
    let min_reach = (upper_length - lower_length).abs() + 0.01;

    if dist > max_reach || dist < min_reach {
        return None;  // Target unreachable
    }

    // Law of cosines: find knee angle
    // c^2 = a^2 + b^2 - 2ab*cos(C)
    // cos(knee) = (a^2 + b^2 - d^2) / (2ab)
    let knee_cos = (upper_length.powi(2) + lower_length.powi(2) - dist.powi(2))
        / (2.0 * upper_length * lower_length);
    let knee_angle = std::f32::consts::PI - knee_cos.clamp(-1.0, 1.0).acos();

    // Find hip angle offset
    // cos(hip_offset) = (a^2 + d^2 - b^2) / (2ad)
    let hip_cos = (upper_length.powi(2) + dist.powi(2) - lower_length.powi(2))
        / (2.0 * upper_length * dist);
    let hip_offset = hip_cos.clamp(-1.0, 1.0).acos();

    // Build rotations
    let to_target_dir = to_target.normalize();

    // Hip rotation: point toward target, then offset by hip_offset
    let hip_base = Quat::from_rotation_arc(Vec3::NEG_Y, to_target_dir);

    // Apply knee hint to determine bend direction
    let bend_axis = to_target_dir.cross(knee_hint).normalize();
    let hip_rot = hip_base * Quat::from_axis_angle(bend_axis, hip_offset);

    // Knee rotation: simple bend around local X axis
    let knee_rot = Quat::from_rotation_x(knee_angle);

    Some((hip_rot, knee_rot))
}
```

---

## Step 4: IK Blend Factor

Control how much IK overrides animation based on gait and foot phase.

### Gait-Based Blend

| Gait | Base IK Blend | Reason |
|------|---------------|--------|
| Standing | 1.0 | Feet fully planted |
| Walking | 0.8 | Mostly grounded |
| Trotting | 0.6 | Mix of ground/air |
| Cantering | 0.4 | More air time |
| Galloping | 0.2 | Mostly airborne |
| Swimming | 0.0 | No ground contact |
| Jumping | 0.0 | Fully airborne |

### Foot Phase Modulation

Within a gait cycle, feet alternate between planted and lifted.

```rust
pub fn get_ik_blend_for_foot(
    gait: Gait,
    foot_phase: f32,  // 0.0-1.0, where 0.0-0.5 = stance, 0.5-1.0 = swing
) -> f32 {
    let base_blend = match gait {
        Gait::Standing => 1.0,
        Gait::Walking => 0.8,
        Gait::Trotting => 0.6,
        Gait::Cantering => 0.4,
        Gait::Galloping => 0.2,
        Gait::Swimming => 0.0,
    };

    // During stance phase (foot down): full blend
    // During swing phase (foot up): reduced blend
    let phase_multiplier = if foot_phase < 0.5 {
        1.0  // Stance - foot should be on ground
    } else {
        0.2  // Swing - foot in air, mostly animation
    };

    base_blend * phase_multiplier
}
```

---

## Step 5: Final Pose Assembly

Combine animation pose with IK adjustments.

```rust
pub fn apply_quadruped_ik(
    skeleton: &mut Skeleton,
    animation_pose: &Pose,
    ik: &QuadrupedIK,
    leg_bones: &[LegBoneIndices; 4],
    gait: Gait,
    animation_time: f32,
) {
    // Apply body adjustments to root/pelvis
    skeleton.root_position.y = ik.smoothed_root_height;
    skeleton.bones[PELVIS_BONE].local_rotation =
        animation_pose.bones[PELVIS_BONE].local_rotation * ik.smoothed_pelvis_tilt;

    // Process each leg
    for (leg_idx, (placement, bones)) in ik.foot_placements.iter()
        .zip(leg_bones.iter())
        .enumerate()
    {
        if !placement.is_grounded {
            continue;  // Use pure animation for airborne feet
        }

        // Calculate foot phase for this leg
        let phase_offset = [0.0, 0.5, 0.5, 0.0][leg_idx];  // Diagonal pairs
        let foot_phase = (animation_time + phase_offset).fract();
        let ik_blend = get_ik_blend_for_foot(gait, foot_phase);

        if ik_blend < 0.01 {
            continue;  // Effectively no IK
        }

        // Get current hip position (after body adjustment)
        let hip_world = skeleton.get_bone_world_position(bones.hip);

        // IK target: ground point + foot thickness
        let foot_target = Vec3::new(
            hip_world.x,  // Foot stays under hip (X)
            placement.ground_height + FOOT_HEIGHT_OFFSET,
            hip_world.z,  // Foot stays under hip (Z)
        );

        // Knee hint: front legs bend forward, back legs bend backward
        let knee_hint = if leg_idx < 2 { Vec3::Z } else { Vec3::NEG_Z };

        // Solve IK
        if let Some((hip_rot, knee_rot)) = solve_two_bone_ik(
            hip_world,
            foot_target,
            ik.upper_leg_length,
            ik.lower_leg_length,
            knee_hint,
        ) {
            // Blend IK result with animation
            let anim_hip_rot = animation_pose.bones[bones.hip].local_rotation;
            let anim_knee_rot = animation_pose.bones[bones.upper].local_rotation;

            skeleton.bones[bones.hip].local_rotation =
                anim_hip_rot.slerp(hip_rot, ik_blend);
            skeleton.bones[bones.upper].local_rotation =
                anim_knee_rot.slerp(knee_rot, ik_blend);

            // Optional: rotate foot to match ground normal
            let foot_rot = Quat::from_rotation_arc(Vec3::Y, placement.ground_normal);
            skeleton.bones[bones.foot].local_rotation =
                skeleton.bones[bones.foot].local_rotation.slerp(foot_rot, ik_blend * 0.5);
        }
    }
}
```

---

## Bone Naming Convention

For the IK system to work, GLTF models must have consistently named bones:

```
REQUIRED BONE HIERARCHY:

Root
└── Pelvis
    ├── Spine (optional chain)
    │   └── Neck
    │       └── Head
    ├── Front_L_Hip
    │   └── Front_L_Upper
    │       └── Front_L_Lower
    │           └── Front_L_Foot
    ├── Front_R_Hip
    │   └── Front_R_Upper
    │       └── Front_R_Lower
    │           └── Front_R_Foot
    ├── Back_L_Hip
    │   └── Back_L_Upper
    │       └── Back_L_Lower
    │           └── Back_L_Foot
    └── Back_R_Hip
        └── Back_R_Upper
            └── Back_R_Lower
                └── Back_R_Foot
```

### Bone Name Aliases

The loader should recognize common naming conventions:

| Standard Name | Aliases |
|--------------|---------|
| Front_L_Hip | FrontLeft_Hip, FL_Hip, Shoulder.L |
| Front_L_Upper | FrontLeft_Upper, FL_UpperLeg, UpperArm.L |
| Front_L_Lower | FrontLeft_Lower, FL_LowerLeg, LowerArm.L |
| Front_L_Foot | FrontLeft_Foot, FL_Foot, Hand.L |
| Back_L_Hip | BackLeft_Hip, BL_Hip, Hip.L |
| Back_L_Upper | BackLeft_Upper, BL_UpperLeg, UpperLeg.L |
| Back_L_Lower | BackLeft_Lower, BL_LowerLeg, LowerLeg.L |
| Back_L_Foot | BackLeft_Foot, BL_Foot, Foot.L |

---

## Species Configuration

Different quadrupeds have different proportions:

```rust
pub struct QuadrupedConfig {
    pub upper_leg_length: f32,
    pub lower_leg_length: f32,
    pub body_length: f32,      // Front-to-back
    pub body_width: f32,       // Left-to-right
    pub foot_height_offset: f32,
    pub foot_offsets: [Vec3; 4],
}

// Example configurations
pub const HORSE_CONFIG: QuadrupedConfig = QuadrupedConfig {
    upper_leg_length: 0.6,
    lower_leg_length: 0.5,
    body_length: 1.4,
    body_width: 0.5,
    foot_height_offset: 0.05,
    foot_offsets: [
        Vec3::new(-0.25, 0.0, 0.5),   // Front left
        Vec3::new(0.25, 0.0, 0.5),    // Front right
        Vec3::new(-0.25, 0.0, -0.5),  // Back left
        Vec3::new(0.25, 0.0, -0.5),   // Back right
    ],
};

pub const WOLF_CONFIG: QuadrupedConfig = QuadrupedConfig {
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
};
```

---

## Performance Considerations

### Costs Per Frame

| Operation | Cost | With 50 Animals |
|-----------|------|-----------------|
| Terrain raycasts | 4 per animal | 200 raycasts |
| IK solves | 4 per animal | 200 solves |
| Skeleton transforms | ~20 bones | 1000 bone updates |

### Optimizations

1. **Staggered updates**: Update half the feet each frame
2. **Distance LOD**: Skip IK for animals far from camera
3. **Caching**: Reuse foot placements if animal hasn't moved
4. **SIMD**: Batch IK solves with parallel computation

```rust
pub fn should_update_ik(animal: &Animal, camera_pos: Vec3) -> bool {
    let dist = (animal.position - camera_pos).length();

    match dist {
        d if d < 20.0 => true,           // Full IK
        d if d < 50.0 => animal.id % 2 == frame_count % 2,  // Half rate
        _ => false,                       // No IK, pure animation
    }
}
```

---

## Integration Points

### With Animation System

```rust
// In animal update loop:
fn update_animal_visuals(animal: &mut Animal, terrain: &Terrain, dt: f32) {
    // 1. Update animation state machine
    animal.update_animation(dt);

    // 2. Sample animation pose at current time
    let anim_pose = animal.skeleton.sample_animation(
        animal.current_animation,
        animal.animation_time,
    );

    // 3. Probe terrain for IK
    animal.ik.foot_placements = probe_terrain_for_feet(
        animal.position,
        animal.rotation,
        &animal.ik.foot_offsets,
        terrain,
    );

    // 4. Update body adjustments (smoothed)
    animal.ik.smoothed_root_height = calculate_root_height(...);
    animal.ik.smoothed_pelvis_tilt = calculate_pelvis_tilt(...);

    // 5. Apply IK to skeleton
    apply_quadruped_ik(
        &mut animal.skeleton,
        &anim_pose,
        &animal.ik,
        &animal.leg_bones,
        animal.gait,
        animal.animation_time,
    );

    // 6. Upload skeleton transforms to GPU
    animal.skeleton.compute_final_transforms();
}
```

### With Rendering Pipeline

The `animal_model_pipeline.rs` needs to receive per-bone transforms:

```rust
// Instance data expanded for skeletal animation
pub struct AnimalInstance {
    pub model_matrix: [[f32; 4]; 4],
    pub bone_matrices: [[[f32; 4]; 4]; MAX_BONES],  // NEW
    pub color: [f32; 3],
    pub emissive: f32,
}
```

---

## Testing Checklist

- [ ] Horse stands correctly on flat ground
- [ ] Horse legs adapt to gentle slope (10 degrees)
- [ ] Horse legs adapt to steep slope (30 degrees)
- [ ] Horse body tilts appropriately on slope
- [ ] Legs don't hyper-extend on dips
- [ ] Legs don't clip through rises
- [ ] Smooth transitions when terrain changes
- [ ] IK blends correctly during trot animation
- [ ] IK reduces correctly during gallop animation
- [ ] Performance acceptable with 50 animals
