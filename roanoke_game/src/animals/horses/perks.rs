//! Horse Perk Tree System
//!
//! Five branches of perks that enhance different aspects of horse capabilities:
//! - Bond: Relationship perks affecting loyalty and communication
//! - Endurance: Stamina, health, and recovery
//! - Speed: Movement speed, acceleration, and agility
//! - Combat: Battle readiness and offensive capabilities
//! - Utility: Work abilities and carrying capacity

use super::types::HorseStats;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The five branches of the perk tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerkBranch {
    /// Relationship and communication perks
    Bond,
    /// Stamina and health perks
    Endurance,
    /// Speed and agility perks
    Speed,
    /// Combat and bravery perks
    Combat,
    /// Work and carrying perks
    Utility,
}

impl PerkBranch {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bond => "Bond",
            Self::Endurance => "Endurance",
            Self::Speed => "Speed",
            Self::Combat => "Combat",
            Self::Utility => "Utility",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Bond => "Strengthen the connection between horse and rider",
            Self::Endurance => "Improve stamina, health, and recovery",
            Self::Speed => "Enhance movement speed and agility",
            Self::Combat => "Increase battle effectiveness and bravery",
            Self::Utility => "Boost work capabilities and carrying capacity",
        }
    }

    /// Get color for UI
    pub fn color(&self) -> [f32; 3] {
        match self {
            Self::Bond => [0.8, 0.4, 0.6],      // Pink/magenta
            Self::Endurance => [0.4, 0.7, 0.3], // Green
            Self::Speed => [0.3, 0.5, 0.9],     // Blue
            Self::Combat => [0.9, 0.3, 0.2],    // Red
            Self::Utility => [0.7, 0.6, 0.3],   // Gold/brown
        }
    }

    /// Get all perks in this branch
    pub fn perks(&self) -> Vec<HorsePerk> {
        HorsePerk::all()
            .filter(|p| p.branch() == *self)
            .collect()
    }

    /// Iterator over all branches
    pub fn all() -> impl Iterator<Item = PerkBranch> {
        [Self::Bond, Self::Endurance, Self::Speed, Self::Combat, Self::Utility].into_iter()
    }
}

/// Individual perks that can be unlocked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorsePerk {
    // === Bond Branch (Tier 1-5) ===
    /// Tier 1: Horse comes when whistled
    TrustedCompanion,
    /// Tier 1: Reduced fear from player actions
    CalmPresence,
    /// Tier 2: Horse stays nearby when dismounted
    LoyalFollower,
    /// Tier 2: Better response to voice commands
    DeepUnderstanding,
    /// Tier 3: Horse protects player from threats
    ProtectiveInstinct,
    /// Tier 3: Faster trust/bond gain
    RapidBonding,
    /// Tier 4: Horse can find player from far away
    HomewardBound,
    /// Tier 4: Shared danger sense
    SixthSense,
    /// Tier 5: Ultimate bond - near-telepathic connection
    SoulBond,

    // === Endurance Branch (Tier 1-5) ===
    /// Tier 1: +20% stamina
    ExtendedStamina,
    /// Tier 1: Faster stamina recovery
    QuickRecovery,
    /// Tier 2: +20% health
    Hardiness,
    /// Tier 2: Reduced stamina drain while walking
    EfficientGait,
    /// Tier 3: Stamina recovers while trotting
    SecondWind,
    /// Tier 3: Less damage from falls
    SturdilyBuilt,
    /// Tier 4: +30% stamina and health
    IronConstitution,
    /// Tier 4: Resistance to weather effects
    WeatherHardened,
    /// Tier 5: Legendary endurance
    Tireless,

    // === Speed Branch (Tier 1-5) ===
    /// Tier 1: +15% speed
    SwiftHooves,
    /// Tier 1: Faster acceleration
    QuickStart,
    /// Tier 2: +20% gallop speed
    WindRunner,
    /// Tier 2: Better turning at speed
    NimbleTurn,
    /// Tier 3: Less speed lost on rough terrain
    AllTerrainSpeed,
    /// Tier 3: Brief speed burst ability
    SprintBurst,
    /// Tier 4: Maintains top speed longer
    MarathonRunner,
    /// Tier 4: Superior jumping
    HighJumper,
    /// Tier 5: Legendary speed
    Thunderhooves,

    // === Combat Branch (Tier 1-5) ===
    /// Tier 1: Less fear from combat
    Steadfast,
    /// Tier 1: Rearing attack available
    DefensiveKick,
    /// Tier 2: Charge attack damage
    PowerfulCharge,
    /// Tier 2: Doesn't flee from predators
    PredatorResistance,
    /// Tier 3: Trample damage to enemies
    WarStamp,
    /// Tier 3: Can wear heavy armor
    ArmorBearer,
    /// Tier 4: Coordinated attack with rider
    BattleSynergy,
    /// Tier 4: Fear aura against enemies
    IntimidatingPresence,
    /// Tier 5: Legendary war horse
    Destrier,

    // === Utility Branch (Tier 1-5) ===
    /// Tier 1: +25% carry capacity
    PackMule,
    /// Tier 1: Better plow/cart performance
    StrongBack,
    /// Tier 2: +35% carry capacity
    BeastOfBurden,
    /// Tier 2: Reduced equipment weight
    EfficientLoad,
    /// Tier 3: Can pull heavier wagons
    DraftMaster,
    /// Tier 3: Items take less durability damage
    CarefulCarrier,
    /// Tier 4: Bonus farming efficiency
    FieldWorker,
    /// Tier 4: Auto-collect nearby items
    Scavenger,
    /// Tier 5: Ultimate pack horse
    CaravanLeader,
}

