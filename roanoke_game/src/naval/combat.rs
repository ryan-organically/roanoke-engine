//! Naval combat system
//!
//! Handles ship-to-ship combat including:
//! - Cannon fire and damage
//! - Boarding actions
//! - Crew combat
//! - Surrender and capture

use serde::{Deserialize, Serialize};
use super::{Ship, GunSide, AmmoType, GunSize, Faction};
use std::f32::consts::PI;

/// Active naval battle state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavalBattle {
    pub id: u64,
    pub participants: Vec<u64>,        // Ship IDs
    pub started_at: f64,               // Game time
    pub engagement_range: f32,
    pub current_phase: BattlePhase,
    pub boarding_in_progress: Option<BoardingCombat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattlePhase {
    Approach,     // Ships closing
    Broadside,    // Exchanging fire
    Chase,        // One ship fleeing
    Boarding,     // Grappled and fighting
    Disengaging,  // Breaking off
    Concluded,    // Battle over
}

impl NavalBattle {
    pub fn new(id: u64, attacker: u64, defender: u64, game_time: f64) -> Self {
        Self {
            id,
            participants: vec![attacker, defender],
            started_at: game_time,
            engagement_range: 500.0,
            current_phase: BattlePhase::Approach,
            boarding_in_progress: None,
        }
    }

    pub fn add_participant(&mut self, ship_id: u64) {
        if !self.participants.contains(&ship_id) {
            self.participants.push(ship_id);
        }
    }
}

