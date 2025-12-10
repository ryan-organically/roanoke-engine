# Idle & AFK Activity System Specification

## Overview

When players go idle (no input for extended periods), their character autonomously performs contextual idle activities rather than standing motionless. After 5 minutes of inactivity, the character transitions into immersive idle behaviors that reflect the environment, time of day, and available resources. This creates a living, breathing character even when the player steps away.

## Design Philosophy

A frontiersman doesn't stand frozen in place. When there's nothing pressing, they rest, tend to their gear, observe their surroundings, or simply enjoy a moment of peace. The idle system transforms AFK moments into atmospheric vignettes that reinforce the game's themes of survival, contemplation, and connection to the land.

---

## Idle State Machine

```
[ACTIVE]
    |
    | (No input for 30 seconds)
    v
[RESTLESS] ──────────────────────────────┐
    |                                     |
    | (No input for 2 minutes)            | (Any input)
    v                                     |
[IDLE_TRANSITION] ────────────────────────┤
    |                                     |
    | (Animation complete)                |
    v                                     |
[IDLE_ACTIVITY] ──────────────────────────┤
    |                                     |
    | (Activity duration complete)        |
    v                                     |
[IDLE_TRANSITION] ─── (New activity) ─────┘
    |
    | (Any input during any idle state)
    v
[WAKE_TRANSITION]
    |
    | (Stand up animation complete)
    v
[ACTIVE]
```

---

## Timing Thresholds

| Threshold | Duration | Trigger |
|-----------|----------|---------|
| **Micro-idle** | 10 seconds | Weight shift, look around |
| **Restless** | 30 seconds | Stretch, yawn, fidget |
| **Pre-idle** | 2 minutes | Crouch, inspect ground |
| **Full Idle** | 5 minutes | Sit, make camp, read |
| **Deep Idle** | 15 minutes | Sleep, meditate |
| **Extended AFK** | 30 minutes | Character finds shelter |

---

## Idle Activities

### Tier 1: Micro-Idle (10-30 seconds)
Subtle animations that don't interrupt gameplay feel.

| Activity | Animation | Duration |
|----------|-----------|----------|
| **Weight Shift** | Shift weight foot to foot | 3s |
| **Look Around** | Head turns, scanning horizon | 4s |
| **Stretch Neck** | Roll neck side to side | 2s |
| **Check Hands** | Examine palms briefly | 3s |
| **Breathe Deep** | Visible deep breath | 2s |

---

### Tier 2: Restless (30s - 2 min)
More noticeable fidgeting and minor activities.

| Activity | Animation | Duration |
|----------|-----------|----------|
| **Full Stretch** | Arms overhead stretch | 5s |
| **Yawn** | Cover mouth, yawn | 4s |
| **Crack Knuckles** | Flex hands | 3s |
| **Scratch Head** | Thoughtful head scratch | 3s |
| **Adjust Belt** | Fidget with equipment | 4s |
| **Kick Dirt** | Toe scuffs ground | 2s |
| **Cross Arms** | Fold arms, shift stance | 5s |

---

### Tier 3: Pre-Idle (2-5 min)
Transitional activities suggesting the character is settling in.

| Activity | Animation | Duration |
|----------|-----------|----------|
| **Crouch & Inspect** | Kneel, examine ground | 8s |
| **Pick Up Stone** | Grab nearby rock, toss it | 6s |
| **Listen Carefully** | Hand to ear, frozen | 5s |
| **Check Sky** | Look up, assess weather | 7s |
| **Inventory Check** | Pat pockets, check gear | 10s |
| **Find Spot** | Look around for place to sit | 8s |

---

### Tier 4: Full Idle (5+ min)
The character commits to a restful activity.