impl HorsePerk {
    /// Get the branch this perk belongs to
    pub fn branch(&self) -> PerkBranch {
        match self {
            Self::TrustedCompanion | Self::CalmPresence | Self::LoyalFollower |
            Self::DeepUnderstanding | Self::ProtectiveInstinct | Self::RapidBonding |
            Self::HomewardBound | Self::SixthSense | Self::SoulBond
                => PerkBranch::Bond,

            Self::ExtendedStamina | Self::QuickRecovery | Self::Hardiness |
            Self::EfficientGait | Self::SecondWind | Self::SturdilyBuilt |
            Self::IronConstitution | Self::WeatherHardened | Self::Tireless
                => PerkBranch::Endurance,

            Self::SwiftHooves | Self::QuickStart | Self::WindRunner |
            Self::NimbleTurn | Self::AllTerrainSpeed | Self::SprintBurst |
            Self::MarathonRunner | Self::HighJumper | Self::Thunderhooves
                => PerkBranch::Speed,

            Self::Steadfast | Self::DefensiveKick | Self::PowerfulCharge |
            Self::PredatorResistance | Self::WarStamp | Self::ArmorBearer |
            Self::BattleSynergy | Self::IntimidatingPresence | Self::Destrier
                => PerkBranch::Combat,

            Self::PackMule | Self::StrongBack | Self::BeastOfBurden |
            Self::EfficientLoad | Self::DraftMaster | Self::CarefulCarrier |
            Self::FieldWorker | Self::Scavenger | Self::CaravanLeader
                => PerkBranch::Utility,
        }
    }

    /// Get the tier (1-5) of this perk
    pub fn tier(&self) -> u8 {
        match self {
            // Tier 1
            Self::TrustedCompanion | Self::CalmPresence |
            Self::ExtendedStamina | Self::QuickRecovery |
            Self::SwiftHooves | Self::QuickStart |
            Self::Steadfast | Self::DefensiveKick |
            Self::PackMule | Self::StrongBack
                => 1,

            // Tier 2
            Self::LoyalFollower | Self::DeepUnderstanding |
            Self::Hardiness | Self::EfficientGait |
            Self::WindRunner | Self::NimbleTurn |
            Self::PowerfulCharge | Self::PredatorResistance |
            Self::BeastOfBurden | Self::EfficientLoad
                => 2,

            // Tier 3
            Self::ProtectiveInstinct | Self::RapidBonding |
            Self::SecondWind | Self::SturdilyBuilt |
            Self::AllTerrainSpeed | Self::SprintBurst |
            Self::WarStamp | Self::ArmorBearer |
            Self::DraftMaster | Self::CarefulCarrier
                => 3,

            // Tier 4
            Self::HomewardBound | Self::SixthSense |
            Self::IronConstitution | Self::WeatherHardened |
            Self::MarathonRunner | Self::HighJumper |
            Self::BattleSynergy | Self::IntimidatingPresence |
            Self::FieldWorker | Self::Scavenger
                => 4,

            // Tier 5
            Self::SoulBond | Self::Tireless | Self::Thunderhooves |
            Self::Destrier | Self::CaravanLeader
                => 5,
        }
    }