/// Result of a cannon shot
#[derive(Debug, Clone)]
pub struct ShotResult {
    pub hit: bool,
    pub hull_damage: u32,
    pub sail_damage: u32,
    pub crew_casualties: u32,
    pub fire_started: bool,
    pub critical_hit: Option<CriticalHit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CriticalHit {
    MagazineExplosion, // Catastrophic
    MastDestroyed,
    HelmDestroyed,
    CaptainKilled,
    GunDestroyed,
    WaterlineHit,      // Severe flooding
}

/// Calculate shot accuracy and results
pub fn calculate_shot(
    shooter: &Ship,
    target: &Ship,
    gun: GunSize,
    ammo: AmmoType,
    range: f32,
) -> ShotResult {
    // Base hit chance
    let base_accuracy = 0.3;

    // Range modifier (closer = more accurate)
    let optimal_range = gun.range() * 0.5;
    let range_mod = if range < optimal_range {
        1.0
    } else {
        (1.0 - (range - optimal_range) / gun.range()).max(0.0)
    };

    // Crew experience modifier
    let crew_mod = 0.5 + shooter.crew_experience * 0.5;

    // Target size modifier (bigger ships easier to hit)
    let size_mod = match target.class {
        super::ShipClass::Canoe | super::ShipClass::Rowboat => 0.5,
        super::ShipClass::Pinnace | super::ShipClass::Shallop => 0.7,
        super::ShipClass::Sloop | super::ShipClass::Brigantine => 0.9,
        super::ShipClass::Merchantman | super::ShipClass::Frigate => 1.0,
        super::ShipClass::Galleon | super::ShipClass::ManOfWar => 1.2,
    };

    // Sea state/movement penalties would go here

    let hit_chance = (base_accuracy * range_mod * crew_mod * size_mod).min(0.9);
    let hit = rand_float() < hit_chance;

    if !hit {
        return ShotResult {
            hit: false,
            hull_damage: 0,
            sail_damage: 0,
            crew_casualties: 0,
            fire_started: false,
            critical_hit: None,
        };
    }

    // Calculate damage
    let base_dmg = gun.base_damage();
    let hull_damage = (base_dmg as f32 * ammo.hull_damage_modifier()) as u32;
    let sail_damage = (base_dmg as f32 * ammo.rigging_damage_modifier() * 0.5) as u32;

    // Crew casualties
    let crew_damage = (base_dmg as f32 * ammo.crew_damage_modifier() * 0.1) as u32;
    let crew_casualties = (crew_damage as f32 * (target.crew_count as f32 / 100.0).min(1.0)) as u32;

    // Fire chance
    let fire_started = rand_float() < ammo.fire_chance();

    // Critical hit chance (5%)
    let critical_hit = if rand_float() < 0.05 {
        Some(random_critical())
    } else {
        None
    };

    ShotResult {
        hit: true,
        hull_damage,
        sail_damage,
        crew_casualties,
        fire_started,
        critical_hit,
    }
}

/// Calculate shot with extracted shooter parameters (avoids borrow issues)
fn calculate_shot_params(
    shooter_crew_exp: f32,
    target: &Ship,
    gun: GunSize,
    ammo: AmmoType,
    range: f32,
) -> ShotResult {
    // Base hit chance
    let base_accuracy = 0.3;

    // Range modifier (closer = more accurate)
    let optimal_range = gun.range() * 0.5;
    let range_mod = if range < optimal_range {
        1.0
    } else {
        (1.0 - (range - optimal_range) / gun.range()).max(0.0)
    };

    // Crew experience modifier
    let crew_mod = 0.5 + shooter_crew_exp * 0.5;

    // Target size modifier (bigger ships easier to hit)
    let size_mod = match target.class {
        super::ShipClass::Canoe | super::ShipClass::Rowboat => 0.5,
        super::ShipClass::Pinnace | super::ShipClass::Shallop => 0.7,
        super::ShipClass::Sloop | super::ShipClass::Brigantine => 0.9,
        super::ShipClass::Merchantman | super::ShipClass::Frigate => 1.0,
        super::ShipClass::Galleon | super::ShipClass::ManOfWar => 1.2,
    };

    let hit_chance = (base_accuracy * range_mod * crew_mod * size_mod).min(0.9);
    let hit = rand_float() < hit_chance;

    if !hit {
        return ShotResult {
            hit: false,
            hull_damage: 0,
            sail_damage: 0,
            crew_casualties: 0,
            fire_started: false,
            critical_hit: None,
        };
    }

    // Calculate damage
    let base_dmg = gun.base_damage();
    let hull_damage = (base_dmg as f32 * ammo.hull_damage_modifier()) as u32;
    let sail_damage = (base_dmg as f32 * ammo.rigging_damage_modifier() * 0.5) as u32;

    // Crew casualties
    let crew_damage = (base_dmg as f32 * ammo.crew_damage_modifier() * 0.1) as u32;
    let crew_casualties = (crew_damage as f32 * (target.crew_count as f32 / 100.0).min(1.0)) as u32;

    // Fire chance
    let fire_started = rand_float() < ammo.fire_chance();

    // Critical hit chance (5%)
    let critical_hit = if rand_float() < 0.05 {
        Some(random_critical())
    } else {
        None
    };

    ShotResult {
        hit: true,
        hull_damage,
        sail_damage,
        crew_casualties,
        fire_started,
        critical_hit,
    }
}

fn random_critical() -> CriticalHit {
    let roll = (rand_float() * 6.0) as u32;
    match roll {
        0 => CriticalHit::MagazineExplosion,
        1 => CriticalHit::MastDestroyed,
        2 => CriticalHit::HelmDestroyed,
        3 => CriticalHit::CaptainKilled,
        4 => CriticalHit::GunDestroyed,
        _ => CriticalHit::WaterlineHit,
    }
}

impl CriticalHit {
    pub fn apply(&self, ship: &mut Ship) {
        match self {
            Self::MagazineExplosion => {
                // Catastrophic - ship likely lost
                ship.hull_hp = ship.hull_hp / 4;
                ship.crew_count = ship.crew_count / 2;
                ship.is_on_fire = true;
                ship.fire_intensity = 0.8;
            }
            Self::MastDestroyed => {
                ship.mast_damage = 1.0;
                ship.sail_hp = ship.sail_hp / 3;
            }
            Self::HelmDestroyed => {
                // Can't steer effectively
                // Would need additional state tracking
            }
            Self::CaptainKilled => {
                ship.crew_morale -= 30.0;
            }
            Self::GunDestroyed => {
                // Remove a random gun
                if !ship.guns.is_empty() {
                    ship.guns.pop();
                }
            }
            Self::WaterlineHit => {
                ship.flooding_rate += 0.15;
            }
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::MagazineExplosion => "Magazine explodes!",
            Self::MastDestroyed => "Mast destroyed!",
            Self::HelmDestroyed => "Helm destroyed!",
            Self::CaptainKilled => "Captain killed!",
            Self::GunDestroyed => "Gun destroyed!",
            Self::WaterlineHit => "Waterline hit!",
        }
    }
}

/// Fire a broadside from one ship to another
pub fn fire_broadside(
    shooter: &mut Ship,
    target: &mut Ship,
    side: GunSide,
    ammo: AmmoType,
) -> Vec<ShotResult> {
    let mut results = Vec::new();

    // Calculate range
    let dx = target.position[0] - shooter.position[0];
    let dz = target.position[1] - shooter.position[1];
    let range = (dx * dx + dz * dz).sqrt();

    // Extract shooter data needed for calculations (to avoid borrow conflict)
    let shooter_crew_exp = shooter.crew_experience;

    // Collect gun indices that are ready to fire
    let ready_guns: Vec<usize> = shooter.guns
        .iter()
        .enumerate()
        .filter(|(_, gun)| gun.side == side && gun.ready_to_fire())
        .map(|(i, _)| i)
        .collect();

    for gun_idx in ready_guns {
        let gun_size = shooter.guns[gun_idx].size;
        let result = calculate_shot_params(shooter_crew_exp, target, gun_size, ammo, range);

        if result.hit {
            target.take_damage(result.hull_damage, ammo);
            target.sail_hp = target.sail_hp.saturating_sub(result.sail_damage);
            target.take_crew_damage(result.crew_casualties);

            if result.fire_started && !target.is_on_fire {
                target.is_on_fire = true;
                target.fire_intensity = 0.2;
            }

            if let Some(crit) = result.critical_hit {
                crit.apply(target);
            }
        }

        shooter.guns[gun_idx].fire();
        results.push(result);
    }

    results
}

/// Check if ships can grapple for boarding
pub fn can_grapple(ship1: &Ship, ship2: &Ship) -> bool {
    let dx = ship2.position[0] - ship1.position[0];
    let dz = ship2.position[1] - ship1.position[1];
    let distance = (dx * dx + dz * dz).sqrt();

    // Must be very close and roughly parallel
    distance < 20.0 && (ship1.current_speed - ship2.current_speed).abs() < 2.0
}

/// Boarding combat state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardingCombat {
    pub attacker_id: u64,
    pub defender_id: u64,
    pub attacker_crew: u32,
    pub defender_crew: u32,
    pub attacker_morale: f32,
    pub defender_morale: f32,
    pub zones: [ZoneControl; 5],
    pub rounds_elapsed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneControl {
    Attacker,
    Defender,
    Contested,
}

impl BoardingCombat {
    pub fn new(attacker: &Ship, defender: &Ship) -> Self {
        Self {
            attacker_id: attacker.id,
            defender_id: defender.id,
            attacker_crew: attacker.crew_count,
            defender_crew: defender.crew_count,
            attacker_morale: attacker.crew_morale,
            defender_morale: defender.crew_morale,
            zones: [
                ZoneControl::Contested, // Gangway
                ZoneControl::Contested, // Main deck
                ZoneControl::Defender,  // Quarterdeck
                ZoneControl::Defender,  // Below decks
                ZoneControl::Defender,  // Captain's cabin
            ],
            rounds_elapsed: 0,
        }
    }

