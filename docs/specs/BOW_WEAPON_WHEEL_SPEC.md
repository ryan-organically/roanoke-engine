# BOW, ARROW & WEAPON WHEEL SPECIFICATION

**Version**: 1.0
**Date**: 2024-12-05
**Status**: DESIGN PHASE

---

## TABLE OF CONTENTS

1. [Weapon Wheel System](#1-weapon-wheel-system)
2. [Bow Mechanics](#2-bow-mechanics)
3. [Arrow Physics & Types](#3-arrow-physics--types)
4. [Projectile Signatures (Space Bevel Effect)](#4-projectile-signatures-space-bevel-effect)
5. [Firearms Integration](#5-firearms-integration)
6. [Implementation Roadmap](#6-implementation-roadmap)

---

## 1. WEAPON WHEEL SYSTEM

### 1.1 Overview

Radial menu activated by holding a key, allowing quick weapon selection without pausing gameplay.

### 1.2 Controls

| Action | Input |
|--------|-------|
| Open wheel | Hold `TAB` or `Q` |
| Select weapon | Mouse direction while holding |
| Confirm selection | Release key |
| Cancel | Press `ESC` or move to center |

### 1.3 Wheel Layout

```
                    [RIFLE]
                       |
         [BOW]    _____|_____    [PISTOL]
              \  /           \  /
               \/    CENTER   \/
               /\   (FISTS)   /\
              /  \___________/  \
        [AXE]         |         [KNIFE]
                      |
                   [SPEAR]
```

**8 Slots Total:**
- Slot 1 (N): Rifle/Musket
- Slot 2 (NE): Pistol
- Slot 3 (E): Knife
- Slot 4 (SE): Spear
- Slot 5 (S): Reserved
- Slot 6 (SW): Axe/Tomahawk
- Slot 7 (W): Bow
- Slot 8 (NW): Reserved
- Center: Unarmed/Fists

### 1.4 Visual Design

```
┌─────────────────────────────────────┐
│                                     │
│            ┌─────────┐              │
│      ┌─────┤  RIFLE  ├─────┐        │
│      │     └────┬────┘     │        │
│   ┌──┴──┐       │       ┌──┴──┐     │
│   │ BOW │       │       │PISTOL│    │
│   └──┬──┘   ┌───┴───┐   └──┬──┘     │
│      │     │ FISTS │      │        │
│      │     │(center)│      │        │
│   ┌──┴──┐   └───┬───┘   ┌──┴──┐     │
│   │ AXE │       │       │KNIFE│     │
│   └──┬──┘       │       └──┬──┘     │
│      │     ┌────┴────┐     │        │
│      └─────┤  SPEAR  ├─────┘        │
│            └─────────┘              │
│                                     │
│  [Currently Equipped: BOW]          │
│  [Arrows: 24/30]                    │
└─────────────────────────────────────┘
```

### 1.5 Wheel Behavior

- **Time Slowdown**: Game time slows to 20% while wheel is open
- **Highlight**: Selected segment glows/pulses
- **Preview**: Selected weapon model shown in center
- **Ammo Display**: Current ammo count for ranged weapons
- **Fade In/Out**: 150ms transition

### 1.6 Data Structure

```rust
pub struct WeaponWheel {
    pub slots: [Option<WeaponSlot>; 8],
    pub selected_index: Option<usize>,
    pub is_open: bool,
    pub open_time: f32,
}

pub struct WeaponSlot {
    pub weapon_type: WeaponType,
    pub display_name: String,
    pub icon_texture: String,
    pub ammo_current: u32,
    pub ammo_max: u32,
    pub is_unlocked: bool,
}

pub enum WeaponType {
    Unarmed,
    Bow,
    Rifle,
    Pistol,
    Knife,
    Axe,
    Spear,
    Tomahawk,
}
```

---

## 2. BOW MECHANICS

### 2.1 States

```
┌─────────┐    Hold RMB    ┌─────────┐    Hold    ┌─────────┐
│  IDLE   │ ────────────► │ DRAWING │ ────────► │  DRAWN  │
└─────────┘                └─────────┘            └─────────┘
     ▲                          │                      │
     │                          │ Release early        │ Release LMB
     │                          ▼                      ▼
     │                    ┌─────────┐            ┌─────────┐
     └────────────────────│ CANCEL  │            │  FIRE   │
                          └─────────┘            └─────────┘
```

### 2.2 Draw Mechanics

| Phase | Duration | Draw Power | Accuracy |
|-------|----------|------------|----------|
| Start Draw | 0.0 - 0.3s | 0% - 30% | Very Low |
| Mid Draw | 0.3 - 0.8s | 30% - 80% | Low |
| Full Draw | 0.8 - 1.2s | 80% - 100% | High |
| Overdraw | 1.2s+ | 100% (arms shake) | Decreasing |

### 2.3 Controls

| Action | Input | Notes |
|--------|-------|-------|
| Draw bow | Hold RMB | Begins draw animation |
| Aim | Mouse movement | Free aim while drawing |
| Release arrow | Release RMB | Fires at current draw power |
| Cancel draw | Press R or switch weapon | Returns arrow to quiver |
| Zoom aim | Hold SHIFT while drawn | Slight zoom, steadier aim |

### 2.4 Draw Power Effects

```rust
pub struct BowState {
    pub draw_time: f32,          // Time held
    pub draw_power: f32,         // 0.0 - 1.0
    pub arm_fatigue: f32,        // Increases over time at full draw
    pub is_drawing: bool,
    pub arrows_remaining: u32,
}

impl BowState {
    pub fn calculate_draw_power(&self) -> f32 {
        let base_power = (self.draw_time / FULL_DRAW_TIME).min(1.0);

        // Smooth easing curve
        let eased = 1.0 - (1.0 - base_power).powi(3);

        eased
    }

    pub fn calculate_accuracy(&self) -> f32 {
        let base_accuracy = self.draw_power;

        // Fatigue reduces accuracy after full draw
        let fatigue_penalty = (self.arm_fatigue * 0.3).min(0.5);

        (base_accuracy - fatigue_penalty).max(0.2)
    }
}
```

### 2.5 Visual Feedback

**Draw Stages:**
1. **0-30%**: Bow barely bent, string slack
2. **30-60%**: Bow bending, string taut
3. **60-90%**: Full bend, arrow pulled back
4. **90-100%**: Maximum draw, slight glow on arrow
5. **Overdraw**: Arms start shaking, aim wobbles

**UI Indicators:**
- Draw power arc around crosshair
- Arrow count in corner
- Fatigue warning at overdraw

### 2.6 Animation States

```rust
pub enum BowAnimation {
    Idle,
    DrawStart,
    Drawing { progress: f32 },
    FullDraw { shake_intensity: f32 },
    Release,
    Reload,  // Nocking new arrow
}
```

---

## 3. ARROW PHYSICS & TYPES

### 3.1 Arrow Flight Model

```rust
pub struct Arrow {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quat,        // Arrow orientation
    pub spin: f32,             // Rotation around shaft axis
    pub draw_power: f32,       // Power at release (affects speed)
    pub arrow_type: ArrowType,
    pub time_alive: f32,
    pub has_hit: bool,
}

// Physics constants
const GRAVITY: f32 = 9.81;
const AIR_RESISTANCE: f32 = 0.02;
const ARROW_MASS: f32 = 0.025;  // 25 grams

impl Arrow {
    pub fn update(&mut self, dt: f32) {
        // Gravity
        self.velocity.y -= GRAVITY * dt;

        // Air resistance (drag)
        let speed = self.velocity.length();
        let drag = self.velocity.normalize() * speed * speed * AIR_RESISTANCE;
        self.velocity -= drag * dt;

        // Update position
        self.position += self.velocity * dt;

        // Arrow rotates to face velocity direction (weathervaning)
        if speed > 1.0 {
            let target_dir = self.velocity.normalize();
            let current_dir = self.rotation * Vec3::Z;
            let rotation_speed = 5.0 * dt;
            self.rotation = Quat::slerp(
                self.rotation,
                Quat::from_rotation_arc(Vec3::Z, target_dir),
                rotation_speed
            );
        }

        // Spin for stabilization
        self.spin += 720.0 * dt;  // Degrees per second

        self.time_alive += dt;
    }
}
```

### 3.2 Arrow Types

| Type | Damage | Speed | Special |
|------|--------|-------|---------|
| Standard | 50 | 60 m/s | None |
| Broadhead | 75 | 55 m/s | Bleed damage |
| Bodkin | 60 | 70 m/s | Armor piercing |
| Fire | 40 | 55 m/s | Ignites target |
| Hunting | 65 | 60 m/s | +50% vs animals |
| Blunt | 20 | 50 m/s | Stun, non-lethal |

```rust
pub enum ArrowType {
    Standard,
    Broadhead { bleed_damage: f32 },
    Bodkin { armor_pierce: f32 },
    Fire { burn_duration: f32 },
    Hunting { animal_bonus: f32 },
    Blunt { stun_duration: f32 },
}
```

### 3.3 Arrow Speed Calculation

```rust
fn calculate_arrow_speed(draw_power: f32, arrow_type: &ArrowType) -> f32 {
    let base_speed = match arrow_type {
        ArrowType::Standard => 60.0,
        ArrowType::Broadhead { .. } => 55.0,
        ArrowType::Bodkin { .. } => 70.0,
        ArrowType::Fire { .. } => 55.0,
        ArrowType::Hunting { .. } => 60.0,
        ArrowType::Blunt { .. } => 50.0,
    };

    // Minimum 40% speed even at low draw
    let power_multiplier = 0.4 + (draw_power * 0.6);

    base_speed * power_multiplier
}
```

### 3.4 Arrow Drop Table

At full draw (60 m/s initial velocity):

| Distance | Drop | Flight Time |
|----------|------|-------------|
| 10m | 0.14m | 0.17s |
| 25m | 0.85m | 0.42s |
| 50m | 3.4m | 0.83s |
| 75m | 7.6m | 1.25s |
| 100m | 13.6m | 1.67s |

---

## 4. PROJECTILE SIGNATURES (SPACE BEVEL EFFECT)

### 4.1 Concept

Projectiles (arrows and bullets) leave a **space-distortion trail** behind them - a visual effect that **magnifies/bevels** the world geometry visible through the trail, like looking through a cylindrical lens or gravitational lensing.

```
    Normal View          With Projectile Trail
    ============         =====================

    |  |  |  |           |  |  |  |
    |  |  |  |           |  |╱  ╲|  |
    |  |  |  |    ──►    | ╱ ══ ╲ |   ← Magnified/distorted
    |  |  |  |           |╲ ══ ╱|      region behind projectile
    |  |  |  |           | ╲__╱ |
    |  |  |  |           |  |  |  |
```

### 4.2 Visual Effect Description

**The "Space Bevel" Trail:**
1. **Shape**: Tapered cone/cylinder behind projectile
2. **Effect**: Refracts/magnifies scene geometry within the trail
3. **Falloff**: Strongest near projectile, fades over trail length
4. **Color Tint**: Subtle chromatic aberration at edges

**For Arrows:**
- Trail length: 2-3 meters
- Trail width: 10-20 cm
- Magnification: 1.1x - 1.3x
- Duration: Fades over 0.3 seconds after passing

**For Bullets:**
- Trail length: 1-2 meters
- Trail width: 3-5 cm
- Magnification: 1.2x - 1.5x (more intense, faster)
- Duration: Fades over 0.1 seconds

### 4.3 Shader Implementation

```wgsl
// projectile_trail.wgsl - Space Bevel Effect

struct TrailUniforms {
    projectile_pos: vec3<f32>,
    projectile_dir: vec3<f32>,      // Normalized velocity
    trail_length: f32,
    trail_radius: f32,
    magnification: f32,
    time: f32,
    view_pos: vec3<f32>,
    _padding: f32,
}

@group(0) @binding(0) var<uniform> trail: TrailUniforms;
@group(0) @binding(1) var scene_texture: texture_2d<f32>;
@group(0) @binding(2) var scene_sampler: sampler;

// Calculate distance from point to line segment (projectile trail)
fn distance_to_trail(world_pos: vec3<f32>) -> vec2<f32> {
    let trail_start = trail.projectile_pos;
    let trail_end = trail.projectile_pos - trail.projectile_dir * trail.trail_length;

    let line_vec = trail_end - trail_start;
    let point_vec = world_pos - trail_start;

    let t = clamp(dot(point_vec, line_vec) / dot(line_vec, line_vec), 0.0, 1.0);
    let closest = trail_start + line_vec * t;

    let dist = distance(world_pos, closest);
    return vec2<f32>(dist, t);  // distance and position along trail
}

// Main distortion calculation
fn calculate_distortion(uv: vec2<f32>, world_pos: vec3<f32>) -> vec2<f32> {
    let trail_info = distance_to_trail(world_pos);
    let dist = trail_info.x;
    let along_trail = trail_info.y;

    // Tapered radius - wider at projectile, narrower at tail
    let tapered_radius = trail.trail_radius * (1.0 - along_trail * 0.7);

    if (dist > tapered_radius) {
        return uv;  // Outside trail, no distortion
    }

    // Calculate distortion strength
    let normalized_dist = dist / tapered_radius;
    let strength = (1.0 - normalized_dist * normalized_dist);  // Quadratic falloff
    let fade = 1.0 - along_trail;  // Fade toward tail

    // Direction from trail center to this point (for radial distortion)
    let to_center = normalize(world_pos - (trail.projectile_pos - trail.projectile_dir * along_trail * trail.trail_length));

    // Magnification effect - push UVs outward from center
    let magnify_amount = strength * fade * (trail.magnification - 1.0);
    let distorted_uv = uv + to_center.xy * magnify_amount * 0.1;

    return distorted_uv;
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>, @location(1) world_pos: vec3<f32>) -> @location(0) vec4<f32> {
    let distorted_uv = calculate_distortion(uv, world_pos);

    // Sample with slight chromatic aberration at edges
    let aberration = 0.002;
    let r = textureSample(scene_texture, scene_sampler, distorted_uv + vec2<f32>(aberration, 0.0)).r;
    let g = textureSample(scene_texture, scene_sampler, distorted_uv).g;
    let b = textureSample(scene_texture, scene_sampler, distorted_uv - vec2<f32>(aberration, 0.0)).b;

    return vec4<f32>(r, g, b, 1.0);
}
```

### 4.4 Trail Rendering Pipeline

**Two-Pass Approach:**

1. **Pass 1**: Render scene to offscreen texture (already done for light shafts)
2. **Pass 2**: Render trails with distortion shader, sampling from Pass 1

```rust
pub struct ProjectileTrailPipeline {
    pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    bind_group_layout: BindGroupLayout,
}

pub struct TrailInstance {
    pub position: Vec3,
    pub direction: Vec3,
    pub trail_length: f32,
    pub trail_radius: f32,
    pub magnification: f32,
    pub age: f32,  // For fade-out
}

impl ProjectileTrailPipeline {
    pub fn render_trails(
        &self,
        encoder: &mut CommandEncoder,
        scene_texture: &TextureView,
        output: &TextureView,
        trails: &[TrailInstance],
    ) {
        // Update uniforms for each trail and render
        for trail in trails {
            if trail.age < trail.max_age() {
                self.render_single_trail(encoder, scene_texture, output, trail);
            }
        }
    }
}
```

### 4.5 Trail Manager

```rust
pub struct ProjectileTrailManager {
    active_trails: Vec<TrailInstance>,
    max_trails: usize,
}

impl ProjectileTrailManager {
    pub fn spawn_arrow_trail(&mut self, arrow: &Arrow) {
        self.active_trails.push(TrailInstance {
            position: arrow.position,
            direction: arrow.velocity.normalize(),
            trail_length: 2.5,
            trail_radius: 0.15,
            magnification: 1.2,
            age: 0.0,
        });
    }

    pub fn spawn_bullet_trail(&mut self, bullet: &Bullet) {
        self.active_trails.push(TrailInstance {
            position: bullet.position,
            direction: bullet.velocity.normalize(),
            trail_length: 1.5,
            trail_radius: 0.05,
            magnification: 1.4,
            age: 0.0,
        });
    }

    pub fn update(&mut self, dt: f32) {
        // Update trail ages and remove expired
        self.active_trails.retain_mut(|trail| {
            trail.age += dt;
            trail.age < 0.5  // Keep for 0.5 seconds
        });
    }
}
```

### 4.6 Visual Variations

**Arrow Signature:**
- Wider, slower-moving distortion
- Slight "whoosh" blur in direction of travel
- Optional: Feather particle effects at tail

**Bullet Signature:**
- Thin, intense distortion
- Heat shimmer effect
- Optional: Brief smoke wisp

**Special Arrows:**
- Fire Arrow: Orange/red chromatic shift, ember particles
- Ice Arrow: Blue tint, crystalline refraction pattern
- Lightning Arrow: Electric crackling distortion

---

## 5. FIREARMS INTEGRATION

### 5.1 Weapon Types

| Weapon | Fire Rate | Reload | Range | Ammo |
|--------|-----------|--------|-------|------|
| Flintlock Pistol | Single | 3.0s | Short | Lead ball |
| Musket | Single | 4.5s | Long | Lead ball |
| Blunderbuss | Single | 3.5s | Short | Shot spread |

### 5.2 Bullet Physics

```rust
pub struct Bullet {
    pub position: Vec3,
    pub velocity: Vec3,
    pub caliber: f32,
    pub damage: f32,
    pub time_alive: f32,
}

// Bullets are much faster, less drop
const BULLET_SPEED_PISTOL: f32 = 250.0;   // m/s
const BULLET_SPEED_MUSKET: f32 = 400.0;   // m/s
const BULLET_DRAG: f32 = 0.001;           // Less drag than arrows

impl Bullet {
    pub fn update(&mut self, dt: f32) {
        self.velocity.y -= GRAVITY * dt;

        let speed = self.velocity.length();
        let drag = self.velocity.normalize() * speed * speed * BULLET_DRAG;
        self.velocity -= drag * dt;

        self.position += self.velocity * dt;
        self.time_alive += dt;
    }
}
```

### 5.3 Reload Animation States

```rust
pub enum FirearmState {
    Ready,
    Firing { recoil_time: f32 },
    ReloadStart,
    PourPowder { progress: f32 },
    RamBall { progress: f32 },
    Prime { progress: f32 },
    ReloadEnd,
}
```

---

## 6. IMPLEMENTATION ROADMAP

### Phase 1: Core Weapon System
- [ ] Weapon slot data structure
- [ ] Weapon switching logic
- [ ] Basic equip/unequip animations
- [ ] Ammo tracking

### Phase 2: Weapon Wheel UI
- [ ] Radial menu rendering
- [ ] Mouse direction detection
- [ ] Time slowdown while open
- [ ] Slot highlighting
- [ ] Weapon preview in center

### Phase 3: Bow Mechanics
- [ ] Draw state machine
- [ ] Draw power calculation
- [ ] Aim sway/stability
- [ ] Release animation
- [ ] Arrow nocking animation

### Phase 4: Arrow Physics
- [ ] Projectile entity system
- [ ] Gravity and drag simulation
- [ ] Arrow rotation (weathervaning)
- [ ] Collision detection
- [ ] Stuck arrow persistence

### Phase 5: Space Bevel Effect
- [ ] Offscreen render target setup
- [ ] Trail distortion shader
- [ ] Trail instance management
- [ ] Chromatic aberration
- [ ] Per-projectile-type variations

### Phase 6: Firearms
- [ ] Reload state machine
- [ ] Bullet physics
- [ ] Muzzle flash/smoke
- [ ] Reload animations
- [ ] Period-accurate sounds

### Phase 7: Polish
- [ ] Sound effects (draw, release, impact)
- [ ] Hit feedback (screen shake, slow-mo)
- [ ] Arrow/bullet trails particle effects
- [ ] Aim assist (optional)
- [ ] Controller support

---

## APPENDIX: REFERENCE VALUES

### Historical Bow Specs (English Longbow)
- Draw weight: 80-180 lbs
- Arrow speed: 50-70 m/s
- Effective range: 200-300m
- Rate of fire: 10-12 arrows/minute (skilled)

### Game Balance Targets
- Full draw time: 1.2 seconds
- Arrows per minute: 8-10
- Headshot multiplier: 2.5x
- Body shot damage: 50-75 HP
- Max effective range: 75m

---

*Specification authored for Roanoke Engine*
*Ready for implementation upon approval*