    /// Get prerequisites for this perk
    pub fn prerequisites(&self) -> &'static [HorsePerk] {
        match self {
            // Tier 1 - no prerequisites
            Self::TrustedCompanion | Self::CalmPresence |
            Self::ExtendedStamina | Self::QuickRecovery |
            Self::SwiftHooves | Self::QuickStart |
            Self::Steadfast | Self::DefensiveKick |
            Self::PackMule | Self::StrongBack
                => &[],

            // Tier 2
            Self::LoyalFollower => &[Self::TrustedCompanion],
            Self::DeepUnderstanding => &[Self::CalmPresence],
            Self::Hardiness => &[Self::ExtendedStamina],
            Self::EfficientGait => &[Self::QuickRecovery],
            Self::WindRunner => &[Self::SwiftHooves],
            Self::NimbleTurn => &[Self::QuickStart],
            Self::PowerfulCharge => &[Self::Steadfast],
            Self::PredatorResistance => &[Self::DefensiveKick],
            Self::BeastOfBurden => &[Self::PackMule],
            Self::EfficientLoad => &[Self::StrongBack],

            // Tier 3
            Self::ProtectiveInstinct => &[Self::LoyalFollower],
            Self::RapidBonding => &[Self::DeepUnderstanding],
            Self::SecondWind => &[Self::Hardiness, Self::EfficientGait],
            Self::SturdilyBuilt => &[Self::Hardiness],
            Self::AllTerrainSpeed => &[Self::WindRunner],
            Self::SprintBurst => &[Self::WindRunner, Self::NimbleTurn],
            Self::WarStamp => &[Self::PowerfulCharge],
            Self::ArmorBearer => &[Self::PredatorResistance],
            Self::DraftMaster => &[Self::BeastOfBurden],
            Self::CarefulCarrier => &[Self::EfficientLoad],

            // Tier 4
            Self::HomewardBound => &[Self::ProtectiveInstinct],
            Self::SixthSense => &[Self::ProtectiveInstinct, Self::RapidBonding],
            Self::IronConstitution => &[Self::SecondWind],
            Self::WeatherHardened => &[Self::SturdilyBuilt],
            Self::MarathonRunner => &[Self::AllTerrainSpeed],
            Self::HighJumper => &[Self::SprintBurst],
            Self::BattleSynergy => &[Self::WarStamp],
            Self::IntimidatingPresence => &[Self::WarStamp, Self::ArmorBearer],
            Self::FieldWorker => &[Self::DraftMaster],
            Self::Scavenger => &[Self::CarefulCarrier],

            // Tier 5
            Self::SoulBond => &[Self::HomewardBound, Self::SixthSense],
            Self::Tireless => &[Self::IronConstitution, Self::WeatherHardened],
            Self::Thunderhooves => &[Self::MarathonRunner, Self::HighJumper],
            Self::Destrier => &[Self::BattleSynergy, Self::IntimidatingPresence],
            Self::CaravanLeader => &[Self::FieldWorker, Self::Scavenger],
        }
    }

    /// Get perk point cost
    pub fn cost(&self) -> u8 {
        self.tier()
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::TrustedCompanion => "Trusted Companion",
            Self::CalmPresence => "Calm Presence",
            Self::LoyalFollower => "Loyal Follower",
            Self::DeepUnderstanding => "Deep Understanding",
            Self::ProtectiveInstinct => "Protective Instinct",
            Self::RapidBonding => "Rapid Bonding",
            Self::HomewardBound => "Homeward Bound",
            Self::SixthSense => "Sixth Sense",
            Self::SoulBond => "Soul Bond",

            Self::ExtendedStamina => "Extended Stamina",
            Self::QuickRecovery => "Quick Recovery",
            Self::Hardiness => "Hardiness",
            Self::EfficientGait => "Efficient Gait",
            Self::SecondWind => "Second Wind",
            Self::SturdilyBuilt => "Sturdily Built",
            Self::IronConstitution => "Iron Constitution",
            Self::WeatherHardened => "Weather Hardened",
            Self::Tireless => "Tireless",

            Self::SwiftHooves => "Swift Hooves",
            Self::QuickStart => "Quick Start",
            Self::WindRunner => "Wind Runner",
            Self::NimbleTurn => "Nimble Turn",
            Self::AllTerrainSpeed => "All-Terrain Speed",
            Self::SprintBurst => "Sprint Burst",
            Self::MarathonRunner => "Marathon Runner",
            Self::HighJumper => "High Jumper",
            Self::Thunderhooves => "Thunderhooves",

            Self::Steadfast => "Steadfast",
            Self::DefensiveKick => "Defensive Kick",
            Self::PowerfulCharge => "Powerful Charge",
            Self::PredatorResistance => "Predator Resistance",
            Self::WarStamp => "War Stamp",
            Self::ArmorBearer => "Armor Bearer",
            Self::BattleSynergy => "Battle Synergy",
            Self::IntimidatingPresence => "Intimidating Presence",
            Self::Destrier => "Destrier",

            Self::PackMule => "Pack Mule",
            Self::StrongBack => "Strong Back",
            Self::BeastOfBurden => "Beast of Burden",
            Self::EfficientLoad => "Efficient Load",
            Self::DraftMaster => "Draft Master",
            Self::CarefulCarrier => "Careful Carrier",
            Self::FieldWorker => "Field Worker",
            Self::Scavenger => "Scavenger",
            Self::CaravanLeader => "Caravan Leader",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::TrustedCompanion => "Horse comes when whistled from greater distance",
            Self::CalmPresence => "Horse is less easily spooked by player actions",
            Self::LoyalFollower => "Horse stays close when dismounted",
            Self::DeepUnderstanding => "Better response to voice commands",
            Self::ProtectiveInstinct => "Horse will defend you from attackers",
            Self::RapidBonding => "Trust and bond increase 50% faster",
            Self::HomewardBound => "Horse can find its way back to you from anywhere",
            Self::SixthSense => "Shared awareness of nearby dangers",
            Self::SoulBond => "Perfect synchronization with your mount",

            Self::ExtendedStamina => "+20% maximum stamina",
            Self::QuickRecovery => "Stamina recovers 30% faster",
            Self::Hardiness => "+20% maximum health",
            Self::EfficientGait => "Reduced stamina drain at lower speeds",
            Self::SecondWind => "Stamina slowly recovers even while trotting",
            Self::SturdilyBuilt => "50% less damage from falls",
            Self::IronConstitution => "+30% stamina and health",
            Self::WeatherHardened => "Immune to weather-based debuffs",
            Self::Tireless => "Stamina drains 50% slower at all gaits",

            Self::SwiftHooves => "+15% base speed",
            Self::QuickStart => "50% faster acceleration",
            Self::WindRunner => "+20% gallop speed",
            Self::NimbleTurn => "Tighter turns at high speed",
            Self::AllTerrainSpeed => "Maintain speed on rough terrain",
            Self::SprintBurst => "Activate a brief 40% speed boost",
            Self::MarathonRunner => "Maintain top speed 50% longer",
            Self::HighJumper => "Jump 30% higher and farther",
            Self::Thunderhooves => "+35% speed, fastest horse in the land",

            Self::Steadfast => "Horse rarely panics in combat",
            Self::DefensiveKick => "Rear and kick attacking enemies",
            Self::PowerfulCharge => "Deal damage by charging into enemies",
            Self::PredatorResistance => "Won't flee from predator animals",
            Self::WarStamp => "Trample damage to knocked-down enemies",
            Self::ArmorBearer => "Can equip heavy barding armor",
            Self::BattleSynergy => "+30% damage when rider is attacking",
            Self::IntimidatingPresence => "Nearby enemies are frightened",
            Self::Destrier => "Ultimate war horse abilities",

            Self::PackMule => "+25% carry capacity",
            Self::StrongBack => "+20% plow/cart pulling power",
            Self::BeastOfBurden => "+35% carry capacity",
            Self::EfficientLoad => "Equipment weight reduced by 25%",
            Self::DraftMaster => "Pull the heaviest wagons",
            Self::CarefulCarrier => "Carried items take 50% less damage",
            Self::FieldWorker => "Farm work is 30% more efficient",
            Self::Scavenger => "Auto-collect items within 5 meters",
            Self::CaravanLeader => "Can lead pack trains, +50% capacity",
        }
    }

    /// Get stat modifiers for this perk
    pub fn stat_modifiers(&self) -> PerkStatModifiers {
        match self {
            Self::ExtendedStamina => PerkStatModifiers { stamina_mult: 1.2, ..Default::default() },
            Self::Hardiness => PerkStatModifiers { health_mult: 1.2, ..Default::default() },
            Self::IronConstitution => PerkStatModifiers { stamina_mult: 1.3, health_mult: 1.3, ..Default::default() },
            Self::SwiftHooves => PerkStatModifiers { speed_mult: 1.15, ..Default::default() },
            Self::WindRunner => PerkStatModifiers { speed_mult: 1.2, ..Default::default() },
            Self::Thunderhooves => PerkStatModifiers { speed_mult: 1.35, ..Default::default() },
            Self::PackMule => PerkStatModifiers { carry_mult: 1.25, ..Default::default() },
            Self::BeastOfBurden => PerkStatModifiers { carry_mult: 1.35, ..Default::default() },
            Self::CaravanLeader => PerkStatModifiers { carry_mult: 1.5, ..Default::default() },
            Self::StrongBack => PerkStatModifiers { strength_mult: 1.2, ..Default::default() },
            Self::DraftMaster => PerkStatModifiers { strength_mult: 1.4, ..Default::default() },
            Self::QuickStart => PerkStatModifiers { accel_mult: 1.5, ..Default::default() },
            Self::HighJumper => PerkStatModifiers { agility_mult: 1.3, ..Default::default() },
            _ => PerkStatModifiers::default(),
        }
    }

    /// Iterator over all perks
    pub fn all() -> impl Iterator<Item = HorsePerk> {
        [
            // Bond
            Self::TrustedCompanion, Self::CalmPresence, Self::LoyalFollower,
            Self::DeepUnderstanding, Self::ProtectiveInstinct, Self::RapidBonding,
            Self::HomewardBound, Self::SixthSense, Self::SoulBond,
            // Endurance
            Self::ExtendedStamina, Self::QuickRecovery, Self::Hardiness,
            Self::EfficientGait, Self::SecondWind, Self::SturdilyBuilt,
            Self::IronConstitution, Self::WeatherHardened, Self::Tireless,
            // Speed
            Self::SwiftHooves, Self::QuickStart, Self::WindRunner,
            Self::NimbleTurn, Self::AllTerrainSpeed, Self::SprintBurst,
            Self::MarathonRunner, Self::HighJumper, Self::Thunderhooves,
            // Combat
            Self::Steadfast, Self::DefensiveKick, Self::PowerfulCharge,
            Self::PredatorResistance, Self::WarStamp, Self::ArmorBearer,
            Self::BattleSynergy, Self::IntimidatingPresence, Self::Destrier,
            // Utility
            Self::PackMule, Self::StrongBack, Self::BeastOfBurden,
            Self::EfficientLoad, Self::DraftMaster, Self::CarefulCarrier,
            Self::FieldWorker, Self::Scavenger, Self::CaravanLeader,
        ].into_iter()
    }
}