| Activity | Context Required | Animation | Duration | Loop |
|----------|------------------|-----------|----------|------|
| **Sit on Ground** | Any flat terrain | Cross-legged sit | 30s setup | Yes |
| **Sit on Rock** | Near rock/log | Perch on object | 20s setup | Yes |
| **Lean on Tree** | Near tree | Shoulder against bark | 15s setup | Yes |
| **Make Small Fire** | Has kindling + clear area | Build campfire | 45s setup | Yes |
| **Read Journal** | Has journal item | Pull out book, read | 20s setup | Yes |
| **Whittle Stick** | Has knife + near trees | Carve wood | 25s setup | Yes |
| **Skip Stones** | Near water | Throw stones at water | 15s setup | Yes |
| **Watch Wildlife** | Animals nearby | Track animal with eyes | 10s setup | Yes |
| **Cloud Watch** | Daytime, clear/partly cloudy | Lie back, look up | 30s setup | Yes |
| **Star Gaze** | Nighttime, clear sky | Lie back, look up | 30s setup | Yes |
| **Pray/Meditate** | Any | Kneel, bow head | 20s setup | Yes |
| **Hum/Whistle** | Any | Idle + audio | 5s setup | Yes |
| **Pet Companion** | Has animal companion | Stroke animal | 15s setup | Yes |
| **Tend Wound** | Has injury | Wrap bandage | 30s setup | No |
| **Sharpen Blade** | Has edged weapon | Whetstone on blade | 25s setup | Yes |
| **Check Map** | Has map item | Unfold, study | 20s setup | Yes |

---

### Tier 5: Deep Idle (15+ min)
Extended rest activities for long AFK periods.

| Activity | Context | Animation | Notes |
|----------|---------|-----------|-------|
| **Nap** | Daytime | Lean back, close eyes | Snoring audio |
| **Sleep** | Nighttime | Curl up / use bedroll | Full sleep cycle |
| **Deep Meditation** | Any | Lotus position | Mystical particles |
| **Fishing** | Near water + has rod | Cast line, wait | Can catch fish! |
| **Tend Fire** | Has campfire | Poke fire, add wood | Fire stays lit |
| **Craft Simple Item** | Has materials | Weave basket, etc. | Produces item |

---

### Tier 6: Extended AFK (30+ min)
Self-preservation behaviors.

| Activity | Trigger | Behavior |
|----------|---------|----------|
| **Seek Shelter** | Rain/storm incoming | Character moves to nearest cover |
| **Build Lean-to** | No shelter nearby | Construct basic shelter |
| **Retreat to Safety** | Predator nearby | Character moves away from danger |
| **Return to Camp** | Has established camp | Walk back to camp location |

---

## Context Selection Algorithm

```rust
fn select_idle_activity(player: &Player, world: &World, idle_tier: IdleTier) -> IdleActivity {
    let context = gather_context(player, world);
    let available = get_activities_for_tier(idle_tier);

    // Filter by context requirements
    let valid: Vec<_> = available.iter()
        .filter(|a| a.requirements_met(&context))
        .collect();

    // Weight by relevance
    let weighted: Vec<_> = valid.iter()
        .map(|a| (a, calculate_weight(a, &context)))
        .collect();

    // Select with weighted random
    weighted_random_select(&weighted)
}

struct IdleContext {
    // Environment
    terrain_type: TerrainType,
    near_water: bool,
    near_tree: bool,
    near_rock: bool,
    near_shelter: bool,

    // Time & Weather
    time_of_day: TimeOfDay,
    weather: WeatherType,
    temperature: f32,

    // Player State
    health_percent: f32,
    stamina_percent: f32,
    hunger_level: f32,
    has_injury: bool,

    // Inventory
    has_journal: bool,
    has_knife: bool,
    has_fishing_rod: bool,
    has_map: bool,
    has_bedroll: bool,
    has_kindling: bool,

    // Nearby
    animals_nearby: Vec<AnimalType>,
    npcs_nearby: Vec<NpcId>,
    threats_nearby: bool,

    // Companions
    has_animal_companion: bool,
    companion_type: Option<CompanionType>,
}
```

---

## Activity Weights

Higher weight = more likely to be selected.

