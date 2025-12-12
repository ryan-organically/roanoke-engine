# Horse Animation Blender Bridge Specification

**Version:** 2.0.0
**Last Updated:** 2024-12-11
**Status:** PARTIAL - Engine ready, model needs updates

---

## Spec Version History

| Version | Date       | Changes                                                |
|---------|------------|--------------------------------------------------------|
| 2.0.0   | 2024-12-11 | Added versioning, engine pipeline status, GPU skinning |
| 1.0.0   | 2024-xx-xx | Initial spec                                           |

---

## Current Model Status

### Horse.gltf Inventory

```yaml
file: assets/models/animals/Horse.gltf
size: 3.6 MB
format: GLTF (not GLB)
texture: None (uses baseColorFactor materials)
skeleton: Yes (AnimalArmature)
joint_count: ~50+ bones
skinned: Yes (JOINTS_0, WEIGHTS_0 attributes present)
```

### Available Animations (Actual)

| Animation Name  | Duration | Loop | Notes                    |
|-----------------|----------|------|--------------------------|
| Idle            | ?        | Yes  | Standing idle            |
| Idle_2          | ?        | Yes  | Alternate idle           |
| Idle_Headlow    | ?        | Yes  | Head lowered (grazing?)  |
| Idle_HitReact1  | ?        | No   | Damage reaction          |
| Idle_HitReact2  | ?        | No   | Damage reaction alt      |
| Walk            | ?        | Yes  | Walking gait             |
| Gallop          | ?        | Yes  | Running gait             |
| Gallop_Jump     | ?        | No   | Jump during gallop       |
| Jump_toIdle     | ?        | No   | Landing transition       |

### Expected vs Actual Mapping

| Expected (Spec 1.0) | Actual Available | Status      |
|---------------------|------------------|-------------|
| Horse_Graze         | Idle_Headlow     | RENAMED     |
| Horse_Trot          | Walk             | RENAMED     |
| Horse_Gallop        | Gallop           | OK          |
| Horse_Turn          | (none)           | MISSING     |
| (none)              | Idle             | EXTRA - USE |
| (none)              | Idle_2           | EXTRA       |
| (none)              | Idle_HitReact*   | EXTRA       |

---

## Engine Pipeline Status

### Loader Status (gltf_loader.rs)

```yaml
skeleton_loading: IMPLEMENTED
animation_loading: IMPLEMENTED
joint_weights: IMPLEMENTED (JOINTS_0, WEIGHTS_0)
inverse_bind_matrices: IMPLEMENTED
keyframe_sampling: IMPLEMENTED
```

### Render Pipeline Status (animal_model_pipeline.rs)

```yaml
animation_storage: IMPLEMENTED (SkeletonGpu, AnimationGpu)
animation_upload: IMPLEMENTED (upload_species_animation)
animation_sampling: IMPLEMENTED (sample_animation_root_transform)
gpu_skinning_shader: NOT IMPLEMENTED
skinned_vertex_buffer: NOT IMPLEMENTED
joint_matrix_buffer: NOT IMPLEMENTED
```

### Shader Status (animal_model.wgsl)

```yaml
current_vertex_format: position, normal, uv
skinned_vertex_format: NOT IMPLEMENTED (needs joints, weights)
joint_matrix_uniform: NOT IMPLEMENTED
skinning_calculation: NOT IMPLEMENTED
```

---

## Required Implementation for GPU Skinning

### 1. Skinned Vertex Format

```wgsl
struct SkinnedVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) joints: vec4<u32>,   // NEW
    @location(4) weights: vec4<f32>,  // NEW
}
```

### 2. Joint Matrix Buffer

```wgsl
const MAX_JOINTS: u32 = 64u;

@group(3) @binding(0)
var<uniform> joint_matrices: array<mat4x4<f32>, MAX_JOINTS>;
```

### 3. Skinning Calculation

```wgsl
fn apply_skinning(
    position: vec3<f32>,
    normal: vec3<f32>,
    joints: vec4<u32>,
    weights: vec4<f32>
) -> VertexOutput {
    var skinned_pos = vec3<f32>(0.0);
    var skinned_normal = vec3<f32>(0.0);

    for (var i = 0u; i < 4u; i = i + 1u) {
        let joint_idx = joints[i];
        let weight = weights[i];

        if (weight > 0.0) {
            let joint_mat = joint_matrices[joint_idx];
            skinned_pos += weight * (joint_mat * vec4<f32>(position, 1.0)).xyz;
            skinned_normal += weight * (mat3x3<f32>(
                joint_mat[0].xyz,
                joint_mat[1].xyz,
                joint_mat[2].xyz
            ) * normal);
        }
    }

    // ... continue with model transform
}
```