/// Stat modifiers from perks
#[derive(Debug, Clone, Copy, Default)]
pub struct PerkStatModifiers {
    pub health_mult: f32,
    pub stamina_mult: f32,
    pub speed_mult: f32,
    pub strength_mult: f32,
    pub agility_mult: f32,
    pub carry_mult: f32,
    pub accel_mult: f32,
}

impl PerkStatModifiers {
    pub fn combine(&self, other: &PerkStatModifiers) -> PerkStatModifiers {
        PerkStatModifiers {
            health_mult: self.health_mult.max(1.0) * other.health_mult.max(1.0),
            stamina_mult: self.stamina_mult.max(1.0) * other.stamina_mult.max(1.0),
            speed_mult: self.speed_mult.max(1.0) * other.speed_mult.max(1.0),
            strength_mult: self.strength_mult.max(1.0) * other.strength_mult.max(1.0),
            agility_mult: self.agility_mult.max(1.0) * other.agility_mult.max(1.0),
            carry_mult: self.carry_mult.max(1.0) * other.carry_mult.max(1.0),
            accel_mult: self.accel_mult.max(1.0) * other.accel_mult.max(1.0),
        }
    }
}

/// The complete perk tree for a horse
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HorsePerkTree {
    /// Unlocked perks
    unlocked: HashSet<HorsePerk>,
    /// Points invested per branch
    points_per_branch: [u8; 5],
}

