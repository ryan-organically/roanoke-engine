//! Combat system - damage processing between animals and player

use super::behavior::{AttackState, BehaviorState};
use super::entity::{Animal, AnimalId, DamageSource};
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

/// Player attacks an animal - returns true if killed
pub fn player_attack_animal(
    manager: &mut AnimalManager,
    animal_id: AnimalId,
    damage: f32,
    _weapon_type: Option<&str>, // For weakness checking later
) -> Option<AnimalAttackResult> {
    let animal = manager.get(animal_id)?;
    let species = animal.species;
    let was_alive = animal.is_alive();
    let pos = animal.position;

    // TODO: Check weapon vs weakness for bonus damage
    let final_damage = damage;

    // Apply damage
    let killed = manager.damage_animal(animal_id, final_damage);

    Some(AnimalAttackResult {
        hit: true,
        damage_dealt: final_damage,
        killed,
        was_alive,
        species,
        position: pos,
    })
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
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
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