| Factor | Weight Modifier |
|--------|-----------------|
| **Time Appropriate** | +50 (e.g., star gaze at night) |
| **Weather Appropriate** | +30 (e.g., seek shelter in rain) |
| **Has Required Item** | +20 |
| **Near Required Object** | +40 |
| **Health Low** | +60 to tend wound |
| **Recently Used** | -80 (variety preference) |
| **Player Has Done Before** | +10 (familiarity) |
| **Faction Appropriate** | +25 (native meditation, colonial prayer) |

---

## Wake-Up Behavior

When player provides input during idle:

### Interrupt Priority

| Input Type | Response |
|------------|----------|
| **Movement (WASD)** | Immediate stand, cancel activity |
| **Mouse Look** | Head turns first, then stand if continued |
| **Jump** | Quick stand + jump |
| **Attack** | Combat stance immediately |
| **Interact (E)** | Depends on distance to target |
| **Inventory (Tab)** | Stay seated, open menu |
| **Escape** | Stand, open pause menu |

### Wake Animation Timing

| Idle State | Wake Time |
|------------|-----------|
| Standing idle | Instant |
| Sitting | 0.8s |
| Lying down | 1.5s |
| Sleeping | 2.5s (groggy) |
| At campfire | 1.0s |

### Groggy Effect (Deep Sleep Wake)
- 3 second reduced movement speed
- Slight camera blur fade-out
- Yawn animation overlay
- "Waking up..." status indicator

---

## Visual & Audio Feedback

### Idle State Indicators

| State | Visual | Audio |
|-------|--------|-------|
| Micro-idle | None | None |
| Restless | Subtle sway | Occasional sigh |
| Pre-idle | Looking around | Contemplative hum |
| Full Idle | Activity animation | Contextual sounds |
| Deep Idle | Relaxed posture | Breathing/snoring |
| Sleep | Closed eyes, slow breathing | Soft snoring |

### Activity-Specific Audio

| Activity | Ambient Sound |
|----------|---------------|
| Campfire | Crackling fire |
| Whittling | Scraping knife |
| Reading | Page turns |
| Fishing | Water lapping, line cast |
| Sharpening | Whetstone grinding |
| Humming | Procedural melody |
| Praying | Whispered words |

### Environmental Reactions

| Event | Idle Response |
|-------|---------------|
| Animal approaches | Head tracks animal |
| NPC walks by | Nod greeting |
| Weather changes | Look at sky |
| Loud noise | Startle, look toward |
| Predator spotted | Stand immediately, alert |

---

## Campfire System (Special Case)

When player is idle with kindling and in appropriate location:

### Auto-Camp Sequence

```
1. [5 min idle] Character looks around
2. [5:10] Gathers nearby sticks (if available)
3. [5:30] Clears small area
4. [6:00] Arranges kindling
5. [6:30] Strikes flint (if has flint) or rubs sticks
6. [7:00] Fire starts (small)
7. [7:30] Adds fuel, fire grows
8. [8:00+] Sits by fire, tends it periodically
```

### Fire Benefits While Idle
- Warmth (prevents hypothermia)
- Light (extends visibility at night)
- Predator deterrent (animals stay away)
- Cooking (if has raw food, auto-cooks)
- Social (nearby NPCs may join)

### Fire Duration
- Requires fuel every 10 minutes (auto-tends if has wood)
- Dies after 30 minutes without fuel
- Character will gather more wood if nearby and idle

---

