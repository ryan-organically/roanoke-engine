# Character-Animal Transition State Specification

Robust state machine for all transitions between player character and animal models during interaction, mounting, combat, and ambient activities.

---

## Overview

### Design Goals

- **Interruption-Safe**: Any transition can be interrupted without leaving invalid state
- **Animation Sync**: Character and animal models blend seamlessly during transitions
- **Input Buffer**: Player inputs queue during transitions, execute on completion
- **Rollback Support**: Failed transitions restore previous state cleanly
- **Network-Ready**: State changes are discrete and serializable for multiplayer

### Scope

This spec covers transitions involving:
- Mounting/dismounting horses and other rideable animals
- Taming interactions (approach, feed, pet, calm)
- Combat with animals (attack, dodge, grapple)
- Utility interactions (grooming, healing, leading)
- Passive proximity events (animal reactions to player)

---

## Core State Architecture

### TransitionController

Central coordinator managing all character-animal transitions.

```rust
pub struct TransitionController {
    /// Current transition in progress (if any)
    active_transition: Option<ActiveTransition>,

    /// Queued inputs during transition
    input_buffer: VecDeque<BufferedInput>,

    /// Rollback snapshot for failed transitions
    rollback_state: Option<RollbackSnapshot>,

    /// Transition history for debugging
    history: VecDeque<TransitionRecord>,
}

pub struct ActiveTransition {
    pub kind: TransitionKind,
    pub phase: TransitionPhase,
    pub progress: f32,           // 0.0 - 1.0
    pub duration: f32,           // Total seconds
    pub elapsed: f32,
    pub target_entity: Option<EntityId>,
    pub can_cancel: bool,
    pub priority: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionPhase {
    /// Preparing to start (validation, positioning)
    Preparing,
    /// Animation playing, models blending
    Executing,
    /// Finalizing state changes
    Completing,
    /// Transition was cancelled/interrupted
    Cancelled,
    /// Transition failed (animal fled, etc.)
    Failed,
}
```

### TransitionKind

All possible transition types.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionKind {
    // === Mounting ===
    MountHorse,
    MountFromLeft,
    MountFromRight,
    MountRunningJump,       // Mount while horse is moving
    DismountStandard,
    DismountEmergency,      // Fall/thrown off
    DismountCombat,         // Quick dismount for combat

    // === Taming ===
    ApproachWild,           // Slow approach to wild animal
    ExtendHand,             // Offering hand to sniff
    OfferFood,              // Feeding wild animal
    AttemptPet,             // Petting/calming
    ApplyCalming,           // Using calming item/skill
    MountWildAttempt,       // First mount attempt on wild horse
    BuckingRide,            // Staying on bucking horse
    TamingSuccess,          // Animal accepts player
    TamingFailure,          // Animal flees/attacks

    // === Combat ===
    AttackAnimal,           // Player attacks
    AnimalAttack,           // Animal attacks player
    DodgeAnimal,            // Player dodges animal attack
    BlockAnimal,            // Player blocks with shield/weapon
    GrappleAnimal,          // Close combat grapple
    KnockedDown,            // Player knocked to ground
    GettingUp,              // Recovery from knockdown
    AnimalKilled,           // Finishing blow transition
    PlayerDeathByAnimal,    // Player killed by animal

    // === Utility ===
    LeadAnimal,             // Taking lead rope
    ReleaseAnimal,          // Releasing lead
    GroomAnimal,            // Grooming interaction
    HealAnimal,             // Applying medicine
    EquipSaddle,            // Saddling horse
    RemoveSaddle,           // Unsaddling

    // === Ambient ===
    AnimalNoticesPlayer,    // Animal becomes aware
    AnimalApproaches,       // Curious animal approaches
    AnimalFlees,            // Animal runs away
    AnimalThreatDisplay,    // Warning behavior
}
```

---

## State Machine

### Master State Diagram

```
                                    ┌─────────────────┐
                                    │   CHARACTER     │
                                    │   FREE_ROAM     │
                                    └────────┬────────┘
                                             │
           ┌─────────────────────────────────┼─────────────────────────────────┐
           │                                 │                                 │
           ▼                                 ▼                                 ▼
    ┌──────────────┐                 ┌──────────────┐                 ┌──────────────┐
    │   MOUNTING   │                 │   TAMING     │                 │   COMBAT     │
    │   CONTEXT    │                 │   CONTEXT    │                 │   CONTEXT    │
    └──────┬───────┘                 └──────┬───────┘                 └──────┬───────┘
           │                                 │                                 │
     ┌─────┴─────┐                    ┌──────┴──────┐                   ┌──────┴──────┐
     ▼           ▼                    ▼             ▼                   ▼             ▼
  Mounting   Dismounting         Approaching   Interacting         Attacking    Defending
     │           │                    │             │                   │             │
     ▼           ▼                    ▼             ▼                   ▼             ▼
  ┌──────────────┐                 ┌──────────────┐                 ┌──────────────┐
  │   MOUNTED    │                 │   BONDING    │                 │  STAGGERED/  │
  │   STATE      │◄───────────────►│   STATE      │                 │  DOWNED      │
  └──────────────┘                 └──────────────┘                 └──────────────┘