---

## Animation State Mapping

### Engine Behavior to Animation

| Horse Behavior State  | Animation to Play | Playback Speed |
|-----------------------|-------------------|----------------|
| Idle (standing)       | Idle              | 1.0            |
| Idle (grazing)        | Idle_Headlow      | 1.0            |
| Walking (speed < 3)   | Walk              | speed / 2.0    |
| Trotting (speed 3-6)  | Walk              | 1.0 + speed/6  |
| Galloping (speed > 6) | Gallop            | speed / 10.0   |
| Taking damage         | Idle_HitReact1    | 1.0            |
| Jumping               | Gallop_Jump       | 1.0            |

### Transition Blend Times

| From          | To            | Blend (sec) |
|---------------|---------------|-------------|
| Idle          | Walk          | 0.3         |
| Walk          | Gallop        | 0.25        |
| Gallop        | Walk          | 0.25        |
| Walk          | Idle          | 0.4         |
| Any           | Idle_HitReact | 0.1         |

---

## Blender Export Checklist

### Geometry
- [ ] Triangle count: 10K-15K (current may be higher)
- [ ] Apply all modifiers before export
- [ ] Remove interior/hidden faces
- [ ] Check UV unwrapping

### Texture
- [ ] Export embedded texture OR use baseColorFactor
- [ ] Max 1024x1024 resolution
- [ ] Include AO baked into base color

### Skeleton
- [ ] Deform bones only (no IK controls)
- [ ] Clean bone naming (see IK requirements below)
- [ ] Rest pose: legs straight down
- [ ] Root bone at origin (0,0,0)

### Animations
- [ ] Bake all animations at 24 FPS
- [ ] Remove redundant keyframes (Blender Clean Keyframes)
- [ ] Seamless loops (frame 1 = last frame)
- [ ] Root bone stationary (engine applies movement)

### Export Settings
```
Format: GLTF Embedded (.gltf)
Include: Animations, Skinning
Geometry: Apply Modifiers, UVs, Normals
Animation: 24fps sampling, Optimize size
```

---

## IK Bone Requirements

### Required Leg Chain Structure

```
Root
└── Pelvis
    ├── Spine → Neck → Head
    ├── Front_L_Hip → Front_L_Upper → Front_L_Lower → Front_L_Foot
    ├── Front_R_Hip → Front_R_Upper → Front_R_Lower → Front_R_Foot
    ├── Back_L_Hip → Back_L_Upper → Back_L_Lower → Back_L_Foot
    └── Back_R_Hip → Back_R_Upper → Back_R_Lower → Back_R_Foot
```

### Bone Name Aliases

The engine recognizes these patterns (case-insensitive):

```
Front Left:  Front_L_*, FL_*, Shoulder.L, *Arm.L
Front Right: Front_R_*, FR_*, Shoulder.R, *Arm.R
Back Left:   Back_L_*, BL_*, Hip.L, *Leg.L
Back Right:  Back_R_*, BR_*, Hip.R, *Leg.R
```

---

## Action Items

### Immediate (For Idle Animation)
1. [ ] Implement GPU skinning in shader
2. [ ] Update vertex buffer to include joint indices/weights
3. [ ] Upload joint matrices per frame
4. [ ] Play "Idle" animation on idle horses

### Future (Full Animation System)
1. [ ] Implement animation blending system
2. [ ] Add animation state machine per horse
3. [ ] Support animation transitions with crossfade
4. [ ] Add "Turn" animation or procedural turning

---

## Agentic Update Protocol

When updating this spec:

1. **Increment version** following semver:
   - MAJOR: Breaking changes to model format
   - MINOR: New animations or features added
   - PATCH: Documentation fixes, clarifications

2. **Update status tables** when:
   - Engine pipeline changes
   - New animations added to model
   - Implementation status changes

3. **Verify against actual model** by running:
   ```bash
   # Check actual animations in Horse.gltf
   grep '"name"' assets/models/animals/Horse.gltf | grep -iE "idle|walk|trot|gallop|graze"
   ```

4. **Test with engine** to confirm:
   - Skeleton loads correctly (check console output)
   - Animations are recognized
   - Joint weights apply properly