    /// Resolve one round of boarding combat
    pub fn resolve_round(&mut self) -> BoardingRoundResult {
        self.rounds_elapsed += 1;

        // Combat strength based on crew and morale
        let att_strength = self.attacker_crew as f32 * (self.attacker_morale / 100.0);
        let def_strength = self.defender_crew as f32 * (self.defender_morale / 100.0);

        let total = att_strength + def_strength;
        if total < 1.0 {
            return BoardingRoundResult::Stalemate;
        }

        // Casualties proportional to opposing strength
        let att_casualties = (def_strength * 0.1 * rand_float()) as u32;
        let def_casualties = (att_strength * 0.1 * rand_float()) as u32;

        self.attacker_crew = self.attacker_crew.saturating_sub(att_casualties);
        self.defender_crew = self.defender_crew.saturating_sub(def_casualties);

        // Morale damage from casualties
        self.attacker_morale -= att_casualties as f32 * 0.5;
        self.defender_morale -= def_casualties as f32 * 0.5;

        // Zone control shifts based on strength advantage
        let advantage = att_strength / total;

        for zone in &mut self.zones {
            if *zone == ZoneControl::Contested {
                if advantage > 0.6 && rand_float() < 0.3 {
                    *zone = ZoneControl::Attacker;
                } else if advantage < 0.4 && rand_float() < 0.3 {
                    *zone = ZoneControl::Defender;
                }
            }
        }

        // Check for victory conditions
        if self.defender_crew == 0 || self.defender_morale <= 0.0 {
            return BoardingRoundResult::AttackerVictory;
        }
        if self.attacker_crew == 0 || self.attacker_morale <= 0.0 {
            return BoardingRoundResult::DefenderVictory;
        }

        // Check if all zones captured
        if self.zones.iter().all(|z| *z == ZoneControl::Attacker) {
            return BoardingRoundResult::AttackerVictory;
        }
        if self.zones.iter().all(|z| *z == ZoneControl::Defender) {
            return BoardingRoundResult::DefenderVictory;
        }

        BoardingRoundResult::Continuing {
            attacker_casualties: att_casualties,
            defender_casualties: def_casualties,
        }
    }

