//! Animal entity - runtime state for individual animals

use super::behavior::BehaviorState;
use super::types::{AnimalSpecies, StatusEffectType};
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Unique runtime identifier for animal instances
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnimalId(pub u64);

/// Unique identifier for a pack of animals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackId(pub u64);

/// What the animal is currently targeting
#[derive(Debug, Clone)]
pub enum Target {
    Player,
    Position(Vec3),
    Animal(AnimalId),
    FleeFrom(Vec3),
}

/// Source of damage
#[derive(Debug, Clone)]
pub enum DamageSource {
    Player,
    Animal(AnimalId),
    Environment,
    StatusEffect,
}

/// Active status effect on an entity
#[derive(Debug, Clone)]
pub struct ActiveStatusEffect {
    pub effect_type: StatusEffectType,
    pub remaining_duration: f32,
    pub source: DamageSource,
}

/// Animation state for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    Walking,
    Running,
    Attacking,
    TakingDamage,
    Dying,
    Dead,
}

/// Runtime state for a single animal instance
#[derive(Debug)]
pub struct Animal {
    // Identity
    pub id: AnimalId,
    pub species: AnimalSpecies,
    pub pack_id: Option<PackId>,

    // Transform
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,

    // Physics
    pub on_ground: bool,
    pub in_water: bool,

    // Combat state
    pub current_health: f32,
    pub max_health: f32,
    pub active_effects: Vec<ActiveStatusEffect>,
    pub attack_cooldowns: Vec<f32>,
    pub last_damage_time: Option<Instant>,
    pub damage_source: Option<DamageSource>,

    // AI state
    pub behavior_state: BehaviorState,
    pub target: Option<Target>,
    pub home_position: Vec3,
    pub territory_radius: f32,
    pub awareness: f32, // 0.0 = unaware, 1.0 = fully alert
    pub last_seen_player: Option<(Vec3, Instant)>,

    // Animation
    pub animation_state: AnimationState,
    pub animation_time: f32,

    // Spawning
    pub spawn_chunk: (i32, i32),
    pub despawn_timer: Option<f32>,
}

impl Animal {
    /// Create a new animal at the given position
    pub fn new(
        id: AnimalId,
        species: AnimalSpecies,
        position: Vec3,
        health: f32,
        chunk: (i32, i32),
    ) -> Self {
        let num_attacks = species.attacks().len();

        Self {
            id,
            species,
            pack_id: None,
            position,
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            on_ground: true,
            in_water: false,
            current_health: health,
            max_health: health,
            active_effects: Vec::new(),
            attack_cooldowns: vec![0.0; num_attacks],
            last_damage_time: None,
            damage_source: None,
            behavior_state: BehaviorState::Idle,
            target: None,
            home_position: position,
            territory_radius: 30.0,
            awareness: 0.0,
            last_seen_player: None,
            animation_state: AnimationState::Idle,
            animation_time: 0.0,
            spawn_chunk: chunk,
            despawn_timer: None,
        }
    }

    /// Check if the animal is alive
    pub fn is_alive(&self) -> bool {
        self.current_health > 0.0
    }

    /// Check if the animal is dead
    pub fn is_dead(&self) -> bool {
        self.current_health <= 0.0
    }

    /// Get the current speed based on terrain and state
    pub fn current_speed(&self) -> f32 {
        let base_stats = self.species.base_stats();
        let base_speed = if self.in_water {
            base_stats.speed_in_water.unwrap_or(base_stats.speed * 0.5)
        } else {
            base_stats.speed
        };

        // Modify by behavior state
        let state_modifier = match self.behavior_state {
            BehaviorState::Idle | BehaviorState::Alert(_) => 0.0,
            BehaviorState::Patrol => 0.5,
            BehaviorState::Pursue(_) => 1.0,
            BehaviorState::Attack(_) => 0.3,
            BehaviorState::Flee(_) => 1.2, // Faster when fleeing
            BehaviorState::Dead => 0.0,
        };

        base_speed * state_modifier
    }

    /// Get the direction the animal is facing (XZ plane)
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    /// Face toward a target position
    pub fn look_at(&mut self, target: Vec3) {
        let direction = (target - self.position).normalize();
        if direction.length_squared() > 0.001 {
            let yaw = direction.z.atan2(direction.x);
            self.rotation = Quat::from_rotation_y(-yaw + std::f32::consts::FRAC_PI_2);
        }
    }

    /// Apply damage to this animal
    pub fn take_damage(&mut self, amount: f32, source: DamageSource) {
        self.current_health = (self.current_health - amount).max(0.0);
        self.last_damage_time = Some(Instant::now());
        self.damage_source = Some(source);

        // Increase awareness when damaged
        self.awareness = 1.0;

        // Update animation
        if self.current_health <= 0.0 {
            self.animation_state = AnimationState::Dying;
            self.behavior_state = BehaviorState::Dead;
        } else {
            self.animation_state = AnimationState::TakingDamage;
        }
    }

