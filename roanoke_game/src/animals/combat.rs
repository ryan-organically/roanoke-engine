//! Combat system - damage processing between animals and player

use super::behavior::{AttackState, BehaviorState};
use super::entity::AnimalId;
use super::manager::AnimalManager;
use super::types::StatusEffectType;
use glam::Vec3;

/// Result of an attack attempt
#[derive(Debug)]
pub struct AttackResult {
    pub hit: bool,
    pub damage: f32,
    pub effect: Option<StatusEffectType>,
    pub knockback: Option<Vec3>,
    pub attack_name: &'static str,
}

/// Process attacks from animals to the player
pub fn process_animal_attacks(
    manager: &mut AnimalManager,
    player_pos: Vec3,
) -> Vec<AttackResult> {
    let mut results = Vec::new();

    // Collect attacking animals
    let attackers: Vec<AnimalId> = manager
        .animals_iter()
        .filter(|a| matches!(a.behavior_state, BehaviorState::Attack(AttackState::Striking)))
        .map(|a| a.id)
        .collect();

    for id in attackers {
        if let Some(result) = process_single_attack(manager, id, player_pos) {
            results.push(result);
        }
    }

    results
}

/// Process a single animal's attack
fn process_single_attack(
    manager: &mut AnimalManager,
    attacker_id: AnimalId,
    player_pos: Vec3,
) -> Option<AttackResult> {
    let animal = manager.get(attacker_id)?;
    let species = animal.species;
    let stats = species.base_stats();

    // Distance check
    let dist = animal.position.distance(player_pos);
    if dist > stats.attack_range * 1.5 {
        return None; // Miss - player moved out of range
    }

    // Select attack
    let attacks = species.attacks();
    let attack_idx = animal.select_attack(dist)?;
    let attack = attacks.get(attack_idx)?;

    // Apply difficulty modifier to damage
    let difficulty = manager.difficulty;
    let base_damage = attack.damage;
    let modified_damage = base_damage * difficulty.damage_multiplier();

    // Calculate knockback direction
    let knockback = if matches!(
        attack.effect,
        Some(StatusEffectType::Knockback) | Some(StatusEffectType::Knockdown)
    ) {
        let dir = (player_pos - animal.position).normalize_or_zero();
        Some(dir * 5.0) // Knockback strength
    } else {
        None
    };

    // Set cooldown and transition state
    if let Some(animal) = manager.get_mut(attacker_id) {
        animal.perform_attack(attack_idx);
        animal.behavior_state = BehaviorState::Attack(AttackState::Recovering);
    }

    Some(AttackResult {
        hit: true,
        damage: modified_damage,
        effect: attack.effect,
        knockback,
        attack_name: attack.name,
    })
}

/// Combat context from player progression
#[derive(Debug, Clone, Default)]
pub struct CombatContext {
    /// Damage multiplier from skills (e.g., Beast Slayer +50%)
    pub damage_bonus: f32,
    /// Critical hit chance from skills
    pub crit_chance: f32,
    /// Critical hit damage multiplier
    pub crit_multiplier: f32,
    /// Whether this is a stealth attack
    pub is_stealth: bool,
    /// Whether player has predator sense (warns of ambushes)
    pub has_predator_sense: bool,
    /// Detection range reduction from skills
    pub detection_reduction: f32,
    /// Armor/damage reduction
    pub damage_reduction: f32,
}

impl CombatContext {
    /// Create combat context from player progression skills
    pub fn from_skills(
        species_name: &str,
        is_stealth: bool,
        hunting_damage_bonus: f32,
        detection_reduction: f32,
        has_apex_predator: bool,
        has_shadow_hunter: bool,
        has_predator_sense: bool,
    ) -> Self {
        let mut ctx = Self {
            damage_bonus: hunting_damage_bonus,
            crit_chance: 0.05, // Base 5% crit
            crit_multiplier: 1.5,
            is_stealth,
            has_predator_sense,
            detection_reduction,
            damage_reduction: 0.0,
        };

        // Shadow Hunter: +100% stealth damage, +20% crit chance on stealth
        if is_stealth && has_shadow_hunter {
            ctx.damage_bonus += 1.0;
            ctx.crit_chance += 0.20;
        }

        // Apex Predator: +25% crit multiplier, +10% base crit
        if has_apex_predator {
            ctx.crit_chance += 0.10;
            ctx.crit_multiplier += 0.25;
        }

        ctx
    }
}