    pub fn is_concluded(&self) -> bool {
        self.attacker_crew == 0
            || self.defender_crew == 0
            || self.attacker_morale <= 0.0
            || self.defender_morale <= 0.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BoardingRoundResult {
    Continuing {
        attacker_casualties: u32,
        defender_casualties: u32,
    },
    AttackerVictory,
    DefenderVictory,
    Stalemate,
}

/// Check surrender conditions
pub fn should_surrender(ship: &Ship) -> bool {
    // Ships surrender when situation is hopeless
    ship.crew_morale < 15.0
        || ship.crew_count < 5
        || (ship.hull_hp < 30 && ship.is_on_fire)
}

/// Calculate prize value for captured ship
pub fn calculate_prize_value(ship: &Ship) -> u32 {
    let base = ship.class.base_price();
    let condition = ship.hull_hp as f32 / ship.max_hull_hp as f32;
    let cargo_value: u32 = ship.cargo.iter().map(|c| c.value).sum();

    (base as f32 * condition) as u32 + cargo_value
}

/// Combat AI decision making
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatAction {
    Approach,
    MaintainDistance,
    FireBroadside(GunSide),
    AttemptBoarding,
    Flee,
    Surrender,
    RepairAndRegroup,
}

pub fn decide_combat_action(ship: &Ship, enemy: &Ship, battle: &NavalBattle) -> CombatAction {
    // Calculate relative strength
    let our_strength = combat_strength(ship);
    let enemy_strength = combat_strength(enemy);
    let ratio = our_strength / enemy_strength.max(1.0);

    // Check desperate situations
    if should_surrender(ship) {
        return CombatAction::Surrender;
    }

    if ship.hull_hp < 50 || ship.crew_count < ship.class.base_stats().min_crew {
        if ratio < 0.5 {
            return CombatAction::Flee;
        }
        return CombatAction::RepairAndRegroup;
    }

    // Calculate range
    let dx = enemy.position[0] - ship.position[0];
    let dz = enemy.position[1] - ship.position[1];
    let range = (dx * dx + dz * dz).sqrt();

    // Determine optimal engagement
    match battle.current_phase {
        BattlePhase::Approach => {
            if range > 200.0 {
                CombatAction::Approach
            } else {
                // Determine which side to present
                let relative_angle = dz.atan2(dx) - ship.heading;
                if relative_angle.cos() > 0.0 {
                    CombatAction::FireBroadside(GunSide::Starboard)
                } else {
                    CombatAction::FireBroadside(GunSide::Port)
                }
            }
        }
        BattlePhase::Broadside => {
            // Check if boarding is advantageous
            if ratio > 1.2 && range < 30.0 && ship.crew_count > enemy.crew_count {
                CombatAction::AttemptBoarding
            } else if ship.ready_guns() > 0 {
                // Continue firing
                let relative_angle = dz.atan2(dx) - ship.heading;
                if relative_angle.cos() > 0.0 {
                    CombatAction::FireBroadside(GunSide::Starboard)
                } else {
                    CombatAction::FireBroadside(GunSide::Port)
                }
            } else {
                CombatAction::MaintainDistance
            }
        }
        BattlePhase::Chase => {
            if ratio > 0.8 {
                CombatAction::Approach
            } else {
                CombatAction::Flee
            }
        }
        BattlePhase::Boarding => {
            CombatAction::AttemptBoarding
        }
        _ => CombatAction::MaintainDistance,
    }
}

fn combat_strength(ship: &Ship) -> f32 {
    let gun_power: f32 = ship.guns.iter().map(|g| g.size.base_damage() as f32).sum();
    let crew_factor = ship.crew_count as f32 * (ship.crew_morale / 100.0);
    let hull_factor = ship.hull_hp as f32 / ship.max_hull_hp as f32;

    (gun_power + crew_factor * 10.0) * hull_factor
}

fn rand_float() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}