    /// Apply a status effect
    pub fn apply_effect(&mut self, effect_type: StatusEffectType, source: DamageSource) {
        // Check if we already have this effect
        for effect in &mut self.active_effects {
            if effect.effect_type == effect_type {
                // Refresh duration
                effect.remaining_duration = effect_type.duration();
                return;
            }
        }

        // Add new effect
        self.active_effects.push(ActiveStatusEffect {
            effect_type,
            remaining_duration: effect_type.duration(),
            source,
        });
    }

    /// Update status effects, returns total damage dealt this tick
    pub fn update_effects(&mut self, dt: f32) -> f32 {
        let mut total_damage = 0.0;

        self.active_effects.retain_mut(|effect| {
            total_damage += effect.effect_type.damage_per_second() * dt;
            effect.remaining_duration -= dt;
            effect.remaining_duration > 0.0
        });

        if total_damage > 0.0 {
            self.current_health = (self.current_health - total_damage).max(0.0);
        }

        total_damage
    }

    /// Update attack cooldowns
    pub fn update_cooldowns(&mut self, dt: f32) {
        for cooldown in &mut self.attack_cooldowns {
            *cooldown = (*cooldown - dt).max(0.0);
        }
    }

    /// Check if a specific attack is ready
    pub fn attack_ready(&self, attack_index: usize) -> bool {
        self.attack_cooldowns
            .get(attack_index)
            .map(|&cd| cd <= 0.0)
            .unwrap_or(false)
    }

    /// Get the best available attack for the current situation
    pub fn select_attack(&self, distance_to_target: f32) -> Option<usize> {
        let attacks = self.species.attacks();
        let base_range = self.species.base_stats().attack_range;

        // Find first ready attack that's in range
        for (i, _attack) in attacks.iter().enumerate() {
            if self.attack_ready(i) && distance_to_target <= base_range {
                return Some(i);
            }
        }

        None
    }

    /// Trigger an attack, setting cooldown
    pub fn perform_attack(&mut self, attack_index: usize) {
        if let Some(cooldown) = self.attack_cooldowns.get_mut(attack_index) {
            let attacks = self.species.attacks();
            if let Some(attack) = attacks.get(attack_index) {
                *cooldown = attack.cooldown;
                self.animation_state = AnimationState::Attacking;
            }
        }
    }

    /// Update animation state based on behavior
    pub fn update_animation(&mut self, dt: f32) {
        self.animation_time += dt;

        // Auto-transition from one-shot animations
        match self.animation_state {
            AnimationState::TakingDamage if self.animation_time > 0.3 => {
                self.animation_state = AnimationState::Idle;
                self.animation_time = 0.0;
            }
            AnimationState::Attacking if self.animation_time > 0.5 => {
                self.animation_state = AnimationState::Idle;
                self.animation_time = 0.0;
            }
            AnimationState::Dying if self.animation_time > 2.0 => {
                self.animation_state = AnimationState::Dead;
            }
            _ => {}
        }

        // Set animation based on movement
        if self.animation_state == AnimationState::Idle {
            let speed = self.velocity.length();
            if speed > 5.0 {
                self.animation_state = AnimationState::Running;
            } else if speed > 0.5 {
                self.animation_state = AnimationState::Walking;
            }
        }
    }

    /// Get the damage flash intensity (0.0 = no flash, 1.0 = full flash)
    /// Flash lasts 0.3 seconds and fades out
    pub fn damage_flash_intensity(&self) -> f32 {
        const FLASH_DURATION: f32 = 0.3; // seconds

        match &self.last_damage_time {
            Some(time) => {
                let elapsed = time.elapsed().as_secs_f32();
                if elapsed < FLASH_DURATION {
                    // Fade out over the duration
                    1.0 - (elapsed / FLASH_DURATION)
                } else {
                    0.0
                }
            }
            None => 0.0,
        }
    }

    /// Get color and emissive multipliers for damage flash effect
    /// Returns (color_tint: [f32; 3], emissive_boost: f32)
    pub fn damage_flash_effect(&self) -> ([f32; 3], f32) {
        let intensity = self.damage_flash_intensity();
        if intensity > 0.0 {
            // Blend towards white/red with intensity
            let red_boost = 1.0 + intensity * 0.5;   // More red
            let other_boost = 1.0 + intensity * 0.3; // White-ish flash
            ([red_boost, other_boost, other_boost], intensity * 1.5)
        } else {
            ([1.0, 1.0, 1.0], 0.0)
        }
    }
}