impl HorsePerkTree {
    pub fn new() -> Self {
        Self {
            unlocked: HashSet::new(),
            points_per_branch: [0; 5],
        }
    }

    /// Check if a perk is unlocked
    pub fn has_perk(&self, perk: HorsePerk) -> bool {
        self.unlocked.contains(&perk)
    }

    /// Check if a perk can be unlocked
    pub fn can_unlock(&self, perk: HorsePerk, available_points: u8) -> Result<(), &'static str> {
        if self.has_perk(perk) {
            return Err("Perk already unlocked");
        }

        if available_points < perk.cost() {
            return Err("Not enough perk points");
        }

        // Check prerequisites
        for prereq in perk.prerequisites() {
            if !self.has_perk(*prereq) {
                return Err("Prerequisites not met");
            }
        }

        Ok(())
    }

    /// Unlock a perk
    pub fn unlock(&mut self, perk: HorsePerk) -> Result<u8, &'static str> {
        if self.has_perk(perk) {
            return Err("Perk already unlocked");
        }

        // Check prerequisites
        for prereq in perk.prerequisites() {
            if !self.has_perk(*prereq) {
                return Err("Prerequisites not met");
            }
        }

        self.unlocked.insert(perk);

        // Track points per branch
        let branch_idx = match perk.branch() {
            PerkBranch::Bond => 0,
            PerkBranch::Endurance => 1,
            PerkBranch::Speed => 2,
            PerkBranch::Combat => 3,
            PerkBranch::Utility => 4,
        };
        self.points_per_branch[branch_idx] += perk.cost();

        Ok(perk.cost())
    }

    /// Get total stat modifiers from all unlocked perks
    pub fn total_stat_modifiers(&self) -> PerkStatModifiers {
        let mut total = PerkStatModifiers {
            health_mult: 1.0,
            stamina_mult: 1.0,
            speed_mult: 1.0,
            strength_mult: 1.0,
            agility_mult: 1.0,
            carry_mult: 1.0,
            accel_mult: 1.0,
        };

        for perk in &self.unlocked {
            let mods = perk.stat_modifiers();
            total = total.combine(&mods);
        }

        total
    }

    /// Apply stat bonuses to base stats
    pub fn apply_stat_bonuses(&self, mut stats: HorseStats) -> HorseStats {
        let mods = self.total_stat_modifiers();

        if mods.health_mult > 1.0 {
            stats.health *= mods.health_mult;
        }
        if mods.stamina_mult > 1.0 {
            stats.stamina *= mods.stamina_mult;
        }
        if mods.speed_mult > 1.0 {
            stats.speed *= mods.speed_mult;
        }
        if mods.strength_mult > 1.0 {
            stats.strength *= mods.strength_mult;
        }
        if mods.agility_mult > 1.0 {
            stats.agility *= mods.agility_mult;
        }
        if mods.carry_mult > 1.0 {
            stats.carry_capacity *= mods.carry_mult;
        }
        if mods.accel_mult > 1.0 {
            stats.acceleration *= mods.accel_mult;
        }

        stats
    }

    /// Get points invested in a branch
    pub fn branch_points(&self, branch: PerkBranch) -> u8 {
        let idx = match branch {
            PerkBranch::Bond => 0,
            PerkBranch::Endurance => 1,
            PerkBranch::Speed => 2,
            PerkBranch::Combat => 3,
            PerkBranch::Utility => 4,
        };
        self.points_per_branch[idx]
    }

    /// Get total points invested
    pub fn total_points_invested(&self) -> u8 {
        self.points_per_branch.iter().sum()
    }

    /// Get all unlocked perks
    pub fn unlocked_perks(&self) -> impl Iterator<Item = &HorsePerk> {
        self.unlocked.iter()
    }

    /// Get unlocked perks count
    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }

    /// Get perks in a branch with unlock status
    pub fn branch_status(&self, branch: PerkBranch) -> Vec<(HorsePerk, bool)> {
        branch.perks()
            .into_iter()
            .map(|p| (p, self.has_perk(p)))
            .collect()
    }
}
