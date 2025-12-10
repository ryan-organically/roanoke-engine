# Horse Animation Guide for Blender Bridge

Context for Blender-Claude to create and optimize horse animations for the Roanoke Engine.

---

## Target Animations (4 only)

| Animation | Type    | Duration | Notes                                              |
|-----------|---------|----------|----------------------------------------------------|
| Graze     | looping | 3-4 sec  | Head down grazing with subtle movements            |
| Trot      | looping | 0.8 sec  | Medium gait, diagonal leg pairs move together      |
| Gallop    | looping | 0.5 sec  | Full speed, suspension phase (all legs off ground) |
| Turn      | looping | 1 sec    | Weight shift, can blend with Trot/Gallop           |

---

## Animation Details

### Graze - Idle/resting state
- Head lowered to ground
- Head reaches 45 degrees side-to-side occasionally while lowered (grazing different spots)
- Occasional tail tassel swish
- Occasional ear flick
- Loop seamlessly

### Trot - Medium movement speed
- Two-beat diagonal gait (front-left + back-right, then front-right + back-left)
- Some vertical bounce in the body
- Head steady with slight nod

### Gallop - Fast movement
- Four-beat gait with suspension phase
- Strong back leg push-off
- Neck extends forward
- Mane/tail should flow (if rigged)

### Turn - Directional blending
- Weight shifts to inside legs
- Head turns toward direction
- Can be left/right variants OR single animation to mirror

---

## Export Specifications

```
FORMAT:           GLTF Embedded (.gltf)
GEOMETRY:         10K-15K triangles
TEXTURE:          1024x1024, baked AO into base color
ANIMATION FPS:    24
BONES:            Deform bones only (remove IK/controls)
```

---

## Naming Convention

Name the actions in Blender exactly:

```
Horse_Graze
Horse_Trot
Horse_Gallop
Horse_Turn
```

---

## State Mapping (Engine Reference)

| Horse Behavior        | Animation |
|-----------------------|-----------|
| Standing idle         | Graze     |
| Walking (speed < 3)   | Trot (slowed playback) |
| Trotting (speed 3-6)  | Trot      |
| Galloping (speed > 6) | Gallop    |
| Changing direction    | Turn (blend) |

---

## Animation Transition System

### Approach: Crossfade Blending + Speed-Driven Locomotion

```
                    +-------------------------------------+
                    |         BEHAVIOR SYSTEM             |
                    |  (encephalon emotions/decisions)    |
                    +-----------------+-------------------+
                                      |
                                      v
                    +-------------------------------------+
                    |       TARGET STATE + SPEED          |
                    |   (Idle, Moving, Fleeing, etc.)     |
                    +-----------------+-------------------+
                                      |
                                      v
+------------------------------------------------------------------+
|                    ANIMATION BLENDER                             |
|                                                                  |
|   Current Anim -----> CROSSFADE (0.3s) -----> Target Anim        |
|                                                                  |
|   Locomotion: Speed interpolates between Trot <---> Gallop       |
|   Turn: Additive layer on top of locomotion                      |
+------------------------------------------------------------------+
```

### Transition Rules

| From   | To     | Blend Time | Trigger                        |
|--------|--------|------------|--------------------------------|
| Graze  | Trot   | 0.4s       | speed > 0.5 OR alert           |
| Trot   | Gallop | 0.3s       | speed > 6.0                    |
| Gallop | Trot   | 0.3s       | speed < 5.0                    |
| Trot   | Graze  | 0.5s       | speed < 0.5 AND calm           |
| Gallop | Graze  | 0.6s       | speed < 0.5 AND calm           |
| Any    | Turn   | additive   | angular_velocity > threshold   |

### Engine Implementation

```rust
// Animation blending state
pub struct AnimationBlend {
    current_anim: AnimationClip,
    target_anim: Option<AnimationClip>,
    blend_factor: f32,        // 0.0 = current, 1.0 = target
    blend_duration: f32,      // seconds to complete blend
    playback_speed: f32,      // scale animation speed with movement
}

// Each frame:
fn update_animation(horse: &mut Horse, dt: f32) {
    let speed = horse.velocity.length();

    // Determine target animation from behavior + speed
    let target = match (horse.behavior_state, speed) {
        (_, s) if s > 6.0       => Anim::Gallop,
        (_, s) if s > 0.5       => Anim::Trot,
        (Calm, _)               => Anim::Graze,
        (Alert, _)              => Anim::Trot,  // ready to move
        _ => Anim::Graze,
    };

    // Start blend if target changed
    if target != horse.anim.current_anim {
        horse.anim.target_anim = Some(target);
        horse.anim.blend_factor = 0.0;
        horse.anim.blend_duration = get_blend_time(current, target);
    }

    // Advance blend
    if let Some(target) = horse.anim.target_anim {
        horse.anim.blend_factor += dt / horse.anim.blend_duration;
        if horse.anim.blend_factor >= 1.0 {
            horse.anim.current_anim = target;
            horse.anim.target_anim = None;
        }
    }

    // Scale playback speed with movement
    horse.anim.playback_speed = match horse.anim.current_anim {
        Anim::Trot   => remap(speed, 0.5..6.0, 0.7..1.3),
        Anim::Gallop => remap(speed, 6.0..12.0, 0.8..1.2),
        _ => 1.0,
    };
}
```

---

## Transition Requirements for Blender

For smooth blending to work, animations must follow these rules:

### 1. Consistent Root Position
- All animations start with horse at origin
- Root bone stays at (0,0,0) - movement is applied by engine