/// Player attacks an animal - returns true if killed
pub fn player_attack_animal(
    manager: &mut AnimalManager,
    animal_id: AnimalId,
    base_damage: f32,
    weapon_type: Option<&str>,
    combat_ctx: &CombatContext,
) -> Option<AnimalAttackResult> {
    let animal = manager.get(animal_id)?;
    let species = animal.species;
    let species_name = format!("{:?}", species);
    let was_alive = animal.is_alive();
    let pos = animal.position;
    let was_alert = animal.awareness > 0.5;

    // Calculate final damage with skill bonuses
    let mut final_damage = base_damage * (1.0 + combat_ctx.damage_bonus);

    // Stealth bonus if enemy wasn't aware
    if combat_ctx.is_stealth && !was_alert {
        final_damage *= 1.5; // 50% bonus for true stealth hit
    }

    // Check for critical hit
    let is_critical = rand_crit(combat_ctx.crit_chance);
    if is_critical {
        final_damage *= combat_ctx.crit_multiplier;
    }

    // Check weapon vs weakness for bonus damage
    if let Some(weapon) = weapon_type {
        final_damage *= get_weapon_effectiveness(weapon, &species_name);
    }

    // Apply damage
    let killed = manager.damage_animal(animal_id, final_damage);

    // If this was a stealth kill or headshot, update animal aggro
    if combat_ctx.is_stealth && killed {
        // Stealth kill doesn't alert nearby animals as much
        manager.reduce_nearby_awareness(pos, 30.0, 0.3);
    }

    Some(AnimalAttackResult {
        hit: true,
        damage_dealt: final_damage,
        killed,
        was_alive,
        species,
        position: pos,
        was_stealth: combat_ctx.is_stealth && !was_alert,
        was_critical: is_critical,
    })
}

/// Simple random for critical hits
fn rand_crit(chance: f32) -> bool {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let frame = COUNTER.fetch_add(1, Ordering::Relaxed);
    let hash = frame.wrapping_mul(0x517cc1b727220a95);
    (hash as f32 / u64::MAX as f32) < chance
}

/// Get weapon effectiveness against species
fn get_weapon_effectiveness(weapon: &str, species: &str) -> f32 {
    // Weapons have different effectiveness vs different animals
    match (weapon, species) {
        // Bows are great vs most animals
        ("bow", _) => 1.2,
        ("hunter_bow", _) => 1.3,
        ("hunter_bow_improved", _) => 1.4,

        // Spears are good vs large game
        ("spear", "BlackBear" | "AmericanAlligator") => 1.4,
        ("spear", _) => 1.1,

        // Knives for finishing/small game
        ("knife", "TimberRattlesnake" | "Copperhead") => 1.5,
        ("hunter_knife", _) => 1.2,

        // Clubs/blunt for knockdown
        ("club", _) => 0.9, // Less damage but more stagger

        // Default
        _ => 1.0,
    }
}

/// Player attack result with extended info
pub fn player_attack_animal_simple(
    manager: &mut AnimalManager,
    animal_id: AnimalId,
    damage: f32,
    _weapon_type: Option<&str>,
) -> Option<AnimalAttackResult> {
    // Simple version without combat context for backward compatibility
    player_attack_animal(manager, animal_id, damage, _weapon_type, &CombatContext::default())
}

/// Result of player attacking an animal
#[derive(Debug)]
pub struct AnimalAttackResult {
    pub hit: bool,
    pub damage_dealt: f32,
    pub killed: bool,
    pub was_alive: bool,
    pub species: super::types::AnimalSpecies,
    pub position: Vec3,
    pub was_stealth: bool,
    pub was_critical: bool,
}

/// Find the closest animal to a position within range
pub fn find_closest_animal(
    manager: &AnimalManager,
    position: Vec3,
    max_range: f32,
) -> Option<(AnimalId, f32)> {
    manager
        .query_radius(position, max_range)
        .into_iter()
        .filter_map(|id| {
            manager.get(id).and_then(|a| {
                if a.is_alive() {
                    Some((id, a.position.distance(position)))
                } else {
                    None
                }
            })
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
}

/// Check if player is being targeted by any nearby predators
pub fn get_threats(manager: &AnimalManager, player_pos: Vec3, range: f32) -> Vec<ThreatInfo> {
    manager
        .query_radius(player_pos, range)
        .into_iter()
        .filter_map(|id| {
            let animal = manager.get(id)?;
            if !animal.is_alive() {
                return None;
            }

            let is_threat = matches!(
                animal.behavior_state,
                BehaviorState::Pursue(_) | BehaviorState::Attack(_)
            );

            if is_threat {
                Some(ThreatInfo {
                    id,
                    species: animal.species,
                    position: animal.position,
                    distance: animal.position.distance(player_pos),
                    danger_level: animal.species.danger_level(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Information about a threatening animal
#[derive(Debug)]
pub struct ThreatInfo {
    pub id: AnimalId,
    pub species: super::types::AnimalSpecies,
    pub position: Vec3,
    pub distance: f32,
    pub danger_level: u8,
}