## Data Structures

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdleSystem {
    pub state: IdleState,
    pub time_since_input: f32,
    pub current_activity: Option<IdleActivity>,
    pub activity_progress: f32,
    pub activities_performed: Vec<IdleActivityType>,
    pub last_activity_time: f64,
    pub campfire: Option<CampfireState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdleState {
    Active,
    MicroIdle,
    Restless,
    PreIdle,
    IdleTransition,
    FullIdle,
    DeepIdle,
    ExtendedAfk,
    WakeTransition,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdleActivity {
    pub activity_type: IdleActivityType,
    pub setup_duration: f32,
    pub loop_duration: Option<f32>,
    pub can_loop: bool,
    pub position_offset: Vec3,
    pub rotation_offset: f32,
    pub animation_id: String,
    pub audio_id: Option<String>,
    pub particle_effect: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdleActivityType {
    // Tier 1 - Micro
    WeightShift,
    LookAround,
    StretchNeck,
    CheckHands,
    BreatheDeep,

    // Tier 2 - Restless
    FullStretch,
    Yawn,
    CrackKnuckles,
    ScratchHead,
    AdjustBelt,
    KickDirt,
    CrossArms,

    // Tier 3 - Pre-Idle
    CrouchInspect,
    PickUpStone,
    ListenCarefully,
    CheckSky,
    InventoryCheck,
    FindSpot,

    // Tier 4 - Full Idle
    SitGround,
    SitRock,
    LeanTree,
    MakeFire,
    ReadJournal,
    WhittleStick,
    SkipStones,
    WatchWildlife,
    CloudWatch,
    StarGaze,
    PrayMeditate,
    HumWhistle,
    PetCompanion,
    TendWound,
    SharpenBlade,
    CheckMap,

    // Tier 5 - Deep Idle
    Nap,
    Sleep,
    DeepMeditation,
    Fishing,
    TendFire,
    CraftSimple,

    // Tier 6 - Extended AFK
    SeekShelter,
    BuildLeanto,
    RetreatSafety,
    ReturnCamp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampfireState {
    pub position: Vec3,
    pub fuel_remaining: f32,
    pub fire_size: f32,
    pub time_since_tend: f32,
    pub items_cooking: Vec<CookingItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdleActivityRequirements {
    pub terrain: Option<Vec<TerrainType>>,
    pub near_water: Option<bool>,
    pub near_tree: Option<bool>,
    pub near_rock: Option<bool>,
    pub time_of_day: Option<Vec<TimeOfDay>>,
    pub weather: Option<Vec<WeatherType>>,
    pub min_health: Option<f32>,
    pub required_items: Vec<String>,
    pub required_companion: Option<bool>,
    pub no_threats: bool,
}

impl IdleActivity {
    pub fn requirements_met(&self, context: &IdleContext) -> bool {
        let req = &self.requirements;

        // Check terrain
        if let Some(terrains) = &req.terrain {
            if !terrains.contains(&context.terrain_type) {
                return false;
            }
        }

        // Check proximity requirements
        if req.near_water == Some(true) && !context.near_water { return false; }
        if req.near_tree == Some(true) && !context.near_tree { return false; }
        if req.near_rock == Some(true) && !context.near_rock { return false; }

        // Check time of day
        if let Some(times) = &req.time_of_day {
            if !times.contains(&context.time_of_day) {
                return false;
            }
        }

        // Check weather
        if let Some(weathers) = &req.weather {
            if !weathers.contains(&context.weather) {
                return false;
            }
        }

        // Check items
        for item in &req.required_items {
            if !context.has_item(item) {
                return false;
            }
        }

        // Check threats
        if req.no_threats && context.threats_nearby {
            return false;
        }

        true
    }
}
```

---

## Integration Points

### Input System
```rust
fn update_idle_system(idle: &mut IdleSystem, input: &InputState, dt: f32) {
    if input.any_input() {
        if idle.state != IdleState::Active {
            idle.state = IdleState::WakeTransition;
            idle.start_wake_animation();
        }
        idle.time_since_input = 0.0;
    } else {
        idle.time_since_input += dt;
        idle.update_state_machine();
    }
}
```

### Animation System
```rust
fn get_player_animation(player: &Player, idle: &IdleSystem) -> AnimationId {
    match idle.state {
        IdleState::Active => player.movement_animation(),
        IdleState::MicroIdle => idle.current_activity.animation_id,
        IdleState::FullIdle => idle.current_activity.animation_id,
        IdleState::WakeTransition => "stand_up".into(),
        // ...
    }
}
```

### Camera System
```rust
fn update_idle_camera(camera: &mut Camera, idle: &IdleSystem) {
    match idle.state {
        IdleState::FullIdle | IdleState::DeepIdle => {
            // Gentle camera sway
            camera.add_idle_drift(0.001);

            // Slow zoom out to show more environment
            camera.target_fov = 55.0; // Wider than normal 45
        }
        IdleState::WakeTransition => {
            // Snap back to normal
            camera.target_fov = 45.0;
        }
        _ => {}
    }
}
```

### Threat Response
```rust
fn check_idle_threats(idle: &mut IdleSystem, threats: &[Threat]) {
    if idle.state.is_idle() && !threats.is_empty() {
        let closest = threats.iter().min_by_key(|t| t.distance as i32);
        if let Some(threat) = closest {
            if threat.distance < THREAT_WAKE_DISTANCE {
                idle.force_wake(WakeReason::Threat);
                // Also triggers alert animation
            }
        }
    }
}
```

---

## Configuration

### Timing (Adjustable in Settings)

```rust
pub struct IdleConfig {
    pub micro_idle_threshold: f32,      // Default: 10.0
    pub restless_threshold: f32,        // Default: 30.0
    pub pre_idle_threshold: f32,        // Default: 120.0
    pub full_idle_threshold: f32,       // Default: 300.0 (5 min)
    pub deep_idle_threshold: f32,       // Default: 900.0 (15 min)
    pub extended_afk_threshold: f32,    // Default: 1800.0 (30 min)
    pub activity_variety_weight: f32,   // Default: 0.8
    pub enable_auto_camp: bool,         // Default: true
    pub enable_auto_shelter: bool,      // Default: true
    pub enable_idle_camera: bool,       // Default: true
}
```

### Disable Conditions

Idle system disabled when:
- In combat
- In dialogue
- In menu/UI
- Underwater
- Falling
- On horse (separate horse idle system)
- In cutscene

---

## Implementation Priority

### Phase 1 (Core)
- [ ] Idle state machine
- [ ] Time tracking since last input
- [ ] Basic sitting animation
- [ ] Wake-up on input

### Phase 2 (Variety)
- [ ] Micro-idle animations (weight shift, look around)
- [ ] Restless animations (stretch, yawn)
- [ ] Context detection (near water, tree, rock)
- [ ] Activity selection algorithm

### Phase 3 (Full Idle)
- [ ] All Tier 4 activities
- [ ] Item-based activities (read journal, sharpen blade)
- [ ] Environmental reactions
- [ ] Idle audio

### Phase 4 (Deep Features)
- [ ] Campfire auto-build
- [ ] Deep idle (nap, sleep)
- [ ] Extended AFK shelter-seeking
- [ ] Companion interactions

### Phase 5 (Polish)
- [ ] Idle camera drift
- [ ] Activity-specific particles
- [ ] NPC reactions to idle player
- [ ] Fishing mini-rewards

---

## Multiplayer Considerations (Future)

- Other players see your idle animations
- Can interact with idle players (wave, poke)
- Idle players show "AFK" indicator after 10 min
- Auto-shelter prevents death from weather
- PvP servers: idle players still vulnerable

---

## Performance Notes

- Idle state check: Once per frame (cheap)
- Context gathering: Once per state transition (moderate)
- Activity selection: Once per activity change (moderate)
- Animation blending: Standard animation system cost
- No pathfinding during most idle states

---

## Historical Notes

Colonial-era frontiersmen and native peoples both had rich traditions of rest and contemplation:

- **Fireside culture**: The campfire was the social center, where stories were told and skills passed down
- **Whittling**: A universal pastime for anyone with a knife and time to spare
- **Prayer & meditation**: Both European and native traditions valued spiritual reflection
- **Nature observation**: Skilled hunters spent hours watching animal behavior
- **Oral tradition**: Humming, singing, and storytelling filled quiet moments

This system aims to capture that spirit of purposeful rest that was essential to frontier life.