### 2. Matching Start/End Poses
- Frame 1 and last frame should match for seamless loops
- Feet positions should align at loop points

### 3. Similar Poses at Transition Points
- Graze "head lifting" frame should approximate Trot "head neutral" frame
- Trot "full stride" frame should approximate Gallop "similar leg position"
- This reduces "pop" during blends

### 4. Turn as Additive
- Turn animation should be relative offsets, not absolute poses
- OR: Provide `Horse_Turn_Left` and `Horse_Turn_Right` as full-body variants

---

## Optional: Transition Animations (Future)

For higher fidelity later, specific transition animations can be added:

```
Horse_Graze_to_Trot     (horse lifts head, shifts weight, first step)
Horse_Trot_to_Gallop    (acceleration burst)
Horse_Gallop_to_Stop    (skidding halt)
```

Crossfade blending works well enough to start without these.

---

## File Size Optimization

### Texture Optimization
- Export textures at 1024x1024 max (engine caps at 4096, but 1K is plenty)
- Use PNG with compression, or JPEG for base color (smaller than embedded base64)
- Consider texture atlasing - single texture for entire horse body

### Geometry
- Target 10K-15K triangles (current models have ~50K+ which is excessive)
- Decimate modifier at 0.3-0.5 ratio preserves silhouette
- Remove interior faces, backfaces that will never be seen

### Animation Data
- Bake animations at 24fps, not 60fps (halves keyframe data)
- Remove redundant keyframes with Blender's "Clean Keyframes" operator
- Keep only essential bones (remove IK bones, control bones before export)

---

## GLTF Export Settings

```
FORMAT: GLTF Embedded (.gltf) - NOT GLB

INCLUDE:
[x] Selected Objects
[x] Animations
[x] Skinning (armature/bones)

GEOMETRY:
[x] Apply Modifiers
[x] UVs
[x] Normals
[ ] Tangents (not used)
[ ] Vertex Colors (not used)

ANIMATION:
[ ] Use Current Frame as Rest Pose (OFF)
[x] Export NLA Strips as separate animations
[x] Optimize Animation Size
[x] Sampling Rate: 24fps

MATERIALS:
[x] Export Materials
[x] Images: Automatic or JPEG (smaller)
```

---

## IK Bone Requirements (Critical)

The engine uses runtime Inverse Kinematics to adapt legs to terrain. This requires a specific bone structure.

### Required Bone Hierarchy

```
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

### Bone Naming

Use these exact names OR common aliases the engine recognizes:

| Required Name | Acceptable Aliases |
|---------------|-------------------|
| Front_L_Hip | FrontLeft_Hip, FL_Hip, Shoulder.L |
| Front_L_Upper | FrontLeft_Upper, FL_UpperLeg, UpperArm.L |
| Front_L_Lower | FrontLeft_Lower, FL_LowerLeg, LowerArm.L |
| Front_L_Foot | FrontLeft_Foot, FL_Foot, Hand.L |
| Front_R_Hip | FrontLeft_Hip, FR_Hip, Shoulder.R |
| Front_R_Upper | FrontRight_Upper, FR_UpperLeg, UpperArm.R |
| Front_R_Lower | FrontRight_Lower, FR_LowerLeg, LowerArm.R |
| Front_R_Foot | FrontRight_Foot, FR_Foot, Hand.R |
| Back_L_Hip | BackLeft_Hip, BL_Hip, Hip.L |
| Back_L_Upper | BackLeft_Upper, BL_UpperLeg, UpperLeg.L |
| Back_L_Lower | BackLeft_Lower, BL_LowerLeg, LowerLeg.L |
| Back_L_Foot | BackLeft_Foot, BL_Foot, Foot.L |
| Back_R_Hip | BackRight_Hip, BR_Hip, Hip.R |
| Back_R_Upper | BackRight_Upper, BR_UpperLeg, UpperLeg.R |
| Back_R_Lower | BackRight_Lower, BR_LowerLeg, LowerLeg.R |
| Back_R_Foot | BackRight_Foot, BR_Foot, Foot.R |

### IK Constraints

1. **Accurate bone lengths** - Engine uses actual bone lengths for IK solving
2. **Straight rest pose** - Legs should be straight down in rest pose (like T-pose for bipeds)
3. **Proper parenting** - Each leg must be a clean chain: Hip → Upper → Lower → Foot
4. **No constraints** - Remove all Blender IK constraints before export (engine handles IK at runtime)

### Why This Matters

The engine casts rays from each foot position to find ground height, then uses two-bone IK to bend each leg so the foot reaches the ground. Without proper bone hierarchy, legs won't adapt to terrain.

```
ON FLAT GROUND:              ON SLOPE:
      o (hip)                     o (hip)
      |                           |\
      |                           | \
      |                           |  \
      o (knee)                    o   | (knee bends)
      |                          /    |
      |                         /     |
      * (foot on ground)       *------* (feet at different heights)
```

---

## Summary

| Requirement | Value |
|-------------|-------|
| Triangle count | 10K-15K |
| Texture size | 1024x1024 |
| Animation FPS | 24 |
| Format | GLTF Embedded (.gltf) |
| Animations | Horse_Graze, Horse_Trot, Horse_Gallop, Horse_Turn |
| Bones | Deform only (no IK/controls) |
| Root | At origin, stationary |
| Loops | Seamless (frame 1 = last frame) |
| Leg bones | 4 chains: Hip → Upper → Lower → Foot |
| Rest pose | Legs straight down |
