# Karma Light Beams System Specification

Divine light beams that reveal the world differently based on the player's moral standing. The cosmos itself guides or judges the player through targeted illumination.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Core Concept](#2-core-concept)
3. [Light Beam Types](#3-light-beam-types)
4. [Karma Thresholds](#4-karma-thresholds)
5. [Revelation Targets](#5-revelation-targets)
6. [Visual Design](#6-visual-design)
7. [Weather Integration](#7-weather-integration)
8. [Data Structures](#8-data-structures)
9. [Beam Behavior](#9-beam-behavior)
10. [Audio Integration](#10-audio-integration)
11. [Implementation Phases](#11-implementation-phases)

---

## 1. Overview

### Design Philosophy

The world watches. The sun is not merely a light source—it is a witness. For those who walk in harmony with nature, its rays guide them to sustenance and sanctuary. For those who take without giving, the shadows themselves become allies, revealing prey and hidden places where darker deeds go unseen.

This is not a reward/punishment system. Both paths receive guidance. The difference is *what* is revealed.

### Key Principles

- **Subtle, Not Obvious**: Beams should feel like natural lighting phenomena, not glowing quest markers
- **Contextual**: Beams only appear when relevant (hungry near fruit, lost near springs)
- **Earned**: Higher karma magnitude = more frequent/reliable revelations
- **Atmospheric**: Integrates with weather, time of day, and environment

---

## 2. Core Concept

### The Duality

| Karma Alignment | Light Source | Beam Color | Reveals |
|-----------------|--------------|------------|---------|
| **Positive (Guardian)** | Sun breaks through clouds | Warm gold, soft white | Life, sustenance, sanctuary |
| **Negative (Destroyer)** | Gaps in canopy, moon shafts | Cool silver, pale blue | Prey, hidden places, escape routes |
| **Neutral (Wanderer)** | Occasional flickers | Neutral white | Random mix, unreliable |

### The Witness Effect

The player should never feel *told* what to do. Instead, they notice:
- "The sun seems to catch that berry bush just right..."
- "Moonlight pools at the cave entrance..."
- "A shaft of light falls on the deer's flank..."

---

## 3. Light Beam Types

### 3.1 Blessing Beams (Positive Karma)

```rust
pub enum BlessingBeamType {
    /// Highlights fruit-bearing trees and edible plants
    SunsGift,

    /// Reveals fresh water sources (springs, clean streams)
    LifeWaters,

    /// Illuminates safe shelter locations
    Sanctuary,

    /// Shows paths that avoid danger
    GuidedPath,

    /// Highlights friendly/tameable animals
    KindredSpirit,

    /// Reveals medicinal herbs when injured
    HealersLight,
}
```

**Characteristics:**
- Warm color temperature (2700K-4000K gold tones)
- Soft edges, gentle falloff
- Often accompanied by dust motes, pollen particles
- Subtle lens flare at source
- Sound: Faint wind chimes, birdsong swells

### 3.2 Shadow Beams (Negative Karma)

```rust
pub enum ShadowBeamType {
    /// Highlights huntable prey animals
    HuntersGaze,

    /// Reveals hidden cave entrances, crevices
    DarkHollow,

    /// Shows escape routes when pursued
    FlightPath,

    /// Illuminates valuable/stealable items
    ThiefsGleam,

    /// Reveals ambush positions
    PredatorsPerch,

    /// Shows weak points in structures/creatures
    CruelInsight,
}
```

**Characteristics:**
- Cool color temperature (6500K-8000K silver/blue)
- Sharp edges, defined shaft
- Dust motes appear as floating ash/embers
- No lens flare (light avoids the eye)
- Sound: Low drone, insect buzz, distant thunder

### 3.3 Wanderer Flickers (Neutral Karma)

```rust
pub enum WandererFlickerType {
    /// Brief, unreliable hints
    FadingGlimpse,

    /// Points to random interactables
    ChaoticGuidance,

    /// Sometimes helpful, sometimes misleading
    TricksterLight,
}
```

**Characteristics:**
- Inconsistent color temperature
- Flickers in and out
- Short duration (1-3 seconds)
- May point to nothing useful
- Sound: Static, whispers

---

## 4. Karma Thresholds

Integration with existing NatureBalance system from `NATURE_MORALITY_WEATHER_KARMA_SPEC.md`:

```rust
pub struct KarmaLightConfig {
    /// Minimum absolute karma to trigger any beams
    pub activation_threshold: f32,        // Default: 15.0

    /// Karma level for reliable, frequent beams
    pub reliable_threshold: f32,          // Default: 40.0

    /// Karma level for enhanced beams (brighter, longer)
    pub enhanced_threshold: f32,          // Default: 65.0

    /// Karma level for legendary beams (special revelations)
    pub legendary_threshold: f32,         // Default: 85.0
}

pub fn get_beam_tier(karma: f32) -> BeamTier {
    let magnitude = karma.abs();

    if magnitude < 15.0 {
        BeamTier::None
    } else if magnitude < 40.0 {
        BeamTier::Faint      // Occasional, brief
    } else if magnitude < 65.0 {
        BeamTier::Clear      // Regular, reliable
    } else if magnitude < 85.0 {
        BeamTier::Strong     // Frequent, bright
    } else {
        BeamTier::Legendary  // Special revelations
    }
}
```

### Beam Frequency by Tier

| Tier | Check Interval | Trigger Chance | Duration | Intensity |
|------|----------------|----------------|----------|-----------|
| None | - | 0% | - | - |
| Faint | 60s | 20% | 3-5s | 0.3 |
| Clear | 30s | 45% | 8-12s | 0.6 |
| Strong | 15s | 70% | 15-25s | 0.85 |
| Legendary | 10s | 90% | 30-60s | 1.0 |

---

## 5. Revelation Targets

### 5.1 Positive Karma Targets

```rust
pub struct BlessingTargets {
    /// Trees/bushes with ripe fruit
    pub fruit_sources: Vec<Entity>,

    /// Clean water springs and streams
    pub water_sources: Vec<Entity>,

    /// Safe cave entrances, overhangs
    pub shelter_locations: Vec<Entity>,

    /// Non-hostile, tameable creatures
    pub friendly_fauna: Vec<Entity>,

    /// Medicinal plants (context: player injured)
    pub healing_herbs: Vec<Entity>,

    /// Sacred groves, spirit stones
    pub sacred_sites: Vec<Entity>,
}
```

**Context Triggers:**
- `fruit_sources`: Player hunger > 30%
- `water_sources`: Player thirst > 40% OR player lost
- `shelter_locations`: Storm approaching OR nightfall within 30 min
- `friendly_fauna`: Player has food to offer
- `healing_herbs`: Player health < 70%
- `sacred_sites`: Near karma milestone

### 5.2 Negative Karma Targets

```rust
pub struct ShadowTargets {
    /// Huntable animals in range
    pub prey_animals: Vec<Entity>,

    /// Hidden cave entrances
    pub hidden_caves: Vec<Entity>,

    /// Climbable escape routes
    pub escape_routes: Vec<Entity>,

    /// Valuable/lootable items
    pub valuables: Vec<Entity>,

    /// Concealment positions
    pub ambush_spots: Vec<Entity>,

    /// Structural weak points
    pub vulnerabilities: Vec<Entity>,
}
```

**Context Triggers:**
- `prey_animals`: Player hunger > 20% OR hunting stance
- `hidden_caves`: Player exploring new area
- `escape_routes`: Player in combat OR being pursued
- `valuables`: Player near settlement/camp
- `ambush_spots`: Player stalking target
- `vulnerabilities`: Player in combat with tough enemy

---

## 6. Visual Design

### 6.1 Beam Geometry

```rust
pub struct LightBeamVisual {
    /// Origin point (sky position)
    pub origin: Vec3,

    /// Target point (world object)
    pub target: Vec3,

    /// Beam width at origin
    pub width_top: f32,           // Default: 2.0m

    /// Beam width at target
    pub width_bottom: f32,        // Default: 0.5m

    /// Overall intensity (0.0-1.0)
    pub intensity: f32,

    /// Color temperature in Kelvin
    pub color_temp: f32,

    /// Particle density
    pub particle_density: f32,

    /// Edge softness (0=sharp, 1=very soft)
    pub edge_softness: f32,
}
```

### 6.2 Shader Parameters

Extends existing `light_shafts.wgsl`:

```wgsl
struct KarmaBeamUniforms {
    // Base light shaft params
    beam_origin: vec3<f32>,
    beam_target: vec3<f32>,
    intensity: f32,

    // Karma-specific
    color_temp: f32,           // Kelvin
    edge_softness: f32,
    pulse_speed: f32,          // For breathing effect
    particle_influence: f32,   // How much particles affect brightness

    // Atmospheric
    fog_interaction: f32,      // How fog scatters the beam
    dust_density: f32,
    dust_color: vec3<f32>,
}
```

### 6.3 Color Palettes

**Blessing Beams (Positive):**
```rust
pub const BLESSING_COLORS: [(f32, f32, f32); 4] = [
    (1.0, 0.95, 0.8),    // Warm white
    (1.0, 0.85, 0.6),    // Golden
    (1.0, 0.9, 0.7),     // Soft amber
    (0.95, 0.95, 0.85),  // Pure daylight
];
```

**Shadow Beams (Negative):**
```rust
pub const SHADOW_COLORS: [(f32, f32, f32); 4] = [
    (0.8, 0.85, 1.0),    // Cool white
    (0.7, 0.8, 0.95),    // Pale blue
    (0.85, 0.85, 0.9),   // Silver
    (0.6, 0.7, 0.85),    // Deep twilight
];
```

---

## 7. Weather Integration

### 7.1 Weather States and Beam Behavior

| Weather | Positive Karma | Negative Karma |
|---------|----------------|----------------|
| **Clear** | Strong sun beams, full intensity | Weaker (too bright), wait for shade |
| **Partly Cloudy** | Sun breaks through clouds dramatically | Beam from cloud gaps |
| **Overcast** | Rare, powerful breaks | Optimal conditions, soft diffuse beams |
| **Stormy** | Lightning illuminates briefly | Storm reveals hidden paths |
| **Foggy** | Beams scatter beautifully | Beams pierce fog, reveal through haze |
| **Night Clear** | Moonbeams (reduced) | Full moon beams at max power |
| **Night Overcast** | Minimal | Moon gaps reveal prey |

### 7.2 Karma Weather Influence

From existing karma spec, weather responds to karma:

```rust
pub fn calculate_beam_availability(
    karma: f32,
    weather: &WeatherState,
    time_of_day: f32,
) -> BeamAvailability {
    let is_positive = karma > 0.0;
    let magnitude = karma.abs();

    // Positive karma = more sun breaks
    // Negative karma = more cloud cover (but this HELPS shadow beams)

    let base_chance = match weather.weather_type {
        WeatherType::Clear => {
            if is_positive { 0.8 } else { 0.3 }
        }
        WeatherType::Overcast => {
            if is_positive { 0.4 } else { 0.9 }
        }
        WeatherType::Stormy => {
            if is_positive { 0.2 } else { 0.6 }
        }
        // ... etc
    };

    BeamAvailability {
        chance: base_chance * (magnitude / 100.0),
        intensity_modifier: weather.get_light_transmission(),
    }
}
```

### 7.3 The Overcast Advantage (Negative Karma)

When karma is significantly negative, overcast weather becomes an *ally*:
- Clouds gather more frequently
- But gaps appear precisely where shadow beams need them
- Creates a "the darkness watches over you" feeling
- Prey animals are illuminated by silver light through cloud breaks

---

## 8. Data Structures

### 8.1 Core System

```rust
pub struct KarmaLightBeamSystem {
    /// Current active beams
    pub active_beams: Vec<ActiveBeam>,

    /// Maximum concurrent beams
    pub max_beams: usize,           // Default: 3

    /// Time since last beam check
    pub check_timer: f32,

    /// Configuration
    pub config: KarmaLightConfig,

    /// Cached potential targets
    pub target_cache: TargetCache,

    /// Audio handles for active beams
    pub audio_handles: Vec<AudioHandle>,
}

pub struct ActiveBeam {
    pub beam_type: BeamType,
    pub target_entity: Entity,
    pub target_position: Vec3,
    pub origin_position: Vec3,
    pub intensity: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub fade_state: FadeState,
    pub visual: LightBeamVisual,
}

pub enum FadeState {
    FadingIn { progress: f32, duration: f32 },
    Sustained,
    FadingOut { progress: f32, duration: f32 },
}

pub enum BeamType {
    Blessing(BlessingBeamType),
    Shadow(ShadowBeamType),
    Wanderer(WandererFlickerType),
}
```

### 8.2 Target Cache

```rust
pub struct TargetCache {
    /// Last update time
    pub last_update: f32,

    /// Update interval
    pub update_interval: f32,      // Default: 5.0s

    /// Search radius from player
    pub search_radius: f32,        // Default: 100.0m

    /// Categorized targets
    pub blessing_targets: BlessingTargets,
    pub shadow_targets: ShadowTargets,
}

impl TargetCache {
    pub fn update(&mut self, player_pos: Vec3, world: &World) {
        // Query spatial index for nearby entities
        // Categorize by type and relevance
        // Sort by priority (distance, player need, etc.)
    }

    pub fn get_best_target(
        &self,
        karma: f32,
        player_state: &PlayerState,
    ) -> Option<(Entity, BeamType)> {
        // Select most contextually relevant target
        // Consider player needs (hunger, health, etc.)
        // Weight by karma magnitude
    }
}
```

---

## 9. Beam Behavior

### 9.1 Lifecycle

```rust
impl ActiveBeam {
    pub fn update(&mut self, dt: f32) -> BeamStatus {
        self.lifetime += dt;

        match &mut self.fade_state {
            FadeState::FadingIn { progress, duration } => {
                *progress += dt / *duration;
                self.intensity = ease_out_cubic(*progress);

                if *progress >= 1.0 {
                    self.fade_state = FadeState::Sustained;
                }
            }
            FadeState::Sustained => {
                // Gentle breathing/pulsing
                self.intensity = 0.85 + 0.15 * (self.lifetime * 0.5).sin();

                // Check if should start fading
                if self.lifetime > self.max_lifetime - 2.0 {
                    self.fade_state = FadeState::FadingOut {
                        progress: 0.0,
                        duration: 2.0,
                    };
                }
            }
            FadeState::FadingOut { progress, duration } => {
                *progress += dt / *duration;
                self.intensity = 1.0 - ease_in_cubic(*progress);

                if *progress >= 1.0 {
                    return BeamStatus::Finished;
                }
            }
        }

        BeamStatus::Active
    }
}
```

### 9.2 Target Tracking

```rust
impl ActiveBeam {
    pub fn track_target(&mut self, world: &World) {
        // Get current target position
        if let Some(target_transform) = world.get::<Transform>(self.target_entity) {
            let new_pos = target_transform.translation;

            // Smooth follow for moving targets (animals)
            self.target_position = self.target_position.lerp(
                new_pos,
                0.1  // Smooth factor
            );

            // Recalculate origin (sky position above target)
            self.origin_position = self.calculate_sky_origin();
        }
    }

    fn calculate_sky_origin(&self) -> Vec3 {
        // Origin is high above target, slightly offset toward sun/moon
        let sky_height = 50.0;
        let celestial_offset = self.get_celestial_direction() * 10.0;

        Vec3::new(
            self.target_position.x + celestial_offset.x,
            self.target_position.y + sky_height,
            self.target_position.z + celestial_offset.z,
        )
    }
}
```

### 9.3 Player Interaction

```rust
pub fn should_dismiss_beam(beam: &ActiveBeam, player: &Player) -> bool {
    let distance_to_target = player.position.distance(beam.target_position);

    // Dismiss when player reaches target
    if distance_to_target < 3.0 {
        return true;
    }

    // Dismiss if player moves far away (gave up)
    if distance_to_target > 150.0 {
        return true;
    }

    // Dismiss if target no longer valid (animal fled, fruit picked)
    if !beam.target_entity.is_valid() {
        return true;
    }

    false
}
```

---

## 10. Audio Integration

### 10.1 Sound Design

```rust
pub struct BeamAudio {
    /// Ambient loop while beam active
    pub ambient_loop: AudioSource,

    /// One-shot on beam appearance
    pub appear_sound: AudioSource,

    /// One-shot on beam fade
    pub fade_sound: AudioSource,

    /// Volume (scales with intensity)
    pub volume: f32,

    /// Spatial position (at target)
    pub position: Vec3,
}
```

### 10.2 Sound by Beam Type

**Blessing Beams:**
- Appear: Soft chime, ascending tone
- Ambient: Wind through leaves, distant birdsong, gentle hum
- Fade: Descending chime, releasing breath

**Shadow Beams:**
- Appear: Low rumble, crack of distant thunder
- Ambient: Insect drone, wolf howl (distant), cold wind
- Fade: Growl fading, silence rushing in

**Wanderer Flickers:**
- Appear: Static pop, whisper fragment
- Ambient: Inconsistent hum, broken melody
- Fade: Cut to silence (abrupt)

---

## 11. Implementation Phases

### Phase 1: Core Visual System
- [ ] Create karma beam shader (extend light_shafts.wgsl)
- [ ] Implement beam geometry generation
- [ ] Add beam spawn/despawn with fade transitions
- [ ] Basic color temperature system

### Phase 2: Karma Integration
- [ ] Connect to NatureBalance system
- [ ] Implement karma threshold checks
- [ ] Create beam type selection based on karma sign
- [ ] Add beam frequency scaling

### Phase 3: Target Detection
- [ ] Implement spatial query for valid targets
- [ ] Create target categorization system
- [ ] Add context-aware target selection (hunger, health, etc.)
- [ ] Build target cache with update intervals

### Phase 4: Weather Integration
- [ ] Connect to WeatherSystem
- [ ] Implement weather-based beam availability
- [ ] Add cloud break timing for overcast
- [ ] Create day/night beam variations

### Phase 5: Polish & Atmosphere
- [ ] Add particle systems (dust motes, pollen, ash)
- [ ] Implement audio integration
- [ ] Add screen-space effects (subtle bloom)
- [ ] Fine-tune timing and feel

### Phase 6: Special Revelations
- [ ] Legendary tier unique beams
- [ ] Sacred site revelations
- [ ] Story-significant illuminations
- [ ] Rare celestial events

---

## Integration Points

### With Existing Systems

| System | Integration |
|--------|-------------|
| `NatureBalance` | Read karma value, subscribe to changes |
| `WeatherSystem` | Query state, influence cloud breaks |
| `light_shaft_pipeline.rs` | Extend shader, share render pass |
| `Animal System` | Query prey/friendly fauna positions |
| `Flora System` | Query fruit trees, medicinal plants |
| `Cave/Terrain System` | Query hidden entrances, shelters |
| `Player State` | Read hunger, thirst, health for context |
| `Audio System` | Spatial sound for beam ambience |

### Required Entity Components

```rust
/// Marks an entity as a potential karma beam target
#[derive(Component)]
pub struct KarmaBeamTarget {
    pub target_type: KarmaTargetType,
    pub priority: f32,
    pub min_karma_magnitude: f32,
    pub required_karma_sign: Option<KarmaSign>,
}

pub enum KarmaTargetType {
    // Positive targets
    FruitSource,
    WaterSource,
    Shelter,
    FriendlyFauna,
    HealingHerb,
    SacredSite,

    // Negative targets
    PreyAnimal,
    HiddenCave,
    EscapeRoute,
    Valuable,
    AmbushSpot,
    Vulnerability,
}
```