```

### Mounting Context States

```rust
pub enum MountingState {
    /// Player approaching mount point
    Approaching {
        target_horse: HorseId,
        approach_side: MountSide,
        distance: f32,
    },

    /// Positioning for mount animation
    Positioning {
        target_horse: HorseId,
        mount_point: Vec3,
        player_start: Vec3,
    },

    /// Mount animation playing
    Mounting {
        target_horse: HorseId,
        animation_progress: f32,
        ik_blend: f32,           // Hand/foot placement blend
    },

    /// Settling into mounted position
    Settling {
        horse: HorseId,
        settle_progress: f32,
    },

    /// Fully mounted, control transferred
    Mounted {
        horse: HorseId,
        stirrup_ik: bool,
        rein_ik: bool,
    },

    /// Dismount initiated
    Dismounting {
        horse: HorseId,
        dismount_type: DismountType,
        animation_progress: f32,
    },

    /// Landing after dismount
    Landing {
        landing_point: Vec3,
        landing_type: LandingType,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum MountSide {
    Left,
    Right,
    Rear,       // Vaulting over rear
    Running,    // Side doesn't matter for running mount
}

#[derive(Debug, Clone, Copy)]
pub enum DismountType {
    Standard,       // Clean dismount
    Quick,          // Fast dismount (combat)
    Emergency,      // Thrown/falling
    Forced,         // Bucked off
}

#[derive(Debug, Clone, Copy)]
pub enum LandingType {
    Standing,       // Clean landing on feet
    Rolling,        // Combat roll
    Stumbling,      // Rough landing, brief stagger
    Falling,        // Failed landing, knockdown
}
```

### Taming Context States

```rust
pub enum TamingState {
    /// Observing wild animal from distance
    Observing {
        target: AnimalId,
        distance: f32,
        animal_awareness: f32,
    },

    /// Slow approach, watching animal reaction
    Approaching {
        target: AnimalId,
        approach_speed: f32,
        crouch_level: f32,      // 0 = standing, 1 = full crouch
        hand_position: HandPosition,
    },

    /// Animal assessing player
    BeingAssessed {
        target: AnimalId,
        animal_decision_timer: f32,
        player_stillness: f32,
    },

    /// Offering food to animal
    OfferingFood {
        target: AnimalId,
        food_item: ItemId,
        hand_extended: bool,
        animal_interest: f32,
    },

    /// Animal eating from hand
    Feeding {
        target: AnimalId,
        feed_progress: f32,
        bond_gain: f32,
    },

    /// Attempting to pet/touch animal
    Petting {
        target: AnimalId,
        contact_point: PetLocation,
        pet_progress: f32,
        animal_tolerance: f32,
    },

    /// Wild horse mount attempt
    WildMountAttempt {
        target: HorseId,
        mount_progress: f32,
        stability: f32,         // How well player is holding on
    },

    /// Riding bucking horse
    Bucking {
        horse: HorseId,
        intensity: f32,
        player_balance: f32,
        input_window: bool,     // QTE window active
    },

    /// Taming completed successfully
    TamingComplete {
        animal: AnimalId,
        bond_established: f32,
    },

    /// Taming failed
    TamingFailed {
        animal: AnimalId,
        reason: TamingFailureReason,
        cooldown: f32,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum HandPosition {
    AtSide,
    Extended,
    Offering,
    Touching,
}

#[derive(Debug, Clone, Copy)]
pub enum PetLocation {
    Head,
    Neck,
    Shoulder,
    Flank,
    Muzzle,
}

#[derive(Debug, Clone, Copy)]
pub enum TamingFailureReason {
    AnimalFled,
    AnimalAttacked,
    PlayerMovedTooFast,
    TrustInsufficient,
    Interrupted,
    AnimalSpooked,
}
```

### Combat Context States

```rust
pub enum AnimalCombatState {
    /// No combat active
    Neutral,

    /// Player initiating attack
    PlayerAttacking {
        target: AnimalId,
        attack_type: AttackType,
        windup_progress: f32,
    },

    /// Attack connecting
    PlayerStriking {
        target: AnimalId,
        hit_location: HitZone,
        damage: f32,
    },

    /// Player recovery after attack
    PlayerRecovering {
        recovery_time: f32,
        can_cancel: bool,
    },

    /// Animal attacking player
    AnimalAttacking {
        attacker: AnimalId,
        attack_type: AnimalAttackType,
        telegraph_time: f32,    // Warning before hit
    },

    /// Player dodging
    PlayerDodging {
        dodge_direction: Vec3,
        i_frames: f32,          // Invincibility frames remaining
    },

    /// Player blocking
    PlayerBlocking {
        block_direction: Vec3,
        stamina_cost: f32,
        block_strength: f32,
    },

    /// Player hit by animal
    PlayerHit {
        attacker: AnimalId,
        damage: f32,
        stagger_duration: f32,
        knockback: Vec3,
    },

    /// Player knocked to ground
    PlayerKnocked {
        knockdown_type: KnockdownType,
        recovery_timer: f32,
        vulnerable: bool,
    },

    /// Player getting up from ground
    GettingUp {
        getup_type: GetupType,
        progress: f32,
        invulnerable: bool,
    },

    /// Grappling with animal
    Grappling {
        animal: AnimalId,
        grapple_type: GrappleType,
        player_advantage: f32,  // -1 to 1
        qte_active: bool,
    },

    /// Animal dying/death animation
    AnimalDying {
        animal: AnimalId,
        death_animation: f32,
    },

    /// Player death by animal
    PlayerDying {
        killer: AnimalId,
        death_type: DeathType,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum KnockdownType {
    FrontFall,      // Knocked forward
    BackFall,       // Knocked backward
    SideFall,       // Knocked sideways
    Ragdoll,        // Full ragdoll (heavy hit)
}

#[derive(Debug, Clone, Copy)]
pub enum GetupType {
    Quick,          // Fast recovery
    Standard,       // Normal recovery
    Slow,           // Damaged/exhausted
    Combat,         // Roll to feet
}

#[derive(Debug, Clone, Copy)]
pub enum GrappleType {
    BearHug,        // Large animal grab
    Pounced,        // Knocked down by pounce
    Coiled,         // Snake constriction
    BitHold,        // Wolf/cougar bite hold
    DeathRoll,      // Alligator grapple
}
```

---

## Transition Timing

### Duration Table

| Transition | Duration (sec) | Can Cancel | Priority |
|------------|----------------|------------|----------|
| MountStandard | 1.5 | 0.0-0.5 | 5 |
| MountFromLeft | 1.2 | 0.0-0.4 | 5 |
| MountFromRight | 1.2 | 0.0-0.4 | 5 |
| MountRunningJump | 0.8 | No | 6 |
| DismountStandard | 1.0 | 0.0-0.3 | 5 |
| DismountCombat | 0.5 | No | 7 |
| DismountEmergency | 0.3 | No | 8 |
| ApproachWild | Variable | Yes | 3 |
| OfferFood | 2.0 | 0.0-1.0 | 4 |
| AttemptPet | 1.5 | 0.0-0.8 | 4 |
| BuckingRide | Variable | No (auto-fail) | 7 |
| AttackAnimal | 0.3-1.0 | 0.0-0.2 | 6 |
| DodgeAnimal | 0.4 | No | 8 |
| KnockedDown | 0.5 | No | 9 |
| GettingUp | 0.8-1.5 | 0.3+ | 6 |
| Grappling | Variable | QTE only | 9 |

### Cancel Windows

Transitions have phases where cancellation is allowed:

```rust
impl ActiveTransition {
    pub fn can_cancel_now(&self) -> bool {
        if !self.can_cancel {
            return false;
        }

        match self.kind {
            TransitionKind::MountHorse => self.progress < 0.33,
            TransitionKind::DismountStandard => self.progress < 0.30,
            TransitionKind::ApproachWild => true,
            TransitionKind::OfferFood => self.progress < 0.50,
            TransitionKind::AttemptPet => self.progress < 0.53,
            TransitionKind::AttackAnimal => self.progress < 0.20,
            TransitionKind::GettingUp => self.progress > 0.40,
            _ => false,
        }
    }
}
```

---

## Animation Synchronization

### IK Targets During Mounting

```rust
pub struct MountIKTargets {
    /// Player's left hand target (saddle horn, mane)
    pub left_hand: Option<IKTarget>,

    /// Player's right hand target (saddle cantle, reins)
    pub right_hand: Option<IKTarget>,

    /// Player's left foot target (stirrup)
    pub left_foot: Option<IKTarget>,

    /// Player's right foot target (stirrup or swing)
    pub right_foot: Option<IKTarget>,

    /// Player pelvis target (saddle seat)
    pub pelvis: Option<IKTarget>,

    /// Blend weight for all IK (0 = animation, 1 = IK)
    pub blend: f32,
}

pub struct IKTarget {
    pub world_position: Vec3,
    pub world_rotation: Quat,
    pub bone_space_offset: Vec3,
}

impl MountIKTargets {
    /// Calculate IK targets for current mount phase
    pub fn calculate(
        horse: &Horse,
        player_skeleton: &Skeleton,
        mount_phase: f32,
        mount_side: MountSide,
    ) -> Self {
        let saddle_transform = horse.get_saddle_transform();

        match mount_side {
            MountSide::Left => Self::left_mount_targets(saddle_transform, mount_phase),
            MountSide::Right => Self::right_mount_targets(saddle_transform, mount_phase),
            MountSide::Running => Self::running_mount_targets(saddle_transform, mount_phase),
            MountSide::Rear => Self::rear_vault_targets(saddle_transform, mount_phase),
        }
    }

    fn left_mount_targets(saddle: Mat4, phase: f32) -> Self {
        // Phase 0.0-0.3: Approach and grab saddle
        // Phase 0.3-0.6: Swing leg over
        // Phase 0.6-1.0: Settle into seat

        let blend = if phase < 0.1 {
            phase / 0.1  // Fade in
        } else if phase > 0.9 {
            (1.0 - phase) / 0.1  // Fade out to mounted IK
        } else {
            1.0
        };

        Self {
            left_hand: Some(IKTarget {
                world_position: saddle.transform_point3(Vec3::new(-0.2, 0.3, 0.1)),
                world_rotation: Quat::IDENTITY,
                bone_space_offset: Vec3::ZERO,
            }),
            right_hand: Some(IKTarget {
                world_position: saddle.transform_point3(Vec3::new(0.2, 0.3, -0.1)),
                world_rotation: Quat::IDENTITY,
                bone_space_offset: Vec3::ZERO,
            }),
            left_foot: if phase > 0.2 {
                Some(IKTarget {
                    world_position: saddle.transform_point3(Vec3::new(-0.3, -0.4, 0.0)),
                    world_rotation: Quat::IDENTITY,
                    bone_space_offset: Vec3::ZERO,
                })
            } else {
                None
            },
            right_foot: if phase > 0.5 {
                Some(IKTarget {
                    world_position: saddle.transform_point3(Vec3::new(0.3, -0.4, 0.0)),
                    world_rotation: Quat::IDENTITY,
                    bone_space_offset: Vec3::ZERO,
                })
            } else {
                None
            },
            pelvis: if phase > 0.6 {
                Some(IKTarget {
                    world_position: saddle.transform_point3(Vec3::new(0.0, 0.0, 0.0)),
                    world_rotation: Quat::IDENTITY,
                    bone_space_offset: Vec3::ZERO,
                })
            } else {
                None
            },
            blend,
        }
    }
}
```

### Cross-Entity Animation Events

```rust
pub enum TransitionAnimEvent {
    // Player animation events
    PlayerFootPlanted { foot: Foot, surface: SurfaceType },
    PlayerHandContact { hand: Hand, target: ContactTarget },
    PlayerWeightTransfer { from: WeightBone, to: WeightBone },

    // Animal animation events
    AnimalReacts { reaction: AnimalReaction },
    AnimalWeightShift { direction: Vec3 },
    AnimalVocalization { sound: AnimalSound },

    // Sync points
    SyncPoint { name: &'static str },
    PhaseComplete { phase: u8 },

    // Physics events
    EnableRagdoll { body_part: BodyPart },
    DisableRagdoll,
    ApplyImpulse { bone: BoneName, force: Vec3 },
}

pub enum ContactTarget {
    SaddleHorn,
    SaddleCantle,
    Stirrup(Side),
    Mane,
    Reins,
    AnimalBody(BodyRegion),
}

pub enum AnimalReaction {
    EarsForward,
    EarsBack,
    HeadTurn { direction: f32 },
    TailSwish,
    WeightShift,
    Snort,
    Whinny,
    Stamp { foot: u8 },
}
```

---

## Input Handling

### Input Buffer System

```rust
pub struct InputBuffer {
    buffer: VecDeque<BufferedInput>,
    max_size: usize,
    max_age: f32,
}

pub struct BufferedInput {
    pub input: PlayerInput,
    pub timestamp: f32,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub enum PlayerInput {
    // Movement
    Move(Vec3),
    Jump,
    Crouch,
    Sprint,

    // Interaction
    Interact,
    Mount,
    Dismount,
    Attack(AttackInput),
    Dodge(Vec3),
    Block,

    // Taming
    OfferItem(ItemId),
    ExtendHand,
    Calm,

    // Utility
    Whistle,
    Call,
    Command(AnimalCommand),
}

impl InputBuffer {
    /// Process buffered input when transition completes
    pub fn process_on_complete(&mut self, current_time: f32) -> Option<PlayerInput> {
        // Remove stale inputs
        self.buffer.retain(|i| current_time - i.timestamp < self.max_age);

        // Get highest priority valid input
        self.buffer
            .iter()
            .filter(|i| current_time - i.timestamp < self.max_age)
            .max_by_key(|i| i.priority)
            .map(|i| i.input.clone())
    }

    /// Buffer input during transition
    pub fn buffer(&mut self, input: PlayerInput, time: f32, priority: u8) {
        if self.buffer.len() >= self.max_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(BufferedInput {
            input,
            timestamp: time,
            priority,
        });
    }
}
```

### Priority Interrupts

Higher priority transitions can interrupt lower priority ones:

```rust
impl TransitionController {
    pub fn attempt_interrupt(&mut self, new_kind: TransitionKind) -> bool {
        let new_priority = new_kind.priority();

        if let Some(ref active) = self.active_transition {
            if new_priority > active.priority {
                // Higher priority - force interrupt
                self.force_interrupt(new_kind);
                return true;
            } else if new_priority == active.priority && active.can_cancel_now() {
                // Same priority - check cancel window
                self.cancel_current();
                self.start_transition(new_kind);
                return true;
            }
            return false;
        }

        self.start_transition(new_kind);
        true
    }

    fn force_interrupt(&mut self, new_kind: TransitionKind) {
        // Emergency transitions (damage, knockdown) bypass normal flow
        if let Some(ref mut active) = self.active_transition {
            active.phase = TransitionPhase::Cancelled;

            // Snapshot for potential rollback
            self.rollback_state = Some(self.create_snapshot());
        }

        self.start_transition(new_kind);
    }
}

impl TransitionKind {
    pub fn priority(&self) -> u8 {
        match self {
            // Highest priority - cannot be interrupted
            Self::PlayerDeathByAnimal => 10,
            Self::KnockedDown => 9,
            Self::DismountEmergency => 9,
            Self::Grappling => 9,

            // High priority - interrupts most things
            Self::DodgeAnimal => 8,
            Self::AnimalAttack => 8,
            Self::DismountCombat => 7,
            Self::BuckingRide => 7,

            // Medium priority - normal interactions
            Self::AttackAnimal => 6,
            Self::MountRunningJump => 6,
            Self::GettingUp => 6,
            Self::MountHorse | Self::MountFromLeft | Self::MountFromRight => 5,
            Self::DismountStandard => 5,

            // Low priority - interruptible
            Self::OfferFood | Self::AttemptPet | Self::GroomAnimal => 4,
            Self::ApproachWild => 3,

            // Ambient - always interruptible
            Self::AnimalNoticesPlayer | Self::AnimalApproaches => 2,
            _ => 3,
        }
    }
}
```

---

## Rollback & Recovery

### State Snapshots

```rust
#[derive(Clone)]
pub struct RollbackSnapshot {
    pub player_state: PlayerSnapshot,
    pub animal_state: Option<AnimalSnapshot>,
    pub transition_kind: TransitionKind,
    pub timestamp: f32,
}

#[derive(Clone)]
pub struct PlayerSnapshot {
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,
    pub animation_state: AnimationState,
    pub mounted: bool,
    pub mount_id: Option<HorseId>,
}

#[derive(Clone)]
pub struct AnimalSnapshot {
    pub id: EntityId,
    pub position: Vec3,
    pub rotation: Quat,
    pub behavior_state: BehaviorState,
    pub mount_state: MountState,
}

impl TransitionController {
    /// Create snapshot before risky transition
    fn create_snapshot(&self) -> RollbackSnapshot {
        // Capture current state
        RollbackSnapshot {
            player_state: self.capture_player_state(),
            animal_state: self.capture_animal_state(),
            transition_kind: self.active_transition
                .as_ref()
                .map(|t| t.kind)
                .unwrap_or(TransitionKind::ApproachWild),
            timestamp: self.current_time,
        }
    }

    /// Restore from snapshot on failure
    pub fn rollback(&mut self) {
        if let Some(snapshot) = self.rollback_state.take() {
            self.apply_player_snapshot(&snapshot.player_state);
            if let Some(animal_snap) = &snapshot.animal_state {
                self.apply_animal_snapshot(animal_snap);
            }
            self.active_transition = None;
        }
    }
}
```

### Failure Handling

```rust
impl TransitionController {
    pub fn handle_failure(&mut self, reason: TransitionFailure) {
        let transition = match &self.active_transition {
            Some(t) => t,
            None => return,
        };

        match reason {
            TransitionFailure::AnimalFled => {
                // Animal ran away during interaction
                self.active_transition.as_mut().unwrap().phase = TransitionPhase::Failed;
                self.play_failure_animation(FailureAnim::ReachEmpty);
                self.rollback();
            }

            TransitionFailure::AnimalAttacked => {
                // Animal attacked during taming
                self.force_interrupt(TransitionKind::AnimalAttack);
            }

            TransitionFailure::PlayerHit => {
                // Player hit during mount/interaction
                self.force_interrupt(TransitionKind::PlayerHit);
            }

            TransitionFailure::AnimalMoved => {
                // Animal moved out of range
                if transition.kind.is_mount_transition() {
                    self.play_failure_animation(FailureAnim::StumbleOff);
                }
                self.rollback();
            }

            TransitionFailure::StaminaDepleted => {
                // Player ran out of stamina (bucking)
                self.force_interrupt(TransitionKind::DismountEmergency);
            }

            TransitionFailure::InputTimeout => {
                // Player didn't respond to QTE
                match transition.kind {
                    TransitionKind::BuckingRide => {
                        self.force_interrupt(TransitionKind::DismountEmergency);
                    }
                    TransitionKind::Grappling => {
                        self.force_interrupt(TransitionKind::KnockedDown);
                    }
                    _ => self.rollback(),
                }
            }
        }

        // Log for debugging
        self.history.push_back(TransitionRecord {
            kind: transition.kind,
            result: TransitionResult::Failed(reason),
            duration: transition.elapsed,
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TransitionFailure {
    AnimalFled,
    AnimalAttacked,
    AnimalMoved,
    PlayerHit,
    StaminaDepleted,
    InputTimeout,
    Interrupted,
    InvalidState,
}
```

---

## Camera Transitions

### Camera State During Transitions

```rust
pub struct TransitionCamera {
    /// Base camera state
    pub mode: CameraMode,

    /// Blend between modes
    pub blend_progress: f32,
    pub blend_from: CameraMode,
    pub blend_to: CameraMode,

    /// Transition-specific adjustments
    pub offset_override: Option<Vec3>,
    pub fov_override: Option<f32>,
    pub target_override: Option<Vec3>,
}

#[derive(Debug, Clone, Copy)]
pub enum CameraMode {
    FirstPerson,
    ThirdPersonClose,
    ThirdPersonMedium,
    ThirdPersonFar,
    Mounted,
    Cinematic { preset: CinematicPreset },
    Combat,
    Downed,
}

impl TransitionCamera {
    pub fn update_for_transition(&mut self, transition: &ActiveTransition, dt: f32) {
        match transition.kind {
            TransitionKind::MountHorse |
            TransitionKind::MountFromLeft |
            TransitionKind::MountFromRight => {
                // Pull camera back during mount
                self.blend_to = CameraMode::Mounted;
                self.blend_progress = transition.progress;

                // Cinematic angle for mount animation
                if transition.progress < 0.7 {
                    self.offset_override = Some(Vec3::new(
                        -2.0 * (1.0 - transition.progress),
                        1.5,
                        3.0,
                    ));
                }
            }

            TransitionKind::DismountStandard |
            TransitionKind::DismountCombat => {
                self.blend_to = CameraMode::ThirdPersonMedium;
                self.blend_progress = transition.progress;
            }

            TransitionKind::DismountEmergency => {
                // Quick snap to combat camera
                self.mode = CameraMode::Combat;
                self.blend_progress = 1.0;
            }

            TransitionKind::KnockedDown => {
                self.blend_to = CameraMode::Downed;
                self.blend_progress = transition.progress.min(1.0);

                // Camera shake
                let shake = (transition.elapsed * 20.0).sin() * (1.0 - transition.progress) * 0.1;
                self.offset_override = Some(Vec3::new(shake, 0.0, shake * 0.5));
            }

            TransitionKind::GettingUp => {
                self.blend_to = CameraMode::ThirdPersonMedium;
                self.blend_progress = transition.progress;
            }

            TransitionKind::ApproachWild |
            TransitionKind::OfferFood |
            TransitionKind::AttemptPet => {
                // Subtle zoom for intimate interactions
                self.fov_override = Some(60.0 - transition.progress * 10.0);
            }

            _ => {}
        }
    }
}
```

---

## Control Handoff

### Player-to-Mount Control Transfer

```rust
pub struct ControlHandoff {
    /// Who currently has movement control
    pub controller: ControlTarget,

    /// Blend between control sources
    pub blend: f32,

    /// Input remapping active
    pub remapping: Option<InputRemapping>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlTarget {
    Player,
    Mount,
    Shared { player_weight: f32 },
    Disabled,
}

pub struct InputRemapping {
    pub move_to_reins: bool,
    pub jump_to_rear: bool,
    pub sprint_to_gallop: bool,
    pub crouch_to_slow: bool,
    pub attack_mounted: bool,
}

impl ControlHandoff {
    pub fn update_for_mount_transition(&mut self, phase: f32, mounting: bool) {
        if mounting {
            // Transfer control to mount as mounting completes
            self.blend = phase;
            self.controller = if phase < 0.5 {
                ControlTarget::Player
            } else if phase < 0.9 {
                ControlTarget::Shared { player_weight: 1.0 - phase }
            } else {
                ControlTarget::Mount
            };

            // Enable mounted input remapping
            if phase > 0.8 {
                self.remapping = Some(InputRemapping {
                    move_to_reins: true,
                    jump_to_rear: true,
                    sprint_to_gallop: true,
                    crouch_to_slow: true,
                    attack_mounted: true,
                });
            }
        } else {
            // Dismounting - transfer back to player
            self.blend = 1.0 - phase;
            self.controller = if phase < 0.3 {
                ControlTarget::Mount
            } else if phase < 0.7 {
                ControlTarget::Shared { player_weight: phase }
            } else {
                ControlTarget::Player
            };

            // Disable mounted remapping
            if phase > 0.5 {
                self.remapping = None;
            }
        }
    }

    pub fn process_input(&self, input: PlayerInput, mount: Option<&mut Horse>) -> ProcessedInput {
        match self.controller {
            ControlTarget::Player => ProcessedInput::Player(input),

            ControlTarget::Mount => {
                if let Some(remap) = &self.remapping {
                    ProcessedInput::Mount(remap.remap(input))
                } else {
                    ProcessedInput::Ignored
                }
            }

            ControlTarget::Shared { player_weight } => {
                // Split input between player and mount
                ProcessedInput::Split {
                    player: input.clone(),
                    mount: self.remapping.as_ref().map(|r| r.remap(input)),
                    weight: player_weight,
                }
            }

            ControlTarget::Disabled => ProcessedInput::Ignored,
        }
    }
}
```

---

## Integration Points

### With Animation System

```rust
// In animation update loop
fn update_character_animation(
    player: &mut Player,
    transition_controller: &TransitionController,
    dt: f32,
) {
    if let Some(transition) = &transition_controller.active_transition {
        // Apply transition animation
        let anim = get_transition_animation(transition.kind);
        player.animation.play_transition(anim, transition.progress);

        // Apply IK if needed
        if let Some(ik_targets) = transition_controller.get_ik_targets() {
            player.skeleton.apply_ik(&ik_targets);
        }
    } else {
        // Normal animation update
        player.animation.update(dt);
    }
}
```

### With Physics System

```rust
// In physics update
fn update_transition_physics(
    player: &mut Player,
    transition_controller: &TransitionController,
    physics: &mut PhysicsWorld,
) {
    if let Some(transition) = &transition_controller.active_transition {
        match transition.kind {
            TransitionKind::KnockedDown |
            TransitionKind::DismountEmergency => {
                // Enable ragdoll for realistic fall
                physics.enable_ragdoll(player.entity);
            }

            TransitionKind::GettingUp => {
                // Transition from ragdoll to animated
                let blend = transition.progress;
                physics.blend_ragdoll_to_animation(player.entity, blend);
            }

            TransitionKind::MountHorse |
            TransitionKind::DismountStandard => {
                // Kinematic during mount/dismount
                physics.set_kinematic(player.entity, true);
            }

            _ => {}
        }
    }
}
```

### With Animal AI

```rust
// In animal behavior update
fn notify_animal_of_transition(
    animal: &mut Animal,
    transition: &ActiveTransition,
    player_pos: Vec3,
) {
    match transition.kind {
        TransitionKind::ApproachWild => {
            animal.awareness = (animal.awareness + 0.01).min(1.0);
            if transition.progress > 0.5 {
                animal.behavior_state = BehaviorState::Alert(AlertState::Looking);
            }
        }

        TransitionKind::OfferFood => {
            if animal.curiosity_level > 0.3 {
                animal.target = Some(Target::Position(player_pos));
            }
        }

        TransitionKind::AttemptPet => {
            // Animal decides to accept or reject
            let accept_chance = animal.taming_progress * 0.5 + animal.curiosity_level * 0.3;
            if rand::random::<f32>() > accept_chance {
                animal.behavior_state = BehaviorState::Flee(FleeState::Running);
            }
        }

        _ => {}
    }
}
```

---

## Testing Checklist

### Mounting/Dismounting
- [ ] Clean mount from left side
- [ ] Clean mount from right side
- [ ] Running mount while horse trotting
- [ ] Mount interrupted by damage
- [ ] Mount fails when horse moves away
- [ ] Dismount while stationary
- [ ] Dismount while moving (quick)
- [ ] Emergency dismount (bucked off)
- [ ] Dismount onto sloped terrain
- [ ] Mount/dismount in water

### Taming
- [ ] Approach alerts animal appropriately
- [ ] Slow approach reduces flee chance
- [ ] Fast approach triggers flee
- [ ] Food offering accepted by hungry animal
- [ ] Food offering rejected by scared animal
- [ ] Pet succeeds with sufficient trust
- [ ] Pet fails and animal flees
- [ ] Wild mount attempt triggers bucking
- [ ] Bucking QTE success leads to taming
- [ ] Bucking QTE failure throws player

### Combat
- [ ] Attack connects during telegraph window
- [ ] Dodge grants i-frames correctly
- [ ] Block reduces damage appropriately
- [ ] Knockdown triggers ragdoll
- [ ] Recovery from knockdown works
- [ ] Grapple QTE determines outcome
- [ ] Death animation plays fully
- [ ] Combat interrupts taming correctly

### Edge Cases
- [ ] Transition interrupted by another transition
- [ ] Multiple rapid mount/dismount attempts
- [ ] Transition during loading screen
- [ ] Animal despawns during interaction
- [ ] Network lag during transition (multiplayer)
- [ ] Save/load during active transition
